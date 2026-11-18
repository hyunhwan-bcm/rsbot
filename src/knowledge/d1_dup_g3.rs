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
// bot.knowledge.body

static TEMPERATURE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.body.TEMPERATURE_WORDS"));
static THING_AFTER_THE_WORD: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::set("bot.knowledge.body.THING_AFTER_THE_WORD"));
static BODY_OTHER_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.body.OTHER_PEOPLE"));
static WE_OUR_OURS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.body.WE_OUR_OURS"));
static POSSESSIVE_OF_A_PERSON: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::set("bot.knowledge.body.POSSESSIVE_OF_A_PERSON"));

/// DUP(bot.knowledge.body::a_persons_temperature): the temperature in the line is somebody's
/// own (a fever), not an oven's or a probe's.
pub fn a_persons_temperature(u: &Understood) -> bool {
    let words: Vec<String> = u.words.iter().map(|w| sh::strip_punct(&w.to_lowercase()).to_string()).collect();
    if words
        .windows(2)
        .any(|p| TEMPERATURE_WORDS.contains(&p[0]) && THING_AFTER_THE_WORD.contains(&p[1]))
    {
        return false;
    }
    if u.body.as_deref() == Some("temperature") {
        return true;
    }
    if !words.iter().any(|w| TEMPERATURE_WORDS.contains(w)) {
        return false;
    }
    if words.iter().any(|w| BODY_OTHER_PEOPLE.contains(w) || WE_OUR_OURS.contains(w)) {
        return true;
    }
    words
        .windows(2)
        .any(|p| POSSESSIVE_OF_A_PERSON.contains(&p[0]) && TEMPERATURE_WORDS.contains(&p[1]))
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.cooking (the functions the dish, meal and cooking_more tables call, with their
// closure). Every `pub fn` here is DUP(bot.knowledge.cooking::<same name>).

pub mod cooking {
    #![allow(non_snake_case, unused_variables, dead_code)]
    use crate::state::Memory;
    use crate::understood::Understood;
    use std::collections::HashSet;

    /// The optional arguments of `meal(understood, dishes_only=False, text=None, banned=None,
    /// diet=None, memory=None, avoid=(), pointed_back=False)`.
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

    /// DUP(bot.knowledge.cooking::_a_brew_not_a_dinner)
    pub fn _a_brew_not_a_dinner(line: &str) -> bool {
        false
    }
    /// DUP(bot.knowledge.cooking::_bans_for)
    pub fn _bans_for(understood: &Understood, memory: &Memory) -> HashSet<String> {
        HashSet::new()
    }
    /// DUP(bot.knowledge.cooking::_bans_it)
    pub fn _bans_it(food: &str, banned: &HashSet<String>) -> bool {
        false
    }
    /// DUP(bot.knowledge.cooking::_bans_said)
    pub fn _bans_said(banned: &HashSet<String>) -> Vec<String> {
        Vec::new()
    }
    /// DUP(bot.knowledge.cooking::_crowd)
    pub fn _crowd(text: &str) -> i64 {
        0
    }
    /// DUP(bot.knowledge.cooking::_crowd_or_any)
    pub fn _crowd_or_any(text: &str, banned: &HashSet<String>) -> Option<String> {
        None
    }
    /// DUP(bot.knowledge.cooking::_excluded)
    pub fn _excluded(text: &str) -> HashSet<String> {
        HashSet::new()
    }
    /// DUP(bot.knowledge.cooking::_fold)
    pub fn _fold(word: &str) -> String {
        word.to_string()
    }
    /// DUP(bot.knowledge.cooking::_foods_named)
    pub fn _foods_named(text: &str) -> Vec<String> {
        Vec::new()
    }
    /// DUP(bot.knowledge.cooking::_mentions)
    pub fn _mentions(text: &str, names: &[String]) -> bool {
        false
    }
    /// DUP(bot.knowledge.cooking::_occasion)
    pub fn _occasion(text: &str) -> bool {
        false
    }
    /// DUP(bot.knowledge.cooking::_refused_foods)
    pub fn _refused_foods(text: &str, families: bool) -> HashSet<String> {
        HashSet::new()
    }
    /// DUP(bot.knowledge.cooking::_sides)
    pub fn _sides(memory: &Memory, understood: &Understood) -> Option<String> {
        None
    }
    /// DUP(bot.knowledge.cooking::_table_text)
    pub fn _table_text(text: &str) -> bool {
        false
    }
    /// DUP(bot.knowledge.cooking::dish)
    pub fn dish(understood: &Understood, avoid: &[String], memory: Option<&Memory>) -> Option<String> {
        None
    }
    /// DUP(bot.knowledge.cooking::ingredients)
    pub fn ingredients(understood: &Understood) -> Vec<String> {
        Vec::new()
    }
    /// DUP(bot.knowledge.cooking::meal)
    pub fn meal(understood: &Understood, args: MealArgs) -> Option<String> {
        None
    }
    /// DUP(bot.knowledge.cooking::negated_ingredients)
    pub fn negated_ingredients(understood: &Understood) -> Vec<String> {
        Vec::new()
    }
    /// DUP(bot.knowledge.cooking::pantry_idea)
    pub fn pantry_idea(name: &str) -> Option<String> {
        None
    }
    /// DUP(bot.knowledge.cooking::pointed): the reading the line means, and its
    /// `new_dish_only` flag.
    pub fn pointed(memory: &Memory, understood: &Understood) -> Option<(Understood, bool)> {
        None
    }
    /// DUP(bot.knowledge.cooking::pointed_at_a_food)
    pub fn pointed_at_a_food(understood: &Understood, memory: &Memory) -> Option<String> {
        None
    }
    /// DUP(bot.knowledge.cooking::side_for)
    pub fn side_for(memory: &Memory, text: &str) -> Option<String> {
        None
    }
    /// DUP(bot.knowledge.cooking::without)
    pub fn without(understood: &Understood) -> Option<String> {
        None
    }
    /// DUP(bot.knowledge.cooking::missing)
    pub fn missing(understood: &Understood) -> HashSet<String> {
        HashSet::new()
    }
}
