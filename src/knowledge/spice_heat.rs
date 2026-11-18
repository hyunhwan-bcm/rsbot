//! Port of `bot/knowledge/spice_heat.py`: how hot a spice is - one row per spice, keyed by the
//! names it is written as and the heat words.

use crate::data;
use crate::knowledge::built;
use crate::knowledge::entry::{Entry, Reply};

/// `_rows()`: one row per spice in `HEAT`, the names and the verdict in the same shape.
pub fn _rows() -> Vec<Entry> {
    let hot = data::get("bot.knowledge.spice_heat.HOT").strs();
    let not_a_spice_line = data::get("bot.knowledge.spice_heat.NOT_A_SPICE_LINE").clone();
    let more: Vec<String> = [
        "instead", "substitute", "ran out", "my curry", "too spicy", "too hot", "what does a dash", "what is a dash", "how much is a dash", "dash mean",
        "what does a pinch", "what is a pinch", "how much is a pinch", "pinch mean",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    data::get("bot.knowledge.spice_heat.HEAT")
        .items()
        .iter()
        .map(|row| {
            let names = row.items()[0].strs();
            built::entry(
                &format!("spice-heat-{}", names[0].replace(' ', "-").replace('\'', "")),
                vec![built::part(&names), built::part(&hot)],
                Reply::Text(row.items()[1].str().to_string()),
                &[not_a_spice_line.clone(), built::words(&more)],
                1,
                vec![],
                None,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn rows_rebuild() {
        crate::data::load();
        crate::knowledge::built::assert_rows_match(&super::_rows(), "bot.knowledge.spice_heat.ENTRIES");
    }
}
