//! Port of `bot/modules/saving_followup.py`: a line that is part of a saving plan already in the
//! chat (the rate, the date, the pot, the gap) is answered as that plan's next figure.
#![allow(non_snake_case, clippy::all)]

use super::chat::{py, said, truthy};
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

static A_LENGTH_IN_MONTHS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.saving_followup.A_LENGTH_IN_MONTHS"));

/// Python: bot/modules/saving_followup.py:57 `_lower_first(answer)`
pub fn _lower_first(answer: &str) -> String {
    let mut chars = answer.chars();
    let Some(first) = chars.next() else { return String::new() };
    // `text[:2].isupper()`: the first two characters hold a cased one and none lower-case.
    if !first.is_uppercase() || text::is_upper(&text::slice(answer, 0, Some(2))) {
        return answer.to_string();
    }
    first.to_lowercase().collect::<String>() + chars.as_str()
}

/// Python: bot/modules/saving_followup.py:65 `_the_money_answer(question, memory)`
pub fn _the_money_answer(question: &str, memory: &mut Memory) -> String {
    if question.is_empty() {
        return String::new();
    }
    let was = memory.understood.take();
    memory.understood = Some(crate::understanding::understand(question, Some(&*memory)));
    let answer = crate::modules::answers::_respond(question, memory).unwrap_or_default();
    memory.understood = was;
    if answer.is_empty() {
        return String::new();
    }
    said(crate::memory::saving::without_what_was_just_said(&answer, memory)).unwrap_or(answer)
}

/// Python: bot/modules/saving_followup.py:94 `_the_reply(text, memory)`. Python keeps a small
/// cache keyed by the memory object's identity; reading afresh gives the same reply.
pub fn _the_reply(text_: &str, memory: &mut Memory) -> String {
    _read(text_, memory).unwrap_or_default()
}

/// Python: bot/modules/saving_followup.py:114 `respond(text, memory)` - a runtime hook, wrapped
/// by `again.replied` (Python: `respond = _said_before.replied(respond)`).
pub fn respond(text_: &str, memory: &mut Memory) -> Option<String> {
    let reply = _the_reply(text_, memory);
    if reply.is_empty() {
        return None;
    }
    if text::strip(&reply).is_empty() {
        return Some(reply);
    }
    match super::chat::attempt(|| super::chat::opt_str(crate::memory::again::instead(&reply, text_, memory))) {
        Some(Some(framed)) if !framed.is_empty() => Some(framed),
        _ => Some(reply),
    }
}

/// Python: bot/modules/saving_followup.py:121 `TAKES_AS_IT_COMES(text, memory)`
pub fn TAKES_AS_IT_COMES(text_: &str, memory: &mut Memory) -> bool {
    !_the_reply(text_, memory).is_empty()
}

/// `format(n, ",")`.
fn with_commas(v: &Value) -> String {
    match v {
        Value::Int(i) => text::thousands(*i),
        other => text::thousands(other.as_int().unwrap_or_else(|| panic!("format() of {other:?}"))),
    }
}

/// Python: bot/modules/saving_followup.py:134 `_read(text, memory)`
pub fn _read(text_: &str, memory: &mut Memory) -> Option<String> {
    use crate::memory::saving as s;
    if let Some(gap) = said(s::the_gap_left(text_, memory)) {
        return Some(gap);
    }
    if let Some(position) = said(s::the_pot_position(text_, memory)) {
        return Some(position);
    }
    if let Some(more) = said(s::the_extra_a_month_asked(text_, memory)) {
        return Some(more);
    }
    if let Some(same) = said(s::the_same_figure_a_month(text_, memory)) {
        return Some(same);
    }
    if let Some(enough) = said(s::a_yes_no_about_the_rate(text_, memory)) {
        return Some(enough);
    }
    if let Some(need_a_date) = said(s::a_rate_with_no_date(text_, memory)) {
        return Some(need_a_date);
    }
    if let Some(raised) = said(s::a_rate_raised(text_, memory)) {
        return Some(raised);
    }
    if let Some(restated) = said(s::a_rate_restated(text_, memory)) {
        return Some(restated);
    }
    if let Some(put_in) = said(s::a_rate_put_in_with_a_verb(text_, memory)) {
        return Some(put_in);
    }
    if let Some(at_the_chat_rate) = said(s::a_length_at_the_rate_in_the_chat(text_, memory)) {
        let answer = _the_money_answer(&at_the_chat_rate, memory);
        if !answer.is_empty() {
            return Some(answer);
        }
    }
    if let Some(no_price) = said(s::a_length_with_no_price(text_, memory)) {
        return Some(no_price);
    }
    if let Some(price) = said(s::a_price_in_an_answer(text_, memory)) {
        return Some(price);
    }
    if let Some(given) = said(s::a_rate_for_the_goal(text_, memory)) {
        return Some(given);
    }
    if truthy(s::asks_how_long(text_)) {
        if let Some(length) = said(s::how_long_it_takes(memory)) {
            return Some(length);
        }
    }
    let pot_alone = py(s::the_pot_said_alone(text_));
    if pot_alone.truthy() {
        let pot = pot_alone.as_int().unwrap_or(0);
        let whole = py(s::_the_goal_in_the_chat(memory)).as_int().unwrap_or(0);
        if whole > pot {
            let rest = whole - pot;
            let question = py(s::rate_question_for(&Value::Int(rest), ""));
            let answer = _the_money_answer(question.str(), memory);
            if !answer.is_empty() {
                let opener = py(s::without_the_pot_opener(&answer));
                return Some(format!("£{} still to find: {}", text::thousands(rest), _lower_first(opener.str())));
            }
        }
    }
    if truthy(s::asks_about_saving(text_)) && truthy(s::_names_a_goal(text_)) {
        let both = py(s::the_pot_and_the_rest(text_));
        let (pot, rest) = (both.items().first().cloned().unwrap_or(Value::Int(0)), both.items().get(1).cloned().unwrap_or(Value::Int(0)));
        let mut answer = String::new();
        if py(s::_a_rate_said(text_)).items().first().is_some_and(|v| v.truthy()) || truthy(s::a_length_was_said(text_)) {
            let plain = _the_money_answer(text_, memory);
            if !plain.is_empty() && A_LENGTH_IN_MONTHS.search(&plain).is_some() {
                answer = plain;
            }
        }
        if answer.is_empty() {
            let question = py(s::rate_question(text_, memory));
            answer = _the_money_answer(question.str(), memory);
        }
        if !answer.is_empty() {
            if pot.truthy() && rest.truthy() {
                let opener = py(s::without_the_pot_opener(&answer));
                return Some(format!(
                    "£{} to find on top of the £{} you already have: {}",
                    with_commas(&rest),
                    with_commas(&pot),
                    _lower_first(opener.str())
                ));
            }
            return Some(answer);
        }
    }
    if truthy(s::asks_the_rate(text_, memory)) {
        let question = py(s::rate_question(text_, memory));
        let mut answer = _the_money_answer(question.str(), memory);
        if !answer.is_empty() && truthy(s::said_already(&answer, memory)) {
            let mut other = said(s::a_figure_each_week(memory)).unwrap_or_default();
            if other.is_empty() || truthy(s::said_already(&other, memory)) {
                let fresh = py(s::the_figure_a_week(memory));
                other = if fresh.truthy() { _the_money_answer(fresh.str(), memory) } else { String::new() };
            }
            if !other.is_empty() && !truthy(s::said_already(&other, memory)) {
                answer = other;
            }
        }
        if !answer.is_empty() {
            return Some(answer);
        }
    }
    if truthy(s::is_only_a_date(text_)) {
        if let Some(settled) = said(s::a_date_on_a_settled_rate(text_, memory)) {
            return Some(settled);
        }
        let money_question = py(s::rate_question(text_, memory));
        if money_question.truthy() {
            let answer = _the_money_answer(money_question.str(), memory);
            if !answer.is_empty() {
                return Some(answer);
            }
        }
    }
    if let Some(again) = said(s::another_sum_asked(text_, memory)) {
        let mut fresh = memory.clone();
        fresh.understood = None;
        let question = py(s::rate_question(&again, &fresh));
        let mut answer = _the_money_answer(question.str(), &mut fresh);
        if answer.is_empty() {
            answer = _the_money_answer(&again, &mut fresh);
        }
        if !answer.is_empty() {
            return Some(answer);
        }
    }
    None
}
