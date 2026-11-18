//! Port of `bot/modules/their_taste.py`: a taste of their own, said plainly as a whole line ("i
//! love thai food", "i cant stand mushrooms"), gets a reply that answers back - what the bot
//! thinks of the thing and a question only the person can answer - instead of the person's own
//! sentence handed back. A fallback band (-960): it stands down wherever the composer or the
//! greeting band below it has a line of its own, and after heavy news.

use crate::data::{self, Value};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::runtime::guarded;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

fn name(n: &str) -> String {
    format!("bot.modules.their_taste.{n}")
}

static LIKES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("_LIKES")));
static DISLIKES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("_DISLIKES")));
static LIKE_VERBS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("_LIKE_VERBS")));
static DISLIKE_VERBS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("_DISLIKE_VERBS")));
static FAN_OF: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_FAN_OF")));
static TONE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_TONE")));
static LEAD: LazyLock<String> = LazyLock::new(|| data::string(&name("_LEAD")));
static THING: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_THING")));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&name("_NOT_A_THING")));
static A_PERSON: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_A_PERSON")));
static THEIRS_BUT_NOT_A_THING: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_THEIRS_BUT_NOT_A_THING")));
static NOT_A_TASTE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_NOT_A_TASTE")));
static I_SAY: LazyLock<String> = LazyLock::new(|| data::string(&name("_I_SAY")));
static TYPED: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get(&name("_TYPED"))
        .items()
        .iter()
        .map(|pair| (pair.items()[0].str().to_string(), pair.items()[1].str().to_string()))
        .collect()
});
static STORY: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_STORY")));
static HEAVY: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_HEAVY")));
static PLACEHOLDER_OPENINGS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("_PLACEHOLDER_OPENINGS")));

/// `re.sub(r"^" + _LEAD, "", said)`.
static LEADING: LazyLock<Re> = LazyLock::new(|| Re::new(&format!("^{}", *LEAD), pyre::U));
/// `re.match(r"^(?:your|this|that|these|those)\s+", rest)`.
static NOT_THEIRS: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:your|this|that|these|those)\s+", pyre::U));

/// The heads `_a_taste` tries, in its order: (kind, `re.compile(_I_SAY + verb + r"\s+", re.I)`),
/// every verb escaped except `_FAN_OF`'s own pattern, which closes each kind's list.
static HEADS: LazyLock<Vec<(&'static str, Re)>> = LazyLock::new(|| {
    let mut heads = Vec::new();
    for (kind, verbs) in [("like", &*LIKE_VERBS), ("dislike", &*DISLIKE_VERBS)] {
        for verb in verbs {
            heads.push((kind, Re::new(&format!(r"{}{}\s+", *I_SAY, pyre::escape(verb)), pyre::I | pyre::U)));
        }
        heads.push((kind, Re::new(&format!(r"{}{}\s+", *I_SAY, FAN_OF.pattern()), pyre::I | pyre::U)));
    }
    heads
});

/// `" ".join(str(reply or "").lower().split())` opens on one of the placeholder openings.
fn a_placeholder(reply: &str) -> bool {
    let said = text::squash(&text::lower(reply));
    PLACEHOLDER_OPENINGS.iter().any(|opening| said.starts_with(opening.as_str()))
}

/// Python: bot/modules/their_taste.py:161 `_the_composer_would_answer(text, memory)`: the
/// composer has a line of its own for this line, rather than a placeholder.
pub fn _the_composer_would_answer(text: &str, memory: &mut Memory) -> bool {
    // `fallback.reply_for` is `bot.compose.fallback_chain.reply_for(text, memory, verdict=True)`.
    let Some(reply) = guarded(|| crate::compose::fallback_chain::reply_for(text, memory, true)) else {
        return false; // a composer that cannot be asked claims nothing
    };
    if text::squash(&reply).is_empty() {
        return false;
    }
    !a_placeholder(&reply)
}

/// Python: bot/modules/their_taste.py:175 `_the_greeting_would_answer(text, memory)`: the band
/// below this module has a line of its own, rather than a placeholder.
pub fn _the_greeting_would_answer(text: &str, memory: &mut Memory) -> bool {
    // A reading that fails leaves the line to this module.
    let Some(reply) = guarded(|| crate::modules::greeting::respond(text, memory)) else {
        return false;
    };
    let reply = reply.unwrap_or_default();
    if text::squash(&reply).is_empty() {
        return false;
    }
    !a_placeholder(&reply)
}

/// Python: bot/modules/their_taste.py:204 `_the_line_before(memory)`: what the person said on
/// the turn before this one, or "".
pub fn _the_line_before(memory: &Memory) -> String {
    memory.turns.last().map(|turn| turn.user.clone()).unwrap_or_default()
}

/// Python: bot/modules/their_taste.py:210 `_the_thread_is_heavy(memory)`: the turn before this
/// one was heavy news (the composer's reading, or the plainer words in `_HEAVY`).
pub fn _the_thread_is_heavy(memory: &Memory) -> bool {
    let before = _the_line_before(memory);
    if before.is_empty() {
        return false;
    }
    if guarded(|| crate::compose::fallback::_reads_as_heavy_news(&before)).unwrap_or(false) {
        return true;
    }
    HEAVY.is_match(&before)
}

/// Python: bot/modules/their_taste.py:223 `_said_plainly(clean)`: the line without the
/// throat-clearing and the trailing tone words.
pub fn _said_plainly(clean: &str) -> String {
    let mut said = clean.to_string();
    for (typed, proper) in TYPED.iter() {
        if let Some(rest) = said.strip_prefix(typed.as_str()) {
            said = format!("{proper}{rest}");
        }
    }
    let mut said = LEADING.sub("", &said);
    loop {
        let stripped = TONE.sub("", &said);
        let changed = stripped != said;
        said = stripped;
        if !changed {
            break;
        }
    }
    text::strip(&said).to_string()
}

/// Python: bot/modules/their_taste.py:238 `_a_taste(clean)`: (kind, the thing as they said it)
/// for a line that is a taste on its own, or None.
pub fn _a_taste(clean: &str) -> Option<(&'static str, String)> {
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    for (kind, head) in HEADS.iter() {
        let Some(found) = head.match_(clean) else { continue };
        let rest = text::strip(&clean[found.end()..]);
        if rest.is_empty() || STORY.is_match(rest) {
            continue;
        }
        let Some(thing) = THING.match_(rest) else { continue };
        let rest = thing.name_str("thing").to_string();
        if NOT_A_THING.contains(&rest) || A_PERSON.is_match(&rest) {
            continue;
        }
        if *kind == "dislike" && NOT_A_TASTE.is_match(&rest) {
            continue;
        }
        // A thing they own, an infinitive, or a place held by a word that stands for one.
        let first = text::split(&rest).into_iter().next().unwrap_or_default();
        if ["to", "your", "our", "it", "that", "this", "them", "here", "there"].contains(&first.as_str()) {
            continue;
        }
        if first == "my" {
            let held = text::strip(&text::slice(&rest, 3, None)).to_string();
            if held.is_empty() || THEIRS_BUT_NOT_A_THING.is_match(&held) {
                continue;
            }
        }
        // A thing of the bot's own can carry a word in front of it: "i love this recipe".
        if NOT_THEIRS.match_(&rest).is_some() {
            continue;
        }
        return Some((kind, rest));
    }
    None
}

/// Python: bot/modules/their_taste.py:277 `_the_thing_said(thing)`: the thing in the person's own
/// words, with their "my" turned round.
pub fn _the_thing_said(thing: &str) -> String {
    let mut words = text::split(thing);
    if words.first().map(String::as_str) == Some("my") {
        words[0] = "your".to_string();
    }
    words.join(" ")
}

/// Python: bot/modules/their_taste.py:285 `_pick(pool, memory, said)`: one line of the pool,
/// seeded by the person's line and never the one just given (the composer's `pick`).
pub fn _pick(pool: &[String], memory: &Memory, said: &str) -> String {
    let memory = Value::from_plain(&serde_json::Value::Object(memory.to_plain()));
    crate::compose::pick(pool, &memory, said)
}

/// Python: bot/modules/their_taste.py:295 `_the_reply(kind, thing, memory)`
pub fn _the_reply(kind: &str, thing: &str, memory: &Memory) -> String {
    let thing = _the_thing_said(thing);
    let pool: &[String] = if kind == "like" { &LIKES } else { &DISLIKES };
    let line = _pick(pool, memory, &text::squash(&thing));
    let mut chars = thing.chars();
    let capital = match chars.next() {
        Some(head) => format!("{}{}", head.to_uppercase(), chars.as_str()),
        None => String::new(),
    };
    line.replace("{Thing}", &capital).replace("{thing}", &thing)
}

/// Python: bot/modules/their_taste.py:302 `respond(text, memory)`: the line, when it is a taste
/// on its own and the thread is not a heavy one.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let clean = _said_plainly(&text::squash(&text::lower(text)));
    if clean.is_empty() || clean.contains('?') {
        return None;
    }
    let (kind, thing) = _a_taste(&clean)?;
    if _the_thread_is_heavy(memory) {
        return None;
    }
    // The composer's own line for the thing the last answer leaned on is the better answer.
    if guarded(|| obstacle_answer_for(text, memory)).unwrap_or(false) {
        return None;
    }
    if _the_composer_would_answer(text, memory) {
        return None;
    }
    if _the_greeting_would_answer(text, memory) {
        return None;
    }
    Some(_the_reply(kind, &thing, memory))
}

/// `_obstacle.answer_for(text, memory)` read as a truth value.
/// CONTRACT: `memory::obstacle::answer_for` returns `()`, so no answer can come back yet; the
/// call is still made so a raise is a raise and its memory writes happen.
fn obstacle_answer_for(text: &str, memory: &mut Memory) -> bool {
    use crate::modules::arbiter::Reading;
    crate::memory::obstacle::answer_for(text, memory).is_truthy()
}
