//! Python's `str` predicates (`isalpha`, `isdigit`, `isupper`, ...) with CPython's own Unicode
//! tables (Unicode 15.0, exported by `scratchpad/make_pyuni.py` into `pyuni.txt`), so non-English
//! text reads exactly as PyBot reads it. ASCII takes a fast path.

use std::sync::LazyLock;

struct Tables {
    alpha: Vec<(u32, u32)>,
    digit: Vec<(u32, u32)>,
    decimal: Vec<(u32, u32)>,
    numeric: Vec<(u32, u32)>,
    lower: Vec<(u32, u32)>,
    upper: Vec<(u32, u32)>,
    title: Vec<(u32, u32)>,
}

static TABLES: LazyLock<Tables> = LazyLock::new(|| {
    let mut t = Tables { alpha: vec![], digit: vec![], decimal: vec![], numeric: vec![], lower: vec![], upper: vec![], title: vec![] };
    for line in include_str!("pyuni.txt").lines() {
        let mut parts = line.split(' ');
        let name = parts.next().unwrap_or("");
        let ranges: Vec<(u32, u32)> = parts
            .filter(|p| !p.is_empty())
            .map(|p| {
                let (a, b) = p.split_once('-').unwrap();
                (u32::from_str_radix(a, 16).unwrap(), u32::from_str_radix(b, 16).unwrap())
            })
            .collect();
        match name {
            "alpha" => t.alpha = ranges,
            "digit" => t.digit = ranges,
            "decimal" => t.decimal = ranges,
            "numeric" => t.numeric = ranges,
            "lower" => t.lower = ranges,
            "upper" => t.upper = ranges,
            "title" => t.title = ranges,
            _ => {}
        }
    }
    t
});

fn within(ranges: &[(u32, u32)], c: char) -> bool {
    let c = c as u32;
    ranges.binary_search_by(|(a, b)| if *b < c { std::cmp::Ordering::Less } else if *a > c { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Equal }).is_ok()
}

/// `c.isalpha()` for one character.
pub fn ch_alpha(c: char) -> bool {
    if c.is_ascii() { c.is_ascii_alphabetic() } else { within(&TABLES.alpha, c) }
}
/// `c.isdigit()` for one character.
pub fn ch_digit(c: char) -> bool {
    if c.is_ascii() { c.is_ascii_digit() } else { within(&TABLES.digit, c) }
}
/// `c.isdecimal()`.
pub fn ch_decimal(c: char) -> bool {
    if c.is_ascii() { c.is_ascii_digit() } else { within(&TABLES.decimal, c) }
}
/// `c.isnumeric()`.
pub fn ch_numeric(c: char) -> bool {
    if c.is_ascii() { c.is_ascii_digit() } else { within(&TABLES.numeric, c) }
}
/// `c.isalnum()`.
pub fn ch_alnum(c: char) -> bool {
    ch_alpha(c) || ch_decimal(c) || ch_digit(c) || ch_numeric(c)
}
/// A lower-case character (`c.islower()`).
pub fn ch_lower(c: char) -> bool {
    if c.is_ascii() { c.is_ascii_lowercase() } else { within(&TABLES.lower, c) }
}
/// An upper-case character (`c.isupper()`).
pub fn ch_upper(c: char) -> bool {
    if c.is_ascii() { c.is_ascii_uppercase() } else { within(&TABLES.upper, c) }
}
fn ch_title(c: char) -> bool {
    !c.is_ascii() && within(&TABLES.title, c)
}

/// `s.isalpha()`.
pub fn is_alpha(s: &str) -> bool {
    !s.is_empty() && s.chars().all(ch_alpha)
}
/// `s.isdigit()`.
pub fn is_digit(s: &str) -> bool {
    !s.is_empty() && s.chars().all(ch_digit)
}
/// `s.isalnum()`.
pub fn is_alnum(s: &str) -> bool {
    !s.is_empty() && s.chars().all(ch_alnum)
}
/// `s.isupper()`: no lower-case or title-case character, and at least one upper-case one.
pub fn is_upper(s: &str) -> bool {
    let mut cased = false;
    for c in s.chars() {
        if ch_lower(c) || ch_title(c) {
            return false;
        }
        if ch_upper(c) {
            cased = true;
        }
    }
    cased
}
/// `s.islower()`.
pub fn is_lower(s: &str) -> bool {
    let mut cased = false;
    for c in s.chars() {
        if ch_upper(c) || ch_title(c) {
            return false;
        }
        if ch_lower(c) {
            cased = true;
        }
    }
    cased
}
