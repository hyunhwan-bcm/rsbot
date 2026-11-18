//! Port of `bot/compose/for_the_day.py`: a question about today's weather that is really a
//! decision ("is it too hot to run"): what decides it, then the place that has the figure.

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use std::sync::LazyLock;

static A_DAY_WEIGHED: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.for_the_day.A_DAY_WEIGHED"));
static A_DAY_ASKED: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.for_the_day.A_DAY_ASKED"));
static A_COLD_DAY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.for_the_day.A_COLD_DAY"));
static TOO_HOT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.for_the_day.TOO_HOT"));
static TOO_COLD: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.for_the_day.TOO_COLD"));

/// Python: bot/compose/for_the_day.py:58 `for_the_ask(clean, repaired=None)`
///
/// The pool of lines for a day weighed against an activity, or None. `repaired` None is "" here
/// (Python skips both alike).
pub fn for_the_ask(clean: &str, repaired: &str) -> Option<Vec<String>> {
    for said in [clean, repaired] {
        if said.is_empty() {
            continue;
        }
        if A_DAY_WEIGHED.is_match(said) || A_DAY_ASKED.is_match(text::strip(said)) {
            return Some(if A_COLD_DAY.is_match(said) { TOO_COLD.clone() } else { TOO_HOT.clone() });
        }
    }
    None
}
