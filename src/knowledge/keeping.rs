//! Port of `bot/knowledge/keeping.py`: shelf life of the store cupboard, the salad drawer and the
//! salad, ripening, and food that has turned. Tables and entries are data from the bundle.
//!
//! Build-time only: `_dated_row`'s outer body and `_ripens`'s outer body build entries (their
//! inner lambda / reply are ported below).

use crate::data;
use crate::data::FnRef;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{escape, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.keeping.{name}"))
}
fn string(name: &str) -> String {
    data::string(&format!("bot.knowledge.keeping.{name}"))
}
/// A table of tuples of strings / tuples: each row's items, as (names, texts...).
fn rows(name: &str) -> Vec<(Vec<String>, Vec<String>)> {
    data::get(&format!("bot.knowledge.keeping.{name}"))
        .items()
        .iter()
        .map(|row| {
            let items = row.items();
            (items[0].strs(), items[1..].iter().map(|v| v.str().to_string()).collect())
        })
        .collect()
}

static OATS_BAG: LazyLock<String> = LazyLock::new(|| string("OATS_BAG"));
static OATS_COOKED: LazyLock<String> = LazyLock::new(|| string("OATS_COOKED"));
static _MUSH_VERDICT: LazyLock<String> = LazyLock::new(|| string("_MUSH_VERDICT"));
static _MUSH_FIX: LazyLock<String> = LazyLock::new(|| string("_MUSH_FIX"));
static _EATING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_EATING_WORDS"));
static FREEZE_SWEETS: LazyLock<Vec<(Vec<String>, Vec<String>)>> = LazyLock::new(|| rows("FREEZE_SWEETS"));
static STORECUPBOARD_KEEP: LazyLock<Vec<(Vec<String>, Vec<String>)>> = LazyLock::new(|| rows("STORECUPBOARD_KEEP"));
static CUT_OUT_FOR: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.keeping.CUT_OUT_FOR"));
static TIMES_A_THAW: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.keeping.TIMES_A_THAW"));
static ASKS_HOW_LONG: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.keeping.ASKS_HOW_LONG"));
static SOFT_CHEESES: LazyLock<Vec<String>> = LazyLock::new(|| strs("SOFT_CHEESES"));
static BLUE_CHEESES: LazyLock<Vec<String>> = LazyLock::new(|| strs("BLUE_CHEESES"));
static KEEP_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("KEEP_ASKS"));
static RIPEN_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("RIPEN_ASKS"));
static TELL_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("TELL_ASKS"));
/// `RIPEN_TABLE`: (names, [test, trick, keep]), with the `\b<name>\b` patterns for the chat read.
static RIPEN_TABLE: LazyLock<Vec<(Vec<String>, Vec<String>, Vec<Re>)>> = LazyLock::new(|| {
    rows("RIPEN_TABLE")
        .into_iter()
        .map(|(names, texts)| {
            let rxs = names.iter().map(|n| Re::new(&format!(r"\b{}\b", escape(n)), 0)).collect();
            (names, texts, rxs)
        })
        .collect()
});
static _CRUNCH_VERDICT: LazyLock<String> = LazyLock::new(|| string("_CRUNCH_VERDICT"));
static _CRUNCH_FIX: LazyLock<String> = LazyLock::new(|| string("_CRUNCH_FIX"));
static _YOGHURT_WHEY: LazyLock<String> = LazyLock::new(|| string("_YOGHURT_WHEY"));
static _YOGHURT_FIX: LazyLock<String> = LazyLock::new(|| string("_YOGHURT_FIX"));
static _FLOUR_LUMPS: LazyLock<String> = LazyLock::new(|| string("_FLOUR_LUMPS"));
static _FLOUR_MITES: LazyLock<String> = LazyLock::new(|| string("_FLOUR_MITES"));
static _CRUNCH_SAFE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_CRUNCH_SAFE"));
static OFF_BERRIES: LazyLock<Vec<String>> = LazyLock::new(|| strs("OFF_BERRIES"));
static OFF_LEAVES: LazyLock<Vec<String>> = LazyLock::new(|| strs("OFF_LEAVES"));
static OFF_SOFT: LazyLock<Vec<String>> = LazyLock::new(|| strs("OFF_SOFT"));
static OFF_CITRUS: LazyLock<Vec<String>> = LazyLock::new(|| strs("OFF_CITRUS"));
static OFF_FIRM: LazyLock<Vec<String>> = LazyLock::new(|| strs("OFF_FIRM"));
static OFF_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| strs("OFF_NAMES"));
static _OFF_GROUP_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_OFF_GROUP_WORDS"));

fn any_in<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    words.iter().any(|w| text.contains(w.as_ref()))
}

/// `" ".join(str(understood.get(k) or "") for k in keys).lower()` over clean, norm_text and text.
fn clean_norm_text(understood: &Understood) -> String {
    format!("{} {} {}", understood.clean, understood.norm_text, understood.text).to_lowercase()
}

/// The bowl for "how long does porridge keep", the bag for "how long do oats last".
pub fn _oats_kept(_memory: &mut Memory, understood: &Understood) -> String {
    let norm = if understood.norm.is_empty() { String::new() } else { crate::knowledge::d2_shared::py_repr_list(&understood.norm) };
    let said = format!("{} {} {}", understood.clean, understood.norm_text, norm).to_lowercase();
    if said.contains("cooked") || said.contains("made") || (said.contains("porridge") && !said.contains("oats")) {
        return OATS_COOKED.clone();
    }
    OATS_BAG.clone()
}

/// Slimy mushrooms: the verdict first when the line asks whether to eat them, else the fix.
pub fn _slimy_mushrooms(_memory: &mut Memory, understood: &Understood) -> String {
    let said = clean_norm_text(understood);
    if any_in(&said, &_EATING_WORDS) {
        return format!("{} {}", *_MUSH_VERDICT, *_MUSH_FIX);
    }
    format!("{} {}", *_MUSH_FIX, *_MUSH_VERDICT)
}

/// The longest name of a table row in the padded line (" name " or starting with "name ").
fn best_row<'a>(said: &str, table: impl Iterator<Item = (&'a Vec<String>, &'a String)>) -> Option<String> {
    let mut best: Option<(usize, &String)> = None;
    for (names, reply) in table {
        for name in names {
            if said.contains(&format!(" {name} ")) || said.starts_with(&format!("{name} ")) {
                let len = name.chars().count();
                if best.is_none_or(|(l, _)| len > l) {
                    best = Some((len, reply));
                }
                break;
            }
        }
    }
    best.map(|(_, r)| r.clone())
}

/// The freezer answer for the sweet or the milk the line names.
pub fn _freeze_a_sweet(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let said = format!(" {} ", line.to_lowercase());
    best_row(&said, FREEZE_SWEETS.iter().map(|(names, texts)| (names, &texts[0])))
}

/// Three sauces, three lengths: a bought jar, a chilled carton, and a batch you made.
pub fn _pasta_sauce_keep(_memory: &mut Memory, understood: &Understood) -> String {
    let line = format!(" {} ", understood.clean);
    if any_in(&line, &[" freeze", " freezer", " frozen", " freezing"]) {
        return "Sauce freezes well: cool it quickly, then freeze it in meal-sized portions for \
                about 3 months, and reheat it once until it is steaming hot. A jar of sauce keeps \
                5 days in the fridge once it is opened, and sauce you made keeps 3 to 4 days - so \
                if there is more than you will use, the freezer is the better place for it."
            .to_string();
    }
    if any_in(&line, &[" carton", " cartons", " tub", " fresh pasta sauce", " chilled sauce", " fresh sauce"]) {
        return "A fresh or chilled carton of pasta sauce keeps 3 to 4 days in the fridge once it \
                is opened, and it is worth transferring it out of the carton into a tub with a \
                lid. It freezes well in portions for about 3 months; reheat it once, until it is \
                steaming hot, and bin it if it smells sour or has mould on the surface."
            .to_string();
    }
    if any_in(
        &line,
        &[
            " homemade", " made it", " made my own", " made this", " batch", " i made", " my own sauce", " left over from",
            " from scratch",
        ],
    ) {
        return "Sauce you made keeps 3 to 4 days in the fridge in a lidded tub, or 3 months in \
                the freezer in portions, and it is better for being cooled quickly and put away \
                while it is still warm rather than left out. Reheat it once, all the way through, \
                and bin it if it smells sour, fizzes or has mould on top."
            .to_string();
    }
    "A jar of pasta sauce keeps 5 days in the fridge once it is opened, with the lid back \
     on, and until the date on the jar while it is still sealed: the label's use-by is for \
     the sealed jar, not for the open one. Sauce you made yourself keeps 3 to 4 days, and \
     either freezes for about 3 months in portions - reheat it once, all the way through."
        .to_string()
}

/// The keeping answer for the cupboard food the line names.
pub fn _storecupboard_keep(_memory: &mut Memory, understood: &Understood) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let said = format!(" {} ", line.to_lowercase());
    let frozen = any_in(&said, &["freezer", "freeze", "frozen", "defrost", "thaw"]);
    let found = best_row(
        &said,
        STORECUPBOARD_KEEP.iter().map(|(names, texts)| {
            let reply = if frozen && texts.len() > 1 { &texts[1] } else { &texts[0] };
            (names, reply)
        }),
    );
    if let Some(found) = found {
        return found;
    }
    "Most of these keep far longer than the date on the packet: soy sauce and vinegar for \
     years, tea for about six months past its date, and an opened jar of peanut butter for \
     about three months. The date is a quality date on most of them, not a safety one."
        .to_string()
}

/// True when the line says the thing has already been kept somewhere for a length of time.
pub fn _kept_a_while(reading: &Understood) -> bool {
    reading.kept.is_some()
}

/// True when "cut out" is about being suited to a job, so the cold-chain row stands down.
pub fn _cut_out_for_is_an_aptitude(understood: &Understood) -> bool {
    let said = if understood.clean.is_empty() { &understood.norm_text } else { &understood.clean };
    CUT_OUT_FOR.is_match(said)
}

/// True when the line asks how long thawing something takes, so the food-state card stands down.
pub fn _a_thaw_that_is_a_job(understood: &Understood) -> bool {
    let said = if understood.clean.is_empty() { &understood.norm_text } else { &understood.clean };
    if !TIMES_A_THAW.is_match(said) {
        return false;
    }
    ASKS_HOW_LONG.is_match(said)
}

/// The mould verdict for the cheese the line names, and the general one when it names none.
pub fn _cheese_mould(_memory: &mut Memory, reading: &Understood) -> String {
    let words = format!(" {} ", reading.clean.to_lowercase());
    for name in BLUE_CHEESES.iter() {
        if words.contains(name.as_str()) {
            return format!(
                "The blue in {name} is meant to be there. Green or white fuzz on it is not, so a \
                 blue cheese that has grown it goes in the bin."
            );
        }
    }
    for name in SOFT_CHEESES.iter() {
        if words.contains(name.as_str()) {
            return format!(
                "Mould on {name} means the whole piece goes in the bin: the threads run all the \
                 way through a soft cheese, so there is nothing to cut back to."
            );
        }
    }
    "Cut the mould off hard cheese an inch past it and the rest is fine. Soft cheese, \
     cream cheese and anything already grated or sliced goes in the bin, because the \
     threads reach further than you can see."
        .to_string()
}

/// Which sentence of the row the line is asking for: "test", "trick" or "keep".
pub fn _ripen_lead(text: &str) -> &'static str {
    if any_in(text, &TELL_ASKS) {
        return "test";
    }
    if any_in(text, &RIPEN_ASKS) {
        return "trick";
    }
    if any_in(text, &KEEP_ASKS) {
        return "keep";
    }
    "test"
}

/// `_ripens(test, trick, keep).<locals>.reply`: the ripeness test, the trick to ripen it, or how
/// it keeps, in the order the line asks for them.
pub fn _ripens_reply(test: &str, trick: &str, keep: &str, understood: &Understood) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let text = line.to_lowercase();
    let mut lead = _ripen_lead(&text);
    if lead == "test" && !any_in(&text, &TELL_ASKS) {
        // Python reads str(leans_on): the repr of the dict the reading holds.
        if let Some(leans) = &understood.leans_on {
            lead = _ripen_lead(&format!("{} {}", text, leans.py_repr().to_lowercase()));
        }
    }
    if lead == "keep" {
        return keep.to_string();
    }
    if lead == "trick" {
        return format!("{trick} {test}");
    }
    format!("{test} {trick}")
}

fn ripens_reply(f: &FnRef, _memory: &mut Memory, understood: &Understood) -> Option<String> {
    let cell = |name: &str| f.cell(name).map(|v| v.str().to_string()).unwrap_or_default();
    Some(_ripens_reply(&cell("test"), &cell("trick"), &cell("keep"), understood))
}

/// The current line with the line it leans on and the last few turns behind it.
pub fn _ripen_haystack(understood: &Understood, memory: &Memory) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let mut parts = vec![line.clone()];
    // Python appends leans_on["text"] when it is a dict.
    if let Some(leans) = &understood.leans_on {
        parts.push(leans.text.clone());
    }
    let turns = &memory.turns;
    if !turns.is_empty() {
        for turn in &turns[turns.len().saturating_sub(3)..] {
            parts.push(turn.user.clone());
        }
    }
    parts.join(" ").to_lowercase()
}

/// The table row whose fruit the text names: (test, trick, keep), or None.
fn ripen_row_for(text: &str) -> Option<&'static Vec<String>> {
    for (_names, texts, rxs) in RIPEN_TABLE.iter() {
        if rxs.iter().any(|rx| rx.is_match(text)) {
            return Some(texts);
        }
    }
    None
}

/// The table row whose fruit the text names, as (names, test, trick, keep), or None.
pub fn _ripen_row_for(text: &str) -> Option<(Vec<String>, String, String, String)> {
    for (names, texts, rxs) in RIPEN_TABLE.iter() {
        if rxs.iter().any(|rx| rx.is_match(text)) {
            return Some((names.clone(), texts[0].clone(), texts[1].clone(), texts[2].clone()));
        }
    }
    None
}

/// "will they ripen" and "how long until they are ready": the fruit was named a line ago.
pub fn _ripen_from_the_chat(memory: &mut Memory, understood: &Understood) -> String {
    if let Some(texts) = ripen_row_for(&_ripen_haystack(understood, memory)) {
        return format!("{} {}", texts[1], texts[0]);
    }
    "Most fruit ripens on the counter in 2 to 4 days, and a paper bag with a banana in it \
     halves that; the fridge is what stops it, so leave them out until they give. Once \
     they give, they keep a few days in the fridge."
        .to_string()
}

/// Whether the line says any of the words (clean, normalised and typed lines together).
pub fn _said_anything<S: AsRef<str>>(understood: &Understood, words: &[S]) -> bool {
    any_in(&clean_norm_text(understood), words)
}

/// Crisps, biscuits and crackers gone soft: the verdict first when the line asks if it is safe.
pub fn _crunch_reply(_memory: &mut Memory, understood: &Understood) -> String {
    if _said_anything(understood, &_CRUNCH_SAFE) {
        return format!("{} {}", *_CRUNCH_VERDICT, *_CRUNCH_FIX);
    }
    format!("{} {}", *_CRUNCH_FIX, *_CRUNCH_VERDICT)
}

/// Whey on a yoghurt, and how to keep it from separating when the line asks.
pub fn _yoghurt_reply(_memory: &mut Memory, understood: &Understood) -> String {
    if _said_anything(understood, &["keep", "keeping", "stop", "runny", "watery", "separate"]) {
        return format!("{} {}", *_YOGHURT_FIX, *_YOGHURT_WHEY);
    }
    _YOGHURT_WHEY.clone()
}

/// Lumpy flour, or the mites when the line says something is moving in it.
pub fn _flour_reply(_memory: &mut Memory, understood: &Understood) -> String {
    if _said_anything(
        understood,
        &["weevil", "weevils", "mite", "mites", "bug", "bugs", "worm", "worms", "moving", "insects", "maggots"],
    ) {
        return _FLOUR_MITES.clone();
    }
    _FLOUR_LUMPS.clone()
}

/// True when the line names the fruit, the vegetable or the punnet that has turned.
pub fn _produce_is_named(understood: &Understood) -> bool {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let low = format!(" {} ", text::squash(&line.to_lowercase()));
    OFF_NAMES.iter().any(|name| low.contains(&format!(" {name} ")))
}

/// The singular of a produce name, for the sentence that names it.
pub fn _off_one(name: &str) -> String {
    if let Some(stem) = name.strip_suffix("ies") {
        return format!("{stem}y");
    }
    for ending in ["oes", "ches", "shes", "sses"] {
        if name.ends_with(ending) {
            return name[..name.len() - 2].to_string();
        }
    }
    if name == "leaves" || name == "leafes" {
        return "leaf".to_string();
    }
    if name.ends_with('s') && !name.ends_with("ss") {
        return name[..name.len() - 1].to_string();
    }
    name.to_string()
}

/// The mould verdict for the produce the line names, in the kitchen rather than the garden.
pub fn _produce_gone_off(_memory: &mut Memory, reading: &Understood) -> String {
    let said = format!(" {} ", reading.clean.to_lowercase());
    let stripped = text::strip(&said).to_string();

    // The one produce name in the line from this group (longest form first); a category word
    // gives None unless the branch takes categories.
    let thing = |group: &[String], categories: bool| -> Option<String> {
        let mut sorted: Vec<&String> = group.iter().collect();
        sorted.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
        for name in sorted {
            if said.contains(&format!(" {name} ")) || stripped.starts_with(&format!("{name} ")) {
                if _OFF_GROUP_WORDS.contains(name) {
                    return if categories { Some(name.clone()) } else { None };
                }
                return Some(name.clone());
            }
        }
        None
    };
    // ("the tomatoes", true) for a plural, ("the melon", false) for one.
    let phrase = |name: Option<String>| -> (Option<String>, bool) {
        match name {
            None => (None, false),
            Some(name) => {
                let one = _off_one(&name);
                let plural = one != name;
                (Some(format!("the {name}")), plural)
            }
        }
    };

    let berries = phrase(thing(&OFF_BERRIES, false));
    let leaves = phrase(thing(&OFF_LEAVES, true));
    let citrus = phrase(thing(&OFF_CITRUS, false));
    let soft = phrase(thing(&OFF_SOFT, false));
    let firm = phrase(thing(&OFF_FIRM, false));

    if let Some(b) = berries.0.filter(|s| !s.is_empty()) {
        return format!(
            "Bin every {} with fuzz on it and the ones touching it, then wash the rest and eat \
             them today. On a berry the mould threads run under the skin where you cannot see \
             them, so there is nothing to cut back to, and a closed punnet passes it on fast. \
             Spread what is sound on kitchen paper in the fridge and it will hold a day or \
             two.",
            _off_one(&b[4..])
        );
    }
    if leaves.0.is_some_and(|s| !s.is_empty()) {
        return "Pick out the slimy or furry leaves and bin them: that slime is bacteria, not \
                water, and cooking will not make it safe to eat. Wash the rest well and eat or \
                cook them today, because in a closed bag it spreads to whatever touches it. Dry \
                the next lot in a tea towel and keep them in a bag with a sheet of kitchen \
                paper, because wet leaves in a sealed bag are what went wrong in the first place."
            .to_string();
    }
    if let Some(c) = citrus.0.filter(|s| !s.is_empty()) {
        return format!(
            "Bin {c} and check the rest of the bag. Mould on citrus starts on the skin and the \
             flesh under it is soft and wet, so there is nothing worth cutting back to - and \
             dry the drawer and the bag, because a damp one passes it on. The sound ones keep \
             a couple of weeks loose in the fridge."
        );
    }
    if let Some(s) = soft.0.filter(|s| !s.is_empty()) {
        return format!(
            "Bin {}. {} mostly water, so the mould threads run right through {} and there is \
             nothing to cut back to, and cooking does not make it safe again. A soft patch \
             with no mould on it is just older - trim that away and cook the rest today.",
            s,
            if soft.1 { "They are" } else { "It is" },
            if soft.1 { "them" } else { "it" }
        );
    }
    if let Some(f) = firm.0.filter(|s| !s.is_empty()) {
        return format!(
            "Cut the mould off with an inch to spare and use the rest, unless the flesh under \
             it is soft or smells wrong. A firm fruit or root does not carry the threads deep, \
             so a small patch on an apple or a carrot is not the whole of it - and bin {f}, \
             because it spreads to what it touches in a drawer."
        );
    }
    "Anything soft and wet - a berry, a tomato, a leaf - goes in the bin once it has fuzz \
     on it, and the sound ones it touched are washed and eaten today. A firm fruit or root \
     can have the patch cut away with an inch to spare, unless the flesh underneath is \
     soft or smells off."
        .to_string()
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.keeping:<lambda>:1194", |_: &FnRef, u: &Understood| !_cut_out_for_is_an_aptitude(u)),
    ("bot.knowledge.keeping:<lambda>:1200", |_: &FnRef, u: &Understood| !_a_thaw_that_is_a_job(u)),
    ("bot.knowledge.keeping:_dated_row.<locals>.<lambda>:986", |_: &FnRef, u: &Understood| !_kept_a_while(u)),
    ("bot.knowledge.keeping:_produce_is_named:1994", |_: &FnRef, u: &Understood| _produce_is_named(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.keeping:_cheese_mould:1174", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_cheese_mould(m, u))),
    ("bot.knowledge.keeping:_crunch_reply:1821", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_crunch_reply(m, u))),
    ("bot.knowledge.keeping:_flour_reply:1833", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_flour_reply(m, u))),
    ("bot.knowledge.keeping:_freeze_a_sweet:621", |_: &FnRef, m: &mut Memory, u: &Understood| _freeze_a_sweet(m, u)),
    ("bot.knowledge.keeping:_oats_kept:53", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_oats_kept(m, u))),
    ("bot.knowledge.keeping:_pasta_sauce_keep:662", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_pasta_sauce_keep(m, u))
    }),
    ("bot.knowledge.keeping:_produce_gone_off:2034", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_produce_gone_off(m, u))
    }),
    ("bot.knowledge.keeping:_ripen_from_the_chat:1629", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_ripen_from_the_chat(m, u))
    }),
    ("bot.knowledge.keeping:_ripens.<locals>.reply:1539", ripens_reply),
    ("bot.knowledge.keeping:_slimy_mushrooms:258", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_slimy_mushrooms(m, u))
    }),
    ("bot.knowledge.keeping:_storecupboard_keep:927", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_storecupboard_keep(m, u))
    }),
    ("bot.knowledge.keeping:_yoghurt_reply:1827", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_yoghurt_reply(m, u))),
];


// ---------------------------------------------------------------------------------------------
// The row factories (`knowledge::built`).

use crate::data::Value;
use crate::knowledge::built;
use crate::knowledge::entry::{Entry, Reply};

/// `_dated_row(row)`: a pack-date row from a `DATE_ROWS` row (names, reply[, more asks]).
pub fn _dated_row(row: &Value) -> Entry {
    let r = row.items();
    let names = r[0].strs();
    let mut asks = data::get("bot.knowledge.keeping.DATE_ASK").strs();
    if r.len() > 2 {
        asks.extend(r[2].strs());
    }
    let stand_down = data::get("bot.knowledge.keeping.DATED_STAND_DOWN").get(&names[0]).cloned().unwrap_or(Value::None);
    built::entry(
        &format!("keep-{}-dated", names[0]),
        vec![built::part(&names), built::part(&asks)],
        built::reply_of(&r[1]),
        &[stand_down],
        1,
        vec![],
        Some(built::function("bot.knowledge.keeping:_dated_row.<locals>.<lambda>:986")),
    )
}

/// `_ripens(test, trick, keep)`: the ripeness reply over its three sentences (`_ripens_reply`).
pub fn _ripens(test: &str, trick: &str, keep: &str) -> FnRef {
    built::closure(
        "bot.knowledge.keeping:_ripens.<locals>.reply:1539",
        vec![("keep", Value::from(keep)), ("test", Value::from(test)), ("trick", Value::from(trick))],
    )
}

#[cfg(test)]
mod factory_tests {
    #[test]
    fn dated_rows_rebuild() {
        crate::data::load();
        let rows: Vec<_> = crate::data::get("bot.knowledge.keeping.DATE_ROWS").items().iter().map(super::_dated_row).collect();
        crate::knowledge::built::assert_rows_match(&rows, "bot.knowledge.keeping.ENTRIES");
    }
}
