//! Port of `bot/knowledge/idioms.py`: what an everyday idiom means - one row per idiom, the
//! phrase in every key group so it outscores the body row its words would otherwise reach.

use crate::data::{self, Value};
use crate::knowledge::built;
use crate::knowledge::entry::{Entry, Reply};

/// An id from the phrase: its letters and figures, the rest as hyphens.
pub fn _slug(phrase: &str) -> String {
    let kept: String = phrase.chars().map(|c| if c.is_alphanumeric() { c } else { ' ' }).collect();
    crate::py::text::split(&kept).join("-")
}

/// `_idiom(row)`: one idiom from an `_IDIOMS` row (phrase, also, said[, avoid]).
pub fn _idiom(row: &Value) -> Entry {
    let r = row.items();
    let phrase = r[0].str().to_string();
    let mut words = vec![phrase.clone()];
    words.extend(r[1].strs().into_iter().filter(|one| *one != phrase));
    let neighbours = data::get("bot.knowledge.idioms._LITERAL_NEIGHBOURS").get(&phrase).cloned().unwrap_or(Value::None);
    let own = if r.len() > 3 { r[3].clone() } else { Value::None };
    built::entry(
        &format!("idiom-{}", _slug(&phrase)),
        vec![built::part(&words), built::part(&words), built::part(&words)],
        Reply::Text(r[2].str().to_string()),
        &[data::get("bot.knowledge.idioms._NOT_THE_EXPRESSION").clone(), neighbours, own],
        1,
        vec![],
        None,
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn rows_rebuild() {
        crate::data::load();
        let rows: Vec<_> = crate::data::get("bot.knowledge.idioms._IDIOMS").items().iter().map(super::_idiom).collect();
        crate::knowledge::built::assert_rows_match(&rows, "bot.knowledge.idioms.ENTRIES");
    }
}
