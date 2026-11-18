//! Port of `bot/knowledge/vegetables.py`: how long a vegetable takes, and what "done" looks like
//! when it gets there; roasting times; and the fixes for a vegetable gone soggy, wet or limp.
//!
//! The tables and the entries are data from the bundle. The entry factory (`_reply_for`) and
//! the `ENTRIES` builders are ported too ([`entries`]), and give the exported entries.

use crate::data::{self, FnRef, Value};
use crate::knowledge::built;
use crate::knowledge::entry::{Entry, Reply};
use crate::knowledge::lookup::tokens;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

const P: &str = "bot.knowledge.vegetables.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}

macro_rules! strs {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($n)).strs());)* };
}
strs!(
    VEG_ASKS, VEG_HOW_ASKS, VEG_NAMED_METHOD, VEG_OTHER, VEG_TIME_WORDS, VEG_NOT_FOR_US, VEG_ASK_BETWEEN, VEG_FILLER,
    VEG_COOK, RULE_OUT, ROAST_WORDS, ROAST_NOT_FOR_US, VEG_SOFT_NAMES, VEG_ROASTED, VEG_WET,
    VEG_LEAVES, VEG_PODS, VEG_RAW_FAULTS, VEG_PREVENTION, VEG_GENERIC, VEG_COOKED, VEG_BRASSICAS,
);
static _RULE_OUT_REACH: LazyLock<usize> = LazyLock::new(|| g("_RULE_OUT_REACH").as_int().unwrap() as usize);
/// `VEG_TIME`: (names, what to say).
static VEG_TIME: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| name_rows("VEG_TIME"));
/// `VEG_HOW_EXTRA`: (names, how to cook them).
static VEG_HOW_EXTRA: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| name_rows("VEG_HOW_EXTRA"));
static WATER_FAMILIES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| name_rows("WATER_FAMILIES"));
static VEG_HOW: LazyLock<IndexMap<String, String>> =
    LazyLock::new(|| g("VEG_HOW").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect());
/// `ROAST_MINUTES`: {name: (minutes, how to cut it)}, in the dict's order.
static ROAST_MINUTES: LazyLock<IndexMap<String, (String, String)>> = LazyLock::new(|| {
    g("ROAST_MINUTES").pairs().iter().map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].str().to_string()))).collect()
});
static VEG_EXTRA_AVOID: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| g("VEG_EXTRA_AVOID").str_list_map());

fn name_rows(name: &str) -> Vec<(Vec<String>, String)> {
    g(name).items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
}

/// `re.search(r"(?<![a-z])" + re.escape(phrase) + r"(?![a-z])", ...)`, compiled once per phrase.
fn phrase_re(phrase: &str) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<String, &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(r) = cache.get(phrase) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(&format!("(?<![a-z]){}(?![a-z])", pyre::escape(phrase)), 0)));
    cache.insert(phrase.to_string(), r);
    r
}

/// True when the phrase is in the line, on word boundaries ("soft" is not "soften").
pub fn _phrase_in(text_: &str, phrase: &str) -> bool {
    phrase_re(phrase).is_match(text_)
}

/// `str(text).lower().replace("'", " ").split()`.
fn words_of(text_: &str) -> Vec<String> {
    text::split(&text::lower(text_).replace('\'', " "))
}

/// True for "carrots take to roast" and "carrots take in the oven", and nothing else.
pub fn _takes_to(words: &[String], position: usize, spot: usize) -> bool {
    let between: &[String] = if position + 1 <= spot && spot <= words.len() { &words[position + 1..spot] } else { &[] };
    if between.is_empty() || !between.iter().all(|w| VEG_FILLER.contains(w)) {
        return false;
    }
    if between.iter().any(|w| VEG_COOK.contains(w)) {
        return false;
    }
    between.iter().any(|w| matches!(w.as_str(), "take" | "takes" | "took"))
}

/// Where the phrase sits among the words of the line, or None (the plural counts as the word).
pub fn _near(text_: &str, phrase: &str) -> Option<usize> {
    let wanted = text::lower(phrase);
    words_of(text_).iter().position(|word| *word == wanted || *word == format!("{wanted}s") || format!("{word}s") == wanted)
}

/// True when the whole word (or its plain plural) is in the line, matched exactly.
pub fn _a_word(text_: &str, word: &str) -> bool {
    tokens(text_).iter().any(|one| {
        *one == word || *one == format!("{word}s") || format!("{one}s") == word || (one.ends_with("es") && one[..one.len() - 2] == *word)
    })
}

/// True when `name` is in the line as whole words, singular or plural (the last word takes
/// the plural, the rest has to match exactly).
pub fn _name_in(text_: &str, name: &str) -> bool {
    let wanted = tokens(name);
    let have = tokens(text_);
    if wanted.is_empty() || have.is_empty() {
        return false;
    }
    let last = wanted.len() - 1;
    if have.len() < wanted.len() {
        return false;
    }
    (0..=have.len() - wanted.len()).any(|i| {
        let run = &have[i..i + wanted.len()];
        run[..last] == wanted[..last] && _a_word(&run[last..].join(" "), &wanted[last])
    })
}

/// True when the line names one of these foods only to rule it out: a rule-out word close in
/// front of it.
pub fn _ruled_out(text_: &str, names: &[String]) -> bool {
    let words = words_of(text_);
    if words.is_empty() {
        return false;
    }
    for name in names {
        let wanted = words_of(name);
        if wanted.is_empty() || words.len() < wanted.len() {
            continue;
        }
        for start in 0..=words.len() - wanted.len() {
            let run = &words[start..start + wanted.len()];
            let n = wanted.len();
            if run[..n - 1] != wanted[..n - 1] && run != wanted.as_slice() {
                continue;
            }
            if !_a_word(&run[n - 1..].join(" "), &wanted[n - 1]) {
                continue;
            }
            let before = &words[start.saturating_sub(*_RULE_OUT_REACH)..start];
            if before.iter().any(|w| RULE_OUT.contains(w)) {
                return true;
            }
        }
    }
    false
}

/// The line as the person typed it (`memory["understood"]["text"]`), else the reading's text.
pub fn _typed_line(memory: &Memory, understood: &Understood) -> String {
    if let Some(now) = &memory.understood {
        if !now.text.is_empty() {
            return now.text.clone();
        }
    }
    understood.text.clone()
}

/// The entry reply for one vegetable: a closure over its names, its time (`said`) and how to
/// cook it (`how`), run by [`_reply_for_answer`].
pub fn _reply_for(names: &[String], said: Option<&str>, how: Option<&str>) -> FnRef {
    built::closure(
        "bot.knowledge.vegetables:_reply_for.<locals>.answer:413",
        vec![("how", how.into()), ("names", built::words(names)), ("said", said.into())],
    )
}

const IN_THE_OVEN: [&str; 5] = ["in the oven", "in an oven", "in my oven", "in the air fryer", "in a hot oven"];

/// `_reply_for(names, said, how).<locals>.answer`: the vegetable's own time, or how to cook it,
/// or None when the line asks something else.
pub fn _reply_for_answer(names: &[String], said: Option<&str>, how: Option<&str>, memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = _typed_line(memory, understood);
    if text_.is_empty() {
        return None;
    }
    let t = text_.as_str();
    if !VEG_ASKS.iter().any(|w| _a_word(t, w)) {
        return None; // "i bought a cauliflower" is not a timing question
    }
    if _ruled_out(t, names) {
        return None;
    }
    let asks_a_time = VEG_TIME_WORDS.iter().any(|w| _phrase_in(t, w));
    let how_asked = how.is_some() && VEG_HOW_ASKS.iter().any(|w| _a_word(t, w)) && !asks_a_time && !VEG_NAMED_METHOD.iter().any(|w| _a_word(t, w));
    if !how_asked && !asks_a_time {
        return None; // "how do i cook cauliflower" is the method's question, not this one
    }
    if ["freezer", "freeze", "freezing"].iter().any(|w| _a_word(t, w)) && !["boil", "boiling", "steam", "steaming", "cook", "cooking", "blanch"].iter().any(|w| _a_word(t, w)) {
        return None;
    }
    let mut sorted: Vec<&String> = names.iter().collect();
    sorted.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    let where_ = sorted.into_iter().find(|name| _name_in(t, name))?;
    if IN_THE_OVEN.iter().any(|p| _phrase_in(t, p)) {
        return None;
    }
    let position = _near(t, text::split_ref(where_).last().copied().unwrap_or(""));
    let words = words_of(t);
    for word in VEG_OTHER.iter() {
        if !_phrase_in(t, word) {
            continue;
        }
        if text::split_ref(word).contains(&where_.as_str()) {
            return None;
        }
        let spot = _near(t, word);
        let (Some(position), Some(spot)) = (position, spot) else { continue };
        if spot < position {
            let between = &words[spot + 1..position];
            if !between.iter().any(|w| VEG_ASK_BETWEEN.contains(w)) {
                return None; // "how long do i roast carrots": the method is the question
            }
        } else if spot > position && _takes_to(&words, position, spot) {
            return None;
        } else if spot as i64 - position as i64 <= 2 {
            return None; // "carrots gone soft", "cabbage in the fridge"
        }
    }
    let head = text::upper_first(where_);
    let how_said = how.unwrap_or("None");
    if how_asked {
        if _a_word(t, "frozen") {
            return Some(format!("{head}, from frozen: straight into already boiling water and take 2 minutes off - {how_said}"));
        }
        return Some(format!("{head}: {how_said}"));
    }
    let Some(said) = said.filter(|s| !s.is_empty()) else {
        // `head + ": " + how` with no time and no method raises TypeError
        let how = how.unwrap_or_else(|| panic!("TypeError: can only concatenate str (not \"NoneType\") to str"));
        return Some(format!("{head}: {how}"));
    };
    if _a_word(t, "frozen") {
        return Some(format!("{head}, from frozen: go straight into already boiling water and take 2 minutes off - {said}"));
    }
    Some(format!("{head}: {said}"))
}

/// How long one vegetable takes in the oven, or None when the line asks something else.
pub fn _roast_reply(memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = _typed_line(memory, understood);
    if text_.is_empty() {
        return None;
    }
    let t = text_.as_str();
    if !ROAST_WORDS.iter().any(|w| _phrase_in(t, w)) {
        return None;
    }
    if ROAST_NOT_FOR_US.iter().any(|w| _a_word(t, w)) {
        return None;
    }
    let names: Vec<String> = ROAST_MINUTES.keys().cloned().collect();
    if _ruled_out(t, &names) {
        return None; // the vegetables the line turned down are not what to roast
    }
    let mut sorted: Vec<&String> = ROAST_MINUTES.keys().collect();
    sorted.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    let where_ = sorted.into_iter().find(|name| _name_in(t, name))?;
    let (minutes, how) = ROAST_MINUTES.get(where_).cloned().unwrap_or_else(|| ("25 to 30".into(), "cut into even pieces".into()));
    Some(format!(
        "{}: {minutes} minutes in the oven at 200C (180C fan), {how}, in a single layer with a little oil so it browns rather than steams. It is done when the edges are browned and a knife goes into the thickest piece easily.",
        text::upper_first(where_)
    ))
}

static NOT_A_SLUG: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9]+", 0));

/// An entry id for a vegetable: "butternut squash" is `veg-butternut-squash`.
pub fn _slug(name: &str) -> String {
    format!("veg-{}", text::strip_chars(&NOT_A_SLUG.sub("-", &text::lower(name)), "-"))
}

/// The fix for a vegetable that has gone wet, per family; None when the line names none.
pub fn _watery_veg(understood: &Understood) -> Option<String> {
    let joined = understood.words.join(" ");
    let said = text::lower(if joined.is_empty() { &understood.clean } else { &joined });
    let mut best: Option<(usize, &String)> = None;
    for (names, fix) in WATER_FAMILIES.iter() {
        for name in names {
            if _phrase_in(&said, name) && best.is_none_or(|b| text::len(name) > b.0) {
                best = Some((text::len(name), fix));
            }
        }
    }
    best.map(|b| b.1.clone())
}

/// The fix for a vegetable that came out soft, wet or limp, or None. `understood` None is
/// Python's default.
pub fn _soft_veg(text_: &str, understood: Option<&Understood>) -> Option<String> {
    if text_.is_empty() {
        return None;
    }
    let mut t = text_.to_string();
    if !VEG_SOFT_NAMES.iter().any(|name| _name_in(&t, name)) {
        if let Some(understood) = understood {
            let said = text::lower(&understood.text);
            if !said.is_empty() && said != t {
                t = said;
            }
        }
    }
    let t = t.as_str();
    let phrase = |list: &[String]| list.iter().any(|w| _phrase_in(t, w));
    let named = |list: &[String]| list.iter().any(|w| _name_in(t, w));
    if ["slimy", "smelly", "sour", "gone off"].iter().any(|w| _phrase_in(t, w)) {
        return Some("A raw vegetable that is slimy and smells is going off, so bin the affected leaves or the whole bag: that slime is bacteria, not water, and cooking will not make it safe to eat. Slime on a cooked vegetable is the harmless kind - it means too long in the water - so drain it, press the water out and dress it with oil and lemon.".into());
    }
    if phrase(&VEG_RAW_FAULTS) && !phrase(&VEG_COOKED) {
        return Some("A raw vegetable that has gone limp has lost water, not flavour: it is still safe to eat, and half an hour in a bowl of iced water firms it up again for a salad. After that, cook it - roast it at 200C, or put it in a soup or a stir-fry - because a limp one will not crisp back on its own. For the next lot, a bag or a lidded tub in the salad drawer keeps it firm for weeks, and cutting the green tops off carrots stops them pulling the water out of the root.".into());
    }
    if phrase(&VEG_PREVENTION) && !named(&VEG_GENERIC) {
        return None;
    }
    if named(&VEG_PODS) {
        return Some("Peas and beans go from squeaky to mushy in about a minute: 2 minutes for frozen peas straight into boiling water, 4 to 5 for green beans, and drain them the moment they are bright. If they are already soft, toss them in butter with a squeeze of lemon and serve them as they are - a soft pea will not firm up - or let them cool into a salad or a soup.".into());
    }
    if named(&VEG_LEAVES) {
        return Some("Spinach, kale and other leaves go slimy in a pan of water, so cook them in almost none: a hot pan with a knob of butter and a spoon of water, 1 to 2 minutes until they have just collapsed, then press the water out. Raw leaves that have only gone limp come back in 15 minutes in iced water; cooked ones that are slimy were in too much water, so squeeze them dry and dress them with butter or a little vinegar, or fold them into a sauce.".into());
    }
    if named(&VEG_BRASSICAS) {
        return Some("Broccoli, cauliflower and cabbage go soft and grey from too long at a rolling boil: 4 to 5 minutes for broccoli florets, 6 to 8 for sprouts or shredded cabbage, 12 to 15 for a whole cauliflower, then straight into a colander with the lid off, because it goes on cooking in its own steam. If it has already gone grey and sulphury, give it a minute in a hot pan with butter and garlic, or put it in a gratin or a soup.".into());
    }
    if named(&VEG_WET) {
        return Some("Courgette, aubergine and tomato are mostly water, so that water has to come out before the heat goes on: slice them, salt them for 10 minutes and pat them dry, then fry or roast in one layer in a hot pan and do not crowd it, because a crowded pan steams them instead of browning. Already limp: turn the heat up to drive the water off, or put them in a sauce or a soup.".into());
    }
    if phrase(&VEG_ROASTED) {
        return Some("A vegetable that comes out of the oven soft was steamed rather than roasted: it went in wet, or the tray was crowded. Dry it, toss it in oil, spread it in one layer with air between the pieces, and give it 200C (180C fan) with the door shut. Already limp: turn the heat up for the last 10 minutes to colour what you can, and serve it with something sharp - it will not crisp back.".into());
    }
    Some("A vegetable goes limp and wet from too long in the water, from a lid left on so the steam cannot escape, or from a pan too small for the amount: cook it in a shallow layer with a splash of water rather than a deep panful, taste it a minute before the time in the recipe, and drain it the second a knife slides in, lid off. If it is already soft, drive the water off in a hot dry pan with a little butter, or put it in a soup or a sauce, because it will not crisp again. A raw vegetable gone limp in the fridge is the same rescue: it has lost water rather than been overcooked, so roast it at 200C or put it in a soup within the day.".into())
}

/// The vegetable rows `ENTRIES` builds from the tables, in order: one per vegetable, then the
/// roast row. The rows written out whole in the module (the crisping, watery and soft rows) are
/// the bundle's.
pub fn entries() -> Vec<Entry> {
    let words = |list: &[String]| built::words(list);
    let mut out = Vec::new();
    for (names, said) in VEG_TIME.iter() {
        let how = VEG_HOW.get(&names[0]).map(String::as_str);
        let extra = VEG_EXTRA_AVOID.get(&names[0]).cloned().unwrap_or_default();
        out.push(built::entry(
            &_slug(&names[0]),
            vec![built::part(names), built::part(&VEG_ASKS)],
            Reply::Fn(_reply_for(names, Some(said), how)),
            &[words(&VEG_NOT_FOR_US), words(&extra)],
            2,
            vec![],
            None,
        ));
    }
    for (names, how) in VEG_HOW_EXTRA.iter() {
        out.push(built::entry(
            &_slug(&names[0]),
            vec![built::part(names), built::part(&VEG_ASKS)],
            Reply::Fn(_reply_for(names, None, Some(how))),
            &[words(&VEG_NOT_FOR_US)],
            2,
            vec![],
            None,
        ));
    }
    let roast: Vec<String> = ROAST_MINUTES.keys().cloned().collect();
    let asked: Vec<String> = [
        "in the oven", "oven", "roast", "roasted", "roasting", "bake", "baked", "baking", "air fryer", "how long", "long", "minutes",
        "time", "roasting time", "roast time", "time in the oven", "oven time", "how long in the oven",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    out.push(built::entry(
        "veg-roast",
        vec![built::part(&roast), built::part(&asked)],
        Reply::Fn(built::function("bot.knowledge.vegetables:_roast_reply:582")),
        &[words(&VEG_NOT_FOR_US)],
        1,
        vec![],
        None,
    ));
    out
}

fn reply_for(f: &FnRef, memory: &mut Memory, understood: &Understood) -> Option<String> {
    let names = f.cell("names").map(Value::strs).unwrap_or_default();
    let said = f.cell("said").and_then(Value::as_str);
    let how = f.cell("how").and_then(Value::as_str);
    _reply_for_answer(&names, said, how, memory, understood)
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.vegetables:_reply_for.<locals>.answer:413", reply_for),
    ("bot.knowledge.vegetables:_roast_reply:582", |_, m, u| _roast_reply(m, u)),
    ("bot.knowledge.vegetables:<lambda>:775", |_, _, u| _watery_veg(u)),
    ("bot.knowledge.vegetables:<lambda>:949", |_, m, u| _soft_veg(&_typed_line(m, u), Some(u))),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The rows `entries()` builds are the rows Python's import built.
    #[test]
    fn the_built_rows_are_the_exported_ones() {
        data::load();
        let exported = built::exported("bot.knowledge.vegetables.ENTRIES");
        let ours = entries();
        for (mine, theirs) in ours.iter().zip(exported.iter()) {
            assert_eq!(built::same_entry(mine, theirs), None);
        }
        assert_eq!(exported[ours.len() - 1].id, "veg-roast");
    }

    #[test]
    fn words_and_names() {
        data::load();
        assert_eq!(_slug("Butternut Squash"), "veg-butternut-squash");
        assert!(_name_in("how long do runner beans take", "runner bean"));
        assert!(_ruled_out("i dont like carrots how long for leeks", &["carrot".into()]));
        assert!(!_ruled_out("carrots with no butter", &["carrot".into()]));
        assert_eq!(_near("how long do i roast carrots", "carrot"), Some(5));
    }
}
