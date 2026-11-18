//! Port of `bot/knowledge/procedures.py`: what happens at a medical procedure - one row per
//! procedure, read as an explanation and as a stated complaint.

use crate::data;
use crate::knowledge::built;
use crate::knowledge::entry::{Entry, KeyPart, Reply};

/// `_a_row(name, variants, reply, extra_avoid=(), bias=0)`.
pub fn _a_row(name: &str, variants: &[String], reply: &str, extra_avoid: &[String], bias: i64) -> Entry {
    let g = |n: &str| data::get(&format!("bot.knowledge.procedures.{n}")).clone();
    let mut row = built::entry(
        name,
        vec![built::part(variants), built::part(&g("_WHAT_HAPPENS").strs())],
        Reply::Text(reply.to_string()),
        &[g("_NOT_A_REQUEST_TO_EXPLAIN"), g("_NOT_AN_EMERGENCY"), g("_NOT_A_PERSON"), built::words(extra_avoid)],
        bias,
        variants.iter().cloned().map(KeyPart::Word).collect(),
        None,
    );
    row.explain = true;
    row.report = true;
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_rebuild() {
        data::load();
        let fixed = crate::knowledge::lookup::_avoid_words(&["_NOT_A_REQUEST_TO_EXPLAIN", "_NOT_AN_EMERGENCY", "_NOT_A_PERSON"].map(|n| data::get(&format!("bot.knowledge.procedures.{n}")).clone())).len();
        let rows: Vec<Entry> = built::exported("bot.knowledge.procedures.ENTRIES")
            .iter()
            .filter(|r| r.explain && r.report)
            .map(|r| {
                let crate::knowledge::entry::KeyPart::Any(variants) = &r.key[0] else { panic!() };
                let Reply::Text(said) = &r.reply else { panic!() };
                _a_row(&r.id, variants, said, &r.avoid[fixed..], r.bias)
            })
            .collect();
        built::assert_rows_match(&rows, "bot.knowledge.procedures.ENTRIES");
    }
}
