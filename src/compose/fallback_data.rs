//! Port of `bot/compose/fallback_data.py`: the composer's pools are data (read from the bundle
//! by the files that use them); the two readers it defines are here.

use crate::data::{self, Value};
use crate::py::re::{Re, I};
use crate::py::text;
use std::sync::LazyLock;

/// `THE_PLACE_POOLS`: (pattern text, pool), searched with `re.I` at run time.
static THE_PLACE_POOLS: LazyLock<Vec<(Re, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback_data.THE_PLACE_POOLS")
        .items()
        .iter()
        .map(|row| {
            let row: &[Value] = row.items();
            (Re::new(row[0].str(), I), row[1].strs())
        })
        .collect()
});
/// `THE_BOTS_OWN_ANSWER`: (compiled pattern, pool).
static THE_BOTS_OWN_ANSWER: LazyLock<Vec<(&'static Re, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback_data.THE_BOTS_OWN_ANSWER")
        .items()
        .iter()
        .map(|row| {
            let row: &'static [Value] = row.items();
            (row[0].as_re().unwrap(), row[1].strs())
        })
        .collect()
});

/// `" " + " ".join(clean.lower().split()) + " "`
fn padded(clean: &str) -> String {
    format!(" {} ", text::squash(&text::lower(clean)))
}

/// Python: bot/compose/fallback_data.py:8608 `the_place_the_answer_is_kept(clean)`
///
/// The pool for where this kind of answer is kept, or None when the line names no kind.
pub fn the_place_the_answer_is_kept(clean: &str) -> Option<Vec<String>> {
    let said = padded(clean);
    THE_PLACE_POOLS.iter().find(|(pattern, _)| pattern.is_match(&said)).map(|(_, pool)| pool.clone())
}

/// Python: bot/compose/fallback_data.py:8725 `the_bots_own_answer(clean)`
///
/// The bot's answer about itself, or None when the line is not the bot being asked about itself.
pub fn the_bots_own_answer(clean: &str) -> Option<Vec<String>> {
    let said = padded(clean);
    THE_BOTS_OWN_ANSWER.iter().find(|(pattern, _)| pattern.is_match(&said)).map(|(_, pool)| pool.clone())
}
