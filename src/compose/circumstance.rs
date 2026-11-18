//! Port of `bot/compose/circumstance.py`: a whole line that states the person's situation
//! ("im a carer", "we rent") is acknowledged and asked about, not filed as a bare fact.

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use std::sync::LazyLock;

static _TONE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.circumstance._TONE"));
/// `_SITUATIONS`: (kind, pattern, lines), in order.
static _SITUATIONS: LazyLock<Vec<(String, &'static Re, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.circumstance._SITUATIONS")
        .items()
        .iter()
        .map(|row| {
            let row: &'static [Value] = row.items();
            (row[0].str().to_string(), row[1].as_re().unwrap(), row[2].strs())
        })
        .collect()
});
/// `re.sub(r"\s+", " ", ...)`, written inline in `_line`.
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));

/// Python: bot/compose/circumstance.py:99 `_line(clean)`
///
/// The line as this reader wants it: lower case, no edge punctuation, no tone on the end.
pub fn _line(clean: &str) -> String {
    let said = text::lower(text::strip(&clean.replace('\u{2019}', "'")));
    let said = text::strip_chars(&said, " .,!;:-\"'");
    let mut said = SPACES.sub(" ", said);
    loop {
        let trimmed = _TONE.sub("", &said);
        if trimmed == said {
            return said;
        }
        said = trimmed;
    }
}

/// Python: bot/compose/circumstance.py:110 `reads_as_situation(clean)`
///
/// The kind of situation the whole line is, or None.
pub fn reads_as_situation(clean: &str) -> Option<String> {
    let said = _line(clean);
    if said.is_empty() || said.contains('?') {
        return None;
    }
    _SITUATIONS.iter().find(|(_, pattern, _)| pattern.match_(&said).is_some()).map(|(kind, _, _)| kind.clone())
}

/// Python: bot/compose/circumstance.py:124 `pool(clean)`
///
/// The lines that answer this situation, or None when the line is not one.
pub fn pool(clean: &str) -> Option<Vec<String>> {
    let kind = reads_as_situation(clean).filter(|k| !k.is_empty())?;
    // `_POOLS[kind]`
    _SITUATIONS.iter().find(|(k, _, _)| *k == kind).map(|(_, _, lines)| lines.clone())
}
