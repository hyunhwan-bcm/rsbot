//! Port of `bot/compose/tone_again.py`: "make it firmer", "shorter", "more polite" after the bot
//! handed over a draft - the same draft said again in that tone.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_strs, bundle_vals, format_named, inline_re};
use crate::py::random::Random;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

bundle_sets!("bot.compose.tone_again"; FIRM, POLITE, SHORT, FORMAL, CASUAL, TONE_EXTRA);
bundle_res!("bot.compose.tone_again"; FRAME, SOFT_WORDS, THE_ASK, DATE_ASKED, THANKED, HELLO, SIGN_OFF, LESS);
bundle_strs!("bot.compose.tone_again"; FIRM_LINES, POLITE_LINES, SHORT_LINES, FORMAL_LINES, CASUAL_LINES, BOTH_LINES);
bundle_vals!("bot.compose.tone_again"; SOFT, CONTRACTIONS, TURNED_OVER, ALREADY);
inline_re!(NOT_A_WORD_CHAR, r"[^a-z' -]");
inline_re!(A_WORD, r"[a-z']+");
inline_re!(QUOTED, r#""([^"]{8,400})""#);
inline_re!(CURLY_QUOTED, "\u{201c}([^\u{201d}]{8,400})\u{201d}");
inline_re!(PLEASE, r"\bplease\b", crate::py::re::I);
inline_re!(SENTENCE_BREAK, r"(?<=[.?!])\s+");

/// `(pattern, instead)` pairs, applied with `re.sub(pattern, instead, said, flags=re.I)`.
fn pairs(table: &'static crate::data::Value) -> Vec<(Re, String)> {
    table.items().iter().map(|p| (Re::new(p.items()[0].str(), crate::py::re::I), p.items()[1].str().to_string())).collect()
}
static SOFT_PAIRS: LazyLock<Vec<(Re, String)>> = LazyLock::new(|| pairs(&SOFT));
static CONTRACTION_PAIRS: LazyLock<Vec<(Re, String)>> = LazyLock::new(|| pairs(&CONTRACTIONS));

/// `said if said.endswith(("?", "!", ".")) else said + "."`.
fn with_a_stop(said: String) -> String {
    if said.ends_with(['?', '!', '.']) { said } else { format!("{said}.") }
}

/// Python: bot/compose/tone_again.py:100 `_clean(line)`
pub fn _clean(line: &str) -> String {
    let low = text::squash(&com4::straight(&line.to_lowercase()));
    NOT_A_WORD_CHAR.sub(" ", &low)
}

/// Python: bot/compose/tone_again.py:117 `_tones(low)`
///
/// The tones the line names, in the order they are read.
pub fn _tones(low: &str) -> Vec<String> {
    let turned: Vec<String> =
        LESS.findall_str(low).iter().filter_map(|w| TURNED_OVER.get(w).map(|v| v.str().to_string())).collect();
    let names = ["firm", "polite", "short", "formal", "casual"];
    let mut found: Vec<String> = names.iter().filter(|n| turned.iter().any(|t| t == *n)).map(|n| n.to_string()).collect();
    for word in A_WORD.findall_str(low) {
        for (name, group) in [("firm", &*FIRM), ("polite", &*POLITE), ("short", &*SHORT), ("formal", &*FORMAL), ("casual", &*CASUAL)] {
            if group.contains(&word) && !found.iter().any(|f| f == name) {
                found.push(name.to_string());
            }
        }
    }
    found
}

/// Python: bot/compose/tone_again.py:132 `_is_tone_only(low)`
///
/// True when nothing but a tone is named: no subject for anybody to look up.
pub fn _is_tone_only(low: &str) -> bool {
    A_WORD.findall_str(low).iter().all(|w| {
        FIRM.contains(w) || POLITE.contains(w) || SHORT.contains(w) || FORMAL.contains(w) || CASUAL.contains(w) || TONE_EXTRA.contains(w)
            || FRAME.fullmatch(w).is_some()
    })
}

/// Python: bot/compose/tone_again.py:141 `_the_draft(text)`
///
/// The message the bot handed over, or "" when the last reply held none.
pub fn _the_draft(text_: &str) -> String {
    for pattern in [&*QUOTED, &*CURLY_QUOTED] {
        if let Some(found) = pattern.search(text_) {
            return text::squash(found.str(1));
        }
    }
    String::new()
}

/// Python: bot/compose/tone_again.py:154 `_the_draft_near(memory)`
///
/// The draft in the last reply, or in the one before it.
pub fn _the_draft_near(memory: &Memory) -> String {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(2)..].iter().rev().map(|t| _the_draft(&t.bot)).find(|d| !d.is_empty()).unwrap_or_default()
}

/// Python: bot/compose/tone_again.py:169 `_firm(draft)`
///
/// The same message with the apology, the hedge and the "whenever" taken out.
pub fn _firm(draft: &str) -> String {
    let mut said = draft.to_string();
    for (pattern, instead) in SOFT_PAIRS.iter() {
        said = pattern.sub(instead, &said);
    }
    let said = text::strip(&SIGN_OFF.sub("", &text::squash(&said))).to_string();
    let mut said = said;
    if THE_ASK.search(&said).is_none() {
        said = format!("{}. Please confirm when it will be done", text::rstrip_chars(&said, " .;,"));
    }
    with_a_stop(text::rstrip_chars(&said, " .;,").to_string())
}

/// Python: bot/compose/tone_again.py:182 `_polite(draft)`
///
/// The same message with one courtesy on it: the ask is already there, the thanks are not.
pub fn _polite(draft: &str) -> String {
    let mut said = text::strip(&SIGN_OFF.sub("", &text::squash(draft))).to_string();
    if PLEASE.search(&said).is_none() && THE_ASK.search(&said).is_some() {
        said = text::rstrip_chars(&said, " .;,").to_string();
        if let Some(head) = said.strip_suffix('?') {
            said = format!("{}, please?", text::rstrip(head));
        } else {
            said.push_str(", please");
        }
    } else if THE_ASK.search(&said).is_none() {
        // A draft with no ask in it yet still gets one.
        said = format!("{}. Please let me know what happens next", text::rstrip_chars(&said, " .;,"));
    }
    let mut said = text::rstrip_chars(&said, " .;,").to_string();
    if THANKED.search(&said).is_none() {
        said.push_str(if said.ends_with(['?', '!']) { " Thank you." } else { ". Thank you" });
    }
    with_a_stop(said)
}

/// Python: bot/compose/tone_again.py:201 `_short(draft)`
///
/// The first sentence and the ask, which is what a shorter version keeps.
pub fn _short(draft: &str) -> String {
    let parts: Vec<String> = SENTENCE_BREAK.split(&text::squash(draft)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    match parts.len() {
        0 => String::new(),
        1 => parts[0].clone(),
        _ => {
            let mut kept = vec![parts[0].clone()];
            if let Some(asked) = parts[1..].iter().find(|p| THE_ASK.search(p).is_some()) {
                kept.push(asked.clone());
            }
            kept.join(" ")
        }
    }
}

/// Python: bot/compose/tone_again.py:212 `_formal(draft)`
pub fn _formal(draft: &str) -> String {
    let mut said = text::squash(draft);
    for (pattern, instead) in CONTRACTION_PAIRS.iter() {
        said = pattern.sub(instead, &said);
    }
    said = HELLO.sub("Dear Sir or Madam, ", &said);
    if !said.starts_with("Dear") {
        said = format!("Dear Sir or Madam, {said}");
    }
    said = SIGN_OFF.sub("Yours faithfully.", &said);
    text::squash(&said)
}

/// Python: bot/compose/tone_again.py:223 `_casual(draft)`
pub fn _casual(draft: &str) -> String {
    let said = HELLO.sub("Hi, ", &text::squash(draft));
    text::squash(&SIGN_OFF.sub("Cheers.", &said))
}

/// Python: bot/compose/tone_again.py:290 `retone(memory, line)`
///
/// The draft just given, said again in the tone this line asks for, or None.
pub fn retone(memory: &Memory, line: &str) -> Option<String> {
    let original = _the_draft_near(memory);
    if original.is_empty() {
        return None;
    }
    let low = _clean(line);
    if low.is_empty() || !_is_tone_only(&low) {
        return None;
    }
    let tones = _tones(&low);
    if tones.is_empty() {
        return None;
    }
    let has = |t: &str| tones.iter().any(|x| x == t);
    let (pool, draft): (&Vec<String>, String) = if has("firm") && has("polite") {
        (&BOTH_LINES, _polite(&_firm(&original)))
    } else if has("firm") {
        (&FIRM_LINES, _firm(&original))
    } else if has("short") {
        (&SHORT_LINES, _short(&original))
    } else if has("formal") {
        (&FORMAL_LINES, _formal(&original))
    } else if has("casual") {
        (&CASUAL_LINES, _casual(&original))
    } else {
        (&POLITE_LINES, _polite(&original))
    };
    let draft = text::strip(&text::squash(&draft)).to_string();
    if draft.is_empty() {
        return None;
    }
    // A draft the tone did not change: the draft is already like that, and saying so is a real
    // answer.
    if text::strip_chars(&draft, " .;,").to_lowercase() == text::strip_chars(&original, " .;,").to_lowercase() {
        let line = ALREADY.get(&tones[0]).or_else(|| ALREADY.get("polite")).expect("ALREADY has polite").str();
        return Some(format_named(line, &[("draft", &draft)]));
    }
    let seed = memory.turns.len() as i128 * 7919 + com4::ord_sum(&low);
    Some(format_named(Random::new(seed).choice(pool), &[("draft", &draft)]))
}

#[allow(dead_code)]
fn _unused() -> [&'static Re; 2] {
    [&SOFT_WORDS, &DATE_ASKED]
}
