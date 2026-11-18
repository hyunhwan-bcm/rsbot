//! Port of `bot/knowledge/picks.py`: a second recommendation after a pick, from the same table
//! of films or books the pick came from. The tables are entertainment's, read from the bundle.

use crate::data::{self, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use indexmap::IndexMap;
use std::collections::HashSet;
use std::sync::LazyLock;

const P: &str = "bot.knowledge.picks.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}

/// `PICKS`: {medium: {mood: the sentence}}, in the dicts' order.
static PICKS: LazyLock<IndexMap<String, IndexMap<String, String>>> = LazyLock::new(|| {
    g("PICKS")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.pairs().iter().map(|(m, s)| (m.str().to_string(), s.str().to_string())).collect()))
        .collect()
});
static MEDIUM_WORDS: LazyLock<Vec<(String, Vec<String>)>> =
    LazyLock::new(|| g("MEDIUM_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect());
static MOOD_NEAR: LazyLock<IndexMap<String, Vec<String>>> =
    LazyLock::new(|| g("MOOD_NEAR").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect());
static PICK_ORDER: LazyLock<Vec<String>> = LazyLock::new(|| g("PICK_ORDER").strs());
static SERIES_SIDE: LazyLock<HashSet<String>> = LazyLock::new(|| g("SERIES_SIDE").str_set());
static _TITLED: LazyLock<&'static Re> = LazyLock::new(|| g("_TITLED").as_re().unwrap());
static _MOOD_LEAD: LazyLock<&'static Re> = LazyLock::new(|| g("_MOOD_LEAD").as_re().unwrap());
static WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

fn words_of(said: &str) -> HashSet<String> {
    WORD.findall_str(&text::lower(said)).into_iter().collect()
}

/// The works a reply names, which it writes between two asterisks.
pub fn _titles_in(text_: &str) -> Vec<String> {
    _TITLED.findall_str(text_).iter().map(|one| text::strip(one).to_string()).collect()
}

/// Which table a line is asking from: "film" or "book", or None.
pub fn _medium_of(text_: &str) -> Option<String> {
    let found = words_of(text_);
    MEDIUM_WORDS.iter().find(|(_, cues)| cues.iter().any(|c| found.contains(c))).map(|(m, _)| m.clone())
}

/// The line being answered: the reading carries it, since memory does not hold it yet.
/// `understood` None is Python's default (no reading).
pub fn _the_line(understood: Option<&Understood>, memory: &Memory) -> String {
    if let Some(u) = understood {
        for said in [Some(u.text.as_str()), Some(u.clean.as_str()), u.extra.get("line").and_then(|v| v.as_str())].into_iter().flatten() {
            if !text::strip(said).is_empty() {
                return said.to_string();
            }
        }
    }
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

/// The medium of the pick the last reply gave and the works it named: (medium, named). A reply
/// that names fewer than two works is not a pick.
pub fn _pick_in_play(memory: &Memory, understood: Option<&Understood>) -> (Option<String>, HashSet<String>) {
    let replies: Vec<&str> = memory.turns.iter().map(|t| t.bot.as_str()).filter(|b| !b.is_empty()).collect();
    let Some(last) = replies.last() else { return (None, HashSet::new()) };
    if _titles_in(last).len() < 2 {
        return (None, HashSet::new());
    }
    let named: HashSet<String> = replies.iter().flat_map(|r| _titles_in(r)).map(|t| text::lower(&t)).collect();
    let here = _the_line(understood, memory);
    let medium = _medium_of(&here).or_else(|| _medium_of(last));
    (medium, named)
}

/// The moods of the table a line names, in table order.
pub fn _moods_named(text_: &str, table: &IndexMap<String, String>) -> Vec<String> {
    let words = words_of(text_);
    table.keys().filter(|mood| words.contains(*mood)).cloned().collect()
}

/// A second recommendation, from the same table as the pick just given; None when the last
/// reply was not a list of picks.
pub fn _other_pick(memory: &Memory, understood: &Understood) -> Option<String> {
    let (medium, named) = _pick_in_play(memory, Some(understood));
    let medium = medium.filter(|m| !m.is_empty())?;
    let asked = _the_line(Some(understood), memory);
    if words_of(&asked).iter().any(|w| SERIES_SIDE.contains(w)) {
        return None;
    }
    let table = PICKS.get(&medium).unwrap_or_else(|| panic!("KeyError: {medium:?}"));
    let turns = &memory.turns;
    let before: Vec<&str> = turns[..turns.len().saturating_sub(1)].iter().map(|t| t.user.as_str()).collect();
    let mut order: Vec<String> = Vec::new();
    for part in [asked.clone(), before.last().map(|s| s.to_string()).unwrap_or_default(), before.join(" ")] {
        for mood in _moods_named(&part, table) {
            if order.contains(&mood) {
                continue;
            }
            order.push(mood.clone());
            order.extend(MOOD_NEAR.get(&mood).into_iter().flatten().filter(|near| table.contains_key(*near)).cloned());
        }
    }
    order.extend(PICK_ORDER.iter().filter(|m| table.contains_key(*m)).cloned());
    order.extend(table.keys().cloned());
    let mut seen_moods: HashSet<String> = HashSet::new();
    for mood in order {
        if !seen_moods.insert(mood.clone()) {
            continue;
        }
        let sentence = table.get(&mood).unwrap_or_else(|| panic!("KeyError: {mood:?}"));
        let titles = _titles_in(sentence);
        if titles.is_empty() {
            continue;
        }
        if titles.iter().any(|t| named.contains(&text::lower(t))) {
            continue;
        }
        return Some(format!("Something different then: {}", _MOOD_LEAD.subn("", sentence, 0)));
    }
    None
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.picks:_other_pick:107", |_, m, u| _other_pick(m, u))];
