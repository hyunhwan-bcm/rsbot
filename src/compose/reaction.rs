//! Port of `bot/compose/reaction.py`: a line that is only a reaction to the answer above ("wow",
//! "seriously?") or a check on it ("is that right"), answered as that.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_strs, bundle_vals, inline_re};
use crate::py::text;
use crate::state::Memory;

bundle_vals!("bot.compose.reaction"; PHRASES);
bundle_sets!("bot.compose.reaction"; LEAD);
bundle_strs!("bot.compose.reaction"; AWE_REPLIES, DOUBT_REPLIES, CARE_REPLIES, CHECK_REPLIES);
bundle_res!("bot.compose.reaction"; NO_ANSWER, A_CARE_ANSWER, A_BODY_LINE, CHECK, OWN_PLAN, NO_STANCE, THEIR_REACTION);
inline_re!(A_WORD, r"[a-z']+");

/// Python: bot/compose/reaction.py:175 `_checking(text)`
///
/// Whether the line asks the bot to judge the answer it has just given.
pub fn _checking(text_: &str) -> bool {
    let said = text::squash(&text_.to_lowercase());
    if THEIR_REACTION.search(&said).is_some() {
        return false;
    }
    CHECK.search(&said).is_some()
}

/// Python: bot/compose/reaction.py:183 `_bare(text)`
///
/// Which kind of reaction a line that is nothing but a reaction is, or "".
pub fn _bare(text_: &str) -> String {
    let squashed = text::squash(&com4::straight(&text_.to_lowercase()));
    let clean = text::strip_chars(&squashed, " .,!?;:\"'");
    let words = A_WORD.findall_str(clean);
    if words.is_empty() || words.len() > 5 {
        return String::new();
    }
    for size in [3usize, 2, 1] {
        if size > words.len() {
            continue;
        }
        for start in 0..=words.len() - size {
            let Some(kind) = PHRASES.get(&words[start..start + size].join(" ")).map(|k| k.str()).filter(|k| !k.is_empty()) else {
                continue;
            };
            if words[..start].iter().chain(words[start + size..].iter()).all(|w| LEAD.contains(w)) {
                return kind.to_string();
            }
        }
    }
    String::new()
}

/// Python: bot/compose/reaction.py:201 `_an_answer(reply)`
///
/// True when the bot's last line was an answer rather than a nod, a question or a hello.
pub fn _an_answer(reply: &str) -> bool {
    let line = text::squash(reply);
    if text::len(&line) < 60 || line.ends_with('?') {
        return false;
    }
    NO_ANSWER.search(&line.to_lowercase()).is_none()
}

/// Python: bot/compose/reaction.py:213 `reply_for(text, memory)`
///
/// The reply to a line that is only a reaction to the answer above, or None.
pub fn reply_for(text_: &str, memory: &Memory) -> Option<String> {
    let mut kind = _bare(text_);
    if kind.is_empty() && _checking(text_) {
        // A check on the answer above, not a reaction.
        kind = "check".to_string();
    }
    if kind.is_empty() {
        return None;
    }
    let last = memory.turns.last()?;
    if !_an_answer(&last.bot) {
        return None;
    }
    let said = last.user.to_lowercase();
    let check = kind == "check";
    if check && OWN_PLAN.search(&said).is_some() {
        // A judgement asked about the person's own plan is theirs to weigh.
        return None;
    }
    if check {
        // The plan the composer already rules on is not a check on the answer either.
        let (plan, plan_line) = crate::compose::fallback::the_plan_and_its_line(memory);
        if !plan.is_empty() || !plan_line.is_empty() {
            return None;
        }
    }
    let above_text = last.bot.to_lowercase();
    if check && NO_STANCE.search(&above_text).is_some() {
        return None;
    }
    let care = A_BODY_LINE.search(&said).is_some() && A_CARE_ANSWER.search(&above_text).is_some();
    if check && care {
        // A check on a body answer is a question about the body, not a judgement of the advice.
        return None;
    }
    let pool: &Vec<String> = if check {
        &CHECK_REPLIES
    } else if care {
        &CARE_REPLIES
    } else if kind == "awe" {
        &AWE_REPLIES
    } else {
        &DOUBT_REPLIES
    };
    // The pick comes off the line, the answer it is reacting to and the length of the
    // conversation so far, and never off a clock.
    let seen = memory.turns.len();
    let above = text::squash(&last.bot.to_lowercase());
    let seed = format!("{}|{seen}|{above}", text::squash(&text_.to_lowercase()));
    let sum: i128 = seed.chars().enumerate().map(|(i, c)| (i as i128 + 1) * c as i128).sum();
    Some(pool[(sum % pool.len() as i128) as usize].clone())
}

/// Python: bot/compose/reaction.py:276 `main(argv=None)`
///
/// The module's command line: the kind of a bare reaction, or the reply to one after `--after`
/// an answer. `argv` None reads the process arguments.
pub fn main(argv: Option<&[String]>) {
    let args: Vec<String> = match argv {
        Some(a) => a.to_vec(),
        None => std::env::args().skip(1).collect(),
    };
    let said = args.first().cloned().unwrap_or_else(|| "wow".to_string());
    let mut after = String::new();
    if let Some(at) = args.iter().position(|a| a == "--after") {
        after = args.get(at + 1).cloned().unwrap_or_default();
    }
    if after.is_empty() {
        let kind = _bare(&said);
        println!("{}", if kind.is_empty() { "(nothing)".to_string() } else { kind });
        return;
    }
    let memory = Memory {
        turns: vec![crate::state::Turn { user: String::new(), bot: after, module: None }],
        ..Memory::new()
    };
    println!("{}", reply_for(&said, &memory).filter(|r| !r.is_empty()).unwrap_or_else(|| "(nothing)".to_string()));
}
