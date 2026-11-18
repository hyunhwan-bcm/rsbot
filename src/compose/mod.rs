//! Port of PyBot's `bot/compose` package: the parts the intent observer calls. Each Python file
//! `bot/compose/<name>.py` is `<name>.rs` here.

pub mod a_count_of_packets;
pub mod a_detail_dropped;
pub mod a_document_lives_with_an_office;
pub mod a_first_time;
pub mod a_number_the_bot_knows;
pub mod a_place_nobody_can_see;
pub mod a_thing_being_weighed;
pub mod a_thing_wants_doing;
pub mod a_trouble;
pub mod a_wait_about_a_body;
pub mod aboutness;
pub mod badnews;
pub mod breakup;
pub mod check_with_you;
pub mod checking;
pub mod childwords;
pub mod circumstance;
pub mod colour_on_the_body;
pub mod com4;
pub mod doubt;
pub mod draft;
pub mod facts;
pub mod fallback;
pub mod fallback_chain;
pub mod fallback_data;
pub mod fallback_pools;
pub mod fallback_words;
pub mod find_a_place;
pub mod finding_out;
pub mod findout;
pub mod for_the_day;
pub mod gap;
pub mod goodnews;
pub mod griefplea;
pub mod habit;
pub mod handoff;
pub mod hands;
pub mod harm;
pub mod hedge;
pub mod how_it_went;
pub mod invitation;
pub mod is_it_worth_it;
pub mod judgement;
pub mod kept_nearby;
pub mod lead;
pub mod lead_in;
pub mod leaving;
pub mod limit;
pub mod liveask;
pub mod loss;
pub mod missing_half;
pub mod morning_after;
pub mod moving;
pub mod nearby_one;
pub mod news_ask;
pub mod no_advice;
pub mod no_eyes;
pub mod no_page;
pub mod not_knowing;
pub mod nottheirbest;
pub mod occasionplan;
pub mod onething;
pub mod only_they_know;
pub mod onmybehalf;
pub mod onscreen;
pub mod onsubject;
pub mod outlook;
pub mod own_ask;
pub mod own_facts;
pub mod pace;
pub mod place_facts;
pub mod plan_ask;
pub mod plans;
pub mod pleased;
pub mod postcode;
pub mod prices;
pub mod quantity;
pub mod reaction;
pub mod reading;
pub mod recall;
pub mod receiving_end;
pub mod remark;
pub mod repair;
pub mod rotten_luck;
pub mod routes;
pub mod rule;
pub mod ruledout;
pub mod safety_doubt;
pub mod safetyask;
pub mod said_about_me;
pub mod saidby;
pub mod setting_off;
pub mod stepwords;
pub mod still;
pub mod stretches;
pub mod talktime;
pub mod talkto;
pub mod taste;
pub mod the_course_of_it;
pub mod the_rule_for_it;
pub mod the_shape_without_the_fact;
pub mod the_trouble_named;
pub mod the_turn_after;
pub mod their_day;
pub mod their_detail;
pub mod their_life;
pub mod theirperson;
pub mod thread_detail;
pub mod toknow;
pub mod tone_again;
pub mod trouble;
pub mod unseen;
pub mod urgent;
pub mod verdict;
pub mod waited_for;
pub mod ward;
pub mod what_decides_it;
pub mod what_i_do_know;
pub mod who;
pub mod who_to_see;
pub mod whole;
pub mod why;
pub mod worn;
pub mod clause;
pub mod fit;
pub mod what_it_asks;



use crate::data::Value;
use crate::py::random::Random;
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};

com4::bundle_sets!("bot.compose"; NEGATORS, NEARLY, AUX, CONTRACTED_AUX, DOUBT, CANNOT, SURE, DOUBTED, BELIEVE_IT,
    _OPENS_A_SENTENCE, _NOT_A_FRAGMENT);
com4::bundle_res!("bot.compose"; SAYS_IT_HAS_LINGERED, STILL_AT_IT, _SHORT_RUN, _PERSON_WORD, _CAPITAL_I,
    _ONE_WORD_THEN_A_BREAK, OPENS_ON_A_FRAGMENT_QUESTION, A_STEP_ASK);
com4::bundle_strs!("bot.compose"; LINGERED_LINE, _HELLO_OPENS, STEP_WHEN_LOW);
com4::inline_re!(A_WORD, r"[a-z']+");

/// Python: bot/compose/__init__.py:43 `says_it_has_lingered(clean, illness=False)`
///
/// True when the line says the thing has been going long enough to be a different question.
/// `illness` is kept for the callers that already know they read a cold or a bug, and it no
/// longer decides on its own.
pub fn says_it_has_lingered(clean: &str, illness: bool) -> bool {
    let _ = illness;
    SAYS_IT_HAS_LINGERED.search(clean).is_some() && _SHORT_RUN.search(clean).is_none()
}

/// `word.replace("'", "").replace("’", "")`.
fn no_apostrophe(word: &str) -> String {
    word.replace('\'', "").replace('\u{2019}', "")
}

/// Python: bot/compose/__init__.py:84 `negator(word)`
///
/// True when the word turns the next one round: "not", "didn't", "can't".
pub fn negator(word: &str) -> bool {
    let plain = no_apostrophe(word);
    if NEGATORS.contains(&plain) {
        return true;
    }
    // "didnt" is "didn't", and only an auxiliary in front of the "nt" says so: see
    // CONTRACTED_AUX for why the "nt" alone is not enough.
    text::len(&plain) > 3 && plain.ends_with("nt") && CONTRACTED_AUX.contains(&plain[..plain.len() - 2])
}

/// Python: bot/compose/__init__.py:94 `turned(words)`
///
/// True when these words turn the next one round: "not", "didn't", and also "can t", which is
/// how a line that lost its apostrophe splits into two tokens.
pub fn turned<S: AsRef<str>>(words: &[S]) -> bool {
    for (i, word) in words.iter().enumerate() {
        let word = word.as_ref();
        if negator(word) {
            return true;
        }
        if word == "t" && i != 0 && AUX.contains(words[i - 1].as_ref()) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/__init__.py:112 `turns_round(words)`
///
/// True when these words (the ones just before a cue) turn it round. A doubt word after
/// can't/couldn't/never/always is an intensifier, not a negation.
pub fn turns_round<S: AsRef<str>>(words: &[S]) -> bool {
    if !turned(words) {
        return false;
    }
    let plain: Vec<String> = words.iter().map(|w| no_apostrophe(w.as_ref())).collect();
    if plain.iter().any(|w| DOUBT.contains(w)) && plain.iter().any(|w| CANNOT.contains(w)) {
        return false;
    }
    true
}

/// Python: bot/compose/__init__.py:128 `negated(clean, phrase, window=3)`
///
/// True when a negator stands just before the phrase: "im not feeling great". A phrase that
/// opens with a negator is a shape of its own ("not ok" is a feeling), so it is left alone.
pub fn negated(clean: &str, phrase: &str, window: i64) -> bool {
    let parts = text::split_ref(phrase);
    if !parts.is_empty() && negator(parts[0]) {
        return false;
    }
    let rx = Re::new(&format!(r"\b{}\b", pyre::escape(phrase)), 0);
    for m in rx.finditer(clean) {
        let before = text::split_ref(&clean[..m.start()]);
        if turns_round(com4::tail(&before, window)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/__init__.py:142 `nearly(clean, phrase, window=2)`
///
/// True when "nearly" or "almost" stands just before the phrase: "im almost happy".
pub fn nearly(clean: &str, phrase: &str, window: i64) -> bool {
    let parts = text::split_ref(phrase);
    if !parts.is_empty() && NEARLY.contains(parts[0]) {
        return false;
    }
    let rx = Re::new(&format!(r"\b{}\b", pyre::escape(phrase)), 0);
    for m in rx.finditer(clean) {
        let before = text::split_ref(&clean[..m.start()]);
        if com4::tail(&before, window).iter().any(|w| NEARLY.contains(*w)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/__init__.py:168 `doubted(clean, window=4)`
///
/// True when the line only thinks, hopes or doubts what it says afterwards.
pub fn doubted(clean: &str, window: i64) -> bool {
    let plain: Vec<String> = text::split_ref(clean)
        .into_iter()
        .map(|w| text::strip_chars(w, ".,!?;:\"'").replace('\u{2019}', ""))
        .collect();
    if plain.iter().any(|w| DOUBTED.contains(w)) {
        return true;
    }
    for (i, word) in plain.iter().enumerate() {
        let before = com4::before(&plain, i, window);
        if !SURE.contains(word) || !turns_round(before) {
            continue;
        }
        if DOUBT.contains(word) && before.iter().any(|w| CANNOT.contains(w)) {
            // "i cant believe i passed" - a cant in front of the verb is an intensifier, the
            // same reading `turns_round` makes for a cue it can see.
            continue;
        }
        if BELIEVE_IT.contains(word) && i + 1 < plain.len() && plain[i + 1] == "it" {
            // "i dont believe it, i passed" is delight at the news, not a doubt about it.
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/compose/__init__.py:202 `said_again(text)`
///
/// The sentence with whoever it is about blanked, so two sayings of it compare equal.
pub fn said_again(text: &str) -> String {
    _PERSON_WORD.sub("@", &com4::flat(text))
}

/// Python: bot/compose/__init__.py:236 `_opens_with_a_name(body)`
///
/// True when the reply opens with a name or a place rather than a word that starts a sentence.
pub fn _opens_with_a_name(body: &str) -> bool {
    let Some(m) = _ONE_WORD_THEN_A_BREAK.match_(text::strip(body)) else { return false };
    let word = m.str(1).to_lowercase();
    !_OPENS_A_SENTENCE.contains(text::strip_chars(&word, "'\u{2019}"))
}

/// Python: bot/compose/__init__.py:250 `greet_with(name, reply)`
///
/// The answer to a line that introduced someone, with the hello in front of it. A reply that
/// already opens with a hello is left alone, and so is an empty one (Python hands back a reply
/// that is not a str as it came; the callers here only ever hold a str).
pub fn greet_with(name: &str, reply: &str) -> String {
    if name.is_empty() || text::strip(reply).is_empty() {
        return reply.to_string();
    }
    let body = text::strip(reply);
    let plain = body.to_lowercase();
    if text::starts_with_any(&plain, &_HELLO_OPENS) || plain.starts_with(&format!("i'm {}", name.to_lowercase())) {
        return body.to_string();
    }
    let name = text::strip(name);
    // The person's own voice keeps its capital ("I'm not sure about that one, sorry"); every
    // other sentence is part of the greeting and starts lower down.
    if _CAPITAL_I.match_(body).is_some() {
        return format!("Hi {name} \u{2014} {body}");
    }
    if _opens_with_a_name(body) {
        // "Bristol — nice" is a place, and a place does not lose its capital.
        return format!("Hi {name}. {body}");
    }
    let mut chars = body.chars();
    let first = chars.next().map(|c| c.to_lowercase().collect::<String>()).unwrap_or_default();
    format!("Hi {name} \u{2014} {first}{}", chars.as_str())
}

/// Python: bot/compose/__init__.py:289 `_asks_about_a_thing_nobody_said(option, last, said)`
///
/// True when a pool line opens on a fragment question about a thing neither side has named.
pub fn _asks_about_a_thing_nobody_said(option: &str, last: Option<&str>, said: &str) -> bool {
    let last = last.unwrap_or("");
    if !last.is_empty() && text::strip(last).ends_with('?') {
        return false; // the bot has just asked something, so a question back is fair
    }
    let Some(m) = OPENS_ON_A_FRAGMENT_QUESTION.match_(option) else { return false };
    let fragment = m.str(1).to_lowercase();
    let words: HashSet<String> = A_WORD.findall_str(&fragment).into_iter().filter(|w| text::len(w) > 2).collect();
    if words.is_empty() || words.iter().any(|w| _NOT_A_FRAGMENT.contains(w)) {
        return false;
    }
    let heard: HashSet<String> = A_WORD.findall_str(&format!("{last} {said}").to_lowercase()).into_iter().collect();
    !words.iter().any(|w| heard.contains(w))
}

/// Python: bot/compose/__init__.py:310 `pick(options, memory, seed_text='')`
///
/// One of options, never the reply just given, preferring one the bot has not said lately.
/// `seed_text` is the user's line when the caller has it: two different lines answered from the
/// same pool should not get the same sentence just because they are turn one of their
/// conversation. Same line, same turn, same reply, so a run can be compared.
pub fn pick<S: AsRef<str>>(options: &[S], memory: &Memory, seed_text: &str) -> String {
    let turns = &memory.turns;
    let last: Option<&str> = turns.last().map(|t| t.bot.as_str());
    let mut options: Vec<&str> = options.iter().map(|o| o.as_ref()).collect();
    // A line that opens by asking about something nobody has said yet is not a line that can
    // open a conversation, so it is passed over while there is another one in the pool.
    let standing: Vec<&str> =
        options.iter().copied().filter(|o| !_asks_about_a_thing_nobody_said(o, last, seed_text)).collect();
    if !standing.is_empty() && standing.len() < options.len() {
        options = standing;
    }
    // A pool's returning lines ("Hello again. What's next?") speak about a chat that has
    // happened. On the opening turn there is none, so they are passed over for one that can open
    // a conversation.
    if !crate::memory::history::an_earlier_turn(memory, "") {
        let plain = com4::memory_value(memory);
        let backed: Vec<&str> = options
            .iter()
            .copied()
            .filter(|o| !crate::memory::history::a_past_that_is_not_there(o, &plain, None))
            .collect();
        if !backed.is_empty() {
            options = backed;
        }
    }
    // Drop the reply just given first, so an adjacent repeat cannot happen, then prefer an
    // unseen one.
    let mut fresh: Vec<&str> = options.iter().copied().filter(|o| Some(*o) != last).collect();
    if fresh.is_empty() {
        fresh = options.clone();
    }
    // A reply this conversation has already had is not said again while there is another one
    // to say, however long ago it was. Where every option has been said, the one said longest
    // ago is the least like a repeat of what is on the screen.
    let mut said: HashMap<String, i64> = HashMap::new();
    for (position, turn) in turns.iter().enumerate() {
        said.entry(said_again(&turn.bot)).or_insert(position as i64);
    }
    let said_at = |o: &str| -> i64 { said.get(&said_again(o)).copied().unwrap_or(-1) };
    let unsaid: Vec<&str> = fresh.iter().copied().filter(|o| !said.contains_key(&said_again(o))).collect();
    if !unsaid.is_empty() {
        fresh = unsaid;
    } else {
        // `min()` of an empty pool raises in Python too.
        let oldest = fresh.iter().map(|o| said_at(o)).min().expect("pick: an empty pool");
        fresh.retain(|o| said_at(o) == oldest);
    }
    let seed: i128 = if !seed_text.is_empty() {
        turns.len() as i128 * 7919 + com4::ord_sum(seed_text)
    } else {
        turns.len() as i128 // a caller without the line gets the same choice as before
    };
    Random::new(seed).choice(&fresh).to_string()
}

/// Python: bot/compose/__init__.py:359 `_name_just_given(memory)`
///
/// The name the person gave in this very turn, plain, or "". This reads the store, not the
/// words, so a name mentioned in passing ("my brother jess") is not a person introducing
/// themselves.
pub fn _name_just_given(memory: &Memory) -> String {
    if memory.get("aside").is_some_and(Value::truthy) {
        // The line was put to the entries on its own account, with the chat left behind.
        // Nobody is introducing themselves in those, and the name in the store was given turns
        // ago.
        return String::new();
    }
    let Some(Value::List(log)) = memory.get("facts_log") else { return String::new() };
    let now = Value::Int(memory.turns.len() as i64);
    for entry in log.iter().rev() {
        if !entry.is_dict() || entry.get("turn").unwrap_or(&Value::None) != &now {
            continue;
        }
        if entry.get("key").and_then(Value::as_str) != Some("name") {
            continue;
        }
        if let Some(value) = entry.get("value").and_then(Value::as_str) {
            if !text::strip(value).is_empty() {
                return text::strip(value).to_string();
            }
        }
    }
    String::new()
}

/// Python: bot/compose/__init__.py:392 `with_name(reply, memory)`
///
/// The answer the person is owed, with the name they just gave in front of it. Nothing is added
/// for a reply that already says the name, or on a turn where no name was given.
pub fn with_name(reply: Option<&str>, memory: &Memory) -> Option<String> {
    let reply = reply?;
    if text::strip(reply).is_empty() {
        return Some(reply.to_string());
    }
    let name = _name_just_given(memory);
    if name.is_empty() {
        return Some(reply.to_string());
    }
    let plain = text::squash(&name);
    if !text::is_alpha(&plain) || text::len(&plain) > 20 {
        return Some(reply.to_string());
    }
    // "nice to meet you, jess." already greets them, so the word is matched on its boundaries
    // rather than with spaces around it: a comma or a full stop after the name still counts.
    let rx = Re::new(&format!(r"\b{}'?s?\b", pyre::escape(&plain.to_lowercase())), 0);
    if rx.search(&reply.to_lowercase()).is_some() {
        return Some(reply.to_string());
    }
    Some(greet_with(&text::capitalize(&plain), reply))
}

/// Python: bot/compose/__init__.py:467 `_says_one_of(text, said)`
///
/// True when the bot's last line says one of `said`, dressed up or not: the pool line is looked
/// for inside the reply, case-folded, because a greeting in front of it drops the capital.
pub fn _says_one_of<S: AsRef<str>>(text: &str, said: &[S]) -> bool {
    let flat = text.to_lowercase();
    said.iter().any(|line| !line.as_ref().is_empty() && flat.contains(&line.as_ref().to_lowercase()))
}

/// Python: bot/compose/__init__.py:483 `step_when_low(clean, memory, comfort_said)`
///
/// A next step for a bare "what should i do" after one of the bot's own comfort lines.
/// `comfort_said` is the set of lines the calling composer can say as comfort. A step asked for
/// anywhere else - after a recipe, a verdict, a safety line - is that thread's and this returns
/// None.
pub fn step_when_low<S: AsRef<str>>(clean: &str, memory: &Memory, comfort_said: &[S]) -> Option<String> {
    if A_STEP_ASK.match_(text::strip(clean)).is_none() {
        return None;
    }
    let last = text::strip(com4::last_bot(memory));
    if last.is_empty() || !_says_one_of(last, comfort_said) {
        return None;
    }
    // The comfort line over news is not an offer about how they are feeling, so the first step
    // of a bad day is not the answer to the ask that follows it. Stand down here, and the ask
    // reaches whoever read the news. The one news that keeps the step is somebody leaving.
    // (Python wraps this reading in `try/except Exception`: a reading that fails is a reading
    // not made.)
    let said = com4::flat(com4::last_user(memory));
    let stands_down = com4::attempt(|| {
        // A leaving reported in the third person has no counsel of its own: stand down here and
        // the ask reaches the ask reader.
        if badnews::reads_as_a_reported_leaving(&said) {
            return true;
        }
        badnews::reads_as_bad_news(&said)
            && !badnews::reads_as_the_leaving(&said)
            && !badnews::reads_as_a_leaving_already_gone(&said)
            && !badnews::reads_as_a_sacking(&said)
    });
    if stands_down == Some(true) {
        return None;
    }
    Some(pick(&STEP_WHEN_LOW, memory, clean))
}
