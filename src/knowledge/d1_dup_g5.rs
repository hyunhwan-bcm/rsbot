//! DUP ports (group 5 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::understood::Understood;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
/// `date.today()` as (year, month, day), from the shared clock (`py::clock`).
pub fn today() -> (i64, u32, u32) {
    use chrono::Datelike;
    let d = crate::py::clock::today();
    (d.year() as i64, d.month(), d.day())
}

/// `date.today().month`.
pub fn this_month() -> u32 {
    today().1
}

// ---------------------------------------------------------------------------------------------
// Python's `int(s)` for the digit runs a regex `\d` captures (any Unicode decimal digit).

/// The value of one Unicode decimal digit (category Nd), as `int()` reads it.
fn digit_value(c: char) -> Option<u32> {
    if let Some(d) = c.to_digit(10) {
        return Some(d);
    }
    // The zeros of the Nd blocks; every Nd digit is its block's zero plus 0..9.
    const ZEROS: &[u32] = &[
        0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6, 0x0D66, 0x0DE6,
        0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80, 0x1A90, 0x1B50, 0x1BB0,
        0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0, 0xFF10, 0x104A0, 0x10D30,
        0x11066, 0x110F0, 0x11136, 0x111D0, 0x112F0, 0x11450, 0x114D0, 0x11650, 0x116C0, 0x11730, 0x118E0,
        0x11950, 0x11C50, 0x11D50, 0x11DA0, 0x16A60, 0x16AC0, 0x16B50, 0x1D7CE, 0x1D7D8, 0x1D7E2, 0x1D7EC,
        0x1D7F6, 0x1E140, 0x1E2F0, 0x1E950, 0x1FBF0,
    ];
    let cp = c as u32;
    ZEROS.iter().find(|&&z| cp >= z && cp < z + 10).map(|&z| cp - z)
}

/// `int(s)` for a run of decimal digits (what `(\d+)` captured); None when it is not one.
pub fn py_int(s: &str) -> Option<i64> {
    if s.is_empty() {
        return None;
    }
    let mut n: i64 = 0;
    for c in s.chars() {
        n = n.checked_mul(10)?.checked_add(digit_value(c)? as i64)?;
    }
    Some(n)
}

// ---------------------------------------------------------------------------------------------
// bot/compose/repair.py: the deed a person owns up to.

static PET_OR_PROFESSIONAL: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.compose.repair.PET_OR_PROFESSIONAL"));
static PHYSICAL_HARM: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.PHYSICAL_HARM"));
static GRIEF: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._GRIEF"));
static A_BABY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._A_BABY"));
static REGRET_FIRST: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._REGRET_FIRST"));
static NOT_A_DEED: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.NOT_A_DEED"));
static NOBODY_TO_REPAIR: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.NOBODY_TO_REPAIR"));

/// A `((name, pattern), ...)` table of the bundle.
fn named_patterns(name: &str) -> Vec<(String, &'static Re)> {
    data::get(name)
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            Some((row.first()?.as_str()?.to_string(), row.get(1)?.as_re()?))
        })
        .collect()
}

static LAPSES: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| named_patterns("bot.compose.repair.LAPSES"));
static DEEDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| named_patterns("bot.compose.repair.DEEDS"));

/// DUP(bot.compose.repair::deed_of): the deed the person reports doing to somebody, or None.
/// `clean` is the line (a reading's `clean or text`).
pub fn deed_of(clean: &str) -> Option<String> {
    let text = text::squash(&clean.to_lowercase().replace('\u{2019}', "'"));
    if text.is_empty() {
        return None;
    }
    if PET_OR_PROFESSIONAL.is_match(&text) || PHYSICAL_HARM.is_match(&text) {
        return None;
    }
    if GRIEF.is_match(&text) {
        return None;
    }
    if A_BABY.is_match(&text) {
        return None;
    }
    if REGRET_FIRST.is_match(&text) {
        return None;
    }
    for (deed, pattern) in LAPSES.iter() {
        if pattern.is_match(&text) {
            return Some(deed.clone());
        }
    }
    if NOT_A_DEED.is_match(&text) || NOBODY_TO_REPAIR.is_match(&text) {
        return None;
    }
    for (deed, pattern) in DEEDS.iter() {
        if pattern.is_match(&text) {
            return Some(deed.clone());
        }
    }
    None
}

/// DUP(bot.compose.repair::owns_the_deed): the line reports something the person did to
/// somebody close (given a reading, which `deed_of` reads as `clean or text`).
pub fn owns_the_deed(reading: &Understood) -> bool {
    deed_of(crate::knowledge::d1_shared::clean_or_text(reading)).is_some()
}

// ---------------------------------------------------------------------------------------------
// bot/compose/draft.py: the sample subject line of the email-shape card.

static SUBJECT_EXAMPLES: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| {
    data::get("bot.compose.draft.SUBJECT_EXAMPLES")
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            Some((row.first()?.as_re()?, row.get(1)?.as_str()?.to_string()))
        })
        .collect()
});

/// DUP(bot.compose.draft::example_subject): the sample subject line for the email-shape card,
/// in the world the line names (a trade, a school, a surgery, a landlord, work, money); the
/// invoice when it names none.
pub fn example_subject(understood: &Understood) -> String {
    let line = crate::knowledge::d1_shared::clean_or_text(understood);
    for (pattern, example) in SUBJECT_EXAMPLES.iter() {
        if pattern.is_match(line) {
            return example.clone();
        }
    }
    "Invoice 2144 - payment due".to_string()
}

// ---------------------------------------------------------------------------------------------
// Python's `repr` of a string and of a list of strings (what `str(a_list)` prints).

/// `repr(s)` for a str: single quotes unless the text has a single quote and no double one.
pub fn py_repr_str(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `str(list_of_strings)`: "['a', 'b']".
pub fn py_repr_str_list<S: AsRef<str>>(items: &[S]) -> String {
    let parts: Vec<String> = items.iter().map(|s| py_repr_str(s.as_ref())).collect();
    format!("[{}]", parts.join(", "))
}

/// Python's `sorted(keys, key=len, reverse=True)`: longest first, ties in their given order.
pub fn longest_first(keys: &[String]) -> Vec<String> {
    let mut v: Vec<String> = keys.to_vec();
    v.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    v
}
