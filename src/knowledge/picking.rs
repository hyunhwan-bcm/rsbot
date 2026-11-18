//! Port of `bot/knowledge/picking.py`: which sort of potato, rice, flour... for a job.
//! `_chooser` and `_pick` are factories: their closures (the guard and the reply) are ported, and
//! read the values the factory captured.

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::{globals, re_cached};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self, Re};
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

globals!("bot.knowledge.picking", re { _NOT_A_CHOICE, _A_FLOURY_OR_WAXY }, value { ENTRIES });

static POTATO_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bpotato", re::I));

/// The potato chooser `_potato_kind_is_named` builds.
static POTATO_CHOOSER: LazyLock<Chooser> =
    LazyLock::new(|| _chooser(&["potato", "potatoes", "spud", "spuds", "new potatoes"]));

/// The repaired line, lower case, with a space at each end, for a whole-word search.
pub fn _line(understood: &Understood) -> String {
    let text = if understood.plain.is_empty() { &understood.clean } else { &understood.plain };
    format!(" {} ", text.to_lowercase())
}

/// The ingredient words as one alternation, longest first so "plum tomatoes" wins.
pub fn _names_pattern(names: &[&str]) -> String {
    let mut sorted: Vec<&str> = names.to_vec();
    sorted.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
    sorted.iter().map(|n| re::escape(n)).collect::<Vec<_>>().join("|")
}

/// The regexes a `_chooser(names)` guard reads: the choosing word has to be about this ingredient.
pub struct Chooser {
    pub lead: Re,
    pub cut: Re,
    pub bare: Re,
    pub trail: Re,
    pub kinds: Re,
}

impl Chooser {
    /// The guard `_chooser` returns.
    pub fn guard(&self, understood: &Understood) -> bool {
        chooses(&_line(understood), [&self.lead, &self.cut, &self.bare, &self.trail, &self.kinds])
    }
}

/// `_chooser(names)`: a guard that wants the choosing word attached to one of these ingredients.
pub fn _chooser(names: &[&str]) -> Chooser {
    let ingredient = _names_pattern(names);
    Chooser {
        lead: Re::new(
            &format!(
                r"\b(which|what (sort|kind|type|variety)s? of|(sort|kind|type|variety)s? of|best|better|right)\s+(the\s+)?({ingredient})\b"
            ),
            0,
        ),
        cut: Re::new(&format!(r"\b(which|what)\s+((cut|piece|joint|part|side)s?\s+of\s+)(the\s+)?({ingredient})\b"), 0),
        bare: Re::new(&format!(r"\bwhat\s+(the\s+)?({ingredient})\b[^.!?]{{0,15}}?\bfor\b"), 0),
        trail: Re::new(
            &format!(
                r"\b({ingredient})\b[^.!?]{{0,30}}?\b((should|do|shall|would) i (get|buy|use|pick)|is best|are best|best for|better for|good for)\b"
            ),
            0,
        ),
        kinds: Re::new(
            &format!(
                r"\b(?:what|which)\s+(?:are\s+the\s+|are\s+)?(?:different\s+|main\s+|other\s+|the\s+)?(?:kinds|types|sorts|varieties)\s+of\s+(?:the\s+)?({ingredient})\b|\b(?:what|which)\s+({ingredient})s?\b[^.!?]{{0,24}}?\b(?:are there|do you get|do you sell|come in)\b"
            ),
            0,
        ),
    }
}

/// The body of a chooser guard: not a method, portion or health line, and one of the shapes.
fn chooses(text: &str, shapes: [&Re; 5]) -> bool {
    if _NOT_A_CHOICE.is_match(text) {
        return false;
    }
    shapes.iter().any(|r| r.is_match(text))
}

/// `_chooser.<locals>.guard` for an entry: the regexes it captured.
fn chooser_guard(f: &FnRef, understood: &Understood) -> bool {
    let cell = |name: &str| f.cell(name).and_then(|v| v.as_re()).unwrap_or_else(|| panic!("chooser cell {name}"));
    chooses(&_line(understood), [cell("lead"), cell("cut"), cell("bare"), cell("trail"), cell("kinds")])
}

/// True when the line asks which sort of potato, by the chooser or by the sort's own word.
pub fn _potato_kind_is_named(understood: &Understood) -> bool {
    if POTATO_CHOOSER.guard(understood) {
        return true;
    }
    let mut said = if understood.text.is_empty() { understood.clean.clone() } else { understood.text.clone() };
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    _A_FLOURY_OR_WAXY.is_match(&said) && POTATO_WORD.is_match(&said)
}

/// True when the line is choosing a variety of one of these ingredients (any entry's guard).
pub fn choosing(understood: &Understood) -> bool {
    ENTRIES.items().iter().any(|e| {
        let guard = e.attr("guard").and_then(|g| g.as_fn()).expect("'NoneType' object is not callable");
        match guard.name() {
            "bot.knowledge.picking:_potato_kind_is_named" => _potato_kind_is_named(understood),
            "bot.knowledge.picking:_chooser.<locals>.guard" => chooser_guard(guard, understood),
            _ => crate::knowledge::registry::guard(guard, understood),
        }
    })
}

/// `_pick.<locals>.reply`: the sentence for the first use the line names, or the default.
fn pick_reply(f: &FnRef, _memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = _line(understood);
    for row in f.cell("rows").map(|v| v.items()).unwrap_or(&[]) {
        let (pattern, said) = (row.items()[0].str(), row.items()[1].str());
        if re_cached(pattern, 0).is_match(&text) {
            return Some(said.to_string());
        }
    }
    f.cell("default").and_then(|v| v.as_str()).map(str::to_string)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.picking:_chooser.<locals>.guard:110", chooser_guard),
    ("bot.knowledge.picking:_potato_kind_is_named:63", |_: &FnRef, u: &Understood| _potato_kind_is_named(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.picking:_pick.<locals>.reply:131", pick_reply)];


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/knowledge/picking.py:129 `_pick(rows, default)` (never called on the trace corpus)
    pub fn _pick(rows: &Value, default: &Value) -> Value {
        crate::unported!("bot.knowledge.picking:_pick")
    }

}

