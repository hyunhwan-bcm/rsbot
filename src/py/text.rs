//! Python `str` methods the port leans on, with Python's behaviour where it differs from Rust's.
//!
//! Positions are byte offsets unless a function says it counts characters (code points), which
//! is what Python's indices are. On ASCII the two are the same.

/// Python's `str.isspace()` for one character: Unicode White_Space plus the four ASCII
/// separators (0x1c-0x1f) Python also counts.
pub fn is_space(c: char) -> bool {
    c.is_whitespace() || ('\x1c'..='\x1f').contains(&c)
}

/// `s.split()`: runs of whitespace, no empty pieces.
pub fn split(s: &str) -> Vec<String> {
    s.split(is_space).filter(|p| !p.is_empty()).map(str::to_string).collect()
}

/// `s.split()` as borrowed pieces.
pub fn split_ref(s: &str) -> Vec<&str> {
    s.split(is_space).filter(|p| !p.is_empty()).collect()
}

/// `" ".join(s.split())`: whitespace collapsed to single spaces, ends trimmed.
pub fn squash(s: &str) -> String {
    split_ref(s).join(" ")
}

/// `s.split(sep)` with a literal separator (keeps empty pieces, as Python does).
pub fn split_on(s: &str, sep: &str) -> Vec<String> {
    s.split(sep).map(str::to_string).collect()
}

/// `s.split(sep, maxsplit)`.
pub fn splitn_on(s: &str, sep: &str, maxsplit: usize) -> Vec<String> {
    s.splitn(maxsplit + 1, sep).map(str::to_string).collect()
}

/// `s.rsplit(sep, maxsplit)`.
pub fn rsplitn_on(s: &str, sep: &str, maxsplit: usize) -> Vec<String> {
    let mut v: Vec<String> = s.rsplitn(maxsplit + 1, sep).map(str::to_string).collect();
    v.reverse();
    v
}

/// `s.lower()`.
pub fn lower(s: &str) -> String {
    s.to_lowercase()
}

/// `s.upper()`.
pub fn upper(s: &str) -> String {
    s.to_uppercase()
}

/// `s.strip()`.
pub fn strip(s: &str) -> &str {
    s.trim_matches(is_space)
}

/// `s.lstrip()`.
pub fn lstrip(s: &str) -> &str {
    s.trim_start_matches(is_space)
}

/// `s.rstrip()`.
pub fn rstrip(s: &str) -> &str {
    s.trim_end_matches(is_space)
}

/// `s.strip(chars)`.
pub fn strip_chars<'a>(s: &'a str, chars: &str) -> &'a str {
    s.trim_matches(|c| chars.contains(c))
}

/// `s.lstrip(chars)`.
pub fn lstrip_chars<'a>(s: &'a str, chars: &str) -> &'a str {
    s.trim_start_matches(|c| chars.contains(c))
}

/// `s.rstrip(chars)`.
pub fn rstrip_chars<'a>(s: &'a str, chars: &str) -> &'a str {
    s.trim_end_matches(|c| chars.contains(c))
}

/// `s.capitalize()`: first character upper-cased, the rest lower-cased.
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => {
            let mut out: String = f.to_uppercase().collect();
            out.push_str(&c.as_str().to_lowercase());
            out
        }
    }
}

/// The first character upper-cased and the rest left alone (`s[:1].upper() + s[1:]`).
pub fn upper_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

/// `s.title()`: a letter after a non-letter is upper-cased, every other letter lower-cased.
/// (Python's rule: "they're" -> "They'Re".)
pub fn title(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_cased = false;
    for c in s.chars() {
        if prev_cased {
            out.extend(c.to_lowercase());
        } else {
            out.extend(c.to_uppercase());
        }
        prev_cased = c.is_alphabetic();
    }
    out
}

/// `s.isdigit()`.
pub fn is_digit(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_numeric() && c.to_digit(10).is_some() || c.is_ascii_digit())
}

/// `s.isalpha()`.
pub fn is_alpha(s: &str) -> bool {
    !s.is_empty() && s.chars().all(char::is_alphabetic)
}

/// `s.isalnum()`.
pub fn is_alnum(s: &str) -> bool {
    !s.is_empty() && s.chars().all(char::is_alphanumeric)
}

/// `s.isupper()`: at least one cased character and no lower-case one.
pub fn is_upper(s: &str) -> bool {
    s.chars().any(char::is_uppercase) && !s.chars().any(char::is_lowercase)
}

/// `s.islower()`.
pub fn is_lower(s: &str) -> bool {
    s.chars().any(char::is_lowercase) && !s.chars().any(char::is_uppercase)
}

/// Number of characters (Python's `len(s)`).
pub fn len(s: &str) -> usize {
    s.chars().count()
}

/// `s[start:end]` with Python's character indices (negative counts from the end, clamped).
pub fn slice(s: &str, start: isize, end: Option<isize>) -> String {
    let n = s.chars().count() as isize;
    let norm = |i: isize| -> usize { (if i < 0 { (n + i).max(0) } else { i.min(n) }) as usize };
    let a = norm(start);
    let b = end.map(norm).unwrap_or(n as usize);
    if a >= b {
        return String::new();
    }
    s.chars().skip(a).take(b - a).collect()
}

/// Byte offset of the `n`th character (or the end).
pub fn byte_at(s: &str, n: usize) -> usize {
    s.char_indices().nth(n).map(|(b, _)| b).unwrap_or(s.len())
}

/// Character index of a byte offset (Python's index for a Rust match position).
pub fn char_index(s: &str, byte: usize) -> usize {
    s[..byte].chars().count()
}

/// `s.find(sub)` as a character index, or -1.
pub fn find(s: &str, sub: &str) -> isize {
    match s.find(sub) {
        Some(b) => char_index(s, b) as isize,
        None => -1,
    }
}

/// `s.rfind(sub)` as a character index, or -1.
pub fn rfind(s: &str, sub: &str) -> isize {
    match s.rfind(sub) {
        Some(b) => char_index(s, b) as isize,
        None => -1,
    }
}

/// `s.count(sub)` (non-overlapping).
pub fn count(s: &str, sub: &str) -> usize {
    if sub.is_empty() {
        return s.chars().count() + 1;
    }
    s.matches(sub).count()
}

/// `s.partition(sep)`.
pub fn partition<'a>(s: &'a str, sep: &str) -> (&'a str, &'a str, &'a str) {
    match s.find(sep) {
        Some(i) => (&s[..i], &s[i..i + sep.len()], &s[i + sep.len()..]),
        None => (s, "", ""),
    }
}

/// `s.rpartition(sep)`.
pub fn rpartition<'a>(s: &'a str, sep: &str) -> (&'a str, &'a str, &'a str) {
    match s.rfind(sep) {
        Some(i) => (&s[..i], &s[i..i + sep.len()], &s[i + sep.len()..]),
        None => ("", "", s),
    }
}

/// `s.endswith(tuple)`.
pub fn ends_with_any<S: AsRef<str>>(s: &str, ends: &[S]) -> bool {
    ends.iter().any(|e| s.ends_with(e.as_ref()))
}

/// `s.startswith(tuple)`.
pub fn starts_with_any<S: AsRef<str>>(s: &str, starts: &[S]) -> bool {
    starts.iter().any(|e| s.starts_with(e.as_ref()))
}

/// `s.translate(table)` for a char -> replacement table.
pub fn translate(s: &str, table: &[(char, &str)]) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match table.iter().find(|(k, _)| *k == c) {
            Some((_, r)) => out.push_str(r),
            None => out.push(c),
        }
    }
    out
}

/// Python's `str(float)` / `repr(float)` for the values PyBot formats (shortest round-trip,
/// with ".0" on whole numbers, exponent form outside 1e-4..1e16).
pub fn float_repr(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let a = x.abs();
    if a != 0.0 && !(1e-4..1e16).contains(&a) {
        // Rust's {:e} is shortest round-trip too; Python writes e-05 / e+16
        let s = format!("{:e}", x);
        let (m, e) = s.split_once('e').unwrap();
        let e: i32 = e.parse().unwrap();
        let m = if m.contains('.') { m.to_string() } else { m.to_string() };
        return format!("{}e{}{:02}", m, if e < 0 { '-' } else { '+' }, e.abs());
    }
    let s = format!("{}", x);
    if s.contains('.') { s } else { format!("{s}.0") }
}

/// Python's `round(x)` (banker's rounding) as an integer.
pub fn round_half_even(x: f64) -> i64 {
    let r = x.round();
    if (x - x.trunc()).abs() == 0.5 {
        let t = x.trunc();
        if (t as i64) % 2 == 0 { t as i64 } else { r as i64 }
    } else {
        r as i64
    }
}

/// `"{:,}".format(n)`.
pub fn thousands(n: i64) -> String {
    let s = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn python_like() {
        assert_eq!(title("they're here"), "They'Re Here");
        assert_eq!(capitalize("hELLO world"), "Hello world");
        assert_eq!(squash("  a \n b  "), "a b");
        assert_eq!(float_repr(2.0), "2.0");
        assert_eq!(float_repr(0.1), "0.1");
        assert_eq!(float_repr(1e-5), "1e-05");
        assert_eq!(round_half_even(2.5), 2);
        assert_eq!(round_half_even(3.5), 4);
        assert_eq!(thousands(1234567), "1,234,567");
        assert_eq!(slice("hello", -3, None), "llo");
    }
}
