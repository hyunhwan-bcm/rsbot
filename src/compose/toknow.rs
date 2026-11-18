//! Port of `bot/compose/toknow.py`: "anything i should know about X" answered with what to check
//! before agreeing to something, the knowledge base's card about X, or a question about which part.

use crate::compose::fallback_chain::caught;
use crate::data;
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

const M: &str = "bot.compose.toknow.";
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.toknow.", stringify!($name))));)*
    };
}
regexes!(A_WHAT_TO_KNOW_ASKED, A_THING_YOU_AGREE_TO, A_PRONOUN_NOT_A_THING, THE_WORDS_THAT_END_THE_THING);
static BEFORE_YOU_AGREE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}BEFORE_YOU_AGREE")));
static WHAT_TO_KNOW_FIRST: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}WHAT_TO_KNOW_FIRST")));
static WHAT_THE_BOT_CAN_SAY_ABOUT_IT: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs(&format!("{M}WHAT_THE_BOT_CAN_SAY_ABOUT_IT")));
static A_FEW_WORDS_OF_THING: LazyLock<usize> =
    LazyLock::new(|| data::get(&format!("{M}A_FEW_WORDS_OF_THING")).as_int().unwrap() as usize);
/// The two `re.search` patterns written inline in `_the_thing_asked_about`.
static KNOW_ABOUT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bknow\b[^?.!]{0,10}?\babout\b\s*([^?.!]{1,60})", crate::py::re::I));
static ABOUT_BEFORE: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\babout\b\s*([^?.!]{1,60})\b(?:before|first)\b", crate::py::re::I));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/toknow.py:63 `_the_thing_asked_about(clean)`
///
/// "Anything I should know about the job" -> "the job", or "" (only this file calls it).
pub fn _the_thing_asked_about(clean: &str) -> String {
    let found = KNOW_ABOUT.search(clean).or_else(|| ABOUT_BEFORE.search(clean));
    let Some(found) = found else { return String::new() };
    let mut thing = text::strip_chars(found.str(1), " ?.!,").to_string();
    if let Some(cut) = THE_WORDS_THAT_END_THE_THING.search(&thing) {
        thing = text::strip_chars(&thing[..cut.start()], " ?.!,").to_string();
    }
    if thing.is_empty() || A_PRONOUN_NOT_A_THING.match_(&thing).is_some() {
        return String::new();
    }
    if text::split_ref(&thing).len() > *A_FEW_WORDS_OF_THING {
        return String::new();
    }
    thing
}

/// Python: bot/compose/toknow.py:81 `_a_card_about(thing)`
///
/// The knowledge base's own answer about a named thing, or "" (only this file calls it).
pub fn _a_card_about(thing: &str) -> String {
    for frame in WHAT_THE_BOT_CAN_SAY_ABOUT_IT.iter() {
        let line = frame.replacen("{}", thing, 1);
        let said = caught(|| crate::modules::answers::respond(&line, &mut Memory::new())).flatten();
        if let Some(said) = said.filter(|s| !s.is_empty()) {
            return said;
        }
    }
    String::new()
}

/// Python: bot/compose/toknow.py:95 `ask_about_what_to_look_out_for(clean, memory=None)`
///
/// None when the line is not this ask, "" when it is and there is nothing to say, else a reply.
pub fn ask_about_what_to_look_out_for(clean: &str, memory: &Memory) -> Option<String> {
    let said = text::squash(&text::lower(clean));
    if said.is_empty() || !A_WHAT_TO_KNOW_ASKED.is_match(&said) {
        return None;
    }
    if A_THING_YOU_AGREE_TO.is_match(&said) {
        return Some(pick(&BEFORE_YOU_AGREE, memory, &said));
    }
    let asked_about = _the_thing_asked_about(&said);
    if asked_about.is_empty() {
        return Some(String::new());
    }
    let card = _a_card_about(&asked_about);
    if !card.is_empty() {
        return Some(card);
    }
    let shaped = pick(&WHAT_TO_KNOW_FIRST, memory, &said);
    if !shaped.is_empty() {
        return Some(shaped.replace("{thing}", &asked_about));
    }
    Some(String::new())
}
