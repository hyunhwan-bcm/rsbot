//! DUP ports (group 3 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.

use crate::data::{self, Value};
use crate::knowledge::d1_shared as sh;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup

/// DUP(bot.knowledge.lookup::tokens): the loose words of a line, lower case, for matching
/// against keys (`WORD.findall(text.translate(_CURLY).lower())`, WORD = `[a-z0-9]+(?:'[a-z]+)?`).
pub fn tokens(text: &str) -> Vec<String> {
    let low: String = text
        .chars()
        .map(|c| match c {
            '\u{2019}' | '\u{2018}' | '\u{2bc}' => '\'',
            c => c,
        })
        .collect::<String>()
        .to_lowercase();
    let b = low.as_bytes();
    let is_w = |c: u8| c.is_ascii_lowercase() || c.is_ascii_digit();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if is_w(b[i]) {
            let s = i;
            while i < b.len() && is_w(b[i]) {
                i += 1;
            }
            if i + 1 < b.len() && b[i] == b'\'' && b[i + 1].is_ascii_lowercase() {
                i += 1;
                while i < b.len() && b[i].is_ascii_lowercase() {
                    i += 1;
                }
            }
            out.push(low[s..i].to_string());
        } else {
            i += 1;
        }
    }
    out
}

/// DUP(bot.knowledge.lookup::MIN_TYPO_LENGTH).
pub static MIN_TYPO_LENGTH: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.knowledge.lookup.MIN_TYPO_LENGTH").as_int().unwrap_or(4) as usize);

/// DUP(bot.knowledge.lookup::_one_edit): true when the two words are one slip apart (a letter
/// wrong, missing, extra, or two neighbours swapped).
pub fn _one_edit(shorter: &str, longer: &str) -> bool {
    if shorter == longer {
        return true;
    }
    let mut s: Vec<char> = shorter.chars().collect();
    let mut l: Vec<char> = longer.chars().collect();
    if s.len().abs_diff(l.len()) > 1 {
        return false;
    }
    if s.len() > l.len() {
        std::mem::swap(&mut s, &mut l);
    }
    let mut i = 0;
    while i < s.len() && s[i] == l[i] {
        i += 1;
    }
    if s.len() < l.len() {
        return s[i..] == l[i + 1..];
    }
    let (rest, tail) = (&s[i..], &l[i..]);
    if rest.get(1..) == tail.get(1..) {
        return true;
    }
    rest.len() > 1 && rest[0] == tail[1] && rest[1] == tail[0] && rest[2..] == tail[2..]
}

static ENDINGS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.ENDINGS"));

/// DUP(bot.knowledge.lookup::_stem): the word a form belongs to (radiator/radiators are one).
pub fn _stem(word: &str) -> &str {
    let n = word.chars().count();
    for ending in ENDINGS.iter() {
        if word.ends_with(ending.as_str()) && n >= ending.chars().count() + 3 {
            return &word[..word.len() - ending.len()];
        }
    }
    word
}

static COMMON: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup._COMMON"));
static COMMON_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.COMMON_WORDS"));
static STEMS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup._STEMS"));
static COLOUR_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.COLOUR_WORDS"));
static LEX_CHAT: LazyLock<HashSet<String>> = LazyLock::new(|| dict_keys("bot.understanding.lexicon.CHAT"));
static LEX_EXPAND: LazyLock<HashSet<String>> = LazyLock::new(|| dict_keys("bot.understanding.lexicon.EXPAND"));

fn dict_keys(name: &str) -> HashSet<String> {
    data::get(name).pairs().iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect()
}

/// DUP(bot.knowledge.lookup::_engine_reads): true when the understanding engine already reads
/// this word, as English or chat-speak, so it is never bent into a key word as a slip.
pub fn _engine_reads(word: &str) -> bool {
    COMMON.contains(word)
        || COMMON_WORDS.contains(word)
        || LEX_CHAT.contains(word)
        || LEX_EXPAND.contains(word)
        || STEMS.contains(_stem(word))
        || every_word().contains(word)
        || COLOUR_WORDS.contains(word)
}

static GONE_OFF_SIGNS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.GONE_OFF_SIGNS"));

/// DUP(bot.knowledge.lookup::says_it_has_gone_off): true when the line says the food has gone
/// off, or asks whether it has.
pub fn says_it_has_gone_off(clean: &str) -> bool {
    let low = format!(" {} ", clean.to_lowercase());
    GONE_OFF_SIGNS
        .iter()
        .any(|sign| low.contains(&format!(" {sign} ")) || low.contains(&format!(" {sign}")))
}

// ---------------------------------------------------------------------------------------------
// bot.understanding

/// DUP(bot.understanding.lexicon::every_word): every word the reading's own tables (lexicon,
/// topics, kinds, body) have a meaning for, as one set: the closing fence on a typo.
pub fn every_word() -> &'static HashSet<String> {
    static WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
        let mut words = HashSet::new();
        for prefix in [
            "bot.understanding.lexicon.",
            "bot.understanding.topics.",
            "bot.understanding.kinds.",
            "bot.understanding.body.",
        ] {
            for full in data::names_with_prefix(prefix) {
                let name = &full[prefix.len()..];
                if name.contains('.') || name.starts_with('_') || name.contains("MISSPELLING") {
                    continue;
                }
                let value = data::get(full);
                let items: Vec<&Value> = match value {
                    Value::Set(v) | Value::Tuple(v) | Value::List(v) => v.iter().collect(),
                    Value::Dict(d) => d.iter().map(|(k, _)| k).collect(),
                    _ => continue,
                };
                for item in items {
                    let Some(s) = item.as_str() else { continue };
                    for word in s.to_lowercase().replace(',', " ").split_whitespace() {
                        if word.chars().all(char::is_alphabetic) {
                            words.insert(word.to_string());
                        }
                    }
                }
            }
        }
        words
    });
    &WORDS
}

/// DUP(bot.understanding::line_as_read): the line with the engine's repairs (`plain`), or the
/// line as typed when there is no plain line.
pub fn line_as_read(u: &Understood) -> String {
    if !u.plain.trim().is_empty() {
        return u.plain.clone();
    }
    sh::clean_or_text(u).to_string()
}

static SENSE_ELSEWHERE_FORMS: LazyLock<HashMap<String, Vec<(i64, HashSet<String>)>>> = LazyLock::new(|| {
    data::get("bot.understanding.topics.SENSE_ELSEWHERE_FORMS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| {
            let tiers = v
                .items()
                .iter()
                .map(|t| {
                    let parts = t.items();
                    (parts[0].as_int().unwrap_or(0), parts[1].str_set())
                })
                .collect();
            Some((k.as_str()?.to_string(), tiers))
        })
        .collect()
});
static BODY_TWO_LIFE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.topics.BODY_TWO_LIFE"));
static OWNING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.topics.OWNING_WORDS"));
static TEETHING_IS_A_FAULT: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::set("bot.understanding.topics.TEETHING_IS_A_FAULT"));
static TEETHING_AND_A_CHILD: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::set("bot.understanding.topics.TEETHING_AND_A_CHILD"));

/// DUP(bot.understanding.topics::_teething_is_a_fault): "teething" is a new thing finding its
/// feet, not a child cutting teeth.
pub fn _teething_is_a_fault(norm: &[String]) -> bool {
    norm.iter().any(|w| TEETHING_IS_A_FAULT.contains(w)) && !norm.iter().any(|w| TEETHING_AND_A_CHILD.contains(w))
}

/// DUP(bot.understanding.topics::_sense_cue): a cue word sits near one of the two-life words.
pub fn _sense_cue(norm: &[String], positions: &[usize], tiers: &[(i64, HashSet<String>)]) -> bool {
    for (cue_at, word) in norm.iter().enumerate() {
        for (reach, cues) in tiers {
            if !cues.contains(word) {
                continue;
            }
            for &spot in positions {
                let (spot, cue) = (spot as i64, cue_at as i64);
                if spot - reach <= cue && cue <= spot + reach {
                    return true;
                }
            }
        }
    }
    false
}

/// DUP(bot.understanding.topics::_means_something_else): the word at `position` names another
/// thing in this line.
pub fn _means_something_else(norm: &[String], position: usize) -> bool {
    if norm[position] == "teething" && _teething_is_a_fault(norm) {
        return true;
    }
    let Some(tiers) = SENSE_ELSEWHERE_FORMS.get(&norm[position]) else { return false };
    if tiers.is_empty() {
        return false;
    }
    if BODY_TWO_LIFE.contains(&norm[position]) && position > 0 && OWNING_WORDS.contains(&norm[position - 1]) {
        return true;
    }
    _sense_cue(norm, &[position], tiers)
}

/// DUP(bot.understanding.topics::names_another_thing): positions of two-life words a cue has
/// taken to their other life.
pub fn names_another_thing(norm: &[String]) -> Vec<usize> {
    (0..norm.len())
        .filter(|&at| SENSE_ELSEWHERE_FORMS.contains_key(&norm[at]) && _means_something_else(norm, at))
        .collect()
}

/// DUP(bot.understanding.topics::says_another_thing): the two-life words in a line that a cue
/// has taken to their other life.
pub fn says_another_thing(text: &str) -> HashSet<String> {
    static SPLIT: LazyLock<crate::py::re::Re> = LazyLock::new(|| crate::py::re::Re::new(r"[^a-z0-9]+", 0));
    let norm: Vec<String> = SPLIT.split(&text.to_lowercase()).into_iter().filter(|w| !w.is_empty()).collect();
    names_another_thing(&norm).into_iter().map(|at| norm[at].clone()).collect()
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.cleaning and bot.compose.fit

static LANDED_ON: LazyLock<&'static crate::py::re::Re> = LazyLock::new(|| data::re("bot.knowledge.cleaning.LANDED_ON"));
static LANDED_ON_THINGS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cleaning.LANDED_ON_THINGS"));

/// DUP(bot.knowledge.cleaning::landed_on_a_thing): the line says what is on a garment or a
/// surface ("i got egg on my shirt"), not what it is made of.
pub fn landed_on_a_thing(u: &Understood) -> bool {
    let text = format!(" {} ", sh::text_or_clean(u).to_lowercase());
    for m in LANDED_ON.finditer(&text) {
        if LANDED_ON_THINGS.iter().any(|t| t == m.str(1)) {
            return true;
        }
    }
    if [" on it ", " on them ", " on those ", " on that ", " on this "].iter().any(|p| text.contains(p)) {
        return LANDED_ON_THINGS.iter().any(|w| text.contains(w.as_str()));
    }
    false
}

/// DUP(bot.knowledge.cleaning::not_a_mark_on_a_thing): the stand-down for a food row.
pub fn not_a_mark_on_a_thing(u: &Understood) -> bool {
    !landed_on_a_thing(u)
}

static FIT_WANTS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fit.WANTS"));

/// DUP(bot.compose.fit::wanted): the shape of answer this turn asked for, from the store
/// (`memory["understood"]["wants"]`, else `memory["intent_wants"]`).
pub fn fit_wanted(memory: &crate::state::Memory) -> Option<String> {
    if let Some(u) = &memory.understood {
        if let Some(w) = &u.wants {
            if FIT_WANTS.contains(w) {
                return Some(w.clone());
            }
        }
    }
    memory.intent_wants.clone().filter(|w| FIT_WANTS.contains(w))
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup: the kinds ask, and the engine's `find`

static KINDS_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KINDS_WORDS"));
static KINDS_SHAPE: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.lookup.KINDS_SHAPE"));
static KINDS_ASKS_ME: LazyLock<&'static crate::py::re::Re> = LazyLock::new(|| data::re("bot.knowledge.lookup.KINDS_ASKS_ME"));
static KINDS_ASKS_WITH_A_VERB: LazyLock<&'static crate::py::re::Re> =
    LazyLock::new(|| data::re("bot.knowledge.lookup.KINDS_ASKS_WITH_A_VERB"));
static KINDS_SAYS_HOW_IT_IS: LazyLock<&'static crate::py::re::Re> =
    LazyLock::new(|| data::re("bot.knowledge.lookup.KINDS_SAYS_HOW_IT_IS"));
static KINDS_NAMED_A_USE: LazyLock<&'static crate::py::re::Re> =
    LazyLock::new(|| data::re("bot.knowledge.lookup.KINDS_NAMED_A_USE"));

/// DUP(bot.knowledge.lookup::asks_the_kinds): the line asks which families a thing comes in
/// ("what kinds of tea are there"), in its own words or leaning on the kinds ask before it.
pub fn asks_the_kinds(u: &Understood) -> bool {
    let said = sh::clean_or_text(u);
    if !said.is_empty() && KINDS_ASKS_ME.is_match(said) {
        return false;
    }
    if !said.is_empty() && !KINDS_NAMED_A_USE.is_match(said) {
        let cleaned = said.to_lowercase().replace(['[', ']', ',', '\''], " ");
        let words = format!(" {} ", cleaned.split_whitespace().collect::<Vec<_>>().join(" "));
        let lowered = said.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
        let says_it = u.intent == "news"
            || (u.kind != "question" && !KINDS_ASKS_WITH_A_VERB.is_match(&lowered) && KINDS_SAYS_HOW_IT_IS.is_match(&lowered));
        if !says_it && KINDS_WORDS.iter().any(|k| words.contains(k.as_str())) {
            return true;
        }
    }
    u.wants.as_deref() == Some(KINDS_SHAPE.as_str())
        && !KINDS_NAMED_A_USE.is_match(said)
        && u.asks.iter().all(|a| a == "what" || a == "yesno")
}

/// DUP(bot.knowledge.lookup::find) as `find(understood) is not None`. UNRESOLVED: the matching
/// engine (lookup.find, score, _the_best_entry ...) belongs to the engine workstream and is not
/// ported; None stands for "the engine could not answer" (the callers treat it as Python treats
/// an exception there).
pub fn lookup_find_is_some(_u: &Understood) -> Option<bool> {
    None
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.cooking (the functions the dish, meal and cooking_more tables call, with their
// closure). Every `pub fn` here is DUP(bot.knowledge.cooking::<same name>).

pub mod cooking {
    #![allow(non_snake_case, unused_variables, dead_code)]
    //! VERIFY ONLY: delegates to D2's port of bot.knowledge.cooking.
    use crate::knowledge::cooking as c;
    use crate::state::Memory;
    use crate::understood::Understood;
    use std::collections::HashSet;

    #[derive(Default, Clone, Copy)]
    pub struct MealArgs<'a> {
        pub dishes_only: bool,
        pub text: Option<&'a str>,
        pub banned: Option<&'a HashSet<String>>,
        pub diet: Option<&'a str>,
        pub memory: Option<&'a Memory>,
        pub avoid: &'a [String],
        pub pointed_back: bool,
    }
    pub fn _a_brew_not_a_dinner(line: &str) -> bool { c::_a_brew_not_a_dinner(line) }
    pub fn _bans_for(understood: &Understood, memory: &Memory) -> HashSet<String> { c::_bans_for(understood, &mut memory.clone()) }
    pub fn _bans_it(food: &str, banned: &HashSet<String>) -> bool { c::_bans_it(food, banned) }
    pub fn _bans_said(banned: &HashSet<String>) -> Vec<String> { c::_bans_said(banned) }
    pub fn _crowd(text: &str) -> i64 { c::_crowd(text) }
    pub fn _crowd_or_any(text: &str, banned: &HashSet<String>) -> Option<String> { Some(c::_crowd_or_any(text, banned)) }
    pub fn _excluded(text: &str) -> HashSet<String> { c::_excluded(text) }
    pub fn _fold(word: &str) -> String { c::_fold(word) }
    pub fn _foods_named(text: &str) -> Vec<String> { c::_foods_named(text) }
    pub fn _mentions(text: &str, names: &[String]) -> bool { c::_mentions(text, names) }
    pub fn _occasion(text: &str) -> bool { c::_occasion(text) }
    pub fn _refused_foods(text: &str, families: bool) -> HashSet<String> { c::_refused_foods(text, families) }
    pub fn _sides(memory: &Memory, understood: &Understood) -> Option<String> { c::_sides(&mut memory.clone(), understood) }
    pub fn _table_text(text: &str) -> bool { c::_table_text(text) }
    pub fn dish(understood: &Understood, avoid: &[String], memory: Option<&Memory>) -> Option<String> { c::dish(understood, avoid, memory) }
    pub fn ingredients(understood: &Understood) -> Vec<String> { c::ingredients(understood) }
    pub fn meal(understood: &Understood, a: MealArgs) -> Option<String> {
        c::meal(understood, c::MealArgs { dishes_only: a.dishes_only, text: a.text, banned: a.banned.cloned(), diet: a.diet.map(String::from), memory: a.memory, avoid: a.avoid.to_vec(), pointed_back: a.pointed_back })
    }
    pub fn negated_ingredients(understood: &Understood) -> Vec<String> { c::negated_ingredients(understood) }
    pub fn pantry_idea(name: &str) -> Option<String> { c::pantry_idea(name) }
    pub fn pointed(memory: &Memory, understood: &Understood) -> Option<(Understood, bool)> { c::pointed(memory, understood) }
    pub fn pointed_at_a_food(understood: &Understood, memory: &Memory) -> Option<String> { c::pointed_at_a_food(understood, &mut memory.clone()) }
    pub fn side_for(memory: &Memory, text: &str) -> Option<String> { c::side_for(memory, text) }
    pub fn without(understood: &Understood) -> Option<String> { c::without(understood) }
    pub fn missing(understood: &Understood) -> HashSet<String> { c::missing(understood) }
}
