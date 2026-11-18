//! Port of `bot/knowledge/meat_in_a_card.py`: the meat a card uses as an ingredient (chicken
//! stock, meat juices) swapped for the plant version, for a person who eats neither meat nor
//! fish. A helper, not a domain: no rows of its own.

use crate::data;
use crate::py::text;
use std::sync::LazyLock;

/// `MEAT_IN_A_CARD_SWAPS`: (pattern, the plant version).
static MEAT_IN_A_CARD_SWAPS: LazyLock<Vec<(&'static crate::py::re::Re, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.meat_in_a_card.MEAT_IN_A_CARD_SWAPS")
        .items()
        .iter()
        .map(|r| (r.items()[0].as_re().expect("a regex"), r.items()[1].str().to_string()))
        .collect()
});

/// `swapped_out(text, line, avoided)`: the reply with the meat it uses as an ingredient swapped,
/// or `text` unchanged. The contract passes the avoided families as a slice (Python's set, whose
/// order the test does not read); `swapped_out_of` takes Python's `avoided=None` as well.
pub fn swapped_out(text_: &str, line: &str, avoided: &[String]) -> String {
    swapped_out_of(text_, line, Some(avoided))
}

/// `swapped_out(text, line="", avoided=None)`, with `None` for "nothing said about it".
pub fn swapped_out_of(text_: &str, line: &str, avoided: Option<&[String]>) -> String {
    if text::strip(text_).is_empty() {
        return text_.to_string();
    }
    let meat_or_fish = data::get("bot.knowledge.meat_in_a_card.MEAT_OR_FISH").strs();
    if let Some(avoided) = avoided {
        if !avoided.iter().any(|a| meat_or_fish.contains(a)) {
            return text_.to_string();
        }
    }
    if data::re("bot.knowledge.cooking.PLANT_SAID").is_match(text_) {
        return text_.to_string();
    }
    let said_here = text::lower(line);
    let mut out = text_.to_string();
    for (pattern, swap) in MEAT_IN_A_CARD_SWAPS.iter() {
        if pattern.is_match(&said_here) {
            continue;
        }
        out = crate::knowledge::cooking::_rewritten_outside_swaps(&out, pattern, swap, None);
    }
    out
}
