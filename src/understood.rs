//! The understanding engine's reading of a line: Python's `memory["understood"]` dict.
//!
//! Every field PyBot writes is here, with the type it holds on single-turn chat (worked out from
//! the reading of 2,198 prompts; see `harness/oracle_dump.py`). Fields that only fill in on a
//! later turn of a conversation (`kept`, `taking_up`, `looks_back`, ...) are `Option<String>`
//! and stay `None` in this port, which answers one line at a time.
//!
//! It (de)serialises to the same plain JSON as the Python dict (`data/export/understood_*.jsonl`),
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Number {
    pub value: Option<Num>,
    pub unit: Option<String>,
    pub text: String,
}

/// What "it", "that", "they", ... point back at. Single-turn: nothing yet.
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
    pub followup: Option<bool>,
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
    pub kept: Option<String>,
    pub kind: String,
    pub intent: String,
    pub asks_how_i_am: bool,
    pub presence: bool,
    pub asks: Vec<String>,
    pub tag: bool,
    pub taking_up: Option<String>,
    pub shift: Option<String>,
    pub care_home: Option<String>,
    pub setup: Option<String>,
    pub help_ask: bool,
    pub asks_a_verdict: bool,
    pub emotion: Option<String>,
    pub state_owner: Option<String>,
    pub shortage: Option<String>,
    pub polarity: i64,
    pub warmth: Option<String>,
    pub body: Option<String>,
    pub own_output: Option<String>,
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
    pub summons: Option<String>,
    pub asks_back: bool,
    pub stand_in: Option<String>,
    pub excludes: Vec<String>,
    pub constraint: bool,
    pub confusion: Option<IndexMap<String, String>>,
    pub suggestion: Option<IndexMap<String, String>>,
    pub ruled_out: Vec<String>,
    pub norm: Vec<String>,
    pub norm_text: String,
    pub fixes: IndexMap<String, String>,
    pub clauses: Vec<Vec<String>>,
    pub focus: Vec<String>,
    pub asking: Vec<String>,
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
    pub leans_on: Option<String>,
    pub looks_back: Option<String>,
    /// Written by the intent observer: the shape of answer the line wants ("steps", "yesno", ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wants: Option<String>,
}
