//! DUP ports (group 2 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;


// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup


// was DUP(bot.knowledge.lookup::a_weight): now the real port
pub use crate::knowledge::lookup::a_weight;

// ---------------------------------------------------------------------------------------------
// bot.memory.topics / bot.memory.named: what the chat is on, read from the memory store's
// records (`memory["topic"]`, `memory["topics"]`, `memory["user"]`), which live in
// `Memory::extra` under their Python keys.



/// `memory.topics.context(memory, n)` (the real port); None where Python raises (a topic record
/// that is not a dict).
pub fn topics_context(memory: &Memory, n: usize) -> Option<Vec<String>> {
    crate::runtime::guarded(|| crate::memory::topics::context(memory, n as i64))
}







// was DUP(bot.memory.topics::words): now the real port
pub use crate::memory::topics::words as topics_words;

// was DUP(bot.memory.topics::stem): now the real port
pub use crate::memory::topics::stem as topics_stem;

// was DUP(bot.memory.topics::keywords): now the real port
pub use crate::memory::topics::keywords as topics_keywords;






// was DUP(bot.memory.topics::gist): now the real port
pub use crate::memory::topics::gist as topics_gist;




// was DUP(bot.memory.topics::subject): now the real port
pub use crate::memory::topics::subject as topics_subject;




// was DUP(bot.memory.named::subject_word): now the real port
pub use crate::memory::named::subject_word;

// ---------------------------------------------------------------------------------------------
// bot.knowledge.clothing



// was DUP(bot.knowledge.clothing::not_a_fight_about_clothes): now the real port
pub use crate::knowledge::clothing::not_a_fight_about_clothes;

// ---------------------------------------------------------------------------------------------
// bot.compose.repair

fn repair_re(name: &str) -> &'static Re {
    data::re(&format!("bot.compose.repair.{name}"))
}
fn repair_table(name: &str) -> Vec<(String, &'static Re)> {
    data::get(&format!("bot.compose.repair.{name}"))
        .items()
        .iter()
        .map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().expect("a regex")))
        .collect()
}

static PET_OR_PROFESSIONAL: LazyLock<&'static Re> = LazyLock::new(|| repair_re("PET_OR_PROFESSIONAL"));
static PHYSICAL_HARM: LazyLock<&'static Re> = LazyLock::new(|| repair_re("PHYSICAL_HARM"));
static GRIEF: LazyLock<&'static Re> = LazyLock::new(|| repair_re("_GRIEF"));
static A_BABY: LazyLock<&'static Re> = LazyLock::new(|| repair_re("_A_BABY"));
static REGRET_FIRST: LazyLock<&'static Re> = LazyLock::new(|| repair_re("_REGRET_FIRST"));
static NOT_A_DEED: LazyLock<&'static Re> = LazyLock::new(|| repair_re("NOT_A_DEED"));
static NOBODY_TO_REPAIR: LazyLock<&'static Re> = LazyLock::new(|| repair_re("NOBODY_TO_REPAIR"));
static AMENDS: LazyLock<&'static Re> = LazyLock::new(|| repair_re("AMENDS"));
static ASKS_FOR_THE_WORDS: LazyLock<&'static Re> = LazyLock::new(|| repair_re("ASKS_FOR_THE_WORDS"));
static ASKS_WHAT_TO_DO: LazyLock<&'static Re> = LazyLock::new(|| repair_re("ASKS_WHAT_TO_DO"));
static LAPSES: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| repair_table("LAPSES"));
static DEEDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| repair_table("DEEDS"));

/// The line a repair test reads: lower-cased, curly apostrophes made plain, whitespace squashed.
fn repair_text(clean: &str) -> String {
    text::squash(&clean.to_lowercase().replace('\u{2019}', "'"))
}

/// DUP(bot.compose.repair::deed_of): the deed the person reports doing to somebody, or None.
/// Takes the line (a reading's `clean or text`).
pub fn deed_of(clean: &str) -> Option<String> {
    let text_ = repair_text(clean);
    if text_.is_empty() {
        return None;
    }
    if PET_OR_PROFESSIONAL.is_match(&text_) || PHYSICAL_HARM.is_match(&text_) {
        return None;
    }
    if GRIEF.is_match(&text_) || A_BABY.is_match(&text_) || REGRET_FIRST.is_match(&text_) {
        return None;
    }
    if let Some((deed, _)) = LAPSES.iter().find(|(_, p)| p.is_match(&text_)) {
        return Some(deed.clone());
    }
    if NOT_A_DEED.is_match(&text_) || NOBODY_TO_REPAIR.is_match(&text_) {
        return None;
    }
    DEEDS.iter().find(|(_, p)| p.is_match(&text_)).map(|(deed, _)| deed.clone())
}

/// DUP(bot.compose.repair::owns_the_deed) for a reading: true when the line reports something
/// the person did to somebody close.
pub fn owns_the_deed(reading: &Understood) -> bool {
    deed_of(crate::knowledge::d1_shared::clean_or_text(reading)).is_some()
}

/// DUP(bot.compose.repair::owns_the_deed) for a line of text.
pub fn owns_the_deed_line(line: &str) -> bool {
    deed_of(line).is_some()
}

/// DUP(bot.compose.repair::amends_to): the line asks how to make amends to somebody.
pub fn amends_to(clean: &str) -> bool {
    let text_ = repair_text(clean);
    if text_.is_empty() || !AMENDS.is_match(&text_) {
        return false;
    }
    !(PET_OR_PROFESSIONAL.is_match(&text_) || PHYSICAL_HARM.is_match(&text_))
}

/// DUP(bot.compose.repair::a_repair_is_open): the turns just said are one repair, with the asks
/// in the middle of it.
pub fn a_repair_is_open(memory: &Memory) -> bool {
    let told: Vec<String> = memory.turns.iter().map(|t| t.user.to_lowercase()).collect();
    for said in told[told.len().saturating_sub(3)..].iter().rev() {
        let said = text::squash(said);
        if said.is_empty() {
            continue;
        }
        if owns_the_deed_line(&said) || amends_to(&said) {
            return true;
        }
        if text::split_ref(&said).len() > 9 || !(ASKS_FOR_THE_WORDS.is_match(&said) || ASKS_WHAT_TO_DO.is_match(&said)) {
            return false;
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup (more)


// was DUP(bot.knowledge.lookup::asks_how_often): now the real port
pub use crate::knowledge::lookup::asks_how_often;

// ---------------------------------------------------------------------------------------------
// bot.knowledge.cooking


// was DUP(bot.knowledge.cooking::_fold): now the real port
pub use crate::knowledge::cooking_b::_fold as cooking_fold;

// was DUP(bot.knowledge.cooking::_refused_foods): now the real port
pub use crate::knowledge::cooking_b::_refused_foods as cooking_refused_foods;
