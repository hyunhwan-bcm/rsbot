//! Port of `bot/compose/their_detail.py`: a detail dropped in after an answer ("its for six",
//! "im going to rome") turned into the question it implies and put to the knowledge base.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_vals, inline_re};
use crate::compose::fallback;
use crate::data::{self, Value};
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

bundle_res!("bot.compose.their_detail"; ASKS, DIET_WORD, COUNT, GOING, MONTH, CONSIDERING, FOOD_WORDS);
bundle_vals!("bot.compose.their_detail"; NUMBER);
inline_re!(ONE_MORE, r"\b(?:more|extra|another)\b");
inline_re!(A_LEAD_ARTICLE, r"^(?:(?:buying|getting|ordering)\s+)?(?:a|an|the)\s+");
inline_re!(A_LONG_WORD, r"[a-z']{4,}");

static NEUTRAL: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.compose.fallback.NEUTRAL"));

/// Python: bot/compose/their_detail.py:85 `_about_food(memory)`
///
/// Whether the turn above was about food, which is what a headcount is a detail of.
pub fn _about_food(memory: &Memory) -> bool {
    let mut said: Vec<String> = Vec::new();
    if let Some(last) = memory.get("last_answer").filter(|l| l.is_dict()) {
        if let Some(reply) = last.get("reply").filter(|r| r.truthy()) {
            said.push(com4::py_str(reply));
        }
    }
    if let Some(turn) = memory.turns.iter().rev().find(|t| !t.bot.is_empty()) {
        said.push(turn.bot.clone());
    }
    said.iter().any(|one| FOOD_WORDS.search(&one.to_lowercase()).is_some())
}

/// Python: bot/compose/their_detail.py:98 `_asked(kind, detail, line)`
///
/// The question this detail implies, written out of the line's own words, or None.
pub fn _asked(kind: &str, detail: &str, line: &str) -> Option<String> {
    match kind {
        "dietword" => Some(format!("what can i cook for a {detail}")),
        "count" if detail == "one more" => Some("how much food for one more person".to_string()),
        "count" => Some(format!("how much food for {detail} people")),
        "going" => Some(match MONTH.search(line) {
            Some(month) => format!("what should i know before a trip to {detail} in {}", month.str(1).to_lowercase()),
            None => format!("what should i know before a trip to {detail}"),
        }),
        "considering" => Some(format!("what do i need to know before buying {detail}")),
        _ => None,
    }
}

/// Python: bot/compose/their_detail.py:116 `_the_read(clean)`
///
/// The kind of detail this line adds, and the words it names, or None for `(None, None)`.
pub fn _the_read(clean: &str) -> Option<(String, String)> {
    if clean.is_empty() || clean.contains('?') || ASKS.search(clean.split(',').next().unwrap_or("")).is_some() {
        return None;
    }
    if text::split_ref(clean).len() > 16 {
        return None;
    }
    if let Some(m) = DIET_WORD.search(clean) {
        return Some(("dietword".into(), m.name_str("diet").to_lowercase()));
    }
    if let Some(m) = COUNT.search(clean) {
        let number = m.name_str("n").to_lowercase();
        if ONE_MORE.search(clean).is_some() {
            return Some(("count".into(), "one more".into()));
        }
        let said = NUMBER.get(&number).map(|v| v.str().to_string()).unwrap_or(number);
        return Some(("count".into(), said));
    }
    if let Some(m) = GOING.search(clean) {
        return Some(("going".into(), text::strip(m.name_str("place")).to_string()));
    }
    if let Some(m) = CONSIDERING.search(clean) {
        return Some(("considering".into(), A_LEAD_ARTICLE.sub("", text::strip(m.name_str("thing")))));
    }
    None
}

/// Python: bot/compose/their_detail.py:142 `_a_real_answer(reply)`
///
/// True when the knowledge base answered rather than hedging or filing the line.
pub fn _a_real_answer(reply: Option<&str>) -> bool {
    let said = text::squash(reply.unwrap_or(""));
    if text::len(&said) < 40 {
        return false;
    }
    let pools = Value::Tuple(std::sync::Arc::new(vec![(*NEUTRAL).clone()]));
    !(fallback::_is_a_hedge(&said) || fallback::_in_a_pool(&said, &pools))
}

/// Python: bot/compose/their_detail.py:150 `_said_already(memory)`
///
/// Every line the bot has said in this conversation, in its flat form.
pub fn _said_already(memory: &Memory) -> Vec<String> {
    memory.turns.iter().filter(|t| !t.bot.is_empty()).map(|t| text::squash(&t.bot)).collect()
}

/// Python: bot/compose/their_detail.py:160 `reply_for(text, memory)`
///
/// The knowledge base's answer to the question this detail implies, or None.
pub fn reply_for(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text_.to_lowercase());
    let (kind, detail) = _the_read(&clean)?;
    if kind == "count" && !_about_food(memory) {
        // A number of people is a portion question only over a meal.
        return None;
    }
    let question = _asked(&kind, &detail, &clean)?;
    // The implied question is put to the entries on its own, with an empty memory.
    let reply = com4::attempt(|| {
        let mut fresh = Memory::new();
        crate::modules::answers::respond(&question, &mut fresh)
    })?;
    if !_a_real_answer(reply.as_deref()) {
        return None;
    }
    let reply = reply.unwrap_or_default();
    if kind == "going" || kind == "considering" {
        // The card has to name the thing the person named.
        let words = A_LONG_WORD.findall_str(&detail);
        let said = reply.to_lowercase();
        if !words.is_empty() && !words.iter().any(|w| said.contains(w.as_str())) {
            return None;
        }
    }
    if _said_already(memory).contains(&text::squash(&reply)) {
        // The card has been said already: saying it again is worse than the receipt.
        return None;
    }
    Some(reply)
}
