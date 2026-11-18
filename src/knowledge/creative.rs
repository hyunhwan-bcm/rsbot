//! Port of PyBot's `bot/knowledge/creative.py`: plots, prose, names and costumes.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

/// How to make a plot believable.
pub fn _story(_memory: &Memory, _understood: &Understood) -> String {
    "Build it from a want, a wall and a turn: what the character wants in the first scene, what stops them, and the one thing that changes the plan. A plot that sounds far-fetched is usually missing the practical steps - the key, the alibi, the person who noticed - so make the method vague and the aftermath exact: the cup half drunk, the dog still alive, the wrong person blamed. If you want a villain's plan to be believable, give them one small mistake a careful reader could spot on a second read, and let a minor character be the one who spots it.".to_string()
}

/// How to fix flat prose.
pub fn _prose(_memory: &Memory, _understood: &Understood) -> String {
    "Cut the adverbs and put one thing a reader can see in every paragraph: not 'the kitchen was messy' but 'the bin was full of cereal boxes'. Then vary the sentence length - short where it is tense, longer where it is calm - and read it aloud, because the sentence you stumble on is the one to fix. That is three passes, and they fix most flat writing.".to_string()
}

/// How to name a business or a band.
pub fn _naming(_memory: &Memory, _understood: &Understood) -> String {
    "Name the thing, not the idea: a bakery called Crust, a cafe called The Second Cup, a band called The Rosy Hours. Two short words joined - place plus trade, your own name plus what you sell - beats anything clever, because people have to say it out loud and spell it down a phone. Say it with 'I'm just going to' in front of it, check that the domain and the social handle are free before you fall in love with it, and check Companies House for the limited company: a name can be yours to trade under and still be taken as a company.".to_string()
}

static _THE_METHOD: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.creative._THE_METHOD"));
/// `_COSTUME_TIPS`: (words compiled as `\bword\b`, tip).
static _COSTUME_TIPS: LazyLock<Vec<(Vec<Re>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.creative._COSTUME_TIPS")
        .items()
        .iter()
        .map(|row| {
            let r = row.items();
            (r[0].strs().iter().map(|w| Re::new(&format!(r"\b{}\b", sh::escape(w)), 0)).collect(), r[1].str().to_string())
        })
        .collect()
});
static _COSTUME_MAKING: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.creative._COSTUME_MAKING"));

/// The tip for the first costume occasion the line names ("" for none).
pub fn _costume_word(line: &str) -> String {
    for (words, tip) in _COSTUME_TIPS.iter() {
        if words.iter().any(|w| w.is_match(line)) {
            return tip.clone();
        }
    }
    String::new()
}

/// Whether the line is about making a costume.
pub fn _making_a_costume(understood: &Understood) -> bool {
    _COSTUME_MAKING.is_match(sh::clean_or_text(understood))
}

/// The costume method, with the tip for the occasion the line names.
pub fn _a_costume(_memory: &Memory, understood: &Understood) -> String {
    let line = sh::clean_or_text(understood).to_lowercase();
    let tip = _costume_word(&line);
    if !tip.is_empty() {
        return format!("{} {}", *_THE_METHOD, tip);
    }
    _THE_METHOD.clone()
}

fn g_making(_f: &FnRef, u: &Understood) -> bool {
    _making_a_costume(u)
}
fn r_story(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_story(m, u))
}
fn r_prose(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_prose(m, u))
}
fn r_naming(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_naming(m, u))
}
fn r_costume(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_a_costume(m, u))
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.creative:_making_a_costume:121", g_making)];
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.creative:_story:19", r_story),
    ("bot.knowledge.creative:_prose:30", r_prose),
    ("bot.knowledge.creative:_naming:39", r_naming),
    ("bot.knowledge.creative:_a_costume:127", r_costume),
];
