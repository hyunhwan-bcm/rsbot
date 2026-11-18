//! Port of `bot/modules/safety.py`: an emergency reply, a first-aid reply for a small injury, or
//! None to pass. PRIORITY 9000, above the answer modules: an emergency is never outvoted.
//!
//! The module is a long chain of readings, each a set of patterns from the exported bundle
//! (`safety_tables`, `safety_cards` and `safety_phrases` are re-exported under `safety`'s own
//! names, as Python imports them). Every function of the Python file is here, in the file's order;
//! `respond` is ported rule for rule, comments shortened.
//!
//! How the Python is carried over:
//!
//! * A pattern tuple is a [`Strs`] static named as in Python; `_matches(PATTERNS, line)` is
//!   [`_matches`], which compiles each pattern once (Python's `_COMPILED`). A concatenation
//!   `_matches(A + B, line)` is `_matches(&A, line) || _matches(&B, line)`, the same search in the
//!   same order; an inline tuple is an array of `&str`.
//! * A compiled regex is an [`Rx`], a string constant a [`Str`], a frozenset a [`StrSet`].
//! * `pick(pool, memory)` is the composer's `pick`, handed the memory as a Python-like value.
//! * Offsets Python counts in characters (windows before a match, distances) are counted in
//!   characters here too.

#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::runtime::guarded;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::{LazyLock, Mutex, OnceLock};

/// A global of `bot.modules.safety` from the bundle.
fn b(name: &str) -> &'static Value {
    data::get(&format!("bot.modules.safety.{name}"))
}

// ---------------------------------------------------------------------------------------------
// Patterns, compiled once (Python's `_COMPILED`, and `re`'s own cache for `re.search(str, ...)`).

fn compiled_with(pattern: &str, flags: u32) -> &'static Re {
    static PLAIN: LazyLock<Mutex<HashMap<String, &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    static FLAGGED: LazyLock<Mutex<HashMap<(String, u32), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    if flags == pyre::U {
        let mut cache = PLAIN.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(rx) = cache.get(pattern) {
            return rx;
        }
        let rx: &'static Re = Box::leak(Box::new(Re::new(pattern, flags)));
        cache.insert(pattern.to_string(), rx);
        return rx;
    }
    let mut cache = FLAGGED.lock().unwrap_or_else(|e| e.into_inner());
    let key = (pattern.to_string(), flags);
    if let Some(rx) = cache.get(&key) {
        return rx;
    }
    let rx: &'static Re = Box::leak(Box::new(Re::new(pattern, flags)));
    cache.insert(key, rx);
    rx
}

/// `re.compile(pattern)`, cached.
fn compiled(pattern: &str) -> &'static Re {
    compiled_with(pattern, pyre::U)
}

/// `re.search(pattern, line)` for a pattern given as a string.
fn search<'t>(pattern: &str, line: &'t str) -> Option<pyre::Match<'t>> {
    compiled(pattern).search(line)
}

/// `bool(re.search(pattern, line))`.
fn found(pattern: &str, line: &str) -> bool {
    compiled(pattern).is_match(line)
}

/// `re.sub(pattern, repl, line)` (with `flags`).
fn sub_with(pattern: &str, repl: &str, line: &str, flags: u32) -> String {
    compiled_with(pattern, flags).sub(repl, line)
}

fn pats_of(value: &'static Value) -> Vec<&'static Re> {
    match value {
        Value::Str(s) => vec![compiled(s)],
        Value::Re(rx) => vec![&**rx],
        _ => value
            .items()
            .iter()
            .map(|item| match item {
                Value::Str(s) => compiled(s),
                Value::Re(rx) => &**rx,
                _ => panic!("re.compile of a value that is not a pattern"),
            })
            .collect(),
    }
}

/// A tuple of strings from the bundle: a pattern list for `_matches`, or a pool for `pick`.
pub struct Strs {
    name: &'static str,
    strs: OnceLock<Vec<String>>,
    pats: OnceLock<Vec<&'static Re>>,
}

impl Strs {
    pub const fn new(name: &'static str) -> Strs {
        Strs { name, strs: OnceLock::new(), pats: OnceLock::new() }
    }
    /// The patterns, compiled.
    pub fn pats(&self) -> &[&'static Re] {
        self.pats.get_or_init(|| pats_of(b(self.name)))
    }
}

impl Deref for Strs {
    type Target = [String];
    fn deref(&self) -> &[String] {
        self.strs.get_or_init(|| b(self.name).strs())
    }
}

/// A compiled regex from the bundle.
pub struct Rx {
    name: &'static str,
    cell: OnceLock<&'static Re>,
}

impl Rx {
    pub const fn new(name: &'static str) -> Rx {
        Rx { name, cell: OnceLock::new() }
    }
}

impl Deref for Rx {
    type Target = Re;
    fn deref(&self) -> &Re {
        self.cell.get_or_init(|| b(self.name).as_re().unwrap_or_else(|| panic!("{} is not a regex", self.name)))
    }
}

/// A string constant from the bundle (also a lone pattern, when `_matches` is handed one).
pub struct Str {
    name: &'static str,
    cell: OnceLock<&'static str>,
}

impl Str {
    pub const fn new(name: &'static str) -> Str {
        Str { name, cell: OnceLock::new() }
    }
}

impl Deref for Str {
    type Target = str;
    fn deref(&self) -> &str {
        self.cell.get_or_init(|| b(self.name).as_str().unwrap_or_else(|| panic!("{} is not a string", self.name)))
    }
}

/// A frozenset of strings from the bundle (bundle order, which is sorted).
pub struct StrSet {
    name: &'static str,
    cell: OnceLock<(HashSet<String>, Vec<String>)>,
}

impl StrSet {
    pub const fn new(name: &'static str) -> StrSet {
        StrSet { name, cell: OnceLock::new() }
    }
    fn get(&self) -> &(HashSet<String>, Vec<String>) {
        self.cell.get_or_init(|| {
            let items = b(self.name).strs();
            (items.iter().cloned().collect(), items)
        })
    }
    pub fn contains(&self, item: &str) -> bool {
        self.get().0.contains(item)
    }
    /// The items, in the bundle's order.
    pub fn items(&self) -> &[String] {
        &self.get().1
    }
}

/// What `_matches` accepts: a pattern tuple, a lone pattern, or an inline tuple of strings.
pub trait Patterns {
    fn found_in(&self, line: &str) -> bool;
}

impl Patterns for Strs {
    fn found_in(&self, line: &str) -> bool {
        self.pats().iter().any(|rx| rx.is_match(line))
    }
}

impl Patterns for Str {
    fn found_in(&self, line: &str) -> bool {
        compiled(self).is_match(line)
    }
}

impl Patterns for str {
    fn found_in(&self, line: &str) -> bool {
        compiled(self).is_match(line)
    }
}

impl<const N: usize> Patterns for [&str; N] {
    fn found_in(&self, line: &str) -> bool {
        self.iter().any(|pattern| compiled(pattern).is_match(line))
    }
}

impl Patterns for [&str] {
    fn found_in(&self, line: &str) -> bool {
        self.iter().any(|pattern| compiled(pattern).is_match(line))
    }
}

impl Patterns for [String] {
    fn found_in(&self, line: &str) -> bool {
        self.iter().any(|pattern| compiled(pattern).is_match(line))
    }
}

impl Patterns for Vec<String> {
    fn found_in(&self, line: &str) -> bool {
        self.as_slice().found_in(line)
    }
}

impl Patterns for [&'static Re] {
    fn found_in(&self, line: &str) -> bool {
        self.iter().any(|rx| rx.is_match(line))
    }
}

/// Python: bot/modules/safety.py:1933 `_matches(patterns, line)`: the line matches any of the
/// patterns, each compiled once (a lone pattern is a tuple of one).
pub fn _matches<P: Patterns + ?Sized>(patterns: &P, line: &str) -> bool {
    patterns.found_in(line)
}

// ---------------------------------------------------------------------------------------------
// Tables that are not a plain tuple, a regex, a string or a set.

/// A `{str: tuple of str}` dict (`REPLIES`, `_MINOR_REPLIES`, `HEAL_TIME`, `HEAD_REPLIES`), in
/// the dict's order.
pub struct PoolMap {
    name: &'static str,
    cell: OnceLock<Vec<(String, Vec<String>)>>,
}

impl PoolMap {
    pub const fn new(name: &'static str) -> PoolMap {
        PoolMap { name, cell: OnceLock::new() }
    }
    pub fn items(&self) -> &[(String, Vec<String>)] {
        self.cell.get_or_init(|| b(self.name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect())
    }
    pub fn get(&self, key: &str) -> Option<&[String]> {
        self.items().iter().find(|(k, _)| k == key).map(|(_, v)| v.as_slice())
    }
    /// `TABLE[key]`: a missing key raises.
    pub fn at(&self, key: &str) -> &[String] {
        self.get(key).unwrap_or_else(|| panic!("KeyError: {key} in {}", self.name))
    }
}

/// A `{str: str}` or `{str: int}` dict, kept as values.
pub struct ValMap {
    name: &'static str,
    cell: OnceLock<HashMap<String, Value>>,
}

impl ValMap {
    pub const fn new(name: &'static str) -> ValMap {
        ValMap { name, cell: OnceLock::new() }
    }
    fn map(&self) -> &HashMap<String, Value> {
        self.cell.get_or_init(|| b(self.name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.clone())).collect())
    }
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.map().get(key).and_then(Value::as_str)
    }
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.map().get(key).and_then(Value::as_int)
    }
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map().get(key)
    }
    pub fn contains(&self, key: &str) -> bool {
        self.map().contains_key(key)
    }
}

/// A list of (pattern, replacement) pairs (`SHE`, `HE`, `ME`), or of (name, pattern) pairs.
pub struct Pairs {
    name: &'static str,
    cell: OnceLock<Vec<(String, String)>>,
}

impl Pairs {
    pub const fn new(name: &'static str) -> Pairs {
        Pairs { name, cell: OnceLock::new() }
    }
}

impl Deref for Pairs {
    type Target = [(String, String)];
    fn deref(&self) -> &[(String, String)] {
        self.cell.get_or_init(|| {
            b(self.name)
                .items()
                .iter()
                .map(|pair| (pair.items()[0].str().to_string(), pair.items()[1].str().to_string()))
                .collect()
        })
    }
}

static REPLIES: PoolMap = PoolMap::new("REPLIES");
static _MINOR_REPLIES: PoolMap = PoolMap::new("_MINOR_REPLIES");
static HEAL_TIME: PoolMap = PoolMap::new("HEAL_TIME");
static CASE_ALIAS: ValMap = ValMap::new("CASE_ALIAS");
static CASE_WORDS: ValMap = ValMap::new("CASE_WORDS");
static GENERAL: ValMap = ValMap::new("GENERAL");
static MISSPELLINGS: ValMap = ValMap::new("MISSPELLINGS");
static TYPED_TYPOS: ValMap = ValMap::new("TYPED_TYPOS");
static _THIRD: ValMap = ValMap::new("_THIRD");
static AFTER: ValMap = ValMap::new("AFTER");
static WAITING: ValMap = ValMap::new("WAITING");
static DAY_TOTAL_MAX: ValMap = ValMap::new("DAY_TOTAL_MAX");
static DOSE_LIMITS: ValMap = ValMap::new("DOSE_LIMITS");
static DOSE_WORDS: ValMap = ValMap::new("DOSE_WORDS");
static _DAY_TOTAL_WORDS: ValMap = ValMap::new("_DAY_TOTAL_WORDS");
static SHE: Pairs = Pairs::new("SHE");
static HE: Pairs = Pairs::new("HE");
static ME: Pairs = Pairs::new("ME");
static HEAL_WORDS: Pairs = Pairs::new("HEAL_WORDS");
/// `BLACKED_OUT_SIGHT`: (shape, needs a person) rows; only the shape is read here.
static BLACKED_OUT_SIGHT: LazyLock<Vec<String>> =
    LazyLock::new(|| b("BLACKED_OUT_SIGHT").items().iter().map(|row| row.items()[0].str().to_string()).collect());
/// `lexicon.CONTRACTIONS`, in the dict's order.
static CONTRACTIONS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.understanding.lexicon.CONTRACTIONS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});

/// One row of `CASES`: the case, its patterns, and whether it needs a person.
pub struct Case {
    pub name: String,
    pub patterns: Vec<&'static Re>,
    pub needs_person: bool,
}

static CASES: LazyLock<Vec<Case>> = LazyLock::new(|| {
    b("CASES")
        .items()
        .iter()
        .map(|row| {
            let items: &'static [Value] = row.items();
            Case { name: items[0].str().to_string(), patterns: pats_of(&items[1]), needs_person: items[2].truthy() }
        })
        .collect()
});

/// One row of `MINOR`: the injury, its patterns, and its replies.
pub struct MinorRow {
    pub name: String,
    pub patterns: Vec<&'static Re>,
    pub replies: Vec<String>,
}

static MINOR: LazyLock<Vec<MinorRow>> = LazyLock::new(|| {
    b("MINOR")
        .items()
        .iter()
        .map(|row| {
            let items: &'static [Value] = row.items();
            MinorRow { name: items[0].str().to_string(), patterns: pats_of(&items[1]), replies: items[2].strs() }
        })
        .collect()
});

/// `INJURY_WORDS`: (injury, patterns) rows.
static INJURY_WORDS: LazyLock<Vec<(String, Vec<&'static Re>)>> = LazyLock::new(|| {
    b("INJURY_WORDS")
        .items()
        .iter()
        .map(|row| {
            let items: &'static [Value] = row.items();
            (items[0].str().to_string(), pats_of(&items[1]))
        })
        .collect()
});

// ---------------------------------------------------------------------------------------------
// The composer's pick, and the memory notes.

/// The memory as a Python-like value, for the composer's `pick`.
fn memory_value(memory: &Memory) -> Value {
    Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
}

/// `pick(options, memory)`.
fn pick(options: &[String], memory: &Memory) -> String {
    #[cfg(test)]
    if tests::FIRST_PICK.with(|first| first.get()) {
        return options[0].clone();
    }
    crate::compose::pick(options, &memory_value(memory), "")
}

/// `pick(options, memory, seed_text)`.
fn pick_seeded(options: &[String], memory: &Memory, seed: &str) -> String {
    #[cfg(test)]
    if tests::FIRST_PICK.with(|first| first.get()) {
        return options[0].clone();
    }
    crate::compose::pick(options, &memory_value(memory), seed)
}

/// `str(x or "")` of a memory value.
fn value_str(value: Option<&Value>) -> String {
    match value {
        Some(Value::Str(s)) => s.to_string(),
        Some(v) if v.truthy() => match v {
            Value::Int(i) => i.to_string(),
            Value::Bool(true) => "True".to_string(),
            Value::Float(f) => text::float_repr(*f),
            other => format!("{:?}", other.to_plain()),
        },
        _ => String::new(),
    }
}

/// The memory's `safety_case`, when it is a string.
fn safety_case(memory: &Memory) -> Option<String> {
    memory.get("safety_case").and_then(Value::as_str).map(str::to_string)
}

/// `any(isinstance(t, dict) and t.get("module") == "safety" for t in turns[-2:])`.
fn safety_answered_lately(memory: &Memory) -> bool {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(2)..].iter().any(|turn| turn.module.as_deref() == Some("safety"))
}

// ---------------------------------------------------------------------------------------------
// The module-level tables and patterns the functions read, by their Python names.

static ABOUT_THE_SPEAKER: Strs = Strs::new("ABOUT_THE_SPEAKER");
static ABUSE: Strs = Strs::new("ABUSE");
static ABUSE_BENIGN_REASON: Strs = Strs::new("ABUSE_BENIGN_REASON");
static ABUSE_BLOW: Strs = Strs::new("ABUSE_BLOW");
static ABUSE_BUTTONHOLED: Strs = Strs::new("ABUSE_BUTTONHOLED");
static ABUSE_CHILD_REFUSAL: Strs = Strs::new("ABUSE_CHILD_REFUSAL");
static ABUSE_CHILD_TAKES_MONEY: Strs = Strs::new("ABUSE_CHILD_TAKES_MONEY");
static ABUSE_DISCLOSURE: Strs = Strs::new("ABUSE_DISCLOSURE");
static ABUSE_DISCLOSURE_FINE: Strs = Strs::new("ABUSE_DISCLOSURE_FINE");
static ABUSE_EMPLOYER_REFUSAL: Strs = Strs::new("ABUSE_EMPLOYER_REFUSAL");
static ABUSE_FINE: Strs = Strs::new("ABUSE_FINE");
static ABUSE_FINE_ANIMAL: Strs = Strs::new("ABUSE_FINE_ANIMAL");
static ABUSE_FINE_PLACE: Strs = Strs::new("ABUSE_FINE_PLACE");
static ABUSE_HANDS_ON: Strs = Strs::new("ABUSE_HANDS_ON");
static ABUSE_HIT_ME: Strs = Strs::new("ABUSE_HIT_ME");
static ABUSE_KIDS_AT_HOME: Strs = Strs::new("ABUSE_KIDS_AT_HOME");
static ABUSE_KIDS_CARD: Strs = Strs::new("ABUSE_KIDS_CARD");
static ABUSE_MEMORY: Strs = Strs::new("ABUSE_MEMORY");
static ABUSE_NOT_HIT: Strs = Strs::new("ABUSE_NOT_HIT");
static ABUSE_NOW: Strs = Strs::new("ABUSE_NOW");
static ABUSE_OBJECTS: Strs = Strs::new("ABUSE_OBJECTS");
static ABUSE_OBJECTS_OFF: Strs = Strs::new("ABUSE_OBJECTS_OFF");
static ABUSE_OBJECTS_PERSON: Strs = Strs::new("ABUSE_OBJECTS_PERSON");
static ABUSE_PARENT_REFUSAL: Strs = Strs::new("ABUSE_PARENT_REFUSAL");
static ABUSE_PARENT_REFUSAL_HARD: Strs = Strs::new("ABUSE_PARENT_REFUSAL_HARD");
static ABUSE_PARTNER_DID_IT: Strs = Strs::new("ABUSE_PARTNER_DID_IT");
static ABUSE_PLAY: Strs = Strs::new("ABUSE_PLAY");
static ABUSE_PLAY_HARD: Strs = Strs::new("ABUSE_PLAY_HARD");
static ABUSE_PURSUIT_HARD: Strs = Strs::new("ABUSE_PURSUIT_HARD");
static ABUSE_QUARREL: Strs = Strs::new("ABUSE_QUARREL");
static ABUSE_ROOM_CARD: Strs = Strs::new("ABUSE_ROOM_CARD");
static ABUSE_ROOM_FAMILY: Strs = Strs::new("ABUSE_ROOM_FAMILY");
static ABUSE_SCARED: Strs = Strs::new("ABUSE_SCARED");
static ABUSE_SCARED_REPLY: Strs = Strs::new("ABUSE_SCARED_REPLY");
static ABUSE_VERBAL_FINE: Strs = Strs::new("ABUSE_VERBAL_FINE");
static ACCIDENT: Strs = Strs::new("ACCIDENT");
static ADULT_HIT: Strs = Strs::new("ADULT_HIT");
static ADULT_PERSON: Strs = Strs::new("ADULT_PERSON");
static AFTER_EXERCISE: Strs = Strs::new("AFTER_EXERCISE");
static AIRWAY_CHANGED: Strs = Strs::new("AIRWAY_CHANGED");
static ALARM_NUISANCE: Strs = Strs::new("ALARM_NUISANCE");
static ALLERGY_AIRWAY: Strs = Strs::new("ALLERGY_AIRWAY");
static ALLERGY_FACE: Strs = Strs::new("ALLERGY_FACE");
static ALLERGY_OFF: Strs = Strs::new("ALLERGY_OFF");
static ALLERGY_SIGN_NAMED: Strs = Strs::new("ALLERGY_SIGN_NAMED");
static AMBULANCE_CASES: StrSet = StrSet::new("AMBULANCE_CASES");
static ANIMAL: Strs = Strs::new("ANIMAL");
static ANIMAL_KICK: Strs = Strs::new("ANIMAL_KICK");
static ANIMAL_KICK_LOOKALIKE: Strs = Strs::new("ANIMAL_KICK_LOOKALIKE");
static ANY_REFUSAL_TO_GO: Strs = Strs::new("ANY_REFUSAL_TO_GO");
static AN_ACT_BESIDE_THE_ITCH: Strs = Strs::new("AN_ACT_BESIDE_THE_ITCH");
static AN_ITCH_LOOKALIKE: Strs = Strs::new("AN_ITCH_LOOKALIKE");
static APPLIANCE_SMOKE: Strs = Strs::new("APPLIANCE_SMOKE");
static AQUARIUM_WORDS: Strs = Strs::new("AQUARIUM_WORDS");
static ASKING_WHY_IT_HURTS: Strs = Strs::new("ASKING_WHY_IT_HURTS");
static ASKS_ABOUT_THE_INJURY: Strs = Strs::new("ASKS_ABOUT_THE_INJURY");
static ASKS_GO_BARE: Strs = Strs::new("ASKS_GO_BARE");
static ASKS_IF_CALL: Strs = Strs::new("ASKS_IF_CALL");
static ASKS_IF_GOING: Strs = Strs::new("ASKS_IF_GOING");
static ASK_CALL_REPLIES: Strs = Strs::new("ASK_CALL_REPLIES");
static ASK_GO_REPLIES: Strs = Strs::new("ASK_GO_REPLIES");
static ASPIRIN: Strs = Strs::new("ASPIRIN");
static ASPIRIN_REPLY: Strs = Strs::new("ASPIRIN_REPLY");
static AVOID_BEFORE: Rx = Rx::new("AVOID_BEFORE");
static A_BARE_ACTOR_LASH: Rx = Rx::new("A_BARE_ACTOR_LASH");
static A_BODY_PART: Strs = Strs::new("A_BODY_PART");
static A_BREATHER: Strs = Strs::new("A_BREATHER");
static A_FOLLOW_UP_ABOUT_THE_INJURY: StrSet = StrSet::new("A_FOLLOW_UP_ABOUT_THE_INJURY");
static A_GRANDCHILD_OF_AN_AGE: Rx = Rx::new("A_GRANDCHILD_OF_AN_AGE");
static A_GROWN_AGE: Rx = Rx::new("A_GROWN_AGE");
static A_GROWN_BLOW: Rx = Rx::new("A_GROWN_BLOW");
static A_HAND_ON_THE_PERSON: Strs = Strs::new("A_HAND_ON_THE_PERSON");
static A_HARM_VERB_CUT: Str = Str::new("A_HARM_VERB_CUT");
static A_HURT_OF_A_BODY_PART: Str = Str::new("A_HURT_OF_A_BODY_PART");
static A_PERSON: Strs = Strs::new("A_PERSON");
static A_PLAN_ORDINARY: Strs = Strs::new("A_PLAN_ORDINARY");
static A_PLAN_SAID: Strs = Strs::new("A_PLAN_SAID");
static A_PLATE_SHAPE: Strs = Strs::new("A_PLATE_SHAPE");
static A_REAL_DANGER: Strs = Strs::new("A_REAL_DANGER");
static A_REAL_INJURY: Strs = Strs::new("A_REAL_INJURY");
static A_SMALL_AMOUNT: Strs = Strs::new("A_SMALL_AMOUNT");
static A_SMALL_CHILD_LASHES_OUT: Rx = Rx::new("A_SMALL_CHILD_LASHES_OUT");
static A_SMALL_CHILD_SAID: Rx = Rx::new("A_SMALL_CHILD_SAID");
static A_WISH_OR_TALK_OF_IT: Strs = Strs::new("A_WISH_OR_TALK_OF_IT");
static A_WORKOUT_CUE: Str = Str::new("A_WORKOUT_CUE");
static BABY_BURNING: Strs = Strs::new("BABY_BURNING");
static BABY_FELL: Strs = Strs::new("BABY_FELL");
static BABY_FELL_OFF: Strs = Strs::new("BABY_FELL_OFF");
static BABY_FELL_OVER: Strs = Strs::new("BABY_FELL_OVER");
static BABY_FELL_RED: Strs = Strs::new("BABY_FELL_RED");
static BABY_FELL_SURFACE: Strs = Strs::new("BABY_FELL_SURFACE");
static BACK_ON: Strs = Strs::new("BACK_ON");
static BARE_LAUGH: Rx = Rx::new("BARE_LAUGH");
static BATTERY_FIRE: Strs = Strs::new("BATTERY_FIRE");
static BITE_DONE: Strs = Strs::new("BITE_DONE");
static BITE_ON_ANIMAL: Strs = Strs::new("BITE_ON_ANIMAL");
static BLACKOUT_A_THING: Strs = Strs::new("BLACKOUT_A_THING");
static BLACKOUT_HEART: Strs = Strs::new("BLACKOUT_HEART");
static BLACKOUT_OFF: Strs = Strs::new("BLACKOUT_OFF");
static BLACKOUT_REPEAT: Strs = Strs::new("BLACKOUT_REPEAT");
static BLACKOUT_STILL_DOWN: Strs = Strs::new("BLACKOUT_STILL_DOWN");
static BLACKOUT_URGENT: Strs = Strs::new("BLACKOUT_URGENT");
static BLEEDING: Strs = Strs::new("BLEEDING");
static BLEED_VOLUME: Strs = Strs::new("BLEED_VOLUME");
static BLIND_CORD: Strs = Strs::new("BLIND_CORD");
static BLISTER_CAUSE_BURN: Strs = Strs::new("BLISTER_CAUSE_BURN");
static BLISTER_HOWTO: Strs = Strs::new("BLISTER_HOWTO");
static BLISTER_STATE: Strs = Strs::new("BLISTER_STATE");
static BLISTER_STATE_REPLIES: Strs = Strs::new("BLISTER_STATE_REPLIES");
static BLISTER_WORD: Strs = Strs::new("BLISTER_WORD");
static BLOW_ON_ME: Strs = Strs::new("BLOW_ON_ME");
static BLUE: Strs = Strs::new("BLUE");
static BLUE_CHANGE: Strs = Strs::new("BLUE_CHANGE");
static BLUE_CLOTH: Strs = Strs::new("BLUE_CLOTH");
static BLUE_COLD: Strs = Strs::new("BLUE_COLD");
static BLUE_COLD_CAUSE: Strs = Strs::new("BLUE_COLD_CAUSE");
static BLUE_COLD_SELF: Strs = Strs::new("BLUE_COLD_SELF");
static BLUE_COSMETIC: Strs = Strs::new("BLUE_COSMETIC");
static BLUE_FACE_PART: Strs = Strs::new("BLUE_FACE_PART");
static BLUE_FINE: Strs = Strs::new("BLUE_FINE");
static BLUE_FOOD: Strs = Strs::new("BLUE_FOOD");
static BLUE_HUMAN: Strs = Strs::new("BLUE_HUMAN");
static BLUE_LIMB: Strs = Strs::new("BLUE_LIMB");
static BLUE_LIPS_FACE: Strs = Strs::new("BLUE_LIPS_FACE");
static BLUE_MARK: Strs = Strs::new("BLUE_MARK");
static BLUE_OTHER_PERSON: Strs = Strs::new("BLUE_OTHER_PERSON");
static BLUE_PERSON_SUBJECT: Strs = Strs::new("BLUE_PERSON_SUBJECT");
static BLUE_SELF: Strs = Strs::new("BLUE_SELF");
static BLUE_STRONG: Strs = Strs::new("BLUE_STRONG");
static BLUE_TINGE: Strs = Strs::new("BLUE_TINGE");
static BODY_IN_THE_ROOM: Strs = Strs::new("BODY_IN_THE_ROOM");
static BODY_LIMB: Str = Str::new("BODY_LIMB");
static BONE_FINE: Strs = Strs::new("BONE_FINE");
static BONE_ORDINARY: Strs = Strs::new("BONE_ORDINARY");
static BONE_OUT_LOOSE: Strs = Strs::new("BONE_OUT_LOOSE");
static BONE_PAST: Strs = Strs::new("BONE_PAST");
static BONE_RED: Strs = Strs::new("BONE_RED");
static BONE_STILL: Strs = Strs::new("BONE_STILL");
static BONE_THROAT: Strs = Strs::new("BONE_THROAT");
static BREATH_AT_REST: Strs = Strs::new("BREATH_AT_REST");
static BREATH_BARE: Strs = Strs::new("BREATH_BARE");
static BREATH_CAUSE: Strs = Strs::new("BREATH_CAUSE");
static BREATH_CAUSE_RED: Strs = Strs::new("BREATH_CAUSE_RED");
static BREATH_EFFORT: Strs = Strs::new("BREATH_EFFORT");
static BREATH_EFFORT_RED: Strs = Strs::new("BREATH_EFFORT_RED");
static BREATH_LAUGH: Strs = Strs::new("BREATH_LAUGH");
static BREATH_LAUGH_REPLY: Strs = Strs::new("BREATH_LAUGH_REPLY");
static BREATH_OVER: Strs = Strs::new("BREATH_OVER");
static BREATH_PAST_CUE: Strs = Strs::new("BREATH_PAST_CUE");
static BREATH_PAST_OVER: Strs = Strs::new("BREATH_PAST_OVER");
static BREATH_RED: Strs = Strs::new("BREATH_RED");
static BREATH_ROOM_RED: Rx = Rx::new("BREATH_ROOM_RED");
static BREATH_SETTLED: Rx = Rx::new("BREATH_SETTLED");
static BREATH_SHORT: Strs = Strs::new("BREATH_SHORT");
static BREATH_SPORT: Strs = Strs::new("BREATH_SPORT");
static BREATH_STILL_RED: Strs = Strs::new("BREATH_STILL_RED");
static BURNER_THING: Strs = Strs::new("BURNER_THING");
static BURNT_BODY_VERB: Strs = Strs::new("BURNT_BODY_VERB");
static BURNT_FOOD: Strs = Strs::new("BURNT_FOOD");
static BURNT_GRIME: Strs = Strs::new("BURNT_GRIME");
static BURNT_MARKS: Strs = Strs::new("BURNT_MARKS");
static BURNT_SELF: Strs = Strs::new("BURNT_SELF");
static BURN_MARK_ON_A_BODY: Strs = Strs::new("BURN_MARK_ON_A_BODY");
static BURN_OR_SCALD_WORD: Strs = Strs::new("BURN_OR_SCALD_WORD");
static BURN_WORD: Strs = Strs::new("BURN_WORD");
static CALL_HAS_A_SIGN: Strs = Strs::new("CALL_HAS_A_SIGN");
static CALL_HELP: Strs = Strs::new("CALL_HELP");
static CALL_HELP_REPLY: Strs = Strs::new("CALL_HELP_REPLY");
static CHEST: Strs = Strs::new("CHEST");
static CHEST_ADJ: Strs = Strs::new("CHEST_ADJ");
static CHEST_ANSWERED_ELSEWHERE: Str = Str::new("CHEST_ANSWERED_ELSEWHERE");
static CHEST_ASKED: Strs = Strs::new("CHEST_ASKED");
static CHEST_ASKED_OWN: Strs = Strs::new("CHEST_ASKED_OWN");
static CHEST_COUGH_STRAIN_REPLY: Strs = Strs::new("CHEST_COUGH_STRAIN_REPLY");
static CHEST_EFFORT: Strs = Strs::new("CHEST_EFFORT");
static CHEST_FEELING: Strs = Strs::new("CHEST_FEELING");
static CHEST_FEELING_RED: Strs = Strs::new("CHEST_FEELING_RED");
static CHEST_KEPT_BY_ANOTHER_CARD: Strs = Strs::new("CHEST_KEPT_BY_ANOTHER_CARD");
static CHEST_MENTIONED: Strs = Strs::new("CHEST_MENTIONED");
static CHEST_RED_FLAGS: Strs = Strs::new("CHEST_RED_FLAGS");
static CHEST_SLOW: Strs = Strs::new("CHEST_SLOW");
static CHEST_SORE_MOVE: Strs = Strs::new("CHEST_SORE_MOVE");
static CHEST_SPREAD: Strs = Strs::new("CHEST_SPREAD");
static CHEST_SPREADING: Strs = Strs::new("CHEST_SPREADING");
static CHEST_STRAIN: Strs = Strs::new("CHEST_STRAIN");
static CHEST_STRAIN_MOVE: Strs = Strs::new("CHEST_STRAIN_MOVE");
static CHEST_STRAIN_REPLY: Strs = Strs::new("CHEST_STRAIN_REPLY");
static CHEST_STRAIN_STOP: Strs = Strs::new("CHEST_STRAIN_STOP");
static CHEST_TRAINING: Strs = Strs::new("CHEST_TRAINING");
static CHEST_URGENT: Strs = Strs::new("CHEST_URGENT");
static CHEST_WORK: Strs = Strs::new("CHEST_WORK");
static CHILDHOOD_FIT: Strs = Strs::new("CHILDHOOD_FIT");
static CHILD_AGGRESSOR: Strs = Strs::new("CHILD_AGGRESSOR");
static CHILD_ANON: Strs = Strs::new("CHILD_ANON");
static CHILD_DRANK_WEAK: Strs = Strs::new("CHILD_DRANK_WEAK");
static CHILD_FEVER_NOT_EATING: Strs = Strs::new("CHILD_FEVER_NOT_EATING");
static CHILD_FEVER_RED: Strs = Strs::new("CHILD_FEVER_RED");
static CHILD_GROWN: Strs = Strs::new("CHILD_GROWN");
static CHILD_HIT_NOW: Strs = Strs::new("CHILD_HIT_NOW");
static CHILD_SCHOOL_AGE: Strs = Strs::new("CHILD_SCHOOL_AGE");
static CHILD_TAKER: Str = Str::new("CHILD_TAKER");
static CHILD_WORD: Str = Str::new("CHILD_WORD");
static CHILD_YOUNG: Strs = Strs::new("CHILD_YOUNG");
static CHOKE_AIR_OK: Strs = Strs::new("CHOKE_AIR_OK");
static CHOKE_BABY: Strs = Strs::new("CHOKE_BABY");
static CHOKE_HABIT: Strs = Strs::new("CHOKE_HABIT");
static CHOKE_HELP: Strs = Strs::new("CHOKE_HELP");
static CHOKE_NOT_PERSON: Strs = Strs::new("CHOKE_NOT_PERSON");
static CHOKE_NOW: Strs = Strs::new("CHOKE_NOW");
static CHOKE_OVER: Strs = Strs::new("CHOKE_OVER");
static CHOKE_PERSON: Strs = Strs::new("CHOKE_PERSON");
static CHOKE_PREVENT: Strs = Strs::new("CHOKE_PREVENT");
static CHOKE_RED: Strs = Strs::new("CHOKE_RED");
static CHOKE_SETTLED: Strs = Strs::new("CHOKE_SETTLED");
static CHOKE_STILL_RED: Strs = Strs::new("CHOKE_STILL_RED");
static CHOKE_WORDS: Strs = Strs::new("CHOKE_WORDS");
static CHORE_CAUSE: Strs = Strs::new("CHORE_CAUSE");
static CHRONIC_BREATH: Strs = Strs::new("CHRONIC_BREATH");
static CLEANABLE_SURFACE: Strs = Strs::new("CLEANABLE_SURFACE");
static COLD_ILLNESS: Strs = Strs::new("COLD_ILLNESS");
static COLLAPSE_HINT: Strs = Strs::new("COLLAPSE_HINT");
static CONFUSED_THING: Strs = Strs::new("CONFUSED_THING");
static CONFUSION_NAMED_CAUSE: Strs = Strs::new("CONFUSION_NAMED_CAUSE");
static CORRECTION_OF_PERSON: Strs = Strs::new("CORRECTION_OF_PERSON");
static COUGH_FINE: Strs = Strs::new("COUGH_FINE");
static COUNTERFACTUAL_IDIOM: Strs = Strs::new("COUNTERFACTUAL_IDIOM");
static COUNTERFACTUAL_WISH: Strs = Strs::new("COUNTERFACTUAL_WISH");
static CPR_ASK: Strs = Strs::new("CPR_ASK");
static CPR_BABY: Strs = Strs::new("CPR_BABY");
static CPR_BABY_SCENE: Strs = Strs::new("CPR_BABY_SCENE");
static CPR_CHILD: Strs = Strs::new("CPR_CHILD");
static CPR_CHILD_SCENE: Strs = Strs::new("CPR_CHILD_SCENE");
static CPR_HOW: Strs = Strs::new("CPR_HOW");
static CPR_HOWTO: Strs = Strs::new("CPR_HOWTO");
static CPR_HOW_BABY: Strs = Strs::new("CPR_HOW_BABY");
static CPR_HOW_CHILD: Strs = Strs::new("CPR_HOW_CHILD");
static CPR_NOT_ALIVE: Strs = Strs::new("CPR_NOT_ALIVE");
static CPR_NOT_A_BODY: Strs = Strs::new("CPR_NOT_A_BODY");
static CPR_NOT_EFFORT: Strs = Strs::new("CPR_NOT_EFFORT");
static CPR_NO_PULSE: Strs = Strs::new("CPR_NO_PULSE");
static CPR_NO_PULSE_NOW: Strs = Strs::new("CPR_NO_PULSE_NOW");
static CPR_ROOM_EMPTY: Strs = Strs::new("CPR_ROOM_EMPTY");
static CPR_ROUGH: Strs = Strs::new("CPR_ROUGH");
static CPR_SCENE: Strs = Strs::new("CPR_SCENE");
static CPR_SCENE_CASES: Strs = Strs::new("CPR_SCENE_CASES");
static CPR_START: Strs = Strs::new("CPR_START");
static CPR_STILL_ALIVE: Strs = Strs::new("CPR_STILL_ALIVE");
static CPR_STOP: Strs = Strs::new("CPR_STOP");
static CPR_STOP_REPLY: Strs = Strs::new("CPR_STOP_REPLY");
static CPR_WHEN_ALIVE: Strs = Strs::new("CPR_WHEN_ALIVE");
static CRISIS_PLAN_CARRY: Strs = Strs::new("CRISIS_PLAN_CARRY");
static CRISIS_PLAN_ORDINARY: Strs = Strs::new("CRISIS_PLAN_ORDINARY");
static CRISIS_PLAN_REPLY: Strs = Strs::new("CRISIS_PLAN_REPLY");
static CRISIS_PLAN_SAID: Strs = Strs::new("CRISIS_PLAN_SAID");
static CRISIS_WORD: Rx = Rx::new("CRISIS_WORD");
static CRITICAL: Strs = Strs::new("CRITICAL");
static CRITICAL_ONE: Strs = Strs::new("CRITICAL_ONE");
static CRUSH_HARD: Strs = Strs::new("CRUSH_HARD");
static CRUSH_OFF: Strs = Strs::new("CRUSH_OFF");
static CUT_MYSELF_OFF: Strs = Strs::new("CUT_MYSELF_OFF");
static CUT_NOT_STOPPING: Strs = Strs::new("CUT_NOT_STOPPING");
static CUT_SELF: Str = Str::new("CUT_SELF");
static DAY_TOTAL_SIGN: Strs = Strs::new("DAY_TOTAL_SIGN");
static DAY_TOTAL_WORD: Strs = Strs::new("DAY_TOTAL_WORD");
static DEATH_IN_THE_LINE: Strs = Strs::new("DEATH_IN_THE_LINE");
static DEATH_REPORT: Strs = Strs::new("DEATH_REPORT");
static DEATH_SAID: Strs = Strs::new("DEATH_SAID");
static DEATH_WORDS: Strs = Strs::new("DEATH_WORDS");
static DENTAL_NUMB_ASK: Strs = Strs::new("DENTAL_NUMB_ASK");
static DENTAL_NUMB_FOLLOW: Strs = Strs::new("DENTAL_NUMB_FOLLOW");
static DENTAL_NUMB_RED: Strs = Strs::new("DENTAL_NUMB_RED");
static DESERVE_DENIED: Rx = Rx::new("DESERVE_DENIED");
static DESPAIR: Strs = Strs::new("DESPAIR");
static DESPAIR_WITHOUT_NOTHING_LEFT: Strs = Strs::new("DESPAIR_WITHOUT_NOTHING_LEFT");
static DETAIL_AFTER_ESCALATION: Strs = Strs::new("DETAIL_AFTER_ESCALATION");
static DETAIL_REPLY: Strs = Strs::new("DETAIL_REPLY");
static DEVICE_BURNING: Strs = Strs::new("DEVICE_BURNING");
static DEVICE_OLD: Strs = Strs::new("DEVICE_OLD");
static DEVICE_TEMP: Strs = Strs::new("DEVICE_TEMP");
static DIE_OF_THE_FEELING: Strs = Strs::new("DIE_OF_THE_FEELING");
static DISTRESS_BESIDE_IT: Strs = Strs::new("DISTRESS_BESIDE_IT");
static DOG_ATTACK_NOW: Strs = Strs::new("DOG_ATTACK_NOW");
static DOG_BIT_A_PERSON: Strs = Strs::new("DOG_BIT_A_PERSON");
static DOG_BLEED: Strs = Strs::new("DOG_BLEED");
static DOG_HABIT: Strs = Strs::new("DOG_HABIT");
static DOG_ITCH: Strs = Strs::new("DOG_ITCH");
static DOG_MOUTHING: Strs = Strs::new("DOG_MOUTHING");
static DOG_SELF_BITE: Strs = Strs::new("DOG_SELF_BITE");
static DOSE_ALREADY_GIVEN: Strs = Strs::new("DOSE_ALREADY_GIVEN");
static DOSE_ASKED: Strs = Strs::new("DOSE_ASKED");
static DOSE_COURSE: Strs = Strs::new("DOSE_COURSE");
static DOSE_COURSE_KEEP: Strs = Strs::new("DOSE_COURSE_KEEP");
static DOSE_DOUBLE: Strs = Strs::new("DOSE_DOUBLE");
static DOSE_RECENT: Strs = Strs::new("DOSE_RECENT");
static DOSE_REGULAR: Strs = Strs::new("DOSE_REGULAR");
static DOSE_STATED: Strs = Strs::new("DOSE_STATED");
static DROWNED_THING: Strs = Strs::new("DROWNED_THING");
static DROWNING_HYPERBOLE: Strs = Strs::new("DROWNING_HYPERBOLE");
static DYING_OTHER: Strs = Strs::new("DYING_OTHER");
static EFFORT_AFTER: Rx = Rx::new("EFFORT_AFTER");
static EFFORT_ANYWHERE: Strs = Strs::new("EFFORT_ANYWHERE");
static EFFORT_BENIGN: Strs = Strs::new("EFFORT_BENIGN");
static EFFORT_HEART_CLUSTER: Strs = Strs::new("EFFORT_HEART_CLUSTER");
static EFFORT_MEANS: Strs = Strs::new("EFFORT_MEANS");
static EFFORT_MORE_RED: Strs = Strs::new("EFFORT_MORE_RED");
static EFFORT_NOT_AN_ACCIDENT: Strs = Strs::new("EFFORT_NOT_AN_ACCIDENT");
static EFFORT_PATTERN: Strs = Strs::new("EFFORT_PATTERN");
static EFFORT_POSTURAL: Strs = Strs::new("EFFORT_POSTURAL");
static EFFORT_RECOVERY: Strs = Strs::new("EFFORT_RECOVERY");
static EFFORT_RED_HEART: Strs = Strs::new("EFFORT_RED_HEART");
static EFFORT_SICK: Strs = Strs::new("EFFORT_SICK");
static EFFORT_SICK_PAIR: Strs = Strs::new("EFFORT_SICK_PAIR");
static EFFORT_SOMEONE_ELSE: Strs = Strs::new("EFFORT_SOMEONE_ELSE");
static EFFORT_TOO_LONG: Strs = Strs::new("EFFORT_TOO_LONG");
static EFFORT_TRAVEL_PAIN: Strs = Strs::new("EFFORT_TRAVEL_PAIN");
static EFFORT_WEAK: Strs = Strs::new("EFFORT_WEAK");
static ELDER_KIN: Strs = Strs::new("ELDER_KIN");
static ELECTRIC_COOKER: Strs = Strs::new("ELECTRIC_COOKER");
static ELECTRIC_FINE: Strs = Strs::new("ELECTRIC_FINE");
static ELECTRIC_SHOCK_CPR: Strs = Strs::new("ELECTRIC_SHOCK_CPR");
static ELECTRIC_SHOCK_DEATH: Strs = Strs::new("ELECTRIC_SHOCK_DEATH");
static ELECTRIC_SHOCK_DEFINITION: Strs = Strs::new("ELECTRIC_SHOCK_DEFINITION");
static ELECTRIC_SHOCK_DOWN: Strs = Strs::new("ELECTRIC_SHOCK_DOWN");
static ELECTRIC_SHOCK_NEARLY: Strs = Strs::new("ELECTRIC_SHOCK_NEARLY");
static ELECTRIC_SHOCK_NOT_ELECTRIC: Strs = Strs::new("ELECTRIC_SHOCK_NOT_ELECTRIC");
static ELECTRIC_SHOCK_STATIC: Strs = Strs::new("ELECTRIC_SHOCK_STATIC");
static ELECTRIC_SHOCK_STORY: Strs = Strs::new("ELECTRIC_SHOCK_STORY");
static ELECTRIC_SHOCK_SUPPLY: Strs = Strs::new("ELECTRIC_SHOCK_SUPPLY");
static ELECTRIC_SHOCK_WIRED: Strs = Strs::new("ELECTRIC_SHOCK_WIRED");
static END_ALL: Strs = Strs::new("END_ALL");
static END_HELPLESS: Strs = Strs::new("END_HELPLESS");
static END_IT_ALL_JOKE: Strs = Strs::new("END_IT_ALL_JOKE");
static END_IT_MEANS: Strs = Strs::new("END_IT_MEANS");
static END_IT_WITH: Str = Str::new("END_IT_WITH");
static END_IT_WITH_A_THING: Strs = Strs::new("END_IT_WITH_A_THING");
static END_IT_WITH_CONNECTOR: Str = Str::new("END_IT_WITH_CONNECTOR");
static END_IT_WITH_NOUN: Str = Str::new("END_IT_WITH_NOUN");
static END_IT_WITH_PERSON: Str = Str::new("END_IT_WITH_PERSON");
static END_MEANS: Strs = Strs::new("END_MEANS");
static END_MEANS_OTHER: Strs = Strs::new("END_MEANS_OTHER");
static END_THING: Strs = Strs::new("END_THING");
static ESCALATED_ALREADY: Strs = Strs::new("ESCALATED_ALREADY");
static EXERTION: Strs = Strs::new("EXERTION");
static EXERTION_CHEST: Strs = Strs::new("EXERTION_CHEST");
static EXERTION_CLASS: Strs = Strs::new("EXERTION_CLASS");
static EXERTION_SMALL: Strs = Strs::new("EXERTION_SMALL");
static EXPECTING_OTHER: Strs = Strs::new("EXPECTING_OTHER");
static EYE_FINE: Strs = Strs::new("EYE_FINE");
static FADE_EVERYDAY: Strs = Strs::new("FADE_EVERYDAY");
static FAINT_STORY: Strs = Strs::new("FAINT_STORY");
static FAINT_WITH_A_BANG: Strs = Strs::new("FAINT_WITH_A_BANG");
static FALL_ANSWERED: Strs = Strs::new("FALL_ANSWERED");
static FALL_ASLEEP: Strs = Strs::new("FALL_ASLEEP");
static FALL_CHECK_OFF: Strs = Strs::new("FALL_CHECK_OFF");
static FALL_CORE: Strs = Strs::new("FALL_CORE");
static FALL_FINE: Strs = Strs::new("FALL_FINE");
static FALL_NOT_BREATHING: Strs = Strs::new("FALL_NOT_BREATHING");
static FALL_NOW: Strs = Strs::new("FALL_NOW");
static FALL_PAST: Strs = Strs::new("FALL_PAST");
static FALL_PATTERN_HURT: Strs = Strs::new("FALL_PATTERN_HURT");
static FALL_RED: Strs = Strs::new("FALL_RED");
static FALL_REPEAT: Strs = Strs::new("FALL_REPEAT");
static FALL_STROKE_SIGN: Strs = Strs::new("FALL_STROKE_SIGN");
static FALL_THING: Strs = Strs::new("FALL_THING");
static FALL_WORD_IDIOM: Strs = Strs::new("FALL_WORD_IDIOM");
static FEMALE_WORDS: Strs = Strs::new("FEMALE_WORDS");
static FEVER_ASK: Strs = Strs::new("FEVER_ASK");
static FEVER_CASES: StrSet = StrSet::new("FEVER_CASES");
static FEVER_HOME_CARE: StrSet = StrSet::new("FEVER_HOME_CARE");
static FEVER_NOT_THE_WEATHER: Strs = Strs::new("FEVER_NOT_THE_WEATHER");
static FEVER_RASH_FINE: Strs = Strs::new("FEVER_RASH_FINE");
static FEVER_REPORT: Strs = Strs::new("FEVER_REPORT");
static FEVER_SIGN: Strs = Strs::new("FEVER_SIGN");
static FEVER_STILL_NOW: Strs = Strs::new("FEVER_STILL_NOW");
static FICTION_NOW: Strs = Strs::new("FICTION_NOW");
static FIGURATIVE: Strs = Strs::new("FIGURATIVE");
static FIGURATIVE_BREATH: Rx = Rx::new("FIGURATIVE_BREATH");
static FINGER_SIDE: Strs = Strs::new("FINGER_SIDE");
static FIRE: Strs = Strs::new("FIRE");
static FIRE_DRILL: Strs = Strs::new("FIRE_DRILL");
static FIRE_ELSEWHERE: Strs = Strs::new("FIRE_ELSEWHERE");
static FIRE_NOW: Strs = Strs::new("FIRE_NOW");
static FIRE_OUT: Strs = Strs::new("FIRE_OUT");
static FIRE_SCALE: Strs = Strs::new("FIRE_SCALE");
static FIRST_FIT: Strs = Strs::new("FIRST_FIT");
static FIRST_PERSON: Strs = Strs::new("FIRST_PERSON");
static FIRST_PERSON_WISH: Strs = Strs::new("FIRST_PERSON_WISH");
static FITTING_IN: Strs = Strs::new("FITTING_IN");
static FITTING_MANNER: Rx = Rx::new("FITTING_MANNER");
static FITTING_MANNER_TAIL: Rx = Rx::new("FITTING_MANNER_TAIL");
static FITTING_THING: Strs = Strs::new("FITTING_THING");
static FIT_ALL_BETTER: Strs = Strs::new("FIT_ALL_BETTER");
static FIT_CASES: Strs = Strs::new("FIT_CASES");
static FIT_ROUTINE: Strs = Strs::new("FIT_ROUTINE");
static FIT_STILL_RINGS: Strs = Strs::new("FIT_STILL_RINGS");
static FIT_THIS_WEEK: Strs = Strs::new("FIT_THIS_WEEK");
static FLAMES_NOT_FIRE: Strs = Strs::new("FLAMES_NOT_FIRE");
static FLOPPY_RED_FLAG: Strs = Strs::new("FLOPPY_RED_FLAG");
static FLOPPY_SLEEPY: Strs = Strs::new("FLOPPY_SLEEPY");
static FLOPPY_THING: Strs = Strs::new("FLOPPY_THING");
static FOLLOW_UP: Strs = Strs::new("FOLLOW_UP");
static FOLLOW_UP_ANCHOR: Strs = Strs::new("FOLLOW_UP_ANCHOR");
static FOOD_BONE: Strs = Strs::new("FOOD_BONE");
static FOOD_CUT: Strs = Strs::new("FOOD_CUT");
static FOOD_WORDS: Strs = Strs::new("FOOD_WORDS");
static FORAGED_FINE: Strs = Strs::new("FORAGED_FINE");
static FRICTION_BURN: Strs = Strs::new("FRICTION_BURN");
static FRICTION_BURN_HEAT: Strs = Strs::new("FRICTION_BURN_HEAT");
static FRICTION_BURN_NOUN: Strs = Strs::new("FRICTION_BURN_NOUN");
static FRICTION_WORD: Strs = Strs::new("FRICTION_WORD");
static FRIGHTENED: Strs = Strs::new("FRIGHTENED");
static FRIGHTENED_REPLY: Strs = Strs::new("FRIGHTENED_REPLY");
static FUEL_AND_SOLVENT: Str = Str::new("FUEL_AND_SOLVENT");
static GAME_NOT_A_BLOW: Strs = Strs::new("GAME_NOT_A_BLOW");
static GARDEN_WORDS: Strs = Strs::new("GARDEN_WORDS");
static GASPING_THIRST: Strs = Strs::new("GASPING_THIRST");
static GASP_FINE: Strs = Strs::new("GASP_FINE");
static GASP_RED: Strs = Strs::new("GASP_RED");
static GAS_IN_ROOM: Strs = Strs::new("GAS_IN_ROOM");
static GAS_NOW: Strs = Strs::new("GAS_NOW");
static GAS_ON_YOU: Strs = Strs::new("GAS_ON_YOU");
static GAS_PAST: Strs = Strs::new("GAS_PAST");
static GAS_SMELL: Strs = Strs::new("GAS_SMELL");
static GAS_WORD: Strs = Strs::new("GAS_WORD");
static GIVING_UP_GROUP: Strs = Strs::new("GIVING_UP_GROUP");
static GIVING_UP_HELD_ON: Strs = Strs::new("GIVING_UP_HELD_ON");
static GO_IN_NOT_ABOUT_IT: Strs = Strs::new("GO_IN_NOT_ABOUT_IT");
static GO_IS_A_CALL: Strs = Strs::new("GO_IS_A_CALL");
static GO_ON_LIVING: Strs = Strs::new("GO_ON_LIVING");
static GRATER_CUT: Strs = Strs::new("GRATER_CUT");
static GREY_ALARM: Strs = Strs::new("GREY_ALARM");
static GREY_ARRIVING: Strs = Strs::new("GREY_ARRIVING");
static GREY_BODY: Strs = Strs::new("GREY_BODY");
static GREY_COLOUR: Strs = Strs::new("GREY_COLOUR");
static GREY_HAIR: Strs = Strs::new("GREY_HAIR");
static GREY_SELF: Strs = Strs::new("GREY_SELF");
static GRIME_CLEANING: Strs = Strs::new("GRIME_CLEANING");
static GRIME_OFF: Strs = Strs::new("GRIME_OFF");
static GRUNTING_FINE: Strs = Strs::new("GRUNTING_FINE");
static GRUNTING_NOW: Strs = Strs::new("GRUNTING_NOW");
static HAIR_SCORCH: Strs = Strs::new("HAIR_SCORCH");
static HAIR_SCORCH_ANSWER: Strs = Strs::new("HAIR_SCORCH_ANSWER");
static HAIR_SCORCH_SKIN: Strs = Strs::new("HAIR_SCORCH_SKIN");
static HANGED_DONE: Strs = Strs::new("HANGED_DONE");
static HANGED_LONG_AGO: Strs = Strs::new("HANGED_LONG_AGO");
static HARD_TO_WAKE: Strs = Strs::new("HARD_TO_WAKE");
static HARM_DOING: Rx = Rx::new("HARM_DOING");
static HEADACHE_BAD: Strs = Strs::new("HEADACHE_BAD");
static HEADACHE_STORY: Strs = Strs::new("HEADACHE_STORY");
static HEADACHE_SUDDEN: Strs = Strs::new("HEADACHE_SUDDEN");
static HEADACHE_WORST_EVER: Strs = Strs::new("HEADACHE_WORST_EVER");
static HEAD_BANG: Strs = Strs::new("HEAD_BANG");
static HEAD_FALL: Strs = Strs::new("HEAD_FALL");
static HEAD_FALL_SIGN: Strs = Strs::new("HEAD_FALL_SIGN");
static HEAD_KNOCKOUT: Strs = Strs::new("HEAD_KNOCKOUT");
static HEAD_OUT_COLD: Strs = Strs::new("HEAD_OUT_COLD");
static HEAD_PAST: Strs = Strs::new("HEAD_PAST");
static HEAD_RED: Strs = Strs::new("HEAD_RED");
static HEAD_SEEN: Strs = Strs::new("HEAD_SEEN");
static HEARD_ASSAULT_OFF: Strs = Strs::new("HEARD_ASSAULT_OFF");
static HEAT_BREATH_RED: Strs = Strs::new("HEAT_BREATH_RED");
static HEAT_CAUSE: Strs = Strs::new("HEAT_CAUSE");
static HEAT_NOT_WEATHER: Strs = Strs::new("HEAT_NOT_WEATHER");
static HELD_CRISIS_DETAIL: Strs = Strs::new("HELD_CRISIS_DETAIL");
static HELD_CRISIS_OFF: Strs = Strs::new("HELD_CRISIS_OFF");
static HELP_ARRIVED: Strs = Strs::new("HELP_ARRIVED");
static HELP_ASK_MORE: Strs = Strs::new("HELP_ASK_MORE");
static HELP_SOMEONE: Strs = Strs::new("HELP_SOMEONE");
static HIT_AS_A_CHILD: Strs = Strs::new("HIT_AS_A_CHILD");
static HIT_IN_FUN: Strs = Strs::new("HIT_IN_FUN");
static HIT_IN_FUN_HARD: Strs = Strs::new("HIT_IN_FUN_HARD");
static HIT_THE_LAUGH_IS_HIS: Strs = Strs::new("HIT_THE_LAUGH_IS_HIS");
static HIT_THING_NOT_PERSON: Strs = Strs::new("HIT_THING_NOT_PERSON");
static HIT_WITH_A_THING: Strs = Strs::new("HIT_WITH_A_THING");
static HOPELESS: Strs = Strs::new("HOPELESS");
static HOW_LONG: Strs = Strs::new("HOW_LONG");
static HOW_TO_TREAT: Strs = Strs::new("HOW_TO_TREAT");
static HUMAN_ANCHOR: Strs = Strs::new("HUMAN_ANCHOR");
static HUMAN_BITTEN: Strs = Strs::new("HUMAN_BITTEN");
static HUMAN_OWN_INJURY: Strs = Strs::new("HUMAN_OWN_INJURY");
static HUMAN_TOO: Strs = Strs::new("HUMAN_TOO");
static HURT_HIMSELF_BARE: Strs = Strs::new("HURT_HIMSELF_BARE");
static HURT_HIMSELF_MEANT: Strs = Strs::new("HURT_HIMSELF_MEANT");
static HYPERBOLE_BLOCK: Strs = Strs::new("HYPERBOLE_BLOCK");
static HYPERBOLE_WISH: Strs = Strs::new("HYPERBOLE_WISH");
static HYPO: Strs = Strs::new("HYPO");
static HYPO_HOW_KNOW: Strs = Strs::new("HYPO_HOW_KNOW");
static HYPO_LIE_IN: Strs = Strs::new("HYPO_LIE_IN");
static HYPO_NOW: Strs = Strs::new("HYPO_NOW");
static HYPO_NO_MOUTH: Strs = Strs::new("HYPO_NO_MOUTH");
static HYPO_PAST: Strs = Strs::new("HYPO_PAST");
static HYPO_QUESTION: Strs = Strs::new("HYPO_QUESTION");
static HYPO_STRONG: Strs = Strs::new("HYPO_STRONG");
static HYPO_STRONG_OTHER: Strs = Strs::new("HYPO_STRONG_OTHER");
static IDIOM: Strs = Strs::new("IDIOM");
static INSECT_BITE_NEEDS_A_LOOK: Strs = Strs::new("INSECT_BITE_NEEDS_A_LOOK");
static I_DID_NOT_HIT: Strs = Strs::new("I_DID_NOT_HIT");
static I_HIT_SOMEONE: Strs = Strs::new("I_HIT_SOMEONE");
static KICKED_PERSON: Strs = Strs::new("KICKED_PERSON");
static LAUGH_BREATH_RED: Strs = Strs::new("LAUGH_BREATH_RED");
static LAUGH_OTHER_PERSON: Strs = Strs::new("LAUGH_OTHER_PERSON");
static LAUGH_TAKES_BREATH: Strs = Strs::new("LAUGH_TAKES_BREATH");
static LAUGH_WORD: Strs = Strs::new("LAUGH_WORD");
static LEG_WORDS: Str = Str::new("LEG_WORDS");
static LENDER_FINE: Strs = Strs::new("LENDER_FINE");
static LEXICON: StrSet = StrSet::new("LEXICON");
static LIMB_AT_AN_ANGLE: Strs = Strs::new("LIMB_AT_AN_ANGLE");
static LIMB_IS_A_THING: Strs = Strs::new("LIMB_IS_A_THING");
static LIMB_PART: Strs = Strs::new("LIMB_PART");
static LIMP_CAUSE: Strs = Strs::new("LIMP_CAUSE");
static LIMP_LONG: Strs = Strs::new("LIMP_LONG");
static LIMP_MECHANISM: Strs = Strs::new("LIMP_MECHANISM");
static LIMP_NOT: Strs = Strs::new("LIMP_NOT");
static LIMP_ONGOING: Strs = Strs::new("LIMP_ONGOING");
static LIMP_OTHER_LIMB: Strs = Strs::new("LIMP_OTHER_LIMB");
static LIMP_PERSON: Strs = Strs::new("LIMP_PERSON");
static LIMP_REPLACED: Strs = Strs::new("LIMP_REPLACED");
static LIMP_SOMEBODY_ELSE: Strs = Strs::new("LIMP_SOMEBODY_ELSE");
static LIPS_COLD: Strs = Strs::new("LIPS_COLD");
static LIPS_LIMB_TOO: Strs = Strs::new("LIPS_LIMB_TOO");
static LIPS_NUMB: Strs = Strs::new("LIPS_NUMB");
static LIPS_RED_FLAG: Strs = Strs::new("LIPS_RED_FLAG");
static LIPS_SWELL_BARE_OFF: Strs = Strs::new("LIPS_SWELL_BARE_OFF");
static LIPS_SWELL_OFF: Strs = Strs::new("LIPS_SWELL_OFF");
static LIPS_TINGLE_BREATH: Strs = Strs::new("LIPS_TINGLE_BREATH");
static LIPS_TINGLE_ELSEWHERE: Strs = Strs::new("LIPS_TINGLE_ELSEWHERE");
static LIPS_TINGLE_FOOD: Strs = Strs::new("LIPS_TINGLE_FOOD");
static LIPS_TINGLE_MOUTH: Strs = Strs::new("LIPS_TINGLE_MOUTH");
static LIPS_TINGLE_OFF: Strs = Strs::new("LIPS_TINGLE_OFF");
static LIPS_TINGLE_RED: Strs = Strs::new("LIPS_TINGLE_RED");
static LIPS_TINGLE_SAID: Strs = Strs::new("LIPS_TINGLE_SAID");
static LIPS_TINGLE_TRIGGER: Strs = Strs::new("LIPS_TINGLE_TRIGGER");
static LOCKED_IN_RISK: Strs = Strs::new("LOCKED_IN_RISK");
static LONG_AGO: Strs = Strs::new("LONG_AGO");
static LOOKS_REAL: StrSet = StrSet::new("LOOKS_REAL");
static LOSS_SAID: Strs = Strs::new("LOSS_SAID");
static LOSS_SUBJECT_AT_END: Strs = Strs::new("LOSS_SUBJECT_AT_END");
static LOW_SUGAR_NOW: Strs = Strs::new("LOW_SUGAR_NOW");
static MACHINE: Str = Str::new("MACHINE");
static MALE_WORDS: Strs = Strs::new("MALE_WORDS");
static MEAL_BREATH: Strs = Strs::new("MEAL_BREATH");
static MILD: Strs = Strs::new("MILD");
static MILD_BODY_ESCALATION: Str = Str::new("MILD_BODY_ESCALATION");
static MILD_BODY_SIGN: Strs = Strs::new("MILD_BODY_SIGN");
static MILD_CHOKE: Strs = Strs::new("MILD_CHOKE");
static MILD_HARMLESS: Strs = Strs::new("MILD_HARMLESS");
static MILD_PRODUCT_BIG: Strs = Strs::new("MILD_PRODUCT_BIG");
static MILD_PRODUCT_SMALL: Strs = Strs::new("MILD_PRODUCT_SMALL");
static MILD_TAKEN_BY_A_CHILD: Strs = Strs::new("MILD_TAKEN_BY_A_CHILD");
static MONEY_NAMED: Rx = Rx::new("MONEY_NAMED");
static MORNING_ROUTINE: Strs = Strs::new("MORNING_ROUTINE");
static MOUTH_BLEED: Strs = Strs::new("MOUTH_BLEED");
static MOUTH_BLEED_NOW: Strs = Strs::new("MOUTH_BLEED_NOW");
static MOUTH_BLEED_OFF: Strs = Strs::new("MOUTH_BLEED_OFF");
static MOUTH_BLEED_RED: Strs = Strs::new("MOUTH_BLEED_RED");
static MOUTH_TINGLE: Strs = Strs::new("MOUTH_TINGLE");
static MY_OWN_PART: Strs = Strs::new("MY_OWN_PART");
static NAMED_PERSON: Strs = Strs::new("NAMED_PERSON");
static NAMED_SICK_FINE: Strs = Strs::new("NAMED_SICK_FINE");
static NEAR_FIRE: Strs = Strs::new("NEAR_FIRE");
static NEAR_FIRE_OVER: Strs = Strs::new("NEAR_FIRE_OVER");
static NEAR_HIT_ON_A_ROAD: Strs = Strs::new("NEAR_HIT_ON_A_ROAD");
static NEAR_HIT_ON_A_ROAD_NOT: Strs = Strs::new("NEAR_HIT_ON_A_ROAD_NOT");
static NEAR_HIT_ON_A_ROAD_PLACE: Strs = Strs::new("NEAR_HIT_ON_A_ROAD_PLACE");
static NEAR_MISS: Strs = Strs::new("NEAR_MISS");
static NEAR_MISS_HARM: Strs = Strs::new("NEAR_MISS_HARM");
static NEAR_THING_OVER: Strs = Strs::new("NEAR_THING_OVER");
static NEWBORN: Strs = Strs::new("NEWBORN");
static NEWBORN_FINE: Strs = Strs::new("NEWBORN_FINE");
static NIGHT_BREATH: Strs = Strs::new("NIGHT_BREATH");
static NIGHT_BREATH_CHILD: Strs = Strs::new("NIGHT_BREATH_CHILD");
static NIGHT_BREATH_DREAM: Strs = Strs::new("NIGHT_BREATH_DREAM");
static NIGHT_BREATH_RED: Strs = Strs::new("NIGHT_BREATH_RED");
static NOSE_RED: Strs = Strs::new("NOSE_RED");
static NOSE_SITE: Strs = Strs::new("NOSE_SITE");
static NOTHING_LEFT: Strs = Strs::new("NOTHING_LEFT");
static NOT_A_BODY_CUT: Strs = Strs::new("NOT_A_BODY_CUT");
static NOT_A_BREATHER: Strs = Strs::new("NOT_A_BREATHER");
static NOT_A_WOUND_OFF: Strs = Strs::new("NOT_A_WOUND_OFF");
static NOT_BREATHING_NOW: Strs = Strs::new("NOT_BREATHING_NOW");
static NOT_JUST_A_NEAR_THING: Strs = Strs::new("NOT_JUST_A_NEAR_THING");
static NOT_NOW: Strs = Strs::new("NOT_NOW");
static NOT_NOW_FICTION: Strs = Strs::new("NOT_NOW_FICTION");
static NOT_STOPPING: Strs = Strs::new("NOT_STOPPING");
static NOT_STOPPING_BARE: Strs = Strs::new("NOT_STOPPING_BARE");
static NOT_STOPPING_OFF: Strs = Strs::new("NOT_STOPPING_OFF");
static NOT_THEIR_OWN: Str = Str::new("NOT_THEIR_OWN");
static NOT_WAKING_BESIDE: Strs = Strs::new("NOT_WAKING_BESIDE");
static OBJECT_IN_HEAD_BATTERY: Strs = Strs::new("OBJECT_IN_HEAD_BATTERY");
static OBJECT_IN_HEAD_BLOW: Strs = Strs::new("OBJECT_IN_HEAD_BLOW");
static OLDER_CHILD_NAMED: Strs = Strs::new("OLDER_CHILD_NAMED");
static OLD_FALL_FINE: Strs = Strs::new("OLD_FALL_FINE");
static OLD_FALL_NOT: Strs = Strs::new("OLD_FALL_NOT");
static OLD_PERSON: Strs = Strs::new("OLD_PERSON");
static OLD_TIME: Strs = Strs::new("OLD_TIME");
static ORDINARY_REFLEXIVE: Strs = Strs::new("ORDINARY_REFLEXIVE");
static OTHER_BODY: Strs = Strs::new("OTHER_BODY");
static OTHER_DYING: Strs = Strs::new("OTHER_DYING");
static OTHER_RISK: Strs = Strs::new("OTHER_RISK");
static OTHER_SELF_HARM: Strs = Strs::new("OTHER_SELF_HARM");
static OUT_COLD_ASLEEP: Strs = Strs::new("OUT_COLD_ASLEEP");
static OUT_COLD_FOUND: Strs = Strs::new("OUT_COLD_FOUND");
static OUT_COLD_LEFT_TO_OTHERS: Strs = Strs::new("OUT_COLD_LEFT_TO_OTHERS");
static OUT_COLD_LONG: Strs = Strs::new("OUT_COLD_LONG");
static OUT_COLD_NIGHT_REPLY: Strs = Strs::new("OUT_COLD_NIGHT_REPLY");
static OUT_COLD_NOW: Strs = Strs::new("OUT_COLD_NOW");
static OUT_COLD_REPLY: Str = Str::new("OUT_COLD_REPLY");
static OUT_COLD_SLEEPY: Strs = Strs::new("OUT_COLD_SLEEPY");
static OUT_COLD_WONT_WAKE: Strs = Strs::new("OUT_COLD_WONT_WAKE");
static OUT_OF_BREATH: Strs = Strs::new("OUT_OF_BREATH");
static OVERDOSE_DRUG: Strs = Strs::new("OVERDOSE_DRUG");
static OVERDOSE_FOOD: Strs = Strs::new("OVERDOSE_FOOD");
static OVERDOSE_OF_A_TREAT: Strs = Strs::new("OVERDOSE_OF_A_TREAT");
static OVERDOSE_REAL_THING: Strs = Strs::new("OVERDOSE_REAL_THING");
static OVERDOSE_WORK: Strs = Strs::new("OVERDOSE_WORK");
static OWN_CRY: Strs = Strs::new("OWN_CRY");
static OWN_DISTRESS: Strs = Strs::new("OWN_DISTRESS");
static OWN_INJURY: Strs = Strs::new("OWN_INJURY");
static OWN_INJURY_IN_MY_CARE: Strs = Strs::new("OWN_INJURY_IN_MY_CARE");
static OWN_INJURY_OTHER_PERSON: Strs = Strs::new("OWN_INJURY_OTHER_PERSON");
static OWN_INJURY_QUESTION: Strs = Strs::new("OWN_INJURY_QUESTION");
static OWN_INJURY_SOMEONE_OR_A_STORY: Strs = Strs::new("OWN_INJURY_SOMEONE_OR_A_STORY");
static OWN_INJURY_YOU: Strs = Strs::new("OWN_INJURY_YOU");
static OWN_LIFE: Strs = Strs::new("OWN_LIFE");
static OWN_LIMB_AT_AN_ANGLE: Strs = Strs::new("OWN_LIMB_AT_AN_ANGLE");
static OWN_WISH: Strs = Strs::new("OWN_WISH");
static OWN_WISH_WORDS: Strs = Strs::new("OWN_WISH_WORDS");
static PACK_FINE: Strs = Strs::new("PACK_FINE");
static PALE_ADULT_FINE: Strs = Strs::new("PALE_ADULT_FINE");
static PALE_ADULT_ITSELF: Strs = Strs::new("PALE_ADULT_ITSELF");
static PALE_ADULT_WHOLE: Strs = Strs::new("PALE_ADULT_WHOLE");
static PALE_EFFORT: Strs = Strs::new("PALE_EFFORT");
static PALE_EFFORT_NO: Strs = Strs::new("PALE_EFFORT_NO");
static PALE_EFFORT_REAL: Strs = Strs::new("PALE_EFFORT_REAL");
static PALE_EFFORT_RED: Strs = Strs::new("PALE_EFFORT_RED");
static PALE_HEAT: Strs = Strs::new("PALE_HEAT");
static PALE_OTHER_RED: Strs = Strs::new("PALE_OTHER_RED");
static PAN_FIRE_SPREAD: Strs = Strs::new("PAN_FIRE_SPREAD");
static PARENT_REFUSAL_BLOW: Strs = Strs::new("PARENT_REFUSAL_BLOW");
static PAST_ATTEMPT: Strs = Strs::new("PAST_ATTEMPT");
static PAST_ATTEMPT_NOT: Strs = Strs::new("PAST_ATTEMPT_NOT");
static PAST_ATTEMPT_STRONG: Strs = Strs::new("PAST_ATTEMPT_STRONG");
static PAST_BLACKOUT_REPLY: Str = Str::new("PAST_BLACKOUT_REPLY");
static PAST_DEATH: Strs = Strs::new("PAST_DEATH");
static PAST_FIT_REPLY: Str = Str::new("PAST_FIT_REPLY");
static PAST_FIT_WEEK_REPLY: Str = Str::new("PAST_FIT_WEEK_REPLY");
static PAST_HARM: Strs = Strs::new("PAST_HARM");
static PAST_NEAR_ACT: Strs = Strs::new("PAST_NEAR_ACT");
static PAST_WISH: Strs = Strs::new("PAST_WISH");
static PAST_WISH_NOT: Strs = Strs::new("PAST_WISH_NOT");
static PERSON_AS_ATTACKER: Rx = Rx::new("PERSON_AS_ATTACKER");
static PERSON_BURNT: Strs = Strs::new("PERSON_BURNT");
static PERSON_WORD: Strs = Strs::new("PERSON_WORD");
static PET_ALL_RIGHT: Strs = Strs::new("PET_ALL_RIGHT");
static PET_ANIMAL_HURT: Strs = Strs::new("PET_ANIMAL_HURT");
static PET_ANY: Strs = Strs::new("PET_ANY");
static PET_APPOSITIVE: Strs = Strs::new("PET_APPOSITIVE");
static PET_APPOSITIVE_SUB: Rx = Rx::new("PET_APPOSITIVE_SUB");
static PET_BEHAVIOUR: Strs = Strs::new("PET_BEHAVIOUR");
static PET_BLUE_PART: Strs = Strs::new("PET_BLUE_PART");
static PET_BLUE_WENT: Strs = Strs::new("PET_BLUE_WENT");
static PET_BREATH: Strs = Strs::new("PET_BREATH");
static PET_BREATH_NOW: Strs = Strs::new("PET_BREATH_NOW");
static PET_BREATH_ODD: Strs = Strs::new("PET_BREATH_ODD");
static PET_CANT_SETTLE: Strs = Strs::new("PET_CANT_SETTLE");
static PET_COLLAPSE: Strs = Strs::new("PET_COLLAPSE");
static PET_CRISIS: Strs = Strs::new("PET_CRISIS");
static PET_DYING: Strs = Strs::new("PET_DYING");
static PET_EATEN_THING: Strs = Strs::new("PET_EATEN_THING");
static PET_EFFORT: Strs = Strs::new("PET_EFFORT");
static PET_FELL: Strs = Strs::new("PET_FELL");
static PET_FINE: Strs = Strs::new("PET_FINE");
static PET_FIRST_AID_ROWS: StrSet = StrSet::new("PET_FIRST_AID_ROWS");
static PET_FIT: Strs = Strs::new("PET_FIT");
static PET_GASP: Strs = Strs::new("PET_GASP");
static PET_GASP_EXPLAINED: Strs = Strs::new("PET_GASP_EXPLAINED");
static PET_GASP_FINE: Strs = Strs::new("PET_GASP_FINE");
static PET_GASP_NOUN: Strs = Strs::new("PET_GASP_NOUN");
static PET_GASP_ONGOING: Strs = Strs::new("PET_GASP_ONGOING");
static PET_GASP_RED: Strs = Strs::new("PET_GASP_RED");
static PET_GASP_SLEEP: Strs = Strs::new("PET_GASP_SLEEP");
static PET_GASP_SLEEP_FINE: Strs = Strs::new("PET_GASP_SLEEP_FINE");
static PET_GUMS_TOLD: Strs = Strs::new("PET_GUMS_TOLD");
static PET_HARM: Strs = Strs::new("PET_HARM");
static PET_HURTING_ITSELF: Strs = Strs::new("PET_HURTING_ITSELF");
static PET_LONG_AGO: Strs = Strs::new("PET_LONG_AGO");
static PET_MILD: Strs = Strs::new("PET_MILD");
static PET_MOUTH: Strs = Strs::new("PET_MOUTH");
static PET_NOW: Strs = Strs::new("PET_NOW");
static PET_OWN: Strs = Strs::new("PET_OWN");
static PET_OWNS_A_THING: Strs = Strs::new("PET_OWNS_A_THING");
static PET_PASSES_BLOOD: Str = Str::new("PET_PASSES_BLOOD");
static PET_PAST: Strs = Strs::new("PET_PAST");
static PET_SIGHT: Strs = Strs::new("PET_SIGHT");
static PET_SUBJECT: Strs = Strs::new("PET_SUBJECT");
static PET_SUBJECT_PLURAL: Strs = Strs::new("PET_SUBJECT_PLURAL");
static PET_TOOTH: Strs = Strs::new("PET_TOOTH");
static PET_WAKE: Str = Str::new("PET_WAKE");
static PET_WANTS_OUT: Strs = Strs::new("PET_WANTS_OUT");
static PET_WEAK: Strs = Strs::new("PET_WEAK");
static PET_WORDS: Strs = Strs::new("PET_WORDS");
static PET_WOUND: Strs = Strs::new("PET_WOUND");
static PILLS_OVERDOSE: Strs = Strs::new("PILLS_OVERDOSE");
static PLANT_WORDS: Strs = Strs::new("PLANT_WORDS");
static PLAN_FRAME_LAUGH: Strs = Strs::new("PLAN_FRAME_LAUGH");
static PLAYFUL_HIT: Strs = Strs::new("PLAYFUL_HIT");
static PLAY_FACE_REPLIES: Strs = Strs::new("PLAY_FACE_REPLIES");
static POISON_SWALLOW_VERB: Strs = Strs::new("POISON_SWALLOW_VERB");
static POO_FINE: Strs = Strs::new("POO_FINE");
static POO_MEAL: Strs = Strs::new("POO_MEAL");
static POO_PANTS: Strs = Strs::new("POO_PANTS");
static POO_RED: Strs = Strs::new("POO_RED");
static POO_WORD: Strs = Strs::new("POO_WORD");
static PREGNANT_LINE: Strs = Strs::new("PREGNANT_LINE");
static PREVENTION_OR_BURNOUT: Strs = Strs::new("PREVENTION_OR_BURNOUT");
static PRO_NOT_ANSWERING: Strs = Strs::new("PRO_NOT_ANSWERING");
static PUNCTURE_OFF: Strs = Strs::new("PUNCTURE_OFF");
static PURSUIT_FINE: Strs = Strs::new("PURSUIT_FINE");
static PUSHED_AWAY: Strs = Strs::new("PUSHED_AWAY");
static PUSHED_AWAY_BUT_SOMETHING_ELSE: Strs = Strs::new("PUSHED_AWAY_BUT_SOMETHING_ELSE");
static PUSHED_AWAY_REPLY: Strs = Strs::new("PUSHED_AWAY_REPLY");
static PUSHED_AWAY_THEN_RESTRAINT: Strs = Strs::new("PUSHED_AWAY_THEN_RESTRAINT");
static QUESTION_OPENER: Strs = Strs::new("QUESTION_OPENER");
static RASH: Str = Str::new("RASH");
static RASH_COMPANY: Strs = Strs::new("RASH_COMPANY");
static RASH_DEFINITION: Strs = Strs::new("RASH_DEFINITION");
static RASH_FADE: Strs = Strs::new("RASH_FADE");
static RASH_FLAT: Strs = Strs::new("RASH_FLAT");
static RASH_HEADACHE_FINE: Strs = Strs::new("RASH_HEADACHE_FINE");
static RASH_NAMED: Strs = Strs::new("RASH_NAMED");
static RASH_PRESSED: Strs = Strs::new("RASH_PRESSED");
static RASH_RED: Strs = Strs::new("RASH_RED");
static RAYNAUD: Strs = Strs::new("RAYNAUD");
static RAZOR_BURN: Strs = Strs::new("RAZOR_BURN");
static RECENT_FIT: Strs = Strs::new("RECENT_FIT");
static RECENT_FIT_FINE: Strs = Strs::new("RECENT_FIT_FINE");
static RECENT_TIME: Strs = Strs::new("RECENT_TIME");
static RED_ANCHOR: Strs = Strs::new("RED_ANCHOR");
static REMORSE_FOR_THE_ACT: Strs = Strs::new("REMORSE_FOR_THE_ACT");
static RESTRAINT_BESIDE: Strs = Strs::new("RESTRAINT_BESIDE");
static RING_DANGER: Strs = Strs::new("RING_DANGER");
static RING_NOT_WORN: Strs = Strs::new("RING_NOT_WORN");
static RING_WORD: Strs = Strs::new("RING_WORD");
static ROOM_OR_HOUSE: Strs = Strs::new("ROOM_OR_HOUSE");
static ROUTINE_REASON: Strs = Strs::new("ROUTINE_REASON");
static ROUTINE_WAKE: Strs = Strs::new("ROUTINE_WAKE");
static SAFE_NOW: Strs = Strs::new("SAFE_NOW");
static SAID_AS_IT_HAPPENS: Strs = Strs::new("SAID_AS_IT_HAPPENS");
static SCHOOL_PLACE: Strs = Strs::new("SCHOOL_PLACE");
static SEIZURE: Strs = Strs::new("SEIZURE");
static SEIZURE_FIGURATIVE: Strs = Strs::new("SEIZURE_FIGURATIVE");
static SEIZURE_STORY_GUARD: Strs = Strs::new("SEIZURE_STORY_GUARD");
static SELF_CUT: Strs = Strs::new("SELF_CUT");
static SELF_DEAD_FEELING: Strs = Strs::new("SELF_DEAD_FEELING");
static SELF_HARM: Strs = Strs::new("SELF_HARM");
static SELF_HARM_AGAIN: Strs = Strs::new("SELF_HARM_AGAIN");
static SELF_HARM_BY_OTHER: Strs = Strs::new("SELF_HARM_BY_OTHER");
static SELF_HARM_DELIBERATE: Strs = Strs::new("SELF_HARM_DELIBERATE");
static SELF_HARM_DELIBERATE_ACT: Strs = Strs::new("SELF_HARM_DELIBERATE_ACT");
static SELF_HARM_DENIED: Strs = Strs::new("SELF_HARM_DENIED");
static SELF_HARM_DESERVE: Strs = Strs::new("SELF_HARM_DESERVE");
static SELF_HARM_HABIT: Strs = Strs::new("SELF_HARM_HABIT");
static SELF_HARM_INTENT: Strs = Strs::new("SELF_HARM_INTENT");
static SELF_HARM_LOOKALIKE: Strs = Strs::new("SELF_HARM_LOOKALIKE");
static SELF_HARM_NOT_EATING: Strs = Strs::new("SELF_HARM_NOT_EATING");
static SELF_HARM_OWN_PART: Strs = Strs::new("SELF_HARM_OWN_PART");
static SELF_HARM_REASON: Strs = Strs::new("SELF_HARM_REASON");
static SELF_HARM_REPEAT_CAUSE: Strs = Strs::new("SELF_HARM_REPEAT_CAUSE");
static SELF_HARM_REPEAT_CUT: Strs = Strs::new("SELF_HARM_REPEAT_CUT");
static SELF_HARM_REPEAT_GRIND: Strs = Strs::new("SELF_HARM_REPEAT_GRIND");
static SELF_HARM_REPEAT_MEANT: Strs = Strs::new("SELF_HARM_REPEAT_MEANT");
static SELF_HARM_SCARS: Strs = Strs::new("SELF_HARM_SCARS");
static SELF_HARM_SCARS_NOT: Strs = Strs::new("SELF_HARM_SCARS_NOT");
static SELF_HARM_TOPIC_OFF: Strs = Strs::new("SELF_HARM_TOPIC_OFF");
static SELF_OWN_DYING: Strs = Strs::new("SELF_OWN_DYING");
static SELF_REASSURED: Strs = Strs::new("SELF_REASSURED");
static SELF_RUMINATION: Strs = Strs::new("SELF_RUMINATION");
static SELF_TALK: Strs = Strs::new("SELF_TALK");
static SEPSIS_SIGN: Strs = Strs::new("SEPSIS_SIGN");
static SHAKEN: Strs = Strs::new("SHAKEN");
static SHAKEN_FINE: Strs = Strs::new("SHAKEN_FINE");
static SHAKEN_REPLY: Strs = Strs::new("SHAKEN_REPLY");
static SHAKE_ADULT: Strs = Strs::new("SHAKE_ADULT");
static SHAKE_BABY_WORD: Strs = Strs::new("SHAKE_BABY_WORD");
static SHAKE_CHILD: Strs = Strs::new("SHAKE_CHILD");
static SHAKE_CHILD_CONTEXT: Strs = Strs::new("SHAKE_CHILD_CONTEXT");
static SHAKE_CHILD_REPLY: Strs = Strs::new("SHAKE_CHILD_REPLY");
static SHAKE_COPING: Strs = Strs::new("SHAKE_COPING");
static SHAKE_IDIOM: Strs = Strs::new("SHAKE_IDIOM");
static SHAKE_REPLY: Strs = Strs::new("SHAKE_REPLY");
static SHORT_OF_BREATH: Strs = Strs::new("SHORT_OF_BREATH");
static SHOT_FINE: Strs = Strs::new("SHOT_FINE");
static SHOT_SELF: Strs = Strs::new("SHOT_SELF");
static SIGHT_A_WHILE: Strs = Strs::new("SIGHT_A_WHILE");
static SIGHT_WORD: Strs = Strs::new("SIGHT_WORD");
static SLUR_NOT_A_STROKE_WORDS: Strs = Strs::new("SLUR_NOT_A_STROKE_WORDS");
static SLUR_NOT_SUDDEN: Strs = Strs::new("SLUR_NOT_SUDDEN");
static SLUR_RED_BESIDE: Str = Str::new("SLUR_RED_BESIDE");
static SLUR_SMALL_CHILD_CALM: Strs = Strs::new("SLUR_SMALL_CHILD_CALM");
static SLUR_SMALL_CHILD_URGENT: Strs = Strs::new("SLUR_SMALL_CHILD_URGENT");
static SLUR_SMALL_CHILD_URGENT_REPLY: Strs = Strs::new("SLUR_SMALL_CHILD_URGENT_REPLY");
static SLUR_SUDDEN: Strs = Strs::new("SLUR_SUDDEN");
static SMALL_CHILD_LASH_REPLIES: Strs = Strs::new("SMALL_CHILD_LASH_REPLIES");
static SMALL_CHILD_REPLY_GIVEN: Rx = Rx::new("SMALL_CHILD_REPLY_GIVEN");
static SMELL_ON_A_THING: Rx = Rx::new("SMELL_ON_A_THING");
static SMOKE_IS_A_SMELL: Strs = Strs::new("SMOKE_IS_A_SMELL");
static SNAKE_BITE: Strs = Strs::new("SNAKE_BITE");
static SNAKE_LOOKALIKE: Strs = Strs::new("SNAKE_LOOKALIKE");
static SOCKET_THING_FINE: Strs = Strs::new("SOCKET_THING_FINE");
static SOFT_BLEED: Strs = Strs::new("SOFT_BLEED");
static SOFT_BLEED_ANCHOR: Strs = Strs::new("SOFT_BLEED_ANCHOR");
static SOFT_BLEED_OFF: Strs = Strs::new("SOFT_BLEED_OFF");
static SOFT_FACE_PUSH: Str = Str::new("SOFT_FACE_PUSH");
static SOFT_FACE_STOP: Strs = Strs::new("SOFT_FACE_STOP");
static SOMEONE_AS_SUBJECT: Strs = Strs::new("SOMEONE_AS_SUBJECT");
static SOMEONE_ELSES_DESPAIR: Strs = Strs::new("SOMEONE_ELSES_DESPAIR");
static SOMEONE_ELSE_OWN: Strs = Strs::new("SOMEONE_ELSE_OWN");
static SOMEONE_ELSE_RISK: Strs = Strs::new("SOMEONE_ELSE_RISK");
static SOMEONE_HOLDS_THE_MONEY: Rx = Rx::new("SOMEONE_HOLDS_THE_MONEY");
static SOMEONE_REPORTED: Strs = Strs::new("SOMEONE_REPORTED");
static SO_I_TOO: Strs = Strs::new("SO_I_TOO");
static SPEAKER_IS_PATIENT: Strs = Strs::new("SPEAKER_IS_PATIENT");
static SPELL_RED: Strs = Strs::new("SPELL_RED");
static SPICY_MOUTH: Strs = Strs::new("SPICY_MOUTH");
static SPIKED_STAND_DOWN: Strs = Strs::new("SPIKED_STAND_DOWN");
static SPINE_BACK_FALL: Strs = Strs::new("SPINE_BACK_FALL");
static SPINE_BACK_WORK: Strs = Strs::new("SPINE_BACK_WORK");
static SPINE_BREAK: Strs = Strs::new("SPINE_BREAK");
static SPINE_BREAK_ANSWER: Strs = Strs::new("SPINE_BREAK_ANSWER");
static SPINE_NUMB_LOWER: Strs = Strs::new("SPINE_NUMB_LOWER");
static SPOKEN_ATTEMPT_UNDATED: Strs = Strs::new("SPOKEN_ATTEMPT_UNDATED");
static STIFF_FINE: Strs = Strs::new("STIFF_FINE");
static STIFF_NECK: Strs = Strs::new("STIFF_NECK");
static STILL_GOING: Strs = Strs::new("STILL_GOING");
static STING_MILD: Strs = Strs::new("STING_MILD");
static STING_SETTLED: Strs = Strs::new("STING_SETTLED");
static STING_WINGED: Strs = Strs::new("STING_WINGED");
static STING_WORDS: Strs = Strs::new("STING_WORDS");
static STOOL_LONG: Strs = Strs::new("STOOL_LONG");
static STRANGLED_SELF: Strs = Strs::new("STRANGLED_SELF");
static STRANGLE_LOOKALIKE: Strs = Strs::new("STRANGLE_LOOKALIKE");
static STROKE: Strs = Strs::new("STROKE");
static STROKE_FAST: Strs = Strs::new("STROKE_FAST");
static STROKE_FAST_CAUSE: Strs = Strs::new("STROKE_FAST_CAUSE");
static STROKE_FAST_HARD: Strs = Strs::new("STROKE_FAST_HARD");
static STROKE_FAST_JAB: Strs = Strs::new("STROKE_FAST_JAB");
static STROKE_FAST_LIFT_CEILING: Strs = Strs::new("STROKE_FAST_LIFT_CEILING");
static STROKE_FAST_OFF: Strs = Strs::new("STROKE_FAST_OFF");
static STROKE_FAST_PATTERN: Strs = Strs::new("STROKE_FAST_PATTERN");
static STROKE_FAST_SENSORY: Strs = Strs::new("STROKE_FAST_SENSORY");
static STROKE_FAST_SHOULDER: Strs = Strs::new("STROKE_FAST_SHOULDER");
static STROKE_FAST_WEAK: Strs = Strs::new("STROKE_FAST_WEAK");
static STROKE_HAND: Strs = Strs::new("STROKE_HAND");
static STROKE_HAND_CHRONIC: Strs = Strs::new("STROKE_HAND_CHRONIC");
static STROKE_HAND_HARD: Strs = Strs::new("STROKE_HAND_HARD");
static STROKE_HAND_OTHER: Strs = Strs::new("STROKE_HAND_OTHER");
static STROKE_HAND_SELF: Strs = Strs::new("STROKE_HAND_SELF");
static STROKE_PAIR_BOTH: Strs = Strs::new("STROKE_PAIR_BOTH");
static STROKE_WEAK_OFF: Strs = Strs::new("STROKE_WEAK_OFF");
static STRUCK_BY_A_VEHICLE: Strs = Strs::new("STRUCK_BY_A_VEHICLE");
static STRUCK_BY_A_VEHICLE_NOT: Strs = Strs::new("STRUCK_BY_A_VEHICLE_NOT");
static STRUCK_BY_VEHICLE: Strs = Strs::new("STRUCK_BY_VEHICLE");
static STRUCK_HYPOTHETICAL: Strs = Strs::new("STRUCK_HYPOTHETICAL");
static STRUCK_MY_BIKE: Strs = Strs::new("STRUCK_MY_BIKE");
static STRUCK_NOT_NOW: Strs = Strs::new("STRUCK_NOT_NOW");
static STUCK_FINE: Strs = Strs::new("STUCK_FINE");
static SUGAR_IN_A_RECIPE: Strs = Strs::new("SUGAR_IN_A_RECIPE");
static SUGAR_MENTION: Rx = Rx::new("SUGAR_MENTION");
static SUGAR_SHOP: Strs = Strs::new("SUGAR_SHOP");
static SUICIDE_ABOUT_A_THING: Strs = Strs::new("SUICIDE_ABOUT_A_THING");
static SUICIDE_LESSON: Strs = Strs::new("SUICIDE_LESSON");
static SUICIDE_MENTION_CONTEXT: Strs = Strs::new("SUICIDE_MENTION_CONTEXT");
static SUICIDE_OTHER: Strs = Strs::new("SUICIDE_OTHER");
static SUICIDE_TOPIC: Strs = Strs::new("SUICIDE_TOPIC");
static SWALLOWED_BAD: Strs = Strs::new("SWALLOWED_BAD");
static SWALLOWED_BAD_REPLY: Strs = Strs::new("SWALLOWED_BAD_REPLY");
static SWALLOWED_BAD_SELF: Strs = Strs::new("SWALLOWED_BAD_SELF");
static SWALLOWED_CHILD: Strs = Strs::new("SWALLOWED_CHILD");
static SWALLOWED_SELF: Strs = Strs::new("SWALLOWED_SELF");
static SWALLOW_GONE_BREATH: Strs = Strs::new("SWALLOW_GONE_BREATH");
static SWALLOW_GONE_OFF: Strs = Strs::new("SWALLOW_GONE_OFF");
static SWALLOW_GONE_RED: Strs = Strs::new("SWALLOW_GONE_RED");
static SWELLING_NOW: Strs = Strs::new("SWELLING_NOW");
static SWITCH_OFF: Strs = Strs::new("SWITCH_OFF");
static TALKS_ABOUT_IT: Strs = Strs::new("TALKS_ABOUT_IT");
static TALK_HABIT: Strs = Strs::new("TALK_HABIT");
static TEEN_AGE: Strs = Strs::new("TEEN_AGE");
static TEMP_FALLING_WEATHER: Strs = Strs::new("TEMP_FALLING_WEATHER");
static TESTIS_LUMP: Strs = Strs::new("TESTIS_LUMP");
static THE_INJURY_NAMED_BEYOND_A_QUESTION: StrSet = StrSet::new("THE_INJURY_NAMED_BEYOND_A_QUESTION");
static THING_BURNT: Strs = Strs::new("THING_BURNT");
static THING_COLLAPSE: Strs = Strs::new("THING_COLLAPSE");
static THING_FIT_SUBJECT: Strs = Strs::new("THING_FIT_SUBJECT");
static THIRD_PERSON_ACTOR: Strs = Strs::new("THIRD_PERSON_ACTOR");
static THIRD_PERSON_WISH: Strs = Strs::new("THIRD_PERSON_WISH");
static THROAT_THICK: Strs = Strs::new("THROAT_THICK");
static THROAT_TIGHT: Strs = Strs::new("THROAT_TIGHT");
static THROAT_TIGHT_CAUSE: Strs = Strs::new("THROAT_TIGHT_CAUSE");
static THROAT_TIGHT_EFFORT: Strs = Strs::new("THROAT_TIGHT_EFFORT");
static THROAT_TIGHT_MILD: Strs = Strs::new("THROAT_TIGHT_MILD");
static THROAT_TIGHT_RED: Strs = Strs::new("THROAT_TIGHT_RED");
static TICK_ALREADY: Strs = Strs::new("TICK_ALREADY");
static TINY_AGE: Strs = Strs::new("TINY_AGE");
static TINY_AGE_FINE: Strs = Strs::new("TINY_AGE_FINE");
static TINY_SAID: Strs = Strs::new("TINY_SAID");
static TOOK_OUT_OF: Strs = Strs::new("TOOK_OUT_OF");
static TOOTH_CHILD: Strs = Strs::new("TOOTH_CHILD");
static TOOTH_CHIPPED_FINE: Strs = Strs::new("TOOTH_CHIPPED_FINE");
static TOOTH_KNOCK: Strs = Strs::new("TOOTH_KNOCK");
static TOOTH_OUT: Strs = Strs::new("TOOTH_OUT");
static TOOTH_OUT_FINE: Strs = Strs::new("TOOTH_OUT_FINE");
static TORSION_FINE: Strs = Strs::new("TORSION_FINE");
static TORSION_HARD: Strs = Strs::new("TORSION_HARD");
static TORSION_LONG: Strs = Strs::new("TORSION_LONG");
static TORSION_MILD: Strs = Strs::new("TORSION_MILD");
static TORSION_ONSET: Strs = Strs::new("TORSION_ONSET");
static TORSION_RED: Strs = Strs::new("TORSION_RED");
static TORSION_SICK: Strs = Strs::new("TORSION_SICK");
static TRAPPED_HELD_BY_A_PERSON: Rx = Rx::new("TRAPPED_HELD_BY_A_PERSON");
static TRAPPED_NOT_A_WAIT: Rx = Rx::new("TRAPPED_NOT_A_WAIT");
static TRAPPED_SELF_STUCK: Rx = Rx::new("TRAPPED_SELF_STUCK");
static TRAPPED_VEHICLE_NOT_NOW: Strs = Strs::new("TRAPPED_VEHICLE_NOT_NOW");
static TRUE_NEWBORN: Str = Str::new("TRUE_NEWBORN");
static TRUE_NEWBORN_FINE: Strs = Strs::new("TRUE_NEWBORN_FINE");
static UNCONSCIOUS: Strs = Strs::new("UNCONSCIOUS");
static UNRESPONSIVE_MANNER: Strs = Strs::new("UNRESPONSIVE_MANNER");
static UNWELL_BESIDE_THE_PLATE: Strs = Strs::new("UNWELL_BESIDE_THE_PLATE");
static URGENT_NOW: Strs = Strs::new("URGENT_NOW");
static VEHICLE_BURNING_A_PERSON: Strs = Strs::new("VEHICLE_BURNING_A_PERSON");
static VEHICLE_BURNING_A_PERSON_HARD: Strs = Strs::new("VEHICLE_BURNING_A_PERSON_HARD");
static VEHICLE_BURNING_SMELL: Strs = Strs::new("VEHICLE_BURNING_SMELL");
static VEHICLE_BURNING_SMELL_HARD: Strs = Strs::new("VEHICLE_BURNING_SMELL_HARD");
static VEHICLE_EXHAUST: Strs = Strs::new("VEHICLE_EXHAUST");
static VEHICLE_EXHAUST_HARD: Strs = Strs::new("VEHICLE_EXHAUST_HARD");
static VEHICLE_FIRE_OVER: Strs = Strs::new("VEHICLE_FIRE_OVER");
static VEHICLE_FIRE_OVER_HARD: Strs = Strs::new("VEHICLE_FIRE_OVER_HARD");
static VEHICLE_NOT_FIRE: Strs = Strs::new("VEHICLE_NOT_FIRE");
static VEHICLE_STILL_FIRE: Strs = Strs::new("VEHICLE_STILL_FIRE");
static VIOLENT_THREAT: Strs = Strs::new("VIOLENT_THREAT");
static VISIT_OR_TIME: Strs = Strs::new("VISIT_OR_TIME");
static WAKE_RED: Strs = Strs::new("WAKE_RED");
static WAKE_UP_SOMEONE: Strs = Strs::new("WAKE_UP_SOMEONE");
static WATER_BREATH_RED: Strs = Strs::new("WATER_BREATH_RED");
static WATER_DANGER: Strs = Strs::new("WATER_DANGER");
static WATER_DANGER_PERSON: Strs = Strs::new("WATER_DANGER_PERSON");
static WATER_DROWNING_FINE: Strs = Strs::new("WATER_DROWNING_FINE");
static WATER_DROWNING_NOW: Strs = Strs::new("WATER_DROWNING_NOW");
static WATER_FINE: Strs = Strs::new("WATER_FINE");
static WATER_HEAD_FEAR: Strs = Strs::new("WATER_HEAD_FEAR");
static WATER_HUMAN: Strs = Strs::new("WATER_HUMAN");
static WATER_IN_NOW: Strs = Strs::new("WATER_IN_NOW");
static WATER_NOT_A_PERSON: Strs = Strs::new("WATER_NOT_A_PERSON");
static WATER_PERSON_TOO: Strs = Strs::new("WATER_PERSON_TOO");
static WATER_PET: Strs = Strs::new("WATER_PET");
static WATER_SELF: Strs = Strs::new("WATER_SELF");
static WATER_SELF_DIP: Strs = Strs::new("WATER_SELF_DIP");
static WATER_SELF_PUSH_FINE: Strs = Strs::new("WATER_SELF_PUSH_FINE");
static WATER_STILL_BAD: Strs = Strs::new("WATER_STILL_BAD");
static WATER_STORY: Strs = Strs::new("WATER_STORY");
static WATER_SWIM: Strs = Strs::new("WATER_SWIM");
static WATER_THING_DROPPED: Strs = Strs::new("WATER_THING_DROPPED");
static WATER_URGENT: Strs = Strs::new("WATER_URGENT");
static WEIGHT_HIP: Strs = Strs::new("WEIGHT_HIP");
static WEIGHT_JOINT: Strs = Strs::new("WEIGHT_JOINT");
static WEIGHT_LONG: Strs = Strs::new("WEIGHT_LONG");
static WEIGHT_OFF_FEET: Strs = Strs::new("WEIGHT_OFF_FEET");
static WEIGHT_OLD: Strs = Strs::new("WEIGHT_OLD");
static WEIGHT_OLD_FALL: Strs = Strs::new("WEIGHT_OLD_FALL");
static WENT_BLUE_BREATH_RED: Strs = Strs::new("WENT_BLUE_BREATH_RED");
static WENT_BLUE_FINE: Strs = Strs::new("WENT_BLUE_FINE");
static WENT_BLUE_IDIOM: Strs = Strs::new("WENT_BLUE_IDIOM");
static WENT_BLUE_RED: Strs = Strs::new("WENT_BLUE_RED");
static WHEN_HELP_COMES: Strs = Strs::new("WHEN_HELP_COMES");
static WHEN_HELP_REPLY: Strs = Strs::new("WHEN_HELP_REPLY");
static WHOLE_BODY_RED_FLAG: Strs = Strs::new("WHOLE_BODY_RED_FLAG");
static WISH_BY_MODAL: Strs = Strs::new("WISH_BY_MODAL");
static WISH_BY_MODAL_OFF: Strs = Strs::new("WISH_BY_MODAL_OFF");
static WISH_DEAD_OFF: Strs = Strs::new("WISH_DEAD_OFF");
static WISH_FRAME: Strs = Strs::new("WISH_FRAME");
static WISH_FRAMES: Strs = Strs::new("WISH_FRAMES");
static WISH_INTENT: Strs = Strs::new("WISH_INTENT");
static WISH_NAMED: Strs = Strs::new("WISH_NAMED");
static WISH_OWN_BARE: Strs = Strs::new("WISH_OWN_BARE");
static WITHDRAWAL_ALONE_AT_HOME: Strs = Strs::new("WITHDRAWAL_ALONE_AT_HOME");
static WITHDRAWAL_ASK_DANGER: Strs = Strs::new("WITHDRAWAL_ASK_DANGER");
static WITHDRAWAL_BARE: Strs = Strs::new("WITHDRAWAL_BARE");
static WITHDRAWAL_BARE_BAD: Strs = Strs::new("WITHDRAWAL_BARE_BAD");
static WITHDRAWAL_COLD_TURKEY: Strs = Strs::new("WITHDRAWAL_COLD_TURKEY");
static WITHDRAWAL_DANGER: Strs = Strs::new("WITHDRAWAL_DANGER");
static WITHDRAWAL_DURATION: Strs = Strs::new("WITHDRAWAL_DURATION");
static WITHDRAWAL_FINE: Strs = Strs::new("WITHDRAWAL_FINE");
static WITHDRAWAL_FIT_ITS_OWN_ANSWER: Strs = Strs::new("WITHDRAWAL_FIT_ITS_OWN_ANSWER");
static WITHDRAWAL_HAIR_OF_THE_DOG: Strs = Strs::new("WITHDRAWAL_HAIR_OF_THE_DOG");
static WITHDRAWAL_HALLUCINATION: Strs = Strs::new("WITHDRAWAL_HALLUCINATION");
static WITHDRAWAL_LONG_AGO: Strs = Strs::new("WITHDRAWAL_LONG_AGO");
static WITHDRAWAL_NAMED: Strs = Strs::new("WITHDRAWAL_NAMED");
static WITHDRAWAL_NOT_ALCOHOL: Strs = Strs::new("WITHDRAWAL_NOT_ALCOHOL");
static WITHDRAWAL_NOT_NOW: Strs = Strs::new("WITHDRAWAL_NOT_NOW");
static WITHDRAWAL_ROUTINE: Strs = Strs::new("WITHDRAWAL_ROUTINE");
static WITHDRAWAL_SENSES_FINE: Strs = Strs::new("WITHDRAWAL_SENSES_FINE");
static WITHDRAWAL_SEVERE: Strs = Strs::new("WITHDRAWAL_SEVERE");
static WITHDRAWAL_SOBER: Strs = Strs::new("WITHDRAWAL_SOBER");
static WITHDRAWAL_STOP: Strs = Strs::new("WITHDRAWAL_STOP");
static WITHDRAWAL_STRONG: Strs = Strs::new("WITHDRAWAL_STRONG");
static WITHDRAWAL_SYMPTOM: Strs = Strs::new("WITHDRAWAL_SYMPTOM");
static WITHDRAWAL_WEEKS: Strs = Strs::new("WITHDRAWAL_WEEKS");
static WITHDRAWAL_WORSE: Strs = Strs::new("WITHDRAWAL_WORSE");
static WONT_COME_ROUND: Strs = Strs::new("WONT_COME_ROUND");
static WORLD_QUESTION: Strs = Strs::new("WORLD_QUESTION");
static WOUND_NAMED: Strs = Strs::new("WOUND_NAMED");
static YOUNG_KIN: Strs = Strs::new("YOUNG_KIN");
static _AN_ACT_OR_NOT: Str = Str::new("_AN_ACT_OR_NOT");
static _A_BURNT_PERSON: Str = Str::new("_A_BURNT_PERSON");
static _A_DISEASE_CALLED_A_FEVER: Strs = Strs::new("_A_DISEASE_CALLED_A_FEVER");
static _A_LIVE_HAZARD: Str = Str::new("_A_LIVE_HAZARD");
static _A_WANTS_IT_ON: Str = Str::new("_A_WANTS_IT_ON");
static _BURNING_NOW: Rx = Rx::new("_BURNING_NOW");
static _BURNS_UP: Rx = Rx::new("_BURNS_UP");
static _CHEST_OTHER_RED: Strs = Strs::new("_CHEST_OTHER_RED");
static _CLAUSE_BREAK: Rx = Rx::new("_CLAUSE_BREAK");
static _COME_OFF_IT: Rx = Rx::new("_COME_OFF_IT");
static _DAY_TOTAL_COUNT: Rx = Rx::new("_DAY_TOTAL_COUNT");
static _DENIED: Strs = Strs::new("_DENIED");
static _EFFORT_BEFORE: Rx = Rx::new("_EFFORT_BEFORE");
static _EFFORT_DIED: Rx = Rx::new("_EFFORT_DIED");
static _EFFORT_DOING: Rx = Rx::new("_EFFORT_DOING");
static _FACE_GOING_ODD: Str = Str::new("_FACE_GOING_ODD");
static _FACE_WORD: Str = Str::new("_FACE_WORD");
static _FIRE_WORD: Rx = Rx::new("_FIRE_WORD");
static _FIRST_PERSON_ME: Strs = Strs::new("_FIRST_PERSON_ME");
static _GO_ON_PHRASE: Rx = Rx::new("_GO_ON_PHRASE");
static _HARM_LEFT: Strs = Strs::new("_HARM_LEFT");
static _HIT_ME: Rx = Rx::new("_HIT_ME");
static _LAUGH_KILL: Str = Str::new("_LAUGH_KILL");
static _LEARN_PLACE: Str = Str::new("_LEARN_PLACE");
static _NEAR_WORD: Rx = Rx::new("_NEAR_WORD");
static _NEGATED_BEFORE: Rx = Rx::new("_NEGATED_BEFORE");
static _NOTHING_LEFT_PROBE: Str = Str::new("_NOTHING_LEFT_PROBE");
static _STRUCK_VEHICLE: Str = Str::new("_STRUCK_VEHICLE");
static _SUBJECT_TAIL: Rx = Rx::new("_SUBJECT_TAIL");
static _THE_TEMPERATURE_WORDS: Strs = Strs::new("_THE_TEMPERATURE_WORDS");

// ---------------------------------------------------------------------------------------------
// Character offsets, as Python counts them.

/// The character index of a byte offset.
fn cidx(s: &str, byte: usize) -> usize {
    text::char_index(s, byte)
}

/// `s[start:end]` with character indices (clamped, as Python slices are).
fn cslice(s: &str, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    s.chars().skip(start).take(end - start).collect()
}

// ---------------------------------------------------------------------------------------------
// The helpers, in the Python file's order.

/// Python: bot/modules/safety.py:117 `_one(verb, who)`: "they stop breathing" for one person:
/// "she stops breathing".
pub fn _one(verb: &str, who: &str) -> String {
    let verb = match verb {
        "are" => "is".to_string(),
        "were" => "was".to_string(),
        "have" => "has".to_string(),
        "do" => "does".to_string(),
        _ => {
            let ending = if ["sh", "ch", "s", "x", "o"].iter().any(|end| verb.ends_with(end)) { "es" } else { "s" };
            format!("{verb}{ending}")
        }
    };
    format!("{who} {verb}")
}

/// Python: bot/modules/safety.py:674 `_person(line)`: "she", "he" or None when the line names no
/// single other person.
pub fn _person(line: &str) -> Option<String> {
    // A baby or "my 3 year old" is another person, and only the possessive says which.
    if _matches(&CHILD_ANON, line) {
        let her = _matches(&[r"\bher\b", r"\bshe\b"], line);
        if her && !_matches(&[r"\bhis\b", r"\bi\b", r"\bmy\b", r"\bme\b"], line) {
            return Some("she".to_string());
        }
        if _matches(&[r"\bhis\b"], line) && !her {
            return Some("he".to_string());
        }
    }
    let woman = _matches(&FEMALE_WORDS, line);
    let man = _matches(&MALE_WORDS, line);
    if woman && !man {
        return Some("she".to_string());
    }
    if man && !woman {
        return Some("he".to_string());
    }
    None
}

/// Python: bot/modules/safety.py:748 `_dose_count(word)`: the number a dose is said with.
pub fn _dose_count(word: &str) -> Option<i64> {
    let word = text::squash(&text::lower(word));
    if text::is_digit(&word) {
        return Some(word.parse::<i64>().expect("int(word)"));
    }
    if word.contains("couple") {
        return Some(2);
    }
    if word.contains("dozen") {
        return Some(if word.contains("half") { 6 } else { 12 });
    }
    DOSE_WORDS.get_int(&word.replace('-', " "))
}

/// `_DAY_TOTAL_WORDS.get(count)`, else `int(count)`, else None (a ValueError skips the count).
fn day_total_number(count: &str) -> Option<i64> {
    match _DAY_TOTAL_WORDS.get_int(count) {
        Some(n) => Some(n),
        None => int_of(count),
    }
}

/// Python's `int(text)` on a string: optional sign, digits, surrounding whitespace; None for a
/// ValueError.
fn int_of(s: &str) -> Option<i64> {
    text::strip(s).parse::<i64>().ok()
}

/// Python: bot/modules/safety.py:759 `_day_dose_reply(text)`: the day's limit, said against the
/// count the line gives.
pub fn _day_dose_reply(said: &str) -> String {
    let (mut number, mut drug): (Option<i64>, String) = (None, String::new());
    for found in _DAY_TOTAL_COUNT.finditer(said) {
        let (count, word) = (found.str(1), found.str(2));
        let Some(n) = day_total_number(count) else { continue };
        number = Some(n);
        drug = word.to_string();
    }
    let ibuprofen = ["ibuprofen", "nurofen", "brufen"].contains(&drug.as_str());
    let (limit, spelling) = if ibuprofen { (6, "six of the 200mg tablets a day") } else { (8, "eight 500mg paracetamol tablets a day") };
    let tail = if ibuprofen {
        "Take it with food rather than on an empty stomach, and nothing else that contains it. Needing that many for more than a few days is the back to get looked at."
    } else {
        "Four hours between doses, and count anything from a cold remedy as paracetamol too. Running at the full eight for more than a few days means the pain needs looking at rather than more tablets."
    };
    if let Some(number) = number {
        if number < limit {
            return format!("{number} is inside the day's limit of {spelling}. {tail}");
        }
    }
    format!("The day's limit is {spelling}, so there is nothing more to take. {tail}")
}

/// Python: bot/modules/safety.py:785 `_stated_dose(line)`: the reply to a dose the person says
/// they took that is inside the limit, or None.
pub fn _stated_dose(line: &str) -> Option<String> {
    let said = text::squash(&text::lower(line));
    if found(
        r"\bon\s+purpose\b|\b(?:to|trying\s+to)\s+(?:hurt|harm|kill)\s+myself\b|\bsuicid|\bwanted\s+to\s+die\b|\bkill\s+myself\b|\bend\s+it\b",
        &said,
    ) {
        return None;
    }
    if _matches(&[&*CHILD_TAKER, &*CHILD_WORD], line) || _matches(&PET_OWN, line) || _matches(&PET_SUBJECT, line) {
        return None;
    }
    for pattern in DOSE_STATED.iter() {
        let Some(hit) = search(pattern, &said) else { continue };
        let n = _dose_count(hit.str(1));
        let drug = text::squash(&text::lower(hit.get(2).expect("group 2")).replace('-', " "));
        let limit = DOSE_LIMITS.get_int(&drug);
        if let (Some(n), Some(limit)) = (n, limit) {
            if n != 0 && limit != 0 && n > limit {
                return None;
            }
        }
        break;
    }
    if _a_days_dose(line) {
        return Some(_day_dose_reply(&said));
    }
    for pattern in DOSE_DOUBLE.iter() {
        if found(pattern, &said) {
            return Some("One extra dose is usually nothing to worry about: don't take any more for four hours, and skip that one if it lands in the night. If it was more than one extra, or you have liver trouble or drink heavily, ring 111 rather than guessing - and never make up a missed dose by doubling the next one.".to_string());
        }
    }
    for pattern in DOSE_RECENT.iter() {
        if found(pattern, &said) {
            return Some("That is done and inside the limit, so the only thing to keep is the gap: four hours before the next dose, and eight tablets of 500mg in a day at most. If you are still in pain at four hours, that is worth getting looked at rather than topping up early.".to_string());
        }
    }
    None
}

/// Python: bot/modules/safety.py:850 `_fit(reply, line, case=None)`: the reply with its
/// they/them/their pointing at the person the user named. (`case` is given by every caller of
/// this contract; [`_fit_case`] is the Python call with `case` left out.)
pub fn _fit(reply: &str, line: &str, case: &str) -> String {
    _fit_case(reply, line, Some(case))
}

/// `_fit(reply, line, case)` with `case` optional, as Python has it.
pub fn _fit_case(reply: &str, line: &str, case: Option<&str>) -> String {
    if reply.is_empty() {
        return reply.to_string();
    }
    if let Some(case) = case {
        if !FIT_CASES.iter().any(|one| one == case) {
            return reply.to_string();
        }
    }
    let who = _person(line);
    let pairs: &[(String, String)] = match who.as_deref() {
        Some("she") => &SHE,
        Some("he") => &HE,
        _ => return reply.to_string(),
    };
    if pairs.is_empty() {
        return reply.to_string();
    }
    let mut said = reply.to_string();
    for (pattern, replacement) in pairs {
        said = sub_with(pattern, replacement, &said, pyre::I | pyre::U);
    }
    said
}

/// Python: bot/modules/safety.py:887 `_letters_apart(word, target)`: single-letter edits between
/// two words (Levenshtein).
pub fn _letters_apart(word: &str, target: &str) -> i64 {
    let target: Vec<char> = target.chars().collect();
    let mut previous: Vec<i64> = (0..=target.len() as i64).collect();
    for (i, letter) in word.chars().enumerate() {
        let mut current = vec![i as i64 + 1];
        for (j, other) in target.iter().enumerate() {
            let j = j + 1;
            let value = (previous[j] + 1).min(current[j - 1] + 1).min(previous[j - 1] + (letter != *other) as i64);
            current.push(value);
        }
        previous = current;
    }
    *previous.last().expect("never empty")
}

/// Python: bot/modules/safety.py:899 `_near_miss(word, target)`: the same shape as `target` and
/// one or two letters out of it.
pub fn _near_miss(word: &str, target: &str) -> bool {
    if (text::len(word) as i64 - text::len(target) as i64).abs() > 2 {
        return false;
    }
    if text::slice(word, 0, Some(2)) != text::slice(target, 0, Some(2)) || word.chars().last() != target.chars().last() {
        return false;
    }
    _letters_apart(word, target) <= 2
}

/// Python: bot/modules/safety.py:908 `_leet_made_whole(text)`: a letter swapped for the character
/// that looks like it, inside a word ("k!ll", "k1ll").
pub fn _leet_made_whole(said: &str) -> String {
    sub_with(r"(?<=[a-z])[!1|](?=[a-z])", "i", said, pyre::U)
}

/// Python: bot/modules/safety.py:919 `_fold_critical(line)`: the emergency words put back to
/// their own spelling before anything is matched.
pub fn _fold_critical(line: &str) -> String {
    let mut words = text::split(line);
    for i in 0..words.len() {
        let word = words[i].clone();
        if let Some(typo) = TYPED_TYPOS.get_str(&word) {
            words[i] = typo.to_string();
            continue;
        }
        if CRITICAL.iter().any(|one| *one == word) || CRITICAL_ONE.iter().any(|one| *one == word) || LOOKS_REAL.contains(&word) {
            continue;
        }
        // "sezures" is the plural of the misspelling, so the ending is tried both ways.
        let mut tries = vec![(word.clone(), "")];
        if word.ends_with('s') && text::len(&word) > 4 {
            tries.push((text::slice(&word, 0, Some(-1)), "s"));
        }
        for (stem, plural) in tries {
            if LOOKS_REAL.contains(&stem) {
                break;
            }
            let mut hit: Vec<&String> = CRITICAL.iter().filter(|target| _near_miss(&stem, target)).collect();
            if hit.is_empty() {
                hit = CRITICAL_ONE.iter().filter(|target| _near_miss(&stem, target) && _letters_apart(&stem, target) == 1).collect();
            }
            if let Some(target) = hit.first() {
                words[i] = format!("{target}{plural}");
                break;
            }
        }
    }
    words.join(" ")
}

/// Python: bot/modules/safety.py:945 `_norm(text)`: lower case, standard apostrophes, a glued
/// opener split, chat-speak and contractions folded, the emergency words respelt; padded with a
/// space at each end.
pub fn _norm(said: &str) -> String {
    let line = text::lower(said).replace('\u{2019}', "'").replace('\u{2018}', "'");
    let line = sub_with(r"\bu\b", "you", &line, pyre::U);
    let line = sub_with(r"\bpls\b", "please", &line, pyre::U);
    let words = text::split(&line);
    let mut split: Vec<String> = Vec::new();
    for (index, word) in words.iter().enumerate() {
        let after = words.get(index + 1).map(String::as_str);
        match crate::understanding::normalize::glued_words(word, after).filter(|glued| !glued.is_empty()) {
            Some(glued) => split.extend(glued),
            None => split.push(word.clone()),
        }
    }
    let mut line = split.join(" ");
    for (short, long) in CONTRACTIONS.iter() {
        if line.contains(short.as_str()) {
            line = sub_with(&format!(r"\b{}\b", pyre::escape(short)), long, &line, pyre::U);
        }
    }
    format!(" {} ", _fold_critical(&text::squash(&line)))
}

/// Python: bot/modules/safety.py:1160 `_ends_it_with_a_method(line)`: "end it with" a thing that
/// is a means, not a person or a break-up.
pub fn _ends_it_with_a_method(line: &str) -> bool {
    let Some(hit) = search(r"\bend(?:ing)?\s+(?:it|(?:his|her|their)\s+(?:own\s+)?life)\s+with\s+(.*)$", line) else {
        return false;
    };
    let tail = hit.str(1);
    let Some(pronoun) = compiled(r"(?:me|him|her|them|us)\b").match_(tail) else {
        return compiled(&format!(r"{}\b", &*END_IT_WITH_PERSON)).match_(tail).is_none();
    };
    let rest = text::strip(&tail[pronoun.end()..]);
    if rest.is_empty()
        || compiled(&END_IT_WITH_CONNECTOR).match_(rest).is_some()
        || compiled(&format!(r"{}\b", &*END_IT_WITH_NOUN)).match_(rest).is_some()
    {
        return false;
    }
    true
}

/// Python: bot/modules/safety.py:1538 `_ending_a_thing(line)`: ending a thing, not a life.
pub fn _ending_a_thing(line: &str) -> bool {
    if _matches(&END_ALL, line) {
        return false;
    }
    _matches(&END_THING, line) && !_matches(&DEATH_WORDS, line)
}

/// Python: bot/modules/safety.py:1561 `_avoided(line, patterns)`: every pattern that matches is
/// one the line says it is avoiding (`AVOID_BEFORE` within forty characters before it).
pub fn _avoided(line: &str, patterns: &[String]) -> bool {
    let hits: Vec<usize> = patterns.iter().filter_map(|pattern| search(pattern, line).map(|hit| cidx(line, hit.start()))).collect();
    !hits.is_empty() && hits.iter().all(|&at| AVOID_BEFORE.is_match(&cslice(line, at.saturating_sub(40), at)))
}

/// Python: bot/modules/safety.py:1567 `_deserve(line)`: "i deserve to ..." not denied in the
/// forty-two characters before it.
pub fn _deserve(line: &str) -> bool {
    let hits: Vec<usize> = SELF_HARM_DESERVE.iter().filter_map(|pattern| search(pattern, line).map(|hit| cidx(line, hit.start()))).collect();
    if hits.is_empty() {
        return false;
    }
    hits.iter().any(|&at| !DESERVE_DENIED.is_match(&cslice(line, at.saturating_sub(42), at)))
}

/// Python: bot/modules/safety.py:1660 `_habit_explained(line)`: a self-harm habit word the line
/// explains as an accident, with no wish behind it.
pub fn _habit_explained(line: &str) -> bool {
    if _deserve(line) || !_matches(&SELF_HARM_HABIT, line) {
        return false;
    }
    if _matches(&SELF_HARM_DELIBERATE, line) {
        return false;
    }
    let rest: Vec<String> = SELF_HARM_INTENT.iter().filter(|pattern| !SELF_HARM_HABIT.contains(pattern)).cloned().collect();
    if _matches(&rest, line) || _hopeless(line) || _matches(&WISH_INTENT, line) {
        return false;
    }
    _matches(&ACCIDENT, line)
}

/// Python: bot/modules/safety.py:1680 `_a_pet_hurting_itself(line)`
pub fn _a_pet_hurting_itself(line: &str) -> bool {
    if !_matches(&PET_HURTING_ITSELF, line) {
        return false;
    }
    if _matches(&HUMAN_TOO, line) || _matches(&THIRD_PERSON_ACTOR, line) {
        return false;
    }
    !_matches(&FIRST_PERSON_WISH, line) && !_own_wish(line)
}

/// Python: bot/modules/safety.py:1694 `_a_wish_that_is_theirs(line)`: somebody else's wish, not
/// the speaker's.
pub fn _a_wish_that_is_theirs(line: &str) -> bool {
    if !_matches(&THIRD_PERSON_WISH, line) {
        return false;
    }
    if _ends_it_with_a_method(line) {
        return false;
    }
    if _matches(&FIRST_PERSON_WISH, line) || _own_wish(line) || _matches(&SO_I_TOO, line) {
        return false;
    }
    true
}

/// Python: bot/modules/safety.py:1711 `_a_pet_wants_out(line)`
pub fn _a_pet_wants_out(line: &str) -> bool {
    if !_matches(&PET_WANTS_OUT, line) {
        return false;
    }
    if _matches(&HUMAN_TOO, line) || _matches(&THIRD_PERSON_ACTOR, line) {
        return false;
    }
    if _matches(&SO_I_TOO, line) {
        return false;
    }
    !_matches(&FIRST_PERSON_WISH, line) && !_own_wish(line)
}

/// Python: bot/modules/safety.py:1730 `_self_harm_intent(line)`: the speaker's own wish to hurt or
/// end themselves.
pub fn _self_harm_intent(line: &str) -> bool {
    if _a_pet_hurting_itself(line) {
        return false;
    }
    if _a_pet_wants_out(line) {
        return false;
    }
    if _a_wish_that_is_theirs(line) {
        return false;
    }
    if _habit_explained(line) {
        return false;
    }
    if _an_ordinary_helping(line) {
        return false;
    }
    if _a_real_injury_beside_the_plate(line) {
        return false;
    }
    if _matches(&HYPERBOLE_WISH, line) && !_matches(&WISH_INTENT, line) && !_matches(&HYPERBOLE_BLOCK, line) {
        return false;
    }
    if _deserve(line) || _matches(&SELF_HARM_INTENT, line) || _hopeless(line) || _matches(&WISH_INTENT, line) || _matches(&OWN_LIFE, line) {
        if _ending_a_thing(line) {
            return false;
        }
        if _matches(&GARDEN_WORDS, line) && !_matches(&GO_ON_LIVING, line) {
            let rest = _GO_ON_PHRASE.sub(" ", line);
            if !(_own_wish(&rest) || _hopeless(&rest) || _matches(&SELF_HARM, &rest) || _matches(&DESPAIR, &rest)) {
                return false;
            }
        }
        if found(r"\b(?:not|don'?t|doesn'?t|didn'?t|never|shouldn'?t|can'?t)\b[^.?!]{0,14}\bdeserve", line) {
            return false;
        }
        return true;
    }
    if _matches(&SELF_HARM_REPEAT_CAUSE, line) {
        // a cause on the line: not the repeat
    } else if _matches(&SELF_HARM_REPEAT_CUT, line) {
        return true;
    } else if _matches(&SELF_HARM_REPEAT_GRIND, line) && _matches(&SELF_HARM_REPEAT_MEANT, line) {
        return true;
    }
    _matches(&SELF_HARM_DELIBERATE, line)
        && _matches(&SELF_HARM_DELIBERATE_ACT, line)
        && (!_matches(&SELF_HARM_DENIED, line) || _matches(&SELF_HARM_NOT_EATING, line))
}

/// Python: bot/modules/safety.py:1815 `_someone_elses_despair(line)`: somebody else's despair,
/// reported, with no word of the speaker's own.
pub fn _someone_elses_despair(line: &str) -> bool {
    if found(r"\b(?:i|im|i'm|ive|i've|id|i'd|myself|my\s+own)\b", line) {
        return false;
    }
    if _matches(&OWN_WISH, line) || _matches(&FIRST_PERSON_WISH, line) {
        return false;
    }
    let phrase = format!("(?:{})", SOMEONE_ELSES_DESPAIR.join("|"));
    let person = format!("(?:{})", SOMEONE_AS_SUBJECT.join("|"));
    let report = format!("(?:{})", SOMEONE_REPORTED.join("|"));
    if found(&format!(r"{person}[^.?!]{{0,30}}{report}(?:\s+(?:that|she|he|they)\b)?[^.?!]{{0,20}}{phrase}"), line) {
        return true;
    }
    found(&format!(r"{person}\s+(?:(?!{})(?:\w+\s+)){{0,3}}{phrase}", &*NOT_THEIR_OWN), line)
}

/// Python: bot/modules/safety.py:1917 `_hopeless(line)`: one of the HOPELESS wishes, with the
/// negated giving up and a group that gave up left out.
pub fn _hopeless(line: &str) -> bool {
    if !_matches(&HOPELESS, line) {
        return false;
    }
    if _matches(&GIVING_UP_GROUP, line) && !_matches(&FIRST_PERSON_WISH, line) && !_own_wish(line) {
        return false;
    }
    !_matches(&GIVING_UP_HELD_ON, line)
}

/// Python: bot/modules/safety.py:1963 `_a_wish_for_their_own_body(line)`: a wish about a part of
/// their own body that is not a workout.
pub fn _a_wish_for_their_own_body(line: &str) -> bool {
    if !_matches(&SELF_HARM_OWN_PART, line) {
        return false;
    }
    !(found(&A_HURT_OF_A_BODY_PART, line) && found(&A_WORKOUT_CUE, line))
}

/// Python: bot/modules/safety.py:1977 `_idiom(line)`: "i could die", "killing myself laughing":
/// a figure of speech, unless a wish frame or the rest of the line rings.
pub fn _idiom(line: &str) -> bool {
    if _the_effort_frame(line) {
        return true;
    }
    if !_matches(&IDIOM, line) {
        return false;
    }
    if _matches(&PLAN_FRAME_LAUGH, line) {
        return false;
    }
    if found(r"\bdie\s+(?:laughing|of\s+(?:embarrassment|boredom|shame))\b", line) {
        return true;
    }
    !_matches(&WISH_FRAME, line) && !_rest_of_it_rings(line)
}

/// Python: bot/modules/safety.py:2013 `_cut_off_idiom(line)`: "i cut myself off from everyone",
/// with nothing else in the line.
pub fn _cut_off_idiom(line: &str) -> bool {
    if !_matches(&CUT_MYSELF_OFF, line) {
        return false;
    }
    let mut rest = line.to_string();
    for pattern in CUT_MYSELF_OFF.iter() {
        rest = sub_with(pattern, " ", &rest, pyre::U);
    }
    if _matches(&CRISIS_PLAN_SAID, &rest) || CRISIS_WORD.is_match(&rest) {
        return false;
    }
    if _matches(&ABUSE, line) || _matches(&ABUSE_HIT_ME, line) {
        return false;
    }
    !_self_harm_intent(&rest) && !_matches(&_HARM_LEFT, &rest)
}

/// Python: bot/modules/safety.py:2032 `_denied_wish(line)`: the longest denial of a wish, with no
/// "but" and no harm word left beside it.
pub fn _denied_wish(line: &str) -> bool {
    let mut best: Option<(usize, usize)> = None;
    let length = |span: (usize, usize)| line[span.0..span.1].chars().count();
    for pattern in _DENIED.iter() {
        for hit in compiled(pattern).finditer(line) {
            let span = (hit.start(), hit.end());
            if best.map(|b| length(span) > length(b)).unwrap_or(true) {
                best = Some(span);
            }
        }
    }
    let Some((start, end)) = best else { return false };
    let rest = format!("{} {}", &line[..start], &line[end..]);
    if found(r"\b(?:but|though|although|however|yet|except)\b", &rest) {
        return false;
    }
    !found(&_HARM_LEFT.join("|"), &rest)
}

/// Python: bot/modules/safety.py:2047 `_blocked(line)`
pub fn _blocked(line: &str) -> bool {
    if _matches(&WISH_FRAME, line) {
        return false;
    }
    _matches(&FIGURATIVE, line) || _matches(&NOT_NOW, line) || _matches(&MILD, line)
}

/// Python: bot/modules/safety.py:2067 `_a_days_dose(line)`: a count of tablets over a day that is
/// inside the day's limit, said by the person taking them.
pub fn _a_days_dose(line: &str) -> bool {
    if !_matches(&DAY_TOTAL_WORD, line) || _matches(&DAY_TOTAL_SIGN, line) {
        return false;
    }
    if _matches(&[&*CHILD_TAKER, &*CHILD_WORD], line) || _matches(&PET_OWN, line) || _matches(&PET_SUBJECT, line) {
        return false;
    }
    for hit in _DAY_TOTAL_COUNT.finditer(line) {
        let (count, drug) = (hit.str(1), hit.str(2));
        let Some(number) = day_total_number(count) else { continue };
        if number <= DAY_TOTAL_MAX.get_int(drug).unwrap_or(8) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/safety.py:2110 `_the_person_is_in_it(line)`
pub fn _the_person_is_in_it(line: &str) -> bool {
    if _matches(&SAID_AS_IT_HAPPENS, line) || _matches(&OWN_CRY, line) {
        return true;
    }
    _matches(&CHEST_URGENT, line)
}

/// Python: bot/modules/safety.py:2120 `_asking_about_the_world(line)`: a question about how the
/// world works, not a person in it.
pub fn _asking_about_the_world(line: &str) -> bool {
    if text::rstrip(line).ends_with('?') && _matches(&WORLD_QUESTION, line) {
        return true;
    }
    if _the_person_is_in_it(line) {
        return false;
    }
    (_matches(&WORLD_QUESTION, line) && _matches(&QUESTION_OPENER, line)) || _matches(&ASKING_WHY_IT_HURTS, line)
}

/// Python: bot/modules/safety.py:2440 `_nothing_left_clause()`: the DESPAIR clause that reads
/// "i have nothing left" (the last one that does), or the last clause.
pub fn _nothing_left_clause() -> Vec<String> {
    for clause in DESPAIR.iter().rev() {
        let rx = Re::new(clause, pyre::I | pyre::U);
        if rx.check().is_err() {
            continue; // re.error
        }
        if rx.is_match(&_NOTHING_LEFT_PROBE) {
            return vec![clause.clone()];
        }
    }
    DESPAIR.last().cloned().into_iter().collect()
}

/// Python: bot/modules/safety.py:2582 `_a_small_child_did_it(line, memory)`: the blow was a small
/// child's, said in this line or the last few.
pub fn _a_small_child_did_it(line: &str, memory: &Memory) -> bool {
    if !A_SMALL_CHILD_LASHES_OUT.is_match(line) {
        return false;
    }
    if A_GROWN_AGE.is_match(line) {
        return false;
    }
    if A_SMALL_CHILD_SAID.is_match(line) {
        return true;
    }
    if A_GROWN_BLOW.is_match(line) {
        return false;
    }
    if !A_BARE_ACTOR_LASH.is_match(line) {
        return false;
    }
    let mut said = String::new();
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        if !turn.user.is_empty() {
            said.push(' ');
            said.push_str(&_norm(&turn.user));
        }
        if !turn.bot.is_empty() && SMALL_CHILD_REPLY_GIVEN.is_match(&_norm(&turn.bot)) {
            said.push(' ');
            said.push_str(&_norm(&turn.bot));
        }
    }
    A_SMALL_CHILD_SAID.is_match(&said)
}

/// Python: bot/modules/safety.py:2650 `_hit_by_a_loss(line)`: every "it hit me" in the line has a
/// loss as its subject ("losing my job hit me hard").
pub fn _hit_by_a_loss(line: &str) -> bool {
    let clauses = _HIT_ME.finditer(line);
    if clauses.is_empty() {
        return false;
    }
    for hit in clauses {
        let at = cidx(line, hit.start());
        let before = cslice(line, at.saturating_sub(80), at);
        let window = _SUBJECT_TAIL.sub("", &before);
        let window = text::slice(&window, -60, None);
        if !_matches(&LOSS_SUBJECT_AT_END, &window) {
            return false;
        }
    }
    true
}

/// Python: bot/modules/safety.py:2961 `_overdose_of_a_treat(line)`
pub fn _overdose_of_a_treat(line: &str) -> bool {
    (_matches(&OVERDOSE_OF_A_TREAT, line) || _matches(&OVERDOSE_WORK, line)) && !_matches(&OVERDOSE_REAL_THING, line)
}

/// Python: bot/modules/safety.py:4165 `_glass_test_line(line, before)`: the glass test said over a
/// rash named a turn before.
pub fn _glass_test_line(line: &str, before: &str) -> bool {
    if _matches(&FADE_EVERYDAY, line) {
        return false;
    }
    if !(_matches(&RASH_FADE, line) && _matches(&[&*RASH], before)) {
        return false;
    }
    _matches(&RASH_COMPANY, line) || _matches(&RASH_COMPANY, before)
}

/// Python: bot/modules/safety.py:4178 `_said_before(memory, turns=2)`: the person's last lines,
/// normalised, newest first.
pub fn _said_before(memory: &Memory, turns: i64) -> String {
    let all = &memory.turns;
    // `turns[-n:]` (a zero or negative n takes the whole list, as Python's slice does).
    let from = if turns > 0 { all.len().saturating_sub(turns as usize) } else { 0 };
    let said: Vec<String> = all[from..].iter().rev().filter(|turn| !turn.user.is_empty()).map(|turn| _norm(&turn.user)).collect();
    said.join(" ")
}

/// Python: bot/modules/safety.py:4192 `_person_before(memory)`: the newest of the person's last
/// three lines, normalised.
pub fn _person_before(memory: &Memory) -> String {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        if !turn.user.is_empty() {
            return _norm(&turn.user);
        }
    }
    String::new()
}

/// Python: bot/modules/safety.py:4230 `_own_wish(line)`
pub fn _own_wish(line: &str) -> bool {
    _matches(&WISH_OWN_BARE, line) || (_matches(&[r"\bmyself\b"], line) && !_matches(&SELF_TALK, line))
}

/// Python: bot/modules/safety.py:5104 `_a_reason_it_was_not_meant(text)`: a cause on the line
/// that makes a hurt an accident.
pub fn _a_reason_it_was_not_meant(said: &str) -> bool {
    (_matches(&SELF_HARM_LOOKALIKE, said) && !_matches(&A_WISH_OR_TALK_OF_IT, said))
        || (_matches(&AN_ITCH_LOOKALIKE, said) && !_matches(&A_WISH_OR_TALK_OF_IT, said) && !_matches(&AN_ACT_BESIDE_THE_ITCH, said))
        || _an_ordinary_helping(said)
}

/// `_matches(BLEEDING + CUT_NOT_STOPPING, text)`.
fn bleeding_or_not_stopping(said: &str) -> bool {
    _matches(&BLEEDING, said) || _matches(&CUT_NOT_STOPPING, said)
}

/// Python: bot/modules/safety.py:5140 `_a_real_injury_beside_the_plate(text)`
pub fn _a_real_injury_beside_the_plate(said: &str) -> bool {
    _matches(&A_REAL_INJURY, said)
        && _matches(&A_PLATE_SHAPE, said)
        && !_matches(&A_WISH_OR_TALK_OF_IT, said)
        && !_matches(&DISTRESS_BESIDE_IT, said)
        && !bleeding_or_not_stopping(said)
}

/// Python: bot/modules/safety.py:5162 `_an_ordinary_helping(text)`: "i cut myself a slice of
/// cake": the thing after the pronoun is the helping, not the act.
pub fn _an_ordinary_helping(said: &str) -> bool {
    _matches(&ORDINARY_REFLEXIVE, said)
        && !_matches(&A_WISH_OR_TALK_OF_IT, said)
        && !_matches(&DISTRESS_BESIDE_IT, said)
        && !(_matches(&A_BODY_PART, said) && _matches(&A_HARM_VERB_CUT, said))
        && !(_matches(&A_REAL_INJURY, said) && !bleeding_or_not_stopping(said))
        && !_matches(&UNWELL_BESIDE_THE_PLATE, said)
        && !bleeding_or_not_stopping(said)
}

/// Python: bot/modules/safety.py:5329 `_rest_of_it(line)`: the line with the laugh, the effort
/// and the figures of speech taken out.
pub fn _rest_of_it(line: &str) -> String {
    let mut rest = sub_with(&_LAUGH_KILL, " ", line, pyre::U);
    rest = EFFORT_AFTER.sub(" ", &rest);
    for pattern in DIE_OF_THE_FEELING.iter() {
        rest = sub_with(pattern, " ", &rest, pyre::U);
    }
    for pattern in PAST_WISH_NOT.iter() {
        rest = sub_with(pattern, " ", &rest, pyre::U);
    }
    sub_with(r"\b(?:feel|feels|feeling|felt)\s+like\b", " ", &rest, pyre::U)
}

/// Python: bot/modules/safety.py:5348 `_rings(text)`: a wish, an intent or despair in the text.
pub fn _rings(said: &str) -> bool {
    _matches(&WISH_NAMED, said)
        || _matches(&WISH_INTENT, said)
        || _matches(&SELF_HARM_INTENT, said)
        || _matches(&FIRST_PERSON_WISH, said)
        || _matches(&DESPAIR, said)
        || _matches(&NOT_JUST_A_NEAR_THING, said)
}

/// Python: bot/modules/safety.py:5354 `_rest_of_it_rings(line)`
pub fn _rest_of_it_rings(line: &str) -> bool {
    let rest = _rest_of_it(line);
    if text::squash(&rest) == text::squash(line) {
        return false;
    }
    _rings(&rest)
}

/// Python: bot/modules/safety.py:5555 `_the_laugh(line)`: "killing myself laughing", as the joke
/// and nothing else.
pub fn _the_laugh(line: &str) -> bool {
    if _matches(&PLAN_FRAME_LAUGH, line) || _rest_of_it_rings(line) {
        return false;
    }
    if BARE_LAUGH.match_(text::strip(line)).is_some() {
        return false;
    }
    _idiom(line) && search(&_LAUGH_KILL, line).is_some()
}

/// Python: bot/modules/safety.py:5569 `_an_accident_near_thing(line)`: a near thing with a heavy
/// object or on a bike, with no wish in it.
pub fn _an_accident_near_thing(line: &str) -> bool {
    let doing = _EFFORT_DOING.search(line);
    // `doing.group("doing")` is read only when the effort word in front is not there.
    let doing_counts = || match &doing {
        Some(hit) => {
            let what = hit.name("doing").expect("the doing group");
            what != "laughing" && HARM_DOING.match_(what).is_none()
        }
        None => false,
    };
    if !(_EFFORT_BEFORE.is_match(line) || doing_counts()) {
        if !_EFFORT_DIED.is_match(line) || !_COME_OFF_IT.is_match(line) {
            return false;
        }
    }
    !_matches(&EFFORT_NOT_AN_ACCIDENT, line) && !_matches(&EFFORT_MEANS, line) && !_rings(line) && !_rest_of_it_rings(line)
}

/// Python: bot/modules/safety.py:5585 `_near_thing_key(line)`
pub fn _near_thing_key(line: &str) -> String {
    if _person(line).is_some() && !_matches(&[r"\bi\b", r"\bwe\b"], line) { "near_thing_other" } else { "near_thing" }.to_string()
}

/// Python: bot/modules/safety.py:5591 `_the_effort_frame(line)`: "nearly killed myself carrying
/// the sofa": effort, not a wish.
pub fn _the_effort_frame(line: &str) -> bool {
    if _an_accident_near_thing(line) {
        return true;
    }
    let Some(hit) = EFFORT_AFTER.search(line) else { return false };
    let doing = hit.name("doing");
    if let Some(what) = doing {
        if !what.is_empty() && HARM_DOING.match_(what).is_some() {
            return false;
        }
    }
    if doing == Some("laughing") {
        return false;
    }
    if _matches(&EFFORT_MEANS, line) {
        return false;
    }
    !_rings(line) && !_rest_of_it_rings(line)
}

/// Python: bot/modules/safety.py:5741 `_without_object_limbs(line)`: the line with the limbs that
/// are a thing's (a chair leg, a doll's arm) taken out.
pub fn _without_object_limbs(line: &str) -> String {
    let mut line = line.to_string();
    for pattern in LIMB_IS_A_THING.iter() {
        if pattern.contains("bent") {
            continue;
        }
        line = sub_with(pattern, " ", &line, pyre::I | pyre::U);
    }
    line
}

/// Python: bot/modules/safety.py:5821 `_any_of(*groups)`: one alternation of every pattern in the
/// groups.
pub fn _any_of(groups: &[&[String]]) -> String {
    let parts: Vec<String> = groups.iter().flat_map(|group| group.iter()).map(|part| format!("(?:{part})")).collect();
    if parts.len() > 1 {
        format!("(?:{})", parts.join("|"))
    } else {
        parts.first().cloned().expect("IndexError: _any_of of nothing")
    }
}

/// Python: bot/modules/safety.py:5827 `_both(first, *rest)`: one pattern that needs a match of
/// `first` and of `rest`, in either order.
pub fn _both(first: &[String], rest: &[&[String]]) -> Vec<String> {
    vec![format!("(?=.*{})(?=.*{})", _any_of(&[first]), _any_of(rest))]
}

/// Python: bot/modules/safety.py:6166 `_the_only_fever_is_a_disease_name(line)`: "scarlet fever"
/// and no temperature beside it.
pub fn _the_only_fever_is_a_disease_name(line: &str) -> bool {
    if !_matches(&_A_DISEASE_CALLED_A_FEVER, line) {
        return false;
    }
    let mut stripped = line.to_string();
    for pattern in _A_DISEASE_CALLED_A_FEVER.iter() {
        stripped = sub_with(pattern, " ", &stripped, pyre::U);
    }
    !_matches(&_THE_TEMPERATURE_WORDS, &stripped)
}

/// Python: bot/modules/safety.py:6689 `_for_the_person(reply, line, case)`: a pregnancy card
/// turned to the person the line is about ("my wife is pregnant and bleeding").
pub fn _for_the_person(reply: &str, line: &str, case: &str) -> String {
    if reply.is_empty() || case.is_empty() || !case.starts_with("pregnancy_") {
        return reply.to_string();
    }
    if !_matches(&EXPECTING_OTHER, line) {
        return reply.to_string();
    }
    let she = _person(line).as_deref() != Some("he");
    let (obj, subject) = if she { ("her", "she") } else { ("him", "he") };
    let ci = pyre::I | pyre::U;
    let mut said = sub_with(r"\byou're\b", &format!("{subject}'s"), reply, ci);
    said = sub_with(r"\byou've\b", &format!("{subject}'s"), &said, ci);
    said = sub_with(r"\byour\b", obj, &said, ci);
    for verb in ["stops", "stop", "leave", "leaves", "get", "gets", "keep", "keeps", "make", "makes", "take", "takes", "tell", "tells", "with", "for", "to"] {
        said = sub_with(&format!(r"\b{verb}\s+you\b"), &format!("{verb} {obj}"), &said, ci);
    }
    said = sub_with(r"\bthat\s+needs\b", &format!("{subject} needs"), &said, ci);
    said = sub_with(r"\byou\b", subject, &said, ci);
    let third: Vec<String> = THIRD_KEYS.iter().map(|key| pyre::escape(key)).collect();
    let rx = compiled_with(&format!(r"\b(she|he)\s+((?:also|still|just|only)\s+)?({})\b", third.join("|")), ci);
    said = rx.sub_fn(&said, |m| {
        let verb = text::lower(m.str(3));
        format!("{} {}{}", m.str(1), m.str(2), _THIRD.get_str(&verb).unwrap_or_else(|| panic!("KeyError: {verb}")))
    });
    if compiled(r"(?i)^ring\b").match_(&said).is_some() {
        said = format!("Tell {obj} to ring {}", text::lstrip(&text::slice(&said, 4, None)));
    }
    let mut chars = said.chars();
    match chars.next() {
        Some(head) => format!("{}{}", head.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

/// `_THIRD`'s keys, in the dict's order (the alternation `_for_the_person` builds).
static THIRD_KEYS: LazyLock<Vec<String>> = LazyLock::new(|| b("_THIRD").pairs().iter().map(|(k, _)| k.str().to_string()).collect());

/// Python: bot/modules/safety.py:7527 `_fainted_with_the_bang(line)`: the bang came with a faint
/// or a blackout of sight.
pub fn _fainted_with_the_bang(line: &str) -> bool {
    if _matches(&FAINT_WITH_A_BANG, line) {
        return true;
    }
    BLACKED_OUT_SIGHT.iter().any(|shape| found(shape, line))
}

/// Python: bot/modules/safety.py:7564 `_effort_recovery(line)`: out of breath after a class, a
/// match or a run, and getting it back.
pub fn _effort_recovery(line: &str) -> bool {
    (_matches(&EXERTION_CLASS, line) || _matches(&BREATH_EFFORT, line) || _matches(&EXERTION, line))
        && _matches(&EFFORT_RECOVERY, line)
        && !(_matches(&BREATH_RED, line) || _matches(&EFFORT_MORE_RED, line))
        && !(_matches(&NOT_A_BREATHER, line) || _matches(&PET_WORDS, line))
        && !_matches(&EFFORT_TOO_LONG, line)
        && !_matches(&EFFORT_SOMEONE_ELSE, line)
        && !_matches(&[r"\bstill\b", r"\bright\s+now\b"], line)
}

/// Python: bot/modules/safety.py:7654 `_hallucinating(line)`
pub fn _hallucinating(line: &str) -> bool {
    if !_matches(&WITHDRAWAL_HALLUCINATION, line) {
        return false;
    }
    !_matches(&WITHDRAWAL_SENSES_FINE, line)
}

/// Python: bot/modules/safety.py:7661 `_severe(line)`
pub fn _severe(line: &str) -> bool {
    _matches(&WITHDRAWAL_SEVERE, line) || _hallucinating(line)
}

/// Python: bot/modules/safety.py:7921 `_ring_stuck(line)`
pub fn _ring_stuck(line: &str) -> bool {
    _matches(&RING_WORD, line) && _matches(&FINGER_SIDE, line) && _matches(&RING_DANGER, line) && !_matches(&RING_NOT_WORN, line)
}

/// Python: bot/modules/safety.py:7927 `_friction_blister(line)`
pub fn _friction_blister(line: &str) -> bool {
    if !_matches(&BLISTER_WORD, line) || _matches(&BURN_WORD, line) {
        return false;
    }
    _matches(&FRICTION_WORD, line) || (_matches(&BLISTER_HOWTO, line) && !_matches(&BURNT_SELF, line))
}

/// Python: bot/modules/safety.py:7935 `_tooth_out(line)`
pub fn _tooth_out(line: &str) -> bool {
    if !_matches(&TOOTH_OUT, line) || _matches(&TOOTH_OUT_FINE, line) {
        return false;
    }
    if _matches(&TOOTH_CHILD, line) && !_matches(&TOOTH_KNOCK, line) {
        return false;
    }
    true
}

/// Python: bot/modules/safety.py:7965 `_fall_with_sign(line)`: a fall onto the head with a sign
/// that it matters.
pub fn _fall_with_sign(line: &str) -> bool {
    if _matches(&FALL_ASLEEP, line) {
        return false;
    }
    if _matches(&FALL_WORD_IDIOM, line) {
        return false;
    }
    if _matches(&FALL_THING, line) {
        return false;
    }
    if _matches(&FALL_CHECK_OFF, line) {
        return false;
    }
    if _matches(&CONFUSION_NAMED_CAUSE, line) && !_matches(&[r"\b(?:now|worse|still|again|getting\s+worse)\b"], line) {
        return false;
    }
    (_matches(&HEAD_FALL, line) || _matches(&FALL_REPEAT, line)) && _matches(&HEAD_FALL_SIGN, line)
}

/// Python: bot/modules/safety.py:7988 `_out_cold_knockout(line)`
pub fn _out_cold_knockout(line: &str) -> bool {
    if _matches(&OUT_COLD_ASLEEP, line) || _matches(&OUT_COLD_NOW, line) {
        return false;
    }
    _matches(&HEAD_OUT_COLD, line)
}

/// Python: bot/modules/safety.py:7995 `_head_bang(line)`: "red" for a head injury that needs
/// seeing now, "watch" for one to keep an eye on, None otherwise.
pub fn _head_bang(line: &str) -> Option<&'static str> {
    let still = [r"\bstill\b", r"\bright\s+now\b"];
    if !_matches(&HEAD_BANG, line) {
        if !_fall_with_sign(line) {
            if !(_matches(&HEAD_KNOCKOUT, line) || _out_cold_knockout(line)) || _matches(&HEAD_SEEN, line) {
                return None;
            }
            if _matches(&HEAD_PAST, line) && !_matches(&still, line) {
                return None;
            }
            if _tooth_out(line) {
                return None;
            }
            return Some("red");
        }
        return Some("red");
    }
    if _matches(&HEAD_SEEN, line) {
        return None;
    }
    if _matches(&HEAD_PAST, line) && !_matches(&still, line) {
        return None;
    }
    if _matches(&HEAD_RED, line) {
        return Some("red");
    }
    if _fainted_with_the_bang(line) {
        return Some("red");
    }
    Some("watch")
}

/// Python: bot/modules/safety.py:8044 `_head_person(line)`: who the head injury is about, when it
/// is not the speaker.
pub fn _head_person(line: &str) -> Option<String> {
    if found(r"\bi\s+(?:'ve\s+|had\s+|got\s+)?(?:bang|hit|knock|fell|fall|slip|trip|smack|crack|bash|smash)\w*", line) {
        return None;
    }
    if let Some(who) = _person(line).filter(|who| !who.is_empty()) {
        return Some(who);
    }
    if found(r"\b(?:his|him|he)\b", line) {
        return Some("he".to_string());
    }
    if found(r"\b(?:her|she)\b", line) {
        return Some("she".to_string());
    }
    None
}

/// Python: bot/modules/safety.py:8060 `_weight_on_a_joint(line, today=False)`: cannot put weight
/// on a joint (not a hip, not an old fall, not a long one when `today`).
pub fn _weight_on_a_joint(line: &str, today: bool) -> bool {
    if !(_matches(&WEIGHT_OFF_FEET, line) && _matches(&WEIGHT_JOINT, line)) {
        return false;
    }
    if _matches(&WEIGHT_HIP, line) {
        return false;
    }
    if _matches(&WEIGHT_OLD, line) && _matches(&WEIGHT_OLD_FALL, line) {
        return false;
    }
    if today && _matches(&WEIGHT_LONG, line) {
        return false;
    }
    true
}

/// Python: bot/modules/safety.py:8097 `_grime_on_a_surface(line)`: burnt-on grime to clean, not a
/// burn on a person.
pub fn _grime_on_a_surface(line: &str) -> bool {
    if _matches(&GRIME_OFF, line) && _matches(&CLEANABLE_SURFACE, line) && !_matches(&BURNT_SELF, line) {
        return true;
    }
    _matches(&GRIME_CLEANING, line)
        && _matches(&BURNT_GRIME, line)
        && _matches(&CLEANABLE_SURFACE, line)
        && !_matches(&PERSON_BURNT, line)
        && !_matches(&BURNT_SELF, line)
}

/// Python: bot/modules/safety.py:8125 `_pet_first_aid(line)`: the vet's version of the first
/// aid, for a pet's burn, wound or other hurt.
pub fn _pet_first_aid(line: &str) -> Vec<String> {
    if _matches(&BURN_OR_SCALD_WORD, line) {
        return REPLIES.at("pet_burn").to_vec();
    }
    if _matches(&PET_WOUND, line) {
        return _MINOR_REPLIES.at("pet_wound").to_vec();
    }
    REPLIES.at("pet_hurt").to_vec()
}

/// Python: bot/modules/safety.py:8154 `_the_injury_the_chat_is_on(line, memory)`: on a follow-up
/// that only asks about the injury, the injury the last safety card answered, or "".
pub fn _the_injury_the_chat_is_on(line: &str, memory: &Memory) -> String {
    if !_matches(&ASKS_ABOUT_THE_INJURY, line) {
        return String::new();
    }
    let turns = &memory.turns;
    let mut last_card = String::new();
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        if turn.module.as_deref() == Some("safety") {
            last_card = text::squash(&text::lower(&turn.user));
            break;
        }
    }
    if last_card.is_empty() {
        return String::new();
    }
    let earlier: HashSet<String> = text::split(&last_card).into_iter().collect();
    if text::split(line).iter().any(|word| !A_FOLLOW_UP_ABOUT_THE_INJURY.contains(word) && !earlier.contains(word)) {
        return String::new();
    }
    for (name, words) in HEAL_WORDS.iter() {
        if found(words, &last_card) {
            return name.clone();
        }
    }
    // A frozenset, walked in the bundle's (sorted) order: Python's own order is the string hash's.
    for word in THE_INJURY_NAMED_BEYOND_A_QUESTION.items() {
        if found(&format!(r"\b{word}\b"), &last_card) {
            return word.clone();
        }
    }
    String::new()
}

/// Python: bot/modules/safety.py:8184 `_minor(line, memory=None)`: the first-aid replies for a
/// small injury (with the escalation line on a body that is also unwell), or None.
pub fn _minor(line: &str, memory: &Memory) -> Option<Vec<String>> {
    let mut line = line.to_string();
    let injury = _the_injury_the_chat_is_on(&line, memory);
    if !injury.is_empty() && _first_aid(&line).is_none() {
        line = format!("{line} {injury}");
    }
    if _matches(&WHOLE_BODY_RED_FLAG, &line) {
        return None;
    }
    let reply = _first_aid(&line);
    if let Some(reply) = &reply {
        if _matches(&MILD_BODY_SIGN, &line) {
            return Some(reply.iter().map(|one| format!("{one}{}", &*MILD_BODY_ESCALATION)).collect());
        }
    }
    reply
}

/// Python: bot/modules/safety.py:8202 `_minor_replies(name)`: the replies of a MINOR row.
pub fn _minor_replies(name: &str) -> Vec<String> {
    MINOR.iter().find(|row| row.name == name).map(|row| row.replies.clone()).unwrap_or_default()
}

/// Python: bot/modules/safety.py:8210 `_first_aid(line)`: the first-aid replies for the small
/// injury the line describes, or None.
pub fn _first_aid(line: &str) -> Option<Vec<String>> {
    if _self_harm_intent(line) {
        return None;
    }
    if _matches(&SPINE_BREAK, line) {
        if _matches(&SPINE_BACK_WORK, line) && !_matches(&SPINE_BACK_FALL, line) {
            return None;
        }
        if _matches(&SPINE_NUMB_LOWER, line)
            && !_matches(&[r"\b(?:neck|spine|spinal)\b", r"\b(?:paralysed|paralyzed)\b", r"\b(?:can'?t|cant|cannot)\s+move\b"], line)
        {
            return None;
        }
        return Some(SPINE_BREAK_ANSWER.to_vec());
    }
    if _an_ordinary_helping(line) {
        return None;
    }
    if _matches(&PREVENTION_OR_BURNOUT, line) && !_matches(&BURNT_SELF, line) {
        return None;
    }
    let thing_burnt =
        (_matches(&THING_BURNT, line) || _grime_on_a_surface(line)) && !_matches(&A_BODY_PART, line) && !_matches(&BURNT_SELF, line);
    let burner_thing = _matches(&BURNER_THING, line) && !_matches(&A_BODY_PART, line) && !_matches(&BURNT_SELF, line);
    let friction = _friction_blister(line);
    let food_burnt = _matches(&BURNT_FOOD, line) && !_matches(&A_BODY_PART, line) && !_matches(&BURNT_SELF, line);
    let razor = _matches(&RAZOR_BURN, line) && !_matches(&FRICTION_BURN_HEAT, line) && !_matches(&BURNT_BODY_VERB, line);
    if _matches(&FRICTION_BURN, line)
        && !_matches(&FRICTION_BURN_HEAT, line)
        && !razor
        && (_matches(&A_BODY_PART, line) || _matches(&FRICTION_BURN_NOUN, line))
    {
        return Some(REPLIES.at("friction_burn").to_vec());
    }
    if _matches(&BLISTER_STATE, line) && !_matches(&BURN_WORD, line) && !_matches(&BLISTER_CAUSE_BURN, line) {
        return Some(BLISTER_STATE_REPLIES.to_vec());
    }
    if _matches(&HOW_LONG, line) {
        if found(r"\bsun\s?burn", line) {
            return Some(HEAL_TIME.at("sunburn").to_vec());
        }
        for (name, words) in HEAL_WORDS.iter() {
            if name == "cut" && _matches(&FOOD_CUT, line) && !_matches(&A_BODY_PART, line) && !found(&CUT_SELF, line) {
                continue;
            }
            if name == "cut" && _matches(&NOT_A_BODY_CUT, line) && !_matches(&A_BODY_PART, line) && !found(&CUT_SELF, line) {
                continue;
            }
            if found(words, line) {
                return Some(HEAL_TIME.at(name).to_vec());
            }
        }
    }
    if _matches(&GRATER_CUT, line) {
        return Some(_MINOR_REPLIES.at("cut").to_vec());
    }
    for row in MINOR.iter() {
        let name = row.name.as_str();
        let patterns: &[&'static Re] = &row.patterns;
        if name == "burn" && (thing_burnt || burner_thing || friction || razor || food_burnt) {
            continue;
        }
        if name == "cut" && _cut_off_idiom(line) {
            continue;
        }
        if name == "broken" && _matches(&BONE_OUT_LOOSE, line) && _matches(&FOOD_BONE, line) && !found(&BODY_LIMB, line) {
            continue;
        }
        if name == "cut" && _matches(&FOOD_CUT, line) && !_matches(&A_BODY_PART, line) && !found(&CUT_SELF, line) {
            continue;
        }
        if name == "limp" && _matches(&LIMP_OTHER_LIMB, line) {
            return Some(_MINOR_REPLIES.at("sprain").to_vec());
        }
        if (name == "limp" || name == "limp_long") && _matches(&LIMP_NOT, line) {
            continue;
        }
        if (name == "limp" || name == "limp_long") && _matches(&LIMP_MECHANISM, line) {
            continue;
        }
        if name == "limp_long" && !_matches(&LIMP_LONG, line) {
            continue;
        }
        if name == "limp" && _matches(&LIMP_LONG, line) {
            continue;
        }
        if name == "limp" && _matches(&LIMP_CAUSE, line) {
            if _matches(&LIMP_REPLACED, line)
                && _matches(&LIMP_SOMEBODY_ELSE, line)
                && found(r"\b(?:limp\w*|hobbl\w*|gait|walk\w*)\b", line)
            {
                return Some(_MINOR_REPLIES.at("limp_long").to_vec());
            }
            continue;
        }
        if name == "sight_long" && !_matches(&LIMP_ONGOING, line) {
            continue;
        }
        if name == "limp"
            && _matches(&LIMP_ONGOING, line)
            && (_matches(&LIMP_PERSON, line) || found(r"\b(?:limp\w*|hobbl\w*|gait)\b", line) || found(&LEG_WORDS, line))
        {
            return Some(_MINOR_REPLIES.at("limp_long").to_vec());
        }
        if name == "weight_limb_long" && !(_weight_on_a_joint(line, false) && _matches(&WEIGHT_LONG, line)) {
            continue;
        }
        if name == "weight_limb" && !_weight_on_a_joint(line, true) {
            continue;
        }
        if name == "ring_stuck" && !_ring_stuck(line) {
            continue;
        }
        if (name == "tooth_out" || name == "tooth_chipped") && _matches(&PET_TOOTH, line) && _matches(patterns, line) {
            return Some(REPLIES.at("pet_tooth").to_vec());
        }
        if name == "pet_wound" && (_matches(&HUMAN_TOO, line) || _matches(&HUMAN_BITTEN, line)) {
            continue;
        }
        if PET_FIRST_AID_ROWS.contains(name)
            && _matches(&PET_ANIMAL_HURT, line)
            && !_matches(&HUMAN_TOO, line)
            && !_matches(&HUMAN_BITTEN, line)
            && !_matches(&HUMAN_OWN_INJURY, line)
            && !_matches(&PET_PASSES_BLOOD, line)
            && !_matches(&PET_OWNS_A_THING, line)
        {
            return Some(_pet_first_aid(line));
        }
        if name == "tooth_out" && !_tooth_out(line) {
            continue;
        }
        if name == "tooth_chipped" && (_matches(&TOOTH_CHIPPED_FINE, line) || _tooth_out(line)) {
            continue;
        }
        if _matches(patterns, line) {
            return Some(row.replies.clone());
        }
    }
    if _matches(&HOW_TO_TREAT, line) && !thing_burnt && !burner_thing {
        for (name, words) in INJURY_WORDS.iter() {
            if name == "burn" && (friction || razor || food_burnt) {
                continue;
            }
            if _matches(words.as_slice(), line) {
                for row in MINOR.iter() {
                    if row.name == *name {
                        return Some(row.replies.clone());
                    }
                }
            }
        }
    }
    None
}

/// Python: bot/modules/safety.py:8381 `_last_case(memory)`: the case the last safety card (in the
/// last two turns) answered: the noted case while help is waiting, else the pool the card came
/// from.
pub fn _last_case(memory: &Memory) -> Option<String> {
    let turns = &memory.turns;
    let recent = &turns[turns.len().saturating_sub(2)..];
    if !recent.iter().any(|turn| turn.module.as_deref() == Some("safety")) {
        return None;
    }
    // `case in WAITING`: a key of the dict (an unhashable case would raise; the case is a string
    // or None here).
    let case = memory.get("safety_case");
    if let Some(Value::Str(case)) = case {
        if WAITING.contains(case) {
            return Some(case.to_string());
        }
    }
    for turn in recent.iter().rev() {
        if turn.module.as_deref() != Some("safety") {
            continue;
        }
        let said = &turn.bot;
        for (name, options) in REPLIES.items() {
            if options.iter().any(|option| option == said) {
                return Some(name.clone());
            }
        }
        return None;
    }
    None
}

/// Python: bot/modules/safety.py:8474 `_a_settling_fit(line)`: "is fitting in" said of somebody
/// settling in, not a seizure.
pub fn _a_settling_fit(line: &str) -> bool {
    let Some(hit) = FITTING_MANNER.search(line) else { return false };
    FITTING_MANNER_TAIL.match_(text::strip_chars(&line[hit.end()..], " .,!?;:")).is_some()
}

/// Python: bot/modules/safety.py:8508 `_last_said_as_written(memory)`: the last safety card in the
/// last three turns, as written.
pub fn _last_said_as_written(memory: &Memory) -> String {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        if turn.module.as_deref() == Some("safety") {
            return turn.bot.clone();
        }
    }
    String::new()
}

/// Python: bot/modules/safety.py:8518 `_resay(raw, line, memory)`: the last card said again with
/// the person the line corrects to ("it's my mum, not my dad").
pub fn _resay(raw: &str, line: &str, memory: &mut Memory) -> String {
    let mut raw = raw.to_string();
    if let Some(who) = _person(line).filter(|who| !who.is_empty()) {
        memory.set("safety_person", who.as_str());
        let pairs: &[(&str, &str)] = if who == "she" {
            &[("him", "her"), ("his", "her"), ("he", "she"), ("he's", "she's"), ("himself", "herself"), ("them", "her"), ("their", "her")]
        } else {
            &[("her", "him"), ("hers", "his"), ("she", "he"), ("she's", "he's"), ("herself", "himself"), ("them", "him"), ("their", "his")]
        };
        for (word, swap) in pairs {
            raw = sub_with(&format!(r"\b{word}\b"), swap, &raw, pyre::U);
        }
    }
    raw
}

/// Python: bot/modules/safety.py:8548 `_name_the_person(reply, line, memory)`: "them" in a reply
/// turned to the person the line (or the emergency) is about.
pub fn _name_the_person(reply: &str, line: &str, memory: &Memory) -> String {
    let named: Option<String> = match _person(line).filter(|who| !who.is_empty()) {
        Some(who) => Some(who),
        None => match memory.get("safety_person") {
            Some(Value::None) | None => None,
            Some(value) => Some(value_str(Some(value))),
        },
    };
    let Some(named) = named else { return reply.to_string() };
    let pairs: &[(&str, &str)] = if named == "she" {
        &[("them", "her"), ("their", "her"), ("they're", "she's")]
    } else {
        &[("them", "him"), ("their", "his"), ("they're", "he's")]
    };
    let mut reply = reply.to_string();
    for (word, swap) in pairs {
        reply = sub_with(&format!(r"\b{word}\b"), swap, &reply, pyre::U);
    }
    reply
}

/// The person's lines in the last four turns, newest first (`str(turn.get("user") or "")`).
fn their_last_four(memory: &Memory) -> Vec<String> {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(4)..].iter().rev().map(|turn| turn.user.clone()).collect()
}

/// Python: bot/modules/safety.py:8562 `_true_newborn(memory)`: the person said, in the last four
/// turns, that the baby is a newborn.
pub fn _true_newborn(memory: &Memory) -> bool {
    for said in their_last_four(memory) {
        if said.is_empty() {
            continue;
        }
        if _matches(&TRUE_NEWBORN_FINE, &said) {
            return false;
        }
        if _matches(&TRUE_NEWBORN, &said) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/safety.py:8584 `_tiny_baby(memory)`: the person said, in the last four
/// turns, an age that is a tiny baby's.
pub fn _tiny_baby(memory: &Memory) -> bool {
    for said in their_last_four(memory) {
        if said.is_empty() {
            continue;
        }
        if _matches(&TINY_AGE_FINE, &said) {
            return false;
        }
        if _matches(&TINY_AGE, &said) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/safety.py:8606 `_last_said(memory)`: the last safety card in the last
/// three turns, lower case.
pub fn _last_said(memory: &Memory) -> String {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        if turn.module.as_deref() == Some("safety") {
            return text::lower(&turn.bot);
        }
    }
    String::new()
}

/// `any(t.get("module") == "safety" and (t.get("bot") or "") in pool for t in turns[-2:])`.
fn a_card_from(memory: &Memory, pool: &[&[String]]) -> bool {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(2)..]
        .iter()
        .any(|turn| turn.module.as_deref() == Some("safety") && pool.iter().any(|lines| lines.iter().any(|line| *line == turn.bot)))
}

/// Python: bot/modules/safety.py:8672 `_follow_up(line, memory)`: the next line of an emergency
/// thread the last card opened (a hazard left on, a near miss over, a plan said, a slur, CPR, what
/// to do while help comes), or None.
pub fn _follow_up(line: &str, memory: &mut Memory) -> Option<String> {
    let hazard = _said_before(memory, 2);
    if _matches(&_AN_ACT_OR_NOT, line) && _matches(&_A_LIVE_HAZARD, &hazard) && !_matches(&_A_BURNT_PERSON, &hazard) && !_matches(&_A_LIVE_HAZARD, line) {
        let case = _last_case(memory).unwrap_or_default();
        if case == "pan_fire" || case == "clothes_fire" {
            _note(memory, if case == "pan_fire" { "fire" } else { &case });
            return Some(_fit(&pick(REPLIES.at(&case), memory), line, &case));
        }
        _note(memory, "fire");
        if _matches(&_A_WANTS_IT_ON, line) {
            return Some(_fit(&pick(&BACK_ON, memory), line, "fire"));
        }
        return Some(_fit(&pick(&SWITCH_OFF, memory), line, "fire"));
    }
    if safety_case(memory).as_deref() == Some("near_fire")
        && safety_answered_lately(memory)
        && _matches(&NEAR_FIRE_OVER, line)
        && !_says_something_is_burning(line)
    {
        return Some(_fit_case(&pick(REPLIES.at("near_fire_over"), memory), line, None));
    }
    if matches!(safety_case(memory).as_deref(), Some("near_thing") | Some("struck_my_bike"))
        && safety_answered_lately(memory)
        && _matches(&NEAR_THING_OVER, line)
    {
        return Some(_fit_case(&pick(REPLIES.at("near_thing_over"), memory), line, None));
    }
    let last = _last_case(memory).unwrap_or_default();
    if (WISH_FRAMES.iter().any(|frame| *frame == last) || last == "self_harm_other" || last == "poison")
        && _matches(&HELD_CRISIS_DETAIL, line)
        && !_matches(&HELD_CRISIS_OFF, line)
        && !_matches(&CRISIS_PLAN_ORDINARY, line)
    {
        _note(memory, "self_harm");
        return Some(_fit(&pick(REPLIES.at("self_harm"), memory), line, "self_harm"));
    }
    if _matches(&A_PLAN_SAID, line) && !_matches(&A_PLAN_ORDINARY, line) {
        _note(memory, "self_harm");
        return Some(pick(&CRISIS_PLAN_REPLY, memory));
    }
    let said_child = a_card_from(memory, &[REPLIES.at("slur_small_child")]);
    if said_child {
        if _matches(&SLUR_SMALL_CHILD_URGENT, line) {
            return Some(_fit_case(&pick(&SLUR_SMALL_CHILD_URGENT_REPLY, memory), line, None));
        }
        if _matches(&SLUR_SUDDEN, line) {
            _note(memory, "stroke");
            return Some(_fit(&pick(REPLIES.at("stroke"), memory), line, "stroke"));
        }
        if _matches(&SLUR_NOT_SUDDEN, line) {
            return Some(_fit_case(&pick(&SLUR_SMALL_CHILD_CALM, memory), line, None));
        }
        return None;
    }
    if _matches(&DENTAL_NUMB_ASK, line) && safety_case(memory).as_deref() == Some("dental_numb") && safety_answered_lately(memory) {
        return Some(_fit_case(&pick(&DENTAL_NUMB_FOLLOW, memory), line, None));
    }
    if _last_case(memory).unwrap_or_default().starts_with("alcohol_withdrawal") && safety_answered_lately(memory) {
        if _severe(line) {
            _note(memory, "alcohol_withdrawal_severe");
            return Some(_fit_case(&pick(REPLIES.at("alcohol_withdrawal_severe"), memory), line, None));
        }
        for (pattern, key) in [
            (&WITHDRAWAL_WORSE, "alcohol_withdrawal_worse"),
            (&WITHDRAWAL_SYMPTOM, "alcohol_withdrawal_more"),
            (&WITHDRAWAL_DURATION, "alcohol_withdrawal_when"),
            (&WITHDRAWAL_HAIR_OF_THE_DOG, "alcohol_withdrawal_hair"),
            (&WITHDRAWAL_DANGER, "alcohol_withdrawal_danger"),
        ] {
            if _matches(pattern, line) {
                _note(memory, "alcohol_withdrawal_more");
                return Some(_fit_case(&pick(REPLIES.at(key), memory), line, None));
            }
        }
        return None;
    }
    let calm: Vec<&[String]> = [
        "dental_numb",
        "slur_drink",
        "alcohol_withdrawal",
        "alcohol_withdrawal_more",
        "alcohol_withdrawal_when",
        "alcohol_withdrawal_danger",
        "alcohol_withdrawal_worse",
    ]
    .iter()
    .map(|key| REPLIES.at(key))
    .collect();
    if a_card_from(memory, &calm) {
        return None;
    }
    let case = _last_case(memory)?;
    let said = _last_said(memory);
    if ["breathing", "not_breathing", "unconscious", "drowning", "hanging"].contains(&case.as_str()) && _matches(&CPR_STOP, line) {
        return Some(pick(&CPR_STOP_REPLY, memory));
    }
    if _matches(&CPR_ASK, line)
        && (_matches(&CPR_SCENE, &said)
            || ["breathing", "not_breathing", "unconscious", "choking", "drowning", "hanging", "general", "chest"].contains(&case.as_str()))
    {
        let baby = _matches(&CPR_BABY, line) || _matches(&CPR_BABY, &said);
        let child = _matches(&CPR_CHILD, line) || _matches(&CPR_CHILD, &said);
        if _matches(&[r"\btoo\s+(?:hard|fast|deep|soft|slow|gentle)\b", r"\bdoing\s+it\s+(?:right|wrong)\b", r"\bam\s+i\s+doing\b"], line) {
            return Some(pick(&CPR_ROUGH, memory));
        }
        if baby {
            return Some(pick(&CPR_BABY_SCENE, memory));
        }
        if child {
            return Some(pick(&CPR_CHILD_SCENE, memory));
        }
        if _matches(&[r"\bshould\s+i\b", r"\bdo\s+i\b", r"\bshall\s+i\b", r"\bstart\b"], line) {
            return Some(pick(&CPR_START, memory));
        }
        return Some(pick(&CPR_HOW, memory));
    }
    if case == "chest" && _matches(&ASPIRIN, line) {
        return Some(_name_the_person(&pick(&ASPIRIN_REPLY, memory), line, memory));
    }
    if case != "self_harm" && _matches(&WHEN_HELP_COMES, line) {
        return Some(_name_the_person(&pick(&WHEN_HELP_REPLY, memory), line, memory));
    }
    if case == "abuse" && _matches(&ABUSE_SCARED, line) {
        return Some(_name_the_person(&pick(&ABUSE_SCARED_REPLY, memory), line, memory));
    }
    if case != "self_harm" && _matches(&FRIGHTENED, line) {
        return Some(_name_the_person(&pick(&FRIGHTENED_REPLY, memory), line, memory));
    }
    if _matches(&CALL_HELP, line) {
        return Some(_name_the_person(&pick(&CALL_HELP_REPLY, memory), line, memory));
    }
    if _matches(&CORRECTION_OF_PERSON, line) && case != "self_harm" {
        let raw = _last_said_as_written(memory);
        if !raw.is_empty() && _matches(&[r"\b999\b", r"\b911\b", r"\bemergency\b"], &text::lower(&raw)) {
            return Some(_resay(&raw, line, memory));
        }
    }
    if _matches(&DETAIL_AFTER_ESCALATION, line) && !_matches(&ESCALATED_ALREADY, line) {
        return Some(_name_the_person(&pick(&DETAIL_REPLY, memory), line, memory));
    }
    if !_matches(&FOLLOW_UP, line) {
        return None;
    }
    if _matches(&HELP_ARRIVED, line) && !_matches(&HELP_ASK_MORE, line) {
        return None;
    }
    // `CASE_WORDS.get(case) or CASE_WORDS["general"]`
    let case_words = match CASE_WORDS.get_str(&case).filter(|words| !words.is_empty()) {
        Some(words) => words.to_string(),
        None => CASE_WORDS.get_str("general").expect("CASE_WORDS has general").to_string(),
    };
    if !(_matches(&FOLLOW_UP_ANCHOR, line) || _matches(&[case_words.as_str()], line)) {
        return None;
    }
    let mut safe = _matches(&SAFE_NOW, line);
    if memory.get("safety_safe").map(Value::truthy).unwrap_or(false) {
        safe = true;
    }
    let table = if safe { &AFTER } else { &WAITING };
    memory.set("safety_safe", safe);
    // `options if isinstance(options, tuple) else (options,)`
    let options: Vec<String> = match table.get(&case) {
        None => return None,
        Some(Value::Str(one)) => vec![one.to_string()],
        Some(many) => many.strs(),
    };
    let named = memory.get("safety_person").cloned();
    let who = if _person(line).filter(|who| !who.is_empty()).is_some() {
        line.to_string()
    } else {
        match named.filter(Value::truthy) {
            Some(named) => value_str(Some(&named)),
            None => _person_before(memory),
        }
    };
    let mut reply = _fit(&pick(&options, memory), &who, &case);
    if FIRST_FIT.iter().any(|one| *one == case) && _person(line).filter(|who| !who.is_empty()).is_none() && _matches(&FIRST_PERSON, line) {
        for (pattern, replacement) in ME.iter() {
            reply = sub_with(pattern, replacement, &reply, pyre::I | pyre::U);
        }
    }
    Some(reply)
}

/// Python: bot/modules/safety.py:8882 `_note(memory, case)`: the case this card answered (under
/// its alias), and help not yet here.
pub fn _note(memory: &mut Memory, case: &str) {
    let case = CASE_ALIAS.get_str(case).unwrap_or(case).to_string();
    memory.set("safety_case", case);
    memory.set("safety_safe", false);
}

/// Python: bot/modules/safety.py:8902 `_sugar_is_the_shopping(line)`: every sugar named in the
/// line is followed (or preceded) by a price or an amount of it.
pub fn _sugar_is_the_shopping(line: &str) -> bool {
    let clause = compiled(r"\band\b|\bbut\b|[,.?!;]");
    let mut seen = false;
    for hit in SUGAR_MENTION.finditer(line) {
        seen = true;
        let (start, end) = (cidx(line, hit.start()), cidx(line, hit.end()));
        let after = cslice(line, end, end + 60);
        // `re.split(..., 1)[0]` and `[-1]`: the clause up to the first break, and after the first
        // break in the window before.
        let rest = match clause.search(&after) {
            Some(cut) => after[..cut.start()].to_string(),
            None => after.clone(),
        };
        let window = cslice(line, start.saturating_sub(30), start);
        let before = match clause.search(&window) {
            Some(cut) => {
                // maxsplit=1 with groups none: the last piece is everything after the first break
                window[cut.end()..].to_string()
            }
            None => window.clone(),
        };
        if !_matches(&SUGAR_SHOP, &rest) && !_matches(&SUGAR_SHOP, &before) {
            return false;
        }
    }
    seen
}

/// Python: bot/modules/safety.py:8917 `_hypo_reply(line, memory)`: the low-blood-sugar reply, or
/// None when the line is a diet question, a story or a name.
pub fn _hypo_reply(line: &str, memory: &Memory) -> Option<String> {
    if !_matches(&HYPO, line) {
        return None;
    }
    if _sugar_is_the_shopping(line) && !_matches(&HYPO_STRONG, line) {
        return None;
    }
    if _matches(&SUGAR_IN_A_RECIPE, line) && !_matches(&HYPO_STRONG, line) {
        return None;
    }
    if _matches(&HYPO_HOW_KNOW, line) && !_matches(&HYPO_STRONG, line) {
        return None;
    }
    if !_matches(&LOW_SUGAR_NOW, line) && (_matches(&HYPO_QUESTION, line) || (_matches(&HYPO_PAST, line) && !_matches(&HYPO_NOW, line))) {
        return None;
    }
    if _matches(&CONFUSED_THING, line) && !_matches(&HYPO_STRONG, line) {
        return None;
    }
    if _matches(&HYPO_LIE_IN, line) && !_matches(&HYPO_STRONG_OTHER, line) {
        return None;
    }
    let mine = _matches(&[r"\bi\b", r"\bmy\b"], line) && _person(line).filter(|who| !who.is_empty()).is_none();
    if !mine && _matches(&HYPO_NO_MOUTH, line) {
        return Some(_fit(&pick(REPLIES.at("hypo_no_mouth"), memory), line, "hypo"));
    }
    let key = if mine { "hypo_self" } else { "hypo" };
    Some(_fit(&pick(REPLIES.at(key), memory), line, "hypo"))
}

/// Python: bot/modules/safety.py:8948 `_words(text)`: the words of a line, in order, punctuation
/// and hyphens left out.
pub fn _words(said: &str) -> Vec<String> {
    compiled(r"[a-z]+(?:'[a-z]+)*").findall_str(&text::lower(said))
}

/// Python: bot/modules/safety.py:8953 `_a_repair_the_reader_made(memory)`: the words the
/// understanding engine repaired on purpose in this line (`fixes`), lower case.
pub fn _a_repair_the_reader_made(memory: &Memory) -> HashSet<String> {
    let Some(understood) = memory.understood.as_ref() else { return HashSet::new() };
    match &understood.fixes {
        crate::understood::Fixes::Map(fixes) => fixes.keys().map(|word| text::lower(word)).collect(),
        crate::understood::Fixes::List(_) => HashSet::new(),
    }
}

/// Python: bot/modules/safety.py:8970 `_repaired(text, memory)`: the line with the emergency words
/// respelt, and with every word the reader put right (in `plain`) put right here too.
pub fn _repaired(said: &str, memory: &Memory) -> String {
    let typed = crate::understanding::normalize::respell_urgent(said);
    let typed = crate::understanding::normalize::said_apart(&typed);
    let typed_text = crate::understanding::normalize::said_apart(said);
    let plain = memory.understood.as_ref().map(|u| u.plain.clone()).unwrap_or_default();
    if text::strip(&plain).is_empty() {
        return typed;
    }
    let (mine, read, heard) = (_words(&typed_text), _words(&typed), _words(&plain));
    if mine.len() != read.len() || read.len() != heard.len() {
        return typed;
    }
    let mut pieces: Vec<String> = Vec::new();
    let mut last = 0;
    for (at, hit) in compiled(r"[A-Za-z]+(?:'[A-Za-z]+)*").finditer(&typed).iter().enumerate() {
        if heard[at] != read[at]
            && heard[at] != mine[at]
            && (!LEXICON.contains(&mine[at]) || MISSPELLINGS.contains(&mine[at]) || _a_repair_the_reader_made(memory).contains(&text::lower(&mine[at])))
        {
            pieces.push(typed[last..hit.start()].to_string());
            pieces.push(heard[at].clone());
            last = hit.end();
        }
    }
    if pieces.is_empty() {
        return typed;
    }
    pieces.push(typed[last..].to_string());
    pieces.concat()
}

/// Python: bot/modules/safety.py:9037 `_withdrawal(line)`: "severe" or "mild" for alcohol
/// withdrawal said as it happens, else None.
pub fn _withdrawal(line: &str) -> Option<&'static str> {
    if _matches(&WITHDRAWAL_NOT_ALCOHOL, line) || _matches(&WITHDRAWAL_LONG_AGO, line) || _matches(&WITHDRAWAL_NOT_NOW, line) {
        return None;
    }
    if _matches(&WITHDRAWAL_WEEKS, line) && _matches(&WITHDRAWAL_FINE, line) {
        return None;
    }
    if _matches(&WITHDRAWAL_FINE, line) && !(_matches(&WITHDRAWAL_SYMPTOM, line) || _severe(line) || _matches(&WITHDRAWAL_BARE_BAD, line)) {
        return None;
    }
    let sign = _matches(&WITHDRAWAL_SYMPTOM, line) || _severe(line) || _matches(&WITHDRAWAL_BARE_BAD, line);
    let carried = ((_matches(&WITHDRAWAL_STOP, line) || _matches(&WITHDRAWAL_COLD_TURKEY, line)) && (sign || _matches(&WITHDRAWAL_NAMED, line)))
        || (_matches(&WITHDRAWAL_BARE, line) && sign)
        || (_matches(&WITHDRAWAL_SOBER, line) && sign)
        || _matches(&WITHDRAWAL_ROUTINE, line)
        || _matches(&WITHDRAWAL_ALONE_AT_HOME, line);
    if !carried {
        return None;
    }
    if _matches(&PREGNANT_LINE, line) && !_matches(&WITHDRAWAL_STRONG, line) {
        return None;
    }
    Some(if _severe(line) { "severe" } else { "mild" })
}

/// Python: bot/modules/safety.py:9112 `_says_something_is_burning(line)`: a burning word in the
/// line that the line is not talking down.
pub fn _says_something_is_burning(line: &str) -> bool {
    for hit in _BURNING_NOW.finditer(line) {
        let at = cidx(line, hit.start());
        if !_NEGATED_BEFORE.is_match(&cslice(line, at.saturating_sub(18), at)) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/safety.py:9126 `_fire_words_in(line)`: every place in the line that names
/// a fire, including "the curtains went up" (as character offsets: Python keeps the matches and
/// `_Where` positions, and only their `start()` is read).
pub fn _fire_words_in(line: &str) -> Vec<usize> {
    let mut words: Vec<usize> = _FIRE_WORD.finditer(line).iter().map(|hit| cidx(line, hit.start())).collect();
    words.extend(_BURNS_UP.finditer(line).iter().map(|hit| cidx(line, hit.start_of(1).expect("group 1"))));
    words
}

/// Python: bot/modules/safety.py:9143 `_a_fire_that_did_not_happen(line)`: every fire word sits
/// inside a "nearly/almost" clause.
pub fn _a_fire_that_did_not_happen(line: &str) -> bool {
    let near: Vec<usize> = _NEAR_WORD.finditer(line).iter().map(|hit| cidx(line, hit.start())).collect();
    let fire_words = _fire_words_in(line);
    if near.is_empty() || fire_words.is_empty() {
        return false;
    }
    let breaks: Vec<usize> = _CLAUSE_BREAK.finditer(line).iter().map(|hit| cidx(line, hit.start())).collect();
    let close = |start: usize, fire_start: usize| {
        let gap = fire_start as i64 - start as i64;
        if !(0 < gap && gap <= 40) {
            return false;
        }
        !breaks.iter().any(|&b| start < b && b < fire_start)
    };
    fire_words.iter().all(|&fire| near.iter().any(|&start| close(start, fire)))
}

static HEAD_REPLIES: PoolMap = PoolMap::new("HEAD_REPLIES");

/// `template.format(**fields)` for a template that holds only named fields.
fn format_fields(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::new();
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                out.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                out.push('}');
            }
            '{' => {
                let mut field = String::new();
                for n in chars.by_ref() {
                    if n == '}' {
                        break;
                    }
                    field.push(n);
                }
                let value = fields.iter().find(|(k, _)| *k == field).unwrap_or_else(|| panic!("KeyError: {field}"));
                out.push_str(value.1);
            }
            '}' => panic!("ValueError: Single '}}' encountered in format string"),
            c => out.push(c),
        }
    }
    out
}

/// Python: bot/modules/safety.py:9166 `respond(text, memory)` - a runtime hook; its signature is
/// the runtime's. PyBot wraps it with `bot.memory.again.replied` at the bottom of the module, so the
/// earlier-turn check (`again.instead`) is the last thing that happens to a reply.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let reply = _respond(text, memory);
    let Some(reply) = reply else { return None };
    if text::strip(&reply).is_empty() {
        return Some(reply);
    }
    // `instead(reply, text, memory) or reply`; the check never takes an answer off the screen.
    match guarded(|| again_instead(&reply, text, memory)) {
        Some(Some(framed)) if !framed.is_empty() => Some(framed),
        _ => Some(reply),
    }
}

/// `again.instead(reply, text, memory)`: the reply framed as the repeat it is, or None.
/// CONTRACT: `memory::again::instead` returns `()`, so no framing can come back yet; the call is
/// still made so a raise is a raise.
fn again_instead(reply: &str, said: &str, memory: &Memory) -> Option<String> {
    #[allow(clippy::let_unit_value)]
    let _framed = crate::memory::again::instead(reply, said, memory);
    None
}

/// The rule chain of `respond`, before the earlier-turn check: an emergency reply, a first-aid
/// reply for a small injury, or None to pass. Every local of the Python function is declared here
/// and assigned where Python assigns it.
#[allow(unused_assignments, unused_mut, unreachable_code, clippy::all)]
fn _respond(text: &str, memory: &mut Memory) -> Option<String> {
    let mut minor: Option<Vec<String>>;
    let mut _end_it_is_something_else: bool;
    let mut _said_an_attempt: bool;
    let mut _said_a_past_wish: bool;
    let mut _mine_and_undated: bool;
    let mut _scars_seen: bool;
    let mut _remorse_after_hit: bool;
    let mut _crisis_first: bool;
    let mut self_in_it: bool;
    let mut own_distress: bool;
    let mut other_death: bool;
    let mut suicide_elsewhere: bool;
    let mut denied_wish: bool;
    let mut topic_talk: bool;
    let mut intent: bool;
    let mut means_other: bool;
    let mut accident: bool;
    let mut reply: Option<String>;
    let mut withdrawal: Option<&'static str>;
    let mut thing_collapsed: bool;
    let mut own_injury: bool;
    let mut own_despair: bool;
    let mut hypothetical: bool;
    let mut pet_harm: bool;
    let mut pet_past: bool;
    let mut pet_mild: bool;
    let mut pet_all_right: bool;
    let mut pet_gasp_explained: bool;
    let mut pet_gasp_fine: bool;
    let mut pet_gasp: bool;
    let mut pet_gums_told: bool;
    let mut pet_own: bool;
    let mut thing_dying: bool;
    let mut other_dying: bool;
    let mut before: String;
    let mut hurt_is_a_pet: bool;
    let mut soft_bleed: bool;
    let mut head: Option<&'static str>;
    let mut care: bool;
    let mut own_question: bool;
    let mut start: bool;
    let mut wish_named: bool;
    let mut kind: String;
    let mut own_wish: bool;
    let mut _gasp_stripped: String;
    let mut who: Option<String>;
    let mut was_case: Option<String>;
    let mut live: bool;
    let mut sting: bool;
    let mut key: String;
    let mut urgent: bool;
    let mut flat: bool;
    let mut company: bool;
    let mut said: String;
    let mut mine: bool;
    let mut counted: Option<String>;
    // A character standing in for a letter inside a word is that letter, read before the engine
    // below takes the line apart: `k!ll` arrives as three words, one of which the engine folds to
    // `ok`, so the wish was handed on as "ok!ll" and no pattern could see it.
    // `_leet_made_whole(text) or text`
    let mut text: String = {
        let whole = _leet_made_whole(text);
        if whole.is_empty() { text.to_string() } else { whole }
    };
    // The emergency words are the one place a misspelt word costs the person the right answer:
    // "my son swalowed a coin" folded to "Noted: your son swalowed a coin". The understanding
    // engine repairs those words in place, so this reads the line with them put right; every
    // other word, and the punctuation, is exactly what the person typed.
    let line: String = _norm(&_repaired(&text, &*memory));
    // A dream is not happening now. "i dreamt my baby stopped breathing" was the infant CPR card,
    // "i dreamt i was being attacked" the assault card, "i dreamt my teeth fell out" the knocked-
    // out-tooth card and "i had a nightmare that i swallowed a bottle of pills" the overdose card -
    // four emergencies nobody in the line is having. The words of a dream are the words of the
    // thing, so the frame is what separates them, and anything that says it is happening now
    // ("still bleeding", "woke up", "right now", "i can smell smoke") keeps its card.
    if crate::memory::dream::a_dream_is_not_happening(&text) {
        return None;
    }
    // A face pushed into cake, snow or mud is a mess, not an assault: answer it lightly and let
    // the rest of the bot alone. A blow, a hold or a breath word in the line keeps the card.
    if _matches(&SOFT_FACE_PUSH, &line) && !_matches(&SOFT_FACE_STOP, &line) {
        _note(memory, "general");
        return Some(pick(&PLAY_FACE_REPLIES, memory));
    }
    // Remember who the emergency is about, so the third turn can still say "him".
    if _person(&line).as_deref().map_or(false, |s| !s.is_empty()) {
        memory.set("safety_person", _person(&line).expect("named above"));
    }
    // A parent who says out loud that they might shake the baby is the one line that cannot wait
    // for the next arm: it is read here, before everything, and it is answered with a way out.
    if _matches(&SHAKE_CHILD, &line) && !_matches(&SHAKE_IDIOM, &line) && (_matches(&SHAKE_CHILD_CONTEXT, &line) || (_matches(&SHAKE_COPING, &line) && !_matches(&SHAKE_ADULT, &line))) {
        _note(memory, "abuse");
        return Some(pick(if _matches(&SHAKE_BABY_WORD, &line) { &SHAKE_REPLY } else { &SHAKE_CHILD_REPLY }, memory));
    }
    // A long time out cold, or one somebody found them in, before the rows that read "all night"
    // and "until three" as sleep.
    if _matches(&OUT_COLD_LONG, &line) && !_matches(&OUT_COLD_SLEEPY, &line) && !_matches(&OUT_COLD_LEFT_TO_OTHERS, &line) && (_matches(&OUT_COLD_FOUND, &line) || !_matches(&FIRST_PERSON, &line)) {
        _note(memory, "general");
        return Some(pick(&OUT_COLD_NIGHT_REPLY, memory));
    }
    // Things thrown, smashed or punched in the room a person lives in, before the cards that read
    // the same words as a hobby or a clear-out.
    if _matches(&ABUSE_OBJECTS, &line) && _matches(&ABUSE_OBJECTS_PERSON, &line) && !_matches(&ABUSE_OBJECTS_OFF, &line) {
        _note(memory, "abuse");
        return Some(pick(REPLIES.at("abuse"), memory));
    }
    // The plan that arrives a turn later is the same emergency: the crisis card has been given,
    // and "i wrote a note" or "i have the pills" has no wish word of its own to be read. Only
    // inside that frame, and only when the note, the date or the tablets are not an errand.
    if safety_case(memory).as_deref().map_or(false, |c| WISH_FRAMES.iter().any(|one| one == c)) && (_matches(&CRISIS_PLAN_SAID, &line) || _matches(&CRISIS_PLAN_CARRY, &line)) && !_matches(&CRISIS_PLAN_ORDINARY, &line) {
        _note(memory, "self_harm");
        return Some(pick(&CRISIS_PLAN_REPLY, memory));
    }
    minor = _minor(&line, &*memory);
    // ...and the plate with a cut hand beside it is a knife that slipped while somebody was being
    // helped: "my daughter cut herself a slice of cake and cut her finger" was answered, by the card
    // that asks whether she is thinking about suicide, a question nobody in the line is asking
    // (azure-trout, review of #7418). The minor row answers it, and only when the injury is where a
    // knife slips - a hand, a finger, blood - so a cut on an arm or a wrist keeps its card.
    if minor.is_some() && _a_real_injury_beside_the_plate(&line) {
        _note(memory, "minor");
        return Some(_fit(&pick(minor.as_ref().expect("minor"), memory), &line, "minor"));
    }
    // A question about cleaning burnt marks off a worktop is a kitchen job, not a person with a burn.
    if _matches(&BURNT_MARKS, &line) && !_matches(&PERSON_WORD, &line) && !_matches(&BURN_MARK_ON_A_BODY, &line) {
        return None;
    }
    // "i overdosed on cake" is a joke about eating, not a drug: the 999 card is for the drug. A
    // drug word on the same line keeps the card, so "i overdosed on painkillers" is untouched.
    if _matches(&OVERDOSE_FOOD, &line) && !_matches(&OVERDOSE_DRUG, &line) {
        return None;
    }
    // A story about something that happened long ago is not happening now: "i had a bleed when i
    // was pregnant last year" is not this minute, unless the line says when.
    // A fit in childhood is the one old fit that nobody needs to be sent anywhere for: it is
    // already-known history, so it passes rather than raising the GP.
    if _matches(&SEIZURE, &line) && _matches(&CHILDHOOD_FIT, &line) && !_matches(&URGENT_NOW, &line) && !_matches(&STILL_GOING, &line) && !_matches(&ANIMAL, &line) && !_matches(&FIT_STILL_RINGS, &line) {
        return None;
    }
    // A fit a day or a week ago is still not a wait, but the ambulance has no job by now either:
    // it is seen today. "just had a fit" is not this - that one is happening now.
    if _matches(&SEIZURE, &line) && _matches(&FIT_THIS_WEEK, &line) && (_matches(&FIT_ALL_BETTER, &line) || !_matches(&URGENT_NOW, &line)) && !_matches(&STILL_GOING, &line) && !_matches(&FIT_STILL_RINGS, &line) && !_matches(&ANIMAL, &line) && !_matches(&CHILDHOOD_FIT, &line) && !_matches(&SEIZURE_STORY_GUARD, &line) {
        _note(memory, "general");
        return Some(PAST_FIT_WEEK_REPLY.to_string());
    }
    // "i cut myself off from my friends last year" is the phrasal verb, not a cut: the people are
    // what the speaker was cut off from, and there is no wound, no blood and no tool in the line.
    // A wish, a reason or an emergency anywhere in it keeps the card, so "i cut myself off from
    // everyone and i want to die" is still the crisis reply - this only covers being alone.
    if _matches(&CUT_MYSELF_OFF, &line) && !(_matches(&DESPAIR, &line) || _matches(&WISH_INTENT, &line) || _matches(&SELF_HARM_INTENT, &line) || _matches(&SELF_HARM_REASON, &line) || _matches(&URGENT_NOW, &line) || _matches(&OWN_WISH_WORDS, &line) || _matches(&CRISIS_PLAN_SAID, &line)) && !(CRISIS_WORD.is_match(&line)) {
        // A hit with the withdrawal is not a loneliness line and not a wish either: "i cut myself off
        // from my mum because she hit me" was getting the friendship tip, and the abuse reply is the
        // one for it (zippy-kangaroo, #4643).
        if _matches(&ABUSE, &line) || _matches(&ABUSE_HIT_ME, &line) {
            _note(memory, "abuse");
            return Some(_fit(&pick(REPLIES.at("abuse"), memory), &line, "abuse"));
        }
        return None;
    }
    // A near thing on a bike outranks the arm the same line wants, unless somebody is hurt now.
    if _an_accident_near_thing(&line) && !(_matches(&FALL_RED, &line) || _matches(&STILL_GOING, &line) || _matches(&DESPAIR, &line) || _matches(&WISH_INTENT, &line) || _matches(&SELF_HARM_INTENT, &line) || _matches(&CRISIS_PLAN_SAID, &line)) && !_matches(&COUNTERFACTUAL_WISH, &line) {
        _note(memory, "near_thing");
        return Some(_fit(&pick(REPLIES.at(&_near_thing_key(&line)), memory), &line, "near_thing"));
    }
    // A near thing with a heavy object, and a joke about laughing, are the two ways "killed
    // myself" is ordinary. Neither is a crisis and neither deserves a shrug, so they get their own
    // line here - above the old-story return, which was eating "i was killing myself laughing
    // yesterday" - and only when no other case on the line wants it: a chest that hurts after the
    // sofa is still the chest's answer, and a line that names a death is somebody else's turn.
    // "i killed myself laughing at the funeral" was getting "Ha, good. Killed yourself laughing,
    // not the other thing" over an open grave, and "carrying the coffin at my dad's funeral" the
    // lifting advice; the loss module answers both.
    if !_matches(&DEATH_IN_THE_LINE, &line) && !(CASES.iter().any(|case| case.name != "self_harm" && _matches(&case.patterns[..], &line))) {
        if _the_effort_frame(&line) {
            _note(memory, "near_thing");
            return Some(_fit(&pick(REPLIES.at(&_near_thing_key(&line)), memory), &line, "near_thing"));
        }
        if _the_laugh(&line) {
            _note(memory, "laughing_kill");
            return Some(_fit(&pick(REPLIES.at("laughing_kill"), memory), &line, "laughing_kill"));
        }
    }
    // An attempt that already happened is still the attempt: the week on the line is about what to
    // do now, not about whether it is worth talking about. Somebody else's attempt is their crisis,
    // and the reply for them is how to help.
    // "End it with" something that is not a means is a break-up or a cancellation, not the attempt.
    _end_it_is_something_else = _matches(&END_IT_WITH_A_THING, &line) && !_matches(&END_IT_MEANS, &line);
    _said_an_attempt = _matches(&PAST_ATTEMPT, &line) && !_end_it_is_something_else && (_matches(&PAST_ATTEMPT_STRONG, &line) || !_matches(&PAST_ATTEMPT_NOT, &line));
    _said_a_past_wish = _matches(&PAST_WISH, &line) && !_matches(&PAST_WISH_NOT, &line);
    if !_said_a_past_wish && _matches(&PAST_NEAR_ACT, &line) && !_matches(&PAST_WISH_NOT, &line) && !_matches(&PAST_ATTEMPT_NOT, &line) {
        _said_a_past_wish = true;
    }
    // The plainest way of saying it carries no date at all: "i tried to end it", "i have tried to
    // end it" are the attempt, and waiting for a "last week" before answering is how the person who
    // says it shortest gets a shrug. The date changes the reply, not whether there is one.
    _mine_and_undated = _matches(&FIRST_PERSON, &line) && _matches(&SPOKEN_ATTEMPT_UNDATED, &line) && !_end_it_is_something_else;
    // Scars say nothing about when, and somebody asking about their daughter's arms has no date
    // either, so this one shape carries its own gate.
    _scars_seen = _matches(&SELF_HARM_SCARS, &line) && !_matches(&SELF_HARM_SCARS_NOT, &line);
    // A wish said as a thing that nearly happened carries no date and needs none.
    if _matches(&COUNTERFACTUAL_WISH, &line) && !_matches(&ANIMAL, &line) && !_matches(&PAST_WISH_NOT, &line) && !_matches(&COUNTERFACTUAL_IDIOM, &line) {
        _note(memory, "self_harm");
        return Some(_fit(&pick(REPLIES.at("self_harm"), memory), &line, "self_harm"));
    }
    if (_said_an_attempt || _said_a_past_wish || _scars_seen) && (_matches(&OLD_TIME, &line) || _matches(&RECENT_TIME, &line) || _mine_and_undated || _scars_seen || _matches(&TEEN_AGE, &line)) && (!_matches(&URGENT_NOW, &line) || _said_an_attempt) && !_matches(&ANIMAL, &line) && !_idiom(&line) && !_matches(&CUT_MYSELF_OFF, &line) && !(_matches(&PAST_DEATH, &line) && !_matches(&SELF_DEAD_FEELING, &line)) && !(!_matches(&FIRST_PERSON, &line) && _matches(&LONG_AGO, &line) && !_matches(&FIRST_PERSON_WISH, &line)) {
        _note(memory, "self_harm");
        // Somebody else's attempt is their crisis, and the reply for them is how to help.
        if !_matches(&FIRST_PERSON, &line) && _matches(&THIRD_PERSON_ACTOR, &line) && !_matches(&FIRST_PERSON_WISH, &line) && !_matches(&LONG_AGO, &line) {
            return Some(pick(REPLIES.at("self_harm_other"), memory));
        }
        return Some(_fit(&pick(REPLIES.at("self_harm"), memory), &line, "self_harm"));
    }
    // A blow the speaker says they gave, or a childhood they are telling you about, is not one of
    // the cases below and it is not the old-story guard either: every case here is written for the
    // person it was done to, and "when i was a kid" is the guard's own time words. The wish wins a
    // tie, because "i hit my wife and i want to die" is the crisis first.
    // ...and the remorse after it is not the crisis. "i hit my wife once and i hate myself" got
    // the soft empathy line, and somebody who has just said they hit their wife needs what to do
    // tonight, not "what would make today a bit easier". The stand-down is as narrow as the shape:
    // it needs an act, a word that is only remorse about it, and no word that can only be about
    // dying - so "i hit my wife and i want to die", "...and i feel like a burden" and "...and i
    // keep thinking about ending it" all keep the crisis card.
    _remorse_after_hit = _matches(&I_HIT_SOMEONE, &line) && _matches(&REMORSE_FOR_THE_ACT, &line) && !(CRISIS_WORD.is_match(&line)) && !_matches(&WISH_OWN_BARE, &line) && !_self_harm_intent(&line);
    // The dying words win a tie in every shape, not only the two pattern sets: "i hit my wife and i
    // feel like a burden" is the crisis first, and its own word for it is enough to say so.
    _crisis_first = _self_harm_intent(&line) || _own_wish(&line) || _matches(&DESPAIR, &line) || (CRISIS_WORD.is_match(&line));
    if !(_crisis_first && !_remorse_after_hit) {
        if _matches(&I_HIT_SOMEONE, &line) && !_matches(&I_DID_NOT_HIT, &line) && !(_matches(&HIT_THING_NOT_PERSON, &_norm(&text))) {
            _note(memory, "abuse");
            return Some(_fit(&pick(REPLIES.at("abuse_own"), memory), &line, "abuse_own"));
        }
        if _matches(&HIT_AS_A_CHILD, &line) && !_matches(&CHILD_HIT_NOW, &line) && !_matches(&GAME_NOT_A_BLOW, &line) {
            _note(memory, "abuse");
            return Some(_fit(&pick(REPLIES.at("abuse_past"), memory), &line, "abuse_past"));
        }
    }
    // A hanging that already happened is a death, not a rescue: "my grandad hung himself" got
    // "Get them down and start CPR" for a man who died years ago, and "my dad hanged himself last
    // year" fell through to a shrug. The scene - "i found my dad hanging", "he is hanging now" -
    // still keeps the card, and so does the reader asking for help with it.
    if _matches(&HANGED_DONE, &line) && (_matches(&DEATH_SAID, &line) || _matches(&ELDER_KIN, &line) || _matches(&HANGED_LONG_AGO, &line) || (_matches(&SCHOOL_PLACE, &line) && !_matches(&YOUNG_KIN, &line)) || _matches(&OLD_TIME, &line) || _matches(&RECENT_TIME, &line)) && !_matches(&URGENT_NOW, &line) && !_matches(&OWN_WISH, &line) && !_matches(&[r"\bfound\b", r"\bis\s+hanging\b", r"\bstill\b", r"\bnow\b", r"\bhelp\b"], &line) {
        _note(memory, "loss");
        return Some(_fit(&pick(REPLIES.at("death_already"), memory), &line, "loss"));
    }
    // "out cold" is a faint in the words people use, and it used to be read as a knock-out: the
    // line got the head-injury card about "confusion, drowsiness or sickness after a bang on the
    // head" when no bang was ever said, and once that was fixed the words "at school" were enough
    // for the cold-and-school answer. A faint that has already happened keeps the faint advice.
    if _matches(&FAINT_STORY, &line) && _matches(&[r"\bout\s+cold\b"], &line) && !_matches(&OUT_COLD_NOW, &line) && !_matches(&OUT_COLD_WONT_WAKE, &line) && !_matches(&OUT_COLD_ASLEEP, &line) && !_out_cold_knockout(&line) && !_matches(&ANIMAL, &line) && !_matches(&URGENT_NOW, &line) && !_matches(&OLD_TIME, &line) {
        _note(memory, "general");
        return Some(_fit(&*OUT_COLD_REPLY, &line, "general"));
    }
    if _matches(&OLD_TIME, &line) && !_matches(&URGENT_NOW, &line) && !_matches(&STILL_GOING, &line) && !_hopeless(&line) && !_matches(&OWN_WISH, &line) && !(_withdrawal(&line).map_or(false, |s| !s.is_empty())) {
        // A fit is the one old story that still needs someone: a first one is looked into even
        // years later, so it is the GP's rather than nobody's. The family's known epilepsy, a fit
        // of coughing and a laptop that had a fit are not that story, and a pet's fit is the vet's.
        if _matches(&SEIZURE, &line) && !_matches(&FIT_ROUTINE, &line) && !_matches(&SEIZURE_STORY_GUARD, &line) && !_matches(&ANIMAL, &line) {
            _note(memory, "general");
            return Some(PAST_FIT_REPLY.to_string());
        }
        if _matches(&FAINT_STORY, &line) && !_matches(&ANIMAL, &line) {
            _note(memory, "general");
            return Some(PAST_BLACKOUT_REPLY.to_string());
        }
        // A disclosure of something done to the speaker is not the old story this guard turns
        // away. "i was abused when i was little", "i was raped years ago", "i was assaulted last
        // year" all stopped here and came back a shrug: the time words are the reason the guard is
        // here at all, and they are also how an old disclosure is said. The ambulance has no job
        // years later, so it is the abuse arm's own reply rather than the card - and a line that
        // says it is still going on never reaches this block.
        if _matches(&ABUSE_DISCLOSURE, &line) && !_matches(&ABUSE_NOW, &line) && !_matches(&ABUSE_DISCLOSURE_FINE, &line) && !_matches(&ABUSE_FINE, &line) {
            _note(memory, "abuse");
            return Some(_fit(&pick(REPLIES.at("abuse_past"), memory), &line, "abuse"));
        }
        // ...and the reverse: a disclosure that goes on to say it is still happening is not an old
        // story at all, so it must not be turned away here - it is the abuse arm's card.
        if !(_matches(&ABUSE_DISCLOSURE, &line) && _matches(&ABUSE_NOW, &line)) {
            return None;
        }
    }
    // How to do CPR, asked by somebody who is learning rather than somebody in the room with a
    // person who has stopped breathing. The numbers are two sentences and nobody should have to go
    // and search for them; the scene itself is untouched because the room has to be empty.
    if _matches(&CPR_HOWTO, &line) && !(_last_case(&*memory).as_deref().map_or(false, |c| CPR_SCENE_CASES.iter().any(|one| one == c))) && (!_matches(&CPR_ROOM_EMPTY, &line) || _matches(&CPR_NO_PULSE, &line)) && !_matches(&CPR_NOT_EFFORT, &line) && !_matches(&CPR_NOT_A_BODY, &line) && !_matches(&CHEST, &line) && !_matches(&ANIMAL, &line) {
        _note(memory, "general");
        if _matches(&[r"\blearn|learning|course\b|class\b|training|qualification|certificate", r"\bwhere\s+can\s+i\s+(?:do|take)\b"], &line) {
            // "is cpr hard to learn" is a question about a course, not a person on a floor.
            _note(memory, "general");
            return None;
        }
        start = _matches(&[r"\bshould\s+i\s+(?:start|do|begin)\b", r"\bshall\s+i\s+start\b", r"\bam\s+i\s+supposed\s+to\s+start\b"], &line);
        // A pulse or a breath in the question changes what to do, so it is answered before the
        // numbers are handed over.
        if _matches(&CPR_STILL_ALIVE, &line) && !_matches(&CPR_NOT_ALIVE, &line) {
            return Some(pick(&CPR_WHEN_ALIVE, memory));
        }
        if _matches(&CPR_NO_PULSE, &line) {
            return Some(pick(&CPR_NO_PULSE_NOW, memory));
        }
        if _matches(&CPR_BABY, &line) {
            return Some(pick(if start { &CPR_BABY_SCENE } else { &CPR_HOW_BABY }, memory));
        }
        if _matches(&CPR_CHILD, &line) {
            return Some(pick(if start { &CPR_CHILD_SCENE } else { &CPR_HOW_CHILD }, memory));
        }
        if start {
            return Some(pick(&CPR_START, memory));
        }
        return Some(pick(&CPR_HOW, memory));
    }
    // Someone shut in a room with the pills or a blade and not answering: the door is what costs
    // the time, so this comes before the "did they say why" gate.
    if _matches(&LOCKED_IN_RISK, &line) {
        // The user shut in with the pills needs the crisis line for themselves, not the 999 call.
        if found(r"\bm[y\s]*self\b", &line) {
            _note(memory, "self_harm");
            return Some(pick(REPLIES.at("self_harm"), memory));
        }
        _note(memory, "general");
        return Some(pick(REPLIES.at("locked_in"), memory));
    }
    // Someone else hurting their own body is their crisis, not a cut to dress: answer how to help
    // them. It runs before the cut and burn first aid, which "my son is cutting himself" was
    // otherwise getting, with a clean cloth and a plaster.
    // A cut that is bleeding heavily is first aid before anything else: "my brother cut himself
    // and there is blood everywhere" needs the ten minutes of pressure, not the crisis line.
    if _matches(&OTHER_SELF_HARM, &line) && !_matches(&BLEEDING, &line) && !_a_pet_hurting_itself(&line) && !_a_reason_it_was_not_meant(&line) && !_matches(&FIRST_PERSON_WISH, &line) {
        // An overdose is the ambulance whoever took it, and the amount is what decides it: the
        // crisis line is for afterwards. `my friend overdosed` was getting only the crisis card.
        if _matches(&PILLS_OVERDOSE, &line) {
            _note(memory, "poison");
            return Some(_fit(&pick(REPLIES.at("poison_other"), memory), &line, "poison"));
        }
        _note(memory, "self_harm");
        return Some(_fit(&pick(REPLIES.at("someone_else"), memory), &line, "someone_else"));
    }
    // ...unless the suicide is the subject of a lesson, a film or the news, or the thing that is
    // suicide is a job, a drive or a fixture, rather than the person's own plan. The user's own
    // wish in the same line still wins.
    // ...and a subject stays a subject only while the speaker is outside it: a lesson that they
    // keep turning over, or a wish they name in the same breath, is the person themselves.
    self_in_it = _matches(&SELF_RUMINATION, &line) && _matches(&DEATH_WORDS, &line);
    if (_matches(&SUICIDE_MENTION_CONTEXT, &line) || _matches(&SUICIDE_ABOUT_A_THING, &line) || _matches(&SUICIDE_LESSON, &line)) && !_matches(&IDIOM, &line) && !self_in_it && !_matches(&FIRST_PERSON_WISH, &line) && !_own_wish(&line) {
        // A lesson gets an answer of its own rather than falling through to whatever the line
        // happens to look like next (a teacher, a film, a homework question): the subject is heavy
        // and the reply should be about it, without the crisis card for somebody who is fine. A
        // place of learning is enough, because the setting is what makes it a lesson: "the tutor
        // covered self harm in the seminar", "suicide came up at assembly", "we watched a film
        // about suicide in pshe" were falling through to "Got it. Is there anything you'd like a
        // hand with?".
        if (_matches(&SUICIDE_LESSON, &line) || (_matches(&SUICIDE_MENTION_CONTEXT, &line) && _matches(&[&*_LEARN_PLACE], &line))) && !_matches(&LOSS_SAID, &line) {
            _note(memory, "suicide_lesson");
            return Some(_fit(&pick(REPLIES.at("suicide_lesson"), memory), &line, "suicide_lesson"));
        }
        return None;
    }
    if self_in_it {
        _note(memory, "self_harm");
        return Some(_fit(&pick(REPLIES.at("self_harm"), memory), &line, "self_harm"));
    }
    // "suicide" said as a topic, or with somebody else as its subject, is not the reader's own
    // state: the article they read, their father's angry talk and a question about helping a
    // friend have their own answers, and the crisis card asked the wrong person to ring a
    // helpline. Where the line names somebody else at risk, SOMEONE_ELSE_RISK above answers it.
    // Hopelessness is the speaker's own too: "my dad died last year and nothing ever gets better"
    // is a bereavement and a life that has stopped, and the grief in the first half must not stand
    // the card down for the second. Without HOPELESS here the loss answered for both halves.
    // ...and a hopeless phrase that is somebody else's is their crisis, not the speaker's, so it
    // does not count as the speaker's own distress here.
    own_distress = _matches(&OWN_WISH, &line) || (_hopeless(&line) && !_someone_elses_despair(&line)) || _matches(&WISH_BY_MODAL, &line) || (_matches(&OWN_DISTRESS, &line) && !_matches(&SUICIDE_TOPIC, &line));
    // A death already happened: "my dad died by suicide" is a bereavement, and the loss answer is
    // the one for it. Main rang the reader's own crisis line, which told them to ring a helpline
    // about their father's death.
    other_death = _matches(&DEATH_REPORT, &line) && !own_distress;
    // Somebody else keeps bringing it up is not a topic for the reader to chew over and not the
    // reader's own state either: the reply for it is how to help that person, which is the same
    // answer the person-at-risk arms give. Without this, "my neighbour keeps mentioning suicide
    // rates" fell to a shrug once the topic stand-down read the word as a subject.
    // "my neighbour keeps mentioning suicide rates" is the rate and not a neighbour to ask straight
    // out, so the word as a subject stays a topic here; the replies for a person who keeps saying
    // it are the talking-about-it arms above, and the topic stand-downs must not swallow them:
    // "my daughter is doing a talk on self harm" is still those lessons, not a shrug.
    suicide_elsewhere = !own_distress && (other_death || _matches(&SUICIDE_TOPIC, &line) || _matches(&SUICIDE_OTHER, &line));
    // ...and a wish the person has just said they do not have is not a wish either.
    denied_wish = _denied_wish(&line) || _matches(&WISH_DEAD_OFF, &line) || _cut_off_idiom(&line);
    // A talk, a course, a documentary or a self-harm worker is the subject of the line: somebody is
    // talking about the thing, not doing it, and the reader is not in crisis either.
    topic_talk = !_matches(&OWN_WISH, &line) && _matches(&SELF_HARM_TOPIC_OFF, &line);
    // The figure of speech stands this down, but only when the rest of the sentence is not a wish:
    // "i was killing myself laughing and now i feel empty" counts as intent, and "i feel like dying
    // laughing" stays the joke it is.
    intent = _self_harm_intent(&line) && !(_idiom(&line) && !_rest_of_it_rings(&line)) && !suicide_elsewhere && !denied_wish && !topic_talk && !(_matches(&TALKS_ABOUT_IT, &line) || _matches(&HELP_SOMEONE, &line)) && !(_avoided(&line, &[&SELF_HARM_INTENT[..], &SELF_HARM[..], &SELF_HARM_DESERVE[..]].concat()));
    // A relative dying is grief said in advance, not the speaker's own wish to die: the loss answer
    // is the one for it, so the crisis reply stands down unless the speaker says it about themselves.
    // The speaker's own wish in the same line wins over the relative's, which is what OWN_WISH is
    // for: "my nan is going to die and i keep thinking about ending it" is two things, and the
    // crisis is the half that matters. Without that clause the stand-down swallowed the wish too,
    // and the person got the grief answer.
    if intent && _matches(&DYING_OTHER, &line) && !_matches(&SELF_OWN_DYING, &line) && !_own_wish(&line) {
        intent = false;
    }
    // A chest closed by a room you cannot leave is answered rather than dropped: the advice is to
    // get out and slow the breath, with the 999 card kept for when that does not work. Anything
    // else on the line that is a real red flag (chest pain, a crush, smoke) still wins below.
    if FIGURATIVE_BREATH.is_match(&line) && !intent {
        if BREATH_SETTLED.is_match(&line) {
            return None;
        }
        if !(_matches(&CHEST, &line) || _matches(&BREATH_RED, &line) || _matches(&FIRE, &line) || _matches(&GAS_SMELL, &line) || BREATH_ROOM_RED.is_match(&line)) {
            return Some(_fit(&pick(REPLIES.at("breath_panic"), memory), &line, "panic"));
        }
    }
    intent = _self_harm_intent(&line) && !_idiom(&line) && !_matches(&END_IT_ALL_JOKE, &line) && !suicide_elsewhere && !denied_wish && !topic_talk && !(_matches(&TALKS_ABOUT_IT, &line) || _matches(&HELP_SOMEONE, &line)) && !_matches(&WISH_BY_MODAL_OFF, &line);
    // A third person's plan with a means in it keeps the card that names them, whether or not the
    // speaker's own wish fires: "my dad is going to end it with pills" is his plan, and main
    // answered the speaker as if it were theirs. The speaker's own stated wish still wins.
    // ...and a wish said with *any* way of doing it is that person's plan, not only one of the
    // words END_MEANS knows: "my mum wants to end it with her own hands", "my friend wants to end it
    // with her insulin", "my mum wants to end it with her medication" were all read as a breakup,
    // because what follows "with" is not a person. A person after "with" is a breakup again, unless
    // the line says the speaker cannot stop them: "my dad wants to end it with him and i cant stop
    // him". The lookalike guard does not stand this down: "my dad wants to end it with a knife" is
    // a plan, even though a knife is also in the kitchen.
    means_other = _matches(&END_MEANS_OTHER, &line) && (_matches(&END_MEANS, &line) || _ends_it_with_a_method(&line) || _matches(&END_HELPLESS, &line)) && !_matches(&FIRST_PERSON_WISH, &line);
    accident = _matches(&ACCIDENT, &line) && !intent && !_matches(&SELF_HARM_REASON, &line) && (!_matches(&SELF_HARM_AGAIN, &line) || _matches(&CHORE_CAUSE, &line));
    // A stated wish to be hurt or dead is the support reply, whatever the line also mentions.
    if intent {
        // A third person's plan with a means in it is their plan, and it keeps the card that names
        // them on every arm below: the lookalike guard would read "my dad wants to end it with a
        // knife" as a knife in a kitchen, and the person list below has already stood the carer
        // card down for any "end it with".
        if means_other {
            _note(memory, "self_harm");
            return Some(pick(REPLIES.at("self_harm_other"), memory));
        }
        // ...but a habit that a sport, a sleep or a thing explains is an accident said often:
        // "he hurts himself every time he plays rugby" and "she scratches herself in her sleep"
        // are a boy on a pitch and a girl with dry skin, not the act. It goes before every other
        // arm of this branch, or "my son cut himself on glass" is still answered as a crisis.
        if _matches(&SELF_HARM_BY_OTHER, &line) && _a_reason_it_was_not_meant(&line) && !_matches(&FIRST_PERSON_WISH, &line) && !found(r"\bself[-\s]?harm", &line) {
            return None;
        }
        // `my friend overdosed on purpose` is not the user's own crisis: somebody else did it, so
        // the ambulance comes first and the crisis line comes after.
        if !_matches(&FIRST_PERSON, &line) && _matches(&THIRD_PERSON_ACTOR, &line) && _matches(&PILLS_OVERDOSE, &line) {
            _note(memory, "poison");
            return Some(_fit(&pick(REPLIES.at("poison_other"), memory), &line, "poison"));
        }
        // the user is safe and someone with them is at risk, so speak to the helper.
        // A figure of speech about someone else is the figure of speech: "my mum wants to die of
        // embarrassment" and "my son wants to die of boredom" are not a person to be watched over
        // tonight. The same for ending a relationship rather than a life: "he wants to end it with
        // his girlfriend" is not the sentence this card is for.
        if (_matches(&OTHER_RISK, &line) || _matches(&SELF_HARM_BY_OTHER, &line)) && !_matches(&FIRST_PERSON, &line) && !_matches(&IDIOM, &line) && !_matches(&SELF_HARM_TOPIC_OFF, &line) && !(found(&format!(r"\bend(?:ing)?\s+it{}", &*END_IT_WITH), &line)) {
            // `my friend overdosed on purpose` is still an overdose first: the pills are the
            // emergency, and the help for the person with them comes after.
            if _matches(&PILLS_OVERDOSE, &line) {
                _note(memory, "poison");
                return Some(_fit(&pick(REPLIES.at("poison_other"), memory), &line, "poison"));
            }
            _note(memory, "self_harm");
            return Some(pick(REPLIES.at("self_harm_other"), memory));
        }
        // A cut of your own that is still bleeding wants the cloth first: "i cut myself and the
        // blood wont stop" was answered with the crisis line alone on main - no pressure on the
        // wound, no 999 - which is the wrong way round when the bleeding is what hurts them in the
        // next ten minutes. The crisis line is still in this reply, because a cut like this is
        // often self-inflicted, and the bare "i cut myself" keeps the support reply it has.
        if _matches(&SELF_CUT, &line) && (_matches(&BLEEDING, &line) || _matches(&CUT_NOT_STOPPING, &line)) {
            _note(memory, "bleeding");
            return Some(_fit(&pick(REPLIES.at("self_cut_bleeding"), memory), &line, "bleeding"));
        }
        // Hopelessness somebody else has is answered with how to help them, by the same arms that
        // already do it for their plan or their cutting: "my sister said she cant go on" was the
        // speaker's own crisis card, which told them to ring a helpline about themselves.
        if _someone_elses_despair(&line) {
            _note(memory, "self_harm");
            return Some(pick(REPLIES.at("self_harm_other"), memory));
        }
        _note(memory, "self_harm");
        return Some(_fit(&pick(REPLIES.at("self_harm"), memory), &line, "self_harm"));
    }
    // A third person is the one cutting, burning or scratching himself. It goes before the mild
    // guard, or "my son is cutting himself" is a cut to wash, and before the world-question guard,
    // because a parent asking what to do about it still needs the help for the child.
    // "my son hurt himself" with nothing else in the line is asked, not answered: the bare verb is
    // an accident as often as it is self-harm, and the crisis card answers only one of those.
    if _matches(&HURT_HIMSELF_BARE, &line) && !_matches(&HURT_HIMSELF_MEANT, &line) && !_matches(&BLEEDING, &line) && !_a_reason_it_was_not_meant(&line) && !_matches(&FIRST_PERSON_WISH, &line) && !found(r"\bm[y\s]*self\b", &line) {
        _note(memory, "general");
        return Some(pick(REPLIES.at("hurt_himself_asked"), memory));
    }
    if means_other {
        _note(memory, "self_harm");
        return Some(pick(REPLIES.at("self_harm_other"), memory));
    }
    if _matches(&SELF_HARM_BY_OTHER, &line) && !found(r"\bm[y\s]*self\b", &line) && !_matches(&SUICIDE_TOPIC, &line) && !_matches(&SELF_HARM_TOPIC_OFF, &line) && !_matches(&BLEEDING, &line) && !_matches(&FIRST_PERSON_WISH, &line) && !_a_reason_it_was_not_meant(&line) && !_matches(&SELF_REASSURED, &line) && !_matches(&NOT_NOW, &line) && !(found(&format!(r"\bend(?:ing)?\s+it{}", &*END_IT_WITH), &line)) {
        _note(memory, "self_harm");
        return Some(pick(REPLIES.at("self_harm_other"), memory));
    }
    // A hypo is asked before the mild guard does its work: "my blood sugar is low" is blood
    // without a wound, so the guard that keeps "blood sugar" out of the bleeding reply would pass
    // the emergency itself.
    reply = _hypo_reply(&line, &*memory);
    if reply.as_deref().map_or(false, |s| !s.is_empty()) {
        return reply;
    }
    // A pet that is out of puff is the vet's question, not the person's. "my dog gets breathless on
    // the stairs" was getting the human GP card about having a chest listened to, which is the bot
    // answering as if the dog were the user. A red flag or a pet in real trouble keeps its own
    // card, and so does a line with a person's own breathlessness in it.
    if _matches(&PET_BREATH, &line) && !_matches(&HUMAN_TOO, &line) && !_matches(&PET_EFFORT, &line) && (_matches(&BREATH_AT_REST, &line) || _matches(&PET_CANT_SETTLE, &line) || _matches(&PET_BREATH_NOW, &line)) && !_matches(&PET_CRISIS, &line) {
        return Some(_fit(&pick(REPLIES.at("pet_emergency"), memory), &line, "pet_emergency"));
    }
    if _matches(&PET_BREATH, &line) && !_matches(&HUMAN_TOO, &line) && !(_matches(&BREATH_RED, &line) || _matches(&BREATH_AT_REST, &line) || _matches(&PET_CRISIS, &line) || _matches(&PET_EFFORT, &line) || _matches(&PET_CANT_SETTLE, &line) || _matches(&[r"\bgums?\b"], &line)) {
        _note(memory, "pet_breath_mild");
        return Some(_fit(&pick(REPLIES.at("pet_breath_mild"), memory), &line, "pet_breath_mild"));
    }
    // ...but the same small effort with sweat, clamminess or a faint feeling is not the GP's
    // question, it is today's: that cluster is how a heart short of blood announces itself, and the
    // everyday answers were reassuring those lines with "usually just fitness".
    if (_matches(&BREATH_SHORT, &line) || _matches(&EFFORT_RECOVERY, &line)) && _matches(&EXERTION_SMALL, &line) && (_matches(&EFFORT_HEART_CLUSTER, &line) || _matches(&EFFORT_SICK_PAIR, &line) || _matches(&EFFORT_TRAVEL_PAIN, &line)) && !_matches(&BREATH_RED, &line) && !_matches(&BREATH_AT_REST, &line) && !_matches(&CHEST, &line) && !_matches(&PAST_HARM, &line) && !(_matches(&NOT_A_BREATHER, &line) || _matches(&PET_WORDS, &line)) && !_matches(&EFFORT_TOO_LONG, &line) && !_matches(&EFFORT_PATTERN, &line) && !_matches(&EFFORT_BENIGN, &line) && !intent {
        // A parent describing a child, or a person describing their mum, is not the patient: the
        // card for a heart attack is the wrong card for "my son gets out of breath on the stairs
        // and feels dizzy", and the GP visit is the right one.
        if _matches(&EFFORT_SOMEONE_ELSE, &line) {
            return Some(_fit_case(&pick(REPLIES.at("breath_exertion"), memory), &line, None));
        }
        // A dizzy spell on getting up is a blood-pressure drop, not a heart.
        if _matches(&EFFORT_POSTURAL, &line) && !_matches(&EFFORT_TRAVEL_PAIN, &line) {
            return Some(_fit_case(&pick(REPLIES.at("breath_exertion"), memory), &line, None));
        }
        return Some(_fit(&pick(REPLIES.at("breath_effort_heart"), memory), &line, "breath_effort_heart"));
    }
    // "Feel sick" on its own is not the heart's cluster, but it is not the fitness answer either: a
    // person who is breathless and sick on the stairs is a GP visit even when the sick is morning
    // sickness or a bug.
    if (_matches(&BREATH_SHORT, &line) || _matches(&EFFORT_RECOVERY, &line)) && _matches(&EXERTION_SMALL, &line) && _matches(&EFFORT_SICK, &line) && !_matches(&BREATH_RED, &line) && !_matches(&BREATH_AT_REST, &line) && !_matches(&CHEST, &line) && !_matches(&PAST_HARM, &line) && !(_matches(&NOT_A_BREATHER, &line) || _matches(&PET_WORDS, &line)) && !_matches(&EFFORT_BENIGN, &line) && !_matches(&EFFORT_PATTERN, &line) && !intent {
        return Some(_fit_case(&pick(REPLIES.at("breath_exertion"), memory), &line, None));
    }
    // Months of the same thing on the same stairs is not a body pushed today, it is a body that has
    // changed, and the everyday answer ("being puffed after stairs is usually just fitness") is the
    // wrong reassurance for it. The faint side of the cluster does not overrule this: a person who is
    // dizzy on the stairs every time needs the GP either way.
    if (_matches(&BREATH_SHORT, &line) || _matches(&EFFORT_RECOVERY, &line)) && _matches(&EXERTION_SMALL, &line) && _matches(&EFFORT_PATTERN, &line) && !_matches(&BREATH_AT_REST, &line) && !_matches(&CHEST, &line) && !_matches(&PAST_HARM, &line) && !intent {
        return Some(_fit_case(&pick(REPLIES.at("breath_exertion"), memory), &line, None));
    }
    // Out of puff on the stairs or a hill, said plainly, is a GP question: it is new breathlessness
    // and nobody has looked at it. Everything that makes it urgent still rings below and above.
    // A sweat on the stairs stands the fitness answer down but not the doctor: every hot stairwell
    // produces one, and "i get sweaty and out of breath climbing the stairs at the office" is a
    // person asking whether something has changed. So only the heart's side of the red flags keeps
    // the GP out of this one, and a pattern of months is what the GP is for.
    // 
    // EFFORT_BENIGN is not consulted here. It is what keeps the ambulance away from a hot day or a
    // cold, and that is the cluster rule's business; the GP arm is the floor under it. Standing it
    // down as well took the doctor away from a pregnant person breathless on the stairs and from
    // anyone with a stinking cold, because the settling answer ("that is what hard effort does")
    // answers a question they did not ask. Seven ordinary lines went from the GP to that.
    if (_matches(&BREATH_SHORT, &line) || _matches(&EFFORT_RECOVERY, &line)) && _matches(&EXERTION_SMALL, &line) && !(_matches(&BREATH_RED, &line) || _matches(&EFFORT_RED_HEART, &line)) && !_matches(&BREATH_AT_REST, &line) && !_matches(&CHEST, &line) && !_matches(&PAST_HARM, &line) && !intent {
        return Some(_fit_case(&pick(REPLIES.at("breath_exertion"), memory), &line, None));
    }
    // The same reading for a meal: new breathlessness at the table is the GP's, unless something
    // on the red-flag list says otherwise.
    if _matches(&BREATH_SHORT, &line) && _matches(&MEAL_BREATH, &line) && !_matches(&BREATH_RED, &line) && !_matches(&BREATH_AT_REST, &line) && !_matches(&CHEST, &line) && !_matches(&PAST_HARM, &line) && !intent {
        return Some(_fit_case(&pick(REPLIES.at("breath_meal"), memory), &line, None));
    }
    // A choke that is over is the cough that clears itself, but only while the airway is working:
    // "i choked on my coffee and i cant breathe" was getting the sip of water, so the red flags
    // take the line back to the breathing case.
    // A shaken baby is an injury that may not show for hours, so it is checked the same day.
    if _matches(&SHAKEN, &line) && !_matches(&SHAKEN_FINE, &line) {
        _note(memory, "shaken");
        return Some(pick(&SHAKEN_REPLY, memory));
    }
    // A choke on a word is a stutter, so it gets the ordinary reply rather than advice for a
    // blocked airway - unless one of the red signs is in the line too. It is checked before the
    // mild-choke answer because "i nearly choked on my words" carries "nearly choked".
    if _matches(&CHOKE_WORDS, &line) && !_matches(&CHOKE_RED, &line) && !_matches(&CHOKE_STILL_RED, &line) {
        return None;
    }
    if _matches(&MILD_CHOKE, &line) && !_matches(&CHOKE_RED, &line) && !_matches(&CHOKE_STILL_RED, &line) {
        return Some(_fit(&pick(REPLIES.at("mild_choke"), memory), &line, "minor"));
    }
    // Somebody in the water: the fracture card and the toddler's fluids were answering these,
    // because `_minor` and the plain answers both get to the line before the drowning case does.
    // The person is the whole test, so a flooded road, a flooded cellar and a child splashing about
    // in a lesson all keep their own answer.
    if (_matches(&WATER_DANGER, &line) || (_matches(&WATER_DROWNING_NOW, &line) && !_matches(&WATER_DROWNING_FINE, &line))) && (_matches(&WATER_DANGER_PERSON, &line) || _matches(&WATER_DROWNING_NOW, &line)) && !(_matches(&WATER_THING_DROPPED, &line) && !_matches(&WATER_PERSON_TOO, &line)) && !_matches(&WATER_NOT_A_PERSON, &line) && !(_matches(&WATER_PET, &line) && !_matches(&WATER_HUMAN, &line)) && !(_matches(&WATER_SELF_PUSH_FINE, &line) && !_matches(&URGENT_NOW, &line) && !_matches(&WATER_BREATH_RED, &line) && !_matches(&WATER_URGENT, &line)) && !(_matches(&WATER_FINE, &line) && !_matches(&URGENT_NOW, &line)) && !(_matches(&WATER_SWIM, &line) && !_matches(&WATER_URGENT, &line)) && !(_matches(&WATER_HEAD_FEAR, &line) && !_matches(&WATER_STILL_BAD, &line) && !_matches(&WATER_IN_NOW, &line) && !_matches(&WATER_BREATH_RED, &line)) && !(_matches(&WATER_SELF_DIP, &line) && !_matches(&URGENT_NOW, &line) && !_matches(&WATER_BREATH_RED, &line) && !_matches(&WATER_URGENT, &line)) && !(_matches(&WATER_STORY, &line) && !_matches(&URGENT_NOW, &line) && !_matches(&WATER_URGENT, &line)) {
        _note(memory, "drowning");
        if _matches(&WATER_SELF_DIP, &line) || _matches(&WATER_SELF, &line) {
            return Some(_fit(&pick(REPLIES.at("drowning_self"), memory), &line, "drowning"));
        }
        return Some(_fit(&pick(REPLIES.at("drowning"), memory), &line, "drowning"));
    }
    // "there's smoke in the hallway and the fire alarm is going off" is a fire, not an alarm
    // mention: the benign readings of "fire alarm" and "on fire" only hold without the smoke.
    // ...but a line that also says they want to be dead is not a figure of speech: "my dog is going
    // to die and i feel like ending it" is the crisis, whatever else is in it.
    // Alcohol withdrawal, which nothing here answered at all: "i stopped drinking 2 days ago and im
    // shaking" got "Most shaking like this is caffeine, a missed meal, anxiety or a fever", "im 2
    // days off the booze and sweating" got the asking-for-time-off card, and "i stopped drinking 5
    // days ago and i saw things" - the delirium tremens - got "write down what you really drink for
    // a fortnight". Someone stopping after heavy drinking is in the one stretch where the ordinary
    // drinking answers are wrong and the caffeine answer can hurt: the shakes and sweats of the
    // first days are withdrawal, the surgery can make it safe, and confusion, hallucinations or a
    // fit a day or two after the last drink is an ambulance.
    // 
    // The stop has to be a stop of the drink: "i stopped drinking coffee" and "i stopped smoking"
    // are the same sentence about something else. A symptom is what makes it a withdrawal rather
    // than a change of habit, and the words withdrawal and detox say it on their own.
    // ...but a fit said in that same line belongs to the fit answer, which the bot already had and
    // which is the first aid: "i had a seizure after stopping drinking" was the delirium tremens
    // card, and it tells the person lying beside them nothing about lying them on their side, moving
    // the hard things away and timing it. The fit case answers every one of those lines with 999 and
    // the recovery position, and the withdrawal card is what the next turn gets.
    withdrawal = if _matches(&WITHDRAWAL_FIT_ITS_OWN_ANSWER, &line) { None } else { _withdrawal(&line) };
    if withdrawal.map_or(false, |s| !s.is_empty()) && _matches(&WITHDRAWAL_ASK_DANGER, &line) {
        // "should i call 999 for alcohol withdrawal" is the call itself being asked about, and the
        // answer is what the ambulance is for rather than the ambulance.
        _note(memory, "alcohol_withdrawal");
        return Some(pick(REPLIES.at("alcohol_withdrawal_danger"), memory));
    }
    if withdrawal == Some("severe") {
        _note(memory, "alcohol_withdrawal_severe");
        return Some(pick(REPLIES.at("alcohol_withdrawal_severe"), memory));
    }
    if withdrawal.map_or(false, |s| !s.is_empty()) {
        _note(memory, "alcohol_withdrawal");
        return Some(pick(REPLIES.at("alcohol_withdrawal"), memory));
    }
    if (_matches(&MILD, &line) || _matches(&FIGURATIVE, &line)) && !_matches(&FIRE_NOW, &line) && !_matches(&NOT_WAKING_BESIDE, &line) && !(_matches(&DESPAIR, &line) || _matches(&WISH_INTENT, &line)) {
        // What clears the idiom here is the break still going on, not the word "today": in
        // "this heat is killing me today, i broke my leg years ago" the today belongs to the heat,
        // and the fall and history checks below read a break that is still going on the same way.
        if _matches(&PAST_HARM, &line) && !_matches(&STILL_GOING, &line) {
            return None;
        }
        if _matches(&SELF_REASSURED, &line) {
            return None;
        }
        // ...but the idiom only clears the words it is made of: a plain wish in the same line goes
        // on to the self-harm case below, where it is answered.
        wish_named = _matches(&WISH_NAMED, &line) && !_matches(&SUICIDE_MENTION_CONTEXT, &line) && !_idiom(&line);
        if !wish_named {
            return if minor.as_ref().map_or(false, |p| !p.is_empty()) { Some(pick(minor.as_ref().expect("minor"), memory)) } else { None };
        }
    }
    // Someone else is the one at risk: answer how to help them, not with the user's own crisis
    // reply. A bare "they" needs a person in the line, or in the turn before it.
    // The speaker's own wish wins over a relative's in the same line: "my nan is going to die and i
    // keep thinking about ending it" is two things, and the crisis is the half that matters.
    if _matches(&SOMEONE_ELSE_RISK, &line) && (!_matches(&_FIRST_PERSON_ME, &line) || _matches(&HELP_SOMEONE, &line)) && !_own_wish(&line) && !_matches(&IDIOM, &line) && !(found(r"\bend(?:ing)?\s+it\s+with\b", &line)) && !_a_reason_it_was_not_meant(&line) && !_a_pet_hurting_itself(&line) && !_an_ordinary_helping(&line) {
        if _matches(&HUMAN_ANCHOR, &line) || _matches(&PERSON_WORD, &_person_before(&*memory)) {
            _note(memory, "self_harm");
            return Some(_fit(&pick(REPLIES.at("someone_else"), memory), &line, "someone_else"));
        }
    }
    // "my souffle collapsed" is about a cake, so the collapse case is not for it - but a fire, a
    // gas smell or chest pain further along the same line still needs its answer.
    // "the bleeding wont stop" with no site and no volume is a person staring at a wound: pressure
    // first, and the step up only if it soaks through or they go pale. A steak, a pen or a ceiling
    // keeps its own answer, and a nosebleed has its own card.
    if _matches(&NOT_STOPPING_BARE, &line) && _matches(&[r"\b(?:bleed\w*|blood)\b"], &line) && found(r"\b(?:the|his|her|their|my|your)\s+(?:\w+\s+){0,2}bleed\w*\b", &line) && !_matches(&NOT_A_WOUND_OFF, &line) && !_matches(&WOUND_NAMED, &line) && !_matches(&BLEED_VOLUME, &line) && !_matches(&[r"\bnoses?\b", r"\bnose\s*bleed\b", r"\bperiod\b"], &line) && !_matches(&MOUTH_BLEED_OFF, &line) {
        return Some(_fit(&pick(REPLIES.at("bleeding_wont_stop"), memory), &line, "minor"));
    }
    // A mouth that will not stop bleeding gets pressure and a same-day look, said before any of the
    // bleeding arms can call it an ambulance.
    if _matches(&MOUTH_BLEED, &line) && _matches(&MOUTH_BLEED_NOW, &line) && !_matches(&MOUTH_BLEED_RED, &line) && !_matches(&MOUTH_BLEED_OFF, &line) {
        return Some(_fit(&pick(REPLIES.at("mouth_bleed"), memory), &line, "minor"));
    }
    thing_collapsed = _matches(&THING_COLLAPSE, &line) && !_matches(&A_PERSON, &line);
    // "what do i do if i burn my hand" is the way people ask for the first aid for a burn they
    // are about to risk, so the user's own body being the injured thing makes it a real case
    // again. Fiction, someone else and the sign-and-symptom questions stay hypothetical:
    // "what if my son burns his hand" is about the son, not about the person typing.
    own_injury = _matches(&OWN_INJURY, &line) && !_matches(&SOMEONE_ELSE_OWN, &line);
    // "what do i do when i feel hopeless" is the person typing about their own night, and the
    // "what do i do when..." frame was reading it as a question about the world: it got the
    // GENERAL["self_harm"] text, which is written for whoever is helping someone else. Despair
    // in the first person is never hypothetical, the same way the user's own burn is not.
    own_despair = _matches(&DESPAIR, &line) && _matches(&FIRST_PERSON, &line) && !_matches(&SOMEONE_ELSE_OWN, &line);
    hypothetical = _matches(&NOT_NOW_FICTION, &line) || _asking_about_the_world(&line) || (_matches(&NOT_NOW, &line) && !own_injury && !own_despair);
    // A dog choking on a bone or a cat that ate something needs the vet, not a human ambulance.
    // A person with a bite or a scratch on them is not the pet: "my dog bit my hand and it is
    // bleeding" was reaching the vet's card, which tells someone with a bleeding hand to ring the
    // vet about their dog. The wound is on the person, so the human answer below has it.
    pet_harm = (_matches(&PET_HARM, &line) || _matches(&[&*PET_WAKE], &line)) && !_matches(&HUMAN_TOO, &line) && !_matches(&HUMAN_BITTEN, &line);
    // And it still needs an answer: dropping the case left a choking dog with "Noted!".
    pet_past = _matches(&PET_PAST, &line) && !_matches(&PET_NOW, &line);
    pet_mild = _matches(&PET_MILD, &line) && !_matches(&[r"\bcan'?t\s+breathe\b", r"\bnot\s+breathing\b", r"\bunconscious\b", r"\bseizure\b", r"\bchok\w*\b"], &line);
    // "my cat is not responding to her treats", "hes 14 and stiff" and "he landed on his feet" are
    // not tonight's call, but a fine word never talks a dog out of not breathing or bleeding.
    pet_all_right = (_matches(&PET_FINE, &line) || _matches(&PET_ALL_RIGHT, &line)) && !_matches(&[r"\bcan'?t\s+breathe\b", r"\bnot\s+breathing\b", r"\bunconscious\b", r"\bseizure\b", r"\bchok\w*\b", r"\bbleeding\b", r"\bhit\s+by\b", r"\bran\s+over\b"], &line);
    // A pet puffing after a run, gasping for its water or panting in the heat is not a vet call.
    pet_gasp_explained = _matches(&PET_GASP_EXPLAINED, &line) && !_matches(&PET_GASP_RED, &line) && !_matches(&PET_GASP_ONGOING, &line);
    pet_gasp_fine = _matches(&PET_GASP, &line) && !_matches(&PET_GASP_RED, &line) && (_matches(&PET_GASP_FINE, &line) || _matches(&PET_GASP_NOUN, &line) || pet_gasp_explained || (_matches(&PET_GASP_SLEEP, &line) && _matches(&PET_GASP_SLEEP_FINE, &line)));
    // A gasp is the same vet-now case as "cannot breathe", whenever the gasp is not one of the ways
    // a healthy animal gets its breath back: "my cat woke up gasping" and "my rabbit is gasping"
    // were answered "Noted: your cat woke up gasping" and a shrug.
    pet_gasp = _matches(&PET_GASP, &line) && !pet_gasp_fine;
    // "my dog has blue gums and is gasping" wants the press-the-gum test, which says what to do with
    // your hands now and is already in the pet table: the vet's night is the weaker half of it.
    pet_gums_told = _matches(&PET_ANY, &line) && !_matches(&HUMAN_TOO, &line) && _matches(&PET_MOUTH, &line) && _matches(&PET_GUMS_TOLD, &line) && !_matches(&PET_EATEN_THING, &line);
    // The pet named twice over is still the pet: "my dog, he cant breathe" was the human 999 card.
    if _matches(&PET_APPOSITIVE, &line) && (_matches(&PET_CRISIS, &line) || _matches(&PET_BREATH, &line)) && !(_matches(&HUMAN_TOO, &PET_APPOSITIVE_SUB.sub(" ", &line))) && !pet_past && !pet_all_right {
        kind = if _matches(&PET_GASP, &line) || _matches(&PET_BREATH_ODD, &line) { "pet_breath".to_string() } else { "pet_emergency".to_string() };
        if _matches(&PET_FIT, &line) {
            kind = "pet_fit".to_string();
        }
        return Some(_fit(&pick(REPLIES.at(&kind), memory), &line, &kind));
    }
    if pet_harm && (_matches(&PET_CRISIS, &line) || pet_gasp) && !pet_all_right && !pet_gums_told && !pet_past && !pet_mild && !pet_gasp_fine && !(pet_all_right && _matches(&PET_BEHAVIOUR, &line)) && !(hypothetical && !_matches(&CHOKE_NOW, &line)) {
        // A fit or a pet that has gone floppy has its own first minutes, not "it ate something".
        if _matches(&PET_FIT, &line) {
            kind = "pet_fit".to_string();
        } else if (_matches(&PET_COLLAPSE, &line) || _matches(&PET_WAKE, &line) || _matches(&[r"\bwak(?:e|ing)\s+up\b", r"\b(?:won'?t|wont|can'?t|cannot|isn'?t|is\s+not|not)\s+wak\w*"], &line)) && !_matches(&[r"\bcan'?t\s+breathe\b", r"\bnot\s+breathing\b", r"\bbleeding\b"], &line) {
            kind = "pet_collapse".to_string();
        } else if (_matches(&PET_GASP, &line) || _matches(&PET_BREATH_ODD, &line)) && !_matches(&PET_EATEN_THING, &line) {
            kind = "pet_breath".to_string();
        } else {
            kind = "pet_emergency".to_string();
        }
        return Some(_fit(&pick(REPLIES.at(&kind), memory), &line, &kind));
    }
    // A pet that owns the symptom is the vet's: "my dog has chest pains" is not an ambulance, and
    // it is not a shrug either. Only the chest case needs this wider guard, because PET_CRISIS
    // already names the rest of the pet trouble.
    // The animal owns the wound: "a dog bit my cat" is the vet's, and the dog warden's after that,
    // not a person's bite card. A bite on a person keeps the human answer below.
    if _matches(&BITE_ON_ANIMAL, &line) && !_matches(&HUMAN_BITTEN, &line) && !_matches(&FICTION_NOW, &line) && !_matches(&PET_LONG_AGO, &line) {
        kind = if _matches(&[r"\b(?:cat|cats|kitten|kittens)\b"], &line) { "pet_bitten_cat".to_string() } else { "pet_bitten".to_string() };
        return Some(_fit(&pick(REPLIES.at(&kind), memory), &line, &kind));
    }
    pet_own = _matches(&PET_OWN, &line) && !_matches(&HUMAN_TOO, &line);
    if pet_own && _matches(&CHEST, &line) {
        return Some(_fit(&pick(REPLIES.at("pet_emergency"), memory), &line, "pet_emergency"));
    }
    // A pet or a plant that is dying is a loss or a vet bill, not the user's own crisis:
    // "my dog is going to die" was getting the helpline script. It only stands down when the
    // line carries no wish of the speaker's own, so "my dog is going to die and i feel like
    // ending it" keeps the crisis.
    // A garden or a lawn dying is a chore, not a loss: "my lawn is dying and i cant go on" is about
    // the lawn.  So on a line with a garden in it, "i cant go on" is read as "with the lawn" and only
    // an explicit wish (which cannot be about a lawn) stands the guard down.
    if _matches(&GARDEN_WORDS, &line) {
        // "i cant go on living" is the wish with a lawn in front of it: the phrase is read before
        // "cant go on" is folded away as "with the lawn", so the card is not lost to a garden.
        own_wish = _own_wish(&_GO_ON_PHRASE.sub(" ", &line)) || _matches(&GO_ON_LIVING, &line);
    } else {
        // The speaker's own wish keeps the card: "my dog is going to die and i feel like ending
        // it" is both, and the pet guard stood the whole line down for the grief. The intent the
        // self_harm case is about is the same question, so it is asked here too.
        own_wish = _own_wish(&line) || _matches(&DESPAIR, &line) || _self_harm_intent(&line);
    }
    thing_dying = (_matches(&PET_WORDS, &line) || _matches(&PLANT_WORDS, &line) || _matches(&GARDEN_WORDS, &line) || _matches(&AQUARIUM_WORDS, &line)) && !_matches(&HUMAN_TOO, &line) && !own_wish;
    // Someone else who is going to die is grief, not the speaker's crisis: "my nan is going to
    // die" and "my mum is going to die soon" were both getting the helpline script, which tells
    // a person watching a relative die to ring a crisis line. The speaker's own wish still wins.
    other_dying = _matches(&OTHER_DYING, &line) && !own_wish;
    // A class, a match or a run that leaves a person puffed is effort, not a symptom. None of the
    // cases match "i was out of breath after spinning" or "i was puffed out after my run", so those
    // lines fell through to a shrug; the words of recovery are what make them ordinary, and a bare
    // "i cant breathe after boxing" has none and keeps its card.
    if _effort_recovery(&line) && !(CASES.iter().any(|case| _matches(&case.patterns[..], &line))) {
        return Some(_fit(&pick(REPLIES.at("breath_effort"), memory), &line, "breath_effort"));
    }
    // The two halves of the glass test can come in two turns, so the rash row gets a
    // second look at the pair of them before it is passed over.
    before = _said_before(&*memory, 2);
    // The person who was strangled is the reader, and the neck case would answer them as if they
    // were watching: the reply below is the one said to them. A game, a collar or a business keeps
    // its own answer.
    if _matches(&STRANGLED_SELF, &line) && !_matches(&STRANGLE_LOOKALIKE, &line) {
        _note(memory, "abuse");
        return Some(_fit(&pick(REPLIES.at("strangled_self"), memory), &line, "strangled_self"));
    }
    // Harm that is not a bite, a burn or a fall of the usual kind, and that main answered with a
    // script about something else: venom, a hoof, and a loop of cord.
    // ...but a dog kicked by a horse, or a cat in the blind cord, is the pet's: the vet today is the
    // answer, not A&E for a person. "my horse kicked me" is the person's, so the verb's object counts.
    // The verb's object decides, and so does the subject: "my horse kicked me" and "i was kicked by
    // a horse" are the person's ribs, and only "my dog got kicked by a horse" is the vet's.
    hurt_is_a_pet = (_matches(&PET_OWN, &line) || _matches(&PET_WORDS, &line)) && !(_matches(&HUMAN_TOO, &line) || _matches(&KICKED_PERSON, &line));
    for (name, patterns, lookalike) in [
        ("snake_bite", &SNAKE_BITE, Some(&SNAKE_LOOKALIKE)),
        ("animal_kick", &ANIMAL_KICK, Some(&ANIMAL_KICK_LOOKALIKE)),
        ("blind_cord", &BLIND_CORD, None),
    ] {
        // `lookalike and _matches(lookalike, line)`: the empty tuple is no lookalike.
        if _matches(patterns, &line) && !lookalike.map_or(false, |lookalike: &Strs| !lookalike.is_empty() && _matches(lookalike, &line)) {
            if hurt_is_a_pet && name != "snake_bite" {
                _note(memory, "pet_hurt");
                return Some(_fit(&pick(REPLIES.at("pet_hurt"), memory), &line, "pet_hurt"));
            }
            _note(memory, name);
            return Some(_fit(&pick(REPLIES.at(name), memory), &line, name));
        }
    }
    // A road scare is not a shove: "someone nearly hit me at the crossing" reached the abuse card,
    // which sends the person to 999 for a blow that never landed. The road place or the vehicle is
    // what makes it the road's, and a person the speaker lives with keeps it the abuse card's.
    if _matches(&NEAR_HIT_ON_A_ROAD, &line) && (_matches(&NEAR_HIT_ON_A_ROAD_PLACE, &line) || _matches(&[format!(r"{}\b", &*_STRUCK_VEHICLE).as_str()], &line)) && !_matches(&NEAR_HIT_ON_A_ROAD_NOT, &line) {
        _note(memory, "near_thing");
        return Some(_fit(&pick(REPLIES.at(&_near_thing_key(&line)), memory), &line, "near_thing"));
    }
    // A bike a vehicle struck is the rider's accident, not a bike to carry: "a car hit my bike this
    // morning" reached the answer about strapping a bike to roof bars. A person struck as well
    // keeps the 999 card, and a near miss keeps the near thing's, so those are read first.
    if _matches(&STRUCK_MY_BIKE, &line) && !_matches(&STRUCK_BY_VEHICLE, &line) && !_matches(&STRUCK_HYPOTHETICAL, &line) {
        // "my bike nearly got hit by a van" is the same scare as the rider's, not a bike to carry.
        if _matches(&STRUCK_NOT_NOW, &line) {
            _note(memory, "near_thing");
            return Some(_fit(&pick(REPLIES.at(&_near_thing_key(&line)), memory), &line, "near_thing"));
        }
        _note(memory, "struck_my_bike");
        return Some(_fit(&pick(REPLIES.at("struck_my_bike"), memory), &line, "struck_my_bike"));
    }
    for case in CASES.iter() {
        let (name, patterns, _needs_person) = (case.name.as_str(), &case.patterns[..], case.needs_person);
        if !_matches(patterns, &line) {
            if !(name == "red_flags" && _glass_test_line(&line, &before)) {
                continue;
            }
        }
        // "i cut myself a slice of cake" is the knife and the plate, not the act: the thing after
        // the pronoun stands the self-harm row down here, the same way it does for the first aid.
        if name == "self_harm" && _an_ordinary_helping(&line) {
            continue;
        }
        // A heart attack said in the same breath as a red line is still the heart attack: the
        // spreading redness is a doctor today, and the chest is the ambulance.
        if ["wound_streak", "wound_streak_fever"].contains(&name) && _matches(&[r"\bchest\b", r"\bheart\b", r"\bbreastbone\b"], &line) && (_matches(&CHEST, &line) || _matches(&CHEST_ADJ, &line)) {
            continue;
        }
        // Not this call: a near miss, a what-if, a collision years ago, a shunt between two cars,
        // a car or a fence that was hit rather than a person, and a pet under a wheel.
        // Something heavy coming down on a person: the line names the collapse, the person under
        // it and nothing else, so the injury word the other rows want is not there.
        if name == "crush" {
            if _matches(&CRUSH_OFF, &line) && !_matches(&CRUSH_HARD, &line) {
                continue;
            }
        }
        // Somebody described as unresponsive in manner is not a body on the floor either, and the
        // words after the adjective are what say which it is.
        if name == "unconscious" && _matches(&UNRESPONSIVE_MANNER, &line) && !_matches(&BODY_IN_THE_ROOM, &line) {
            continue;
        }
        // A professional who has not replied to a message is not a body on the floor.
        if name == "unconscious" && _matches(&PRO_NOT_ANSWERING, &line) && !_matches(&BODY_IN_THE_ROOM, &line) {
            continue;
            _note(memory, "crush");
            return Some(_fit(&pick(REPLIES.at("crush"), memory), &line, "crush"));
        }
        if name == "struck_by_vehicle" && (_matches(&STRUCK_NOT_NOW, &line) || _matches(&STRUCK_HYPOTHETICAL, &line)) {
            // A near miss with a vehicle is a scare with its own answer, and standing down left
            // "i nearly got run over on my bike" to "Sounds alright. Anything good in it?" and
            // "i almost went under a bus" to the drowning card. The question, the story and the
            // shunt still stand down: only the near word and the vehicle's own verb are answered.
            if _matches(&STRUCK_BY_A_VEHICLE, &line) && !_matches(&STRUCK_BY_A_VEHICLE_NOT, &line) && !_matches(&STRUCK_HYPOTHETICAL, &line) && !(_matches(&STILL_GOING, &line) || _matches(&DESPAIR, &line) || _matches(&WISH_INTENT, &line) || _matches(&SELF_HARM_INTENT, &line) || _matches(&CRISIS_PLAN_SAID, &line)) {
                _note(memory, "near_thing");
                return Some(_fit(&pick(REPLIES.at(&_near_thing_key(&line)), memory), &line, "near_thing"));
            }
            continue;
        }
        // A fire that nearly started is not a fire, for every card in the fire's family: the
        // building's card, stop-drop-roll and the car's card all stand down together, and the
        // near thing is answered as itself rather than left to whatever else reads the line.
        if ["fire", "clothes_fire", "vehicle_fire"].contains(&name) && _a_fire_that_did_not_happen(&line) && !_matches(&FIRE_DRILL, &line) && !(_minor(&line, &*memory).as_ref().map_or(false, |p| !p.is_empty())) {
            _note(memory, "near_fire");
            return Some(_fit(&pick(REPLIES.at("near_fire"), memory), &line, "near_fire"));
        }
        // A jam, a child seat and being kept in a car by a person are not a crash: see
        // TRAPPED_VEHICLE_NOT_NOW.
        // A bite on a hand, a face or over a joint is the wide bite card's, which says to get it
        // seen today: the itch card is a downgrade there ("the spider bit my hand").
        if name == "insect_bite" && _matches(&INSECT_BITE_NEEDS_A_LOOK, &line) {
            continue;
        }
        if name == "trapped_vehicle" && _matches(&TRAPPED_VEHICLE_NOT_NOW, &line) {
            continue;
        }
        if name == "trapped_vehicle" && TRAPPED_HELD_BY_A_PERSON.is_match(&line) {
            continue;
        }
        if name == "trapped_vehicle" && TRAPPED_SELF_STUCK.is_match(&line) && !(TRAPPED_NOT_A_WAIT.is_match(&line)) {
            continue;
        }
        // A pet under a wheel is the vet's call, not the ambulance's - but standing the row down
        // sent "my cat got run over" to the bereavement card, which tells someone whose cat may be
        // alive that it has died. The pet answer is the one to give.
        if name == "struck_by_vehicle" && _matches(&PET_WORDS, &line) && !_matches(&HUMAN_TOO, &line) {
            _note(memory, "pet_hurt");
            return Some(_fit(&pick(REPLIES.at("pet_hurt"), memory), &line, "pet_hurt"));
        }
        // The weather falling is not a fever however confused the person is ("the temperature
        // keeps falling and im confused about what to wear" was the sepsis card).
        if FEVER_CASES.contains(name) && _matches(&TEMP_FALLING_WEATHER, &line) && !_matches(&FEVER_NOT_THE_WEATHER, &line) {
            continue;
        }
        // The chest row is the heart-call card. When the line also says the person has gone down
        // or is not breathing, that card - recovery position, stay on the line, CPR - is the one
        // to give, so the chest row steps aside and lets it through.
        if name == "chest" && _matches(&CHEST_ANSWERED_ELSEWHERE, &line) {
            continue;
        }
        // A laugh is what took the breath: "i cant catch my breath when i laugh" and "i laugh so
        // hard i cant breathe" were both the 999 card on main, and the breath is back in a few
        // seconds after a laugh. The laugh has to be the thing the breath belongs to, and a red
        // flag beside it (blue lips, a chest that hurts, a wheeze) still rings the ambulance.
        if ["breathing", "breath_weeks", "breath_past"].contains(&name) && _matches(&LAUGH_WORD, &line) && _matches(&LAUGH_TAKES_BREATH, &line) && !(_matches(&BREATH_RED, &line) || _matches(&BREATH_AT_REST, &line) || _matches(&LAUGH_BREATH_RED, &line) || _matches(&LAUGH_OTHER_PERSON, &line)) {
            _note(memory, "breath_laugh");
            return Some(_fit(&pick(REPLIES.at("breath_laugh"), memory), &line, "breath_laugh"));
        }
        // The weather is a cause the line names too: "i cant breathe in this heat" was the 999
        // card, telling someone in a heatwave to start CPR on themselves. Heat sends the blood to
        // the skin, so the breath comes back once the person is cool; a chest, a faint or a breath
        // that has not settled keeps the emergency.
        if ["breathing", "breath_weeks", "breath_past"].contains(&name) && _matches(&HEAT_CAUSE, &line) && _matches(&BREATH_BARE, &line) && !_matches(&HEAT_NOT_WEATHER, &line) && !(_matches(&BREATH_RED, &line) || _matches(&BREATH_AT_REST, &line) || _matches(&HEAT_BREATH_RED, &line)) {
            _note(memory, "breath_heat");
            return Some(_fit(&pick(REPLIES.at("breath_heat"), memory), &line, "breath_heat"));
        }
        // A pet that could not breathe is the vet's line, not the human ambulance.
        if name == "breath_past" && _matches(&PET_OWN, &line) && !_matches(&HUMAN_TOO, &line) {
            return Some(_fit(&pick(REPLIES.at("pet_emergency"), memory), &line, "pet_emergency"));
        }
        if name == "breath_past" && !_matches(&BREATH_PAST_CUE, &line) && !_matches(&[&*CHILD_WORD], &line) {
            continue;
        }
        // A birth story or a bout years ago is a memory, not this week's accident.
        if name == "breath_past" && _matches(&BREATH_PAST_OVER, &line) && !_matches(&BREATH_RED, &line) {
            continue;
        }
        // A blue face or blue lips outranks a breath that came back: that is the 999 call now.
        if name == "breath_past" && (_matches(&BLUE_LIPS_FACE, &line) || (_matches(&BLUE, &line) && _matches(&PERSON_WORD, &line) && !_matches(&BLUE_FOOD, &line) && !_matches(&BLUE_COLD_SELF, &line))) {
            continue;
        }
        // A plant, a phone or a plate of chips is not in the water: "i nearly drowned my plant" was
        // the drowning card with "say they were in the water" in it.
        if ["drowning", "water_danger"].contains(&name) && _matches(&DROWNED_THING, &line) {
            continue;
        }
        // "i cut myself off from my friends" is company, not a cut: the first-aid card is the wrong
        // script for a lonely year, and the crisis card was wrong before it.
        if name == "cut" && _cut_off_idiom(&line) {
            continue;
        }
        // A pet that is dying is grief, unless the line says the user wants to follow it.
        // ...and a wish said as a feeling or as being gone is the speaker following it: "my dog is
        // going to die and nobody would notice" is the crisis half, not the vet half.
        // The speaker following the animal is the half that matters, however it is said: "my cat is
        // dying and i dont want to live" is not a vet question, and OWN_WISH is what reads the wish
        // without the words this guard used to look for ("i dont", "i cant", "i cant go on").
        if name == "self_harm" && _matches(&PET_DYING, &line) && !(_matches(&[r"\bi\s+(?:want|wanna|need|might|will|could|should|do)\b", r"\bmy\s*self\b", r"\bme\s+too\b", r"\bwith\s+him\b", r"\bwith\s+her\b"], &line)) && !(_matches(&DESPAIR, &line) || _matches(&WISH_INTENT, &line)) && !_own_wish(&line) {
            return Some(_fit(&pick(REPLIES.at("pet_loss"), memory), &line, "pet_loss"));
        }
        // ...and a wish the speaker has just denied: "im not suicidal", "i would never kill myself".
        // "i feel like cutting my arms" was read as a garden line: `cuttings?` in PLANT_WORDS
        // matches "cutting", so the plant word alone made thing_dying true and stood the card
        // down. A wish to cut the speaker's own body part is not about a plant.
        if name == "self_harm" && ((thing_dying && !_a_wish_for_their_own_body(&line)) || other_dying || suicide_elsewhere || denied_wish || topic_talk) {
            continue;
        }
        // Flames out of a phone or a laptop are the battery answer, not the building card: water
        // and a cloth make a lithium fire worse, and "leave the building straight away" is not what
        // helps with a phone on the desk. A room named in the line keeps the card, because then the
        // fire is the room's and not the device's.
        // A pan alight is answered by the pan's own ten seconds, but only while the fire is still
        // in the pan: smoke, the hood, the ceiling or the alarm is the building's card.
        if name == "pan_fire" && _matches(&PAN_FIRE_SPREAD, &line) {
            continue;
        }
        if name == "fire" && (_matches(&BATTERY_FIRE, &line) || _matches(&DEVICE_BURNING, &line)) && !_matches(&ROOM_OR_HOUSE, &line) {
            continue;
        }
        // Burnt food is smoke, not a fire: the toast line is air the room and open a window.
        if name == "fire" && _matches(&BURNT_FOOD, &line) && !_matches(&FIRE_OUT, &line) && !_matches(&BATTERY_FIRE, &line) {
            continue;
        }
        // A flame in the thing that is built to hold one - a barbecue, a grill, a lit fireplace, a
        // wood burner, a candle, a fire on a screen - belongs to that thing, not to the building,
        // unless the line also says the fire got out: smoke, the ceiling, "caught fire".
        if name == "fire" && _matches(&FLAMES_NOT_FIRE, &line) && !_matches(&FIRE_ELSEWHERE, &line) {
            continue;
        }
        // A dog that keeps biting is a habit, not an attack: see DOG_HABIT.
        if ["dog_attack", "dog_bite"].contains(&name) && _matches(&DOG_HABIT, &line) && !_matches(&DOG_ATTACK_NOW, &line) && !_matches(&DOG_BLEED, &line) && !_matches(&BITE_DONE, &line) {
            continue;
        }
        // A thing bitten first and a hand caught second is mouthing, not an attack: see
        // DOG_MOUTHING. The habit words above do not cover it, because nobody calls it a habit.
        if ["dog_attack", "dog_bite"].contains(&name) && _matches(&DOG_MOUTHING, &line) && !_matches(&DOG_ATTACK_NOW, &line) && !_matches(&DOG_BLEED, &line) {
            continue;
        }
        // An engine that burns oil is the engine question, not a fire.
        if name == "vehicle_fire" && _matches(&VEHICLE_NOT_FIRE, &line) && !_matches(&VEHICLE_STILL_FIRE, &line) {
            continue;
        }
        // A smell of burning in a car is the brakes or the clutch unless a fire word, smoke or a
        // part is in the line with it.
        if ["vehicle_fire", "fire"].contains(&name) && _matches(&VEHICLE_BURNING_SMELL, &line) && !_matches(&VEHICLE_BURNING_SMELL_HARD, &line) {
            continue;
        }
        // A report of a fire that has already been put out is a story, not an emergency.
        if ["vehicle_fire", "fire"].contains(&name) && _matches(&VEHICLE_FIRE_OVER, &line) && !_matches(&VEHICLE_FIRE_OVER_HARD, &line) {
            continue;
        }
        if ["vehicle_fire", "fire"].contains(&name) && _matches(&VEHICLE_BURNING_A_PERSON, &line) && !_matches(&VEHICLE_BURNING_A_PERSON_HARD, &line) {
            continue;
        }
        // A car that burns oil is the oil question, not a fire: the fire case has "car" in front of
        // "burning" and told a driver to leave the building. A smoke word does not make it a fire -
        // "my car is burning oil and there is smoke" is the worn engine - so only a named flame,
        // the bonnet or the dashboard keeps the card here.
        if name == "fire" && _matches(&VEHICLE_NOT_FIRE, &line) && !_matches(&VEHICLE_EXHAUST_HARD, &line) {
            continue;
        }
        // Smoke out of the exhaust is a car that runs badly, not a building on fire.
        if name == "fire" && _matches(&VEHICLE_EXHAUST, &line) && !_matches(&VEHICLE_EXHAUST_HARD, &line) {
            continue;
        }
        // One appliance that smokes, or an alarm set off by the cooking, is not a house fire: the
        // answer is off at the wall, the door shut and a window open, which the home entries give.
        // Flames, a room filling, smoke in the hall or a way out you cannot take keep the card.
        if name == "fire" && !_matches(&FIRE_SCALE, &line) && (_matches(&APPLIANCE_SMOKE, &line) || _matches(&ALARM_NUISANCE, &line)) {
            continue;
        }
        // A smell of smoke on a coat, a jumper, hair or a car seat is a laundry question, and the
        // fire card for it is the bot mistaking a smell for an emergency. The card keeps the lines
        // where there is smoke in a place, where the alarm is going off, or where the fire scale
        // says the fire is more than a smell - and a line with no thing in it at all, because "i
        // can smell smoke" on its own is a warning sign and not a washing question.
        if name == "fire" && _matches(&SMOKE_IS_A_SMELL, &line) && SMELL_ON_A_THING.is_match(&line) && !_matches(&ROOM_OR_HOUSE, &line) && !_matches(&FIRE_SCALE, &line) && !_matches(&ALARM_NUISANCE, &line) {
            continue;
        }
        // "the death of my wife has hit me hard" is a bereavement, not an assault: the alarm rings
        // on the person phrase inside it, so the case stands down when the loss is what hit them.
        if name == "abuse" && _hit_by_a_loss(&line) && !_matches(&ABUSE_NOT_HIT, &line) {
            continue;
        }
        // A dog biting itself, or a scratching, itchy, balding pet, is a skin question for the
        // vet, not a dog attack on a person.
        if ["dog_bite", "dog_attack"].contains(&name) && !_matches(&HUMAN_TOO, &line) && !_matches(&DOG_BIT_A_PERSON, &line) && (_matches(&DOG_SELF_BITE, &line) || (_matches(&PET_OWN, &line) && _matches(&DOG_ITCH, &line))) {
            continue;
        }
        // A collar, a scarf in a door, a game or a business being strangled is not a hand on a
        // throat: the neck case stands down with the victim reply it would otherwise answer.
        if name == "hanging" && _matches(&STRANGLE_LOOKALIKE, &line) {
            continue;
        }
        // "how do i build up without hurting myself": the harm is what they mean to avoid.
        if name == "self_harm" && _avoided(&line, &SELF_HARM) {
            continue;
        }
        // Sore arms after a session is the training answer: "i want to hurt my arms at the gym" was
        // the crisis card on a head that read `hurt my arms` as the wish. A blade keeps the card
        // whatever the rest of the line says, so this only ever takes the line that has the verb
        // `hurt` or `harm` and a workout in it.
        if name == "self_harm" && found(&*A_HURT_OF_A_BODY_PART, &line) && found(&*A_WORKOUT_CUE, &line) {
            continue;
        }
        // A short week is not the end of one: where the line has named the money, the `nothing
        // left` clause is the money, so this case stands down - unless something else in the list
        // rings, so a real wish said next to money keeps the card.
        if name == "self_harm" && MONEY_NAMED.is_match(&line) && _matches(&NOTHING_LEFT, &line) && !(SOMEONE_HOLDS_THE_MONEY.is_match(&line)) && !(_matches(&DESPAIR_WITHOUT_NOTHING_LEFT, &line) || _matches(&WISH_INTENT, &line) || _matches(&SELF_HARM_INTENT, &line)) {
            continue;
        }
        // The weak-side arms this lane added stand down for the ordinary numb lines - a cold hand,
        // a limb slept on, a trapped nerve - unless one of the older stroke arms is in the line.
        // Sight that went days or weeks ago is the eye clinic today, not the ambulance: the weak
        // sight arms are written for what has just happened, and "i cant see out of my left eye for
        // 2 days" came back as 999. The sight that went this morning still rings.
        if name == "stroke" && !_matches(&STROKE, &line) && _matches(&SIGHT_WORD, &line) && _matches(&SIGHT_A_WHILE, &line) {
            _note(memory, "sight_days");
            return Some(_fit(&pick(REPLIES.at("sight_days"), memory), &line, "sight_days"));
        }
        // A hand that keeps dropping and a head that feels confused is a tired or anxious person
        // about themselves, not a stroke: it needs the confusion to be someone else's, or another
        // sign of the same stroke in the line.
        if name == "stroke" && _matches(&STROKE_HAND, &line) && _matches(&STROKE_HAND_SELF, &line) && !_matches(&STROKE_HAND_OTHER, &line) {
            continue;
        }
        // A hand and a weak or confused head with a known cause in the line is that cause: the flu,
        // the cold, Raynaud's, arthritis, a dementia already named, a child of two. The hard signs
        // - the face dropped, the words gone, one side gone - are new and keep the card.
        if name == "stroke" && _matches(&STROKE_HAND, &line) && _matches(&STROKE_HAND_CHRONIC, &line) && !_matches(&STROKE_HAND_HARD, &line) {
            continue;
        }
        // A pet's eye is the vet's question: "my cat is going blind" and "my cat has gone blind in
        // one eye" rang 999 for a stroke in a person (breezy-ruler, #safety). The weak sight and
        // weak-side arms read the animal as the patient, and the pet sight answer already exists.
        // The word stroke itself, or a person in the same line, keeps the card.
        if name == "stroke" && !_matches(&STROKE, &line) && (_matches(&PET_SUBJECT, &line) || _matches(&PET_SUBJECT_PLURAL, &line)) && !(_matches(&HUMAN_TOO, &line) || _matches(&MY_OWN_PART, &line)) {
            // Standing the person's card down is half the answer. With the person's stroke card out
            // of the way, a pet whose side has gone weak fell through to "Got it. Is there anything
            // you'd like a hand with?" - there was no pet weak-side answer to land on, which is the
            // other half of the same bug (steward-teal-donkey, #safety). A side that has gone, a
            // droop, a drag or a tilted head in the animal is the vet now; the pet sight answer
            // above keeps "my cat is going blind". The reply text is ivory-orange's from #10804.
            if _matches(&PET_WEAK, &line) {
                _note(memory, "pet_weak");
                return Some(_fit(&pick(REPLIES.at("pet_weak"), memory), &line, "pet_weak"));
            }
            continue;
        }
        // The FAST pair outranks the wide stand-down: `dentist`, `cold` and `slept on` are in it,
        // so "my face is numb from the dentist and my arm is weak" was stood down here and never
        // reached the pair below. A pair is the pair whatever else is in the line.
        if name == "stroke" && !_matches(&STROKE, &line) && _matches(&STROKE_WEAK_OFF, &line) && !_matches(&STROKE_FAST, &line) {
            continue;
        }
        // An old pet going blind is not a person having a stroke: "my dog is 12 and going blind"
        // rang 999 because "going blind" is one of the stroke signs. The pet has to be the subject,
        // and a first person or a human sign in the same line keeps the card.
        if name == "stroke" && !_matches(&STROKE, &line) && _matches(&PET_SUBJECT, &line) && _matches(&PET_SIGHT, &line) && !_matches(&FIRST_PERSON, &line) && !_matches(&HUMAN_ANCHOR, &line) {
            continue;
        }
        // A nail that went into the shoe is a puncture; a tyre, a wall and a picture hook are not.
        if name == "puncture" && _matches(&PUNCTURE_OFF, &line) {
            continue;
        }
        // A baby who fell and is now sick, sleepy or floppy is the head answer's ambulance, not
        // this same-day check.
        // A bang on the head already has its own watch-or-999 answer further down, and it was
        // right for "my toddler fell off the sofa and hit his head": this card is only for a fall
        // that says nothing else.
        if name == "baby_fell" && _head_bang(&line).map_or(false, |s| !s.is_empty()) {
            continue;
        }
        // A baby who went to sleep did not fall: bedtime, a nap in the car and a dropped dummy are
        // what babies do, and the red-flag list has "asleep" in it, so this guard has to come
        // first or "my baby fell asleep" is the ambulance.
        if name == "baby_fell" && _matches(&BABY_FELL_OFF, &line) && !_matches(&BABY_FELL_SURFACE, &line) {
            continue;
        }
        if name == "baby_fell" && _matches(&BABY_FELL_RED, &line) {
            _note(memory, "baby_fell_red");
            return Some(_fit(&pick(REPLIES.at("baby_fell_red"), memory), &line, "baby_fell_red"));
        }
        // A short fall with nothing after it is learning to walk, and the red flags have already
        // had their turn above.
        if name == "baby_fell" && _matches(&BABY_FELL_OVER, &line) {
            continue;
        }
        // An electric cooker is not the gas ring (see ELECTRIC_COOKER): the cooking answer for
        // "can I leave the slow cooker on while I'm at work" is a better reply than the leak script.
        if name == "gas_left" && _matches(&ELECTRIC_COOKER, &line) && !_matches(&GAS_WORD, &line) {
            continue;
        }
        if pet_harm && ["breathing", "bleeding", "poison", "swallowed_thing", "seizure", "unconscious", "broken", "hypo", "chest", "stroke", "heat", "hanging", "fall", "bone_throat"].contains(&name) {
            continue;
        }
        // The amount of blood does not decide a nose: "my nose is bleeding a lot" is a nosebleed,
        // which has its own card further down, not an ambulance. Anything that makes it the
        // exception (blood that will not stop, a blood thinner, going faint, a knock on the head)
        // keeps the ambulance, and so does a nose that is not the only place bleeding.
        if name == "bleeding" && _matches(&NOSE_SITE, &line) && !_matches(&NOSE_RED, &line) && !(_matches(&[r"\b(?:cut|gash|gashes|wound|wounds|sliced|stabbed|graze[ds]?)\b"], &line)) {
            continue;
        }
        // A nose that is blocked explains the breathing, and lying down or sitting still takes
        // the urgent answer instead: "i cant breathe because my nose is blocked" was ring-999.
        // A blocked nose or a chesty cold does explain the breathing, so it is not the ambulance -
        // but it is not nothing either. It gets the same-day line, because "I'm listening" is no
        // answer to "i cant breathe".
        if name == "breathing" && !_matches(&BREATH_AT_REST, &line) && _matches(&BREATH_CAUSE, &line) && !_matches(&BREATH_CAUSE_RED, &line) {
            _note(memory, "breath_cause");
            if !(_matches(&NIGHT_BREATH, &line) && _matches(&BREATH_CAUSE, &line)) {
                return Some(_fit(&pick(REPLIES.at("breath_cause"), memory), &line, "breath_cause"));
            }
            continue;
        }
        // Breathless in the night, or when lying flat, is the same-day appointment the night case
        // below gives; a red flag or a night still going on keeps the ambulance here.
        if name == "breathing" && _matches(&NIGHT_BREATH, &line) && !_matches(&NIGHT_BREATH_RED, &line) {
            continue;
        }
        // Someone who has stopped breathing needs CPR, not the chest pain or the inhaler reply.
        if name == "chest" && _matches(&NOT_BREATHING_NOW, &line) {
            continue;
        }
        // A cuppa is not an airway: "im gasping for a brew" was ringing 999, which in Britain is
        // how you say you are thirsty.
        // The bare word is also a red flag, so take the gasp out of the line first and let the
        // other red flags decide.
        // The red-flag re-check runs on the line with the gasp word taken out, because
        // "gasping" is itself one of the BREATH_RED patterns: without the strip the guard can
        // never fire. A line that says it cannot breathe is a red flag on its own.
        _gasp_stripped = sub_with(r"\bgasp\w*", "", &line, pyre::I | pyre::U);
        if name == "breathing" && _matches(&GASPING_THIRST, &line) && !(_matches(&BREATH_RED, &_gasp_stripped) || _matches(&BREATH_STILL_RED, &_gasp_stripped)) {
            continue;
        }
        if name == "breathing" && _matches(&NOT_BREATHING_NOW, &line) {
            _note(memory, "not_breathing");
            // 30:2 at 5-6 cm is the adult sequence; a baby or a child gets its own.
            if _matches(&CPR_BABY, &line) {
                return Some(_fit(&pick(&CPR_BABY_SCENE, memory), &line, "not_breathing"));
            }
            if _matches(&CPR_CHILD, &line) {
                return Some(_fit(&pick(&CPR_CHILD_SCENE, memory), &line, "not_breathing"));
            }
            return Some(_fit(&pick(REPLIES.at("not_breathing"), memory), &line, "not_breathing"));
        }
        // A class or a sport named with the words of recovery is effort, not the lungs: "i cant catch
        // my breath after spin class" rang the ambulance for a person who had just done some
        // exercise, and "i was out of breath after spinning" got a shrug. The recovery phrase is what
        // makes it ordinary; a bare "i cant breathe after boxing" has none and keeps its card.
        if ["breathing", "breath_weeks", "breath_past"].contains(&name) && _effort_recovery(&line) {
            return Some(_fit(&pick(REPLIES.at("breath_effort"), memory), &line, "breath_effort"));
        }
        // Laughing hard leaves anyone gasping, and the ambulance for it is a false alarm: the laugh
        // has to be the stated cause ("when i laugh", "laughing makes me"), and at rest, a wheeze,
        // chest pain, blue lips or blood still keep the breath card above.
        if ["breathing", "breath_weeks", "breath_past"].contains(&name) && _matches(&BREATH_LAUGH, &line) && !(_matches(&BREATH_RED, &line) || _matches(&BREATH_STILL_RED, &line) || _matches(&LAUGH_BREATH_RED, &line) || _matches(&LAUGH_OTHER_PERSON, &line)) && !_matches(&BREATH_AT_REST, &line) && !_matches(&[r"\bstill\b", r"\bright\s+now\b"], &line) {
            return Some(_fit_case(&pick(&BREATH_LAUGH_REPLY, memory), &line, None));
        }
        // Breathlessness that comes on with effort, and nothing else, is ordinary.
        if name == "breathing" && _matches(&EXERTION, &line) && !_matches(&BREATH_RED, &line) {
            continue;
        }
        if name == "breathing" && _matches(&SHORT_OF_BREATH, &line) && _matches(&EFFORT_ANYWHERE, &line) && !_matches(&BREATH_RED, &line) && (_matches(&OUT_OF_BREATH, &line) || !_matches(&EFFORT_WEAK, &line)) {
            continue;
        }
        // "the app has trouble breathing", "my car is having trouble breathing": a person or pet
        // in the line always wins, a gadget on its own never does.
        if name == "breathing" && _matches(&NOT_A_BREATHER, &line) && !_matches(&A_BREATHER, &line) && !_matches(&BREATH_RED, &line) {
            continue;
        }
        // Said to be over: "my mum is short of breath and it passed", "hes short of breath but hes
        // fine now". An ambulance for a symptom the person has just told you has gone is the bot not
        // listening, and the cheerful ack it would otherwise get ("Lovely to hear. Enjoy it!") is not
        // honest either, so this gets its own line: good that it passed, and still worth a same-day
        // check. `still` and `right now` keep the ambulance, because those are the present tense.
        // A choke that is over is the cough advice below, not this: "he choked on a grape but hes
        // fine now" wants the cough and the 999 test, not a same-day breathlessness check.
        if name == "breathing" && _matches(&BREATH_OVER, &line) && !(_matches(&BREATH_RED, &line) || _matches(&BREATH_STILL_RED, &line)) && !_matches(&[r"\bstill\b", r"\bright\s+now\b"], &line) && !_matches(&CHOKE_HELP, &line) {
            return Some(_fit(&pick(REPLIES.at("breath_past"), memory), &line, "breath_past"));
        }
        // A habit of breathlessness while talking or singing for a long stretch is not the red flag
        // that "short of breath while talking" is, and main answered it with a shrug.
        if name == "breathing" && _matches(&TALK_HABIT, &line) && !(_matches(&BREATH_RED, &line) || _matches(&BREATH_STILL_RED, &line)) {
            continue;
        }
        // A duration makes it a symptom rather than an emergency: weeks of breathlessness is a
        // same-day appointment, and an ambulance for it frightens someone who was only saying how
        // long it had been going on. A red flag or "can't breathe" anywhere in the line still wins.
        if name == "breathing" && _matches(&CHRONIC_BREATH, &line) && !(_matches(&BREATH_RED, &line) || _matches(&BREATH_STILL_RED, &line)) {
            return Some(_fit(&pick(REPLIES.at("breath_weeks"), memory), &line, "breath_weeks"));
        }
        // Something that went down the wrong way an hour ago and is over is not an emergency.
        // A choking that is over - last year, at lunch, as a kid - is a memory for every arm that
        // answers choking, not only the breathing one.
        if ["breathing", "breath_past", "bone_throat", "bone_throat_red"].contains(&name) && _matches(&CHOKE_OVER, &line) && !_matches(&CHOKE_NOW, &line) {
            // A choke that is over is still worth the cough advice and the 999 test when a person
            // is the one who did it: "my gran choked at lunch" is somebody asking whether that
            // was handled right, and it was getting a shrug instead of any answer at all.
            // The thing that chokes (a sink, a drain, an engine), a habit ("my baby used to choke
            // on lumps", which is the weaning question) and a first-person choke, which
            // `MILD_CHOKE` answers on its own, all stand down exactly as before.
            if _matches(&CHOKE_PERSON, &line) && !_matches(&CHOKE_NOT_PERSON, &line) && !_matches(&CHOKE_HABIT, &line) {
                _note(memory, "choking");
                return Some(_fit(&pick(REPLIES.at("mild_choke"), memory), &line, "minor"));
            }
            continue;
        }
        // A sink, a drain, an engine or a lawn chokes too, and "how do i stop my baby choking" is a
        // question about a baby who is not choking: neither is this minute, so the breathing case
        // stands down and the module that owns the line answers it.
        if name == "breathing" && _matches(&CHOKE_NOT_PERSON, &line) && !_matches(&CHOKE_PERSON, &line) && !_matches(&CHOKE_NOW, &line) {
            continue;
        }
        // Someone choking now gets the back blows and the thrusts while the ambulance comes. The
        // cough that still works, a thing in the line and a choking that is over all stay ordinary.
        // A person choking with no air gets the hands, not a phone number: five back blows, five
        // abdominal thrusts, and the baby or the self version of the same thing. A thing that
        // chokes (sink, drains, engine), the prevention question, a laugh and a choke that is
        // already over are all somebody else's line, so they pass.
        if ["breathing", "stroke"].contains(&name) && _matches(&CHOKE_HELP, &line) && !_matches(&PET_HARM, &line) && !(_matches(&CHOKE_NOT_PERSON, &line) && !_matches(&CHOKE_PERSON, &line)) {
            if _matches(&CHOKE_PREVENT, &line) {
                // How do i stop my baby choking on grapes: the shape of the food, for a baby who
                // is not choking. It is the same question the parent asks a health visitor.
                return Some(_fit(&pick(REPLIES.at("choke_prevent"), memory), &line, "minor"));
            }
            if _matches(&CHOKE_AIR_OK, &line) || _matches(&CHOKE_SETTLED, &line) {
                // The cough that still works is the airway clearing itself: the cough advice, not
                // the five and five, and not a bare ambulance card.
                _note(memory, "choking");
                return Some(_fit(&pick(REPLIES.at("mild_choke"), memory), &line, "minor"));
            }
            if _matches(&CHOKE_BABY, &line) {
                _note(memory, "choking");
                return Some(_fit(&pick(REPLIES.at("choking_baby"), memory), &line, "breathing"));
            }
            kind = if _matches(&FIRST_PERSON, &line) && !_matches(&CHOKE_PERSON, &line) { "choking_self".to_string() } else { "choking_help".to_string() };
            _note(memory, "choking");
            return Some(_fit(&pick(REPLIES.at(&kind), memory), &line, "breathing"));
        }
        if _matches(&BONE_PAST, &line) && !_matches(&BONE_STILL, &line) && _matches(&BONE_THROAT, &line) && !_matches(&BONE_RED, &line) {
            continue;
        }
        // A bone in the throat: the swallow and the breath decide whether it is the ambulance or
        // the same-day check; a bone in the stock pot, the leg or a dog's mouth is none of this.
        if name == "bone_throat" {
            if _matches(&BONE_ORDINARY, &line) && !_matches(&BONE_RED, &line) {
                continue;
            }
            if _matches(&BONE_RED, &line) {
                return Some(_fit(&pick(REPLIES.at("bone_throat_red"), memory), &line, "bone_throat"));
            }
        }
        if name == "self_harm" && _matches(&WORLD_QUESTION, &line) && !_matches(&[r"\bi\b", r"\bmy\b", r"\bmyself\b"], &line) {
            continue;
        }
        if name == "child_drank" && _matches(&CHILD_DRANK_WEAK, &line) && !_matches(&CHILD_YOUNG, &line) {
            continue;
        }
        // The numbing after a filling explains the slurring; a drooping face, a weak limb or
        // confusion with it does not, and those keep walking down to the stroke card.
        // A child that young is usually just learning the sounds; a weak side or a fit beside it
        // is not that, and keeps the ambulance.
        // A weak side or a drooping face with the drink is the ambulance: the drink explains the
        // mouth, not the arm.
        if name == "slur_drink" && _matches(&SLUR_RED_BESIDE, &line) {
            _note(memory, "stroke");
            return Some(_fit(&pick(REPLIES.at("stroke"), memory), &line, "stroke"));
        }
        if name == "slur_small_child" {
            if _matches(&SLUR_RED_BESIDE, &line) || _matches(&[r"\bfit\b", r"\bfever\b", r"\brash\b"], &line) {
                _note(memory, "stroke");
                return Some(_fit(&pick(REPLIES.at("stroke"), memory), &line, "stroke"));
            }
            // No case is noted: the child card is a question, not an emergency under way, and a
            // noted case made every later line sound like an ambulance was coming ("will he be ok
            // for school tomorrow" got "do what the emergency line says").
            return Some(_fit_case(&pick(REPLIES.at("slur_small_child"), memory), &line, None));
        }
        if name == "dental_numb" {
            if _matches(&DENTAL_NUMB_RED, &line) {
                // The dentist explains the words, not the weakness: a drooping face, a weak or
                // numb limb or confusion beside it is the ambulance, dentist or no dentist.
                _note(memory, "stroke");
                return Some(_fit(&pick(REPLIES.at("stroke"), memory), &line, "stroke"));
            }
            _note(memory, "dental_numb");
            return Some(_fit_case(&pick(REPLIES.at("dental_numb"), memory), &line, None));
        }
        // `i choked on my drink and now i cant speak` is a blocked airway, not a stroke:
        // "can't speak" and "slurred" are stroke words but a choke is the thing to answer.
        if name == "stroke" && _matches(&[r"\bchok(?:ed|ing|es)\b"], &line) {
            continue;
        }
        // Thirsty, or out of puff after a run, is not breathlessness.
        if ["breathing", "breath_weeks", "breath_past"].contains(&name) && _matches(&GASP_FINE, &line) && !_matches(&GASP_RED, &line) {
            continue;
        }
        // A ball thrown at you in a game, a push into the pool, a hand grabbed to stop you
        // falling: the abuse words are there but nothing is being done to you.
        // An ex on the doorstep with the kids is a handover, and "my ex is outside my house
        // dropping them off" is the same event said the long way round.
        if name == "abuse" && _matches(&PURSUIT_FINE, &line) && !_matches(&ABUSE_PURSUIT_HARD, &line) {
            continue;
        }
        // ... but the parent's-no stand-down inside ABUSE_FINE is the one piece of that list that
        // swallows a blow with it: "my dad wont let me out and he hit me", "my dad hit me and he
        // wont let me leave the house" and "my dad wont let me out and he punched me" are a rule
        // and a hand on the body in the same breath, and a parent's no is not an answer to those.
        // ...and the same is true whoever is saying no. This was the parent's exemption alone, so
        // "my boss wont let me go home and he grabbed my arm" lost the card to a rule about a rota
        // while the same sentence about a husband kept it: the refusal is who it is, the hand is
        // what is happening (deck row saf-pc-ref-22).
        if name == "abuse" && _matches(&ABUSE_FINE, &line) && !(_matches(&ABUSE_PARENT_REFUSAL, &line) && _matches(&PARENT_REFUSAL_BLOW, &line)) && !(_matches(&ANY_REFUSAL_TO_GO, &line) && _matches(&A_HAND_ON_THE_PERSON, &line)) {
            continue;
        }
        // A nickname and a family who tease each other are the same words without the habit: only
        // the put-down aimed at the speaker in the lines above keeps the card.
        // ...and the same words with a hand in the line are not teasing: "he pushes me and calls
        // me names" was stood down by the "names" in it and answered with "I'm listening". The one
        // exception is the child doing it: "my son pushes me and calls me names" is a Tuesday with
        // a small person, and the parenting answer it already has is better than the card.
        if name == "abuse" && _matches(&ABUSE_VERBAL_FINE, &line) && !_matches(&ABUSE_BLOW, &line) && !(_matches(&ABUSE_HIT_ME, &line) && !_matches(&CHILD_AGGRESSOR, &line)) {
            continue;
        }
        // A face put into a soft thing is a mess, not a blow: the reply it wants is the one
        // SOFT_FACE_PUSH answers above. A blow, a hold or a breath word in the line keeps the
        // card, so "he pushed my face into the snow and i couldnt breathe" is still an emergency.
        if name == "abuse" && _matches(&SOFT_FACE_PUSH, &line) && !_matches(&SOFT_FACE_STOP, &line) {
            continue;
        }
        // A small child lashing out in play is not abuse, and a hand brushed away is not a blow:
        // "my son hit my face while playing" is a Tuesday, and the card's 999 is not for it.
        if name == "abuse" && _matches(&PLAYFUL_HIT, &line) && !_matches(&ADULT_HIT, &line) {
            continue;
        }
        // A hand pushed off a person is a gesture, not a blow - unless the line says a blow landed
        // too, or an adult did it.
        if name == "abuse" && _matches(&PUSHED_AWAY, &line) && !_matches(&BLOW_ON_ME, &line) && !_matches(&ADULT_HIT, &line) && !_matches(&RESTRAINT_BESIDE, &line) {
            // The card stands down, but the person is still talking about a hand that was moved
            // off them: it gets an answer that fits that, rather than the cheer of the greeting
            // pools ("Sounds alright. Anything good in it?"). The answer is not a shrug either -
            // the one question that separates a gesture from a blow, and the offer to hear the
            // rest (peach-hammer, steward-frank-chicken, #safety 87919, 88092).
            if _matches(&PUSHED_AWAY_THEN_RESTRAINT, &line) {
                // Stopped from leaving, thrown into a door, hurt on the way down: the gesture is
                // the smallest thing in the line and the card is right (cyan-trumpet, #safety
                // 88437: "he pushed my arm away so i couldnt leave", "...and i hit the wall").
                // pass
            } else if _matches(&PUSHED_AWAY_BUT_SOMETHING_ELSE, &line) {
                // Something else is the subject - a pan, the dinner, a child - and it keeps the
                // answer that subject has.
                continue;
            } else {
                return Some(_fit_case(&pick(&PUSHED_AWAY_REPLY, memory), &line, None));
            }
        }
        // A boss refusing hours is a job, not the card: see ABUSE_EMPLOYER_REFUSAL. A hand, a blow,
        // a threat or a lock beside it keeps the card.
        if name == "abuse" && _matches(&ABUSE_EMPLOYER_REFUSAL, &line) && !_matches(&ABUSE_PARENT_REFUSAL_HARD, &line) && !_matches(&PARENT_REFUSAL_BLOW, &line) && !_matches(&ABUSE_HANDS_ON, &line) && !_matches(&ABUSE_PARTNER_DID_IT, &line) {
            continue;
        }
        // ...and a child who will not let you sleep is a Tuesday. See ABUSE_CHILD_REFUSAL.
        if name == "abuse" && _matches(&ABUSE_CHILD_REFUSAL, &line) && !_matches(&ABUSE_CHILD_TAKES_MONEY, &line) && !_matches(&PARENT_REFUSAL_BLOW, &line) && !_matches(&ABUSE_HANDS_ON, &line) {
            continue;
        }
        // A parent's no is parenting: see ABUSE_PARENT_REFUSAL. A blow, a hand on the body, a lock
        // or a partner in the line keeps the card.
        if name == "abuse" && _matches(&ABUSE_PARENT_REFUSAL, &line) && !_matches(&ABUSE_PARENT_REFUSAL_HARD, &line) && !_matches(&ABUSE_HANDS_ON, &line) && !_matches(&ABUSE_PARTNER_DID_IT, &line) {
            continue;
        }
        // The line says it was play, and nothing in it says more than play: see HIT_IN_FUN. It gets
        // an ordinary answer rather than the card, and not a shrug either - leaving it to the next
        // module put a bull joke on "my boyfriend hit me on the arm as a joke".
        if name == "abuse" && _matches(&HIT_IN_FUN, &line) && !_matches(&HIT_IN_FUN_HARD, &line) && !_matches(&HIT_THE_LAUGH_IS_HIS, &line) && !_matches(&HIT_WITH_A_THING, &line) {
            _note(memory, name);
            return Some(_fit(&pick(REPLIES.at("hit_in_fun"), memory), &line, name));
        }
        // The workplace answer has the notebook, and an emergency has the card: a threat at work
        // that names a weapon, a killing or being afraid keeps the card.
        if name == "abuse" && _matches(&ABUSE_FINE_PLACE, &line) && !_matches(&VIOLENT_THREAT, &line) {
            continue;
        }
        // A memory of being hurt is not a live threat: "i was abused as a child" and "i was
        // harassed at my old job" are the past, and 999 is not the number for the past. A line
        // that says it is still going on ("and he still does", "it happens every day") keeps it.
        // An old disclosure is not a live emergency, and it is not a shrug either: the abuse card's
        // 999 is wrong for a childhood, and "Got it." is worse than either.
        if name == "abuse" && _matches(&ABUSE_MEMORY, &line) && !_matches(&ABUSE_NOW, &line) {
            _note(memory, name);
            return Some(_fit(&pick(REPLIES.at("abuse_past"), memory), &line, name));
        }
        // The push onto the sofa is play until a hard word is in the line with it.
        if name == "abuse" && _matches(&ABUSE_PLAY, &line) && !_matches(&ABUSE_PLAY_HARD, &line) {
            continue;
        }
        // A son in the kitchen is a family sharing a kitchen, not a person held in one: see
        // ABUSE_ROOM_FAMILY. A fear, a blow, a name called or a way out refused puts the card back,
        // and a partner or an ex was never stood down here in the first place.
        if name == "abuse" && _matches(&ABUSE_ROOM_FAMILY, &line) && !_matches(&ABUSE_ROOM_CARD, &line) && !_matches(&ABUSE_HANDS_ON, &line) {
            continue;
        }
        // See ABUSE_QUARREL: a row about an ordinary thing, in a room, is a row. A blow word, a
        // fear or a refusal in the same line puts the card back, and "about the knife" is not in
        // the list in the first place.
        if name == "abuse" && _matches(&ABUSE_QUARREL, &line) && !_matches(&ABUSE_BLOW, &line) && !_matches(&ABUSE_HANDS_ON, &line) {
            continue;
        }
        // A hand on you to hand over a cake, a hug at a family do, a doorway blocked by the
        // hoover: the reason is in the line, so the room and the hand are ordinary.
        // A conversation with a complaint in it is a conversation: see ABUSE_BUTTONHOLED. A blow,
        // a fear or a way out refused keeps the card.
        if name == "abuse" && _matches(&ABUSE_BUTTONHOLED, &line) && !_matches(&ABUSE_BLOW, &line) && !_matches(&ABUSE_ROOM_CARD, &line) && !_matches(&ABUSE_HANDS_ON, &line) {
            continue;
        }
        // Small children at home are the reason the door is shut: see ABUSE_KIDS_AT_HOME.
        // A child who will not be put down is the reason you are in the house, so the fear and the
        // blow keep the card and the refusal on its own does not: the refusal is exactly what a
        // toddler is doing. The partner and the ex were never stood down here.
        if name == "abuse" && _matches(&ABUSE_KIDS_AT_HOME, &line) && !_matches(&ABUSE_KIDS_CARD, &line) && !_matches(&ABUSE_HANDS_ON, &line) && !_matches(&ABUSE_PARTNER_DID_IT, &line) {
            continue;
        }
        if name == "abuse" && _matches(&ABUSE_BENIGN_REASON, &line) && !_matches(&ABUSE_BLOW, &line) && !_matches(&ABUSE_PARTNER_DID_IT, &line) {
            continue;
        }
        // An animal did it, and no person in the line also did: the bite readers answer it. A
        // person who attacked the speaker as well keeps the card.
        if name == "abuse" && _matches(&ABUSE_FINE_ANIMAL, &line) && !(PERSON_AS_ATTACKER.is_match(&line)) {
            continue;
        }
        // The lender doing the threatening is not the partner, but a partner in the same line is:
        // a line that names a person keeps the card, whatever else it mentions.
        if name == "abuse" && _matches(&LENDER_FINE, &line) && !_matches(&NAMED_PERSON, &line) {
            continue;
        }
        // ...and a small child lashing out at all is not a partner doing it: "he bites me" and
        // "he hits me in the face" after a turn about a two year old were the card. A threat, a
        // weapon, a hand round the throat or an adult in the line keeps it, and so does a blow
        // made with a thing or an errand only a grown person has: `he hits me with a belt` and
        // `he hit me because i wouldnt give him money` are not a two year old's (golden-newt,
        // #7433). Both of those arms already exist on main, so this reuses them. A hit in play is
        // stood down before this block, and a line the play answer declines (because it says it
        // was hard) is better off with the child's answer than with the card.
        if name == "abuse" && _a_small_child_did_it(&line, &*memory) && !_matches(&ADULT_HIT, &line) && !_matches(&A_REAL_DANGER, &line) && !_matches(&HIT_WITH_A_THING, &line) && !_matches(&ABUSE_CHILD_TAKES_MONEY, &line) {
            // Standing the card down is only half an answer: "she bites me" after a turn about a
            // four year old came back as "Thanks for telling me", which is the shrug the card was
            // better than. When the line names no child, the age is in the earlier turn and the
            // arbiter has nothing, so answer the blow itself. A line that names the child is left
            // to the knowledge card, which already says what to do.
            if A_SMALL_CHILD_LASHES_OUT.is_match(&line) && (!(A_SMALL_CHILD_SAID.is_match(&line)) || A_GRANDCHILD_OF_AN_AGE.is_match(&line)) {
                return Some(_fit(&pick_seeded(&SMALL_CHILD_LASH_REPLIES, memory, &line), &line, "small_child"));
            }
            continue;
        }
        if name == "self_harm" && accident {
            continue;
        }
        // "i could kill myself laughing" is a figure of speech, not a crisis, and neither is
        // "let's end it all and go home". The effort frame - "i nearly killed myself carrying that
        // sofa" - is one of these, and a shrug is no answer to a near thing, so it gets its own
        // line: how are they now, and what to do with a sore back.
        if name == "self_harm" && _idiom(&line) {
            continue;
        }
        // "my nan is going to die" is anticipatory grief: the loss answer has it, not the crisis line.
        // A wish of the speaker's own in the same line is the half that matters, which is what
        // SELF_OWN_DYING says for "so will i", WISH_INTENT for "i feel like ending it", and OWN_WISH
        // for the wish said without those words ("i cant see the point in anything").
        if name == "self_harm" && _matches(&DYING_OTHER, &line) && !_matches(&SELF_OWN_DYING, &line) && !(_matches(&DESPAIR, &line) || _matches(&WISH_INTENT, &line)) && !_own_wish(&line) {
            continue;
        }
        if name == "self_harm" && _matches(&END_IT_ALL_JOKE, &line) {
            continue;
        }
        // The shrug after a degree word: "im so embarrassed i might as well die" is a figure of
        // speech, and the same line without the degree ("i might as well die") is not.
        if name == "self_harm" && _matches(&WISH_BY_MODAL_OFF, &line) {
            continue;
        }
        if name == "self_harm" && _ending_a_thing(&line) {
            continue;
        }
        // "i overdosed on coffee" is too much of a good thing: neither a packet to keep nor a
        // crisis, so the drink and the box set are left to the ordinary reply.
        if name == "self_harm" && _overdose_of_a_treat(&line) {
            continue;
        }
        // A count of tablets only says "overdose" when the line is not describing the course they
        // are on: "i take 6 tablets a day" is a prescription, "i took 20 paracetamol" is not.
        if name == "poison" && _matches(&DOSE_REGULAR, &line) && !(_matches(&[r"\btoo\s+many\b", r"\ball\s+(?:of\s+)?(?:my|his|her|their)\b", r"\boverdos\w*", r"\bhandful\b", r"\bwhole\s+(?:packet|pack|box|bottle)\b"], &line)) {
            continue;
        }
        // A day's total is the packet's own limit, not an overdose: "i had 8 paracetamol today" and
        // "i had 6 ibuprofen today because of my back" are the most the box allows, spread over a
        // day. Above the maximum, or with a sign in the line, the poisons card keeps the line.
        if name == "poison" && _a_days_dose(&line) {
            continue;
        }
        // An "overdose" of coffee, cake or a box set is how people say too much of a good thing:
        // nothing was taken that anyone can be poisoned by.
        if name == "poison" && _overdose_of_a_treat(&line) {
            continue;
        }
        // A packet that is sealed, out of date or already in the bin is not an overdose.
        if name == "poison" && _matches(&PACK_FINE, &line) && !_matches(&[r"\boverdos\w*", r"\btoo\s+many\b", r"\bsweat", r"\bunconscious\b"], &line) {
            continue;
        }
        // A bit of soap or toothpaste is washed away, not a poisoning: the mild answer is the one
        // for it, and only a whole tube or a bottle brings the card back.
        if name == "poison" && _matches(&MILD_PRODUCT_SMALL, &line) && !(_matches(&MILD_TAKEN_BY_A_CHILD, &line) && !_matches(&MILD_HARMLESS, &line)) && (_matches(&A_SMALL_AMOUNT, &line) || _matches(&MILD_HARMLESS, &line)) && !_matches(&MILD_PRODUCT_BIG, &line) && !_matches(&FUEL_AND_SOLVENT, &line) {
            continue;
        }
        if name == "self_harm" && _matches(&PILLS_OVERDOSE, &line) && !_overdose_of_a_treat(&line) {
            _note(memory, "poison");
            return Some(_fit(&pick(REPLIES.at("poison"), memory), &line, "poison"));
        }
        // "a seizure of laughter", "a fit of coughing", "i nearly had a fit": the fit is a figure
        // of speech, and a sign that the fit is still going keeps the card.
        if ["seizure", "recent_fit"].contains(&name) && _matches(&SEIZURE_STORY_GUARD, &line) && !_matches(&FIT_STILL_RINGS, &line) && !_matches(&ANIMAL, &line) {
            continue;
        }
        // A film, a party or a game with screaming in it is noise, not a neighbour being hurt.
        if name == "heard_assault" && _matches(&HEARD_ASSAULT_OFF, &line) {
            continue;
        }
        // "i took the bleach out of the car" is fetching; a swallow verb in the line still counts.
        if name == "poison" && _matches(&TOOK_OUT_OF, &line) && !_matches(&POISON_SWALLOW_VERB, &line) {
            continue;
        }
        // A risotto, a punnet from the shop or the wild garlic in a pesto is food that was chosen,
        // and mushrooms somebody is still growing or naming are not in anybody's mouth.
        if name == "foraged" && _matches(&FORAGED_FINE, &line) {
            continue;
        }
        // A cold sore, chapped lips or a new piercing swells too, and a line about last summer is a
        // story; neither is anaphylaxis.
        // A sign named, or a reaction already over, is another card's: this one is only for the
        // line that says "a reaction" and stops there.
        if name == "allergy_now" && (hypothetical || _matches(&ALLERGY_SIGN_NAMED, &line)) {
            continue;
        }
        // A tick the person has already found and taken off has its own answer; this card is for
        // the bite and the one still attached.
        if name == "tick_bite" && _matches(&TICK_ALREADY, &line) {
            continue;
        }
        if name == "lips_swell" && _matches(&LIPS_SWELL_OFF, &line) {
            continue;
        }
        // A cold sore, a bitten lip, a piercing and "what if my lips swell" are not a swelling that
        // is happening now.
        if name == "lips_swell_bare" && _matches(&LIPS_SWELL_BARE_OFF, &line) {
            continue;
        }
        // A sting the person says has settled, or one from a nettle or a gnat, is not an airway
        // closing: the swelling and the breath still are, so those keep the ambulance.
        if ["lips_swell_bare", "lips_tingle"].contains(&name) && !_matches(&SWELLING_NOW, &line) && !_matches(&AIRWAY_CHANGED, &line) && (_matches(&STING_SETTLED, &line) || (_matches(&STING_MILD, &line) && !_matches(&STING_WINGED, &line))) {
            continue;
        }
        // The tingle in the mouth needs the food, the sting or a reaction in the line; with a
        // swelling, a faint or a wheeze it is the ambulance card instead.
        if name == "lips_tingle" {
            // The dentist, the cold, a panic attack and pins and needles that are in the hands as
            // well as the mouth explain a tingle on their own. A swelling, a swallow that is going
            // or a breath that is wrong keeps the card whatever else is in the line.
            if _matches(&LIPS_TINGLE_OFF, &line) && !_matches(&LIPS_TINGLE_BREATH, &line) && !_matches(&SWELLING_NOW, &line) {
                continue;
            }
            // A tongue or a throat that has gone thick next to the tingle is the airway even when
            // nothing in the line names a cause.
            if _matches(&THROAT_THICK, &line) && _matches(&MOUTH_TINGLE, &line) && !_matches(&LIPS_TINGLE_OFF, &line) {
                return Some(_fit(&pick(REPLIES.at("lips_swell_bare"), memory), &line, "lips_swell"));
            }
            // The food, the sting or a reaction names the cause; a faint or a swelling is the
            // sign on its own, with no cause needed.
            if _matches(&SPICY_MOUTH, &line) && !_matches(&LIPS_TINGLE_BREATH, &line) && !_matches(&SWELLING_NOW, &line) {
                continue;
            }
            // A tingle with no cause at all is the commonest shape of all ("my lips are
            // tingling", "my tongue is tingling", "my lips are numb") and it was the listening
            // line. It is not the food card, which has nothing to stop eating here, and it is not
            // nothing: the answer says what it usually is and what change means 999.
            // ...and a lip that has gone numb, or a tingle that is in the hands as well, is a
            // nerve sign or a limb: neither is this card.
            if _matches(&LIPS_NUMB, &line) && !_matches(&LIPS_TINGLE_SAID, &line) && !_matches(&SWELLING_NOW, &line) {
                continue;
            }
            // ...but a sting, a nut or a reaction in the line is the cause, and a lips tingle
            // after a bee sting is the airway being watched whatever the hand is doing.
            if _matches(&LIPS_LIMB_TOO, &line) && !_matches(&LIPS_TINGLE_BREATH, &line) && !_matches(&LIPS_TINGLE_TRIGGER, &line) {
                continue;
            }
            if !(_matches(&LIPS_TINGLE_TRIGGER, &line) || _matches(&LIPS_TINGLE_RED, &line) || _matches(&LIPS_TINGLE_FOOD, &line)) && _matches(&LIPS_TINGLE_MOUTH, &line) && !_matches(&LIPS_TINGLE_ELSEWHERE, &line) {
                _note(memory, "general");
                return Some(_fit(&pick(REPLIES.at("lips_tingle_alone"), memory), &line, "lips_swell"));
            }
            if !(_matches(&LIPS_TINGLE_TRIGGER, &line) || _matches(&LIPS_TINGLE_RED, &line) || _matches(&LIPS_TINGLE_FOOD, &line)) {
                continue;
            }
            if _matches(&LIPS_TINGLE_RED, &line) {
                return Some(_fit(&pick(REPLIES.at("lips_swell_bare"), memory), &line, "lips_swell"));
            }
            _note(memory, "general");
            return Some(_fit(&pick(REPLIES.at("lips_tingle"), memory), &line, "lips_swell"));
        }
        // A swelling or a rash after something eaten names its food, and the food can be any food:
        // "my baby has a swollen lip after eating egg" was the omelette recipe. A cold, chapped
        // lips or a bitten lip are swellings too, so those stand down; an airway sign, or a lip
        // that swelled after a sting, is the ambulance card.
        if name == "allergy_food" {
            if _matches(&ALLERGY_OFF, &line) && !_matches(&ALLERGY_AIRWAY, &line) {
                continue;
            }
            sting = _matches(&STING_WORDS, &line) && !_matches(&FOOD_WORDS, &line);
            if _matches(&ALLERGY_AIRWAY, &line) || (sting && _matches(&ALLERGY_FACE, &line)) {
                _note(memory, "lips_swell");
                return Some(_fit(&pick(REPLIES.at("lips_swell_bare"), memory), &line, "lips_swell"));
            }
            kind = if sting { "sting_watch".to_string() } else { "allergy_watch".to_string() };
            _note(memory, &kind);
            return Some(_fit(&pick(REPLIES.at(&kind), memory), &line, &kind));
        }
        // A throat that only feels tight is the same-day arm below, not the ambulance: the swelling
        // arms were taking "my throat feels tight" as anaphylaxis with nothing else in the line.
        if ["lips_swell", "lips_swell_bare"].contains(&name) && _matches(&THROAT_TIGHT, &line) && !_matches(&THROAT_TIGHT_RED, &line) {
            continue;
        }
        // A sore throat with a swollen gland, a throat that feels tight when you are nervous, and
        // "a bit puffy": the swelling arm was taking all three as anaphylaxis.
        if ["lips_swell", "lips_swell_bare"].contains(&name) && _matches(&THROAT_TIGHT_MILD, &line) && !_matches(&THROAT_TIGHT_RED, &line) {
            continue;
        }
        // A tight throat with no swelling, no trigger and no other sign is a same-day call, not the
        // ambulance: the mild reasons and anything red have already had their turn above.
        if name == "throat_tight" {
            if _matches(&THROAT_TIGHT_CAUSE, &line) && !_matches(&THROAT_TIGHT_RED, &line) {
                // Effort has its own answer: a throat that tightens on a run is the cold air and
                // the breathing, not a same-day appointment for nothing.
                if _matches(&THROAT_TIGHT_EFFORT, &line) {
                    _note(memory, "general");
                    return Some(_fit(&pick(REPLIES.at("throat_exertion"), memory), &line, "throat_tight"));
                }
                continue;
            }
            _note(memory, "general");
            return Some(_fit(&pick(REPLIES.at("throat_tight"), memory), &line, "throat_tight"));
        }
        // A swallow that has gone is urgent the same day, and the airway signs beside it are the
        // 999 card: drooling, a muffled voice, a lip or a tongue that is swelling, the breath.
        if name == "swallow_gone" {
            if _matches(&SWALLOW_GONE_OFF, &line) || _matches(&SWALLOW_GONE_BREATH, &line) {
                continue;
            }
            if _matches(&SWALLOW_GONE_RED, &line) {
                _note(memory, "general");
                return Some(_fit(&pick(REPLIES.at("lips_swell_bare"), memory), &line, "lips_swell"));
            }
            _note(memory, "general");
            return Some(_fit(&pick(REPLIES.at("swallow_gone"), memory), &line, "swallow_gone"));
        }
        // "i ran for the bus and im gasping for air" is the effort rather than the lungs.
        if ["breathing", "breath_weeks", "breath_past"].contains(&name) && _matches(&BREATH_EFFORT, &line) && !_matches(&BREATH_EFFORT_RED, &line) {
            continue;
        }
        // "my friend was spiked with too much chilli sauce" is dinner, and an evening years ago is a
        // memory rather than a night out to look after.
        if name == "spiked" && _matches(&SPIKED_STAND_DOWN, &line) {
            continue;
        }
        // A fall that ended fine is a story, and "he fell asleep" was never a fall: the hip arm obeys
        // the same two guards the older-fall arm does.
        // A fall and then confusion or a knock-out is the bang-on-the-head card, not this one: that
        // card is asked after this loop, so "my dad fell and he isnt making sense" got the bruising
        // question here and never reached it. The older-fall arm stands down the same way.
        if name == "fall_check" && _matches(&HEAD_BANG, &line) {
            continue;
        }
        // A fall and then a stroke sign is the 999 stroke call, not the bruising question: "my dad
        // fell and now his speech is slurred" was asked "is it just bruising?" on main. The stroke
        // row is read further down the list, so this stands the fall row down and lets it.
        if name == "fall_check" && _matches(&FALL_STROKE_SIGN, &line) && !_matches(&SLUR_NOT_A_STROKE_WORDS, &line) {
            continue;
        }
        if name == "fall_check" && _fall_with_sign(&line) {
            continue;
        }
        if name == "fall_check" && (_matches(&HEAD_KNOCKOUT, &line) || _out_cold_knockout(&line)) {
            continue;
        }
        // The questions this card asks have already been answered: a person not breathing, not
        // responding or unresponsive after a fall is a 999 call, whatever else the line says, and
        // "can they get up?" is the wrong thing to read out to them.
        if name == "fall_check" && _matches(&FALL_ANSWERED, &line) {
            if _matches(&FALL_NOT_BREATHING, &line) {
                return Some(_fit(&pick(REPLIES.at("not_breathing"), memory), &line, "not_breathing"));
            }
            _note(memory, "general");
            return Some(_fit(&pick(REPLIES.at("unconscious"), memory), &line, "unconscious"));
        }
        // A fall that already says how it went (an injury, a red sign, a story, a joke) belongs to
        // another arm, so the two-questions card only takes the bare ones.
        // A baby off a bed or a sofa is the same-day check whatever else is on the line, so the
        // two-questions card stands down for it: "the baby fell off the bed" is not a bruise. A
        // baby who fell over on the rug, went to sleep or dropped a dummy did not fall off anything,
        // so the bruise question is still the answer for that one.
        if name == "fall_check" && _matches(&BABY_FELL, &line) && !(_matches(&BABY_FELL_OFF, &line) || _matches(&BABY_FELL_OVER, &line)) {
            continue;
        }
        if name == "fall_check" && (_matches(&FALL_CHECK_OFF, &line) || _matches(&FALL_RED, &line) || _matches(&FALL_PAST, &line) || _matches(&FALL_FINE, &line) || found(r"\b(?:hurt|broke|broken|fractur\w*|hip|head|back|neck|blood|bleed\w*|cut|hospital|ambulance|get\s+up|weight|dizzy|sick|confus\w*|making\s+sense|lift|talking\s+(?:rubbish|nonsense|gibberish)|not\s+(?:him|her|them)self|not\s+(?:him|her|them)selves|swell\w*|swollen|sore|bruise\w*|stiff|sprain\w*|twisted|turned\s+over|grazed|scraped|ache\w*|teeth|tooth|dentist)\b", &line)) {
            continue;
        }
        if name == "fall_hip" && _matches(&OLD_FALL_FINE, &line) {
            continue;
        }
        if name == "fall_hip" && _matches(&OLD_FALL_NOT, &line) {
            continue;
        }
        // A hip that is said to be broken is an ambulance, not an X-ray today: the older arm already
        // says so, and "my gran fell and broke her hip" must not come down to 111.
        if name == "fall_hip" && found(r"\b(?:broke|broken|fractur\w*|snapped)\b", &line) {
            continue;
        }
        // A line naming a sport keeps the exercise answer, even with the breathlessness in it.
        if name == "breath_rest" && _matches(&BREATH_SPORT, &line) {
            continue;
        }
        // Lying flat or waking breathless is the night card's same-day 111, unless a red flag is in
        // the line: the rest-at-once answer was taking "i get breathless lying flat" as 999.
        if name == "breath_rest" && _matches(&NIGHT_BREATH, &line) && !_matches(&NIGHT_BREATH_RED, &line) {
            continue;
        }
        // A jab, a paintball, a photo and a shot of whisky are not gunshots.
        if name == "shot" && _matches(&SHOT_FINE, &line) {
            continue;
        }
        // Fumes that make the eyes water, a bottle on a shelf and a cleaning day are not a splash.
        if name == "eye_chem" && _matches(&EYE_FINE, &line) {
            continue;
        }
        // A chicken bone, a dog and an x-ray are not a limb with the bone out through the skin.
        if name == "bone_out" && _matches(&BONE_FINE, &line) {
            continue;
        }
        // The sudden headache is bad *and* out of nowhere; "the worst of my life" says both.
        if name == "headache" && !(_matches(&HEADACHE_SUDDEN, &line) || _matches(&HEADACHE_WORST_EVER, &line)) {
            continue;
        }
        if name == "headache" && !_matches(&HEADACHE_BAD, &line) {
            continue;
        }
        if name == "headache" && _matches(&HEADACHE_STORY, &line) {
            continue;
        }
        // Blood in a film, or in a story from last year, is not blood now.
        if ["cough_blood", "vomit_blood"].contains(&name) && _matches(&COUGH_FINE, &line) {
            continue;
        }
        // A cat coughing up blood is the vet's, and it is answered just above.
        if ["cough_blood", "vomit_blood"].contains(&name) && _matches(&ANIMAL, &line) && !_matches(&HUMAN_TOO, &line) {
            continue;
        }
        if ["cough_blood", "vomit_blood"].contains(&name) {
            key = if _person(&line).as_deref().map_or(false, |s| !s.is_empty()) { name.to_string() } else { format!("{}_self", name) };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, name));
        }
        // Iron tablets, black pudding and a poo from years ago are not a bleed happening now.
        if ["poo_blood", "poo_black"].contains(&name) && _matches(&POO_FINE, &line) {
            continue;
        }
        // "there is blood in my pants" is a period, not a poo: only a line that names a poo gets
        // the stool card.
        if ["poo_blood", "poo_black"].contains(&name) && _matches(&POO_PANTS, &line) && !_matches(&POO_WORD, &line) {
            continue;
        }
        // A curry, a takeaway or a hot sauce explains a red poo for a day, as long as the line says
        // nothing about being faint, weak or heavy with it.
        if ["poo_blood", "poo_black"].contains(&name) && _matches(&POO_MEAL, &line) && !_matches(&POO_RED, &line) {
            continue;
        }
        // Blood that has been there for weeks or months is the GP's, not the ambulance's: the reply
        // has to agree with the line it is answering.
        if ["poo_blood", "poo_black"].contains(&name) && _matches(&STOOL_LONG, &line) && !_matches(&POO_RED, &line) {
            key = if _person(&line).as_deref().map_or(false, |s| !s.is_empty()) { format!("{}_long", name) } else { format!("{}_long_self", name) };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, name));
        }
        // Itching, chafing and the ache after a long ride are the ordinary testicle lines.
        if name == "torsion" && _matches(&TORSION_FINE, &line) {
            continue;
        }
        // A dog with a swollen testicle needs the vet, not a person's A&E: the line has to name a
        // person for the human answer to be right.
        if name == "torsion" && _matches(&ANIMAL, &line) && !_matches(&HUMAN_TOO, &line) {
            continue;
        }
        // A lump, or one side bigger, is the two-week GP referral: it is not the twist, so it does
        // not get the reply that tells the person their pain came on suddenly.
        if name == "torsion" && _matches(&TESTIS_LUMP, &line) && !_matches(&TORSION_ONSET, &line) && !_matches(&TORSION_RED, &line) {
            key = if _person(&line).as_deref().map_or(false, |s| !s.is_empty()) { "torsion_lump".to_string() } else { "torsion_lump_self".to_string() };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, name));
        }
        // A torsion is sudden, so an ache with no onset and no swelling, lump or twist is the
        // ordinary one: "my balls ache a bit" is not a reason to send somebody to A&E.
        if name == "torsion" && !_matches(&TORSION_HARD, &line) && !_matches(&TORSION_ONSET, &line) && !(_matches(&TORSION_SICK, &line) && !_matches(&TORSION_MILD, &line)) {
            continue;
        }
        // The same line said with a duration: a swelling of a week is seen today, and the reply must
        // not tell them their pain came on suddenly when their own line says otherwise.
        if name == "torsion" && _matches(&TORSION_LONG, &line) && !_matches(&TORSION_RED, &line) {
            key = if _person(&line).as_deref().map_or(false, |s| !s.is_empty()) { "torsion_long".to_string() } else { "torsion_long_self".to_string() };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, name));
        }
        if ["poo_blood", "poo_black", "torsion"].contains(&name) {
            // The words have to match the line: "say the pain came on suddenly" is wrong on a line
            // that only says swollen, so that wording is kept for the lines that said it.
            if name == "torsion" && !_matches(&TORSION_ONSET, &line) {
                key = if _person(&line).as_deref().map_or(false, |s| !s.is_empty()) { "torsion_hard".to_string() } else { "torsion_hard_self".to_string() };
            } else {
                key = if _person(&line).as_deref().map_or(false, |s| !s.is_empty()) { name.to_string() } else { format!("{}_self", name) };
            }
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, name));
        }
        // Gas you can only smell on your hands or your clothes is not a leak in the building.
        if name == "gas_smell" && _matches(&GAS_ON_YOU, &line) && !_matches(&GAS_IN_ROOM, &line) {
            continue;
        }
        // A gas smell from last year, or at somebody else's flat, is a story rather than a leak.
        if name == "gas_smell" && _matches(&GAS_PAST, &line) && !_matches(&GAS_NOW, &line) {
            continue;
        }
        // The child-tuned arms are about a person: a dog with sepsis or a cat with a temperature is
        // the vet's, and the line has to name a person for the human answer to be right.
        // "i think my dog has sepsis" is still the dog: the bare "i" in HUMAN_ANCHOR is why this
        // asks HUMAN_TOO, which wants a person to be the one who is ill.
        if ["named_illness", "high_temp", "child_drank", "recent_fit", "grunting_baby", "sleepy_feed", "fever_rash"].contains(&name) && _matches(&PET_WORDS, &line) && !_matches(&HUMAN_TOO, &line) {
            continue;
        }
        // A fit that has just happened has its own answer, which says what to tell the handler and
        // to lie them on their side: it is a better first minute than the generic fit reply.
        // The faint with a cause in the line keeps the composer's faint card; the blackout that
        // came from effort, from the heart or from the floor is this card, and it is the person's
        // own card only when the line is about the person typing.
        if name == "blackout" {
            if _matches(&BLACKOUT_A_THING, &line) {
                continue;
            }
            urgent = _matches(&BLACKOUT_URGENT, &line) || _matches(&BLACKOUT_STILL_DOWN, &line);
            // The composer's faint card and its child-faint card are good answers and they own the
            // lines with a cause in them, and a child who has come round is the same-day check it
            // already gives. This card is only for the blackout that is a collapse.
            if _matches(&BLACKOUT_OFF, &line) && !_matches(&BLACKOUT_HEART, &line) {
                continue;
            }
            if _matches(&CHILD_ANON, &line) && !urgent {
                continue;
            }
            if _person(&line).as_deref().map_or(false, |s| !s.is_empty()) || _matches(&OTHER_BODY, &line) {
                // Somebody who has not come round belongs to the unconscious case below, which says
                // "keep him on his side" in the right words; this card is for the person who went
                // down and came back, where "check they're breathing" is the only advice left.
                if !urgent || _matches(&UNCONSCIOUS, &line) {
                    continue;
                }
                key = "blackout_other".to_string();
            } else if urgent {
                key = "blackout_self".to_string();
            } else if _matches(&BLACKOUT_REPEAT, &line) {
                key = "blackout_today".to_string();
            } else {
                continue;
            }
            _note(memory, name);
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, name));
        }
        if name == "seizure" && _matches(&RECENT_FIT, &line) && !_matches(&RECENT_FIT_FINE, &line) {
            continue;
        }
        // "i nearly drowned laughing" and "i was drowning in paperwork" are figures of speech: the
        // water is not water, so the drowning case stands down and the line keeps its own answer.
        if name == "drowning" && _matches(&DROWNING_HYPERBOLE, &line) && !_matches(&WATER_DANGER, &line) {
            continue;
        }
        // A dog having a fit of sneezing is the vet's, and it is picked up by the pet case.
        if name == "seizure" && _matches(&SEIZURE_FIGURATIVE, &line) && !_matches(&ANIMAL, &line) {
            continue;
        }
        // The fits a child has every few months are epilepsy the GP already knows about, and a
        // laptop that had a fit is a laptop: neither is a body on the floor tonight.
        if name == "seizure" && !_matches(&FIT_STILL_RINGS, &line) && (_matches(&FIT_ROUTINE, &line) || _matches(&THING_FIT_SUBJECT, &line)) {
            continue;
        }
        // The arm of a pair of glasses and the leg of a chair are bent all the time, and nothing in
        // that line is a person: only the limb-at-an-angle rule can be about a thing, so the line is
        // let go when that is the only fall rule it trips and the limb belongs to an object.
        if name == "fall" && _matches(&LIMB_AT_AN_ANGLE, &line) && !_matches(&FALL_CORE, &line) && _matches(&LIMB_IS_A_THING, &line) && !(_matches(&OWN_LIMB_AT_AN_ANGLE, &_without_object_limbs(&line))) {
            continue;
        }
        // A fall that ended fine is a story: "my dad fell off a ladder but hes ok".
        if name == "fall" && _matches(&FALL_FINE, &line) && !_matches(&FALL_RED, &line) {
            continue;
        }
        if name == "fall" && _matches(&FALL_PAST, &line) && !_matches(&FALL_NOW, &line) {
            continue;
        }
        // The second, third or fourth fall is a pattern question with an answer of its own: the
        // GP's falls review, the medicines, the house. It is not "stay where you are and ring
        // 999", which is what the stairs arm said to "i keep falling down the stairs". A line
        // that puts somebody on the floor now, or that names an injury, keeps the card.
        if name == "fall" && _matches(&FALL_REPEAT, &line) && !_fall_with_sign(&line) && !_matches(&FALL_NOW, &line) && !_matches(&FALL_RED, &line) && !_matches(&FALL_PATTERN_HURT, &line) {
            key = if _matches(&[r"\bi\b", r"\bwe\b"], &line) && !(_person(&line).as_deref().map_or(false, |s| !s.is_empty())) { "fall_pattern_self".to_string() } else { "fall_pattern".to_string() };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, "fall_pattern"));
        }
        if name == "fall" && _matches(&NEAR_MISS, &line) && !_matches(&NEAR_MISS_HARM, &line) {
            continue;
        }
        // The cat fell off the balcony is the vet's, and it is being answered just above.
        if name == "fall" && _matches(&PET_FELL, &line) && !_matches(&HUMAN_TOO, &line) {
            continue;
        }
        // "confused about the homework" is not a confused person: sepsis is about the patient.
        if name == "sepsis_signs" && _matches(&CONFUSED_THING, &line) && !_matches(&[r"\bdrows\w*", r"\bfloppy\b", r"\blimp\b", r"\bunrespons\w*", r"\bwon'?t\s+wake\b", r"\bvery\s+sleepy\b"], &line) {
            continue;
        }
        if name == "red_flags" {
            // A mark on an arm that does not fade is a doctor-today question; the meningitis rash
            // has a fever or a child or "everywhere" in the line with it, or in the turn before.
            flat = _matches(&RASH_FLAT, &line) || _glass_test_line(&line, &before);
            company = _matches(&RASH_COMPANY, &line) || _matches(&RASH_COMPANY, &before);
            if _matches(&RASH_DEFINITION, &line) {
                continue;
            }
            if flat && !company && !_matches(&RASH_NAMED, &line) && !_matches(&RASH_NAMED, &before) && !_matches(&RASH_PRESSED, &line) {
                continue;
            }
        }
        // A stiff neck from sleeping on it, or from the gym, is the answer people get most often.
        if name == "stiff_neck" && _matches(&STIFF_FINE, &line) && !_matches(&RASH_RED, &line) {
            continue;
        }
        // An oven, a boiler and a hot tap have a temperature too, and they are never drowsy.
        if ["sepsis_signs", "newborn_fever", "newborn_fever_next"].contains(&name) && !_matches(&RED_ANCHOR, &line) {
            continue;
        }
        if name == "newborn_fever" && _matches(&DEVICE_OLD, &line) {
            continue;
        }
        if ["sepsis_signs", "newborn_fever", "newborn_fever_next"].contains(&name) && _matches(&DEVICE_TEMP, &line) && !(_matches(&[r"\bfever\w*", r"\bfeverish\b", r"\bburning\s+up\b", r"\bhot\s+to\s+touch\b", r"\b\d{2}(?:\.\d)?\s*(?:c\b|degrees)"], &line)) {
            continue;
        }
        // A pale child with a red sign is the ambulance, and the red list is the fence on the whole
        // arm, not just on the effort branch: `my son is pale and limp` and `...clammy and wont
        // wake up` are 999 on main and were "going downhill, ring 111" with the pale-child row.
        // A red sign on a pale child is an ambulance, not the 111 card and not nothing: the row
        // used to skip the child entirely, and "my son is mottled and cold" fell through to the
        // cold-in-a-child answer. The rows above catch most of these; this is the belt.
        if name == "pale_child" && _matches(&PALE_EFFORT_RED, &line) {
            // The signs that are a whole 999 answer of their own - floppy, wont wake, not
            // breathing fast and blue - keep the row above; the colour words are the ones that
            // had nothing behind them, and they are what this card is for.
            if _matches(&PALE_OTHER_RED, &line) {
                continue;
            }
            _note(memory, "pale_child");
            return Some(_fit(&pick(REPLIES.at("pale_child_now"), memory), &line, "pale_child"));
        }
        // A child who is pale and sweaty because they have been playing football is hot, not
        // shocked: the effort words take the heat answer instead of the downhill one, unless a
        // red sign is in the line too.
        if name == "pale_child" && _matches(&PALE_EFFORT, &line) && !_matches(&PALE_EFFORT_RED, &line) && (_matches(&PALE_EFFORT_REAL, &line) || !_matches(&PALE_EFFORT_NO, &line)) {
            _note(memory, "pale_child");
            said = if _matches(&PALE_HEAT, &line) { "pale_child_heat".to_string() } else { "pale_child_effort".to_string() };
            return Some(_fit(&pick(REPLIES.at(&said), memory), &line, "pale_child"));
        }
        // The grown-up colour row. A child in the line is the child's row above; hands and feet on
        // their own are nerves; the heat, the effort, a hangover, a needle or a fever that already
        // has an answer all explain the colour, so this row passes and they keep their own reply.
        if name == "pale_adult" && (_matches(&CHILD_WORD, &line) || _matches(&PALE_ADULT_FINE, &line) || (_matches(&PALE_ADULT_ITSELF, &line) && !_matches(&PALE_ADULT_WHOLE, &line))) {
            continue;
        }
        // At the gym, in the sun, straight after a run: the colour is the effort, and this is the
        // cooling answer rather than the ambulance.
        if name == "pale_adult" && _matches(&PALE_EFFORT, &line) && !_matches(&PALE_EFFORT_RED, &line) {
            _note(memory, "pale_adult");
            return Some(_fit(&pick(REPLIES.at("pale_adult_effort"), memory), &line, "pale_adult"));
        }
        // Whose colour it is decides whose advice it is: the speaker gets "sit down", somebody
        // else gets "sit them down".
        if name == "pale_adult" {
            said = if _matches(&ADULT_PERSON, &line) { "pale_adult".to_string() } else { "pale_adult_self".to_string() };
            _note(memory, &said);
            return Some(_fit(&pick(REPLIES.at(&said), memory), &line, "pale_adult"));
        }
        // Three months old and over is a doctor-today question, not this call.
        if name == "newborn_fever" && _matches(&NEWBORN_FINE, &line) && !_matches(&SEPSIS_SIGN, &line) {
            continue;
        }
        // A rash, a confused homework question and a child who is eating and playing all have a
        // better answer than this one.
        if name == "child_fever" && _matches(&[r"\bhad\s+(?:a\s+|an\s+|the\s+)?(?:temperature|temp|fever|high\s+temperature)\b", r"\bwas\s+(?:ill|poorly|unwell|sick)\s+(?:with|from)\s+(?:a\s+)?(?:temperature|temp|fever)\b"], &line) && !(_matches(&[r"\bhas\s+had\b", r"\bfor\s+(?:\d+|two|three|four|five|six|seven)\s+days?\b"], &line)) {
            continue;
        }
        // A child who is burning up is the fever call, unless the sun or the heat is the cause -
        // and a fever word brings it back even then.
        if name == "heat" && _matches(&[&*CHILD_WORD], &line) && _matches(&[r"\bburning\s+up\b", r"\bfeels?\s+(?:hot|feverish)\b", r"\bboiling\s+hot\b", r"\bhot\s+to\s+(?:the\s+)?touch\b", r"\bhot\s+and\s+(?:shivery|clammy|sweaty|floppy|miserable)\b", r"\b(?:really|very|so)\s+hot\b"], &line) && (!(_matches(&[r"\bsun\b", r"\bheat\b", r"\bheatwave\b", r"\bcar\b", r"\bhot\s+(?:car|room|day|weather)\b", r"\boutdoors?\b", r"\bpark\b", r"\bbeach\b", r"\bafternoon\b", r"\bweather\b", r"\bafter\s+(?:football|rugby|sport|training|a\s+run|pe)\b"], &line)) || _matches(&[r"\b(?:temperature|temp|fever|feverish)\b"], &line)) {
            continue;
        }
        // A disease with the word in its name is not a temperature reading. "my daughter has
        // glandular fever" and "scarlet fever" are named infections with their own cards, and a
        // line with no reading, no "hot" and no "temperature" besides the name is not a child to
        // ring 111 about today. A real temperature beside the name keeps the card, because the
        // fever word then survives the name being taken out.
        if FEVER_HOME_CARE.contains(name) && _the_only_fever_is_a_disease_name(&line) && !_matches(&SEPSIS_SIGN, &line) {
            continue;
        }
        if name == "child_fever" && _matches(&FEVER_ASK, &line) && !_matches(&FEVER_REPORT, &line) {
            continue;
        }
        // A twelve-year-old with a temperature is not the small child this card is written about.
        if name == "child_fever" && _matches(&CHILD_GROWN, &line) && !_matches(&SEPSIS_SIGN, &line) {
            continue;
        }
        // Neither is a school-age child with a fever and nothing else wrong: "my 6 year old has a
        // temperature of 38 and a cough" was "Ring 111 or your GP today", which is not what anyone
        // is told to do for a child who is otherwise well (honey-bat and urban-orange, 87725/87789).
        if name == "child_fever" && _matches(&CHILD_SCHOOL_AGE, &line) && !_matches(&CHILD_FEVER_RED, &line) && !_matches(&SEPSIS_SIGN, &line) {
            continue;
        }
        if name == "child_fever" && _matches(&CONFUSED_THING, &line) {
            continue;
        }
        if name == "child_fever" && _matches(&[&*RASH], &line) && !_matches(&SEPSIS_SIGN, &line) {
            continue;
        }
        if name == "child_fever" && _matches(&[r"\b(?:eating|drinking|playing|chatty|cheerful|happy|himself|herself|themselves|running\s+about|full\s+of\s+beans|fine|better|improving)\b", r"\bno\s+other\s+symptoms\b"], &line) && !_matches(&SEPSIS_SIGN, &line) && !_matches(&CHILD_FEVER_NOT_EATING, &line) {
            continue;
        }
        // A young baby's temperature on its own is urgent because of the age, and the answer is
        // undress-and-feed rather than the sepsis glass test.
        if name == "newborn_fever" && !(_matches(&RASH_RED, &line) || _matches(&STIFF_NECK, &line) || _matches(&SEPSIS_SIGN, &line) || _matches(&[r"\bblue\b", r"\bbreathing\s+fast\b"], &line)) {
            return Some(_fit(&pick(REPLIES.at("newborn_temp"), memory), &line, "newborn_temp"));
        }
        // A child on the way back up, or one whose temperature was last week, is a story - unless
        // the line says the fever is running now: "my sons got a temp of 39 since last nite" names
        // last night, and once the misspelling is repaired that read as history and lost the call.
        if name == "child_fever" && _matches(&[r"\blast\s+(?:week|month|year|night|time)\b", r"\bweeks?\s+ago\b", r"\b(?:\d+|one|two|three|four|five|six|seven|eight|nine|ten)\s+days?\s+ago\b", r"\bused\s+to\b", r"\bback\s+to\s+(?:normal|himself|herself|school|nursery)\b", r"\bover\s+it\b", r"\bfully\s+better\b"], &line) && !_matches(&FEVER_STILL_NOW, &line) {
            continue;
        }
        if name == "child_fever" && _matches(&PET_WORDS, &line) && !_matches(&HUMAN_TOO, &line) {
            continue;
        }
        // The tiny baby's fever said in the line after the age. The memory carries the age, and
        // this line only has to be about the baby: an age here that is old enough settles it, the
        // speaker's own temperature is an adult's line, and a line that names somebody else (a
        // grandmother, a neighbour) is not the three-week-old.
        if name == "newborn_fever_next" && !_tiny_baby(&*memory) {
            continue;
        }
        // The age came in an earlier line and it is five weeks or more: a plain "she feels hot"
        // with no number is the same-day card for that baby, as it is when the age is in the line,
        // and only a stated strong heat is the hospital call. (see TRUE_NEWBORN)
        if name == "newborn_fever_next" && !(_matches(&BABY_BURNING, &line) || _matches(&FEVER_SIGN, &line)) && !_true_newborn(&*memory) {
            continue;
        }
        if name == "newborn_fever_next" && (_matches(&NEWBORN, &line) || _matches(&TINY_AGE_FINE, &line)) {
            continue;
        }
        // A brother or sister named with years is not the baby in the window. (lively-notebook)
        if name == "newborn_fever_next" && _matches(&OLDER_CHILD_NAMED, &line) {
            continue;
        }
        if name == "newborn_fever_next" && _matches(&ABOUT_THE_SPEAKER, &line) && !_matches(&PERSON_WORD, &line) {
            continue;
        }
        if name == "newborn_fever_next" && _matches(&PERSON_WORD, &line) && !_matches(&TINY_SAID, &line) {
            continue;
        }
        if name == "newborn_fever_next" && _matches(&PET_WORDS, &line) && !_matches(&HUMAN_TOO, &line) {
            continue;
        }
        // The age in memory belongs to one child and the fever in this line can belong to another:
        // "my baby is 3 weeks old" then "my toddler has a fever" is the toddler's card.
        if name == "newborn_fever_next" && _matches(&[r"\btoddlers?\b", r"\bkids?\b", r"\bchildren\b", r"\bchild\b"], &line) {
            continue;
        }
        // The age is in memory and this line only says the baby is hot: the answer is the newborn
        // card, not the sepsis glass test. Without this the reply dispatch aliases the line to the
        // red flags, and "don't let her drive" lands on a three-week-old.
        if name == "newborn_fever_next" && !(_matches(&RASH_RED, &line) || _matches(&STIFF_NECK, &line) || _matches(&SEPSIS_SIGN, &line) || _matches(&[r"\bblue\b", r"\bbreathing\s+fast\b"], &line)) {
            return Some(_fit(&pick(REPLIES.at("newborn_temp"), memory), &line, "newborn_temp"));
        }
        // A child already being treated for it, or one who had it years ago, is a story.
        if name == "named_illness" && _matches(&NAMED_SICK_FINE, &line) && !(_matches(&SEPSIS_SIGN, &line) || _matches(&[r"\bfever\w*", r"\brash\w*", r"\bstiff\s+neck\b"], &line)) {
            continue;
        }
        // A fit every few months is the condition the family already manages; a fit of coughing
        // is not a fit. A temperature of 40 is a number, not a how-to.
        if name == "recent_fit" && _matches(&RECENT_FIT_FINE, &line) {
            continue;
        }
        // A grunt in the cot is a noise; a grunt with a working chest or a floppy baby is not.
        if name == "grunting_baby" && _matches(&GRUNTING_FINE, &line) && !_matches(&GRUNTING_NOW, &line) {
            continue;
        }
        // A heat rash, a nappy rash or chickenpox is the everyday reason for a spot with a fever.
        if name == "fever_rash" && _matches(&FEVER_RASH_FINE, &line) {
            continue;
        }
        // ...and the same everyday rashes stand the headache pair down. A cat with a rash is the
        // vet's, and it was being answered by the meningitis card.
        if name == "rash_headache" && (_matches(&FEVER_RASH_FINE, &line) || _matches(&RASH_HEADACHE_FINE, &line)) {
            continue;
        }
        if name == "rash_headache" && _matches(&PET_WORDS, &line) && !_matches(&HUMAN_TOO, &line) {
            continue;
        }
        // A phone that is warm while charging, a flat battery and a frayed lead are questions
        // about a thing; a blue mood and a blue dress are not a person short of oxygen.
        // A swollen battery that is smoking or hot is the fire card, not the tip about the bin.
        if name == "battery_bulge" && _matches(&[r"\bsmok", r"\bspark", r"\bflames?\b", r"\bon\s+fire\b", r"\bcaught\s+fire\b", r"\bsmell", r"\bhot\s+to\s+touch\b", r"\bmelt"], &line) {
            continue;
        }
        if name == "electric_burn" && _matches(&ELECTRIC_FINE, &line) && !_matches(&[r"\bsmell", r"\bsmoke", r"\bmelt", r"\bspark", r"\bbulging", r"\bflames?\b"], &line) {
            continue;
        }
        // A pet whose gums have gone grey or blue is the vet's case, and the pet table answers
        // that one in the pet's own words: "my puppies gums look grey" was getting the 999 card.
        if name == "blue" && _matches(&PET_ANY, &line) && !_matches(&HUMAN_TOO, &line) && _matches(&PET_MOUTH, &line) {
            continue;
        }
        // Any other pet whose colour changes is a vet now, not a person short of oxygen - and it
        // must not fall through to "Noted: your dog is going blue." or to the laundry answer. The
        // change has to be said or shown on a body part, so a blue collar or a blue bowl stays a
        // question about a thing.
        if name == "blue" && _matches(&PET_ANY, &line) && !_matches(&HUMAN_TOO, &line) && ((_matches(&BLUE_CHANGE, &line) || _matches(&PET_BLUE_WENT, &line)) || _matches(&PET_BLUE_PART, &line)) {
            return Some(_fit(&pick(REPLIES.at("pet_emergency"), memory), &line, "pet_emergency"));
        }
        // The colour card is about a body short of oxygen. A phone, a pan, a pickle or a laptop that
        // went blue is a screen, a kitchen or an electrical question, and main sent every one of
        // them to 999. Only a body part, a person, or a person as the subject of the change keeps it.
        if name == "blue" && !_matches(&BLUE_HUMAN, &line) {
            continue;
        }
        // "Grey" is a hair colour before it is cyanosis. The person-side alternative in BLUE only
        // needs a subject and a colour change, so "im going grey", "why am i going grey" and "my
        // husband is going grey" were all told that a colour change is a 999 call - and every one
        // of them was the grey-hair answer's line. A hair word settles it; without one, a bare
        // grey with no body part, no blue and none of the signs of a real shut-down is hair by
        // default. A grey that names a body part - grey lips, a grey face, grey fingertips, grey
        // gums - is still the card, and so is "he went grey and clammy" or "she went grey and
        // passed out", because those are the signs the card exists for.
        if name == "blue" && !_matches(&GREY_ALARM, &line) && !_matches(&BLUE_LIPS_FACE, &line) && !_matches(&BLUE_TINGE, &line) && (_matches(&GREY_HAIR, &line) || (_matches(&GREY_ARRIVING, &line) && !_matches(&GREY_BODY, &line)) || (_matches(&GREY_COLOUR, &line) && !_matches(&GREY_BODY, &line) && !_matches(&BLUE_HUMAN, &line) && !_matches(&GREY_SELF, &line))) {
            continue;
        }
        // Cold hands, a blue tongue from a slushie, dye on the skin: the colour of a thing, or of a
        // limb that has not changed, is not a person short of oxygen.
        if name == "blue" && _matches(&BLUE_LIMB, &line) && !_matches(&BLUE_LIPS_FACE, &line) && !_matches(&BLUE_TINGE, &line) && !_matches(&BLUE_CHANGE, &line) {
            continue;
        }
        // Cold hands that have gone blue are still cold hands, and a bruise going blue is a bruise.
        if name == "blue" && _matches(&BLUE_LIMB, &line) && _matches(&BLUE_COLD, &line) && !_matches(&BLUE_LIPS_FACE, &line) && !_matches(&BLUE_TINGE, &line) {
            continue;
        }
        // "turning blue" on its own is the breath arm's case for a person, and a pet's colour is
        // the vet's, so the breath arm stands down and the blue arm below answers it.
        if name == "breathing" && _matches(&PET_ANY, &line) && !_matches(&HUMAN_TOO, &line) && _matches(&BLUE_CHANGE, &line) && !_matches(&PET_CRISIS, &line) {
            continue;
        }
        // A garment in the wash going blue is the fabric, not a person short of oxygen.
        if name == "blue" && _matches(&BLUE_CLOTH, &line) && !_matches(&BLUE_LIPS_FACE, &line) && !_matches(&A_BODY_PART, &line) {
            continue;
        }
        // Food in the fridge going blue is mould, not a person short of oxygen. A person has to be
        // the one who changed colour: "my garlic went blue when i pickled it" is the garlic, and
        // the bare "i" in the clause after it is not someone short of oxygen.
        if name == "blue" && _matches(&BLUE_FOOD, &line) && !_matches(&BLUE_PERSON_SUBJECT, &line) && !_matches(&BLUE_OTHER_PERSON, &line) && !_matches(&BLUE_LIPS_FACE, &line) {
            continue;
        }
        // The additions below are the person-side ones: a laugh is not a colour change, make-up is
        // not a colour change, and blue lips that the cold explains want warming up rather than an
        // ambulance. All of it passes when the breathing or the waking is wrong, because then the
        // colour is the least of it and the 999 card is right.
        // A laugh or a held breath that turns a face blue comes back in seconds, so it gets its own
        // light line: standing down was leaving the colour to the laundry ("wash the ruined
        // whites"), which is the wrong answer rather than no answer.
        if name == "blue" && _matches(&WENT_BLUE_IDIOM, &line) && !_matches(&WENT_BLUE_BREATH_RED, &line) {
            _note(memory, "general");
            return Some(_fit(&pick(REPLIES.at("blue_idiom"), memory), &line, name));
        }
        if name == "blue" && _matches(&BLUE_COSMETIC, &line) && !_matches(&WENT_BLUE_BREATH_RED, &line) {
            return None;
        }
        // A baby is the exception to "the cold explains blue lips": the threshold for a small baby
        // is lower, so a blue baby is a 999 call whatever the room is like.
        if name == "blue" && _matches(&BLUE_FACE_PART, &line) && _matches(&BLUE_COLD_CAUSE, &line) && !_matches(&WENT_BLUE_BREATH_RED, &line) && !_matches(&COLD_ILLNESS, &line) && !(_matches(&NEWBORN, &line) || _matches(&[r"\b(?:baby|babies|infant)\b"], &line)) {
            mine = !_matches(&OTHER_BODY, &line);
            _note(memory, "blue");
            return Some(_fit(&pick(REPLIES.at(if mine { "blue_cold" } else { "blue_cold_other" }), memory), &line, "blue"));
        }
        if name == "blue" && !_matches(&BLUE_FACE_PART, &line) && _matches(&LIMB_PART, &line) && _matches(&WENT_BLUE_FINE, &line) && !_matches(&WENT_BLUE_RED, &line) {
            continue;
        }
        if name == "blue" && _matches(&[r"\bchok\w*"], &line) {
            continue;
        }
        if name == "blue" && _matches(&BLUE_COLD_SELF, &line) && _matches(&BLUE_SELF, &line) && !_matches(&BLUE_LIPS_FACE, &line) && !_matches(&LIPS_RED_FLAG, &line) {
            continue;
        }
        if name == "blue" && _matches(&BLUE_MARK, &line) && !_matches(&BLUE_LIPS_FACE, &line) {
            continue;
        }
        if name == "blue" && _matches(&BLUE_LIPS_FACE, &line) && _matches(&LIPS_COLD, &line) && !_matches(&COLD_ILLNESS, &line) && !_matches(&LIPS_RED_FLAG, &line) {
            continue;
        }
        if name == "blue" && _matches(&BLUE_FINE, &line) && !_matches(&BLUE_STRONG, &line) {
            continue;
        }
        // Whose lips they are decides the wording: "my lips are blue" is the reader's own, and
        // "keep them sitting up" is a reply about somebody else.
        if name == "blue" {
            key = if _matches(&OTHER_BODY, &line) { "blue".to_string() } else { "blue_self".to_string() };
            _note(memory, name);
            return Some(_for_the_person(&_fit(&pick(REPLIES.at(&key), memory), &line, name), &line, name));
        }
        // A floppy hat, a limp lettuce or a floppy-eared rabbit is not a sick child.
        if name == "floppy_child" && _matches(&FLOPPY_THING, &line) {
            continue;
        }
        // Floppy on the way to sleep, asked about as a normal thing: answer the question.
        if name == "floppy_child" && _matches(&FLOPPY_SLEEPY, &line) && !_matches(&FLOPPY_RED_FLAG, &line) {
            return Some(_fit(&pick(REPLIES.at("floppy_soft"), memory), &line, "floppy_child"));
        }
        // A joint that will not take weight is an X-ray today, not a spine: "i cant put weight on
        // my ankle" was getting "stay where you are and ring 999". The hip, the floor, a fall
        // from a height and an older person who has gone down keep the ambulance.
        if name == "fall" && _weight_on_a_joint(&line, false) {
            continue;
        }
        if name == "fall" {
            key = if _matches(&[r"\bi\b", r"\bmy\b"], &line) && !(_person(&line).as_deref().map_or(false, |s| !s.is_empty())) { "fall_self".to_string() } else { "fall".to_string() };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, "fall"));
        }
        // A fall in someone older gets checked the same day even when they are up and talking, so
        // this one asks two questions: is this a person old enough for it to matter, and is there
        // anything in the line that says the fall was harmless or never happened.
        // A bang on the head has its own reply, with the signs to watch for, and it is the better
        // answer for "my nan fell and banged her head": this arm only says to get it looked at.
        // A quarrel is not a fall: "i fell owt wiv me sister", typed the way people type it, was
        // the bruising card, and so was "we fell owt over money". The idiom guard lived inside the
        // confusion check only, which these lines do not reach.
        if ["old_fall", "fall", "fall_check"].contains(&name) && _matches(&FALL_WORD_IDIOM, &line) {
            continue;
        }
        if name == "old_fall" && _matches(&HEAD_BANG, &line) {
            continue;
        }
        if name == "old_fall" && _matches(&FALL_STROKE_SIGN, &line) && !_matches(&SLUR_NOT_A_STROKE_WORDS, &line) {
            continue;
        }
        if name == "old_fall" && _fall_with_sign(&line) {
            continue;
        }
        if name == "old_fall" && (_matches(&HEAD_KNOCKOUT, &line) || _out_cold_knockout(&line)) {
            continue;
        }
        if name == "old_fall" && !_matches(&OLD_PERSON, &line) {
            continue;
        }
        if name == "old_fall" && _matches(&OLD_FALL_FINE, &line) {
            continue;
        }
        if name == "old_fall" && _matches(&OLD_FALL_NOT, &line) {
            continue;
        }
        if name == "old_fall" && _matches(&FALL_PAST, &line) && !_matches(&FALL_NOW, &line) {
            continue;
        }
        if name == "old_fall" && _matches(&NEAR_MISS, &line) && !_matches(&NEAR_MISS_HARM, &line) {
            continue;
        }
        if name == "old_fall" && _matches(&PET_FELL, &line) && !_matches(&HUMAN_TOO, &line) {
            continue;
        }
        if name == "old_fall" {
            key = if _matches(&FIRST_PERSON, &line) && !(_person(&line).as_deref().map_or(false, |s| !s.is_empty())) { "old_fall_self".to_string() } else { "old_fall".to_string() };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, "old_fall"));
        }
        // Floppy on the way to sleep, asked about as a normal thing: answer the question (#670).
        if name == "floppy_child" && _matches(&FLOPPY_SLEEPY, &line) && !_matches(&FLOPPY_RED_FLAG, &line) {
            return Some(_fit(&pick(REPLIES.at("floppy_soft"), memory), &line, "floppy_child"));
        }
        // "is fitting" is only a fit when a person is doing it: a shelf fitting is a DIY job.
        // "my cat is fitting in with the new baby" is a new pet, not a seizure: a fit needs a
        // person as its subject, and "fitting in" is the idiom.
        if name == "seizure" && _matches(&[r"\b(?:is|are|keeps?)\s+fitting\b"], &line) && (!_matches(&A_PERSON, &line) || (_matches(&FITTING_IN, &line) || _matches(&FITTING_THING, &line)) || _a_settling_fit(&line)) {
            continue;
        }
        if name == "hypo" {
            key = if _matches(&[r"\bi\b", r"\bmy\b"], &line) && !(_person(&line).as_deref().map_or(false, |s| !s.is_empty())) { "hypo_self".to_string() } else { "hypo".to_string() };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, "hypo"));
        }
        if name == "heat" {
            key = if _matches(&[r"\bi\b", r"\bmy\b"], &line) && !(_person(&line).as_deref().map_or(false, |s| !s.is_empty())) { "heat_self".to_string() } else { "heat".to_string() };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, "heat"));
        }
        if ["red_flags", "stiff_neck", "sepsis_signs", "newborn_fever", "newborn_fever_next"].contains(&name) {
            // The user is the patient unless the line names someone else, and "my baby" is someone
            // else: the reply has to tell them how to help, not how to be helped.
            key = if _matches(&[r"\bi\b", r"\bi'm\b", r"\bim\b", r"\bmy\b"], &line) && !(_matches(&PERSON_WORD, &line) || _matches(&[r"\b(?:newborn|infant|toddler|little\s+one|baby|babies)\b"], &line)) { "red_flags_self".to_string() } else { "red_flags".to_string() };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, "red_flags"));
        }
        if name == "chest" && _matches(&CHEST_FEELING, &line) && !_matches(&_CHEST_OTHER_RED, &line) && !_matches(&CHEST_FEELING_RED, &line) && !_matches(&AFTER_EXERCISE, &line) {
            // The feeling is the cause the person already has, so the answer is the anxiety one and
            // the 999 signs come after it rather than in place of it.
            return Some(_fit(&pick(REPLIES.at("chest_anxiety"), memory), &line, "chest"));
        }
        if name == "chest" && _matches(&CHEST_ADJ, &line) && !_matches(&CHEST, &line) && !_matches(&AFTER_EXERCISE, &line) {
            // "i have a tight chest with a cold", "a tight chest when i cough": tightness said
            // before the word chest is only this entry's line when effort is in it, or when a
            // shape that already matched would have said so anyway.
            continue;
        }
        if name == "chest" && _matches(&AFTER_EXERCISE, &line) && (!_matches(&CHEST_RED_FLAGS, &line) || (_matches(&EXERTION_CHEST, &line) && !_matches(&_CHEST_OTHER_RED, &line))) {
            return Some(_fit(&pick(&CHEST_EFFORT, memory), &line, "chest"));
        }
        if name == "chest" && _matches(&CHEST_TRAINING, &line) && !_matches(&CHEST_SPREADING, &line) {
            continue;
        }
        if thing_collapsed && ["unconscious", "medical"].contains(&name) {
            continue;
        }
        if name == "stuck_machine" && _matches(&STUCK_FINE, &line) && !_matches(&MACHINE, &line) {
            continue;
        }
        // "cant wake up for school/work" is a routine grumble unless something red follows.
        if name == "unconscious" && _matches(&ROUTINE_WAKE, &line) && !_matches(&WAKE_RED, &line) {
            continue;
        }
        // "my friend isnt coming round after all" is a visit and "he hasnt come round yet" is a person
        // still out. The visit words are only believed when nothing in the line says what happened.
        if name == "unconscious" && _matches(&WONT_COME_ROUND, &line) && !_matches(&UNCONSCIOUS, &line) && !_matches(&WAKE_UP_SOMEONE, &line) && _matches(&VISIT_OR_TIME, &line) && !_matches(&COLLAPSE_HINT, &line) {
            continue;
        }
        // A person who is hard to wake in the mornings is a grumble, unless the line says more.
        if name == "unconscious" && _matches(&HARD_TO_WAKE, &line) && !_matches(&UNCONSCIOUS, &line) && !_matches(&WAKE_UP_SOMEONE, &line) && _matches(&MORNING_ROUTINE, &line) && !_matches(&COLLAPSE_HINT, &line) {
            continue;
        }
        // A reason to be up early is the whole story of "i cant wake my mum up for work". The routine
        // words are open-ended, so this is a list of reasons rather than a fence on each shape.
        if name == "unconscious" && _matches(&ROUTINE_REASON, &line) && !(_matches(&WAKE_RED, &line) || _matches(&COLLAPSE_HINT, &line) || _matches(&SPELL_RED, &line)) {
            continue;
        }
        if ["chest"].contains(&name) && _asking_about_the_world(&line) && !(_matches(&[r"\bright\s+now\b", r"\bnow\b", r"\bsudden(?:ly)?\b"], &line)) {
            // ... and the calm version still has the chest in it: a question about causes is not an
            // ambulance, and it is not "go easy on yourself" either (CHEST_ASKED says why). Only
            // where the answer lane has nothing for the line, so the tightness, the reflux and the
            // breathing cards keep theirs.
            if _matches(&CHEST_ASKED_OWN, &line) && !_matches(&CHEST_KEPT_BY_ANOTHER_CARD, &line) {
                return Some(_fit(&pick(&CHEST_ASKED, memory), &line, "chest"));
            }
            continue;
        }
        // A headache and a gunshot are the patient's own line as often as someone else's, and a
        // neighbour or a passer-by has no he or she in the line to read the person from.
        if name == "stuck_machine" {
            mine = !_matches(&OTHER_BODY, &line);
            key = if mine { format!("{}_self", name) } else { name.to_string() };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, name));
        }
        if ["headache", "shot"].contains(&name) {
            if name == "shot" {
                mine = _matches(&SHOT_SELF, &line) && !(_person(&line).as_deref().map_or(false, |s| !s.is_empty()));
            } else {
                mine = _person(&line).is_none() && _matches(&[r"\bi\b", r"\bmy\b"], &line);
            }
            key = if mine { format!("{}_self", name) } else { name.to_string() };
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, name));
        }
        // The night that is already over is an appointment today; anything still happening, or
        // with a red flag in it, stays with the cases above.
        // A shock that killed is a loss, and a shock from childhood is a story.
        // A tool in a socket is the shock waiting to happen; an adapter, a socket being replaced
        // and a socket from years ago are not.
        if name == "socket_thing" && _matches(&SOCKET_THING_FINE, &line) {
            continue;
        }
        // A person who is not breathing or is unconscious beside a shock is a resuscitation,
        // and the shock card's "111 the same day" must never reach them.
        if name == "electric_shock_floor" {
            if _matches(&ELECTRIC_SHOCK_NEARLY, &line) || _matches(&ELECTRIC_SHOCK_DEATH, &line) || (_matches(&ELECTRIC_SHOCK_NOT_ELECTRIC, &line) && !_matches(&ELECTRIC_SHOCK_SUPPLY, &line)) || _matches(&ELECTRIC_SHOCK_CPR, &line) || _matches(&ELECTRIC_SHOCK_STORY, &line) || _matches(&ELECTRIC_SHOCK_DEFINITION, &line) || (_matches(&ELECTRIC_SHOCK_STATIC, &line) && !_matches(&ELECTRIC_SHOCK_WIRED, &line)) {
                continue;
            }
            // Whose body went down decides the wording: "put them on their side" is no use to the
            // person who was knocked out themselves.
            key = if _person(&line).as_deref().map_or(false, |s| !s.is_empty()) || _matches(&OTHER_BODY, &line) { "electric_shock_floor".to_string() } else { "electric_shock_floor_self".to_string() };
            _note(memory, name);
            return Some(_fit(&pick(REPLIES.at(&key), memory), &line, name));
        }
        if name == "electric_shock" && _matches(&ELECTRIC_SHOCK_DOWN, &line) {
            continue;
        }
        // ...and a question about what a shock does is the knowledge card, not the card above.
        if name == "electric_shock" && _matches(&ELECTRIC_SHOCK_DEFINITION, &line) {
            continue;
        }
        if name == "electric_shock" && _matches(&ELECTRIC_SHOCK_DEATH, &line) {
            continue;
        }
        if name == "electric_shock" && _matches(&ELECTRIC_SHOCK_STORY, &line) {
            continue;
        }
        if name == "electric_shock" && _matches(&ELECTRIC_SHOCK_STATIC, &line) && !_matches(&ELECTRIC_SHOCK_WIRED, &line) {
            continue;
        }
        if name == "night_breath" && _matches(&NIGHT_BREATH_RED, &line) {
            continue;
        }
        // A nose that is blocked explains the night gasp the same way it explains the daytime one:
        // "i woke up with a blocked nose and couldnt breathe" is the cold, not the heart.
        if name == "night_breath" && _matches(&BREATH_CAUSE, &line) && !_matches(&NIGHT_BREATH_RED, &line) {
            continue;
        }
        // A nightmare is not the heart: the gasp it leaves behind is the dream coming back.
        if name == "night_breath" && _matches(&NIGHT_BREATH_DREAM, &line) {
            continue;
        }
        // A child's night is the same-day call for the parent, but the words have to be about the
        // child: `my baby was gasping in his sleep` was told to ring 111 and "say it happened in
        // your sleep", which is advice for a grown-up about themselves.
        if name == "night_breath" && _matches(&NIGHT_BREATH_CHILD, &line) && !_matches(&NIGHT_BREATH_RED, &line) {
            _note(memory, name);
            return Some(_fit(&pick(REPLIES.at("night_breath_child"), memory), &line, name));
        }
        // A cat that wakes up gasping is the vet's, the same as a cat that chokes.
        if name == "night_breath" && _matches(&ANIMAL, &line) && !_matches(&HUMAN_TOO, &line) {
            continue;
        }
        if hypothetical {
            let general = GENERAL.get_str(name).unwrap_or("").to_string();
            if !general.is_empty() {
                _note(memory, name);
                return Some(_for_the_person(&general, &line, name));
            }
            continue;
        }
        // Numbness in one arm is a chest when the chest is named in the same line, and a stroke
        // question when it is not.
        // The FAST pair is the one thing in this file that must not be reasoned away, so the
        // stand-down is narrow on purpose: it only reaches a line the FAST arm alone caught, and
        // only with an old cause, an over-use cause or a settled pattern in it.
        if name == "stroke" && _matches(&STROKE_FAST, &line) && !_matches(&STROKE, &line) {
            // The injected arm and the shoulder that stops at shoulder height keep their own
            // answers: `cant lift` is a hard sign, so a cause alone does not stand them down. No
            // face word in the line is what makes it the arm rather than the stroke.
            if !_matches(&_FACE_WORD, &line) && (_matches(&STROKE_FAST_JAB, &line) || _matches(&STROKE_FAST_LIFT_CEILING, &line)) {
                continue;
            }
            if _matches(&STROKE_FAST_OFF, &line) || _matches(&STROKE_FAST_SHOULDER, &line) || (_matches(&STROKE_FAST_PATTERN, &line) && !_matches(&STROKE_PAIR_BOTH, &line) && !_matches(&STROKE_FAST_WEAK, &line)) || (!_matches(&STROKE_FAST_HARD, &line) && !(_matches(&STROKE_FAST_JAB, &line) && _matches(&_FACE_GOING_ODD, &line)) && (_matches(&STROKE_FAST_CAUSE, &line) || _matches(&STROKE_FAST_SENSORY, &line))) {
                continue;
            }
        }
        if name == "arm_numb" && _matches(&CHEST_MENTIONED, &line) {
            continue;
        }
        if name == "chest" && _matches(&CHEST_WORK, &line) && !_matches(&CHEST_URGENT, &line) {
            _note(memory, "chest_slow");
            return Some(_fit(&pick(REPLIES.at("chest_work"), memory), &line, name));
        }
        if name == "chest" && _matches(&CHEST_SLOW, &line) && !_matches(&CHEST_URGENT, &line) {
            _note(memory, "chest_slow");
            return Some(_fit(&pick(REPLIES.at("chest_slow"), memory), &line, name));
        }
        // A chest that hurts with the cough is a strained muscle, not a heart attack. Any heart
        // sign, blood, or a line that says it is happening now keeps the ambulance above.
        if name == "chest" && _matches(&CHEST_SORE_MOVE, &line) && !_matches(&CHEST_URGENT, &line) && !_matches(&CHEST_RED_FLAGS, &line) && !_matches(&CHEST_SPREAD, &line) && !_matches(&CHEST_STRAIN_STOP, &line) && !_matches(&[r"\bblood\b", r"\bbleed\w*", r"\bright\s+now\b"], &line) {
            if _matches(&[r"\bcough\w*"], &line) {
                return Some(_fit_case(&pick(&CHEST_COUGH_STRAIN_REPLY, memory), &line, None));
            }
            return Some(_fit_case(&pick(&CHEST_STRAIN_REPLY, memory), &line, None));
        }
        if name == "chest" && _matches(&CHEST_STRAIN_MOVE, &line) && !_matches(&CHEST_URGENT, &line) && !_matches(&CHEST_RED_FLAGS, &line) && !_matches(&CHEST_SPREAD, &line) && !_matches(&CHEST_STRAIN_STOP, &line) && !_matches(&[r"\bblood\b", r"\bbleed\w*", r"\bright\s+now\b"], &line) {
            return Some(_fit_case(&pick(&CHEST_STRAIN_REPLY, memory), &line, None));
        }
        if name == "chest" && _matches(&CHEST_STRAIN, &line) && !_matches(&CHEST_URGENT, &line) && !_matches(&CHEST_RED_FLAGS, &line) && !_matches(&CHEST_SPREAD, &line) && !_matches(&CHEST_STRAIN_STOP, &line) && !_matches(&[r"\bblood\b", r"\bbleed\w*", r"\bright\s+now\b"], &line) {
            return Some(_fit_case(&pick(&CHEST_COUGH_STRAIN_REPLY, memory), &line, None));
        }
        // A jaw or an arm that aches with no chest in the line is still the ambulance, but the
        // reply must not describe a chest the person never mentioned.
        if name == "chest" && !_matches(&CHEST_MENTIONED, &line) {
            _note(memory, "chest");
            return Some(_for_the_person(&_fit(&pick(REPLIES.at("chest_limb"), memory), &line, "chest"), &line, "chest"));
        }
        // A blow that landed on the nose is not an object up it, and the bruise answer is not
        // "get it looked at today" for a toy: the abuse gate and the parenting answer take it.
        if name == "object_in_head" && _matches(&OBJECT_IN_HEAD_BLOW, &line) && !_matches(&OBJECT_IN_HEAD_BATTERY, &line) {
            continue;
        }
        if name == "object_in_head" && _matches(&OBJECT_IN_HEAD_BATTERY, &line) {
            _note(memory, "object_in_head_battery");
            return Some(_fit(&pick(REPLIES.at("object_in_head_battery"), memory), &line, name));
        }
        // A course of doses spaced hours apart, and the packet's own daily maximum, are what the
        // label says: the poisons card on those teaches a parent that the bot cries wolf on the
        // line where they told the truth.
        if name == "poison" && _matches(&DOSE_COURSE, &line) && !_matches(&DOSE_COURSE_KEEP, &line) {
            continue;
        }
        // Asking whether a bigger dose may be given is not a report that it was given.
        if name == "poison" && _matches(&DOSE_ASKED, &line) && !_matches(&DOSE_ALREADY_GIVEN, &line) {
            continue;
        }
        if name == "dose_stated" {
            counted = _stated_dose(&line);
            if counted.is_none() {
                continue;
            }
            _note(memory, "dose_stated");
            return counted;
        }
        if name == "swallowed_thing" && _matches(&SWALLOWED_BAD, &line) {
            _note(memory, name);
            if _matches(&SPEAKER_IS_PATIENT, &line) && !_matches(&SWALLOWED_CHILD, &line) {
                return Some(pick(&SWALLOWED_BAD_SELF, memory));
            }
            return Some(pick(&SWALLOWED_BAD_REPLY, memory));
        }
        if name == "swallowed_thing" && _matches(&SPEAKER_IS_PATIENT, &line) && !(_matches(&[r"\bmy\s+(?:child|son|daughter|baby|toddler|kid|boy|girl)\b"], &line)) {
            _note(memory, name);
            return Some(pick(&SWALLOWED_SELF, memory));
        }
        // A cut of your own that is still bleeding wants the cloth first, and the crisis line in
        // the same breath: "i cut myself and the blood wont stop" was answered with the crisis line
        // alone on main - no pressure on the wound, no 999 - which is the wrong way round when the
        // bleeding is what hurts them in the next ten minutes. The self_harm case comes before the
        // first aid cases, which is why the crisis reply owned this line, and a bare "i cut myself"
        // keeps the support reply it has.
        // ...unless the line is the wish and not the act: `cut my thighs` is a method phrase the
        // bleeding row reads, and "press on it for ten minutes" is the wrong reply to somebody
        // who has not cut anything yet.
        if name == "self_harm" && !_a_wish_for_their_own_body(&line) && _matches(&SELF_CUT, &line) && (_matches(&BLEEDING, &line) || _matches(&CUT_NOT_STOPPING, &line)) {
            _note(memory, "bleeding");
            return Some(_fit(&pick(REPLIES.at("self_cut_bleeding"), memory), &line, "self_cut_bleeding"));
        }
        // The fit said by the person who had it: "i had a fit" was told to lie *themselves* on
        // their side and to time it on their phone, which the reader cannot do to themselves. Same
        // rule as the electric shock above - whose body it was decides the wording - and a fit
        // somebody else had keeps the card it has.
        if name == "recent_fit" && !(_person(&line).as_deref().map_or(false, |s| !s.is_empty()) || _matches(&OTHER_BODY, &line)) {
            _note(memory, "recent_fit_self");
            return Some(_fit(&pick(REPLIES.at("recent_fit_self"), memory), &line, "recent_fit_self"));
        }
        _note(memory, name);
        return Some(_for_the_person(&_fit(&pick(REPLIES.at(name), memory), &line, name), &line, name));
    }
    // Cold hands and feet that go white, then blue, then red and sore are Raynaud's: common, a GP
    // word, and an answer people were not getting at all - "my toes turn blue when its cold" was
    // answered by the laundry module. The cold has to be in the line: a limb that is simply blue
    // is the went_blue card above, because nothing else explains the colour.
    // A limb the cold explains, said the way people say it, is already answered well ("blue in the
    // cold", "blue with cold" is the Raynaud's card from the health module), so safety keeps out of
    // it: this arm is for the shapes that were getting the laundry card, like "my toes turn blue
    // when its cold" and "my toes went white with the cold".
    if !hypothetical && _matches(&RAYNAUD, &line) && _matches(&[r"\bcold\b", r"\bwinter\b", r"\braynaud\w*\b", r"\bswim\w*", r"\bsea\b", r"\bsnow\b", r"\bfreez\w*", r"\bice\b", r"\bshower\b", r"\bwater\b", r"\bwashing\s+up\b"], &line) && !(_matches(&[r"\b(?:blue|white|purple|grey|gray)\s+(?:in|with|from|out\s+in|and)\s+(?:the\s+|a\s+)?(?:cold|freezing)"], &line)) && !(_matches(&[r"\b(?:swim\w*|swam|the\s+sea|sea\s+water|pool|lido|cold\s+water)\b"], &line)) && !(_matches(&[r"\b(?:can'?t|cannot|struggling\s+to|stopped)\s+breath\w*", r"\bgasping\b", r"\bchest\b", r"\bunconscious\b", r"\bchoking\b"], &line)) {
        return Some(_fit(&pick(REPLIES.at("raynaud"), memory), &line, "minor"));
    }
    // A cut that soaks through the cloth or will not stop is past the ten-minute first aid, but it is
    // not automatically an ambulance: it needs seeing the same day. The red-flag cuts (deep, spurting,
    // a wrist or a thigh) were already taken by the bleeding case above.
    soft_bleed = _matches(&SOFT_BLEED, &line) && _matches(&SOFT_BLEED_ANCHOR, &line) && !_matches(&SOFT_BLEED_OFF, &line);
    if (soft_bleed || _matches(&NOT_STOPPING, &line)) && !_matches(&NOT_STOPPING_OFF, &line) {
        return Some(_fit(&pick(REPLIES.at("bleeding_wont_stop"), memory), &line, "minor"));
    }
    // A bang on the head: what to do now, and the one question that sorts a knock-out from a bump.
    head = if hypothetical { None } else { _head_bang(&line) };
    if head.map_or(false, |s| !s.is_empty()) && _matches(&PET_WORDS, &line) && !_matches(&HUMAN_ANCHOR, &line) {
        head = None;
    }
    if head.map_or(false, |s| !s.is_empty()) {
        who = _head_person(&line);
        let keys = match head.expect("head") {
            "watch" => ("head_bang", "head_bang_other"),
            "red" => ("head_bang_red", "head_bang_red_other"),
            other => panic!("KeyError: {other}"),
        };
        text = pick(HEAD_REPLIES.at(if who.as_deref().map_or(false, |s| !s.is_empty()) { keys.1 } else { keys.0 }), memory);
        if who.as_deref().map_or(false, |s| !s.is_empty()) {
            let (subj, obj) = if who.as_deref() == Some("she") { ("she", "her") } else { ("he", "him") };
            text = format_fields(&text, &[("s", subj), ("o", obj)]);
        }
        return Some(text.clone());
    }
    // "what if i burn my hand" is the first-aid question, so a hypothetical about the user's own
    // body keeps the minor reply; one about somebody else, or a story, does not.
    // A child, a partner or an elderly parent on the line is somebody in the speaker's care, not
    // somebody else: the card for that injury is still the answer, spoken of the person named.
    care = _matches(&OWN_INJURY_IN_MY_CARE, &line);
    own_question = hypothetical && _matches(&OWN_INJURY_QUESTION, &line) && _matches(&[r"\bi\b", r"\bmy\b", r"\byour\b"], &line) && !_matches(&OWN_INJURY_SOMEONE_OR_A_STORY, &line) && (care || !(_matches(&OWN_INJURY_YOU, &line) || _matches(&OWN_INJURY_OTHER_PERSON, &line)));
    // "how many days will it take to heal" is read as a question about the world - the person is
    // not named in it - but a turn after the burn card it is a question about that burn, and the
    // chat is where its subject is. Only a follow-up that borrows its injury this way counts.
    if !own_question && hypothetical && !_the_injury_the_chat_is_on(&line, &*memory).is_empty() {
        own_question = true;
    }
    minor = _minor(&line, &*memory);
    if minor.as_ref().map_or(false, |p| !p.is_empty()) && (!hypothetical || own_question) {
        // `isinstance(minor, tuple)`: every pool `_minor` hands back is a tuple.
        if care && minor.as_ref().expect("minor").len() > 1 {
            let spoken: Vec<String> = minor
                .as_ref()
                .expect("minor")
                .iter()
                .filter(|one| !compiled_with(r"\byou\b|\byour\b", pyre::I | pyre::U).is_match(one))
                .cloned()
                .collect();
            if !spoken.is_empty() {
                minor = Some(spoken);
            }
        }
        return Some(_fit(&pick(minor.as_ref().expect("minor"), memory), &line, "minor"));
    }
    // ...and a singed fringe is not a burn, but it is not "Noted" either: it gets an answer. The skin
    // under it decides which one, so a red or blistered forehead still gets the water.
    if _matches(&HAIR_SCORCH, &line) {
        if _matches(&HAIR_SCORCH_SKIN, &line) || _matches(&MILD_BODY_SIGN, &line) {
            let burn_card = _minor_replies("burn");
            if !burn_card.is_empty() {
                return Some(_fit(&pick(&burn_card, memory), &line, "minor"));
            }
        }
        return Some(pick(&HAIR_SCORCH_ANSWER, memory));
    }
    // The call itself, asked about with nothing to judge: the rule for ringing is the answer.
    if _matches(&ASKS_IF_CALL, &line) && !_matches(&CALL_HAS_A_SIGN, &line) {
        _note(memory, "general");
        return Some(pick(&ASK_CALL_REPLIES, memory));
    }
    // A card that ordered an ambulance has already answered "should i go to hospital", and the
    // answer is the call. The A&E rule of thumb below is for a person deciding about a morning's
    // appointment; after a chest-pain card it was a downgrade inside a live emergency.
    if !_matches(&CALL_HAS_A_SIGN, &line) && (_matches(&ASKS_IF_GOING, &line) || _matches(&ASKS_GO_BARE, &line)) {
        was_case = _last_case(&*memory);
        live = !memory.get("safety_safe").map_or(false, Value::truthy);
        if was_case.as_deref().map_or(false, |c| AMBULANCE_CASES.contains(c)) && live && !_matches(&GO_IN_NOT_ABOUT_IT, &line) {
            let case = was_case.clone().expect("a case in AMBULANCE_CASES");
            _note(memory, &case);
            // `WAITING.get(was_case) or WAITING["general"]`
            let wait = match WAITING.get(&case).filter(|wait| wait.truthy()) {
                Some(wait) => wait.clone(),
                None => WAITING.get("general").expect("KeyError: general").clone(),
            };
            // `wait if isinstance(wait, tuple) else (wait,)`
            let options: Vec<String> = match &wait {
                Value::Tuple(_) => wait.strs(),
                other => vec![value_str(Some(other))],
            };
            let first = pick(&GO_IS_A_CALL, memory);
            return Some(_fit(&format!("{first}{}", pick(&options, memory)), &line, &case));
        }
    }
    if _matches(&ASKS_IF_GOING, &line) && !_matches(&CALL_HAS_A_SIGN, &line) {
        _note(memory, "general");
        return Some(pick(&ASK_GO_REPLIES, memory));
    }
    // A near miss the fire's own cards never saw - "i nearly burnt the house down", "i nearly had a
    // fire but it was fine" - is still a fright, and this is the last place to say so: it comes
    // after every card and after the small injuries, so a burn or a toaster that is smoking is
    // answered by what it is, and only a bare near miss lands here. A fire drill is not a fire.
    if _matches(&NEAR_FIRE, &line) && !_matches(&FIRE_DRILL, &line) && !(_minor(&line, &*memory).as_ref().map_or(false, |p| !p.is_empty())) {
        if _a_fire_that_did_not_happen(&line) {
            _note(memory, "near_fire");
            return Some(_fit(&pick(REPLIES.at("near_fire"), memory), &line, "near_fire"));
        }
        // A fire word outside the near clause is a fire: "i nearly died and then the house was on
        // fire" was reaching the general fallback, because FIRE does not read "the house was on
        // fire" at all - it wants "is on fire". Here the near word has already said a fire is worth
        // a card, so the fire's card is the answer rather than an accident of wording. A line with
        // no fire word in it at all is neither - "prices almost went up" is not a fire - and is
        // left where it was.
        if !_fire_words_in(&line).is_empty() {
            _note(memory, "fire");
            return Some(_fit(&pick(REPLIES.at("fire"), memory), &line, "fire"));
        }
    }
    return _follow_up(&line, memory);
}

#[cfg(test)]
mod tests {
    //! The rule chain checked against PyBot's own, rule by rule: both sides pick the first line of
    //! the pool they chose (`scratch/mod3/safety_ref.py` patches Python's `pick` the same way), so
    //! a difference is a difference in which rule answered, what it noted, or how it fitted the
    //! line. Run with `SAFETY_REF=<reference jsonl> SAFETY_INPUT=<oracle .gz or lines .json>`.
    use super::*;
    use std::cell::Cell;
    use std::io::BufRead;

    thread_local! {
        pub static FIRST_PICK: Cell<bool> = const { Cell::new(false) };
    }

    /// The memory each record's `respond` stage saw (`mem_in` plus every observer's diff).
    fn inputs(path: &str) -> Vec<(String, Memory)> {
        if path.ends_with(".json") {
            let lines: Vec<String> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            return lines.into_iter().map(|line| (line, Memory::new())).collect();
        }
        let reader = std::io::BufReader::new(flate2::read::GzDecoder::new(std::fs::File::open(path).unwrap()));
        let mut out = Vec::new();
        for line in reader.lines() {
            let Ok(line) = line else { break };
            let Ok(rec) = serde_json::from_str::<serde_json::Value>(&line) else { break };
            let mut mem = rec["mem_in"].as_object().cloned().unwrap_or_default();
            for o in rec["observe"].as_array().unwrap() {
                for (k, v) in o["diff"]["set"].as_object().unwrap() {
                    mem.insert(k.clone(), v.clone());
                }
                for k in o["diff"]["del"].as_array().unwrap() {
                    mem.shift_remove(k.as_str().unwrap());
                }
            }
            out.push((rec["text"].as_str().unwrap_or("").to_string(), Memory::from_plain(&mem)));
        }
        out
    }

    #[test]
    #[ignore]
    fn safety_against_the_reference() {
        let (Ok(reference), Ok(input)) = (std::env::var("SAFETY_REF"), std::env::var("SAFETY_INPUT")) else { return };
        crate::runtime::quiet_panics();
        FIRST_PICK.with(|first| first.set(true));
        let cases = inputs(&input);
        let (mut ok, mut bad, mut shown) = (0, 0, 0);
        for line in std::fs::read_to_string(&reference).unwrap().lines() {
            let want: serde_json::Value = serde_json::from_str(line).unwrap();
            let idx = want["idx"].as_u64().unwrap() as usize;
            let Some((said, memory)) = cases.get(idx) else { continue };
            let mut memory = memory.clone();
            let got = guarded(|| _respond(said, &mut memory));
            let raised = got.is_none();
            let reply = got.flatten();
            let case = memory.get("safety_case").and_then(Value::as_str).map(str::to_string);
            let safe = memory.get("safety_safe").and_then(Value::as_bool);
            let person = memory.get("safety_person").and_then(Value::as_str).map(str::to_string);
            let same = reply.as_deref() == want["reply"].as_str()
                && raised == !want["err"].is_null()
                && case.as_deref() == want["case"].as_str()
                && safe == want["safe"].as_bool()
                && person.as_deref() == want["person"].as_str();
            if same {
                ok += 1;
            } else {
                bad += 1;
                if shown < 25 {
                    shown += 1;
                    let panic = crate::runtime::take_last_panic().unwrap_or_default();
                    eprintln!(
                        "#{idx} {said:?}\n   rust: {reply:?} {case:?} {safe:?} {person:?} {}\n   py:   {} {} {} {} {}",
                        if raised { format!("raised {panic}") } else { String::new() },
                        want["reply"], want["case"], want["safe"], want["person"], want["err"]
                    );
                }
            }
        }
        eprintln!("safety against the reference: {ok} same, {bad} different");
    }
}
