//! Port of `bot/knowledge/maths.py`: maths and numbers. Two halves, as in Python: the ENTRIES
//! (fixed facts, data in the bundle) with their two guards and two replies, and `solve(text,
//! understood, memory)`, the computed answers for lines that carry their own sums ("15% of 80",
//! "how many days until christmas", "split 84 between 6"). `solve` is called by the answers
//! module, the memory store and the fallback chain, so every helper is public under its Python
//! name.
//!
//! Python's number semantics are reproduced where they show: ints and floats print differently,
//! `round` is half-to-even, `f"{x:.2f}"` is correctly rounded (Rust's fixed formatting is the
//! same), `Fraction` and `Decimal(..).quantize(.., ROUND_HALF_UP)` are exact, and the arithmetic
//! card evaluates the line's expression with Python's int/float rules (big ints included).
//! The clock is `datetime.date.today()`, pinned to 2026-11-17 unless RSBOT_TODAY=YYYY-MM-DD.

#![allow(non_snake_case)]

use crate::data;
use crate::data::{FnRef, Value};
use crate::knowledge::d2dup_maths as dup;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// =============================================================================================
// Python shims: regex cache, bundle data, numbers, dates

/// A regex compiled once per (pattern, flags), as Python's `re` module caches them.
pub fn rx(pattern: &str, flags: u32) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<(String, u32), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(r) = cache.get(&(pattern.to_string(), flags)) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(pattern, flags)));
    cache.insert((pattern.to_string(), flags), r);
    r
}

/// `re.search(p, s)` with no flags.
fn search<'t>(pattern: &str, s: &'t str) -> Option<Match<'t>> {
    rx(pattern, 0).search(s)
}

fn has(pattern: &str, s: &str) -> bool {
    rx(pattern, 0).is_match(s)
}

fn fullmatch<'t>(pattern: &str, s: &'t str) -> Option<Match<'t>> {
    rx(pattern, 0).fullmatch(s)
}

fn sub(pattern: &str, repl: &str, s: &str) -> String {
    rx(pattern, 0).sub(repl, s)
}

const I: u32 = crate::py::re::I;

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.maths.{name}"))
}

fn gre(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.maths.{name}"))
}

fn gstr(name: &str) -> &'static str {
    g(name).str()
}

fn gstrs(name: &str) -> Vec<String> {
    g(name).strs()
}

/// A `{str: int}` dict global, in insertion order.
fn gint_pairs(name: &str) -> Vec<(String, i64)> {
    g(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
}

fn lookup<'a, V: Copy>(pairs: &'a [(String, V)], key: &str) -> Option<V> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| *v)
}

/// `any(word in text for word in words)`.
fn any_in<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    words.iter().any(|w| text.contains(w.as_ref()))
}

/// `" ".join(s.split())`.
fn squash(s: &str) -> String {
    text::squash(s)
}

/// Python `str.isdigit()` for the strings the regexes hand over (ASCII digits).
fn isdigit(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// Python `float(s)` for a figure the line wrote (digits and points). Python raises ValueError
/// on "1.2.3" or ""; nothing in maths.py catches it, so neither does this.
pub fn py_float(s: &str) -> f64 {
    let t = s.trim();
    let ok = !t.is_empty() && t.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '+');
    match (ok, t.parse::<f64>()) {
        (true, Ok(v)) => v,
        _ => panic!("ValueError: could not convert string to float: {s:?}"),
    }
}

/// Python `int(s)` for a run of digits (an optional sign).
pub fn py_int(s: &str) -> i128 {
    s.trim().parse::<i128>().unwrap_or_else(|_| panic!("ValueError: invalid literal for int(): {s:?}"))
}

/// `f"{x:.{n}f}"` (Rust's fixed formatting is correctly rounded, ties to even, like Python's).
pub fn fixed(x: f64, places: usize) -> String {
    format!("{:.*}", places, x)
}

/// Python `round(x)` on a float: half to even, as an integral float.
pub fn round0(x: f64) -> f64 {
    x.round_ties_even()
}

/// Python `round(x, n)` on a float: the correctly rounded decimal, read back.
pub fn round_n(x: f64, n: usize) -> f64 {
    if !x.is_finite() {
        return x;
    }
    fixed(x, n).parse::<f64>().unwrap()
}

/// Python `int(x)` on a float: towards zero.
fn trunc(x: f64) -> f64 {
    if !x.is_finite() {
        panic!("OverflowError: cannot convert float infinity to integer");
    }
    x.trunc()
}

/// Commas into a run of digits with an optional sign: "-1234567" -> "-1,234,567".
pub fn commas(digits: &str) -> String {
    let (sign, body) = match digits.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", digits),
    };
    let n = body.len();
    let mut out = String::with_capacity(n + n / 3 + 1);
    for (i, c) in body.chars().enumerate() {
        if i > 0 && (n - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{sign}{out}")
}

/// `str(int(x))` for an integral float: exact digits, however big, and no "-0".
pub fn int_str(x: f64) -> String {
    fixed(x + 0.0, 0)
}

/// `f"{int(x):,}"` for an integral float (exact digits, however big).
pub fn int_commas(x: f64) -> String {
    commas(&int_str(x))
}

/// `f"{x:,.2f}"`.
pub fn commas_fixed(x: f64, places: usize) -> String {
    let s = fixed(x, places);
    match s.split_once('.') {
        Some((whole, frac)) => format!("{}.{}", commas(whole), frac),
        None => commas(&s),
    }
}

/// `f"{x:g}"`: six significant figures, trailing zeros dropped, exponent outside 1e-4..1e6.
pub fn fmt_g(x: f64) -> String {
    if x == 0.0 {
        return if x.is_sign_negative() { "-0".into() } else { "0".into() };
    }
    if !x.is_finite() {
        return if x.is_nan() {
            "nan".into()
        } else if x > 0.0 {
            "inf".into()
        } else {
            "-inf".into()
        };
    }
    let sci = format!("{:.5e}", x);
    let (mant, exp) = sci.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    if (-4..6).contains(&exp) {
        let places = (5 - exp).max(0) as usize;
        let s = fixed(x, places);
        if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s }
    } else {
        let m = if mant.contains('.') {
            mant.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            mant.to_string()
        };
        format!("{}e{}{:02}", m, if exp < 0 { '-' } else { '+' }, exp.abs())
    }
}

/// Python `str.capitalize()`.
fn capitalize(s: &str) -> String {
    text::capitalize(s)
}

/// `s[0].upper() + s[1:]` (raises on "" in Python).
fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => panic!("IndexError: string index out of range"),
    }
}

/// `s[:1].upper() + s[1:]` (no error on "").
fn upper_first_safe(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Python `s.strip(chars)`.
fn strip_chars<'a>(s: &'a str, chars: &str) -> &'a str {
    s.trim_matches(|c| chars.contains(c))
}

/// Python's `text[a:b]` with code-point indices taken from byte offsets shifted by `back` chars.
fn chars_before(s: &str, end_byte: usize, back: usize) -> &str {
    let start_char = text::char_index(s, end_byte).saturating_sub(back);
    &s[text::byte_at(s, start_char)..end_byte]
}

// ---------------------------------------------------------------------------------------------
// Python int: arbitrary precision, for the arithmetic card (a line's own sum can be big).

/// A Python int of any size: sign and magnitude (base 2^32, least significant first).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigInt {
    neg: bool,
    mag: Vec<u32>,
}

impl BigInt {
    pub fn zero() -> BigInt {
        BigInt { neg: false, mag: Vec::new() }
    }
    pub fn from_i128(v: i128) -> BigInt {
        let neg = v < 0;
        let mut u = v.unsigned_abs();
        let mut mag = Vec::new();
        while u > 0 {
            mag.push(u as u32);
            u >>= 32;
        }
        BigInt { neg, mag }
    }
    fn trim(mut self) -> BigInt {
        while self.mag.last() == Some(&0) {
            self.mag.pop();
        }
        if self.mag.is_empty() {
            self.neg = false;
        }
        self
    }
    pub fn is_zero(&self) -> bool {
        self.mag.is_empty()
    }
    pub fn is_neg(&self) -> bool {
        self.neg
    }
    /// From a run of ASCII digits.
    pub fn parse(digits: &str) -> Option<BigInt> {
        let (neg, body) = match digits.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, digits.strip_prefix('+').unwrap_or(digits)),
        };
        if body.is_empty() || !body.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let mut mag: Vec<u32> = Vec::new();
        for chunk in body.as_bytes().chunks(9) {
            let n: u32 = std::str::from_utf8(chunk).unwrap().parse().unwrap();
            let scale = 10u64.pow(chunk.len() as u32);
            let mut carry = n as u64;
            for limb in mag.iter_mut() {
                let v = *limb as u64 * scale + carry;
                *limb = v as u32;
                carry = v >> 32;
            }
            while carry > 0 {
                mag.push(carry as u32);
                carry >>= 32;
            }
        }
        Some(BigInt { neg, mag }.trim())
    }
    pub fn to_i128(&self) -> Option<i128> {
        if self.mag.len() > 4 {
            return None;
        }
        let mut u: u128 = 0;
        for (i, l) in self.mag.iter().enumerate() {
            u |= (*l as u128) << (32 * i);
        }
        if u > i128::MAX as u128 {
            return None;
        }
        Some(if self.neg { -(u as i128) } else { u as i128 })
    }
    fn cmp_mag(a: &[u32], b: &[u32]) -> Ordering {
        if a.len() != b.len() {
            return a.len().cmp(&b.len());
        }
        for i in (0..a.len()).rev() {
            if a[i] != b[i] {
                return a[i].cmp(&b[i]);
            }
        }
        Ordering::Equal
    }
    fn add_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry = 0u64;
        for i in 0..a.len().max(b.len()) {
            let v = *a.get(i).unwrap_or(&0) as u64 + *b.get(i).unwrap_or(&0) as u64 + carry;
            out.push(v as u32);
            carry = v >> 32;
        }
        if carry > 0 {
            out.push(carry as u32);
        }
        out
    }
    /// a - b for |a| >= |b|.
    fn sub_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0i64;
        for i in 0..a.len() {
            let mut v = a[i] as i64 - *b.get(i).unwrap_or(&0) as i64 - borrow;
            borrow = if v < 0 {
                v += 1 << 32;
                1
            } else {
                0
            };
            out.push(v as u32);
        }
        out
    }
    pub fn neg(&self) -> BigInt {
        BigInt { neg: !self.neg, mag: self.mag.clone() }.trim()
    }
    pub fn abs(&self) -> BigInt {
        BigInt { neg: false, mag: self.mag.clone() }
    }
    pub fn add(&self, o: &BigInt) -> BigInt {
        if self.neg == o.neg {
            return BigInt { neg: self.neg, mag: Self::add_mag(&self.mag, &o.mag) }.trim();
        }
        match Self::cmp_mag(&self.mag, &o.mag) {
            Ordering::Less => BigInt { neg: o.neg, mag: Self::sub_mag(&o.mag, &self.mag) }.trim(),
            _ => BigInt { neg: self.neg, mag: Self::sub_mag(&self.mag, &o.mag) }.trim(),
        }
    }
    pub fn sub(&self, o: &BigInt) -> BigInt {
        self.add(&o.neg())
    }
    pub fn mul(&self, o: &BigInt) -> BigInt {
        if self.is_zero() || o.is_zero() {
            return BigInt::zero();
        }
        let mut out = vec![0u32; self.mag.len() + o.mag.len()];
        for (i, &a) in self.mag.iter().enumerate() {
            let mut carry = 0u64;
            for (j, &b) in o.mag.iter().enumerate() {
                let v = out[i + j] as u64 + a as u64 * b as u64 + carry;
                out[i + j] = v as u32;
                carry = v >> 32;
            }
            let mut k = i + o.mag.len();
            while carry > 0 {
                let v = out[k] as u64 + carry;
                out[k] = v as u32;
                carry = v >> 32;
                k += 1;
            }
        }
        BigInt { neg: self.neg != o.neg, mag: out }.trim()
    }
    pub fn pow(&self, mut e: u32) -> BigInt {
        let mut base = self.clone();
        let mut acc = BigInt::from_i128(1);
        while e > 0 {
            if e & 1 == 1 {
                acc = acc.mul(&base);
            }
            e >>= 1;
            if e > 0 {
                base = base.mul(&base);
            }
        }
        acc
    }
    pub fn bit_length(&self) -> u64 {
        match self.mag.last() {
            None => 0,
            Some(&top) => (self.mag.len() as u64 - 1) * 32 + (32 - top.leading_zeros() as u64),
        }
    }
    fn bit(&self, i: u64) -> bool {
        let limb = (i / 32) as usize;
        limb < self.mag.len() && (self.mag[limb] >> (i % 32)) & 1 == 1
    }
    pub fn shl(&self, n: u64) -> BigInt {
        if self.is_zero() {
            return self.clone();
        }
        let limbs = (n / 32) as usize;
        let bits = (n % 32) as u32;
        let mut out = vec![0u32; limbs];
        let mut carry = 0u32;
        for &l in &self.mag {
            if bits == 0 {
                out.push(l);
            } else {
                out.push((l << bits) | carry);
                carry = l >> (32 - bits);
            }
        }
        if carry > 0 {
            out.push(carry);
        }
        BigInt { neg: self.neg, mag: out }.trim()
    }
    /// Magnitude shifted right (the sign kept), and whether any 1 bits fell off.
    pub fn shr_mag(&self, n: u64) -> (BigInt, bool) {
        let lost = (0..n.min(self.bit_length())).any(|i| self.bit(i));
        let limbs = (n / 32) as usize;
        if limbs >= self.mag.len() {
            return (BigInt::zero(), lost);
        }
        let bits = (n % 32) as u32;
        let mut out = Vec::with_capacity(self.mag.len() - limbs);
        for i in limbs..self.mag.len() {
            let lo = self.mag[i] >> bits;
            let hi = if bits > 0 { self.mag.get(i + 1).map(|h| h << (32 - bits)).unwrap_or(0) } else { 0 };
            out.push(lo | hi);
        }
        (BigInt { neg: self.neg, mag: out }.trim(), lost)
    }
    /// Magnitude division: (|a| / |b|, |a| % |b|), shift-and-subtract.
    fn divmod_mag(a: &BigInt, b: &BigInt) -> (BigInt, BigInt) {
        assert!(!b.is_zero(), "ZeroDivisionError");
        let (a, b) = (a.abs(), b.abs());
        if Self::cmp_mag(&a.mag, &b.mag) == Ordering::Less {
            return (BigInt::zero(), a);
        }
        if b.mag.len() == 1 {
            let d = b.mag[0] as u64;
            let mut q = vec![0u32; a.mag.len()];
            let mut r = 0u64;
            for i in (0..a.mag.len()).rev() {
                let v = (r << 32) | a.mag[i] as u64;
                q[i] = (v / d) as u32;
                r = v % d;
            }
            return (BigInt { neg: false, mag: q }.trim(), BigInt::from_i128(r as i128));
        }
        let mut q = BigInt::zero();
        let mut r = BigInt::zero();
        for i in (0..a.bit_length()).rev() {
            r = r.shl(1);
            if a.bit(i) {
                r = r.add(&BigInt::from_i128(1));
            }
            if Self::cmp_mag(&r.mag, &b.mag) != Ordering::Less {
                r = BigInt { neg: false, mag: Self::sub_mag(&r.mag, &b.mag) }.trim();
                let limb = (i / 32) as usize;
                if q.mag.len() <= limb {
                    q.mag.resize(limb + 1, 0);
                }
                q.mag[limb] |= 1 << (i % 32);
            }
        }
        (q.trim(), r)
    }
    /// Python `divmod(a, b)`: floor division.
    pub fn divmod(&self, b: &BigInt) -> (BigInt, BigInt) {
        let (q, r) = Self::divmod_mag(self, b);
        let mut q = BigInt { neg: self.neg != b.neg, mag: q.mag }.trim();
        let mut r = BigInt { neg: self.neg, mag: r.mag }.trim();
        if !r.is_zero() && (r.neg != b.neg) {
            q = q.sub(&BigInt::from_i128(1));
            r = r.add(b);
        }
        (q, r)
    }
    pub fn gcd(&self, o: &BigInt) -> BigInt {
        let (mut a, mut b) = (self.abs(), o.abs());
        while !b.is_zero() {
            let (_, r) = Self::divmod_mag(&a, &b);
            a = b;
            b = r;
        }
        a
    }
    pub fn cmp(&self, o: &BigInt) -> Ordering {
        match (self.neg, o.neg) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => Self::cmp_mag(&self.mag, &o.mag),
            (true, true) => Self::cmp_mag(&o.mag, &self.mag),
        }
    }
    /// Decimal digits, with a "-" for negatives.
    pub fn to_decimal(&self) -> String {
        if self.is_zero() {
            return "0".into();
        }
        let mut parts = Vec::new();
        let mut cur = self.abs();
        let billion = BigInt::from_i128(1_000_000_000);
        while !cur.is_zero() {
            let (q, r) = Self::divmod_mag(&cur, &billion);
            parts.push(r.to_i128().unwrap() as u32);
            cur = q;
        }
        let mut s = String::new();
        if self.neg {
            s.push('-');
        }
        s.push_str(&parts.last().unwrap().to_string());
        for p in parts.iter().rev().skip(1) {
            s.push_str(&format!("{p:09}"));
        }
        s
    }
    /// The integral float as a Python int (exact).
    pub fn from_f64(x: f64) -> BigInt {
        assert!(x.is_finite(), "OverflowError: cannot convert float infinity to integer");
        let x = x.trunc();
        if x == 0.0 {
            return BigInt::zero();
        }
        let bits = x.abs().to_bits();
        let exp = ((bits >> 52) & 0x7ff) as i64;
        let frac = bits & ((1u64 << 52) - 1);
        let (m, e) = if exp == 0 { (frac, -1074) } else { (frac | (1u64 << 52), exp - 1075) };
        let v = BigInt::from_i128(m as i128);
        let v = if e >= 0 { v.shl(e as u64) } else { v.shr_mag((-e) as u64).0 };
        if x < 0.0 { v.neg() } else { v }
    }
    /// Python `float(int)`: correctly rounded, OverflowError when too big.
    pub fn to_f64(&self) -> Result<f64, PyErr> {
        let n = self.bit_length();
        if n <= 53 {
            let v = self.to_i128().unwrap() as f64;
            return Ok(v);
        }
        // keep 54 bits + sticky, round half to even to 53
        let shift = n - 54;
        let (top, lost) = self.shr_mag(shift);
        let mut t = top.abs().to_i128().unwrap() as u64; // 54 bits
        let half = t & 1;
        t >>= 1;
        if half == 1 && (lost || t & 1 == 1) {
            t += 1;
        }
        let e = shift as i32 + 1;
        let v = (t as f64) * 2f64.powi(0); // exact (<= 2^53)
        let r = ldexp(v, e);
        if r.is_infinite() {
            return Err(PyErr::Overflow);
        }
        Ok(if self.neg { -r } else { r })
    }
}

/// `ldexp(x, e)` without intermediate overflow for the exponents used here.
fn ldexp(x: f64, e: i32) -> f64 {
    let mut v = x;
    let mut e = e;
    while e > 1000 {
        v *= 2f64.powi(1000);
        e -= 1000;
    }
    while e < -1000 {
        v *= 2f64.powi(-1000);
        e += 1000;
    }
    v * 2f64.powi(e)
}

/// A Python exception the arithmetic card catches (or a complex result it rejects).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PyErr {
    Value,
    ZeroDivision,
    Overflow,
    Syntax,
}

/// Python `a / b` on two ints: correctly rounded (CPython's long_true_divide).
pub fn int_truediv(a: &BigInt, b: &BigInt) -> Result<f64, PyErr> {
    if b.is_zero() {
        return Err(PyErr::ZeroDivision);
    }
    if a.is_zero() {
        return Ok(if a.neg != b.neg { -0.0 } else { 0.0 });
    }
    if a.bit_length() <= 53 && b.bit_length() <= 53 {
        let x = a.to_i128().unwrap() as f64;
        let y = b.to_i128().unwrap() as f64;
        return Ok(x / y);
    }
    const MANT: i64 = 53;
    const MIN_EXP: i64 = -1021;
    const MAX_EXP: i64 = 1024;
    let negate = a.neg != b.neg;
    let (a, b) = (a.abs(), b.abs());
    let diff = a.bit_length() as i64 - b.bit_length() as i64;
    if diff > MAX_EXP {
        return Err(PyErr::Overflow);
    }
    if diff < MIN_EXP - MANT - 1 {
        return Ok(if negate { -0.0 } else { 0.0 });
    }
    let shift = diff.max(MIN_EXP) - MANT - 2;
    let mut inexact = false;
    let x = if shift <= 0 {
        a.shl((-shift) as u64)
    } else {
        let (x, lost) = a.shr_mag(shift as u64);
        inexact |= lost;
        x
    };
    let (q, r) = BigInt::divmod_mag(&x, &b);
    if !r.is_zero() {
        inexact = true;
    }
    let x_bits = q.bit_length() as i64;
    let extra = x_bits.max(MIN_EXP - shift) - MANT;
    let mask: u128 = 1 << (extra - 1);
    let mut v = q.to_i128().unwrap() as u128 | inexact as u128;
    if (v & mask) != 0 && (v & (3 * mask - 1)) != 0 {
        v += mask;
    }
    v &= !(2 * mask - 1);
    let dx = v as f64;
    if shift + x_bits >= MAX_EXP && (shift + x_bits > MAX_EXP || dx == ldexp(1.0, x_bits as i32)) {
        return Err(PyErr::Overflow);
    }
    let r = ldexp(dx, shift as i32);
    Ok(if negate { -r } else { r })
}

// ---------------------------------------------------------------------------------------------
// Python Fraction (exact rationals), on Python ints.

#[derive(Clone, Debug)]
pub struct Fraction {
    pub num: BigInt,
    pub den: BigInt,
}

impl Fraction {
    /// `Fraction(n, d)`, normalised (ZeroDivisionError on d == 0).
    pub fn new(num: BigInt, den: BigInt) -> Fraction {
        assert!(!den.is_zero(), "ZeroDivisionError: Fraction(n, 0)");
        let g = num.gcd(&den);
        let (mut n, mut d) = if g.is_zero() { (num, den) } else { (num.divmod(&g).0, den.divmod(&g).0) };
        if d.is_neg() {
            n = n.neg();
            d = d.neg();
        }
        Fraction { num: n, den: d }
    }
    pub fn int(v: i128) -> Fraction {
        Fraction { num: BigInt::from_i128(v), den: BigInt::from_i128(1) }
    }
    pub fn ratio(n: i128, d: i128) -> Fraction {
        Fraction::new(BigInt::from_i128(n), BigInt::from_i128(d))
    }
    /// `Fraction(float)`: the float's exact value.
    pub fn from_f64(x: f64) -> Fraction {
        assert!(x.is_finite(), "ValueError: cannot convert to Fraction");
        if x == x.trunc() {
            return Fraction { num: BigInt::from_f64(x), den: BigInt::from_i128(1) };
        }
        let bits = x.abs().to_bits();
        let exp = ((bits >> 52) & 0x7ff) as i64;
        let frac = bits & ((1u64 << 52) - 1);
        let (m, e) = if exp == 0 { (frac, -1074) } else { (frac | (1u64 << 52), exp - 1075) };
        let num = BigInt::from_i128(if x < 0.0 { -(m as i128) } else { m as i128 });
        Fraction::new(num, BigInt::from_i128(1).shl((-e) as u64))
    }
    /// `Fraction("1.5")` / `Fraction("3")` for a plain decimal string (ValueError otherwise).
    pub fn from_decimal_str(s: &str) -> Option<Fraction> {
        let s = s.trim();
        let (neg, body) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        let (whole, frac) = match body.split_once('.') {
            Some((w, f)) => (w, f),
            None => (body, ""),
        };
        if (whole.is_empty() && frac.is_empty())
            || !whole.bytes().all(|b| b.is_ascii_digit())
            || !frac.bytes().all(|b| b.is_ascii_digit())
        {
            return None;
        }
        if body.contains('.') && whole.is_empty() && frac.is_empty() {
            return None;
        }
        let digits = format!("{}{}", whole, frac);
        let num = BigInt::parse(if digits.is_empty() { "0" } else { &digits })?;
        let den = BigInt::from_i128(10).pow(frac.len() as u32);
        let f = Fraction::new(num, den);
        Some(if neg { f.neg() } else { f })
    }
    pub fn neg(&self) -> Fraction {
        Fraction { num: self.num.neg(), den: self.den.clone() }
    }
    pub fn add(&self, o: &Fraction) -> Fraction {
        Fraction::new(self.num.mul(&o.den).add(&o.num.mul(&self.den)), self.den.mul(&o.den))
    }
    pub fn mul(&self, o: &Fraction) -> Fraction {
        Fraction::new(self.num.mul(&o.num), self.den.mul(&o.den))
    }
    pub fn div(&self, o: &Fraction) -> Fraction {
        Fraction::new(self.num.mul(&o.den), self.den.mul(&o.num))
    }
    pub fn cmp(&self, o: &Fraction) -> Ordering {
        self.num.mul(&o.den).cmp(&o.num.mul(&self.den))
    }
    pub fn eq(&self, o: &Fraction) -> bool {
        self.cmp(o) == Ordering::Equal
    }
    pub fn is_zero(&self) -> bool {
        self.num.is_zero()
    }
    pub fn den_is_one(&self) -> bool {
        self.den == BigInt::from_i128(1)
    }
    /// `float(fraction)`: correctly rounded.
    pub fn to_f64(&self) -> f64 {
        int_truediv(&self.num, &self.den).unwrap_or_else(|_| panic!("OverflowError"))
    }
    /// `round(fraction)`: half to even, as an int.
    pub fn round(&self) -> BigInt {
        let (floor, rem) = self.num.divmod(&self.den);
        let twice = rem.shl(1);
        match twice.cmp(&self.den) {
            Ordering::Less => floor,
            Ordering::Greater => floor.add(&BigInt::from_i128(1)),
            Ordering::Equal => {
                let (_, r2) = floor.divmod(&BigInt::from_i128(2));
                if r2.is_zero() { floor } else { floor.add(&BigInt::from_i128(1)) }
            }
        }
    }
    /// `Fraction.limit_denominator(max)`: CPython's algorithm.
    pub fn limit_denominator(&self, max_den: i128) -> Fraction {
        let max = BigInt::from_i128(max_den);
        if self.den.cmp(&max) != Ordering::Greater {
            return self.clone();
        }
        let one = BigInt::from_i128(1);
        let (mut p0, mut q0, mut p1, mut q1) = (BigInt::zero(), one.clone(), one.clone(), BigInt::zero());
        let (mut n, mut d) = (self.num.clone(), self.den.clone());
        loop {
            let (a, _) = n.divmod(&d);
            let q2 = q0.add(&a.mul(&q1));
            if q2.cmp(&max) == Ordering::Greater {
                break;
            }
            let np0 = p1.clone();
            let nq0 = q1.clone();
            let np1 = p0.add(&a.mul(&p1));
            p0 = np0;
            q0 = nq0;
            p1 = np1;
            q1 = q2;
            let nd = n.sub(&a.mul(&d));
            n = d;
            d = nd;
        }
        let (k, _) = max.sub(&q0).divmod(&q1);
        let bound1 = Fraction::new(p0.add(&k.mul(&p1)), q0.add(&k.mul(&q1)));
        let bound2 = Fraction::new(p1.clone(), q1.clone());
        let d2 = bound2.add(&self.neg()).abs();
        let d1 = bound1.add(&self.neg()).abs();
        if d2.cmp(&d1) != Ordering::Greater { bound2 } else { bound1 }
    }
    pub fn abs(&self) -> Fraction {
        Fraction { num: self.num.abs(), den: self.den.clone() }
    }
    /// `str(fraction)`: "3/4", or "3" when whole.
    pub fn to_str(&self) -> String {
        if self.den_is_one() {
            self.num.to_decimal()
        } else {
            format!("{}/{}", self.num.to_decimal(), self.den.to_decimal())
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Python Decimal, for `Decimal(written).quantize(Decimal(1).scaleb(-places), ROUND_HALF_UP)`.

/// A decimal number as written: sign, digits and exponent (value = digits * 10^exp).
#[derive(Clone, Debug)]
pub struct Dec {
    neg: bool,
    digits: Vec<u8>,
    exp: i64,
}

impl Dec {
    /// `Decimal("2.675")` for "-?\d+(\.\d+)?".
    pub fn parse(s: &str) -> Dec {
        let (neg, body) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s),
        };
        let (whole, frac) = body.split_once('.').unwrap_or((body, ""));
        let mut digits: Vec<u8> = whole.bytes().chain(frac.bytes()).map(|b| b - b'0').collect();
        // Python keeps the coefficient as written ("007.50" is 750 x 10^-2); leading zeros do not
        // change the value or the formatting below.
        while digits.len() > 1 && digits[0] == 0 {
            digits.remove(0);
        }
        Dec { neg, digits, exp: -(frac.len() as i64) }
    }
    /// `quantize(Decimal('1E{exp}'), rounding=ROUND_HALF_UP)`.
    pub fn quantize_half_up(&self, exp: i64) -> Dec {
        if exp <= self.exp {
            let mut digits = self.digits.clone();
            digits.extend(std::iter::repeat(0).take((self.exp - exp) as usize));
            return Dec { neg: self.neg, digits, exp };
        }
        let drop = (exp - self.exp) as usize;
        let n = self.digits.len();
        let (kept, first_dropped) = if drop >= n {
            (Vec::new(), if drop == n { self.digits[0] } else { 0 })
        } else {
            (self.digits[..n - drop].to_vec(), self.digits[n - drop])
        };
        let mut kept = if kept.is_empty() { vec![0] } else { kept };
        if first_dropped >= 5 {
            let mut i = kept.len();
            loop {
                if i == 0 {
                    kept.insert(0, 1);
                    break;
                }
                i -= 1;
                if kept[i] == 9 {
                    kept[i] = 0;
                } else {
                    kept[i] += 1;
                    break;
                }
            }
        }
        while kept.len() > 1 && kept[0] == 0 {
            kept.remove(0);
        }
        if kept.len() > 28 {
            panic!("decimal.InvalidOperation: quantize result has too many digits");
        }
        Dec { neg: self.neg, digits: kept, exp }
    }
    fn is_zero(&self) -> bool {
        self.digits.iter().all(|&d| d == 0)
    }
    /// `f"{d:f}"`: fixed notation, the sign kept even on zero.
    pub fn fmt_f(&self) -> String {
        let digits: String = self.digits.iter().map(|d| (b'0' + d) as char).collect();
        let body = if self.exp >= 0 {
            if self.is_zero() { "0".to_string() } else { format!("{}{}", digits, "0".repeat(self.exp as usize)) }
        } else {
            let places = (-self.exp) as usize;
            let padded = if digits.len() <= places {
                format!("{}{}", "0".repeat(places + 1 - digits.len()), digits)
            } else {
                digits
            };
            let cut = padded.len() - places;
            format!("{}.{}", &padded[..cut], &padded[cut..])
        };
        if self.neg { format!("-{body}") } else { body }
    }
    /// The value as an exact fraction, for comparisons.
    fn fraction(&self) -> Fraction {
        let digits: String = self.digits.iter().map(|d| (b'0' + d) as char).collect();
        let n = BigInt::parse(&digits).unwrap();
        let n = if self.neg { n.neg() } else { n };
        if self.exp >= 0 {
            Fraction::new(n.mul(&BigInt::from_i128(10).pow(self.exp as u32)), BigInt::from_i128(1))
        } else {
            Fraction::new(n, BigInt::from_i128(10).pow((-self.exp) as u32))
        }
    }
    pub fn cmp(&self, o: &Dec) -> Ordering {
        self.fraction().cmp(&o.fraction())
    }
}

// ---------------------------------------------------------------------------------------------
// Python datetime.date

/// A calendar day, as Python's `datetime.date`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

const MONTH_FULL: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAY_FULL: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

fn is_leap(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

/// `calendar.monthrange(year, month)[1]`.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => panic!("calendar.IllegalMonthError: bad month number {month}"),
    }
}

impl Date {
    /// `datetime.date(y, m, d)`, or None where Python raises ValueError.
    pub fn new(year: i64, month: i64, day: i64) -> Option<Date> {
        if !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
            return None;
        }
        if day < 1 || day > days_in_month(year as i32, month as u32) as i64 {
            return None;
        }
        Some(Date { year: year as i32, month: month as u32, day: day as u32 })
    }
    /// `datetime.date(y, m, d)` that Python would raise on: a panic.
    pub fn of(year: i64, month: i64, day: i64) -> Date {
        Date::new(year, month, day).unwrap_or_else(|| panic!("ValueError: day is out of range for month"))
    }
    /// Days since 0001-01-01 (Python's `toordinal() - 1`).
    fn days(&self) -> i64 {
        let y = self.year as i64 - 1;
        let mut n = y * 365 + y / 4 - y / 100 + y / 400;
        for m in 1..self.month {
            n += days_in_month(self.year, m) as i64;
        }
        n + self.day as i64 - 1
    }
    fn from_days(mut n: i64) -> Date {
        // 400-year cycles of 146097 days
        let mut year = 1 + 400 * n.div_euclid(146097);
        n = n.rem_euclid(146097);
        loop {
            let len = if is_leap(year as i32) { 366 } else { 365 };
            if n < len {
                break;
            }
            n -= len;
            year += 1;
        }
        let mut month = 1;
        loop {
            let len = days_in_month(year as i32, month) as i64;
            if n < len {
                break;
            }
            n -= len;
            month += 1;
        }
        if !(1..=9999).contains(&year) {
            panic!("OverflowError: date value out of range");
        }
        Date { year: year as i32, month, day: n as u32 + 1 }
    }
    /// `date + timedelta(days=n)`.
    pub fn plus(&self, n: i64) -> Date {
        Date::from_days(self.days() + n)
    }
    /// `(self - other).days`.
    pub fn minus(&self, other: &Date) -> i64 {
        self.days() - other.days()
    }
    /// `date.weekday()`: Monday is 0.
    pub fn weekday(&self) -> u32 {
        // 0001-01-01 was a Monday
        self.days().rem_euclid(7) as u32
    }
    /// `date.replace(year=y)`, None where the day does not exist that year.
    pub fn with_year(&self, year: i64) -> Option<Date> {
        Date::new(year, self.month as i64, self.day as i64)
    }
    /// `strftime("%A")`.
    pub fn weekday_name(&self) -> &'static str {
        WEEKDAY_FULL[self.weekday() as usize]
    }
    /// `strftime("%B")`.
    pub fn month_name(&self) -> &'static str {
        MONTH_FULL[self.month as usize - 1]
    }
}

/// `datetime.date.today()`: the pinned clock of the oracle runs (2026-11-17), or RSBOT_TODAY.
pub fn today() -> Date {
    static TODAY: LazyLock<Date> = LazyLock::new(|| {
        if let Ok(s) = std::env::var("RSBOT_TODAY") {
            let p: Vec<i64> = s.split('-').filter_map(|x| x.parse().ok()).collect();
            if p.len() == 3 {
                if let Some(d) = Date::new(p[0], p[1], p[2]) {
                    return d;
                }
            }
        }
        Date { year: 2026, month: 11, day: 17 }
    });
    *TODAY
}

// =============================================================================================
// bot/knowledge/maths.py

/// `MONTHS`: days in each month of an ordinary year, in calendar order.
static MONTHS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("MONTHS"));

/// A number the way a person writes it: 12, 2.5, 1,081 (`_number` on a float).
pub fn _number(value: f64) -> String {
    if !value.is_finite() {
        panic!("cannot round a non-finite number");
    }
    if (value - round0(value)).abs() < 1e-9 {
        return int_commas(round0(value));
    }
    // the extra place only when it makes the number exact: 3 divided by 8 is 0.375, not 0.38,
    // while a third stays 0.33 because no short decimal will ever be right.
    let mut shown = fixed(value, 2);
    for places in [2usize, 3, 4] {
        let s = fixed(value, places);
        if (s.parse::<f64>().unwrap() - value).abs() < 1e-9 {
            shown = s;
            break;
        }
    }
    shown.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// `_number` on a Python int.
pub fn _number_int(value: i128) -> String {
    commas(&value.to_string())
}

/// The numbers in a line, in order.
pub fn _parse(text: &str) -> Vec<f64> {
    rx(r"\d+(?:\.\d+)?", 0).findall_str(&text.replace(',', "")).iter().map(|raw| py_float(raw)).collect()
}

// --- arithmetic ------------------------------------------------------------------------------

/// A Python number met while evaluating a line's sum.
#[derive(Clone, Debug)]
pub enum PyNum {
    Int(BigInt),
    Float(f64),
}

impl PyNum {
    fn float(&self) -> Result<f64, PyErr> {
        match self {
            PyNum::Int(i) => i.to_f64(),
            PyNum::Float(f) => Ok(*f),
        }
    }
}

/// Python's `float.__pow__` (CPython's float_pow, special cases included); a negative base to a
/// fractional power is complex in Python, which the arithmetic card then rejects (Err).
fn float_pow(iv: f64, iw: f64) -> Result<f64, PyErr> {
    if iw == 0.0 {
        return Ok(1.0);
    }
    if iv.is_nan() {
        return Ok(iv);
    }
    if iw.is_nan() {
        return Ok(if iv == 1.0 { 1.0 } else { iw });
    }
    let odd = |w: f64| w.is_finite() && w == w.floor() && (w % 2.0).abs() == 1.0;
    if iw.is_infinite() {
        let a = iv.abs();
        return Ok(if a == 1.0 {
            1.0
        } else if (iw > 0.0) == (a > 1.0) {
            iw.abs()
        } else {
            0.0
        });
    }
    if iv.is_infinite() {
        let o = odd(iw);
        return Ok(if iw > 0.0 {
            if o { iv } else { iv.abs() }
        } else if o {
            0.0f64.copysign(iv)
        } else {
            0.0
        });
    }
    if iv == 0.0 {
        if iw < 0.0 {
            return Err(PyErr::ZeroDivision);
        }
        return Ok(if odd(iw) { iv } else { 0.0 });
    }
    let mut base = iv;
    let mut negate = false;
    if base < 0.0 {
        if iw != iw.floor() {
            return Err(PyErr::Value); // complex
        }
        base = -base;
        negate = odd(iw);
    }
    if base == 1.0 {
        return Ok(if negate { -1.0 } else { 1.0 });
    }
    let ix = base.powf(std::hint::black_box(iw));
    if ix.is_infinite() {
        return Err(PyErr::Overflow);
    }
    Ok(if negate { -ix } else { ix })
}

/// `x ** y` for two floats, through the platform `pow` (as CPython calls it).
fn py_pow(x: f64, y: f64) -> f64 {
    x.powf(std::hint::black_box(y))
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(String),
    Op(&'static str),
}

/// Python's tokenizer, for the characters an arithmetic line can still hold.
fn tokenize(s: &str) -> Result<Vec<Tok>, PyErr> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut i = 0;
    let mut ended = false; // a newline outside brackets ends the expression
    while i < chars.len() {
        let c = chars[i];
        if c == ' ' || c == '\t' || c == '\x0c' {
            i += 1;
            continue;
        }
        if c == '\n' || c == '\r' {
            if depth <= 0 {
                ended = true;
            }
            i += 1;
            continue;
        }
        if ended {
            return Err(PyErr::Syntax);
        }
        if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let mut is_float = false;
            if i < chars.len() && chars[i] == '.' {
                is_float = true;
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let raw: String = chars[start..i].iter().collect();
            if !is_float && raw.len() > 1 && raw.starts_with('0') && raw.chars().any(|d| d != '0') {
                return Err(PyErr::Syntax); // leading zeros in decimal integer literals
            }
            out.push(Tok::Num(raw));
            continue;
        }
        let two: String = chars[i..(i + 2).min(chars.len())].iter().collect();
        let op: &'static str = match (two.as_str(), c) {
            ("**", _) => "**",
            ("//", _) => "//",
            (_, '*') => "*",
            (_, '/') => "/",
            (_, '+') => "+",
            (_, '-') => "-",
            (_, '(') => "(",
            (_, ')') => ")",
            _ => return Err(PyErr::Syntax),
        };
        match op {
            "(" => depth += 1,
            ")" => depth -= 1,
            _ => {}
        }
        i += op.len();
        out.push(Tok::Op(op));
    }
    Ok(out)
}

/// The expression's AST, as `ast.parse(expression, mode="eval")` builds it.
#[derive(Clone, Debug)]
pub enum Node {
    Num(PyNum),
    Unary(&'static str, Box<Node>),
    Bin(&'static str, Box<Node>, Box<Node>),
    /// Anything that parses but is not arithmetic (a call, an empty tuple, `//`).
    Other,
}

struct Parser {
    toks: Vec<Tok>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }
    fn is_op(&self, op: &str) -> bool {
        matches!(self.peek(), Some(Tok::Op(o)) if *o == op)
    }
    fn sum(&mut self) -> Result<Node, PyErr> {
        let mut left = self.term()?;
        loop {
            let op = if self.is_op("+") {
                "+"
            } else if self.is_op("-") {
                "-"
            } else {
                break;
            };
            self.at += 1;
            let right = self.term()?;
            left = Node::Bin(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn term(&mut self) -> Result<Node, PyErr> {
        let mut left = self.factor()?;
        loop {
            let op = if self.is_op("*") {
                "*"
            } else if self.is_op("/") {
                "/"
            } else if self.is_op("//") {
                "//"
            } else {
                break;
            };
            self.at += 1;
            let right = self.factor()?;
            left = if op == "//" { Node::Other } else { Node::Bin(op, Box::new(left), Box::new(right)) };
        }
        Ok(left)
    }
    fn factor(&mut self) -> Result<Node, PyErr> {
        if self.is_op("+") || self.is_op("-") {
            let op = if self.is_op("+") { "+" } else { "-" };
            self.at += 1;
            let inner = self.factor()?;
            return Ok(Node::Unary(op, Box::new(inner)));
        }
        self.power()
    }
    fn power(&mut self) -> Result<Node, PyErr> {
        let base = self.primary()?;
        if self.is_op("**") {
            self.at += 1;
            let exp = self.factor()?;
            return Ok(Node::Bin("**", Box::new(base), Box::new(exp)));
        }
        Ok(base)
    }
    fn primary(&mut self) -> Result<Node, PyErr> {
        let mut node = self.atom()?;
        // a call: "2(3)" parses (with a warning) and is not arithmetic
        while self.is_op("(") {
            self.at += 1;
            if !self.is_op(")") {
                self.sum()?;
            }
            if !self.is_op(")") {
                return Err(PyErr::Syntax);
            }
            self.at += 1;
            node = Node::Other;
        }
        Ok(node)
    }
    fn atom(&mut self) -> Result<Node, PyErr> {
        match self.peek().cloned() {
            Some(Tok::Num(raw)) => {
                self.at += 1;
                if raw.contains('.') {
                    Ok(Node::Num(PyNum::Float(raw.parse::<f64>().map_err(|_| PyErr::Syntax)?)))
                } else {
                    Ok(Node::Num(PyNum::Int(BigInt::parse(&raw).ok_or(PyErr::Syntax)?)))
                }
            }
            Some(Tok::Op("(")) => {
                self.at += 1;
                if self.is_op(")") {
                    self.at += 1;
                    return Ok(Node::Other); // ()
                }
                let inner = self.sum()?;
                if !self.is_op(")") {
                    return Err(PyErr::Syntax);
                }
                self.at += 1;
                Ok(inner)
            }
            _ => Err(PyErr::Syntax),
        }
    }
}

/// `ast.parse(expression, mode="eval")`.
pub fn parse_expression(text: &str) -> Result<Node, PyErr> {
    let toks = tokenize(text)?;
    let mut p = Parser { toks, at: 0 };
    let node = p.sum()?;
    if p.at != p.toks.len() {
        return Err(PyErr::Syntax);
    }
    Ok(node)
}

/// `_eval(node)`: the arithmetic of the parsed line, Python's int and float rules.
pub fn _eval(node: &Node) -> Result<PyNum, PyErr> {
    match node {
        Node::Num(n) => Ok(n.clone()),
        Node::Other => Err(PyErr::Value),
        Node::Unary(op, inner) => {
            let v = _eval(inner)?;
            Ok(match (op, v) {
                (&"-", PyNum::Int(i)) => PyNum::Int(i.neg()),
                (&"-", PyNum::Float(f)) => PyNum::Float(-f),
                (_, v) => v,
            })
        }
        Node::Bin(op, l, r) => {
            let left = _eval(l)?;
            let right = _eval(r)?;
            if *op == "**" {
                let too_big = match &right {
                    PyNum::Int(i) => i.abs().cmp(&BigInt::from_i128(64)) == Ordering::Greater,
                    PyNum::Float(f) => f.abs() > 64.0,
                };
                if too_big {
                    return Err(PyErr::Value);
                }
            }
            binop(op, left, right)
        }
    }
}

fn binop(op: &str, left: PyNum, right: PyNum) -> Result<PyNum, PyErr> {
    if let (PyNum::Int(a), PyNum::Int(b)) = (&left, &right) {
        return match op {
            "+" => Ok(PyNum::Int(a.add(b))),
            "-" => Ok(PyNum::Int(a.sub(b))),
            "*" => Ok(PyNum::Int(a.mul(b))),
            "/" => Ok(PyNum::Float(int_truediv(a, b)?)),
            "**" => {
                if !b.is_neg() {
                    let e = b.to_i128().unwrap() as u32;
                    Ok(PyNum::Int(a.pow(e)))
                } else {
                    Ok(PyNum::Float(float_pow(a.to_f64()?, b.to_f64()?)?))
                }
            }
            _ => Err(PyErr::Value),
        };
    }
    let a = left.float()?;
    let b = right.float()?;
    match op {
        "+" => Ok(PyNum::Float(a + b)),
        "-" => Ok(PyNum::Float(a - b)),
        "*" => Ok(PyNum::Float(a * b)),
        "/" => {
            if b == 0.0 {
                Err(PyErr::ZeroDivision)
            } else {
                Ok(PyNum::Float(a / b))
            }
        }
        "**" => Ok(PyNum::Float(float_pow(a, b)?)),
        _ => Err(PyErr::Value),
    }
}

/// The arithmetic expression in a line, or None. Only digits and operators survive.
pub fn _expression(clean: &str) -> Option<String> {
    let mut text = clean.to_string();
    text = sub(
        r"\b(?:what's|whats|what is|calculate|work out|how much is|how much does|please|tell me|the answer to)\b",
        " ",
        &text,
    );
    // "how do i divide 3 by 8" is the same sum as "3 divided by 8": the asking words carry no
    // number and no operator, so they go before the operator words are read.
    text = sub(
        r"\b(?:how do i|how do you|how can i|how would i|how to|can you|could you|whats the answer to|what is the answer to)\b",
        " ",
        &text,
    );
    text = text.replace(',', "");
    text = sub(r"\bmultiplied by\b|\btimes\b|\bx\b", "*", &text);
    text = sub(r"\bdivide\b\s+(\d[\d.]*)\s+by\s+(\d[\d.]*)", r"\1/\2", &text);
    text = sub(r"\bdivided by\b|\bover\b", "/", &text);
    text = sub(r"\bplus\b|\band\b", "+", &text);
    text = sub(r"\bminus\b|\btake away\b|\bsubtract\b", "-", &text);
    text = sub(r"\bsquared\b", "**2", &text);
    text = sub(r"\bcubed\b", "**3", &text);
    text = sub(r"\b(?:to the power of|to the power|raised to the power of|raised to the|raised to)\b", "**", &text);
    text = text.replace('^', "**");
    let text = text::strip(&text.replace('?', " ")).to_string();
    if fullmatch(r"[0-9+\-*/().\s]+", &text).is_none() {
        return None;
    }
    if !has(r"\d", &text) || !has(r"[+\-*/]", &text) {
        return None;
    }
    if text.chars().count() > 80 {
        return None;
    }
    Some(text)
}

/// The arithmetic card: "That's 4." for "what is 2+2".
pub fn _calc(clean: &str) -> Option<String> {
    let expression = _expression(clean)?;
    let value = parse_expression(&expression).and_then(|node| _eval(&node)).ok()?;
    let said = match value {
        PyNum::Int(i) => commas(&i.to_decimal()),
        PyNum::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                return None;
            }
            _number(f)
        }
    };
    Some(format!("That's {said}."))
}

// --- roots, fractions and simple equations ----------------------------------------------------

/// Whether a Python int (as an integral float's exact value) equals a float.
fn int_eq_float(i: &BigInt, f: f64) -> bool {
    f.is_finite() && f == f.trunc() && BigInt::from_f64(f) == *i
}

/// A square root: "what's the square root of 144", "sqrt 81".
pub fn _root(clean: &str) -> Option<String> {
    if clean.contains("cube root") || clean.contains("fourth root") {
        return None;
    }
    let m = search(r"(?:square root(?: of)?|sqrt|root(?: of)?)\s*([\d,.]+)", clean)?;
    let value = py_float(&m.str(1).replace(',', ""));
    if value < 0.0 {
        return Some("A negative number has no real square root.".into());
    }
    let root = py_pow(value, 0.5);
    if (root - round0(root)).abs() < 1e-9 {
        return Some(format!("The square root of {} is {}.", _number(value), _number(root)));
    }
    Some(format!("The square root of {} is about {}.", _number(value), _number(root)))
}

/// A root asked for in words: "the square root of 144", "cube root of 27".
pub fn _roots(clean: &str) -> Option<String> {
    let m = search(r"(square root|cube root)\s*(?:of|is)?\s*(-?[\d,.]+)", clean)?;
    let value = py_float(&m.str(2).replace(',', ""));
    let (root, exact, name) = if m.str(1) == "cube root" {
        let root = (if value < 0.0 { -1.0 } else { 1.0 }) * py_pow(value.abs(), 1.0 / 3.0);
        let exact = int_eq_float(&BigInt::from_f64(round0(root)).pow(3), value);
        (root, exact, "cube root")
    } else {
        if value < 0.0 {
            return None;
        }
        let root = py_pow(value, 0.5);
        let exact = int_eq_float(&BigInt::from_f64(round0(root)).pow(2), value);
        (root, exact, "square root")
    };
    if exact {
        return Some(format!("The {name} of {} is {}.", _number(value), _number(round0(root))));
    }
    Some(format!("The {name} of {} is about {}.", _number(value), _number(round_n(root, 4))))
}

/// "Is 91 a prime number?": a yes or no answer with the reason, since 91 looks prime.
pub fn _prime(clean: &str) -> Option<String> {
    let m = search(r"is\s+(\d+)\s+(?:a\s+)?prime", clean)?;
    let number = py_int(m.str(1));
    if number < 2 {
        return Some(format!("{number} is not prime: primes start at 2."));
    }
    let limit = trunc(py_pow(number as f64, 0.5)) as i128;
    let divisor = (2..=limit).find(|d| number % d == 0);
    if let Some(d) = divisor {
        return Some(format!("No: {number} is {d} × {}, so it is not prime.", number / d));
    }
    Some(format!("Yes: nothing but 1 and {number} divides {number}, so it is prime."))
}

static _FRACTIONS: LazyLock<Vec<(String, (i128, i128))>> = LazyLock::new(|| {
    g("_FRACTIONS")
        .pairs()
        .iter()
        .map(|(k, v)| {
            let t = v.items();
            (k.str().to_string(), (t[0].as_int().unwrap() as i128, t[1].as_int().unwrap() as i128))
        })
        .collect()
});
static _COUNTS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("_COUNTS"));
static _SINGULAR: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("_SINGULAR").str_map());
static _FRACTION_OF_FRACTION: LazyLock<&'static Re> = LazyLock::new(|| gre("_FRACTION_OF_FRACTION"));
static _FRACTION_OF_A_FRACTION: LazyLock<&'static Re> = LazyLock::new(|| gre("_FRACTION_OF_A_FRACTION"));

fn fractions_get(word: &str) -> Option<(i128, i128)> {
    lookup(&_FRACTIONS, word)
}

/// Python `int(s)` for a string that may not be one (None where Python raises ValueError).
fn int_or_none(s: &str) -> Option<BigInt> {
    let t = s.trim();
    let t2: String = t.replace('_', "");
    if t.is_empty() || t.starts_with('_') || t.ends_with('_') || t.contains("__") {
        return None;
    }
    BigInt::parse(&t2)
}

/// '3/4', '1 1/2' or '1.5' as a Fraction, or None.
pub fn _as_fraction(text: &str) -> Option<Fraction> {
    let text = squash(text);
    let (whole, _, part) = text::rpartition(&text, " ");
    if part.contains('/') {
        let (top, _, bottom) = text::partition(part, "/");
        let top = int_or_none(top)?;
        let bottom = int_or_none(bottom)?;
        if bottom.is_zero() {
            return None;
        }
        let mut value = Fraction::new(top, bottom);
        if !text::strip(whole).is_empty() {
            let w = int_or_none(whole)?;
            value = value.add(&Fraction { num: w, den: BigInt::from_i128(1) });
        }
        return Some(value);
    }
    Fraction::from_decimal_str(part)
}

/// A Fraction the way a recipe reads it: 3/8, 1 1/2, 2.
pub fn _fraction_text(value: &Fraction) -> String {
    let value = value.limit_denominator(64);
    if value.den_is_one() {
        return value.num.to_decimal();
    }
    let (whole, rest) = value.num.divmod(&value.den);
    if !whole.is_zero() {
        return format!("{} {}/{}", whole.to_decimal(), rest.to_decimal(), value.den.to_decimal());
    }
    format!("{}/{}", rest.to_decimal(), value.den.to_decimal())
}

/// A part of a fraction, or of a mixed number: "half of 3/4 cup", "2/3 of 3/8".
pub fn _fraction_of_a_fraction(clean: &str) -> Option<String> {
    let mut found = _FRACTION_OF_FRACTION.search(clean);
    let bare = found.is_none();
    if bare {
        found = _FRACTION_OF_A_FRACTION.search(clean);
    }
    let m = found?;
    let count_word = m.get(1);
    let part_word = m.str(2);
    let mut part = match fractions_get(part_word) {
        Some((t, b)) => Fraction::ratio(t, b),
        None => _as_fraction(part_word)?,
    };
    if part.is_zero() {
        return None;
    }
    if let Some(cw) = count_word.filter(|w| !w.is_empty()) {
        let count = if isdigit(cw) { py_int(cw) } else { lookup(&_COUNTS, cw).unwrap_or(1) as i128 };
        part = part.mul(&Fraction::int(count));
    }
    let amount_text =
        if !bare { m.str(3).to_string() } else { m.as_str().split(" of ").last().unwrap_or("").to_string() };
    let amount = _as_fraction(&amount_text)?;
    let value = part.mul(&amount);
    let mut unit = if bare { String::new() } else { m.str(4).to_string() };
    // a quantity of one or less is singular whatever the person typed
    if value.cmp(&Fraction::int(1)) != Ordering::Greater {
        unit = _SINGULAR.get(&unit).cloned().unwrap_or(unit);
    }
    let said: Vec<&str> = [count_word.unwrap_or(""), part_word].into_iter().filter(|w| !w.is_empty()).collect();
    let said = upper_first(&said.join(" "));
    let said_amount = _fraction_text(&amount) + &(if !bare { format!(" {}", m.str(4)) } else { String::new() });
    let mut answer = format!(
        "{} of {} is {}{}",
        said,
        said_amount,
        _fraction_text(&value),
        if unit.is_empty() { String::new() } else { format!(" {unit}") }
    );
    if unit.starts_with("cup") {
        let spoons = value.mul(&Fraction::int(16));
        let millilitres = value.mul(&Fraction::int(240));
        let spoon_text = if spoons.den_is_one() {
            _fraction_text(&spoons)
        } else {
            format!("about {}", int_str(round0(spoons.to_f64())))
        };
        answer += &format!(", which is {} tablespoons ({} ml)", spoon_text, millilitres.round().to_decimal());
    }
    if bare || amount_text.contains('/') {
        if part.eq(&Fraction::ratio(1, 2)) {
            answer += ": halve the top number and keep the bottom one.";
        } else {
            answer += ": multiply the tops and the bottoms.";
        }
    } else {
        answer += ".";
    }
    Some(answer)
}

/// A simple fraction of an amount: "half of 96", "three quarters of 200", "2/5 of 60".
pub fn _fraction_of(clean: &str) -> Option<String> {
    if let Some(kitchen) = _fraction_of_a_fraction(clean) {
        if !kitchen.is_empty() {
            return Some(kitchen);
        }
    }
    let (top, bottom, amount, label, end);
    if let Some(m) = search(r"\b(\d+)\s*/\s*(\d+)\s+of\s+([\d,.]+)", clean) {
        top = py_float(m.str(1));
        bottom = py_float(m.str(2));
        amount = py_float(&m.str(3).replace(',', ""));
        label = format!("{}/{}", _number(top), _number(bottom));
        end = m.end();
    } else {
        let m = search(
            r"\b(a|an|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|\d+)?\s*(half|halves|third|thirds|quarter|quarters|fifth|fifths|sixth|sixths|eighth|eighths|tenth|tenths)\s+of\s+([\d,.]+)",
            clean,
        )?;
        let mut said = m.get(1).filter(|s| !s.is_empty()).unwrap_or("a").to_string();
        let count: i128 = if isdigit(&said) { py_int(&said) } else { lookup(&_COUNTS, &said).unwrap_or(1) as i128 };
        if isdigit(&said) {
            if let Some((word, _)) = _COUNTS.iter().find(|(_, n)| *n as i128 == count) {
                said = word.clone();
            }
        }
        let (part, whole) = fractions_get(m.str(2)).unwrap();
        top = (part * count) as f64;
        bottom = whole as f64;
        amount = py_float(&m.str(3).replace(',', ""));
        let words: Vec<&str> = [m.get(1).unwrap_or(""), m.str(2)].into_iter().filter(|w| !w.is_empty()).collect();
        let mut l = words.join(" ");
        if isdigit(&said) || isdigit(m.get(1).unwrap_or("")) {
            l = format!("{said} {}", m.str(2));
        }
        label = l;
        end = m.end();
    }
    if bottom == 0.0 || top == 0.0 {
        return None;
    }
    let value = amount * top / bottom;
    // Keep the unit the person used: "three quarters of 60 grams" is 45 grams, not 45.
    let unit = rx(
        r"\s*(?:g|grams?|kg|kilograms?|ml|litres?|liters?|pounds?|oz|ounces?|minutes?|hours?|miles?|km|metres?|meters?)\b",
        0,
    )
    .match_(&clean[end..]);
    let said = unit.map(|u| format!(" {}", text::strip(u.as_str()))).unwrap_or_default();
    Some(format!("{} of {}{} is {}{}.", upper_first(&label), _number(amount), said, _number(round_n(value, 4)), said))
}

pub fn _share_reply(total: f64, people: i128, clean: &str) -> Option<String> {
    if people < 2 {
        return None;
    }
    let money = ["£", "$", "€"].into_iter().find(|s| clean.contains(s)).unwrap_or("");
    let share = total / people as f64;
    Some(format!("That's {money}{} each: {money}{} split {people} ways.", _number(round_n(share, 2)), _number(total)))
}

static _COUNT_VALUE: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("_COUNT_VALUE"));

pub fn _count_value(text: &str) -> Option<i128> {
    let t = text::strip(text).to_lowercase();
    if isdigit(&t) {
        return Some(py_int(&t));
    }
    lookup(&_COUNT_VALUE, &t).map(|v| v as i128)
}

/// "Split 84 pounds between 6 people": a sum divided among a number of people.
pub fn _share(clean: &str) -> Option<String> {
    let a_count = gstr("_A_COUNT");
    let mut found = search(
        &format!(
            r"[£$€]?\s*([\d,.]+)\s*(?:pounds?|quid|euros?|dollars?)?\s*(?:split|shared|divided|share)\s+(?:it\s+)?(?:equally\s+)?(?:between|among|across)\s+(?:the\s+)?({a_count})\b"
        ),
        clean,
    );
    if found.is_none() {
        found = search(
            r"(?:split|share|divide)\s+(?:the bill of\s+|the\s+|it\s+)?[£$€]?\s*([\d,.]+)\s*(?:pounds?|quid|euros?|dollars?)?\s*(?:equally\s+)?(?:between|among|across)\s+(\d+)",
            clean,
        );
    }
    if found.is_none() {
        // "96 between the three of us": no split verb at all, just the sum and who paid it.
        if let Some(between) = search(
            &format!(
                r"[£$€]?\s*([\d,.]+)\s*(?:pounds?|quid|euros?|dollars?|bucks?)?\s+between\s+(?:the\s+)?({a_count})\s+(?:of\s+us|of\s+them|people|friends|adults|ways)\b"
            ),
            clean,
        ) {
            let total = py_float(&between.str(1).replace(',', ""));
            let people = _count_value(between.str(2));
            if let Some(p) = people.filter(|p| *p != 0 && *p >= 2) {
                return _share_reply(total, p, clean);
            }
        }
    }
    let Some(m) = found else {
        // "we split the bill 4 ways on 86 pounds": the verb and the count, then the sum.
        if !has(r"\b(?:split|splitting|share|sharing|divide|dividing)\b", clean) {
            return None;
        }
        let ways = search(&format!(r"({a_count})\s*(?:ways?|people|of us|friends|adults)"), clean)?;
        let total_text: String = if let Some(mw) = search(r"([\d,.]+)\s*(?:pounds?|quid|euros?|dollars?|bucks?)", clean)
        {
            mw.str(1).to_string()
        } else if let Some(ss) = search(r"[£$€]\s*([\d,.]+)", clean) {
            ss.str(1).to_string()
        } else {
            let left: Vec<String> =
                rx(r"\d+(?:\.\d+)?", 0).findall_str(clean).into_iter().filter(|n| n != ways.str(1)).collect();
            if left.is_empty() {
                return None;
            }
            let mut best = &left[0];
            for n in &left[1..] {
                if py_float(n) > py_float(best) {
                    best = n;
                }
            }
            best.clone()
        };
        let total = py_float(&total_text.replace(',', ""));
        let people = _count_value(ways.str(1)).filter(|p| *p != 0)?;
        return _share_reply(total, people, clean);
    };
    let total = py_float(&m.str(1).replace(',', ""));
    let people = _count_value(m.str(2)).filter(|p| *p != 0 && *p >= 2)?;
    _share_reply(total, people, clean)
}

/// A decimal the way a person writes it: 0.125, 0.75, 12.
pub fn _dec(value: f64, places: usize) -> String {
    let s = fixed(value, places);
    let t = s.trim_end_matches('0').trim_end_matches('.');
    if t.is_empty() { "0".into() } else { t.to_string() }
}

/// `top / bottom` on two Python ints (correctly rounded).
fn idiv(top: i128, bottom: i128) -> f64 {
    int_truediv(&BigInt::from_i128(top), &BigInt::from_i128(bottom)).unwrap_or_else(|e| panic!("{e:?}"))
}

/// Both directions: "3/4 as a decimal" and "0.25 as a fraction".
pub fn _fraction_decimal(clean: &str) -> Option<String> {
    if let Some(m) =
        search(r"(\d+)\s*in\s*(\d+)\s*(?:as|to|into)\s*(?:a\s+)?(fraction|decimal|percentage|percent)", clean)
    {
        let (top, bottom, kind) = (py_int(m.str(1)), py_int(m.str(2)), m.str(3));
        if bottom == 0 {
            return None;
        }
        if kind.starts_with("percent") {
            return Some(format!("{top} in {bottom} is {}%.", _dec(idiv(top, bottom) * 100.0, 2)));
        }
        if kind == "decimal" {
            return Some(format!("{top} in {bottom} as a decimal is {}.", _exact_decimal(top, bottom)));
        }
        let part = Fraction::ratio(top, bottom).limit_denominator(1000);
        return Some(format!(
            "{top} in {bottom} as a fraction is {}/{}.",
            part.num.to_decimal(),
            part.den.to_decimal()
        ));
    }
    if let Some(m) = search(r"([\d,.]+)\s*(%|percent|per cent)\s*(?:as|to|into)\s*(?:a\s+)?(fraction|decimal)", clean) {
        let written = m.str(1).trim_end_matches('.');
        // The person's own sign: "25 percent as a fraction" is answered with "percent" in it.
        let sign = if m.str(2) == "%" { "%" } else { " percent" };
        let value = py_float(&written.replace(',', ""));
        if m.str(3) == "decimal" {
            return Some(format!("{written}{sign} as a decimal is {}.", _dec(value / 100.0, 6)));
        }
        let part = Fraction::from_f64(value / 100.0).limit_denominator(1000);
        if part.den_is_one() {
            return Some(format!("{written}{sign} as a fraction is {}.", part.num.to_decimal()));
        }
        return Some(format!("{written}{sign} as a fraction is {}/{}.", part.num.to_decimal(), part.den.to_decimal()));
    }
    if let Some(m) = search(r"(\d+)\s*/\s*(\d+)\s*(?:as|in|to)\s*(?:a\s+)?(decimal|percentage|percent)", clean) {
        let (top, bottom) = (py_int(m.str(1)), py_int(m.str(2)));
        if bottom == 0 {
            return None;
        }
        if m.str(3).starts_with("percent") {
            return Some(format!("{top}/{bottom} is {}%.", _dec(idiv(top, bottom) * 100.0, 2)));
        }
        return Some(format!("{top}/{bottom} as a decimal is {}.", _exact_decimal(top, bottom)));
    }
    let m = search(r"([\d,.]+)\s*(?:as|in|to)\s*(?:a\s+)?(fraction|percentage|percent)", clean)?;
    if rx(r"\s+of\s+[\d,.]+", 0).match_(&clean[m.end()..]).is_some() {
        return None; // "15 as a percentage of 80" is a share of a whole: _what_percent has it
    }
    let written = m.str(1).trim_end_matches('.');
    let value = py_float(&written.replace(',', ""));
    if m.str(2) == "fraction" {
        let part = Fraction::from_f64(value).limit_denominator(1000);
        if part.den_is_one() {
            return Some(format!("{written} is a whole number: {}.", part.num.to_decimal()));
        }
        return Some(format!("{written} is {}/{}.", part.num.to_decimal(), part.den.to_decimal()));
    }
    Some(format!("{written} as a percentage is {}%.", _dec(value * 100.0, 2)))
}

/// A fraction as a decimal: 3/8 is 0.375, not 0.38, and 1/3 is 0.33 recurring.
pub fn _exact_decimal(top: i128, bottom: i128) -> String {
    let mut left = bottom;
    for prime in [2i128, 5] {
        while left % prime == 0 {
            left = left.div_euclid(prime);
        }
    }
    if left == 1 {
        let s = fixed(idiv(top, bottom), 6);
        return s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    format!("{} recurring", _number(idiv(top, bottom)))
}

/// A fraction as a decimal or percentage, or a fraction of a number.
pub fn _fraction(clean: &str) -> Option<String> {
    let m = search(r"(-?\d+)\s*/\s*(\d+)", clean).or_else(|| search(r"(-?\d+)\s+(?:over|upon)\s+(\d+)", clean))?;
    let (top, bottom) = (py_int(m.str(1)), py_int(m.str(2)));
    if bottom == 0 {
        return None;
    }
    if clean.contains("decimal") {
        return Some(format!("{top}/{bottom} as a decimal is {}.", _exact_decimal(top, bottom)));
    }
    if clean.contains("percent") {
        return Some(format!("{top}/{bottom} is {}%.", _number(idiv(top, bottom) * 100.0)));
    }
    // not a second fraction: "2/3 of 3/8" is a fraction of a fraction
    if let Some(of) = search(r"/\s*\d+\s*of\s*([\d,.]+)(?!\s*/\s*\d)", clean) {
        let total = py_float(&of.str(1).replace(',', ""));
        return Some(format!("{top}/{bottom} of {} is {}.", _number(total), _number(idiv(top, bottom) * total)));
    }
    None
}

// --- school arithmetic that carries its own numbers -------------------------------------------

static _PLACES: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("_PLACES"));

/// The same fraction with the common factors divided out, and what was divided out.
pub fn _reduced(top: i128, bottom: i128) -> (i128, i128, i128) {
    let (mut a, mut b) = (top.abs(), bottom.abs());
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    if a == 0 {
        return (top, bottom, 1);
    }
    (top.div_euclid(a), bottom.div_euclid(a), a)
}

/// "simplify 6 over 8": the same fraction with no common factor left.
pub fn _simplify(clean: &str) -> Option<String> {
    let fraction = gstr("_FRACTION");
    let m = search(&format!(r"simplif(?:y|ies|ied)\s+(?:the\s+)?(?:fraction\s+)?{fraction}"), clean)
        .or_else(|| search(&format!(r"{fraction}\s*(?:in|to)\s+(?:its\s+)?(?:simplest form|lowest terms)"), clean))?;
    let (top, bottom) = (py_int(m.str(1)), py_int(m.str(2)));
    if bottom == 0 || top == 0 {
        return None;
    }
    let (simple_top, simple_bottom, by) = _reduced(top, bottom);
    if by == 1 {
        return Some(format!("{top}/{bottom} is already as simple as it goes: nothing divides both numbers."));
    }
    Some(format!("{top}/{bottom} simplifies to {simple_top}/{simple_bottom}: divide the top and the bottom by {by}."))
}

/// "add 1/2 and 1/3": a common bottom, then the tops.
pub fn _add_fractions(clean: &str) -> Option<String> {
    let fraction = gstr("_FRACTION");
    let found = search(&format!(r"{fraction}\s*(\+|plus|and|minus|-)\s*{fraction}"), clean);
    let sentence =
        format!(r"(?:add|subtract|take away)\s+(?:the fractions\s+)?{fraction}\s+(?:and|from|to)\s+{fraction}");
    let other = search(&sentence, clean);
    let (top_a, bottom_a, sign, top_b, bottom_b);
    if let Some(m) = found {
        top_a = py_int(m.str(1));
        bottom_a = py_int(m.str(2));
        sign = if matches!(m.str(3), "-" | "minus") { "-" } else { "+" };
        top_b = py_int(m.str(4));
        bottom_b = py_int(m.str(5));
    } else if let Some(o) = other {
        top_a = py_int(o.str(1));
        bottom_a = py_int(o.str(2));
        top_b = py_int(o.str(3));
        bottom_b = py_int(o.str(4));
        sign = if clean.starts_with("subtract") || clean.contains("take away") { "-" } else { "+" };
    } else {
        return None;
    }
    if bottom_a == 0 || bottom_b == 0 {
        return None;
    }
    let (_, _, shared) = _reduced(bottom_a, bottom_b);
    let bottom = (bottom_a * bottom_b).div_euclid(if shared == 0 { 1 } else { shared });
    let (left, right) = (bottom.div_euclid(bottom_a), bottom.div_euclid(bottom_b));
    let top = top_a * left + if sign == "+" { top_b * right } else { -top_b * right };
    let (simple_top, simple_bottom, _) = _reduced(top, bottom);
    let way = if sign == "+" { "add the tops" } else { "subtract the tops" };
    if bottom_a == bottom_b {
        return Some(format!(
            "{top_a}/{bottom_a} {sign} {top_b}/{bottom_b} = {simple_top}/{simple_bottom}: the bottoms are the same, so just {way}."
        ));
    }
    Some(format!(
        "{top_a}/{bottom_a} {sign} {top_b}/{bottom_b} = {simple_top}/{simple_bottom}. Put both over {bottom} ({}/{bottom} and {}/{bottom}), {way}, then cancel down.",
        top_a * left,
        top_b * right
    ))
}

/// The whole-number factors of one number, smallest first.
pub fn _factors_in_list(words: i128) -> Vec<i128> {
    let mut out = Vec::new();
    let mut step: i128 = 1;
    while step * step <= words {
        if words % step == 0 {
            out.push(step);
            if step != words.div_euclid(step) {
                out.push(words.div_euclid(step));
            }
        }
        step += 1;
    }
    out.sort();
    out
}

/// 1, 2, 3 and 6 - the way a list of numbers is written out loud.
pub fn _said_list(numbers: &[i128]) -> String {
    let said: Vec<String> = numbers.iter().map(|n| n.to_string()).collect();
    if said.len() <= 1 {
        return said.concat();
    }
    format!("{} and {}", said[..said.len() - 1].join(", "), said[said.len() - 1])
}

/// "the factors of 12", "is 3 a factor of 12", "is 12 a multiple of 3".
pub fn _the_factors(clean: &str) -> Option<String> {
    if has(
        r"common factor|common multiple|prime factor|factorise|factorize|factor tree|factor the|factor out|scale factor|conversion factor|factor of safety",
        clean,
    ) {
        return None;
    }
    if let Some(found) =
        search(r"\bis\s+(\d+)\s*(?:,|\bor\b|\band\b|/)?\s*(?:and\s+)?(\d+)?\s+a\s+factor\s+of\s+(\d+)\b", clean)
    {
        let mut asked = vec![py_int(found.str(1))];
        if let Some(two) = found.get(2).filter(|s| !s.is_empty()) {
            asked.push(py_int(two));
        }
        let whole = py_int(found.str(3));
        if whole < 1 || *asked.iter().max().unwrap() > 1_000_000 {
            return None;
        }
        let mut said = Vec::new();
        for number in asked {
            if number > whole {
                said.push(format!("{number} is bigger than {whole}, so no: it cannot divide into it even once."));
            } else if whole % number == 0 {
                said.push(format!(
                    "{number} is a factor of {whole}: {whole} divided by {number} is {}.",
                    whole.div_euclid(number)
                ));
            } else {
                let times = whole.div_euclid(number);
                let left = whole - times * number;
                said.push(format!(
                    "{number} is not a factor of {whole}: {number} goes in {times} time{} with {left} left over.",
                    if times != 1 { "s" } else { "" }
                ));
            }
        }
        if said.len() == 1 {
            return Some(said.remove(0));
        }
        return Some(said.join(" "));
    }
    if let Some(found) = search(r"\bis\s+(\d+)\s+a\s+multiple\s+of\s+(\d+)\b", clean) {
        let (number, base) = (py_int(found.str(1)), py_int(found.str(2)));
        if base == 0 || number.max(base) > 1_000_000 {
            return None;
        }
        let q = number.div_euclid(base);
        if number % base == 0 {
            return Some(format!("Yes: {number} is {base} x {q}, so it is in the {base} times table."));
        }
        return Some(format!(
            "No: {base} x {q} is {}, and {base} x {} is {}, so {number} is not in the {base} times table.",
            base * q,
            q + 1,
            base * (q + 1)
        ));
    }
    if let Some(found) = search(r"\bfactors? of\s+(\d+)\b(?!\s*(?:and|&)\b)", clean) {
        let number = py_int(found.str(1));
        if !(1..=100_000).contains(&number) {
            return None;
        }
        let factors = _factors_in_list(number);
        let mut pairs = Vec::new();
        for &small in &factors {
            if small * small > number {
                break;
            }
            pairs.push(format!("{small} x {}", number.div_euclid(small)));
        }
        let mut said = format!(
            "The factors of {number} are {}: those are the numbers that divide into {number} with nothing left over, counting 1 and {number} itself.",
            _said_list(&factors)
        );
        if factors.len() == 2 {
            return Some(said + &format!(" That makes {number} a prime number."));
        }
        said += &format!(
            " You only have to try up to {} - the square root - because ",
            trunc(py_pow(number as f64, 0.5)) as i128
        );
        said += &format!("the pairs do the rest: {}.", pairs.join(", "));
        return Some(said);
    }
    None
}

/// "highest common factor of 12 and 18", "lowest common multiple of 4 and 6".
pub fn _common_factors(clean: &str) -> Option<String> {
    let asked = if has(r"highest common factor|greatest common (?:factor|divisor)|\bhcf\b|\bgcd\b", clean) {
        "highest common factor"
    } else if has(r"lowest common multiple|least common multiple|\blcm\b", clean) {
        "lowest common multiple"
    } else {
        return None;
    };
    let numbers = rx(r"\b(\d+)\b", 0).findall_str(clean);
    if numbers.len() < 2 {
        return None;
    }
    let (first, second) = (py_int(&numbers[0]), py_int(&numbers[1]));
    if first == 0 || second == 0 {
        return None;
    }
    let (mut a, mut b) = (first, second);
    while b != 0 {
        let r = a.rem_euclid(b);
        a = b;
        b = r;
    }
    if asked == "highest common factor" {
        return Some(format!("The highest common factor of {first} and {second} is {a}."));
    }
    Some(format!("The lowest common multiple of {first} and {second} is {}.", first.div_euclid(a) * second))
}

static _NEAREST: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("_NEAREST"));

/// Python `round(x, n)` for any n (negative too): the exact value rounded half to even.
pub fn round_digits(x: f64, n: i64) -> f64 {
    if !x.is_finite() || x == 0.0 {
        return x;
    }
    if n > 323 {
        return x;
    }
    if n < -308 {
        return 0.0 * x;
    }
    if n >= 0 {
        return round_n(x, n as usize);
    }
    let scale = BigInt::from_i128(10).pow((-n) as u32);
    let exact = Fraction::from_f64(x);
    let q = Fraction::new(exact.num.clone(), exact.den.mul(&scale)).round();
    let back = q.mul(&scale);
    let v = back.to_f64().unwrap_or(f64::INFINITY);
    if v == 0.0 { 0.0f64.copysign(x) } else { v }
}

/// "round 3.456 to two decimal places", "round 2.5 to the nearest whole number".
pub fn _round_to(clean: &str) -> Option<String> {
    let Some(m) = search(
        r"round\s+(-?\d+(?:\.\d+)?)\s*(?:to|at)\s*(?:the\s+)?(\w+)\s*(decimal places?|places?|dp|significant figures?)",
        clean,
    ) else {
        let near =
            search(r"round\s+(-?\d+(?:\.\d+)?)\s*(?:up|down|off)?\s*(?:to|at)\s*(?:the\s+)?nearest\s+(\w+)", clean);
        let near = near.filter(|n| lookup(&_NEAREST, n.str(2)).is_some());
        let Some(near) = near else {
            let bare = search(r"\bround\s+(-?\d+\.\d+)\s*$", clean)?;
            let written = bare.str(1);
            let kept = _places(written, 0);
            let value = Dec::parse(written);
            let why = if value.quantize_half_up(0).cmp(&value) == Ordering::Greater { "up" } else { "down" };
            return Some(format!(
                "{written} is {kept} to the nearest whole number: the digit after the point decides it, so the whole number goes {why}."
            ));
        };
        let (written, word) = (near.str(1), near.str(2));
        let below = lookup(&_NEAREST, word).unwrap();
        if below != 0 {
            let stepped = Dec::parse(written).quantize_half_up(-below);
            return Some(format!("{written} is {} to the nearest {word}.", stepped.fmt_f()));
        }
        let what = if below == 0 { "whole number" } else { word };
        return Some(format!("{written} is {} to the nearest {what}.", _places(written, 0)));
    };
    let written = m.str(1);
    let value = py_float(written);
    let places = lookup(&_PLACES, m.str(2))?;
    let plural = if places != 1 { "s" } else { "" };
    if m.str(3).contains("figure") {
        if value == 0.0 {
            return None;
        }
        let digits = places - value.abs().log10().floor() as i64 - 1;
        let rounded = round_digits(value, digits);
        return Some(format!("{written} is {} to {} significant figure{plural}.", _number(rounded), m.str(2)));
    }
    Some(format!("{written} is {} to {} decimal place{plural}.", _places(written, places as usize), m.str(2)))
}

/// A decimal rounded by the hand rule, so 2.675 is 2.68.
pub fn _places(written: &str, places: usize) -> String {
    let kept = Dec::parse(written).quantize_half_up(-(places as i64));
    let text = kept.fmt_f();
    if !text.contains('.') {
        return text;
    }
    let t = text.trim_end_matches('0').trim_end_matches('.');
    if t.is_empty() { "0".into() } else { t.to_string() }
}

/// The circumference of a circle, from its radius or its diameter.
pub fn _circle_measure(clean: &str) -> Option<String> {
    if !clean.contains("circumference") {
        return None;
    }
    let radius = if let Some(m) = search(r"radius\s*(?:of\s*)?(\d+(?:\.\d+)?)", clean) {
        py_float(m.str(1))
    } else {
        let m = search(r"diameter\s*(?:of\s*)?(\d+(?:\.\d+)?)", clean)?;
        py_float(m.str(1)) / 2.0
    };
    let around = 2.0 * 3.141592653589793 * radius;
    Some(format!(
        "The circumference is 2 x pi x radius, so about {} (3.14 x {}).",
        _number(round_n(around, 2)),
        _number(round_n(2.0 * radius, 2))
    ))
}

/// "volume of a box 10cm by 5cm by 2cm": length x width x height.
pub fn _box_volume(clean: &str) -> Option<String> {
    if !clean.contains("volume") {
        return None;
    }
    let found = rx(
        r"(\d+(?:\.\d+)?)\s*(?:cm|m|mm|metres?|centimetres?|inches?|in\b)?\s*(?:by|x|\*)\s*(\d+(?:\.\d+)?)\s*(?:cm|m|mm)?\s*(?:by|x|\*)\s*(\d+(?:\.\d+)?)",
        0,
    )
    .finditer(clean);
    let first = found.first()?;
    let (length, width, height) = (py_float(first.str(1)), py_float(first.str(2)), py_float(first.str(3)));
    let volume = length * width * height;
    let unit = if clean.contains("cm") {
        "cm"
    } else if has(r"\bmetres?\b|\bm\b", clean) {
        "m"
    } else {
        ""
    };
    Some(format!(
        "{} x {} x {} is {}{}.",
        _number(length),
        _number(width),
        _number(height),
        _number(round_n(volume, 2)),
        if unit.is_empty() { String::new() } else { format!(" {unit}3") }
    ))
}

// --- quadratics ------------------------------------------------------------------------------

static _MATHS_WORDS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("_MATHS_WORDS").items().iter().map(|t| (t.items()[0].str().to_string(), t.items()[1].str().to_string())).collect()
});
static _QUADRATIC_ASK: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_QUADRATIC_ASK"));
static _FACTOR_ASK: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_FACTOR_ASK"));

/// (m, (p, q, r, s)) for m(px + q)(rx + s) in whole numbers, or None.
pub fn _factorised(a: f64, b: f64, c: f64) -> Option<(i128, (i128, i128, i128, i128))> {
    if a != trunc(a) || b != trunc(b) || c != trunc(c) || a < 1.0 {
        return None;
    }
    let (a, b, c) = (a as i128, b as i128, c as i128);
    let mut common = 0;
    for number in [a, b, c] {
        common = _gcd(common, number);
    }
    if common < 1 {
        common = 1;
    }
    let (a, b, c) = (a.div_euclid(common), b.div_euclid(common), c.div_euclid(common));
    let reach = b.abs() + c.abs() + 1;
    for p in 1..=a {
        if a % p != 0 {
            continue;
        }
        let r = a.div_euclid(p);
        for q in -reach..=reach {
            let top = b - r * q;
            if top.rem_euclid(p) != 0 {
                continue;
            }
            let s = top.div_euclid(p);
            if q * s == c {
                return Some((common, (p, q, r, s)));
            }
        }
    }
    None
}

pub fn _gcd(one: i128, other: i128) -> i128 {
    let (mut one, mut other) = (one, other);
    while other != 0 {
        let r = one.rem_euclid(other);
        // Python's % takes the sign of the divisor
        let r = if r != 0 && other < 0 { r + other } else { r };
        one = other;
        other = r;
    }
    one.abs()
}

/// (1, 3, 1, 4) said as (x + 3)(x + 4), and twice the same bracket as (x + 1)^2.
pub fn _said(found: (i128, (i128, i128, i128, i128))) -> String {
    let (common, (p, q, r, s)) = found;
    let bracket = |lead: i128, tail: i128| -> String {
        let left = if lead == 1 { "x".to_string() } else { format!("{lead}x") };
        if tail == 0 {
            return format!("({left})");
        }
        if tail > 0 { format!("({left} + {tail})") } else { format!("({left} - {})", tail.abs()) }
    };
    let (first, second) = (bracket(p, q), bracket(r, s));
    if common != 1 && first == "(x)" {
        return format!("{common}x{second}");
    }
    let said = if first == second { format!("{first}^2") } else { first + &second };
    if common == 1 { said } else { format!("{common}{said}") }
}

/// A quadratic equation with x squared in it, worked out: "x squared + 5x + 6 = 0".
pub fn _quadratic(clean: &str) -> Option<String> {
    let mut text = clean.to_string();
    for (pattern, swap) in _MATHS_WORDS.iter() {
        text = sub(pattern, swap, &text);
    }
    let text = text::strip(&sub(r"\s+", " ", &text)).to_string();
    let lead = search(r"(?:^|\s)([+-]?\d*\.?\d*)\s*\*?\s*x\s*\^\s*2", &text)?;
    let mut rest = text::strip(&text[lead.end()..]).to_string();
    let mut b = 0.0;
    if let Some(linear) = rx(r"([+-])\s*(\d*\.?\d*)\s*\*?\s*x(?![\w^])", 0).match_(&rest) {
        let size = if !linear.str(2).is_empty() { py_float(linear.str(2)) } else { 1.0 };
        b = if linear.str(1) == "+" { size } else { -size };
        rest = text::strip(&rest[linear.end()..]).to_string();
    }
    let mut c = 0.0;
    if let Some(constant) = rx(r"([+-])\s*(\d*\.?\d*)", 0).match_(&rest) {
        let size = if !constant.str(2).is_empty() { py_float(constant.str(2)) } else { 1.0 };
        c = if constant.str(1) == "+" { size } else { -size };
        rest = text::strip(&rest[constant.end()..]).to_string();
    }
    let mut right = 0.0;
    if rest.starts_with('=') {
        let side = text::strip(&rest[1..]).to_string();
        if fullmatch(r"[+-]?\d+\.?\d*", &side).is_none() {
            return None;
        }
        right = py_float(&side);
        rest = String::new();
    }
    if !rest.is_empty() {
        return None;
    }
    let raw = lead.str(1);
    let a = match raw {
        "" | "+" => 1.0,
        "-" => -1.0,
        _ => py_float(raw),
    };
    if a == 0.0 {
        return None;
    }
    let asked = text::split(&clean.replace("^2", " "));
    if right == 0.0 && !asked.iter().any(|w| _QUADRATIC_ASK.contains(w)) {
        return None;
    }
    c -= right;
    let discriminant = b * b - 4.0 * a * c;
    let factorised = _factorised(a, b, c);
    let wants_brackets = asked.iter().any(|w| _FACTOR_ASK.contains(w));
    if discriminant < -1e-9 {
        if wants_brackets {
            return Some("It does not factorise with whole numbers, and no real number solves it either: the square of a number is never negative, so the formula stops at the square root of a negative number.".into());
        }
        return Some("No real number solves that: the square of a number is never negative, so there is no real answer. (The formula gives the square root of a negative number here.)".into());
    }
    if discriminant.abs() < 1e-9 {
        let root = -b / (2.0 * a);
        if wants_brackets {
            if let Some(f) = factorised {
                return Some(format!(
                    "It factorises as {}, so x = {} is the only answer: both brackets are the same, which is why there is one root, not two.",
                    _said(f),
                    _number(root)
                ));
            }
        }
        let mut said = format!(
            "x = {}. There is one answer, not two, because both roots of the formula are the same.",
            _number(root)
        );
        if let Some(f) = factorised {
            said += &format!(" The factorised form is {}.", _said(f));
        }
        return Some(said);
    }
    let sq = py_pow(discriminant, 0.5);
    let mut roots = [(-b + sq) / (2.0 * a), (-b - sq) / (2.0 * a)];
    roots.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let whole = roots.iter().all(|r| (r - round0(*r)).abs() < 1e-9);
    if wants_brackets {
        if let Some(f) = factorised {
            let said =
                format!("It factorises as {}, so x = {} or x = {}.", _said(f), _number(roots[0]), _number(roots[1]));
            if (roots[0] - roots[1]).abs() > 1e-9 {
                return Some(said + " Multiply the brackets out to check you get the equation back.");
            }
            return Some(said);
        }
        return Some(format!(
            "It does not factorise with whole numbers, so put a = {}, b = {} and c = {} into x = (-b +/- the square root of b squared - 4ac) / 2a: that gives x = {} or x = {}.",
            _number(a),
            _number(b),
            _number(c),
            _number(round_n(roots[0], 2)),
            _number(round_n(roots[1], 2))
        ));
    }
    if whole {
        let (left, high) = (int_str(round0(roots[0])), int_str(round0(roots[1])));
        let mut said = format!(
            "x = {left} or x = {high}. Put either one back in and the equation comes to zero, which is the check."
        );
        if let Some(f) = factorised {
            said += &format!(" The factorised form is {}.", _said(f));
        }
        return Some(said);
    }
    let exact = roots.iter().all(|r| (r - round_n(*r, 2)).abs() < 1e-9);
    if let Some(f) = factorised {
        return Some(format!(
            "x = {} or x = {}, and both are exact: the factorised form is {}. Put either one back in and the equation comes to zero.",
            _number(roots[0]),
            _number(roots[1]),
            _said(f)
        ));
    }
    if exact {
        return Some(format!(
            "x = {} or x = {}. Put either one back in and the equation comes to zero, which is the check.",
            _number(roots[0]),
            _number(roots[1])
        ));
    }
    Some(format!(
        "x = {} or x = {}, to two decimal places: it does not factor with whole numbers, so use the formula x = (-b +/- the square root of b squared - 4ac) / 2a.",
        _number(round_n(roots[0], 2)),
        _number(round_n(roots[1], 2))
    ))
}

/// A one-variable equation with x: "solve 2x + 3 = 11".
pub fn _equation(clean: &str) -> Option<String> {
    if !clean.contains('=') || !has(r"(?<![a-z])x(?![a-z])", clean) {
        return None;
    }
    if !(["solve", "find x", "value of x", "what is x"].iter().any(|w| clean.contains(w))
        || fullmatch(r"[\s0-9x+\-=*.]*", clean).is_some())
    {
        return None;
    }
    let m = search(r"([+-]?\d*)\*?x(?![a-z])\s*([+-]\s*\d+(?:\.\d+)?)?\s*=\s*([+-]?\d+(?:\.\d+)?)", clean)?;
    let mut coefficient = if !matches!(m.str(1), "" | "+" | "-") { py_float(m.str(1)) } else { 1.0 };
    if m.str(1) == "-" {
        coefficient = -1.0;
    }
    let add = match m.get(2) {
        Some(s) if !s.is_empty() => py_float(&s.replace(' ', "")),
        _ => 0.0,
    };
    let right = py_float(&m.str(3).replace(' ', ""));
    if coefficient == 0.0 {
        return None;
    }
    let x = (right - add) / coefficient;
    Some(format!("x = {}.", _number(x)))
}

// --- percentages ---------------------------------------------------------------------------

pub fn _percent(clean: &str) -> Option<String> {
    if has(r"(?:%|percent|per cent)\s*(?:tip|gratuity)\b", clean) {
        return None;
    }
    if let Some(after) = search(
        r"(?:costs?|priced at|is|are|was)\s+(?:£|\$)?([\d,.]+)\D.*?(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)\s*off",
        clean,
    ) {
        let (total, part) = (_amount(after.str(1)), _amount(after.str(2)));
        let amount = total * part / 100.0;
        return Some(format!(
            "{}% off {} leaves {} (you save {}).",
            _number(part),
            _number(total),
            _number(total - amount),
            _number(amount)
        ));
    }
    let found = search(
        r"(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)\s*(?:service(?:\s+charge)?|tax|charge|vat)?\s*(of|off|on(?:\s+top\s+of)?)\s*(?:(?:my|our|the|a|an)\s+)?(?:(?:bill|total|check|tab)\s+of\s+)?([\d,.]+)",
        clean,
    )
    .or_else(|| search(r"what(?:'s| is)\s*(\d+(?:\.\d+)?)\s*(of|off)\s*([\d,.]+)\s*%", clean))?;
    let (part, kind, total) = (py_float(found.str(1)), found.str(2), py_float(&found.str(3).replace(',', "")));
    let amount = total * part / 100.0;
    if kind == "off" {
        return Some(format!(
            "{}% off {} leaves {} (you save {}).",
            _number(part),
            _number(total),
            _number(total - amount),
            _number(amount)
        ));
    }
    Some(format!("{}% of {} is {}.", _number(part), _number(total), _number(amount)))
}

pub fn _percentage_change(clean: &str) -> Option<String> {
    let m =
        search(r"(?:percentage|percent)\s+(?:increase|change|difference)\s+from\s+([\d,.]+)\s+to\s+([\d,.]+)", clean)?;
    let start = py_float(&m.str(1).replace(',', ""));
    let end = py_float(&m.str(2).replace(',', ""));
    if start == 0.0 {
        return None;
    }
    let change = (end - start) / start * 100.0;
    let word = if change >= 0.0 { "increase" } else { "decrease" };
    Some(format!("That's a {}% {word}.", _number(change.abs())))
}

/// The number a short phrase gives, as a float.
pub fn _amount(text: &str) -> f64 {
    py_float(&text.replace(',', ""))
}

/// One number as a percentage of another: "what percentage is 15 of 80".
pub fn _what_percent(clean: &str) -> Option<String> {
    let patterns = [
        (r"what (?:percentage|percent) of ([\d,.]+) is ([\d,.]+)", true),
        (r"what (?:percentage|percent) (?:is )?([\d,.]+) (?:of|out of) ([\d,.]+)", false),
        (r"([\d,.]+)\s+(?:is\s+)?what\s+(?:percentage|percent)\s+of\s+([\d,.]+)", false),
        (r"([\d,.]+) (?:out of|of) ([\d,.]+) (?:as a |in )?(?:percentage|percent)", false),
        (r"([\d,.]+) as a (?:percentage|percent) of ([\d,.]+)", false),
    ];
    let mut pair = None;
    for (pattern, total_first) in patterns {
        if let Some(m) = search(pattern, clean) {
            pair = Some(if total_first {
                (_amount(m.str(2)), _amount(m.str(1)))
            } else {
                (_amount(m.str(1)), _amount(m.str(2)))
            });
            break;
        }
    }
    let (part, total) = pair?;
    if total == 0.0 {
        return None;
    }
    let share = part / total * 100.0;
    let said = _number(round_n(share, 1));
    let exact = (share - round0(share)).abs() < 1e-9;
    Some(format!("{} is {}{}% of {}.", _number(part), if exact { "exactly " } else { "about " }, said, _number(total)))
}

/// A share scaled to a bigger group: "1 in 5 of 200" is 40 people.
pub fn _a_share_of_a_group(clean: &str) -> Option<String> {
    let count = gstr("_COUNT");
    let bare = count.trim_matches(|c| c == '(' || c == ')');
    let share = search(&format!(r"\b({bare})\s+in\s+({bare})\b"), clean)?;
    let mut top = lookup(&_COUNTS, share.str(1)).map(|v| v as i128);
    let mut bottom = lookup(&_COUNTS, share.str(2)).map(|v| v as i128);
    if top.is_none() || bottom.is_none() {
        let (t, b) = (int_or_none(share.str(1)), int_or_none(share.str(2)));
        match (t, b) {
            (Some(t), Some(b)) => {
                top = Some(t.to_i128().expect("int too big"));
                bottom = Some(b.to_i128().expect("int too big"));
            }
            _ => return None,
        }
    }
    let (top, bottom) = (top.unwrap(), bottom.unwrap());
    if bottom == 0 || top > bottom {
        return None;
    }
    let rest = format!("{} {}", &clean[..share.start()], &clean[share.end()..]);
    let mut size = None;
    for pattern in [
        r"how many (?:people |of them |would that be )?(?:are |is )?(?:in|of|out of|from|out of a)\s+(\d+)",
        r"\b(?:in|of|out of|from)\s+(\d+)\s+(?:people|persons|students|adults|children|kids|men|women|workers|drivers|homes|households)",
        r"\b(?:out of|of|from)\s+(\d+)\b",
    ] {
        if let Some(m) = search(pattern, &rest) {
            size = Some(py_int(m.str(1)));
            break;
        }
    }
    let size = size.filter(|s| *s >= 2)?;
    let count = idiv(top * size, bottom);
    let answer = if (count - round0(count)).abs() < 1e-9 {
        _number(round0(count))
    } else {
        format!("about {}", _number(round_n(count, 1)))
    };
    let said = if top == 1 {
        format!("1 in {}", _number_int(bottom))
    } else {
        format!("{} in {}", _number_int(top), _number_int(bottom))
    };
    Some(format!(
        "{said} of {} is {answer}. That is the share times the group: {} divided by {}{}",
        _number_int(size),
        _number_int(size),
        _number_int(bottom),
        if top > 1 { format!(", times {}.", _number_int(top)) } else { ".".to_string() }
    ))
}

/// A percentage added to an amount: "add 20% to 40", "40 plus VAT", "add VAT to 240".
pub fn _percent_on(clean: &str) -> Option<String> {
    if let Some(m) = search(
        r"\b(increase|decrease|raise|reduce|lower|go up|go down|goes up|goes down)\s+(?:of\s+)?(?:£|\$)?([\d,.]+)\s+by\s+(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)",
        clean,
    ) {
        let (total, part) = (_amount(m.str(2)), _amount(m.str(3)));
        let going_down = matches!(m.str(1), "decrease" | "reduce" | "lower" | "go down" | "goes down");
        let changed = total * if going_down { 1.0 - part / 100.0 } else { 1.0 + part / 100.0 };
        return Some(format!(
            "{} {} by {}% is {}.",
            _number(total),
            if going_down { "decreased" } else { "increased" },
            _number(part),
            _number(round_n(changed, 2))
        ));
    }
    let plus = |total: f64, part: f64| {
        format!("{} plus {}% is {}.", _number(total), _number(part), _number(total * (1.0 + part / 100.0)))
    };
    let less = |total: f64, part: f64| {
        format!("{} less {}% is {}.", _number(total), _number(part), _number(total * (1.0 - part / 100.0)))
    };
    if let Some(m) = search(
        r"(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)\s+(?:on(?:\s+top\s+of)?|onto|added\s+to)\s+(?:£|\$)?([\d,.]+)",
        clean,
    ) {
        return Some(plus(_amount(m.str(2)), _amount(m.str(1))));
    }
    if let Some(m) = search(
        r"(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)\s+(more|less|fewer|higher|lower)\s+than\s+(?:£|\$|€)?\s*([\d,.]+)",
        clean,
    ) {
        let (part, way, total) = (_amount(m.str(1)), m.str(2), _amount(m.str(3)));
        if matches!(way, "more" | "higher") {
            return Some(plus(total, part));
        }
        return Some(less(total, part));
    }
    if has(
        r"(?:costs?|priced at|is|are|was)\s+(?:£|\$|€)?\s*[\d,.]+\D.*?(?:\d+(?:\.\d+)?)\s*(?:%|percent|per cent)\s*off",
        clean,
    ) {
        return None;
    }
    if let Some(m) = search(
        r"(?:£|\$|€)?\s*([\d,.]+)\s*(?:pounds?|quid|euros?|dollars?|bucks?)\s+(?:[a-z]+\s+){0,3}?(?:with|at|minus|less)\s+(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)\s*off\b",
        clean,
    ) {
        return Some(less(_amount(m.str(1)), _amount(m.str(2))));
    }
    if let Some(m) = search(
        r"(?:add|increase)\s+(?:a\s+)?(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)\s*(?:tip|service|tax|charge)?\s*(?:on|to|onto)\s+(?:£|\$)?([\d,.]+)",
        clean,
    ) {
        return Some(plus(_amount(m.str(2)), _amount(m.str(1))));
    }
    if has(r"\d+(?:\.\d+)?\s*(?:%|percent|per cent)\s*off\s+(?:£|\$)?[\d,.]+", clean) {
        return None;
    }
    if let Some(m) = search(
        r"(?:subtract|deduct|minus|take|taking|remove|knock|strip)\s+(?:a\s+|off\s+)?(?:the\s+)?(\d+(?:\.\d+)?)\s*(?:%|percent|per cent|per cent)\s*(?:off|from|of)\s+(?:£|\$)?([\d,.]+)",
        clean,
    ) {
        return Some(less(_amount(m.str(2)), _amount(m.str(1))));
    }
    if let Some(m) = search(r"([\d,.]+)\s*(?:minus|less|without)\s*(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)", clean) {
        return Some(less(_amount(m.str(1)), _amount(m.str(2))));
    }
    if let Some(m) = search(r"([\d,.]+)\s*(?:plus|and)\s*(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)", clean) {
        return Some(plus(_amount(m.str(1)), _amount(m.str(2))));
    }
    if let Some(m) = search(r"([\d,.]+)\s*(?:plus|with|including|before)\s+vat", clean) {
        let total = _amount(m.str(1));
        return Some(format!("{} plus VAT at 20% is {}.", _number(total), _number(total * 1.2)));
    }
    if let Some(m) = search(
        r"(?:add|plus)\s+(?:(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)\s+)?(?:the\s+)?vat\s+(?:on|to|onto)\s*(?:£|\$)?([\d,.]+)",
        clean,
    ) {
        let part = match m.get(1) {
            Some(p) if !p.is_empty() => _amount(p),
            _ => 20.0,
        };
        let total = _amount(m.str(2));
        return Some(format!(
            "{} plus VAT at {}% is {}.",
            _number(total),
            _number(part),
            _number(total * (1.0 + part / 100.0))
        ));
    }
    let m = search(r"(?:take|takes|taking|remove|removing|strip|knock)\s+(?:the\s+)?vat\s+(?:off|from|out of)\s*(?:£|\$)?([\d,.]+)", clean)
        .or_else(|| search(r"([\d,.]+)\s*(?:minus|less|without|excluding|ex)\s+vat", clean));
    if let Some(m) = m {
        let total = _amount(m.str(1));
        return Some(format!(
            "{} with VAT taken off is {}, and the VAT in it is {}.",
            _number(total),
            _number(total / 1.2),
            _number(total - total / 1.2)
        ));
    }
    None
}

static _PERCENT_ADVICE: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_PERCENT_ADVICE"));
static _MONEY_WORD: LazyLock<&'static Re> = LazyLock::new(|| gre("_MONEY_WORD"));

/// The money a percentage adds to or takes off: "rent 800 going up 5 percent".
pub fn _percent_rise(clean: &str) -> Option<String> {
    let percents = rx(r"(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)", 0).findall_str(clean);
    if percents.len() != 1 {
        return None;
    }
    let part = py_float(&percents[0]);
    if !(0.0 < part && part <= 100.0) {
        return None;
    }
    let money_word = _MONEY_WORD.is_match(clean);
    let mut amount = None;
    for raw in rx(r"(?:£|\$|€)?\s*(\d[\d,]*(?:\.\d+)?)", 0).finditer(clean) {
        let number = _amount(raw.str(1));
        if (number - part).abs() < 1e-9 {
            continue;
        }
        let first: String = text::strip(raw.as_str()).chars().take(1).collect();
        if (!first.is_empty() && "£$€".contains(first.as_str())) || number >= 100.0 || money_word {
            amount = Some(number);
            break;
        }
    }
    let amount = amount?;
    if any_in(clean, &_PERCENT_ADVICE) {
        return None;
    }
    let up = has(&format!(r"\b(?:{})\b", gstr("_PERCENT_UP")), clean);
    let down = has(&format!(r"\b(?:{})\b", gstr("_PERCENT_DOWN")), clean);
    if up == down {
        return None;
    }
    let change = amount * part / 100.0;
    if up {
        return Some(format!(
            "{}% of {} is {}, so it goes up to {}.",
            _number(part),
            _number(amount),
            _number(change),
            _number(amount + change)
        ));
    }
    Some(format!(
        "{}% of {} is {}, so it comes down to {}.",
        _number(part),
        _number(amount),
        _number(change),
        _number(amount - change)
    ))
}

static _PERCENT_ASK_WORD: LazyLock<&'static Re> = LazyLock::new(|| gre("_PERCENT_ASK_WORD"));

/// The last amount the person said, skipping the percentages in what they said.
pub fn _amount_in_the_chat(memory: &Memory) -> Option<f64> {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(3);
    for said in turns[start..].iter().rev() {
        let t = sub(r"\d+(?:\.\d+)?\s*(?:%|percent|per cent)", " ", &said.user.to_lowercase());
        let numbers = rx(r"\d[\d,]*(?:\.\d+)?", 0).findall_str(&t);
        let Some(last) = numbers.last() else { continue };
        let value = _amount(last);
        if value >= 10.0 {
            return Some(value);
        }
    }
    None
}

/// A percentage of the amount the chat already holds: "and 20 percent of that".
pub fn _percent_of_the_figure_in_the_chat(clean: &str, memory: &Memory) -> Option<String> {
    let percents = rx(r"(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)", 0).findall_str(clean);
    if percents.len() != 1 {
        return None;
    }
    if has(r"\d", &sub(r"\d+(?:\.\d+)?\s*(?:%|percent|per cent)", " ", clean)) {
        return None;
    }
    let reference = has(
        r"\b(?:of|on)\s+(?:that|it|this|those|them|the\s+(?:rent|bill|price|salary|total|figure|number|amount|money))\b",
        clean,
    );
    let off_reference = has(r"\b(?:off)\s+(?:that|it|this|those|them)\b", clean);
    let asked = _PERCENT_ASK_WORD.is_match(clean) && has(r"\b(?:be|come\s+to|work\s+out\s+to)\b[\s?.!]*$", clean);
    if !(reference || off_reference || asked) {
        return None;
    }
    let amount = _amount_in_the_chat(memory)?;
    let part = py_float(&percents[0]);
    if !(0.0 < part && part <= 100.0) {
        return None;
    }
    let change = amount * part / 100.0;
    if off_reference {
        return Some(format!(
            "{}% off {} leaves {} (you save {}).",
            _number(part),
            _number(amount),
            _number(amount - change),
            _number(change)
        ));
    }
    Some(format!("{}% of {} is {}.", _number(part), _number(amount), _number(change)))
}

// --- tips and saving goals -------------------------------------------------------------------

/// A tip on a bill: "15% tip on 45", "tip on 80 at 15 percent".
pub fn _tip(clean: &str) -> Option<String> {
    let (part, total);
    if let Some(m) = search(
        r"(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)\s*(?:tip|service|gratuity)\s*(?:on(?:\s+top\s+of)?|for|of)\s*(?:(?:a|an|the|my|our)\s+)?(?:(?:bill|total|check|tab)\s+of\s+)?(?:£|\$|€)?\s*([\d,.]+)",
        clean,
    ) {
        part = _amount(m.str(1));
        total = _amount(m.str(2));
    } else if let Some(m) = search(
        r"(?:tip|service|gratuity)\s*(?:of\s*)?(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)\s*(?:on|for|of)\s*(?:£|\$|€)?\s*([\d,.]+)",
        clean,
    ) {
        part = _amount(m.str(1));
        total = _amount(m.str(2));
    } else {
        let m = search(
            r"(?:tip|service|gratuity)\s*(?:on|for)\s*(?:£|\$|€)?\s*([\d,.]+)\s*(?:at|of)\s*(\d+(?:\.\d+)?)\s*(?:%|percent|per cent)",
            clean,
        )?;
        total = _amount(m.str(1));
        part = _amount(m.str(2));
    }
    let tip = total * part / 100.0;
    Some(format!(
        "A {}% tip on {} is {}, so {} in total.",
        _number(part),
        _number(total),
        _number(tip),
        _number(total + tip)
    ))
}

static _YOU_LEFT: LazyLock<&'static Re> = LazyLock::new(|| gre("_YOU_LEFT"));
static _YOU_LEFT_ANYWHERE: LazyLock<&'static Re> = LazyLock::new(|| gre("_YOU_LEFT_ANYWHERE"));
static _BILL_WORD: LazyLock<&'static Re> = LazyLock::new(|| gre("_BILL_WORD"));
static _A_NOTE: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("_A_NOTE"));

/// Money in a card's shape: 4.80 rather than 4.8, and 6 rather than 6.00.
pub fn _two_places(value: f64) -> String {
    let t = fixed(value, 2);
    match t.strip_suffix(".00") {
        Some(s) => s.to_string(),
        None => t,
    }
}

/// The percentage somebody left on a bill, and the usual tip on that bill.
pub fn _left_on_the_bill(clean: &str) -> Option<String> {
    let (left_text, bill_text) = if let Some(m) = _YOU_LEFT.search(clean) {
        (m.name_str("left").to_string(), m.name_str("bill").to_string())
    } else {
        let loose = _YOU_LEFT_ANYWHERE.search(clean);
        let loose = loose.filter(|_| _BILL_WORD.is_match(clean))?;
        (loose.str(1).to_string(), loose.str(2).to_string())
    };
    let left = match lookup(&_A_NOTE, &left_text) {
        Some(v) if v != 0 => v as f64,
        _ => _amount(&left_text),
    };
    let bill = _amount(&bill_text);
    if left == 0.0 || bill == 0.0 || left <= 0.0 || bill <= 0.0 {
        return None;
    }
    let percent = left / bill * 100.0;
    let money = ["£", "$", "€"].into_iter().find(|s| clean.contains(s)).unwrap_or("£");
    let usual =
        format!("{money}{} to {money}{}", _two_places(round_n(bill * 0.10, 2)), _two_places(round_n(bill * 0.125, 2)));
    let said = _number(round_n(percent, 1));
    if percent >= 10.0 {
        return Some(format!(
            "That's {said}% of the {money}{} - the usual tip is ten to twelve and a half ({usual}), so it's enough.",
            _number(bill)
        ));
    }
    Some(format!(
        "That's {said}% of the {money}{}, under the usual ten to twelve and a half ({usual}). Check the bill for a service charge first, since that is the tip already on it.",
        _number(bill)
    ))
}

/// How long a regular amount takes to reach a goal: "save 50 a month for 1000".
pub fn _save_time(clean: &str) -> Option<String> {
    if !has(r"how long|how many (?:months|weeks|years)|when will", clean) {
        return None;
    }
    let per = search(
        r"(?:save|saving|put away|putting away|set aside|setting aside)\s+(?:£|\$|€)?\s*([\d,.]+)\s*(?:a|per|each)\s*(month|week|year)",
        clean,
    );
    let goal = search(r"(?:to save|save up|reach|get to|hit|for)\s+(?:£|\$|€)?\s*([\d,.]+)", clean);
    let (per, goal) = (per?, goal?);
    let amount = _amount(per.str(1));
    let total = _amount(goal.str(1));
    let unit = per.str(2);
    if amount <= 0.0 || total <= 0.0 {
        return None;
    }
    let count = total / amount;
    let whole = trunc(count);
    let length = if (count - whole).abs() < 1e-9 {
        format!("{} {unit}{}", int_str(whole), if whole == 1.0 { "" } else { "s" })
    } else {
        format!("about {} {unit}s", _number(round_n(count, 1)))
    };
    Some(format!("Saving {} a {unit}, you get to {} in {length}.", _number(amount), _number(total)))
}

// --- speed, distance, time -------------------------------------------------------------------

static _PAY_WORDS: LazyLock<&'static Re> = LazyLock::new(|| gre("_PAY_WORDS"));
static _NOT_PAY: LazyLock<&'static Re> = LazyLock::new(|| gre("_NOT_PAY"));
static _RATE: LazyLock<&'static Re> = LazyLock::new(|| gre("_RATE"));
static _HOURS_WEEK: LazyLock<&'static Re> = LazyLock::new(|| gre("_HOURS_WEEK"));
static _ANNUAL: LazyLock<&'static Re> = LazyLock::new(|| gre("_ANNUAL"));
static _MONEY_BESIDE: LazyLock<&'static Re> = LazyLock::new(|| gre("_MONEY_BESIDE"));

/// The amount with the currency the person used, so a dollar question stays in dollars.
pub fn _money(clean: &str, amount: f64) -> String {
    let sign = if clean.contains('€') || clean.contains("euro") {
        "€"
    } else if clean.contains('$') || clean.contains("dollar") {
        "$"
    } else {
        "£"
    };
    let t = commas_fixed(amount, 2);
    let t = t.strip_suffix(".00").map(str::to_string).unwrap_or(t);
    format!("{sign}{t}")
}

/// The yearly figure in the line, or None when the number is an age or a length of time.
pub fn _annual_pay(clean: &str) -> Option<Match<'_>> {
    let m = _ANNUAL.search(clean)?;
    if m.get(2).is_some_and(|k| !k.is_empty()) {
        return Some(m);
    }
    let digits = sub(r"[.,]", "", m.str(1));
    if digits.chars().count() >= 4 {
        return Some(m);
    }
    if _MONEY_BESIDE.is_match(clean) {
        return Some(m);
    }
    None
}

/// Pay worked out from an hourly rate, or an hourly rate from a yearly salary.
pub fn _wage(clean: &str) -> Option<String> {
    if _NOT_PAY.is_match(clean) {
        return None;
    }
    let hours_match = _HOURS_WEEK.search(clean);
    let hours = hours_match.as_ref().map(|m| py_float(m.str(1))).unwrap_or(40.0);
    if hours <= 0.0 || hours > 100.0 {
        return None;
    }
    let rate = _RATE.search(clean);
    let annual = _annual_pay(clean);
    let asked = _HOURS_WEEK.sub(" ", clean);
    let asks_year = has(r"\b(?:a|per|each|/)\s*(?:year|annum)|\bannual(?:ly)?\b", &asked);
    let asks_month = has(r"\b(?:a|per|each|/)\s*month|\bmonthly\b", &asked);
    let asks_week = has(r"\b(?:a|per|each|/)\s*week|\bweekly\b", &asked);
    let asks_hour = has(r"\b(?:an|a|per|each|/)\s*hour|\bhourly\b", &asked);
    if !(_PAY_WORDS.is_match(clean)
        || (rate.is_some() && (hours_match.is_some() || asks_year || asks_month || asks_week))
        || (annual.is_some() && asks_hour))
    {
        return None;
    }
    if let (Some(rate), None) = (&rate, &annual) {
        let hourly = py_float(rate.str(1));
        let weekly = hourly * hours;
        let yearly = weekly * 52.0;
        let lead = format!("{} an hour at {} hours a week is", _money(clean, hourly), _number(hours));
        if asks_month && !asks_year {
            return Some(format!("{lead} about {} a month before tax.", _money(clean, yearly / 12.0)));
        }
        if asks_week && !(asks_year || asks_month) {
            return Some(format!("{lead} {} a week.", _money(clean, weekly)));
        }
        if asks_year {
            return Some(format!("{lead} about {} a year before tax.", _money(clean, yearly)));
        }
        if _PAY_WORDS.is_match(clean) || hours_match.is_some() {
            return Some(format!(
                "{lead} {} a week, about {} a year before tax.",
                _money(clean, weekly),
                _money(clean, yearly)
            ));
        }
        return None;
    }
    if let Some(annual) = &annual {
        if asks_hour || clean.contains("hourly rate") {
            let mut value = py_float(&annual.str(1).replace(',', ""));
            if annual.get(2).is_some_and(|k| !k.is_empty()) {
                value *= 1000.0;
            }
            let per_hour = value / (hours * 52.0);
            return Some(format!(
                "{} a year at {} hours a week is about {} an hour before tax.",
                _money(clean, value),
                _number(hours),
                _money(clean, per_hour)
            ));
        }
    }
    if clean.contains("hourly rate")
        && (clean.contains("salary") || clean.contains("annual"))
        && _parse(clean).is_empty()
    {
        return Some("Divide your yearly salary by the hours you work in a year. At 40 hours a week that is 2,080 hours, so £30,000 is about £14.42 an hour.".into());
    }
    None
}

pub fn _travel(clean: &str) -> Option<String> {
    let m = search(
        r"([\d,.]+)\s*(miles?|km|kilometres?|kilometers?)\b.*?(?:at|of)\s*([\d,.]+)\s*(?:miles?|km|kilometres?|kilometers?)?\s*(?:an hour|per hour|/hour|mph|kmh|km/h)",
        clean,
    )?;
    let distance = py_float(&m.str(1).replace(',', ""));
    let speed = py_float(&m.str(3).replace(',', ""));
    if speed == 0.0 {
        return None;
    }
    let hours = distance / speed;
    if hours < 1.0 {
        return Some(format!("About {} minutes at {} miles an hour.", _number(hours * 60.0), _number(speed)));
    }
    Some(format!("About {} hours at {} miles an hour.", _number(hours), _number(speed)))
}

/// One row of `_PACES`: (pattern, speed, said, per_mile, far_said, tail, far_tail).
#[derive(Clone, Debug)]
pub struct Pace {
    pattern: String,
    speed: f64,
    said: String,
    per_mile: String,
    far_said: String,
    tail: String,
    far_tail: String,
}

static _PACES: LazyLock<Vec<Pace>> = LazyLock::new(|| {
    g("_PACES")
        .items()
        .iter()
        .map(|row| {
            let t = row.items();
            Pace {
                pattern: t[0].str().to_string(),
                speed: t[1].as_f64().unwrap(),
                said: t[2].str().to_string(),
                per_mile: t[3].str().to_string(),
                far_said: t[4].str().to_string(),
                tail: t[5].str().to_string(),
                far_tail: t[6].str().to_string(),
            }
        })
        .collect()
});
static _A_PACE_WORD: LazyLock<&'static Re> = LazyLock::new(|| gre("_A_PACE_WORD"));
static _DISTANCE: LazyLock<&'static Re> = LazyLock::new(|| gre("_DISTANCE"));
static _AFTER_HALF: LazyLock<&'static Re> = LazyLock::new(|| gre("_AFTER_HALF"));
static _TO_WORK: LazyLock<&'static Re> = LazyLock::new(|| gre("_TO_WORK"));
static _A_DISTANCE_STATED: LazyLock<&'static Re> = LazyLock::new(|| gre("_A_DISTANCE_STATED"));
static _IS_THAT_TOO_MUCH: LazyLock<&'static Re> = LazyLock::new(|| gre("_IS_THAT_TOO_MUCH"));

/// The distance in the line as miles. "half a mile" and "a mile and a half" are both said.
pub fn _distance_in_miles(found: &Match, clean: &str) -> f64 {
    let word = found.name_str("num");
    let units = found.name_str("unit");
    let mut value = if _AFTER_HALF.is_match(clean) || word == "half" {
        0.5
    } else if word.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        py_float(word)
    } else {
        lookup(&_COUNTS, word).unwrap_or(0) as f64
    };
    if has(r"\band a half\b", clean) {
        value += 0.5;
    }
    if units.starts_with("km") || units.starts_with("kilomet") {
        return value * 0.621371;
    }
    if units.starts_with("met") {
        return value / 1609.34;
    }
    value
}

/// An hour and a bit the way a person says it: 75 minutes, or two hours 15 minutes.
pub fn _duration(hours: f64) -> String {
    let minutes = round0(hours * 60.0) as i64;
    if minutes < 60 {
        return format!("{minutes} minutes");
    }
    let (mut hours_, mins) = (minutes.div_euclid(60), minutes.rem_euclid(60));
    let say = |h: i64| if h == 1 { "an hour".to_string() } else { format!("{h} hours") };
    if mins <= 2 {
        return say(hours_);
    }
    if mins >= 58 {
        hours_ += 1;
        return say(hours_);
    }
    format!("{} {mins} minutes", say(hours_))
}

/// What a friend says about a trip of that length - the verdict, not the arithmetic.
pub fn _the_size_of_the_trip(minutes_there_and_back: f64) -> &'static str {
    if minutes_there_and_back <= 45.0 {
        return "That is a short trip - easy to do every day.";
    }
    if minutes_there_and_back <= 100.0 {
        return "That is a real part of the day, but plenty of people do it.";
    }
    "That is a long way to do every day - fine now and then, hard as a routine."
}

/// What the person said on the last few turns, newest first: the way they mean to travel.
pub fn _the_way_it_is_done(memory: Option<&Memory>, back: usize) -> Vec<String> {
    let Some(memory) = memory else { return Vec::new() };
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(back);
    turns[start..].iter().rev().map(|t| t.user.clone()).collect()
}

/// How long a walk, a ride or a run takes when the line gives the distance, not the speed.
pub fn _at_a_pace(clean: &str, memory: Option<&Memory>, a_verdict: bool) -> Option<String> {
    let asked_time = has(r"\bhow long\b|\bhow many (?:minutes|hours)\b|\bhow much longer\b", clean);
    let asked_far = has(r"\bhow far\b", clean);
    let stated = _A_DISTANCE_STATED.match_(clean).is_some()
        && (_A_PACE_WORD.is_match(&format!("{} {}", clean, _the_way_it_is_done(memory, 4).join(" ")))
            || _TO_WORK.is_match(clean));
    if !(asked_time || asked_far || stated) {
        if _IS_THAT_TOO_MUCH.is_match(clean) {
            for line in _the_way_it_is_done(memory, 4) {
                if _A_DISTANCE_STATED.match_(text::strip(&line)).is_some() {
                    return _at_a_pace(text::strip(&line), memory, true);
                }
            }
        }
        return None;
    }
    let mut pace: Option<Pace> = _PACES.iter().find(|p| has(&p.pattern, clean)).cloned();
    if pace.is_none() && stated && _TO_WORK.is_match(clean) {
        let found = _DISTANCE.search(clean);
        let miles = found.as_ref().map(|m| _distance_in_miles(m, clean)).unwrap_or(0.0);
        if miles != 0.0 && miles > 0.0 {
            let each_way = has(r"\beach way\b|\bboth ways\b", clean);
            let car = miles / 50.0 * 60.0;
            let bike = miles / 12.0 * 60.0;
            if each_way {
                return Some(format!(
                    "About {} in the car and {} on a bike, each way - so around {} to {} there and back. {}",
                    _duration(car / 60.0),
                    _duration(bike / 60.0),
                    _duration(2.0 * car / 60.0),
                    _duration(2.0 * bike / 60.0),
                    if a_verdict {
                        _the_size_of_the_trip(2.0 * car)
                    } else {
                        "A ride that short is well within reach, and it is the cheaper of the two."
                    }
                ));
            }
            return Some(format!(
                "About {} in the car and {} on a bike - the ride is the one that saves the money at that distance.",
                _duration(car / 60.0),
                _duration(bike / 60.0)
            ));
        }
    }
    if pace.is_none() && stated {
        'lines: for line in _the_way_it_is_done(memory, 4) {
            for p in _PACES.iter() {
                if has(&p.pattern, &line) {
                    pace = Some(p.clone());
                    break 'lines;
                }
            }
        }
    }
    let pace = pace?;
    let speed_g = fmt_g(pace.speed);
    if let Some(found) = _DISTANCE.search(clean) {
        let miles = _distance_in_miles(&found, clean);
        if miles == 0.0 || miles <= 0.0 {
            return None;
        }
        let minutes = miles / pace.speed * 60.0;
        if asked_far && !asked_time {
            return None;
        }
        if stated {
            if has(r"\beach way\b|\bboth ways\b", clean) {
                let tail_said =
                    if a_verdict { format!(" {}", _the_size_of_the_trip(2.0 * minutes)) } else { String::new() };
                return Some(format!(
                    "About {} {}, so roughly {} there and back - {speed_g} miles an hour, so {}.{tail_said}",
                    _duration(minutes / 60.0),
                    pace.said,
                    _duration(2.0 * minutes / 60.0),
                    pace.per_mile
                ));
            }
            return Some(format!(
                "About {} {} - {speed_g} miles an hour, so {}. {}",
                _duration(minutes / 60.0),
                pace.said,
                pace.per_mile,
                pace.tail
            ));
        }
        return Some(format!(
            "About {} {} - {speed_g} miles an hour, so {}. {}",
            _duration(miles / pace.speed),
            pace.said,
            pace.per_mile,
            pace.tail
        ));
    }
    let hour = has(r"\b(?:in|within|for)\s+(?:an?|one|1)\s+hour\b|\ban hour\b", clean);
    if hour && asked_far {
        return Some(format!(
            "About {speed_g} miles in an hour {} - {}. {}",
            pace.far_said, pace.per_mile, pace.far_tail
        ));
    }
    None
}

// --- unit conversions ------------------------------------------------------------------------

/// A unit conversion, from the one table of units (bot/knowledge/units.py).
pub fn _convert(clean: &str, memory: Option<&mut Memory>) -> Option<String> {
    let fact = _unit_fact(clean);
    let table = dup::units_convert(clean, memory);
    if fact.is_some() && !dup::units_answers_more_than_a_pair(clean) {
        return fact;
    }
    table.filter(|t| !t.is_empty()).or(fact)
}

// --- numbers the chat already holds ----------------------------------------------------------

static TIME_ASKED: LazyLock<&'static Re> = LazyLock::new(|| gre("TIME_ASKED"));
static WHAT_IF: LazyLock<&'static Re> = LazyLock::new(|| gre("WHAT_IF"));
static ABOUT_ASK: LazyLock<&'static Re> = LazyLock::new(|| gre("ABOUT_ASK"));
static CLOCK: LazyLock<&'static Re> = LazyLock::new(|| gre("CLOCK"));
static TAKES: LazyLock<&'static Re> = LazyLock::new(|| gre("TAKES"));
static A_HORIZON: LazyLock<&'static Re> = LazyLock::new(|| gre("A_HORIZON"));
static TIME_ELSEWHERE: LazyLock<&'static Re> = LazyLock::new(|| gre("TIME_ELSEWHERE"));
static RATE: LazyLock<&'static Re> = LazyLock::new(|| gre("RATE"));
static CLAUSE_SPLIT: LazyLock<&'static Re> = LazyLock::new(|| gre("CLAUSE_SPLIT"));
static SAVING_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| gre("SAVING_CLAUSE"));
static SPENDING_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| gre("SPENDING_CLAUSE"));
static OWED: LazyLock<&'static Re> = LazyLock::new(|| gre("OWED"));
static A_DEBT_CLEARED: LazyLock<&'static Re> = LazyLock::new(|| gre("A_DEBT_CLEARED"));
static PERIODS: LazyLock<Vec<(String, f64)>> =
    LazyLock::new(|| g("PERIODS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_f64().unwrap())).collect());
static SAID_A_RATE: LazyLock<&'static Re> = LazyLock::new(|| gre("SAID_A_RATE"));
static WORKED: LazyLock<&'static Re> = LazyLock::new(|| gre("WORKED"));
static _TENS_WORDS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("_TENS_WORDS"));
static _WORD_THOUSANDS: LazyLock<&'static Re> = LazyLock::new(|| gre("_WORD_THOUSANDS"));
static _NUMBER_THOUSANDS: LazyLock<&'static Re> = LazyLock::new(|| gre("_NUMBER_THOUSANDS"));
static _K_THOUSANDS_HERE: LazyLock<&'static Re> = LazyLock::new(|| gre("_K_THOUSANDS_HERE"));
static _MONEY_HERE: LazyLock<&'static Re> = LazyLock::new(|| gre("_MONEY_HERE"));

fn period_months(period: &str) -> f64 {
    lookup(&PERIODS, period).unwrap_or_else(|| panic!("KeyError: {period:?}"))
}

/// The part of the line the rate sits in, so its direction can be read on its own.
pub fn _rate_clause(line: &str, found: &Match) -> String {
    let (m_start, m_end) = (text::char_index(line, found.start()) as i64, text::char_index(line, found.end()) as i64);
    let mut start: i64 = 0;
    for part in CLAUSE_SPLIT.split(line) {
        let end = start + part.chars().count() as i64;
        if start <= m_start && m_end <= end + 1 {
            return part;
        }
        start = end + 1;
    }
    line.to_string()
}

/// What the user said on the last few turns, oldest first.
pub fn _user_lines(memory: &Memory, back: usize) -> Vec<String> {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(back);
    turns[start..].iter().filter(|t| !text::strip(&t.user).is_empty()).map(|t| squash(&t.user.to_lowercase())).collect()
}

/// True when one of these lines puts money by at a rate ("put away 250 a month").
pub fn _a_rate_was_said(lines: &[String]) -> bool {
    for line in lines {
        for found in RATE.finditer(line) {
            let clause = _rate_clause(line, &found);
            if SAVING_CLAUSE.is_match(&clause) && !SPENDING_CLAUSE.is_match(&clause) {
                return true;
            }
        }
    }
    false
}

/// The most recent rate per period in the chat, when the money is going towards something.
pub fn _rate_in_chat(memory: &Memory) -> Option<(f64, String)> {
    let lines = _user_lines(memory, 8);
    for index in (0..lines.len()).rev() {
        let line = &lines[index];
        let put_right = dup::a_rate_put_right(line, if index > 0 { &lines[index - 1] } else { "" });
        if put_right != 0 && _a_rate_was_said(&lines[..index]) {
            return Some((put_right as f64, "month".into()));
        }
        for found in RATE.finditer(line) {
            let clause = _rate_clause(line, &found);
            if !SAVING_CLAUSE.is_match(&clause) || SPENDING_CLAUSE.is_match(&clause) {
                continue;
            }
            let amount = py_float(&found.name_str("amount").replace(',', ""));
            if amount > 0.0 {
                return Some((amount, found.name_str("period").to_string()));
            }
        }
    }
    None
}

/// The total this chat has already worked a time for, read from the bot's own answer.
pub fn _worked_in_chat(memory: &Memory, rate: Option<f64>) -> Option<f64> {
    let mut total = None;
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(8);
    for turn in &turns[start..] {
        let said = squash(&turn.bot);
        for found in WORKED.finditer(&said) {
            let raw = [found.get(1), found.get(2)].into_iter().flatten().find(|p| !p.is_empty()).unwrap_or("");
            let Some(value) = int_or_none(&raw.replace(',', "")).and_then(|v| v.to_i128()) else { continue };
            let value = value as f64;
            if let Some(r) = rate.filter(|r| *r != 0.0) {
                if value < r * 2.0 {
                    continue;
                }
            }
            if value >= 50.0 {
                total = Some(value);
            }
        }
    }
    total
}

/// `str(int(float(x) * 1000))`.
fn thousand(raw: &str) -> String {
    int_str(trunc(py_float(raw) * 1000.0))
}

/// The sums in a line, with a scale word read as thousands: "20 grand" is 20,000.
pub fn _sums(text_in: &str) -> Vec<f64> {
    let mut t = _WORD_THOUSANDS
        .sub_fn(text_in, |m| (lookup(&_TENS_WORDS, &m.str(1).to_lowercase()).unwrap() * 1000).to_string());
    t = _NUMBER_THOUSANDS.sub_fn(&t, |m| thousand(m.str(1)));
    if _MONEY_HERE.is_match(&t) {
        t = _K_THOUSANDS_HERE.sub_fn(&t, |m| thousand(m.str(1)));
    }
    _parse(&t)
}

static GOAL: LazyLock<&'static Re> = LazyLock::new(|| gre("GOAL"));

/// The biggest figure earlier in the chat that is a goal rather than a bill or a wage.
pub fn _goal_in_chat(memory: &Memory, rate: Option<f64>) -> Option<f64> {
    let lines = _user_lines(memory, 8);
    let mut total: Option<f64> = None;
    let a_goal_is_named = rate.is_some() && lines.iter().any(|l| GOAL.is_match(l));
    let mut owed: Option<f64> = None;
    let rate_t = rate.filter(|r| *r != 0.0);
    for line in &lines {
        if !GOAL.is_match(line) && !a_goal_is_named {
            continue;
        }
        if !GOAL.is_match(line) && SPENDING_CLAUSE.is_match(line) {
            continue;
        }
        if dup::the_pot_said_alone(line) != 0 {
            continue;
        }
        if OWED.is_match(line) && !A_DEBT_CLEARED.is_match(line) {
            for value in _sums(&SAID_A_RATE.sub(" ", line)) {
                if rate_t.is_some_and(|r| value < r * 2.0) {
                    continue;
                }
                if value >= 50.0 && owed.is_none_or(|o| value > o) {
                    owed = Some(value);
                }
            }
            continue;
        }
        for value in _sums(&SAID_A_RATE.sub(" ", line)) {
            if rate_t.is_some_and(|r| value < r * 2.0) {
                continue;
            }
            if value >= 50.0 && total.is_none_or(|t| value > t) {
                total = Some(value);
            }
        }
    }
    if total.is_none() {
        total = _worked_in_chat(memory, rate);
    }
    total.or(owed)
}

/// A rate per period and a total to reach, both stated in the chat.
pub fn _rate_and_total(memory: &Memory) -> (Option<f64>, Option<String>, Option<f64>) {
    let Some((rate, period)) = _rate_in_chat(memory) else { return (None, None, None) };
    let mut total = _goal_in_chat(memory, Some(rate));
    let held = dup::the_money_already_held(memory) as f64;
    if let Some(t) = total.filter(|t| *t != 0.0) {
        if 0.0 < held && held < t {
            total = Some(t - held);
        }
    }
    (Some(rate), Some(period), total)
}

/// A count of one period said in months, for the parenthesis in the reply.
pub fn _in_months(count: f64, period: &str) -> f64 {
    count * period_months(period)
}

/// The sentence for "X a month for Y": the count, and how long that is in years.
pub fn _duration_reply(rate: f64, period: &str, total: f64) -> Option<String> {
    let count = total / rate;
    if count < 1.0 || count > 1200.0 {
        return None;
    }
    let nearest = trunc(count + 0.5);
    let (said_count, about) = if (count - nearest).abs() < 0.05 {
        (_number(nearest), "")
    } else {
        (_number(if count < 3.0 { round_n(count, 1) } else { nearest }), "about ")
    };
    let name = if count == 1.0 { period.to_string() } else { format!("{period}s") };
    let mut reply = format!("At {} a {period}, {} takes {about}{said_count} {name}", _number(rate), _number(total));
    let months = _in_months(count, period);
    if period != "year" && months >= 12.0 {
        let whole = round0(months) as i64;
        let (years, rest) = (whole.div_euclid(12), whole.rem_euclid(12));
        let mut said = format!("{years} year{}", if years == 1 { "" } else { "s" });
        if rest != 0 {
            said += &format!(" and {rest} month{}", if rest == 1 { "" } else { "s" });
        }
        let loose = if !about.is_empty() || (months - round0(months)).abs() >= 0.05 { "about " } else { "" };
        reply += &format!(" ({loose}{said})");
    }
    Some(reply + ".")
}

/// How long it takes to reach a total at a rate, when the chat said both and the line asks.
pub fn _time_to_reach(clean: &str, memory: &Memory) -> Option<String> {
    if !TIME_ASKED.is_match(clean) || TIME_ELSEWHERE.is_match(clean) {
        return None;
    }
    if !_parse(clean).is_empty() {
        return None;
    }
    let (rate, period, total) = _rate_and_total(memory);
    let rate = rate.filter(|r| *r != 0.0)?;
    let total = total.filter(|t| *t != 0.0)?;
    _duration_reply(rate, &period.unwrap(), total)
}

/// A line that proposes its own numbers for the sum the chat already holds.
pub fn _asks_a_new_sum(clean: &str) -> bool {
    if WHAT_IF.is_match(clean) {
        return true;
    }
    ABOUT_ASK.is_match(clean) && !CLOCK.is_match(clean)
}

/// The rate and period the bot worked out on the turn before, when it worked one out.
pub fn _last_takes(memory: &Memory) -> Option<(f64, String)> {
    let last = memory.turns.last()?;
    let found = TAKES.search(&last.bot)?;
    let period = found.name_str("period").trim_end_matches('s').to_string();
    Some((py_float(&found.name_str("amount").replace(',', "")), period))
}

/// A rate the line itself proposes for the same total: "what if we do 700?"
pub fn _time_at_a_new_rate(clean: &str, memory: &Memory) -> Option<String> {
    if !_asks_a_new_sum(clean) || TIME_ELSEWHERE.is_match(clean) {
        return None;
    }
    let before = _last_takes(memory)?;
    let (_rate, _period, total) = _rate_and_total(memory);
    let total = total.filter(|t| *t != 0.0)?;
    let found = RATE.search(clean);
    let numbers = _parse(clean);
    let (rate, period) = if let Some(m) = found {
        (py_float(&m.name_str("amount").replace(',', "")), m.name_str("period").to_string())
    } else if numbers.len() == 1 && numbers[0] < total {
        if A_HORIZON.is_match(clean) {
            return None;
        }
        (numbers[0], before.1.clone())
    } else {
        return None;
    };
    if rate == 0.0 || period.is_empty() {
        return None;
    }
    _duration_reply(rate, &period, total)
}

/// A bigger price for the same saving rate: "and what if i find one for 6000, is that daft?"
pub fn _time_at_a_new_total(clean: &str, memory: &Memory) -> Option<String> {
    if !_asks_a_new_sum(clean) || TIME_ELSEWHERE.is_match(clean) {
        return None;
    }
    _last_takes(memory)?;
    let (rate, period, total) = _rate_and_total(memory);
    let rate = rate.filter(|r| *r != 0.0)?;
    let total = total.filter(|t| *t != 0.0)?;
    let period = period.unwrap();
    let numbers = _parse(clean);
    if numbers.len() != 1 || numbers[0] <= total {
        return None;
    }
    let dearer = numbers[0];
    let count = dearer / rate;
    let old = total / rate;
    let nearest = round0(count);
    let said = _number(if (count - nearest).abs() < 0.05 { nearest } else { round_n(count, 1) });
    let name = if count == 1.0 { period.clone() } else { format!("{period}s") };
    let extra = round0(count - old);
    let more = if extra >= 1.0 {
        format!(", so {} {name} more than {}", _number(extra), _number(total))
    } else {
        String::new()
    };
    Some(format!(
        "At {} a {period}, {} takes {said} {name}{more}. Only you know if the dearer one is worth the wait.",
        _number(rate),
        _number(dearer)
    ))
}

/// A rate without a figure to reach yet: "how long would that take?"
pub fn _time_without_a_goal(clean: &str, memory: &Memory) -> Option<String> {
    if !TIME_ASKED.is_match(clean) || TIME_ELSEWHERE.is_match(clean) || !_parse(clean).is_empty() {
        return None;
    }
    let (rate, period) = _rate_in_chat(memory)?;
    if rate == 0.0 {
        return None;
    }
    if _goal_in_chat(memory, Some(rate)).is_some_and(|t| t != 0.0) {
        return None;
    }
    let yearly = rate * 12.0 / period_months(&period);
    let mut said = format!("At {} a {period} that's about {} a year", _number(rate), _number(round0(yearly)));
    if let Some(each_hundred) = _each_hundred(rate, &period) {
        said += &format!(", so every 100 of the price takes {each_hundred}");
    }
    Some(said + ". Tell me the figure you are saving towards and I will work out how long it takes.")
}

static _SMALLER_PERIOD: LazyLock<Vec<(String, (String, f64))>> = LazyLock::new(|| {
    g("_SMALLER_PERIOD")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].as_f64().unwrap())))
        .collect()
});

fn smaller_period(period: &str) -> Option<(String, f64)> {
    _SMALLER_PERIOD.iter().find(|(k, _)| k == period).map(|(_, v)| v.clone())
}

/// How long 100 of the price takes at a rate: "five weeks", "eight days", "2.5 months".
pub fn _each_hundred(rate: f64, period: &str) -> Option<String> {
    if rate == 0.0 || rate <= 0.0 {
        return None;
    }
    let mut period = period.to_string();
    let mut count = 100.0 / rate;
    while count < 1.0 {
        let Some((p, many)) = smaller_period(&period) else { break };
        period = p;
        count *= many;
    }
    if count < 1.0 {
        return None;
    }
    while count >= 13.0 && matches!(period.as_str(), "day" | "week" | "fortnight") {
        let (p, many) = smaller_period(&period).unwrap();
        period = p;
        count /= many;
    }
    if period == "hour" && count >= 12.0 {
        return None;
    }
    let (value, said);
    if count < 3.0 {
        value = round0(count * 2.0) / 2.0;
        said = _number(if value == trunc(value) { trunc(value) } else { value });
    } else {
        value = round0(count);
        let s = _number(value);
        said = if (count - value).abs() >= 0.5 { format!("about {s}") } else { s };
    }
    let name = if (value - 1.0).abs() < 0.05 { period.clone() } else { format!("{period}s") };
    Some(format!("{said} {name}"))
}

static MONEY_GOAL: LazyLock<&'static Re> = LazyLock::new(|| gre("MONEY_GOAL"));

/// The biggest figure on a line that says money is being put by.
pub fn _a_saving_goal_in_chat(memory: &Memory) -> Option<f64> {
    let mut best: Option<f64> = None;
    for line in _user_lines(memory, 8) {
        if !MONEY_GOAL.is_match(&line) || SPENDING_CLAUSE.is_match(&line) {
            continue;
        }
        for value in _sums(&SAID_A_RATE.sub(" ", &line)) {
            if value >= 50.0 && best.is_none_or(|b| value > b) {
                best = Some(value);
            }
        }
    }
    best
}

/// Python `str(value)` for a plan the store keeps (a string, mostly).
fn py_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::None => "None".into(),
        Value::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        other => format!("{other:?}"),
    }
}

/// The figure the newest plan names, if that plan is about money.
pub fn _the_plan_figure(memory: &Memory) -> Option<f64> {
    let user = memory.extra.get("user").filter(|u| u.truthy())?;
    if !matches!(user, Value::Dict(_)) {
        return None;
    }
    let plans: Vec<&Value> =
        user.get("plans").map(|p| p.items().iter().filter(|x| x.truthy()).collect()).unwrap_or_default();
    let newest = match plans.last() {
        Some(p) => Some(*p),
        None => user.get("plan"),
    };
    let newest = newest.filter(|n| n.truthy())?;
    let said = py_str(newest);
    if !MONEY_GOAL.is_match(&said.to_lowercase()) {
        return None;
    }
    let sums: Vec<f64> = _sums(&said).into_iter().filter(|v| *v >= 50.0).collect();
    sums.into_iter().reduce(|a, b| if b > a { b } else { a })
}

/// A figure to reach but no rate to reach it at: "how long will it take".
pub fn _a_goal_without_a_rate(clean: &str, memory: &mut Memory) -> Option<String> {
    if !TIME_ASKED.is_match(clean) || TIME_ELSEWHERE.is_match(clean) || !_parse(clean).is_empty() {
        return None;
    }
    if _rate_in_chat(memory).is_some_and(|(r, _)| r != 0.0) {
        return None;
    }
    let mut total = _the_plan_figure(memory);
    if total.is_none() {
        if let Some(user) = memory.extra.get("user").filter(|u| u.truthy()) {
            if matches!(user, Value::Dict(_))
                && (user.get("plan").is_some_and(|p| p.truthy()) || user.get("plans").is_some_and(|p| p.truthy()))
            {
                return None;
            }
        }
        total = _a_saving_goal_in_chat(memory);
    }
    let total = total.filter(|t| *t != 0.0)?;
    let reading = Understood { clean: clean.to_string(), text: clean.to_string(), ..Default::default() };
    if let Some(saved) = crate::knowledge::money::_time_to_save(memory, &reading).filter(|s| !s.is_empty()) {
        return Some(saved);
    }
    Some(format!("Getting to {} depends on what you can put by - {}", _number(total), gstr("ASKED_FOR_RATE")))
}

static JUST_A_TOTAL: LazyLock<&'static Re> = LazyLock::new(|| gre("JUST_A_TOTAL"));

/// True when the whole line is a swap of the total: "say i need 12000".
pub fn _a_new_total_only(clean: &str) -> bool {
    JUST_A_TOTAL.is_match(text::strip(clean))
}

/// The total, given on the next line: "say i need 12000".
pub fn _figure_just_given(clean: &str, memory: &Memory) -> Option<String> {
    let last = memory.turns.last()?;
    if TIME_ELSEWHERE.is_match(clean) {
        return None;
    }
    let said = &last.bot;
    if !said.contains(gstr("ASKED_FOR_FIGURE")) && !(_a_new_total_only(clean) && WORKED.is_match(&squash(said))) {
        return None;
    }
    let numbers = _parse(clean);
    let (rate, period) = _rate_in_chat(memory).unwrap_or((0.0, String::new()));
    if numbers.len() != 1 || rate == 0.0 || numbers[0] < rate * 2.0 {
        return None;
    }
    _duration_reply(rate, &period, numbers[0])
}

static HOW_MUCH_EACH: LazyLock<&'static Re> = LazyLock::new(|| gre("HOW_MUCH_EACH"));
static A_BARE_RATE_ASK: LazyLock<&'static Re> = LazyLock::new(|| gre("A_BARE_RATE_ASK"));
static EACH_PERIOD: LazyLock<&'static Re> = LazyLock::new(|| gre("EACH_PERIOD"));
static PERIODS_IN_A_YEAR: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("PERIODS_IN_A_YEAR"));
static DEADLINE_NAMED: LazyLock<&'static Re> = LazyLock::new(|| gre("DEADLINE_NAMED"));

/// A figure to reach but no date: how much to put by each month.
pub fn _each_month_for_a_goal(clean: &str, memory: &Memory) -> Option<String> {
    let bare = A_BARE_RATE_ASK.is_match(clean);
    if !(HOW_MUCH_EACH.is_match(clean) || bare) {
        return None;
    }
    if !_parse(clean).is_empty() || TIME_ELSEWHERE.is_match(clean) {
        return None;
    }
    let total = _goal_in_chat(memory, None).filter(|t| *t != 0.0)?;
    if DEADLINE_NAMED.is_match(clean) || _a_deadline_in_chat(memory) {
        let dated = _a_month_for_the_deadline_in_chat(total, memory)?;
        if dated.is_empty() {
            return None;
        }
        return Some(dated);
    }
    let period = _asked_period(clean);
    let per_year = lookup(&PERIODS_IN_A_YEAR, &period).unwrap_or_else(|| panic!("KeyError")) as f64;
    let (year, two) = (total / per_year, total / (per_year * 2.0));
    Some(format!(
        "Saving {} over a year is about {} a {period}; over two years, about {} a {period}.",
        _number(total),
        _number(round0(year)),
        _number(round0(two))
    ))
}

static HORIZON_SAID: LazyLock<&'static Re> = LazyLock::new(|| gre("HORIZON_SAID"));
static HORIZON_COUNTS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("HORIZON_COUNTS"));
static PAST_REPORT: LazyLock<&'static Re> = LazyLock::new(|| gre("PAST_REPORT"));
static DEADLINE_DAYS: LazyLock<Vec<(String, Option<(i64, i64)>)>> = LazyLock::new(|| {
    g("DEADLINE_DAYS")
        .pairs()
        .iter()
        .map(|(k, v)| {
            let md =
                if v.is_none() { None } else { Some((v.items()[0].as_int().unwrap(), v.items()[1].as_int().unwrap())) };
            (k.str().to_string(), md)
        })
        .collect()
});
static MONTH_NUMBERS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("MONTH_NUMBERS"));
static K_THOUSANDS: LazyLock<&'static Re> = LazyLock::new(|| gre("K_THOUSANDS"));
static A_DATE_FOR_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| gre("A_DATE_FOR_THE_THING"));
static SAVE_VERB: LazyLock<&'static Re> = LazyLock::new(|| gre("SAVE_VERB"));
static SAVING_CUE: LazyLock<&'static Re> = LazyLock::new(|| gre("SAVING_CUE"));
static A_DEADLINE_DAY: LazyLock<&'static Re> = LazyLock::new(|| gre("A_DEADLINE_DAY"));

/// The figure in a line that writes 10k, read as 10,000 when the line is about money.
pub fn _k_as_money(clean: &str) -> Option<f64> {
    let found = K_THOUSANDS.search(clean)?;
    if !SAVING_CUE.is_match(clean) {
        return None;
    }
    let amount = py_float(found.str(1)) * 1000.0;
    if amount >= 100.0 { Some(amount) } else { None }
}

/// The date of Easter Sunday (month, day).
pub fn _easter_day(year: i64) -> (i64, i64) {
    let (a, b, c) = (year.rem_euclid(19), year.div_euclid(100), year.rem_euclid(100));
    let (d, e) = (b.div_euclid(4), b.rem_euclid(4));
    let f = (b + 8).div_euclid(25);
    let g = (b - f + 1).div_euclid(3);
    let h = (19 * a + b - d - g + 15).rem_euclid(30);
    let (i, k) = (c.div_euclid(4), c.rem_euclid(4));
    let l = (32 + 2 * e + 2 * i - h - k).rem_euclid(7);
    let m = (a + 11 * h + 22 * l).div_euclid(451);
    let month = (h + l - 7 * m + 114).div_euclid(31);
    let day = (h + l - 7 * m + 114).rem_euclid(31) + 1;
    (month, day)
}

/// The day the month ends on: 31 for December, 30 for June, 28 or 29 for February.
pub fn _the_last_day_of(month: i64, today: Date) -> i64 {
    let year = today.year as i64 + if month < today.month as i64 { 1 } else { 0 };
    days_in_month(year as i32, month as u32) as i64
}

/// The date a line names for the money already in it: ("christmas", date) or None.
pub fn _a_named_deadline(clean: &str, today: Option<Date>) -> Option<(String, Date)> {
    let today = today.unwrap_or_else(self::today);
    let found = DEADLINE_NAMED.search(clean)?;
    let said = found.as_str().to_lowercase();
    let mut name = text::split(&said).last().cloned().unwrap_or_default();
    let a_day = A_DEADLINE_DAY.search(&said);
    for phrase in [
        "the end of next month",
        "the end of this month",
        "the end of next week",
        "the end of this week",
        "the end of next year",
        "the end of this year",
        "the end of the year",
        "the end of the month",
        "the end of the week",
        "next summer",
        "next christmas",
        "next year",
        "next month",
        "next week",
        "the summer",
        "the winter",
    ] {
        if said.contains(phrase) {
            name = phrase.to_string();
            break;
        }
    }
    let (month, day): (i64, i64);
    if let Some(ad) = a_day.as_ref().filter(|ad| lookup(&MONTH_NUMBERS, ad.str(2)).is_some()) {
        month = lookup(&MONTH_NUMBERS, ad.str(2)).unwrap();
        day = py_int(ad.str(1)) as i64;
        name = format!("the {}", ad.as_str().to_lowercase());
        if !(1..=31).contains(&day) {
            return None;
        }
    } else if let Some(mn) = lookup(&MONTH_NUMBERS, &name) {
        month = mn;
        let before = chars_before(clean, found.start(), 12);
        let at_the_end = format!("{said} {before}").contains("end of");
        day = if at_the_end { _the_last_day_of(month, today) } else { 1 };
        if at_the_end {
            name = format!("the end of {name}");
        }
    } else if name == "next summer" || name == "next christmas" {
        let key = name.replace("next ", "");
        let md = DEADLINE_DAYS.iter().find(|(k, _)| *k == key).and_then(|(_, v)| *v).unwrap_or((7, 1));
        month = md.0;
        day = md.1;
    } else if name == "easter" {
        (month, day) = _easter_day(today.year as i64);
    } else if let Some(md) = DEADLINE_DAYS.iter().find(|(k, _)| *k == name).and_then(|(_, v)| *v) {
        (month, day) = md;
    } else if name == "next month" {
        let year = today.year as i64 + (today.month == 12) as i64;
        let month = today.month as i64 % 12 + 1;
        return Some((name, Date::of(year, month, 1)));
    } else if name == "next week" {
        return Some((name, today.plus(7)));
    } else if name == "the end of the month" {
        let last = _the_last_day_of(today.month as i64, today);
        return Some((name, Date::of(today.year as i64, today.month as i64, last)));
    } else if name == "the end of the week" {
        return Some((name, today.plus(7 - today.weekday() as i64)));
    } else if name == "the end of next month" || name == "the end of this month" {
        let step = if name.contains("next") { 1 } else { 0 };
        let m = today.month as i64 + step;
        let (year, m) = (today.year as i64 + (m - 1).div_euclid(12), (m - 1).rem_euclid(12) + 1);
        return Some((name, Date::of(year, m, days_in_month(year as i32, m as u32) as i64)));
    } else if matches!(name.as_str(), "the end of next year" | "the end of this year" | "the end of the year") {
        let year = today.year as i64 + if name.contains("next") { 1 } else { 0 };
        return Some((name, Date::of(year, 12, 31)));
    } else if name == "the end of next week" || name == "the end of this week" {
        let extra = if name.contains("next") { 7 } else { 0 };
        return Some((name, today.plus(7 - today.weekday() as i64 + extra)));
    } else {
        return None;
    }
    let mut when = Date::new(today.year as i64, month, day)?;
    if when <= today {
        when = Date::new(today.year as i64 + 1, month, day)?;
    }
    Some((name, when))
}

/// A goal and a named date: what has to go by each month, and each week when it is close.
pub fn _save_by_a_date(clean: &str, memory: &Memory) -> Option<String> {
    let today = today();
    let (name, when) = _a_named_deadline(clean, Some(today))?;
    if PAST_REPORT.is_match(clean) {
        return None;
    }
    let mut total = _k_as_money(clean);
    if total.is_none_or(|t| t == 0.0) {
        let numbers: Vec<f64> = _parse(clean).into_iter().filter(|n| *n >= 50.0).collect();
        total = if numbers.len() == 1 { Some(numbers[0]) } else { None };
    }
    if total.is_none_or(|t| t == 0.0) {
        if !SAVE_VERB.is_match(clean) && !A_DATE_FOR_THE_THING.is_match(clean) {
            return None;
        }
        total = _goal_in_chat(memory, None);
    }
    let total = total.filter(|t| *t != 0.0)?;
    _a_total_by_a_date(total, &name, when, Some(today))
}

/// A total and a date: the figure that comes out of each pay packet, or None.
pub fn _a_total_by_a_date(total: f64, name: &str, when: Date, today: Option<Date>) -> Option<String> {
    let today = today.unwrap_or_else(self::today);
    let days = when.minus(&today);
    if days < 1 {
        return None;
    }
    let months = days as f64 / 30.44;
    let mut said;
    if months < 1.2 {
        said = format!(
            "\u{a3}{} by {name} is {days} days away, so it is one pay packet: put the \u{a3}{} aside on payday before anything else",
            _number(total),
            _number(round0(total))
        );
    } else {
        said = format!(
            "\u{a3}{} by {name} is {days} days away, so about \u{a3}{} a month",
            _number(total),
            _number(round0(total / months))
        );
        if days <= 120 {
            let weeks = -((-days).div_euclid(7));
            let whole = round0(total) as i128;
            let a_week = -((-whole).div_euclid(weeks as i128));
            said += &format!(
                " - about \u{a3}{} a week over the {weeks} weeks left, which is the number that actually comes out of a pay packet",
                _number_int(a_week)
            );
        }
    }
    Some(said + ". Take it off the top on payday rather than saving what is left at the end.")
}

/// The figure to the date the chat named: "how much a month" after a goal with a deadline.
pub fn _a_month_for_the_deadline_in_chat(total: f64, memory: &Memory) -> Option<String> {
    let today = today();
    for line in _user_lines(memory, 8).iter().rev() {
        let Some((name, when)) = _a_named_deadline(line, Some(today)) else { continue };
        if PAST_REPORT.is_match(line) {
            return None;
        }
        return _a_total_by_a_date(total, &name, when, Some(today));
    }
    None
}

/// A date on its own for a figure already in the chat: "i want it in a year".
pub fn _deadline_for_a_goal(clean: &str, memory: &Memory) -> Option<String> {
    let said = HORIZON_SAID.search(clean);
    if said.is_none() || TIME_ELSEWHERE.is_match(clean) || DEADLINE_NAMED.is_match(clean) {
        return if !DEADLINE_NAMED.is_match(clean) { None } else { _save_by_a_date(clean, memory) };
    }
    let said = said.unwrap();
    let raw = said.name_str("count");
    let count: f64 = if isdigit(raw) {
        py_float(raw)
    } else {
        lookup(&HORIZON_COUNTS, &squash(raw)).map(|v| v as f64).unwrap_or(0.0)
    };
    if count == 0.0 || RATE.is_match(clean) {
        return None;
    }
    let rest = text::strip(&format!("{} {}", &clean[..said.start()], &clean[said.end()..])).to_string();
    let mut numbers = _parse(&rest);
    if let Some(short_hand) = _k_as_money(clean).filter(|s| *s != 0.0) {
        numbers = vec![short_hand];
    }
    let total = if !numbers.is_empty() {
        if numbers.len() != 1 || !GOAL.is_match(&rest) || numbers[0] < 50.0 {
            return None;
        }
        if PAST_REPORT.is_match(&rest) {
            return None;
        }
        Some(numbers[0])
    } else {
        _goal_in_chat(memory, None)
    };
    let total = total.filter(|t| *t != 0.0)?;
    let unit = said.name_str("unit").trim_end_matches('s').to_string();
    let months = count * period_months(&unit);
    if !(1.0..=600.0).contains(&months) {
        return None;
    }
    let per = total / months;
    if per < 1.0 {
        return None;
    }
    let when = if count == 1.0 { format!("a {unit}") } else { format!("{} {unit}s", _number(count)) };
    Some(format!(
        "{} in {when} is {} a month. Set the transfer for the day after payday so the money goes before you can spend it.",
        _number(total),
        _number(round0(per))
    ))
}

/// The period the line asks about, month when it names none.
pub fn _asked_period(clean: &str) -> String {
    EACH_PERIOD.search(clean).map(|m| m.str(1).to_string()).unwrap_or_else(|| "month".into())
}

/// Whether the chat named a deadline ("by december") that a per-month figure would ignore.
pub fn _a_deadline_in_chat(memory: &Memory) -> bool {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(4);
    turns[start..].iter().any(|t| DEADLINE_NAMED.is_match(&t.user))
}

/// A monthly or weekly amount from "save 480 pounds in 6 months".
pub fn _saving(clean: &str) -> Option<String> {
    if !["need", "want", "should", "how much", "each", "per", "plan"].iter().any(|w| clean.contains(w)) {
        return None;
    }
    let amount = search(r"(?:save|saving|saved)\s+(?:up\s+)?[£$€]?\s*([\d,.]+)", clean);
    let when = search(r"(?:in|over|within)\s+(\d+)\s*(month|week|year)s?", clean);
    let (amount, when) = (amount?, when?);
    let total = py_float(&amount.str(1).replace(',', ""));
    let mut count = py_int(when.str(1));
    let mut unit = when.str(2).to_string();
    if unit == "year" {
        count *= 12;
        unit = "month".into();
    }
    if count < 1 {
        return None;
    }
    let per = total / count as f64;
    Some(format!("Saving {} over {count} {unit}s is {} a {unit}.", _number(total), _number(per)))
}

/// Python 3.12's `sum()` of floats (Neumaier's compensated sum).
pub fn py_sum(values: &[f64]) -> f64 {
    let mut f = 0.0f64;
    let mut c = 0.0f64;
    for &x in values {
        let t = f + x;
        if f.abs() >= x.abs() {
            c += (f - t) + x;
        } else {
            c += (x - t) + f;
        }
        f = t;
    }
    if c != 0.0 && c.is_finite() {
        f += c;
    }
    f
}

pub fn _average(clean: &str) -> Option<String> {
    let m = search(r"average(?: of)?\s+(.+)", clean)?;
    let numbers = _parse(m.str(1));
    if numbers.len() < 2 || ["speed", "month", "year"].iter().any(|w| m.str(1).contains(w)) {
        return None;
    }
    let mean = py_sum(&numbers) / numbers.len() as f64;
    Some(format!("The average is {}.", _number(mean)))
}

// --- the size of a floor, a wall, a garden ---------------------------------------------------

static _LENGTH_UNIT: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("_LENGTH_UNIT").str_map());
static _SIDE_BY_SIDE: LazyLock<&'static Re> = LazyLock::new(|| gre("_SIDE_BY_SIDE"));
static _SQUARE_WORD: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("_SQUARE_WORD").str_map());
static _AREA_ASK: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_AREA_ASK"));
static _PERIMETER_ASK: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_PERIMETER_ASK"));
static _SHAPE: LazyLock<&'static Re> = LazyLock::new(|| gre("_SHAPE"));
static _AREA_TARGET: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("_AREA_TARGET").items().iter().map(|t| (t.items()[0].str().to_string(), t.items()[1].str().to_string())).collect()
});
static _METRES_SQUARED: LazyLock<Vec<(String, f64)>> = LazyLock::new(|| {
    g("_METRES_SQUARED").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_f64().unwrap())).collect()
});
static _SURFACES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("_SURFACES").items().iter().map(|t| (t.items()[0].str().to_string(), t.items()[1].str().to_string())).collect()
});
static _COUNTED: LazyLock<&'static Re> = LazyLock::new(|| gre("_COUNTED"));
static _BOUGHT_BY_THE_BAG: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_BOUGHT_BY_THE_BAG"));
static _BUYING: LazyLock<&'static Re> = LazyLock::new(|| gre("_BUYING"));

/// The side lengths named in a line: (sizes as (value, unit)), and the unit named, if any.
pub fn _sides(clean: &str) -> Option<(Vec<(f64, Option<String>)>, Option<String>)> {
    let found = _SIDE_BY_SIDE.search(clean)?;
    let mut sizes: Vec<(f64, Option<String>)> = Vec::new();
    for (value_at, unit_at) in [(1, 2), (3, 4), (5, 6)] {
        let Some(raw) = found.get(value_at) else { continue };
        let unit = _LENGTH_UNIT.get(found.get(unit_at).unwrap_or("")).cloned();
        sizes.push((py_float(raw), unit));
    }
    if sizes.len() < 2 {
        return None;
    }
    let named = sizes.iter().find_map(|(_, u)| u.clone().filter(|u| !u.is_empty()));
    let Some(named) = named else { return Some((sizes, None)) };
    Some((
        sizes
            .into_iter()
            .map(|(v, u)| (v, Some(u.filter(|u| !u.is_empty()).unwrap_or_else(|| named.clone()))))
            .collect(),
        Some(named),
    ))
}

/// The word for the thing whose size was asked, or "floor" when the line names none.
pub fn _surface(clean: &str) -> String {
    for (word, label) in _SURFACES.iter() {
        if has(&format!(r"\b{word}\b"), clean) {
            return label.clone();
        }
    }
    "floor".into()
}

pub fn _square_text(value: f64, unit: &str) -> String {
    let word = _SQUARE_WORD.get(unit).map(String::as_str).unwrap_or("square units");
    format!("{} {word}", _number(round_n(value, 2)))
}

/// "a 4m by 3m room": the two sides of a surface, multiplied.
pub fn _floor_area(clean: &str) -> Option<String> {
    if !any_in(clean, &_AREA_ASK) {
        return None;
    }
    if _BUYING.is_match(clean) && any_in(clean, &_BOUGHT_BY_THE_BAG) {
        return None;
    }
    if _COUNTED.is_match(clean) || _SHAPE.is_match(clean) {
        return None;
    }
    let (sizes, unit) = _sides(clean)?;
    let area = sizes[0].0 * sizes[1].0;
    let thing = _surface(clean);
    let Some(unit) = unit else {
        return Some(format!(
            "A {} by {} {thing} is {} square metres if those are metres, {} square feet if they are feet.",
            _number(sizes[0].0),
            _number(sizes[1].0),
            _number(round_n(area, 2)),
            _number(round_n(area, 2))
        ));
    };
    let sides = sizes[..2].iter().map(|(v, _)| format!("{}{unit}", _number(*v))).collect::<Vec<_>>().join(" by ");
    let target = _AREA_TARGET
        .iter()
        .find(|(w, _)| clean.contains(w.as_str()))
        .map(|(_, t)| t.clone())
        .unwrap_or_else(|| unit.clone());
    let sq = |u: &str| lookup(&_METRES_SQUARED, u);
    if target != unit {
        if let (Some(t), Some(u)) = (sq(&target), sq(&unit)) {
            let wanted = round_n(area * u / t, 2);
            return Some(format!(
                "A {sides} {thing} is {}, which is {}. That is the two sides multiplied: {} x {}.",
                _square_text(wanted, &target),
                _square_text(area, &unit),
                _number(sizes[0].0),
                _number(sizes[1].0)
            ));
        }
    }
    Some(format!(
        "A {sides} {thing} is {}. That is the two sides multiplied: {} x {}.",
        _square_text(area, &unit),
        _number(sizes[0].0),
        _number(sizes[1].0)
    ))
}

/// "the skirting for a 4m by 3m room": the two sides added, then doubled.
pub fn _perimeter_of_a_room(clean: &str) -> Option<String> {
    if !any_in(clean, &_PERIMETER_ASK) {
        return None;
    }
    let (sizes, unit) = _sides(clean)?;
    let (length, width) = (sizes[0].0, sizes[1].0);
    if _SHAPE.is_match(clean) {
        return None;
    }
    if has(r"\b(?:how many|how much)\s+(?:\w+\s+){0,2}skirting\b", clean) {
        return None;
    }
    let around = 2.0 * (length + width);
    let thing = _surface(clean);
    let Some(unit) = unit else {
        return Some(format!(
            "A {} by {} {thing} has a perimeter of {} metres if those are metres: twice ({} + {}).",
            _number(length),
            _number(width),
            _number(round_n(around, 2)),
            _number(length),
            _number(width)
        ));
    };
    Some(format!(
        "A {}{unit} by {}{unit} {thing} has a perimeter of {}{unit}: twice ({} + {}). That is how much skirting, edging or border it takes.",
        _number(length),
        _number(width),
        _number(round_n(around, 2)),
        _number(length),
        _number(width)
    ))
}

pub fn _area(clean: &str) -> Option<String> {
    if let Some(sized) = _floor_area(clean) {
        return Some(sized);
    }
    if !["area", "how big", "how much space"].iter().any(|w| clean.contains(w)) {
        return None;
    }
    if clean.contains("circle") {
        if let Some(radius) = search(r"radius(?: of| is)?\s*([\d,.]+)", clean) {
            let r = py_float(&radius.str(1).replace(',', ""));
            return Some(format!(
                "A circle with radius {} has an area of {}.",
                _number(r),
                _number(3.14159265 * r * r)
            ));
        }
        if let Some(diameter) = search(r"diameter(?: of| is)?\s*([\d,.]+)", clean) {
            let r = py_float(&diameter.str(1).replace(',', "")) / 2.0;
            return Some(format!(
                "A circle with diameter {} has an area of {}.",
                _number(r * 2.0),
                _number(3.14159265 * r * r)
            ));
        }
        return Some("The area of a circle is π × radius²: square the radius, then multiply by 3.14.".into());
    }
    if clean.contains("triangle") {
        let numbers = _parse(clean);
        if numbers.len() >= 2 {
            return Some(format!("That triangle's area is {}.", _number(numbers[0] * numbers[1] / 2.0)));
        }
        return Some("The area of a triangle is ½ × base × height.".into());
    }
    if clean.contains("rectangle") || clean.contains("square") {
        let numbers = _parse(clean);
        if numbers.len() >= 2 {
            return Some(format!("That rectangle's area is {}.", _number(numbers[0] * numbers[1])));
        }
        return Some("The area of a rectangle is length × width.".into());
    }
    None
}

// --- months and the calendar -----------------------------------------------------------------

/// The year a month question names, if it names one ("this year" is the current one).
pub fn _month_year(text_in: &str) -> Option<i64> {
    let today = today();
    if has(r"\bnext year\b", text_in) {
        return Some(today.year as i64 + 1);
    }
    if has(r"\b(?:this year|this time)\b", text_in) {
        return Some(today.year as i64);
    }
    search(r"\b(?:in |of )?((?:19|20)\d\d)\b", text_in).map(|m| py_int(m.str(1)) as i64)
}

/// Whether February has 29 days in that year (the rule, not a table).
pub fn _leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// How long one month is, when that is the whole question.
pub fn _days_in_month(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    let tail = gstr("_MONTH_TAIL");
    for (month, days) in MONTHS.iter() {
        let pattern = format!(
            r"(?:how many days (?:are there )?in (?:the month of )?{month}|how many days (?:does|do) {month} have|how many days (?:do i have|have i got|are left) in {month}|{month} has how many days){tail}"
        );
        if fullmatch(&pattern, &text_).is_some() {
            let year = _month_year(&text_);
            if month == "february" {
                if let Some(year) = year {
                    let longer = _leap(year);
                    return Some(format!(
                        "February {year} has {} days: {year} is {}.",
                        if longer { 29 } else { 28 },
                        if longer { "a leap year" } else { "not a leap year" }
                    ));
                }
                return Some("February has 28 days in an ordinary year and 29 in a leap year. Leap years are divisible by 4, except centuries unless they divide by 400.".into());
            }
            if let Some(year) = year {
                return Some(format!("{} {year} has {days} days.", capitalize(month)));
            }
            return Some(format!("{} has {days} days.", capitalize(month)));
        }
    }
    let pattern = format!(
        r"(?:how many |how long )(?:days? )?(?:are there |does |is |has )?(?:in )?(?:a |one |the )?leap year(?: have| last| long| then| again)?{tail}"
    );
    if fullmatch(&pattern, &text_).is_some() {
        return Some("A leap year has 366 days: 29 February, and the rest of the year as usual. The rule is every four years, except centuries unless they divide by 400.".into());
    }
    None
}

/// The day the line asked about: "Christmas Eve" when it said eve, else Christmas.
pub fn _christmas_name(text_in: &str) -> &'static str {
    if has(r"\b(?:christmas|xmas) eve\b", text_in) { "Christmas Eve" } else { "Christmas" }
}

/// Christmas Day or Christmas Eve in `year`, whichever the line named.
pub fn _christmas_date_of(text_in: &str, year: i64) -> Date {
    Date::of(year, 12, if _christmas_name(text_in) == "Christmas Eve" { 24 } else { 25 })
}

/// The Christmas Day the line means, and how many days away it is (negative once past).
pub fn _christmas_day(text_in: &str) -> (Date, i64) {
    let today = today();
    let christmas = if has(r"\bnext (?:year|christmas)\b", text_in) {
        _christmas_date_of(text_in, today.year as i64 + 1)
    } else {
        let c = _christmas_date_of(text_in, today.year as i64);
        if c < today && !has(r"\bthis (?:year|christmas)\b", text_in) {
            _christmas_date_of(text_in, today.year as i64 + 1)
        } else {
            c
        }
    };
    (christmas, christmas.minus(&today))
}

/// How the wait reads: ahead of us, today, or already gone this year.
pub fn _wait_text(days: i64, name: &str) -> String {
    if days == 0 {
        return format!("{name} is today.");
    }
    if days < 0 {
        let ago = -days;
        return format!("{name} this year was {ago} day{} ago.", if ago == 1 { "" } else { "s" });
    }
    format!("{days} days until {name}.")
}

/// Whole months from one day to another on the calendar, and the days left over.
pub fn _months_between(start: Date, when: Date) -> (i64, i64) {
    let mut months = (when.year as i64 - start.year as i64) * 12 + (when.month as i64 - start.month as i64);
    if months > 0 && _month_on(start, months) > when {
        months -= 1;
    }
    (months, when.minus(&_month_on(start, months)))
}

/// The count in the unit the line asked for, so "in weeks" comes back in weeks.
pub fn _count_in(unit: Option<&str>, days: i64, when: Option<Date>) -> String {
    let unit = unit.filter(|u| !u.is_empty()).unwrap_or("days").to_lowercase();
    if unit.starts_with("sleep") {
        return format!("{days} sleeps");
    }
    if unit.starts_with("week") {
        let (weeks, spare) = (days.div_euclid(7), days.rem_euclid(7));
        if spare == 0 {
            return format!("{weeks} weeks");
        }
        return format!("{weeks} weeks and {spare} day{}", if spare == 1 { "" } else { "s" });
    }
    if unit.starts_with("month") {
        let (months, spare) = match when {
            Some(w) if days > 0 => _months_between(w.plus(-days), w),
            _ => (days.div_euclid(30), days.rem_euclid(30)),
        };
        let m_s = if months == 1 { "" } else { "s" };
        let d_s = if spare == 1 { "" } else { "s" };
        if months != 0 && spare != 0 {
            return format!("{months} month{m_s} and {spare} day{d_s}");
        }
        if months != 0 {
            return format!("{months} month{m_s}");
        }
        return format!("{spare} day{d_s}");
    }
    format!("{days} days")
}

/// How many of the days before Christmas are days the shops are open (Sundays left out).
pub fn _shopping_days(days: i64) -> i64 {
    let today = today();
    let sundays = (1..days).filter(|step| today.plus(*step).weekday() == 6).count() as i64;
    days - sundays
}

/// The countdown in the unit that was asked for, and for the year that was named.
pub fn _countdown_text(found: &Match, text_in: &str) -> String {
    let unit = found.name("unit2").or_else(|| found.name("unit"));
    let year = found.name("year");
    let name = _christmas_name(text_in);
    let (mut christmas, mut days) = _christmas_day(text_in);
    if let Some(year) = year.filter(|y| !y.is_empty()) {
        christmas = _christmas_date_of(text_in, py_int(year) as i64);
        days = christmas.minus(&today());
        if days < 0 {
            let ago = -days;
            return format!("{name} {year} was {ago} day{} ago.", if ago == 1 { "" } else { "s" });
        }
        if days == 0 {
            return format!("{name} {year} is today.");
        }
        return format!("{} until {name} {year}.", _count_in(unit, days, Some(christmas)));
    }
    if days < 0 {
        return _wait_text(days, name);
    }
    if days == 0 {
        return format!("{name} is today.");
    }
    if rx(r"is it\b", 0).match_(text_in).is_some() {
        return format!("Not yet - {} until {name}.", _count_in(unit, days, Some(christmas)));
    }
    if text_in.contains("shopping day") {
        return format!("{days} days until {name} - {} of them shopping days.", _shopping_days(days));
    }
    format!("{} until {name}.", _count_in(unit, days, Some(christmas)))
}

/// A countdown to Christmas, only when the line asks for one.
pub fn _days_until(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    if fullmatch(gstr("_CHRISTMAS_TODAY"), &text_).is_some() {
        let (_, days) = _christmas_day("");
        if days == 0 {
            return Some("Yes - it's Christmas Day today.".into());
        }
        return Some(format!("Not yet - {days} days until Christmas."));
    }
    if fullmatch(gstr("_CHRISTMAS_FIXED"), &text_).is_some() {
        let name = _christmas_name(&text_);
        let (when, days) = _christmas_day(&text_);
        if days == 0 {
            return Some(format!("{name} is today, 25 December, the same date every year."));
        }
        if days < 0 {
            return Some(format!(
                "{name} is always 25 December, so it is neither early nor late: it was on {} {} {}.",
                when.day,
                _MONTH_NAMES[when.month as usize - 1],
                when.year
            ));
        }
        return Some(format!("{name} is always 25 December, so it is neither early nor late: it is {days} days away."));
    }
    let tail = gstr("_CHRISTMAS_TAIL");
    if let Some(found) = fullmatch(&format!("{}{tail}", gstr("_CHRISTMAS_COUNTDOWN")), &text_) {
        return Some(_countdown_text(&found, &text_));
    }
    if let Some(guessing) = fullmatch(&format!("{}{tail}", gstr("_CHRISTMAS_WEEKDAY")), &text_) {
        return Some(_weekday_text(&text_, guessing.name("guess")));
    }
    if let Some(found) = fullmatch(&format!("{}{tail}", gstr("_CHRISTMAS_DATE")), &text_) {
        let (when, days) = _christmas_day(&text_);
        let name = _christmas_name(&text_);
        let named = if name == "Christmas Eve" { name } else { "Christmas Day" };
        if let Some(year) = found.name("year").filter(|y| !y.is_empty()) {
            return Some(format!("{named} is on {} December {year}.", when.day));
        }
        if days < 0 {
            return Some(_wait_text(days, named));
        }
        let day = if days == 0 { "today".to_string() } else { format!("on {} December, {days} days away", when.day) };
        return Some(format!("{named} is {day}."));
    }
    None
}

/// "Is christmas on a friday this year", "what day of the week is christmas".
pub fn _weekday_text(text_in: &str, guess: Option<&str>) -> String {
    let (when, days) = _christmas_day(text_in);
    let name = _christmas_name(text_in);
    let weekday = when.weekday_name();
    let date = format!("{} December {}", when.day, when.year);
    if let Some(guess) = guess.filter(|g| !g.is_empty()) {
        if guess.to_lowercase() == weekday.to_lowercase() {
            return format!("Yes - {name} is on a {weekday} this year, {date}.");
        }
        return format!("No - {name} is on a {weekday} this year, {date}, not a {}.", capitalize(guess));
    }
    format!("{name} is on a {weekday} this year, {date}. That is {days} days away.")
}

// --- Easter and the feasts that hang off it --------------------------------------------------

static _EASTER_OFFSETS: LazyLock<Vec<(String, i64, String)>> = LazyLock::new(|| {
    g("_EASTER_OFFSETS")
        .items()
        .iter()
        .map(|t| (t.items()[0].str().to_string(), t.items()[1].as_int().unwrap(), t.items()[2].str().to_string()))
        .collect()
});
static _MONTH_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_MONTH_NAMES"));

/// Easter Sunday in the Gregorian calendar.
pub fn _easter_sunday(year: i64) -> Date {
    let a = year.rem_euclid(19);
    let (b, c) = (year.div_euclid(100), year.rem_euclid(100));
    let (d, e) = (b.div_euclid(4), b.rem_euclid(4));
    let f = (b + 8).div_euclid(25);
    let g = (b - f + 1).div_euclid(3);
    let h = (19 * a + b - d - g + 15).rem_euclid(30);
    let (i, k) = (c.div_euclid(4), c.rem_euclid(4));
    let l = (32 + 2 * e + 2 * i - h - k).rem_euclid(7);
    let m = (a + 11 * h + 22 * l).div_euclid(451);
    let n = h + l - 7 * m + 114;
    Date::of(year, n.div_euclid(31), n.rem_euclid(31) + 1)
}

/// The year the line means, whether it named one itself, and today.
pub fn _feast_year(text_in: &str) -> (i64, bool, Date) {
    let today = today();
    if let Some(m) = search(r"\b(20\d{2})\b", text_in) {
        let y = py_int(m.str(1)) as i64;
        if (2000..=2100).contains(&y) {
            return (y, true, today);
        }
    }
    if has(r"\bnext (?:year|easter)\b", text_in) {
        return (today.year as i64 + 1, false, today);
    }
    (today.year as i64, false, today)
}

static _MONTHS_SAID: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_MONTHS_SAID"));
static _MONTH_SHORT: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("_MONTH_SHORT"));
static _DAY_ONLY: LazyLock<&'static Re> = LazyLock::new(|| gre("_DAY_ONLY"));

/// The month a name or short spelling stands for, or None.
pub fn _month_number(word: &str) -> Option<i64> {
    let w = text::strip(word).to_lowercase();
    if let Some(i) = _MONTHS_SAID.iter().position(|m| *m == w) {
        return Some(i as i64 + 1);
    }
    lookup(&_MONTH_SHORT, &w)
}

/// That day in that month, the next time it comes round.
pub fn _day_in_a_month(day: i64, month_number: i64, today: Date) -> Option<Date> {
    for year in today.year as i64..today.year as i64 + 9 {
        let Some(when) = Date::new(year, month_number, day) else { continue };
        if when >= today {
            return Some(when);
        }
    }
    None
}

/// The day and month the line names, as the next date it falls, and how to say it.
pub fn _date_the_line_gave(text_in: &str) -> (Option<Date>, Option<String>) {
    let today = today();
    if let Some(found) = rx(gstr("_DAY_MONTH"), I).search(text_in) {
        let day = found.name("day").or_else(|| found.name("day2")).unwrap_or("");
        let month = found.name("month").or_else(|| found.name("month2")).unwrap_or("").to_lowercase();
        let Some(month_number) = _month_number(&month) else { return (None, None) };
        let Some(day) = int_or_none(day).and_then(|d| d.to_i128()) else { return (None, None) };
        return match _day_in_a_month(day as i64, month_number, today) {
            Some(when) => (Some(when), Some(format!("{} {}", when.day, when.month_name()))),
            None => (None, None),
        };
    }
    let Some(bare_day) = _DAY_ONLY.search(text_in) else { return (None, None) };
    let day = py_int(bare_day.name_str("day")) as i64;
    for step in 0..13 {
        let month_index = today.month as i64 - 1 + step;
        let Some(when) = Date::new(today.year as i64 + month_index.div_euclid(12), month_index.rem_euclid(12) + 1, day)
        else {
            continue;
        };
        if when >= today {
            return (Some(when), Some(format!("{} {}", when.day, when.month_name())));
        }
    }
    (None, None)
}

static _SMALL_NUMBER: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| gint_pairs("_SMALL_NUMBER"));
static _HOW_FAR: LazyLock<&'static Re> = LazyLock::new(|| gre("_HOW_FAR"));

/// The same day that many months on, in the month that has it.
pub fn _month_on(day: Date, months_ahead: i64) -> Date {
    let month_index = day.month as i64 - 1 + months_ahead;
    let year = day.year as i64 + month_index.div_euclid(12);
    let month = month_index.rem_euclid(12) + 1;
    let last = days_in_month(year as i32, month as u32) as i64;
    Date::of(year, month, (day.day as i64).min(last))
}

/// The 1st of the month that many months on.
pub fn _first_of_the_month(day: Date, months_ahead: i64) -> Date {
    let month_index = day.month as i64 - 1 + months_ahead;
    Date::of(day.year as i64 + month_index.div_euclid(12), month_index.rem_euclid(12) + 1, 1)
}

/// The date a phrase counted from today points at, and how to say it.
pub fn _relative_day(text_in: &str) -> (Option<Date>, Option<String>) {
    let today = today();
    let said = strip_chars(&squash(&text_in.to_lowercase()), " .!?").to_string();
    let full = |p: &str| fullmatch(p, &said).is_some();
    if full(r"(?:today|tonight|this day)") {
        return (Some(today), Some("today".into()));
    }
    if full(r"tomorrow") {
        return (Some(today.plus(1)), Some("tomorrow".into()));
    }
    if full(r"(?:the )?day after tomorrow") {
        return (Some(today.plus(2)), Some("the day after tomorrow".into()));
    }
    let wd = today.weekday() as i64;
    if full(r"(?:the|this) weekend") {
        return (Some(today.plus((5 - wd).rem_euclid(7))), Some("the weekend".into()));
    }
    if full(r"next weekend") {
        return (Some(today.plus((5 - wd).rem_euclid(7) + 7)), Some("next weekend".into()));
    }
    if full(r"next week") {
        let ahead = (7 - wd).rem_euclid(7);
        return (Some(today.plus(if ahead == 0 { 7 } else { ahead })), Some("next week".into()));
    }
    if full(r"next month") {
        return (Some(_first_of_the_month(today, 1)), Some("next month".into()));
    }
    if full(r"next year") {
        return (Some(Date::of(today.year as i64 + 1, 1, 1)), Some("next year".into()));
    }
    let Some(found) = _HOW_FAR.fullmatch(&said) else { return (None, None) };
    let raw = found.name_str("num").to_lowercase();
    let count = if isdigit(&raw) { Some(py_int(&raw) as i64) } else { lookup(&_SMALL_NUMBER, &raw) };
    let Some(count) = count.filter(|c| *c != 0 && *c <= 999) else { return (None, None) };
    let unit = found.name_str("unit").to_lowercase();
    if unit.starts_with("day") {
        return (Some(today.plus(count)), None);
    }
    if unit.starts_with("week") {
        return (Some(today.plus(7 * count)), None);
    }
    if unit.starts_with("month") {
        return (Some(_month_on(today, count)), None);
    }
    (Some(_month_on(today, 12 * count)), None)
}

/// The count in the unit the line asked for, in the singular when it is one.
pub fn _relative_count_text(unit: Option<&str>, days: i64) -> String {
    let unit = unit.filter(|u| !u.is_empty()).unwrap_or("days").to_lowercase();
    let counted = if unit.starts_with("sleep") || unit.starts_with("night") { "sleeps".to_string() } else { unit };
    if days == 1 {
        return counted.strip_suffix('s').map(str::to_string).unwrap_or(counted);
    }
    counted
}

/// A countdown to a day counted from today: "how many sleeps until the weekend".
pub fn _days_until_a_relative_day(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    let found = rx(gstr("_COUNT_ASK"), I).fullmatch(&text_)?;
    let rest = text::strip(found.name("rest").filter(|r| !r.is_empty()).or_else(|| found.name("rest2")).unwrap_or(""))
        .to_string();
    if rest.is_empty() {
        return None;
    }
    let (when, said) = _relative_day(&rest);
    let when = when?;
    let days = when.minus(&today());
    let unit = found.name("unit").filter(|u| !u.is_empty()).unwrap_or("days").to_lowercase();
    let one = _relative_count_text(Some(&unit), days);
    if days <= 0 {
        return Some(format!("That is today - no {} left to wait.", _relative_count_text(Some(&unit), 2)));
    }
    if days == 1 {
        if said.as_deref() == Some("tomorrow") {
            return Some(format!("Tomorrow, so one {one}."));
        }
        return Some(format!("One {one} until {}.", said.unwrap_or_else(|| "None".into())));
    }
    let said = said.filter(|s| !s.is_empty()).unwrap_or_else(|| _short_date(when));
    Some(format!("{} until {said}.", _count_in(Some(&unit), days, Some(when))))
}

/// "what date is it in two weeks": the calendar arithmetic, said as a date.
pub fn _date_counted_from_today(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    let mut rest = None;
    for pattern in [
        r"what(?:'s|s| is)? the (?:date|day) (?:(?:in|of) )?(?P<rest>.+)",
        r"what date (?:is it|is|will it be) (?P<rest>.+)",
        r"what day (?:is it|will it be) (?P<rest>.+)",
        r"what date is it on (?P<rest>.+)",
    ] {
        if let Some(found) = fullmatch(pattern, &text_) {
            rest = Some(text::strip(found.name_str("rest")).to_string());
            break;
        }
    }
    let rest = rest.filter(|r| !r.is_empty())?;
    let (when, _) = _relative_day(&rest);
    let when = when?;
    if when == today() {
        return Some(format!("That is today, {}.", _short_date(when)));
    }
    Some(format!("That is {}.", _day_text(when)))
}

/// A countdown to an event whose date the bot cannot know: ask for the day, then count.
pub fn _days_until_a_named_day(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    if has(&format!(r"{}\b", gstr("_CHRISTMAS_DAY")), &text_) || has(r"\beaster\b", &text_) {
        return None;
    }
    let found = rx(gstr("_COUNT_ASK"), I).fullmatch(&text_)?;
    let rest = text::strip(found.name("rest").filter(|r| !r.is_empty()).or_else(|| found.name("rest2")).unwrap_or(""))
        .to_string();
    let rest = rx(r"^(?:i|we)\s+(?:go|am going|are going|'m going|'re going|leave)\s+(?:on\s+)?", I).sub("", &rest);
    if rest.is_empty() {
        return None;
    }
    let unit = found.name("unit").filter(|u| !u.is_empty()).unwrap_or("days").to_lowercase();
    let counted =
        if unit.starts_with("sleep") || unit.starts_with("night") { "sleeps".to_string() } else { unit.clone() };
    let event = rx(&format!(r"{}(?:\s+(?:please|thanks|now|yet))?", gstr("_COUNTED_EVENT")), I).fullmatch(&rest);
    let named = event.is_some();
    if !named && _date_the_line_gave(&rest).0.is_none() {
        return None;
    }
    let (when, said) = _date_the_line_gave(&rest);
    if let Some(when) = when {
        let days = when.minus(&today());
        if days == 0 {
            return Some(format!("That is today - no {counted} left to wait."));
        }
        return Some(format!("{} until {}.", _count_in(Some(&unit), days, Some(when)), said.unwrap_or_default()));
    }
    if has(r"\bpay ?day\b", &rest) {
        return Some(format!(
            "I can't see your payday from here, so tell me the date it lands on and I'll count the {counted} to it."
        ));
    }
    let asked = format!(
        "I can't see a calendar from here, so tell me the date - the day and the month - and I'll count it: how many {counted} from today."
    );
    if counted.starts_with("month") {
        return Some(asked);
    }
    Some(
        asked
            + " If it is a child waiting, a paper chain or a row of crosses on the calendar is the same count, and they can do one a night, which does more for the waiting than the number does.",
    )
}

static _REST_OF_A_PERIOD_WORDS: LazyLock<std::collections::HashSet<String>> =
    LazyLock::new(|| g("_REST_OF_A_PERIOD_WORDS").str_set());
static _REST_OF_A_PERIOD: LazyLock<&'static Re> = LazyLock::new(|| gre("_REST_OF_A_PERIOD"));

/// How much of the month or the year is still to come, counted from today.
pub fn _days_left_in_the_month_or_the_year(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    if !_REST_OF_A_PERIOD.is_match(&text_) || !text::split(&text_).iter().any(|w| w == "how") {
        return None;
    }
    let words = rx(r"[a-z0-9]+", 0).findall_str(&text_);
    if words.iter().any(|w| !_REST_OF_A_PERIOD_WORDS.contains(w)) {
        return None;
    }
    if !words.iter().any(|w| matches!(w.as_str(), "left" | "go" | "remaining" | "remain" | "end")) {
        return None;
    }
    let today = today();
    let has_word = |w: &str| words.iter().any(|x| x == w);
    let unit = if has_word("weeks") || has_word("week") { "weeks" } else { "days" };
    let mut month = None;
    for (number, name) in _MONTH_NAMES.iter().enumerate() {
        let low = name.to_lowercase();
        let short: String = low.chars().take(3).collect();
        if has_word(&low) || has_word(&short) {
            month = Some(number as i64 + 1);
        }
    }
    let mut waiting = false;
    let mut extra = String::new();
    let (end, said);
    if has_word("tax") {
        let mut e = Date::of(today.year as i64, 4, 5);
        if e < today {
            e = Date::of(today.year as i64 + 1, 4, 5);
        }
        end = e;
        said = "the tax year".to_string();
        extra = format!(" It ends on {}.", _said_with_a_weekday(end));
    } else if has_word("year") {
        end = Date::of(today.year as i64, 12, 31);
        said = format!("{}", today.year);
    } else if let Some(month) = month.filter(|m| *m != today.month as i64) {
        let year = today.year as i64 + if month < today.month as i64 { 1 } else { 0 };
        end = Date::of(year + month.div_euclid(12), month.rem_euclid(12) + 1, 1).plus(-1);
        said = format!("{}: the last day of it is {}", _MONTH_NAMES[end.month as usize - 1], _said_with_a_weekday(end));
        waiting = true;
    } else {
        end = Date::of(
            today.year as i64 + (today.month as i64).div_euclid(12),
            (today.month as i64).rem_euclid(12) + 1,
            1,
        )
        .plus(-1);
        said = _MONTH_NAMES[end.month as usize - 1].clone();
    }
    let days = end.minus(&today);
    if days <= 0 {
        return Some(format!(
            "That is today - nothing left of {said} after today, and {} in all if you count it.",
            _count_in(Some(unit), 1, None)
        ));
    }
    if waiting {
        return Some(format!(
            "{} until the end of {said}. That is {} counting today.",
            _count_in(Some(unit), days, None),
            _count_in(Some(unit), days + 1, None)
        ));
    }
    Some(format!(
        "{} left in {said} after today, {} counting today.{extra}",
        _count_in(Some(unit), days, None),
        _count_in(Some(unit), days + 1, None)
    ))
}

static _SINCE_ASK: LazyLock<&'static Re> = LazyLock::new(|| gre("_SINCE_ASK"));
static _SINCE_FILLER: LazyLock<&'static Re> = LazyLock::new(|| gre("_SINCE_FILLER"));
static _A_YEAR_IN_THE_LINE: LazyLock<&'static Re> = LazyLock::new(|| gre("_A_YEAR_IN_THE_LINE"));

/// "How long ago was 3 March 2020" names a year, and the year is half the question.
pub fn _year_the_line_gave(text_in: &str) -> Option<i64> {
    _A_YEAR_IN_THE_LINE.search(text_in).map(|m| py_int(m.str(1)) as i64)
}

/// The days between a date the line gives and today, counting backwards.
pub fn _how_long_ago(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    let found = _SINCE_ASK.search(&text_)?;
    let rest = text::strip(&format!("{} {}", &text_[..found.start()], &text_[found.end()..])).to_string();
    let rest = squash(&_SINCE_FILLER.sub(" ", &rest));
    if rest.is_empty() {
        return None;
    }
    let year = _year_the_line_gave(&rest);
    let (mut when, mut said) = _date_the_line_gave(&rest);
    if when.is_none() {
        (when, said) = _a_known_day_that_has_gone(&rest, year);
    } else if let Some(year) = year {
        when = Some(when.unwrap().with_year(year)?);
        said = Some(format!("{} {year}", said.unwrap_or_default()));
    }
    let mut when = when?;
    let said = said.unwrap_or_default();
    let today = today();
    if when > today {
        if year.is_some() {
            return None;
        }
        when = when.with_year(when.year as i64 - 1)?;
    }
    let days = today.minus(&when);
    let unit = found.name("unit").filter(|u| !u.is_empty()).unwrap_or("days").to_lowercase();
    if days == 0 {
        return Some(format!("That was today - {said} is today, so no days ago at all."));
    }
    let mut count = _count_in(Some(&unit), days, None);
    if days >= 1000 && unit.starts_with("day") {
        count = format!("{} days", commas(&days.to_string()));
    }
    Some(format!("{count} since {said} - that was a {}.", _WEEKDAYS[when.weekday() as usize]))
}

static _WORKING_ASK: LazyLock<&'static Re> = LazyLock::new(|| gre("_WORKING_ASK"));
static _WORKING_LEFT: LazyLock<&'static Re> = LazyLock::new(|| gre("_WORKING_LEFT"));
static _WORKING_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| gre("_WORKING_SHAPE"));

/// The next time the day the line names comes round, as a date and how to say it.
pub fn _a_known_day_the_line_names_next(text_in: &str) -> (Option<Date>, Option<String>) {
    let today = today();
    let y = today.year as i64;
    if has(&format!(r"{}\b", gstr("_CHRISTMAS_DAY")), text_in) {
        let eve = has(r"\beve\b", text_in);
        let d = if eve { 24 } else { 25 };
        let mut when = Date::of(y, 12, d);
        if when < today {
            when = Date::of(y + 1, 12, d);
        }
        return (Some(when), Some(if eve { "Christmas Eve" } else { "Christmas" }.into()));
    }
    if has(r"\bnew year(?:'s|s')? (?:day|eve)\b|\bnew year\b", text_in) {
        let mut when = Date::of(y, 1, 1);
        if when < today {
            when = Date::of(y + 1, 1, 1);
        }
        return (Some(when), Some("New Year's Day".into()));
    }
    if has(r"\beaster\b", text_in) {
        let mut when = _easter_sunday(y);
        if when < today {
            when = _easter_sunday(y + 1);
        }
        return (Some(when), Some("Easter Sunday".into()));
    }
    _date_the_line_gave(text_in)
}

/// "how many working hours in a week" is the job's week, not the calendar's.
pub fn _a_working_week(clean: &str) -> Option<String> {
    let said = squash(strip_chars(text::strip(&clean.to_lowercase()), "?.!"));
    let found = _WORKING_SHAPE.fullmatch(&said)?;
    let (unit, span) = (found.name_str("unit"), found.name_str("week"));
    if unit.starts_with("hour") && span == "week" {
        return Some("A full-time week is usually 37.5 to 40 hours - 7.5 or 8 hours a day over five days. The law caps the average at 48 hours a week over 17 weeks unless the person signs to opt out, and part-time hours are usually written as a share of that full-time week.".into());
    }
    if unit.starts_with("hour") && span == "day" {
        return Some("A full-time day is usually 7.5 or 8 hours, and a break of 20 minutes after six hours is the legal minimum in the UK. Contracted hours are the ones that count for pay, so check the contract rather than the rota.".into());
    }
    if unit.starts_with("day") && span == "week" {
        return Some("Five, for a full-time job - Monday to Friday - which is why a week off is five days of leave, not seven. Shift work and four-day weeks are the usual exceptions, and the contract says which one you are on.".into());
    }
    None
}

/// The weekdays, or the whole weekends, between tomorrow and the day the line names.
pub fn _working_days_or_weekends_until(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    let today = today();
    if fullmatch(r"how many (?:working|work|business) days? (?:in|per) a year", &text_).is_some() {
        return Some("About 253 working days in a year: 261 weekdays, less the 8 bank holidays in England and Wales. Leap years add one.".into());
    }
    if fullmatch(r"how many (?:working|work) hours? (?:in|per|to) (?:a|the) week", &text_).is_some() {
        return Some("A full-time week is usually 37.5 to 40 hours - 7.5 or 8 hours a day, five days. The law caps the average at 48 hours a week over 17 weeks unless the person signs to opt out, and part-time hours are usually written as a share of the full-time week.".into());
    }
    if fullmatch(r"how many weekends (?:in|per) a year", &text_).is_some() {
        return Some("104 weekend days in a year - 52 Saturdays and 52 Sundays - plus a day or two in most years, because 365 is not a whole number of weeks.".into());
    }
    let found = _WORKING_ASK.fullmatch(&text_).or_else(|| _WORKING_LEFT.fullmatch(&text_))?;
    let weekends = found.name_str("count").to_lowercase().contains("weekend");
    let rest = text::strip(found.name("rest").unwrap_or("")).to_string();
    let mut period = found.name("period").unwrap_or("").to_string();
    let (mut end, mut said): (Option<Date>, Option<String>) = (None, None);
    if !rest.is_empty() {
        (end, said) = _a_known_day_the_line_names_next(&rest);
        if end.is_none() {
            if !rx(r"\b(?:the\s+|this\s+)?(year|month)\b", I).is_match(&rest) {
                return None;
            }
            period = if rx(r"\bmonth\b", I).is_match(&rest) { "month".into() } else { "year".into() };
        }
    }
    if rest.is_empty() || (end.is_none() && !period.is_empty()) {
        if period == "month" {
            let e = Date::of(
                today.year as i64 + (today.month as i64).div_euclid(12),
                (today.month as i64).rem_euclid(12) + 1,
                1,
            )
            .plus(-1);
            said = Some(format!("the end of {}", _MONTH_NAMES[e.month as usize - 1]));
            end = Some(e);
        } else {
            end = Some(Date::of(today.year as i64, 12, 31));
            said = Some(format!("the end of {}", today.year));
        }
    }
    // Python reads `end` here even when it was never set ("until" a day the line does not name,
    // with no period): that path raises, so it panics here too.
    let end = end.expect("UnboundLocalError: end");
    let said = said.unwrap_or_default();
    let days = end.minus(&today);
    if days <= 0 {
        return None;
    }
    let span: Vec<Date> = (1..=days).map(|step| today.plus(step)).collect();
    if weekends {
        let count = span.iter().filter(|d| d.weekday() == 5 && d.plus(1) < end).count();
        if count == 0 {
            return Some(format!(
                "There is no whole weekend between today and {said} - the next Saturday falls on {} or later.",
                _said_with_a_weekday(end)
            ));
        }
        let first = *span.iter().find(|d| d.weekday() == 5).unwrap();
        return Some(format!(
            "{count} weekend{} until {said}, the first starting on {}.",
            if count == 1 { "" } else { "s" },
            _said_with_a_weekday(first)
        ));
    }
    let (where_, _) = _bank_where(if rest.is_empty() { &text_ } else { &rest });
    let mut years: Vec<i32> = span.iter().map(|d| d.year).collect();
    years.push(end.year);
    years.sort();
    years.dedup();
    let mut holidays = std::collections::HashSet::new();
    for year in years {
        for (_, day) in _bank_holidays(year as i64, &where_) {
            holidays.insert(day);
        }
    }
    let count = span.iter().filter(|d| d.weekday() < 5 && !holidays.contains(*d)).count();
    Some(format!("{count} working days until {said}, out of {days} days in all."))
}

/// The last time a day the bot knows came round, for a count backwards.
pub fn _a_known_day_that_has_gone(text_in: &str, year: Option<i64>) -> (Option<Date>, Option<String>) {
    let today = today();
    let ty = today.year as i64;
    let tail = year.map(|y| format!(" {y}")).unwrap_or_default();
    if has(&format!(r"{}\b", gstr("_CHRISTMAS_DAY")), text_in) {
        let mut when = Date::of(year.unwrap_or(ty), 12, 25);
        if year.is_none() && when > today {
            when = Date::of(ty - 1, 12, 25);
        }
        return (Some(when), Some(format!("Christmas Day{tail}")));
    }
    if has(r"\bnew year(?:'s|s')? (?:day|eve)\b|\bnew year\b", text_in) {
        let mut when = Date::of(year.unwrap_or(ty), 1, 1);
        if year.is_none() && when > today {
            when = Date::of(ty - 1, 1, 1);
        }
        return (Some(when), Some(format!("New Year's Day{tail}")));
    }
    if has(r"\beaster (?:sunday|day)\b|\beaster\b", text_in) {
        let mut when = _easter_sunday(year.unwrap_or(ty));
        if year.is_none() && when > today {
            when = _easter_sunday(ty - 1);
        }
        return (Some(when), Some(format!("Easter Sunday{tail}")));
    }
    (None, None)
}

/// When Easter and the days around it fall, only when the line asks for the date.
pub fn _days_until_easter(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    let (ask_date, date_tail) = (gstr("_ASK_DATE"), gstr("_DATE_TAIL"));
    for (subject, offset, name) in _EASTER_OFFSETS.iter() {
        let asked = format!(
            r"(?:(?:{ask_date})\s+(?:the\s+)?(?:{subject})(?:\s+(?:start|starts|begin|begins))?|(?:{subject})\s+(?:date|dates|20\d{{2}}))"
        );
        let waiting = format!(
            r"(?:how many |how long |how much longer )?(?:days?|weeks?|sleeps?) (?:is it |are there |left |to go |still )?(?:until|till|to|before) (?:the\s+)?(?:{subject})"
        );
        let judging = format!(
            r"(?:(?:is|was|will)|(?:{ask_date}))\s*(?:the\s+)?(?:{subject})\s+(?:going to be\s+)?(?:(?:next year|this year|next easter|this easter|(?:in )?20\d{{2}})\s+)?(?:(?:early|late)\s+or\s+(?:early|late)|early|late|on the early side|on the late side)"
        );
        let counting = fullmatch(&format!("{waiting}{date_tail}"), &text_).is_some();
        let where_ = fullmatch(&format!("{judging}{date_tail}"), &text_).is_some();
        if !counting && !where_ && fullmatch(&format!("{asked}{date_tail}"), &text_).is_none() {
            continue;
        }
        let (year, named, today) = _feast_year(&text_);
        let mut day = _easter_sunday(year).plus(*offset);
        if day < today && !named && !has(r"\bthis (?:year|easter)\b", &text_) {
            day = _easter_sunday(year + 1).plus(*offset);
        }
        let said = format!("{} {}", day.day, _MONTH_NAMES[day.month as usize - 1]);
        if where_ {
            let first = Date::of(day.year as i64, 3, 22).plus(*offset);
            let last = Date::of(day.year as i64, 4, 25).plus(*offset);
            let span = format!(
                "{} {} to {} {}",
                first.day,
                _MONTH_NAMES[first.month as usize - 1],
                last.day,
                _MONTH_NAMES[last.month as usize - 1]
            );
            let tense = if day < today { "was" } else { "is" };
            let sense = if day <= first.plus(9) {
                "early"
            } else if day >= last.plus(-9) {
                "late"
            } else {
                return Some(format!(
                    "{name} {tense} on {said}, which is about the middle of the range it can fall in ({span}) - neither early nor late."
                ));
            };
            return Some(format!("{name} {tense} on {said}, which is {sense}: it can fall anywhere from {span}."));
        }
        if counting && !named && day >= today {
            let days = day.minus(&today);
            if days == 0 {
                return Some(format!("{name} is today."));
            }
            return Some(format!("{days} days until {name}."));
        }
        if named {
            return Some(format!("{name} is on {said} {}.", day.year));
        }
        let days = day.minus(&today);
        if days == 0 {
            return Some(format!("{name} is today."));
        }
        if days < 0 {
            let ago = -days;
            return Some(format!("{name} this year was on {said}, {ago} day{} ago.", if ago == 1 { "" } else { "s" }));
        }
        return Some(format!("{name} is on {said}, {days} days away."));
    }
    None
}

/// The nth weekday of a month: Father's Day is the third Sunday in June.
pub fn _nth_weekday(year: i64, month: i64, weekday: i64, nth: i64) -> Date {
    let day = Date::of(year, month, 1);
    let day = day.plus((weekday - day.weekday() as i64).rem_euclid(7));
    day.plus(7 * (nth - 1))
}

/// One of `_NAMED_DAYS`: the words, the rule and its numbers, the name, and the tail.
struct NamedDay {
    words: Vec<String>,
    rule: (String, Vec<i64>),
    name: String,
    extra: String,
}

static _NAMED_DAYS: LazyLock<Vec<NamedDay>> = LazyLock::new(|| {
    g("_NAMED_DAYS")
        .items()
        .iter()
        .map(|t| {
            let t = t.items();
            let rule = t[1].items();
            NamedDay {
                words: t[0].strs(),
                rule: (rule[0].str().to_string(), rule[1..].iter().map(|v| v.as_int().unwrap()).collect()),
                name: t[2].str().to_string(),
                extra: t[3].str().to_string(),
            }
        })
        .collect()
});

fn named_day_on(rule: &(String, Vec<i64>), year: i64) -> Date {
    let n = &rule.1;
    match rule.0.as_str() {
        "fixed" => Date::of(year, n[0], n[1]),
        "easter" => _easter_sunday(year).plus(n[0]),
        _ => _nth_weekday(year, n[0], n[1], n[2]),
    }
}

/// When one of the named days falls: "when is mothers day", "how many days until halloween".
pub fn _named_day_date(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    let (ask_date, date_tail) = (gstr("_ASK_DATE"), gstr("_DATE_TAIL"));
    for day in _NAMED_DAYS.iter() {
        let subject = day.words.join("|");
        let asked = format!(r"(?:{ask_date})\s+(?:the\s+)?(?:{subject})");
        let waiting = format!(
            r"(?:how many |how long )?(?:days?|weeks?|sleeps?) (?:is it |are there |left |to go )?(?:until|till|to|before) (?:the\s+)?(?:{subject})"
        );
        let counting = fullmatch(&format!("{waiting}{date_tail}"), &text_).is_some();
        if !counting && fullmatch(&format!("{asked}{date_tail}"), &text_).is_none() {
            continue;
        }
        let (year, named_year, today) = _feast_year(&text_);
        let mut when = named_day_on(&day.rule, year);
        if when < today && !named_year && !text_.contains("this year") {
            when = named_day_on(&day.rule, year + 1);
        }
        let said = format!("{} {}", when.day, _MONTH_NAMES[when.month as usize - 1]);
        let (name, extra) = (&day.name, &day.extra);
        if named_year {
            return Some(format!("{name} is on {said} {}.{extra}", when.year));
        }
        let days = when.minus(&today);
        if days == 0 {
            return Some(format!("{name} is today.{extra}"));
        }
        if counting {
            return Some(format!("{days} days until {name}."));
        }
        if days < 0 {
            return Some(format!("{name} this year was on {said}, {} days ago.{extra}", -days));
        }
        return Some(format!("{name} is on {said}, {days} days away.{extra}"));
    }
    None
}

static _SEASONS: LazyLock<Vec<(Vec<String>, (i64, i64), i64)>> = LazyLock::new(|| {
    g("_SEASONS")
        .items()
        .iter()
        .map(|t| {
            let t = t.items();
            let md = t[1].items();
            (t[0].strs(), (md[0].as_int().unwrap(), md[1].as_int().unwrap()), t[2].as_int().unwrap())
        })
        .collect()
});
static _THE_TURN_OF_THE_YEAR: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_THE_TURN_OF_THE_YEAR"));

fn months_days(name: &str) -> i64 {
    lookup(&MONTHS, name).unwrap_or_else(|| panic!("KeyError: {name}"))
}

/// The countdown to a bare month, a season, or New Year, or None.
pub fn _days_until_a_month_or_season(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    let found = rx(gstr("_COUNT_ASK"), I).fullmatch(&text_)?;
    let rest = text::strip(found.name("rest").filter(|r| !r.is_empty()).or_else(|| found.name("rest2")).unwrap_or(""))
        .to_string();
    let rest = rx(r"^(?:the|my|our)\s+", I).sub("", &rest);
    let rest = rx(r"\s+(?:please|then|now|exactly|roughly|again|yet)$", I).sub("", &rest);
    let rest = rx(r"\s+(?:start|starts|begin|begins|come|comes|arrive|arrives|get here)$", I).sub("", &rest);
    if rest.is_empty()
        || rx(r"\b(?:holiday|holidays|break|term|half term|exam|exams|payday|pay day|deadline|due|start|starts|begin|begins|end|ends)\b", I)
            .is_match(&rest)
    {
        return None;
    }
    let today = today();
    let ty = today.year as i64;
    let unit = found.name("unit").filter(|u| !u.is_empty()).unwrap_or("days").to_lowercase();
    if _THE_TURN_OF_THE_YEAR.contains(&rest) {
        let eve = rest.contains("eve");
        let mut when = if eve { Date::of(ty, 12, 31) } else { Date::of(ty, 1, 1) };
        if when < today {
            when = Date::of(when.year as i64 + 1, when.month as i64, when.day as i64);
        }
        let days = when.minus(&today);
        let name = if eve { "New Year's Eve" } else { "New Year's Day" };
        let said = format!("{} {}", when.day, _MONTH_NAMES[when.month as usize - 1]);
        if days == 0 {
            return Some(format!("{name} is today, {said}."));
        }
        let after = if !eve { " New Year's Eve is the night before it." } else { "" };
        return Some(format!("{} until {name} - it is {said}.{after}", _count_in(Some(&unit), days, Some(when))));
    }
    for (words, (month, day), last_month) in _SEASONS.iter() {
        if !words.contains(&rest) {
            continue;
        }
        let mut start = Date::of(ty, *month, *day);
        let last_name = _MONTH_NAMES[*last_month as usize - 1].to_lowercase();
        let mut end = Date::of(ty, *last_month, months_days(&last_name));
        if end < start {
            end = Date::of(ty + 1, *last_month, months_days(&last_name));
        }
        let name = capitalize(&rest);
        if start <= today && today <= end {
            return Some(format!(
                "{name} is here now - it runs to the end of {}. The next one starts on {} {} next year.",
                _MONTH_NAMES[end.month as usize - 1],
                start.day,
                _MONTH_NAMES[start.month as usize - 1]
            ));
        }
        if start < today {
            start = Date::of(ty + 1, *month, *day);
        }
        let days = start.minus(&today);
        return Some(format!(
            "{} until {name} - it starts on {} {}.",
            _count_in(Some(&unit), days, Some(start)),
            start.day,
            _MONTH_NAMES[start.month as usize - 1]
        ));
    }
    if let Some(year_said) = fullmatch(r"(?:(?:the\s+)?year\s+)?(20\d{2}|21\d{2})", &rest) {
        let year = py_int(year_said.str(1)) as i64;
        let when = Date::of(year, 1, 1);
        if when <= today {
            return None;
        }
        let days = when.minus(&today);
        return Some(format!(
            "{} until {year} - it starts on {} {} {year}.",
            _count_in(Some(&unit), days, Some(when)),
            when.day,
            _MONTH_NAMES[when.month as usize - 1]
        ));
    }
    let mut month = _month_number(&rest);
    if month.is_none() {
        let shorts: Vec<String> = _MONTHS_SAID.iter().map(|n| n.chars().take(3).collect()).collect();
        if let Some(i) = shorts.iter().position(|s| *s == rest) {
            month = Some(i as i64 + 1);
        }
    }
    let month = month?;
    let mut when = Date::of(ty, month, 1);
    let mut next_year = false;
    if when < today {
        when = Date::of(ty + 1, month, 1);
        next_year = true;
    }
    let days = when.minus(&today);
    let label = format!("{}{}", _MONTH_NAMES[month as usize - 1], if next_year { " next year" } else { "" });
    Some(format!(
        "{} until {label} - it starts on {} {}{}.",
        _count_in(Some(&unit), days, Some(when)),
        when.day,
        _MONTH_NAMES[when.month as usize - 1],
        if next_year { format!(" {}", when.year) } else { String::new() }
    ))
}

// --- the bank holidays -----------------------------------------------------------------------

static _BANK_HOLIDAY_NEXT: LazyLock<&'static Re> = LazyLock::new(|| gre("_BANK_HOLIDAY_NEXT"));
static _BANK_HOLIDAY_COUNT: LazyLock<&'static Re> = LazyLock::new(|| gre("_BANK_HOLIDAY_COUNT"));
static _BANK_HOLIDAY_LIST: LazyLock<&'static Re> = LazyLock::new(|| gre("_BANK_HOLIDAY_LIST"));
static _SAYS_WEEKDAY: LazyLock<&'static Re> = LazyLock::new(|| gre("_SAYS_WEEKDAY"));
static _WEEKDAYS: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_WEEKDAYS"));

/// 24 -> "24th", 1 -> "1st", 22 -> "22nd".
pub fn _ordinal(number: i64) -> String {
    if (11..=13).contains(&number.rem_euclid(100)) {
        return format!("{number}th");
    }
    let last = number.rem_euclid(10);
    let suffix = if last < 4 { ["th", "st", "nd", "rd"][(last.min(4) % 4) as usize] } else { "th" };
    format!("{number}{suffix}")
}

/// The weekday a date falls on: "what day is the 24th of december".
pub fn _the_weekday_of_a_date(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    let today = today();
    let said = rx(
        r"^(?:what|which)\s+(?:day|date|weekday)\s+(?:of\s+the\s+week\s+)?(?:is|was|will)\s+(?:it\s+)?(?:be\s+)?(?:on\s+)?(?:the\s+)?",
        0,
    )
    .match_(&text_)
    .or_else(|| rx(r"^what\s+day\s+does\s+(?:the\s+)?", 0).match_(&text_))?;
    let backwards = has(r"\bwas\b", said.as_str());
    let mut rest = text::strip(&text_[said.end()..]).to_string();
    rest = sub(r"\s+(?:fall|falls|land|lands)\s+on$", "", &rest);
    let mut year_said = None;
    if let Some(year_found) = search(r"\s+(?:in\s+)?(20\d{2})$", &rest) {
        year_said = Some(py_int(year_found.str(1)) as i64);
        rest = text::strip(&rest[..year_found.start()]).to_string();
    }
    rest = sub(r"\s+(?:this|next)\s+year$", "", &rest);
    if rest.is_empty() || _SAYS_WEEKDAY.is_match(&rest) {
        return None;
    }
    let months_or = gstr("_MONTHS_OR");
    let found = fullmatch(&format!(r"(\d{{1,2}})(?:st|nd|rd|th)?\s+(?:of\s+)?({months_or})"), &rest)
        .or_else(|| fullmatch(&format!(r"({months_or})\s+(\d{{1,2}})(?:st|nd|rd|th)?"), &rest));
    if let Some(found) = found {
        let (first, second) = (found.str(1), found.str(2));
        let (day, month) = if isdigit(first) { (py_int(first) as i64, second) } else { (py_int(second) as i64, first) };
        let month_number = _month_number(month)?;
        let when;
        if let Some(year) = year_said.filter(|y| *y != 0) {
            let w = Date::new(year, month_number, day)?;
            return Some(format!("{} {} {} is a {}.", w.day, w.month_name(), w.year, w.weekday_name()));
        }
        if backwards {
            let mut w = None;
            for years_back in 0..2 {
                let Some(guess) = Date::new(today.year as i64 - years_back, month_number, day) else { continue };
                if guess <= today {
                    w = Some(guess);
                    break;
                }
            }
            when = w;
        } else {
            when = _day_in_a_month(day, month_number, today);
        }
        let when = when?;
        let days = when.minus(&today);
        if days < 0 {
            let ago = -days;
            return Some(format!(
                "{} {} was a {}, {ago} day{} ago.",
                when.day,
                when.month_name(),
                when.weekday_name(),
                if ago == 1 { "" } else { "s" }
            ));
        }
        if days == 0 {
            return Some(format!("{} {} is today, a {}.", when.day, when.month_name(), when.weekday_name()));
        }
        if days == 1 {
            return Some(format!("{} {} is tomorrow, a {}.", when.day, when.month_name(), when.weekday_name()));
        }
        return Some(format!("{} {} is a {}, {days} days away.", when.day, when.month_name(), when.weekday_name()));
    }
    let bare = fullmatch(r"(\d{1,2})(?:st|nd|rd|th)?", &rest)?;
    let day = py_int(bare.str(1)) as i64;
    let mut this_month = None;
    for step in 0..3i64 {
        let shift = if backwards { -step } else { step };
        let month = (today.month as i64 - 1 + shift).rem_euclid(12) + 1;
        let year = today.year as i64 + (today.month as i64 - 1 + shift).div_euclid(12);
        let Some(guess) = Date::new(year, month, day) else { continue };
        if backwards && guess > today {
            continue;
        }
        if !backwards && guess < today {
            continue;
        }
        this_month = Some(guess);
        break;
    }
    let this_month = this_month?;
    let next_month = today.month as i64 % 12 + 1;
    let next_year = today.year as i64 + if today.month == 12 { 1 } else { 0 };
    let following = Date::new(next_year, next_month, day);
    let said_next = match following {
        Some(f) if !backwards && !(f.month == this_month.month && f.year == this_month.year) => {
            format!(", and the {} of next month is a {}", _ordinal(day), f.weekday_name())
        }
        _ => String::new(),
    };
    let days = this_month.minus(&today);
    if days == 0 {
        return Some(format!("Today is the {}, a {}{said_next}.", _ordinal(day), this_month.weekday_name()));
    }
    let month_said =
        if this_month.month == today.month { "this month".to_string() } else { this_month.month_name().to_string() };
    if days < 0 {
        let ago = -days;
        return Some(format!(
            "The {} of {month_said} was a {}, {ago} day{} ago.",
            _ordinal(day),
            this_month.weekday_name(),
            if ago == 1 { "" } else { "s" }
        ));
    }
    Some(format!(
        "The {} of {month_said} is a {}, {days} days away{said_next}.",
        _ordinal(day),
        this_month.weekday_name()
    ))
}

/// A fixed bank holiday that lands at the weekend is taken on the next free weekday.
pub fn _the_wider_day(day: Date, taken: &[Date]) -> Date {
    let mut day = day;
    while day.weekday() > 4 || taken.contains(&day) {
        day = day.plus(1);
    }
    day
}

/// The last weekday of a month: the summer bank holiday is the last Monday in August.
pub fn _last_weekday(year: i64, month: i64, weekday: i64) -> Date {
    let last = if month == 12 { Date::of(year, 12, 31) } else { Date::of(year, month + 1, 1).plus(-1) };
    last.plus(-((last.weekday() as i64 - weekday).rem_euclid(7)))
}

/// Every bank holiday in one year, as (name, date), in order.
pub fn _bank_holidays(year: i64, where_: &str) -> Vec<(String, Date)> {
    let easter = _easter_sunday(year);
    let fixed_days: Vec<(&str, i64, i64)> = match where_ {
        "scotland" => vec![
            ("New Year's Day", 1, 1),
            ("2 January", 1, 2),
            ("St Andrew's Day", 11, 30),
            ("Christmas Day", 12, 25),
            ("Boxing Day", 12, 26),
        ],
        "northern ireland" => vec![
            ("New Year's Day", 1, 1),
            ("St Patrick's Day", 3, 17),
            ("the Battle of the Boyne", 7, 12),
            ("Christmas Day", 12, 25),
            ("Boxing Day", 12, 26),
        ],
        _ => vec![("New Year's Day", 1, 1), ("Christmas Day", 12, 25), ("Boxing Day", 12, 26)],
    };
    let mut taken: Vec<Date> = Vec::new();
    let mut rows: Vec<(String, Date)> = Vec::new();
    for (name, month, day) in fixed_days {
        let moved = _the_wider_day(Date::of(year, month, day), &taken);
        taken.push(moved);
        rows.push((name.to_string(), moved));
    }
    rows.push(("Good Friday".into(), easter.plus(-2)));
    if where_ != "scotland" {
        rows.push(("Easter Monday".into(), easter.plus(1)));
    }
    rows.push(("the early May bank holiday".into(), _nth_weekday(year, 5, 0, 1)));
    rows.push(("the spring bank holiday".into(), _last_weekday(year, 5, 0)));
    if where_ == "scotland" {
        rows.push(("the summer bank holiday".into(), _nth_weekday(year, 8, 0, 1)));
    } else {
        rows.push(("the summer bank holiday".into(), _last_weekday(year, 8, 0)));
    }
    rows.sort_by_key(|row| row.1); // stable, as Python's sorted
    rows
}

static _BANK_WHERE: LazyLock<Vec<(&'static Re, String, String)>> = LazyLock::new(|| {
    g("_BANK_WHERE")
        .items()
        .iter()
        .map(|t| {
            let t = t.items();
            (t[0].as_re().unwrap() as &'static Re, t[1].str().to_string(), t[2].str().to_string())
        })
        .collect()
});

pub fn _bank_where(text_in: &str) -> (String, String) {
    for (pattern, where_, said) in _BANK_WHERE.iter() {
        if pattern.is_match(text_in) {
            return (where_.clone(), said.clone());
        }
    }
    ("england".into(), "in England and Wales".into())
}

pub fn _said_with_a_weekday(when: Date) -> String {
    format!("{} {} {}", _WEEKDAYS[when.weekday() as usize], when.day, _MONTH_NAMES[when.month as usize - 1])
}

/// The next bank holiday, its countdown, or the year's list.
pub fn _next_bank_holiday(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    let asking = _BANK_HOLIDAY_NEXT.fullmatch(&text_).is_some();
    let counting = _BANK_HOLIDAY_COUNT.fullmatch(&text_).is_some();
    let listing = _BANK_HOLIDAY_LIST.fullmatch(&text_).is_some();
    if !(asking || counting || listing) {
        return None;
    }
    let (mut year, _named_year, today) = _feast_year(&text_);
    let (where_, told_where) = _bank_where(&text_);
    let mut rows = _bank_holidays(year, &where_);
    let mut ahead: Vec<(String, Date)> = rows.iter().filter(|r| r.1 >= today).cloned().collect();
    if ahead.is_empty() {
        year += 1;
        rows = _bank_holidays(year, &where_);
        ahead = rows.clone();
    }
    let when = ahead[0].clone();
    let after = if ahead.len() > 1 { ahead[1].clone() } else { _bank_holidays(year + 1, &where_)[0].clone() };
    let days = when.1.minus(&today);
    if listing && !asking && !counting {
        let left: Vec<String> = rows
            .iter()
            .filter(|r| r.1 >= today)
            .map(|(name, day)| format!("{name} on {}", _said_with_a_weekday(*day)))
            .collect();
        let said = left.join(", ");
        if where_ != "england" {
            return Some(format!("{} the bank holidays left this year are {said}.", upper_first_safe(&told_where)));
        }
        return Some(format!(
            "In England and Wales the bank holidays left this year are {said}. Scotland and Northern Ireland keep a few of their own as well."
        ));
    }
    if days == 0 {
        return Some(format!(
            "Today is a bank holiday {told_where}: {}, {}. The next one after that is {} on {}.",
            upper_first_safe(&when.0),
            _said_with_a_weekday(when.1),
            after.0,
            _said_with_a_weekday(after.1)
        ));
    }
    if counting {
        return Some(format!(
            "{days} day{} until the next bank holiday {told_where}: {} on {}.",
            if days == 1 { "" } else { "s" },
            when.0,
            _said_with_a_weekday(when.1)
        ));
    }
    Some(format!(
        "The next bank holiday {told_where} is {} on {}, {days} day{} away. After that it is {} on {}.",
        when.0,
        _said_with_a_weekday(when.1),
        if days == 1 { "" } else { "s" },
        after.0,
        _said_with_a_weekday(after.1)
    ))
}

// --- Today's date, and the next Monday -------------------------------------------------------

static _WEEKDAY_WORDS: LazyLock<Vec<(String, i64, String)>> = LazyLock::new(|| {
    g("_WEEKDAY_WORDS")
        .items()
        .iter()
        .map(|t| (t.items()[0].str().to_string(), t.items()[1].as_int().unwrap(), t.items()[2].str().to_string()))
        .collect()
});
static _MONTH_TITLE: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_MONTH_TITLE"));

/// A date the way a friend says it: the weekday first, then the day and the month.
pub fn _day_text(day: Date) -> String {
    format!("{}, {} {}", day.weekday_name(), day.day, _MONTH_TITLE[day.month as usize - 1])
}

/// Just the day and the month, for when the weekday is already in the sentence.
pub fn _short_date(day: Date) -> String {
    format!("{} {}", day.day, _MONTH_TITLE[day.month as usize - 1])
}

/// The weekday the line names, its index, and whether it said "next".
pub fn _weekday_of(text_in: &str, tail: &str) -> (Option<String>, Option<i64>, bool) {
    for (words, index, name) in _WEEKDAY_WORDS.iter() {
        let pattern =
            format!(r"{tail}(?:(?:the|this|next|coming|on|a week on)\s+)*(?:{words})(?:\s+(?:today|now|already|yet))*");
        if fullmatch(&pattern, text_in).is_some() {
            return (Some(name.clone()), Some(*index), has(r"\bnext\b", text_in));
        }
    }
    (None, None, false)
}

/// Today's date, and when the next Monday is.
pub fn _today(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    let today = today();
    if fullmatch(gstr("_TODAY_ASK"), &text_).is_some() {
        return Some(format!("It's {}.", _day_text(today)));
    }
    let (name, index, _) = _weekday_of(&text_, r"is (?:it|today) (?:still )?");
    if let (Some(_), Some(index)) = (&name, index) {
        if index == today.weekday() as i64 {
            return Some(format!("Yes, it's {}.", _day_text(today)));
        }
        return Some(format!("No, it's {}.", _day_text(today)));
    }
    let ask = r"(?:how many (?:days?|sleeps?) (?:is it |are there |do i have )?(?:until|till|til|to go before|before|to) |how long (?:until|till|til|to) |what date is |what date is it on |what day is |what day is it on |which date is |what day does |what date does |what(?:'s|s| is) the date(?: of| on| next| this)? |what(?:'s|s| is) the (?:day|date) of |when(?:'s|s) |when is |when does )";
    let (name, index, next_one) = _weekday_of(&text_, ask);
    if let (Some(name), Some(index)) = (name, index) {
        let mut ahead = (index - today.weekday() as i64).rem_euclid(7);
        if ahead == 0 && next_one {
            ahead = 7;
        }
        let when = today.plus(ahead);
        return Some(match ahead {
            0 => format!("{name} is today, {}.", _short_date(when)),
            1 => format!("{name} is tomorrow, {}.", _short_date(when)),
            7 => format!("{name} is a week away, on {}.", _short_date(when)),
            _ => format!("{name} is {ahead} days away, on {}.", _short_date(when)),
        });
    }
    None
}

/// The line without politeness and filler, so a question can be matched whole.
pub fn _plain(clean: &str) -> String {
    let t = strip_chars(text::strip(clean), "?.!");
    let t = sub(r"\s*[,;]\s*", " ", t);
    let t = sub(&format!(r"^(?:{}\b[\s]+)+", gstr("_POLITE")), "", &t);
    let t = sub(&format!(r"\b{}\b", gstr("_FILLER")), " ", &t);
    squash(&t)
}

// --- one unit in another ---------------------------------------------------------------------

static _UNIT: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("_UNIT").str_map());
static FACTS: LazyLock<Vec<((String, String), String)>> = LazyLock::new(|| {
    g("FACTS")
        .pairs()
        .iter()
        .map(|(k, v)| ((k.items()[0].str().to_string(), k.items()[1].str().to_string()), v.str().to_string()))
        .collect()
});
static HOW_HEAVY: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("HOW_HEAVY").str_map());
static _HEAVY: LazyLock<&'static Re> = LazyLock::new(|| gre("_HEAVY"));
static _TAILS: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_TAILS"));

/// The fact for this pair, whichever way round the question asked it.
pub fn _unit_pair(counted: &str, of_unit: &str) -> Option<String> {
    let get = |a: &str, b: &str| FACTS.iter().find(|((x, y), _)| x == a && y == b).map(|(_, f)| f.clone());
    get(counted, of_unit).filter(|f| !f.is_empty()).or_else(|| get(of_unit, counted))
}

/// How many of one unit make another, when that is the question asked.
pub fn _unit_fact(clean: &str) -> Option<String> {
    let text_ = _plain(clean);
    if let Some(m) = _HEAVY.fullmatch(&text_) {
        return HOW_HEAVY.get(m.str(1)).cloned();
    }
    if let Some(m) = fullmatch(gstr("_HOW_MUCH"), &text_) {
        let counted = _UNIT.get(m.str(1));
        let of_unit = _UNIT.get(m.str(2));
        if let (Some(c), Some(o)) = (counted, of_unit) {
            return _unit_pair(c, o);
        }
        return None;
    }
    let m = fullmatch(gstr("_AMOUNT"), &text_)?;
    let counted = _UNIT.get(m.str(1))?;
    for pattern in _TAILS.iter() {
        if let Some(tail) = fullmatch(pattern, m.str(2)) {
            if let Some(of_unit) = _UNIT.get(tail.str(1)) {
                return _unit_pair(counted, of_unit);
            }
        }
    }
    None
}

// --- a salary in hours, and an hourly rate in a year -----------------------------------------

static _WAGE_UNIT: LazyLock<&'static Re> = LazyLock::new(|| gre("_WAGE_UNIT"));
static _AN_HOUR: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_AN_HOUR"));
static _HOURS_WORKED: LazyLock<&'static Re> = LazyLock::new(|| gre("_HOURS_WORKED"));
static _WAGE_CUE: LazyLock<&'static Re> = LazyLock::new(|| gre("_WAGE_CUE"));
static _WAGE_CONTEXT: LazyLock<&'static Re> = LazyLock::new(|| gre("_WAGE_CONTEXT"));

/// Python `float(s)` where the caller catches ValueError (None then).
fn float_or_none(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() || !t.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    t.parse::<f64>().ok()
}

/// The hours the line says they work: "37.5 hours a week", "40 hours". 0 when it says none.
pub fn _hours_a_week(text_in: &str) -> f64 {
    for m in _HOURS_WORKED.finditer(text_in) {
        let Some(value) = float_or_none(m.str(1)) else { continue };
        if (5.0..=70.0).contains(&value) {
            return value;
        }
    }
    0.0
}

/// The rate the line states, in pounds and pence: "12.50 an hour" is not £12.
pub fn _hourly_named(text_in: &str) -> f64 {
    let pattern = format!(r"{}\s*([^.;!?]{{0,18}})", gstr("_WAGE"));
    for m in rx(&pattern, 0).finditer(text_in) {
        let tail = format!(" {}", m.get(3).unwrap_or("").to_lowercase());
        if any_in(&tail, &_AN_HOUR) {
            let Some(amount) = float_or_none(&m.str(1).replace(',', "")) else { continue };
            return if m.get(2).is_some_and(|k| !k.is_empty()) { amount * 1000.0 } else { amount };
        }
    }
    0.0
}

/// A bare salary: "on 25k", "i earn 45000". A figure with a rate unit after it is not one.
pub fn _yearly_named(text_in: &str) -> f64 {
    let mut best = 0.0;
    for m in rx(gstr("_WAGE"), 0).finditer(text_in) {
        let Some(mut amount) = float_or_none(&m.str(1).replace(',', "")) else { continue };
        if m.get(2).is_some_and(|k| !k.is_empty()) {
            amount *= 1000.0;
        }
        let after_end = text::char_index(text_in, m.end());
        let window = text::slice(text_in, after_end as isize, Some(after_end as isize + 16));
        if _WAGE_UNIT.match_(&window).is_some() {
            continue;
        }
        if amount >= 1000.0 && amount > best {
            best = amount;
        }
    }
    best
}

pub fn _pounds(value: f64) -> String {
    int_commas(round0(value))
}

/// 37.5 stays 37.5 and 12.50 stays 12.50: pence in a rate are not noise.
pub fn _rate(hours: f64) -> String {
    if (hours - round0(hours)).abs() > 1e-9 { fixed(hours, 2) } else { int_str(round0(hours)) }
}

/// The yearly figure the line names, from "30000 a year", "2200 a month" or a bare "25k".
pub fn _a_yearline(text_in: &str) -> f64 {
    let wage = gstr("_WAGE");
    for (pattern, scale) in [
        (format!(r"{wage}\s*(?:a|per|each|every)\s*year"), 1.0),
        (format!(r"{wage}\s*(?:a|per|each|every)\s*month"), 12.0),
    ] {
        for m in rx(&pattern, 0).finditer(text_in) {
            let Some(mut amount) = float_or_none(&m.str(1).replace(',', "")) else { continue };
            if m.get(2).is_some_and(|k| !k.is_empty()) {
                amount *= 1000.0;
            }
            return amount * scale;
        }
    }
    0.0
}

/// A salary in hours, or an hourly rate in a year. None when the line is not that sum.
pub fn _wage_rate(text_in: &str) -> Option<String> {
    let hours = Some(_hours_a_week(text_in)).filter(|h| *h != 0.0).unwrap_or(37.5);
    let rate = _hourly_named(text_in);
    let yearly = Some(_a_yearline(text_in)).filter(|y| *y != 0.0).unwrap_or_else(|| _yearly_named(text_in));
    let asks_a_year = has(r"\b(?:a year|per year|annual|annually|yearly|salary)\b", text_in);
    if rate != 0.0 && yearly == 0.0 && rate <= 500.0 {
        let year = rate * hours * 52.0;
        return Some(format!(
            "£{} an hour over {} hours a week is about £{} a year before tax, £{} a month. Over 40 hours a week the same rate is £{}, and paid holiday is already in the 52 weeks.",
            _rate(rate),
            _rate(hours),
            _pounds(year),
            _pounds(year / 12.0),
            _pounds(rate * 40.0 * 52.0)
        ));
    }
    if yearly >= 1000.0 {
        let hourly = yearly / (hours * 52.0);
        let mut said = format!(
            "£{} a year over {} hours a week is about £{} an hour before tax, £{} a month, £{} a week.",
            _pounds(yearly),
            _rate(hours),
            _number(round_n(hourly, 2)),
            _pounds(yearly / 12.0),
            _pounds(yearly / 52.0)
        );
        if (hours - 40.0).abs() > 0.01 {
            said += &format!(" Over 40 hours a week it is £{}.", _number(round_n(yearly / (40.0 * 52.0), 2)));
        }
        return Some(said + " Take off any unpaid lunch break first: you are only paid for the hours you work.");
    }
    if asks_a_year && has(r"\b(?:hourly|hour rate|an hour|per hour)\b", text_in) {
        return Some("Take the yearly salary and divide it by the hours you actually work: £30,000 over 37.5 hours a week for 52 weeks is £15.38 an hour, and over 40 hours it is £14.42. Do it before tax, drop any unpaid lunch break, and divide by 12 for the month.".into());
    }
    None
}

/// A computed maths reply, or None when the line is not a calculation.
///
/// `understood` may be an empty reading (the store passes `{}`), and `memory` an empty one (the
/// fallback chain passes `{}`). Every card is worked out before the first answer is taken, as
/// Python builds the whole tuple first: a card that raises makes `solve` raise (a panic here).
pub fn solve(text_in: &str, understood: &Understood, memory: &mut Memory) -> Option<String> {
    let clean = if !understood.clean.is_empty() { understood.clean.clone() } else { squash(&text_in.to_lowercase()) };
    let clean = clean.as_str();
    if dup::in_a_safety_thread(memory) {
        return None;
    }
    if _WAGE_CUE.is_match(clean) && _WAGE_CONTEXT.is_match(clean) {
        if let Some(said) = _wage(clean).or_else(|| _wage_rate(clean)) {
            if !said.is_empty() {
                return Some(said);
            }
        }
    }
    let answers: Vec<Option<String>> = vec![
        _quadratic(clean),
        _simplify(clean),
        _add_fractions(clean),
        _the_factors(clean),
        _common_factors(clean),
        _round_to(clean),
        _circle_measure(clean),
        _box_volume(clean),
        _equation(clean),
        _root(clean),
        _roots(clean),
        _prime(clean),
        _what_percent(clean),
        _percent_on(clean),
        _a_share_of_a_group(clean),
        _fraction_decimal(clean),
        _fraction(clean),
        _fraction_of(clean),
        _share(clean),
        _percent(clean),
        _percentage_change(clean),
        _percent_rise(clean),
        _percent_of_the_figure_in_the_chat(clean, memory),
        _tip(clean),
        _left_on_the_bill(clean),
        _wage(clean),
        _travel(clean),
        _at_a_pace(clean, Some(memory), false),
        _save_by_a_date(clean, memory),
        _save_time(clean),
        _saving(clean),
        _time_to_reach(clean, memory),
        _a_goal_without_a_rate(clean, memory),
        _time_at_a_new_rate(clean, memory),
        _time_at_a_new_total(clean, memory),
        _time_without_a_goal(clean, memory),
        _figure_just_given(clean, memory),
        _deadline_for_a_goal(clean, memory),
        _save_by_a_date(clean, memory),
        _each_month_for_a_goal(clean, memory),
        _a_working_week(clean),
        _convert(clean, Some(memory)),
        _unit_fact(clean),
        _average(clean),
        _area(clean),
        _perimeter_of_a_room(clean),
        _days_in_month(clean),
        _today(clean),
        _days_until(clean),
        _date_counted_from_today(clean),
        _days_until_easter(clean),
        _named_day_date(clean),
        _the_weekday_of_a_date(clean),
        _next_bank_holiday(clean),
        _days_until_a_relative_day(clean),
        _days_until_a_named_day(clean),
        _days_until_a_month_or_season(clean),
        _days_left_in_the_month_or_the_year(clean),
        _working_days_or_weekends_until(clean),
        _how_long_ago(clean),
    ];
    if let Some(answer) = answers.into_iter().flatten().find(|a| !a.is_empty()) {
        return Some(answer);
    }
    _calc(clean)
}

// --- averages --------------------------------------------------------------------------------

static _AVERAGE_FILLER: LazyLock<std::collections::HashSet<String>> = LazyLock::new(|| g("_AVERAGE_FILLER").str_set());
static _MEAN_VERB: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_MEAN_VERB"));
static _STATISTIC_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_STATISTIC_WORDS"));
static _OTHER_STATISTICS: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_OTHER_STATISTICS"));
static _POINTS_AT_A_THING: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_POINTS_AT_A_THING"));

/// The numbers the line types out, in order: "the median of 3, 5 and 10" is 3, 5 and 10.
pub fn _figures(clean: &str) -> Vec<Fraction> {
    rx(r"(?<![\w.])\d+(?:\.\d+)?", 0)
        .findall_str(clean)
        .iter()
        .map(|n| {
            Fraction::from_decimal_str(n).unwrap_or_else(|| panic!("ValueError: Invalid literal for Fraction: {n:?}"))
        })
        .collect()
}

pub fn _short(value: &Fraction) -> String {
    if value.den_is_one() {
        return value.num.to_decimal();
    }
    let s = fixed(value.to_f64(), 2);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn strip_punct(w: &str) -> &str {
    strip_chars(w, "?.!,")
}

/// Whether the line names the statistic it asks about, rather than pointing at a thing.
pub fn _names_a_statistic(asked: &str) -> bool {
    let words: Vec<&str> = text::split_ref(asked).into_iter().map(strip_punct).collect();
    for (index, word) in words.iter().enumerate() {
        if !_STATISTIC_WORDS.iter().any(|s| s == word) {
            continue;
        }
        if _MEAN_VERB.iter().any(|s| s == word) {
            let before = if index > 0 { words[index - 1] } else { "" };
            if _POINTS_AT_A_THING.iter().any(|s| s == before) {
                continue;
            }
            if words.iter().any(|other| _OTHER_STATISTICS.iter().any(|s| s == other)) {
                continue;
            }
        }
        return true;
    }
    false
}

pub fn _about_words(asked: &str) -> String {
    let mut words: Vec<&str> = text::split_ref(asked);
    if words.is_empty() {
        return String::new();
    }
    let mut i = words.len() - 1;
    while i > 0
        && _AVERAGE_FILLER.contains(strip_punct(words[i]))
        && !_MEAN_VERB.iter().any(|m| m == strip_punct(words[i]))
    {
        i -= 1;
    }
    if i > 0 && _MEAN_VERB.iter().any(|m| m == strip_punct(words[i])) {
        words.remove(i);
    }
    words.join(" ")
}

/// The averages card: the line's own numbers worked out, or the three averages explained.
pub fn _averages(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = if !understood.clean.is_empty() { &understood.clean } else { &understood.text };
    let figures = _figures(clean);
    let asked = clean.to_lowercase();
    let about = _about_words(&asked);
    if figures.len() < 2 {
        let words = rx(r"[a-z']+", 0).findall_str(&asked);
        if words.iter().all(|w| _AVERAGE_FILLER.contains(w)) && _names_a_statistic(&asked) {
            if (asked.contains("median") && about.contains("mean")) || asked.contains("difference") {
                return Some(gstr("_AVERAGE_EXPLAIN").to_string());
            }
            if asked.contains("median") {
                return Some(gstr("_MEDIAN_EXPLAIN").to_string());
            }
            if asked.contains("mode") {
                return Some(gstr("_MODE_EXPLAIN").to_string());
            }
            return Some(gstr("_AVERAGE_EXPLAIN").to_string());
        }
        return None;
    }
    let mut ordered = figures.clone();
    ordered.sort_by(|a, b| a.cmp(b)); // stable, as Python's sorted
    let listed = ordered.iter().map(_short).collect::<Vec<_>>().join(", ");
    let total = ordered.iter().fold(Fraction::int(0), |acc, f| acc.add(f));
    let mean = total.div(&Fraction::int(ordered.len() as i128));
    let middle = ordered.len() / 2;
    let median = if ordered.len() % 2 == 1 {
        ordered[middle].clone()
    } else {
        ordered[middle - 1].add(&ordered[middle]).div(&Fraction::int(2))
    };
    let mut counts: Vec<(Fraction, usize)> = Vec::new();
    for figure in &ordered {
        match counts.iter_mut().find(|(f, _)| f.eq(figure)) {
            Some(entry) => entry.1 += 1,
            None => counts.push((figure.clone(), 1)),
        }
    }
    let top = counts.iter().map(|(_, c)| *c).max().unwrap();
    if asked.contains("median") {
        return Some(format!("The median of {listed} is {}: in order, that is the middle one.", _short(&median)));
    }
    if asked.contains("mode") {
        if top == 1 {
            return Some(format!(
                "No number in {listed} repeats, so there is no mode. The mean is {} and the median is {}.",
                _short(&mean),
                _short(&median)
            ));
        }
        let mode = &counts.iter().find(|(_, c)| *c == top).unwrap().0;
        return Some(format!("The mode of {listed} is {}, the number that appears most often.", _short(mode)));
    }
    Some(format!(
        "The mean of {listed} is {}: they add to {} and {} ÷ {} = {}.",
        _short(&mean),
        _short(&total),
        _short(&total),
        ordered.len(),
        _short(&mean)
    ))
}

// --- the rounding card's gate, the spare hour, and primes ------------------------------------

static _ROUND_IS_AROUND: LazyLock<&'static Re> = LazyLock::new(|| gre("_ROUND_IS_AROUND"));
static ROUNDING_NAMED: LazyLock<&'static Re> = LazyLock::new(|| gre("ROUNDING_NAMED"));
static DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));

/// True when the line is about rounding a number, not about working round something.
pub fn _a_rounding_question(understood: &Understood) -> bool {
    let line = if !understood.clean.is_empty() { &understood.clean } else { &understood.text }.to_lowercase();
    if DIGIT.is_match(&line) {
        return true;
    }
    ROUNDING_NAMED.is_match(&line)
}

/// The card needs the verb and needs a number or the name: "round work" is neither.
pub fn _rounding_is_the_ask(understood: &Understood) -> bool {
    _round_is_the_verb(understood) && _a_rounding_question(understood)
}

/// True when "round" in the line is the maths verb rather than a way of saying "around".
pub fn _round_is_the_verb(understood: &Understood) -> bool {
    let said = format!("{} {} {}", understood.clean, understood.norm_text, understood.text).to_lowercase();
    !_ROUND_IS_AROUND.is_match(&said)
}

static _FREE_TIME_ASK: LazyLock<Vec<String>> = LazyLock::new(|| gstrs("_FREE_TIME_ASK"));
static _A_LENGTH_OF_TIME: LazyLock<&'static Re> = LazyLock::new(|| gre("_A_LENGTH_OF_TIME"));
static _A_PAY_WORD: LazyLock<&'static Re> = LazyLock::new(|| gre("_A_PAY_WORD"));

/// False when the line asks what to do with the hour rather than what it pays.
pub fn _not_a_spare_hour(understood: &Understood) -> bool {
    let line = if !understood.clean.is_empty() { &understood.clean } else { &understood.text }.to_lowercase();
    if any_in(&line, &_FREE_TIME_ASK) {
        return false;
    }
    if _A_LENGTH_OF_TIME.is_match(&line) && !_A_PAY_WORD.is_match(&line) {
        return false;
    }
    true
}

static A_NUMBER_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b\d+\b", 0));
static ASKS_HOW_TO_TEST: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(how|work out|check|test|tell|know)\b", 0));

/// The definition, with the test of a number added only when the line asks how to test one.
pub fn _prime_definition(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = squash(&if !understood.clean.is_empty() { &understood.clean } else { &understood.text }.to_lowercase());
    if A_NUMBER_WORD.is_match(&said) {
        return None;
    }
    if ASKS_HOW_TO_TEST.is_match(&said) {
        return Some(format!("{}{}", gstr("PRIME_DEFINITION"), gstr("PRIME_TEST")));
    }
    Some(gstr("PRIME_DEFINITION").to_string())
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.maths:_not_a_spare_hour:5342", |_: &FnRef, u: &Understood| _not_a_spare_hour(u)),
    ("bot.knowledge.maths:_rounding_is_the_ask:5305", |_: &FnRef, u: &Understood| _rounding_is_the_ask(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.maths:_averages:5180", |_: &FnRef, m: &mut Memory, u: &Understood| _averages(m, u)),
    ("bot.knowledge.maths:_prime_definition:5363", |_: &FnRef, m: &mut Memory, u: &Understood| _prime_definition(m, u)),
];
