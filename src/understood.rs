//! The understanding engine's reading of a line: Python's `memory["understood"]` dict.
//!
//! Every key `bot/understanding/__init__.py::_read_line` writes is a typed field here, with the
//! type the Python code puts there (a dict it builds is a struct with the same keys). Keys a
//! module adds to the reading later in a turn (or that a reading read back from JSON carries and
//! this struct does not name) are kept in `extra`, by Python key, so a reading round-trips.
//!
//! It (de)serialises to the same plain JSON as the Python dict (`data/export/oracle_*.jsonl`),
//! which is how the port's reading is checked against PyBot's, field by field.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub text: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub at: i64,
}

/// A Python number: PyBot keeps ints and floats apart ("2" and "2.0" print differently).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Num {
    Int(i64),
    Float(f64),
}

impl Num {
    pub fn as_f64(&self) -> f64 {
        match self {
            Num::Int(i) => *i as f64,
            Num::Float(f) => *f,
        }
    }
}

/// `topics.detect_numbers`: one figure in the line.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Number {
    pub value: Option<Num>,
    pub unit: Option<String>,
    pub text: String,
}

/// `references.resolve`: what "it", "that", "they", ... point back at.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ref {
    pub it: Option<String>,
    pub that: Option<String>,
    pub they: Option<String>,
    pub there: Option<String>,
    pub he: Option<String>,
    pub she: Option<String>,
    pub kind: Option<String>,
    pub followup: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Signals {
    pub question_mark: bool,
    pub shout: bool,
    pub laughter: bool,
    pub ellipsis: bool,
    pub exclamation: bool,
    pub empty: bool,
}

/// `normalize._kept_for`: how long a thing has been kept, and where.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Kept {
    pub place: Option<String>,
    pub time: String,
}

/// `kinds.taking_up`: a thing the person has started doing (or gone back to).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TakingUp {
    pub text: String,
    pub words: Vec<String>,
    pub shape: String,
    pub back: bool,
}

/// `kinds.a_care_home` / `a_home_carried_forward`: someone in a care home. `carried` is the
/// earlier line it came from, only when it was carried forward.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CareHome {
    pub text: String,
    pub who: String,
    pub relation: Option<String>,
    pub pronoun: String,
    pub shape: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carried: Option<String>,
}

/// `kinds.summons_of`: somebody official asking the person to come in.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Summons {
    pub who: String,
    pub shape: String,
}

/// `understanding._stand_in_here` / the noun lent to "one" or "ones".
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StandIn {
    pub word: String,
    pub noun: Option<String>,
}

/// `confusion.confusion_of`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Confusion {
    pub shape: String,
    pub thing: Option<String>,
}

/// `kinds._a_suggestion_from_someone`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Suggestion {
    pub who: String,
    pub source: String,
}

/// `understanding._leans_on`: the earlier line this one carries on from.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LeansOn {
    pub text: String,
    pub words: Vec<String>,
    pub subject: Option<String>,
    pub subject_type: Option<String>,
    pub topic: Option<String>,
}

/// `memory.recall.the_earlier_turn`: the earlier turns a look-back cue points at.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LooksBack {
    pub cue: String,
    pub thing: String,
    pub said: Vec<String>,
}

/// One item of `fixes` once `understand` has folded a confused-word swap into it: Python does
/// `list(alt["fixes"]) + pairs`, which is the dict's keys followed by `(typed, meant)` pairs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FixItem {
    Key(String),
    Pair(String, String),
}

/// `fixes`: `{typed: meant}` from `normalise`, or the list `understand` makes of it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Fixes {
    Map(IndexMap<String, String>),
    List(Vec<FixItem>),
}

impl Default for Fixes {
    fn default() -> Fixes {
        Fixes::Map(IndexMap::new())
    }
}

impl Fixes {
    pub fn is_empty(&self) -> bool {
        match self {
            Fixes::Map(m) => m.is_empty(),
            Fixes::List(v) => v.is_empty(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Understood {
    pub text: String,
    pub clean: String,
    pub words: Vec<String>,
    pub plain: String,
    pub animal_in_hand: String,
    pub change_of_subject: bool,
    pub dropped_subject: bool,
    pub sense: IndexMap<String, String>,
    pub over_time: Option<bool>,
    pub over_time_what: Option<String>,
    pub kept: Option<Kept>,
    pub kind: String,
    pub intent: String,
    pub asks_how_i_am: bool,
    pub presence: bool,
    pub asks: Vec<String>,
    pub tag: bool,
    pub taking_up: Option<TakingUp>,
    pub shift: Option<String>,
    pub care_home: Option<CareHome>,
    pub setup: Option<String>,
    pub help_ask: bool,
    pub asks_a_verdict: bool,
    pub emotion: Option<String>,
    pub state_owner: Option<String>,
    pub shortage: Option<String>,
    pub polarity: i64,
    pub warmth: Option<String>,
    pub body: Option<String>,
    /// The position in `norm` of the word naming the person's own bodily output.
    pub own_output: Option<i64>,
    pub short: bool,
    pub first_person: bool,
    pub about_bot: bool,
    pub about_bot_kind: Option<String>,
    pub counts_the_bots_own_day: bool,
    pub replies_to_bot: bool,
    pub agrees_to_try: bool,
    pub wants_a_quiz: bool,
    pub agreement: Option<String>,
    pub tight_on_money: bool,
    pub amount_left: bool,
    pub routine: bool,
    pub a_load_carried: bool,
    pub car_fault: bool,
    pub summons: Option<Summons>,
    pub asks_back: bool,
    pub stand_in: Option<StandIn>,
    pub excludes: Vec<String>,
    pub constraint: bool,
    pub confusion: Option<Confusion>,
    pub suggestion: Option<Suggestion>,
    pub ruled_out: Vec<String>,
    pub norm: Vec<String>,
    pub norm_text: String,
    pub fixes: Fixes,
    pub clauses: Vec<Vec<String>>,
    /// None only when the line has no clause at all (nothing but punctuation).
    pub focus: Option<Vec<String>>,
    pub asking: Option<Vec<String>>,
    pub signals: Signals,
    pub topic: Option<String>,
    pub topic_score: i64,
    pub topics: Vec<String>,
    pub topic_own: Option<String>,
    pub topic_own_score: i64,
    pub topic_from: Vec<(String, i64)>,
    pub topic_is_a_brush: bool,
    pub unsure: Vec<String>,
    pub entities: Vec<Entity>,
    pub person_names: Vec<String>,
    pub background: Vec<String>,
    pub intensifier: Vec<String>,
    pub intensifier_left: Vec<String>,
    pub subject: Option<String>,
    pub subject_unclear: Option<String>,
    pub ask_subject: Option<String>,
    pub setting: Option<String>,
    pub subject_from: Option<String>,
    pub numbers: Vec<Number>,
    pub tried: Vec<String>,
    pub keywords: Vec<String>,
    #[serde(rename = "ref")]
    pub reference: Ref,
    pub followup: bool,
    pub someone_new: bool,
    pub asks_what_was_meant: bool,
    pub leans_on: Option<LeansOn>,
    /// Written by the understand observer after the reading (`recall.the_earlier_turn`).
    pub looks_back: Option<LooksBack>,
    /// Written by the intent observer: the shape of answer the line wants ("steps", "yesno", ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wants: Option<String>,
    /// `understand`: the ask found inside an errand ("can you look up how long ...").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errand: Option<String>,
    /// `understand`: the earlier line with the new thing swapped in ("and for brown rice?").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asked_again: Option<String>,
    /// Any other key a module put on the reading, by Python key.
    #[serde(flatten)]
    pub extra: IndexMap<String, serde_json::Value>,
}

impl Understood {
    /// The focus clause, or nothing (`line["focus"] or []`).
    pub fn focus_words(&self) -> &[String] {
        self.focus.as_deref().unwrap_or(&[])
    }
    /// `asking` (falls back to nothing).
    pub fn asking_words(&self) -> &[String] {
        self.asking.as_deref().unwrap_or(&[])
    }
}
