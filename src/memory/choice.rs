//! Port of `bot/memory/choice.py`: when the person hands the decision back ("you pick", "your
//! call", "either is fine"), one of the options the bot offered a turn ago, read out of its own
//! last reply; and the thing they settled on, said back on a deciding ask.

use super::pyv;
use super::{last_bot, recent};
use crate::data;
use crate::py::random::Random;
use crate::py::re::{self, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.choice.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn words_of(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}

static _PICK_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("_PICK_ASK"));
static _CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| rx("_CLAUSE"));
static _TAIL_STRIP: LazyLock<&'static Re> = LazyLock::new(|| rx("_TAIL_STRIP"));
static _STEP: LazyLock<&'static Re> = LazyLock::new(|| rx("_STEP"));
static _NOT_A_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("_NOT_A_THING"));
static PICKS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.choice.PICKS"));
static _ASKED: LazyLock<&'static Re> = LazyLock::new(|| rx("_ASKED"));
static _BARE_HAND_BACK: LazyLock<&'static Re> = LazyLock::new(|| rx("_BARE_HAND_BACK"));
static _PICK_SAID: LazyLock<Vec<&'static Re>> =
    LazyLock::new(|| data::get("bot.memory.choice._PICK_SAID").items().iter().map(|v| v.as_re().unwrap()).collect());
static _INSIST_SAID: LazyLock<&'static Re> = LazyLock::new(|| rx("_INSIST_SAID"));
static _INSIST_WRAP: LazyLock<&'static Re> = LazyLock::new(|| rx("_INSIST_WRAP"));
static INSISTED: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.choice.INSISTED"));
static BY_A_RULE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.choice.BY_A_RULE"));
static _A_MEASURE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_MEASURE"));
static DECIDING_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("DECIDING_ASK"));
static STRONG_CHANGE: LazyLock<&'static Re> = LazyLock::new(|| rx("STRONG_CHANGE"));
static WEAK_CHANGE: LazyLock<&'static Re> = LazyLock::new(|| rx("WEAK_CHANGE"));
static CUE: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("CUE"));
static SECOND_FRAME: LazyLock<&'static Re> = LazyLock::new(|| rx("SECOND_FRAME"));
static NOT_AT_THE_START: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("NOT_AT_THE_START"));
static DEFINITE: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("DEFINITE"));
static TRAIL: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("TRAIL"));
static CORRECTION: LazyLock<&'static Re> = LazyLock::new(|| rx("CORRECTION"));
static A_CHOICE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_CHOICE"));
static BARE_CHANGE: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("BARE_CHANGE"));
static DETERMINERS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("DETERMINERS"));
static NO_THEN_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("NO_THEN_THING"));
static CORRECTION_OPENER: LazyLock<&'static Re> = LazyLock::new(|| rx("CORRECTION_OPENER"));
static NO_CONTINUES: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("NO_CONTINUES"));
static VERBISH: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("VERBISH"));
static NOT_THE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("NOT_THE_THING"));
static SETTLED: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.choice.SETTLED"));

static OPENS_WITH_A_JOINER: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:or|and|either|then|else)\s+", re::I | re::U));
static ITALICS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\*([^*\n]{2,40})\*(?: ([^,;.\n]{1,30}))?", re::U));
static NEXT_SENTENCE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\.\s+[A-Z]", re::U));
static SEMICOLON: LazyLock<Re> = LazyLock::new(|| Re::new(r";", re::U));
static LIST_JOINS: LazyLock<Re> = LazyLock::new(|| Re::new(r",\s+|\s+or\s+|\s+and\s+", re::U));
static COMMA: LazyLock<Re> = LazyLock::new(|| Re::new(r",\s+", re::U));
static LABEL: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:^|[.;]\s+)([A-Za-z][^:.;\n]{2,30}):", re::U));
static EITHER_OR_LEAD: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"^\W*(?:so |then )?(?:would you rather|would you prefer|shall we|",
            r"do you want|do you fancy|which would you rather|are you (?:thinking|",
            r"leaning)|how about|which one)\s+"
        ),
        re::I | re::U,
    )
});
static OR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+or\s+", re::U));
static SHOULD_I_LEAD: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"^\W*(?:(?:so|ok|okay|but)[, ]+)?(?:should|could|do|would|shall|can|will)\s+",
            r"(?:i|we)\s+|^\W*(?:is it better to|which is better,?|what about|",
            r"is it worth)\s+"
        ),
        re::U,
    )
});
static AN_ARTICLE: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:a|an|the)\b", re::U));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[A-Za-z']+", re::U));

const OPTION_EDGE: &str = " \t*_-\u{2014}";
const PEEL_EDGE: &str = " *_\u{2014},.;:";

/// The options with a repeat (compared in lower case) dropped, in order.
fn unique(options: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    options.into_iter().filter(|option| seen.insert(option.to_lowercase())).collect()
}

/// Python: bot/memory/choice.py:60 `_clean_option(piece)`
/// One option, trimmed, or "" when the piece is a clause rather than a thing to choose.
pub fn _clean_option(piece: &str) -> String {
    let piece = text::strip_chars(piece, OPTION_EDGE);
    let piece = OPENS_WITH_A_JOINER.sub("", piece);
    let piece = _TAIL_STRIP.sub("", &piece);
    let words = text::split_ref(&piece);
    if piece.is_empty() || words.len() > 11 || text::len(&piece) > 70 {
        return String::new();
    }
    if _CLAUSE.is_match(&piece) {
        return String::new();
    }
    if piece.ends_with(['?', '!', ':']) || piece.ends_with("...") {
        return String::new();
    }
    piece
}

/// Python: bot/memory/choice.py:75 `_from_italics(text)`
/// Options marked out as titles: "*Groundhog Day* for comedy, or *Arrival* ...".
pub fn _from_italics(text_: &str) -> Vec<String> {
    let mut options = Vec::new();
    for found in ITALICS.finditer(text_) {
        let title = _clean_option(found.str(1));
        if title.is_empty() {
            continue;
        }
        // A tail that is a clause ("if you want energy") is the bot's reason, not part of the
        // name: keep the name alone so the pick can still be said.
        let tail = if _CLAUSE.is_match(found.str(2)) { "" } else { found.str(2) };
        let whole = _clean_option(text::strip(&format!("{title} {tail}")));
        options.push(if whole.is_empty() { title } else { whole });
    }
    options
}

/// Python: bot/memory/choice.py:90 `_segment(text)`
/// The part of a line that could hold the list: after a colon or a dash, else all of it.
pub fn _segment(text_: &str) -> String {
    for sep in [":", " \u{2014} ", " - "] {
        let (_, found, tail) = text::rpartition(text_, sep);
        if !found.is_empty() && text::split_ref(tail).len() >= 2 {
            return tail.to_string();
        }
    }
    text_.to_string()
}

/// `things(pieces, steps=False)` in `_from_list`: the pieces that are a thing to choose, not a
/// step or the tail of a sentence. Only a piece split off by a comma alone is checked for step
/// words.
fn things(pieces: &[String], steps: bool) -> Vec<String> {
    let mut out = Vec::new();
    for piece in pieces {
        if _NOT_A_THING.is_match(piece) || (steps && _STEP.is_match(piece)) {
            continue;
        }
        let option = _clean_option(piece);
        if !option.is_empty() {
            out.push(option);
        }
    }
    out
}

fn pieces_of(splitter: &Re, body: &str) -> Vec<String> {
    splitter.split(body).into_iter().filter(|piece| !text::strip(piece).is_empty()).collect()
}

/// Python: bot/memory/choice.py:100 `_from_list(text)`
/// A plain list: "pasta with garlic and oil; eggs on toast; or cold rice fried with veg".
pub fn _from_list(text_: &str) -> Vec<String> {
    let body = _segment(text_);
    // A sentence that follows the list is not one of the options.
    let body = NEXT_SENTENCE.split(&body).swap_remove(0);
    // Only a reply that offers a choice is read as one.
    if body.to_lowercase().contains(" or ") || body.contains(';') {
        let mut options = things(&pieces_of(&SEMICOLON, &body), false);
        if options.len() < 2 {
            options = things(&pieces_of(&LIST_JOINS, &body), false);
        }
        if options.len() >= 2 {
            return options;
        }
    }
    // A comma list is an offer when it is three or more short things.
    let comma: Vec<String> =
        things(&pieces_of(&COMMA, &body), true).into_iter().filter(|o| text::split_ref(o).len() <= 8).collect();
    if comma.len() >= 3 { comma } else { Vec::new() }
}

/// Python: bot/memory/choice.py:140 `_from_labels(text)`
/// A reply that answers with named dishes or options and explains each: the names are the
/// things to choose between, not the explanations.
pub fn _from_labels(text_: &str) -> Vec<String> {
    let labels: Vec<String> =
        LABEL.findall_str(text_).iter().map(|label| _clean_option(label)).filter(|option| !option.is_empty()).collect();
    unique(labels)
}

/// Python: bot/memory/choice.py:156 `_from_question(text)`
/// The bot's own either/or question: "Would you rather go Saturday or Sunday?".
pub fn _from_question(text_: &str) -> Vec<String> {
    if !text::rstrip(text_).ends_with('?') || !text_.to_lowercase().contains(" or ") {
        return Vec::new();
    }
    let stripped = text::strip(text_);
    if !EITHER_OR_LEAD.is_match(stripped) {
        return Vec::new();
    }
    let body = EITHER_OR_LEAD.sub("", stripped);
    let body = text::rstrip_chars(&body, "?");
    OR.split(body).iter().map(|piece| _clean_option(piece)).filter(|option| !option.is_empty()).collect()
}

/// Python: bot/memory/choice.py:169 `options_in(reply)`
/// The options the bot offered in one reply, or [] when it did not offer a choice.
pub fn options_in(reply: &str) -> Vec<String> {
    if reply.is_empty() {
        return Vec::new();
    }
    let text_ = text::squash(reply);
    let readers: [fn(&str) -> Vec<String>; 4] = [_from_italics, _from_labels, _from_question, _from_list];
    for reader in readers {
        // The same thing twice ("the pasta, pasta") is not a choice.
        let options = unique(reader(&text_));
        if options.len() >= 2 {
            return options;
        }
    }
    Vec::new()
}

/// Python: bot/memory/choice.py:188 `asks_for_a_pick(text)`
/// True when the whole line hands the choice back to the bot.
pub fn asks_for_a_pick(text_: &str) -> bool {
    _PICK_ASK.match_(&text::squash(&text_.to_lowercase())).is_some()
}

/// `" ".join(o.split()[1:])`.
fn after_the_first_word(option: &str) -> String {
    text::split_ref(option)[1..].join(" ")
}

/// Python: bot/memory/choice.py:211 `_from_user_line(memory)`
/// The choice the person put on the table themselves: "should i get a cat or a dog".
pub fn _from_user_line(memory: &Memory) -> Vec<String> {
    let turns = recent(Some(memory), 1);
    let line = text::squash(&turns.last().map(|turn| turn.user.to_lowercase()).unwrap_or_default());
    if !line.contains(" or ") {
        return Vec::new();
    }
    let body = SHOULD_I_LEAD.sub("", text::rstrip_chars(&line, "?"));
    let mut options = Vec::new();
    for piece in OR.split(&body) {
        let option = _clean_option(&piece);
        if !option.is_empty() && text::split_ref(&option).len() <= 5 {
            options.push(option);
        }
    }
    if options.len() < 2 {
        return Vec::new();
    }
    // "get a cat or get a dog" is a choice between the cat and the dog: the shared verb is the
    // shape of the question, not part of the answer.
    let first: HashSet<&str> = options.iter().map(|o| text::split_ref(o)[0]).collect();
    if first.len() == 1 && options.iter().all(|o| text::split_ref(o).len() > 1) {
        let rest: Vec<String> = options.iter().map(|o| after_the_first_word(o)).collect();
        if rest.iter().all(|r| r.chars().next().is_some_and(char::is_alphabetic)) {
            options = rest;
        }
    }
    // "get a cat or a dog": the determiner on the second one says the first verb is shared.
    if text::split_ref(&options[0]).len() > 1 && AN_ARTICLE.match_(&options[1]).is_some() {
        options =
            options.iter().map(|o| if text::split_ref(o).len() > 1 { after_the_first_word(o) } else { o.clone() }).collect();
    }
    options
}

/// Python: bot/memory/choice.py:245 `_asked_for_help(memory, n=4)`
/// True when the person has asked for suggestions in the last few turns.
pub fn _asked_for_help(memory: &Memory, n: i64) -> bool {
    recent(Some(memory), n).iter().rev().any(|turn| {
        let line = text::squash(&turn.user);
        !line.is_empty() && _ASKED.match_(&line).is_some()
    })
}

/// Python: bot/memory/choice.py:311 `_peeled(option)`
pub fn _peeled(option: &str) -> String {
    let mut option = text::strip_chars(option, PEEL_EDGE).to_string();
    for _ in 0..3 {
        let Some(found) = _INSIST_WRAP.match_(&option) else { break };
        option = text::strip_chars(found.str(1), PEEL_EDGE).to_string();
    }
    option
}

/// Python: bot/memory/choice.py:330 `_readable(options)`
/// The options a person could actually be choosing between: a name, not a measurement and not
/// a mouthful that runs to a whole clause.
pub fn _readable(options: Vec<String>) -> Vec<String> {
    options
        .into_iter()
        .filter(|option| {
            let text_ = text::strip(option);
            !(text_.is_empty() || text::split_ref(text_).len() > 6 || _A_MEASURE.is_match(text_))
        })
        .collect()
}

/// Python: bot/memory/choice.py:346 `_already_picked(memory, back=3)`
/// The one thing the bot said it would choose, if it said one a turn or two ago.
pub fn _already_picked(memory: &Memory, back: i64) -> Option<String> {
    for turn in recent(Some(memory), back).iter().rev() {
        let said = text::squash(&turn.bot);
        if said.is_empty() || said.ends_with('?') {
            continue;
        }
        for pattern in std::iter::once(&*_INSIST_SAID).chain(_PICK_SAID.iter()) {
            let Some(found) = pattern.match_(&said) else { continue };
            let option = _peeled(found.str(1));
            if !option.is_empty() && text::split_ref(&option).len() <= 8 && !option.contains('?') {
                return Some(option);
            }
        }
    }
    None
}

/// Python: bot/memory/choice.py:363 `reply_for(text, memory)`
/// One option from what the bot just offered, or None when there was no choice to make.
pub fn reply_for(text_: &str, memory: &Memory) -> Option<String> {
    let asked = asks_for_a_pick(text_);
    let shrug = !asked && _BARE_HAND_BACK.match_(&text::squash(&text_.to_lowercase())).is_some();
    if !(asked || shrug) {
        return None;
    }
    let turns = memory.turns.len() as i128;
    let picked = _already_picked(memory, 3);
    let mut options = _readable(options_in(&last_bot(memory)));
    if options.len() < 2 {
        options = _readable(_from_user_line(memory));
    }
    if options.len() >= 2 {
        let short: Vec<String> = options.iter().filter(|o| text::split_ref(o).len() <= 8).cloned().collect();
        if !short.is_empty() {
            options = short;
        }
        let chosen = Random::new(turns).choice(&options).clone();
        let line = Random::new(turns + 7).randrange(PICKS.len());
        return Some(text::upper_first(&PICKS[line].replace("{option}", &chosen)));
    }
    if let Some(picked) = picked {
        // "you decide" then "up to you": the pick is on the screen, so it is given again.
        return Some(pyv::pick(&INSISTED, memory, text_).replace("{option}", &picked));
    }
    if _asked_for_help(memory, 4) {
        // Handed a choice to make with nothing listed: the rule to decide by, not a search hedge.
        return Some(pyv::pick(&BY_A_RULE, memory, text_));
    }
    None
}

/// Python: bot/memory/choice.py:502 `_the_thing_in(line)`
/// The thing named after the word that says they changed their mind, or "".
pub fn _the_thing_in(line: &str) -> String {
    let mut weak = false;
    let mut said = STRONG_CHANGE.search(line);
    if said.is_none() {
        said = WEAK_CHANGE.search(line);
        weak = said.is_some();
    }
    let said = match said {
        None => {
            // "no, a dog": the correction said on its own, with nothing after it but the thing.
            match NO_THEN_THING.match_(line) {
                Some(no) if text::split_ref(&line[no.end()..]).len() <= 3 => no,
                _ => return String::new(),
            }
        }
        Some(said) => {
            if weak && !_hands_the_choice_over(&line[..said.start()]) {
                return String::new();
            }
            said
        }
    };
    let mut tail = &line[said.end()..];
    // "actually no, a scooter" says its own "no" after the change word, and the thing is what
    // follows it.
    for _ in 0..2 {
        let rest = CORRECTION_OPENER.match_(tail).map(|opened| &tail[opened.end()..]).unwrap_or("");
        if text::strip(rest).is_empty()
            || NO_CONTINUES.contains(text::strip_chars(&text::split_ref(rest)[0].to_lowercase(), ",.!?"))
        {
            break;
        }
        tail = rest;
    }
    for _ in 0..3 {
        let Some(frame) = SECOND_FRAME.match_(tail) else { break };
        tail = &tail[frame.end()..];
    }
    if CORRECTION.match_(tail).is_some() {
        // "actually its the radiator" says what something is, and no thing has been chosen.
        return String::new();
    }
    let mut kept: Vec<&str> = Vec::new();
    let mut lead = String::new();
    for word in text::split_ref(tail) {
        let word = text::strip_chars(word, ",.!?;:'\"()");
        if word.is_empty() {
            continue;
        }
        let low = word.to_lowercase();
        if DETERMINERS.contains(&low) && kept.is_empty() {
            // The article is part of the name of the thing, not part of the sentence around it.
            lead = low;
            continue;
        }
        if NOT_THE_THING.contains(&low) && kept.is_empty() {
            // "the small one" keeps its "one": only the words before the thing are skipped.
            continue;
        }
        if !kept.is_empty() && TRAIL.contains(&low) {
            break;
        }
        kept.push(word);
        if kept.len() == 3 {
            break;
        }
    }
    if kept.is_empty() && said.as_str().to_lowercase() == "instead" {
        // The change word can come after the thing: "no a kitten instead" is a kitten.
        return _the_thing_in(&line[..said.start()]);
    }
    if kept.is_empty() || VERBISH.contains(&kept[0].to_lowercase()) {
        return String::new();
    }
    if NOT_AT_THE_START.contains(&kept[0].to_lowercase()) {
        return String::new();
    }
    if weak && !DEFINITE.contains(&lead) {
        // "the grey" is a thing on offer and "a run" is what somebody did this morning.
        return String::new();
    }
    let mut out: Vec<&str> = Vec::new();
    if !lead.is_empty() {
        out.push(&lead);
    }
    out.extend(kept);
    out.join(" ")
}

/// Python: bot/memory/choice.py:573 `_hands_the_choice_over(before)`
/// True when the words before a weak change word are only the ones that hand a choice over.
pub fn _hands_the_choice_over(before: &str) -> bool {
    let words_ = LETTERS.findall_str(before);
    if words_.is_empty() {
        return true;
    }
    words_.len() <= 2 && words_.iter().all(|word| CUE.contains(&word.to_lowercase()))
}

/// Python: bot/memory/choice.py:581 `settled_on(memory)`
/// The thing the person last settled on, in their own words, or "". Read from the last few
/// things they said, newest first, and only from a line that carries a change of mind.
pub fn settled_on(memory: &Memory) -> String {
    for turn in recent(Some(memory), 3).iter().rev() {
        let line = text::squash(&turn.user);
        if line.is_empty() {
            continue;
        }
        let thing = _the_thing_in(&line);
        if thing.is_empty() {
            continue;
        }
        if !_bare_change(&line).is_empty() && !_a_choice_in_hand(memory, &line) {
            // "im 34" / "actually 35": a bare change word alone does not say a choice was made.
            continue;
        }
        return thing;
    }
    String::new()
}

/// Python: bot/memory/choice.py:602 `_bare_change(line)`
/// The change word when every one the line carries is an ordinary English one, or "".
pub fn _bare_change(line: &str) -> String {
    let flat = |found: &str| text::strip(found).to_lowercase();
    for found in STRONG_CHANGE.finditer(line) {
        if !BARE_CHANGE.contains(&flat(found.as_str())) {
            return String::new();
        }
    }
    STRONG_CHANGE.search(line).map(|first| flat(first.as_str())).unwrap_or_default()
}

/// Python: bot/memory/choice.py:615 `_a_choice_in_hand(memory, this)`
/// True when an earlier line of the chat put a choice on the table.
pub fn _a_choice_in_hand(memory: &Memory, this: &str) -> bool {
    for turn in recent(Some(memory), 8).iter().rev() {
        let line = text::squash(&turn.user);
        if line.is_empty() || line == this {
            continue;
        }
        if A_CHOICE.is_match(&line) || line.to_lowercase().contains(" or ") {
            return true;
        }
    }
    false
}

/// Python: bot/memory/choice.py:626 `the_decision(text, memory)`
/// The thing they settled on, said back, on a deciding ask. None on every other line.
pub fn the_decision(text_: &str, memory: &Memory) -> Option<String> {
    let line = text::squash(text_);
    DECIDING_ASK.match_(&line)?;
    let thing = settled_on(memory);
    if thing.is_empty() {
        return None;
    }
    Some(text::upper_first(&_a_way_of_saying_it(&thing, memory)))
}

/// Python: bot/memory/choice.py:638 `_a_way_of_saying_it(thing, memory)`
/// One of the SETTLED lines, not the one used on the last deciding ask.
pub fn _a_way_of_saying_it(thing: &str, memory: &Memory) -> String {
    let last = text::squash(&last_bot(memory)).to_lowercase();
    let said = |frame: &str| frame.replace("{thing}", thing);
    let mut frames: Vec<&String> = SETTLED.iter().filter(|frame| said(frame).to_lowercase() != last).collect();
    if frames.is_empty() {
        frames = SETTLED.iter().collect();
    }
    said(Random::new(memory.turns.len() as i128).choice(&frames).as_str())
}

/// Python: bot/memory/choice.py:646 `offered(memory, n=1)`
/// The options the bot offered over its last n replies, oldest first.
pub fn offered(memory: &Memory, n: i64) -> Vec<String> {
    recent(Some(memory), n).iter().flat_map(|turn| options_in(&turn.bot)).collect()
}
