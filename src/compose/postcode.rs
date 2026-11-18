//! Port of `bot/compose/postcode.py`: a fact only the person's postcode can answer ("who is my
//! MP", "when is my bin day") gets the honest line and the one place that takes a postcode.

use crate::data::{self, Value};
use crate::py::re::Re;
use std::sync::LazyLock;

/// `POSTCODE_LOOKUPS`: (name, pattern, options), in order.
static POSTCODE_LOOKUPS: LazyLock<Vec<(&'static Re, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.postcode.POSTCODE_LOOKUPS")
        .items()
        .iter()
        .map(|row| {
            let row: &'static [Value] = row.items();
            (row[1].as_re().unwrap(), row[2].strs())
        })
        .collect()
});
static POSTCODE_NOT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.postcode.POSTCODE_NOT"));

/// Python: bot/compose/postcode.py:376 `a_postcode_lookup(clean)`
///
/// The honest line for a fact that only the person's postcode can answer, or None.
pub fn a_postcode_lookup(clean: &str) -> Option<Vec<String>> {
    if POSTCODE_NOT.is_match(clean) {
        return None;
    }
    POSTCODE_LOOKUPS.iter().find(|(pattern, _)| pattern.is_match(clean)).map(|(_, options)| options.clone())
}
