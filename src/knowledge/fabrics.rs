//! Port of PyBot's `bot/knowledge/fabrics.py`: what a fabric is good for.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

/// `USES`: (words, reply) rows, each word compiled as `\bword\b`.
static USES: LazyLock<Vec<(Vec<Re>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.fabrics.USES")
        .items()
        .iter()
        .map(|row| {
            let row = row.items();
            let pats = row[0].strs().iter().map(|w| Re::new(&format!(r"\b{}\b", sh::escape(w)), 0)).collect();
            (pats, row[1].str().to_string())
        })
        .collect()
});

/// The line, lower-cased (clean, else text, else the words joined).
pub fn _the_text(reading: &Understood) -> String {
    let text = sh::clean_or_text(reading);
    if text.is_empty() {
        return reading.words.join(" ").to_lowercase();
    }
    text.to_lowercase()
}

/// The card for the use the line names (bedding, towels, ...), or None.
pub fn _the_use(_memory: Option<&Memory>, reading: &Understood) -> Option<String> {
    let text = _the_text(reading);
    USES.iter().find(|(pats, _)| pats.iter().any(|p| p.is_match(&text))).map(|(_, reply)| reply.clone())
}

/// `lambda reading: bool(_the_use(None, reading))`: the row answers only a line naming a use.
pub fn lambda_178(reading: &Understood) -> bool {
    _the_use(None, reading).is_some_and(|r| !r.is_empty())
}

fn g_lambda_178(_f: &FnRef, u: &Understood) -> bool {
    lambda_178(u)
}
fn r_the_use(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _the_use(Some(m), u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.fabrics:<lambda>:178", g_lambda_178)];
pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.fabrics:_the_use:83", r_the_use)];
