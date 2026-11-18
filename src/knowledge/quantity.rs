//! Port of `bot/knowledge/quantity.py`: a figure with a unit on it is an amount, not a code.

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

static UNITS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.quantity.UNITS"));
static AFTER_A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.quantity.AFTER_A_FIGURE"));

/// True when the line counts something in a unit, so a figure is an amount, not a code.
/// `understood` None and `text` "" are Python's defaults.
pub fn a_figure_with_a_unit(understood: Option<&Understood>, text_: &str) -> bool {
    if let Some(u) = understood {
        for number in &u.numbers {
            let unit = text::lower(text::strip(number.unit.as_deref().unwrap_or("")));
            if UNITS.contains(&unit) {
                return true;
            }
        }
    }
    let body = if !text_.is_empty() {
        text_.to_string()
    } else {
        understood.map(|u| if u.clean.is_empty() { u.text.clone() } else { u.clean.clone() }).unwrap_or_default()
    };
    AFTER_A_FIGURE.is_match(&body)
}
