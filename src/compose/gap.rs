//! Port of `bot/compose/gap.py`: a question the bot cannot answer outright (a price, a rule, a
//! distance, a wait) answered with what it can still say usefully about that kind of question.
#![allow(non_upper_case_globals, non_snake_case)]

use crate::compose::com4::{bundle_res, bundle_vals, inline_re};
use crate::data::Value;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.gap"; NAME_ONLY_ASK, A_LONGER_SPAN, ASKS_A_COST, AN_AMOUNT_ALREADY, A_COUNT_IN_A_SPAN,
    A_EVERY_SPAN, A_REPAIR_OR_BUY, A_BROKEN_THING, BARE_DURATION, A_QUEUE, NOT_A_RULE, A_DO_I_NEED, A_RULE_SUBJECT,
    A_HOWTO_HAS_A_SHAPE, NOT_A_HOWTO, NOT_A_RECORD, A_LOOSE_KIND_WORD, THEIR_OWN_PAPER, A_NAMED_DOCUMENT, NOT_HERE,
    ASKS_SOMETHING, A_POINTER, THE_PAPER_ITSELF);
bundle_vals!("bot.compose.gap"; COUNT_WORDS, A_YEAR, EXAMPLE, KINDS, WORRY_SHAPES, PROCEDURE_SHAPES);
inline_re!(MY_OWN_WAIT, concat!(r"\b(?:how long|how soon) (?:do|should|shall|can|will) (?:i|we) ",
    r"(?:wait|hold|hang|leave|give|keep|sit)\b"));

/// Python: bot/compose/gap.py:99 `times_a_year(clean)`
///
/// The count of a repeated spend in a year, and the span it was said in, or None for Python's
/// `(None, None)`.
pub fn times_a_year(clean: &str) -> Option<(i64, String)> {
    if ASKS_A_COST.search(clean).is_none() || AN_AMOUNT_ALREADY.search(clean).is_some() {
        return None;
    }
    if A_LONGER_SPAN.search(clean).is_none() {
        return None;
    }
    let year = |span: &str| A_YEAR.get(span).and_then(Value::as_int).unwrap_or_else(|| panic!("KeyError: {span:?}"));
    let Some(found) = A_COUNT_IN_A_SPAN.search(clean) else {
        // "a coffee every day" says the span and no count, which is one in each.
        let plain = A_EVERY_SPAN.search(clean)?;
        return Some((year(&plain.str(1).to_lowercase()), text::squash(plain.as_str())));
    };
    let (said, span) = (found.str(1).to_lowercase(), found.str(2).to_lowercase());
    let count = if text::is_digit(&said) {
        said.parse::<i64>().ok()
    } else {
        COUNT_WORDS.get(&said).and_then(Value::as_int)
    };
    let count = count.filter(|c| *c != 0)?;
    Some((count * year(&span), text::squash(found.as_str())))
}

/// Python: bot/compose/gap.py:121 `sum_lines(clean)`
///
/// The lines for a repeated spend asked for by the year, or None.
pub fn sum_lines(clean: &str) -> Option<Vec<String>> {
    let (times, said) = times_a_year(clean)?;
    let figure = times * EXAMPLE.as_int().expect("EXAMPLE is an int");
    let written = text::upper_first(&said);
    let (n, fig) = (text::thousands(times), text::thousands(figure));
    Some(vec![
        format!(
            "{written} is {n} a year. Put your own price on one - £5 each is £{fig} - and that is the figure to weigh against cutting it back."
        ),
        format!(
            "{written} is {n} times a year, so put a price on one: at a fiver each that is £{fig}, which is what the habit really costs."
        ),
    ])
}

/// Python: bot/compose/gap.py:136 `a_sum_through_the_year(clean)`
///
/// True when the line adds up a repeated spend over a longer span, which is not a quote.
pub fn a_sum_through_the_year(clean: &str) -> bool {
    times_a_year(clean).is_some()
}

/// Python: bot/compose/gap.py:199 `not_about_one_owned_thing(clean)`
///
/// True when a "worth it" line is not about a thing that is repaired or replaced.
pub fn not_about_one_owned_thing(clean: &str) -> bool {
    !(A_REPAIR_OR_BUY.search(clean).is_some() || A_BROKEN_THING.search(clean).is_some())
}

/// Python: bot/compose/gap.py:275 `NOT_A_QUEUE(text)`
///
/// True when the wait is not somebody else's queue: a bare follow-up, a body, a pan.
pub fn NOT_A_QUEUE(text_: &str) -> bool {
    if BARE_DURATION.search(text_).is_some() || MY_OWN_WAIT.search(text_).is_some() {
        return true;
    }
    A_QUEUE.search(text_).is_none()
}

/// Python: bot/compose/gap.py:363 `not_a_bare_procedure(clean)`
///
/// True when a "do i need to" line names nothing a rule is about.
pub fn not_a_bare_procedure(clean: &str) -> bool {
    A_DO_I_NEED.search(clean).is_some() && A_RULE_SUBJECT.search(clean).is_none()
}

/// Python: bot/compose/gap.py:374 `not_a_rule(clean)`
///
/// Both guards for the rule kind: the board game, and the bare procedure.
pub fn not_a_rule(clean: &str) -> bool {
    NOT_A_RULE.search(clean).is_some() || not_a_bare_procedure(clean)
}

/// Python: bot/compose/gap.py:441 `NOT_A_HOW_TO(text)`
///
/// True when the steps do not fit the ask: a feeling, a person, or no trouble in it.
pub fn NOT_A_HOW_TO(text_: &str) -> bool {
    NOT_A_HOWTO.search(text_).is_some() || A_HOWTO_HAS_A_SHAPE.search(text_).is_none()
}

/// Python: bot/compose/gap.py:510 `_not_a_record(said)`
///
/// True when this line is not about the entry itself, so the record kind stands down.
pub fn _not_a_record(said: &str) -> bool {
    if NOT_A_RECORD.search(said).is_some() {
        return true;
    }
    if A_NAMED_DOCUMENT.search(said).is_some() || THEIR_OWN_PAPER.search(said).is_some() {
        return false;
    }
    A_LOOSE_KIND_WORD.search(said).is_some()
}

/// Python: bot/compose/gap.py:586 `names_only_a_page(reply)`
///
/// True when a reply is a shrug with a search box or a page named as where to look.
pub fn names_only_a_page(reply: &str) -> bool {
    let said = text::squash(&reply.to_lowercase());
    !said.is_empty() && A_POINTER.search(&said).is_some()
}

/// Python: bot/compose/gap.py:609 `names_the_paper(reply)`
///
/// True when the reply already says the person's own paper is where the answer is.
pub fn names_the_paper(reply: &str) -> bool {
    let said = text::squash(&reply.to_lowercase());
    !said.is_empty() && THE_PAPER_ITSELF.search(&said).is_some()
}

/// One guard of a kind: a function of this file (by its Python id) or a compiled pattern.
fn guard_says_not_mine(guard: &Value, clean: &str) -> bool {
    if let Some(f) = guard.as_fn() {
        return match f.name() {
            "bot.compose.gap:NOT_A_HOW_TO" => NOT_A_HOW_TO(clean),
            "bot.compose.gap:not_about_one_owned_thing" => not_about_one_owned_thing(clean),
            "bot.compose.gap:not_a_rule" => not_a_rule(clean),
            "bot.compose.gap:NOT_A_QUEUE" => NOT_A_QUEUE(clean),
            "bot.compose.gap:_not_a_record" => _not_a_record(clean),
            "bot.compose.gap:not_a_bare_procedure" => not_a_bare_procedure(clean),
            "bot.compose.gap:a_sum_through_the_year" => a_sum_through_the_year(clean),
            other => panic!("gap: no port of the guard {other}"),
        };
    }
    guard.as_re().expect("a guard is a function or a pattern").search(clean).is_some()
}

/// Python: bot/compose/gap.py:615 `reads(text)`
///
/// The kind of question this is, and the lines for it, or None for Python's `(None, None)`.
pub fn reads(text_: &str) -> Option<(String, Vec<String>)> {
    let clean = text::squash(&text_.to_lowercase());
    if clean.is_empty() || NOT_HERE.search(&clean).is_some() {
        return None;
    }
    if ASKS_SOMETHING.search(&clean).is_none() && NAME_ONLY_ASK.search(&clean).is_none() {
        return None;
    }
    for row in WORRY_SHAPES.items().iter().chain(KINDS.items()).chain(PROCEDURE_SHAPES.items()) {
        let (kind, pattern, lines, guard) = (&row.items()[0], &row.items()[1], &row.items()[2], &row.items()[3]);
        if pattern.as_re().expect("a kind's pattern").search(&clean).is_some() {
            if guard.truthy() {
                let guards: Vec<&Value> = match guard {
                    Value::Tuple(items) => items.iter().collect(),
                    one => vec![one],
                };
                if guards.iter().any(|g| guard_says_not_mine(g, &clean)) {
                    continue;
                }
            }
            return Some((kind.str().to_string(), lines.strs()));
        }
    }
    None
}

/// Python: bot/compose/gap.py:636 `pool(clean, memory=None)`
///
/// The lines for a gap the bot can still say something useful about, or None.
pub fn pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let _ = memory;
    if let Some(summed) = sum_lines(clean) {
        return Some(summed);
    }
    reads(clean).map(|(_, lines)| lines)
}
