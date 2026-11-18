#![allow(non_snake_case)] // closures are named <factory>__<inner>
//! Port of PyBot's `bot/knowledge/explains.py`: how things work, what abbreviations stand for,
//! the difference between two things, which of two to choose, the coil, and civics.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g7 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn key(name: &str) -> String {
    format!("bot.knowledge.explains.{name}")
}
fn rx(name: &str) -> &'static Re {
    data::re(&key(name))
}
fn strs(name: &str) -> Vec<String> {
    data::strs(&key(name))
}
static _HOW_TO_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOW_TO_ASK"));
static _ASKS_THE_MECHANISM: LazyLock<&'static Re> = LazyLock::new(|| rx("_ASKS_THE_MECHANISM"));
static _ASK_BEFORE: LazyLock<&'static Re> = LazyLock::new(|| rx("_ASK_BEFORE"));
static _ASK_AFTER: LazyLock<&'static Re> = LazyLock::new(|| rx("_ASK_AFTER"));
static _A_DEVICE_IN_THE_LINE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_DEVICE_IN_THE_LINE"));
static _SPACE_AS_A_NEED: LazyLock<&'static Re> = LazyLock::new(|| rx("_SPACE_AS_A_NEED"));
static _SPACE_FOR_A_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("_SPACE_FOR_A_THING"));
static _SPACE_IN_A_PLACE: LazyLock<&'static Re> = LazyLock::new(|| rx("_SPACE_IN_A_PLACE"));
static _SPACE_TO_DO_A_JOB: LazyLock<&'static Re> = LazyLock::new(|| rx("_SPACE_TO_DO_A_JOB"));
static _A_SIZE_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_SIZE_ASK"));
static _THE_ASK_TO_MAKE_ROOM: LazyLock<&'static Re> = LazyLock::new(|| rx("_THE_ASK_TO_MAKE_ROOM"));
static _ANOTHER_COUNTRY: LazyLock<&'static Re> = LazyLock::new(|| rx("_ANOTHER_COUNTRY"));
static _A_PARTY_TALLY: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_PARTY_TALLY"));
static DIFFERENCE_MINE: LazyLock<&'static Re> = LazyLock::new(|| rx("DIFFERENCE_MINE"));
static DIFFERENCE_STRONG_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("DIFFERENCE_STRONG_ASKS"));
static DIFFERENCE_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&key("DIFFERENCE_FILLER")));
static NOT_PLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));
static WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// A list of names, each with its `(?<![a-z])name(?![a-z])` pattern (`_names_in`'s test).
pub struct Names(pub Vec<(String, Re)>);

impl Names {
    pub fn new(names: Vec<String>) -> Names {
        Names(names.into_iter().map(|n| {
            let re = Re::new(&format!("(?<![a-z]){}(?![a-z])", sh::escape(&n)), 0);
            (n, re)
        }).collect())
    }
    /// `_names_in(text, names)`.
    pub fn in_text(&self, text: &str) -> bool {
        self.0.iter().any(|(_, re)| re.is_match(text))
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// `DIFFERENCES`: (left names, right names, reply).
static DIFFERENCES: LazyLock<Vec<(Names, Names, String)>> = LazyLock::new(|| {
    data::get(&key("DIFFERENCES"))
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            (Names::new(r[0].strs()), Names::new(r[1].strs()), r[2].str().to_string())
        })
        .collect()
});
/// `_choosing_rows()`: (left, right, asks, reply, needs), `needs` empty for a four-part row.
static CHOOSING: LazyLock<Vec<(Names, Names, Names, String, Names)>> = LazyLock::new(|| {
    data::get(&key("CHOOSING"))
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            let needs = if r.len() == 5 { r[4].strs() } else { Vec::new() };
            (Names::new(r[0].strs()), Names::new(r[1].strs()), Names::new(r[2].strs()), r[3].str().to_string(), Names::new(needs))
        })
        .collect()
});
static _COIL: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| {
    ["_COIL_SAFE", "_COIL_EFFECT", "_COIL_PREGNANT", "_COIL_LOST", "_COIL_REMOVAL_ASK", "_COIL_SOON", "_COIL_OUT", "_COIL_YOUNG",
     "_COIL_BABY", "_COIL_AFTER", "_COIL_HURT", "_COIL_FREE", "_COIL_BLEED", "_COIL_LIFE"]
        .iter()
        .map(|n| strs(n))
        .collect()
});

#[rustfmt::skip]
const S_2698: &str = "Get it fitted at a sexual health clinic, a young people's clinic or your GP surgery: you book an appointment and the fitting takes about five minutes with a nurse or doctor. It is free on the NHS at any age - under 16 you can still have one without your parents being told, as long as the doctor thinks you understand what you are agreeing to. Expect period-like cramping for a day or two and spotting for a few weeks, so take ibuprofen an hour before, and a copper coil can make periods heavier for the first few months while the hormonal ones make them lighter or stop them. It lasts five years for the copper one and three to eight for the hormonal ones, and a nurse can take it out whenever you want."; // _the_coil
#[rustfmt::skip]
const S_2611: &str = "Yes - it is one of the safest things you can use, and it suits most people, including people who cannot take the pill and people who are breastfeeding. It does not protect against sexually transmitted infections, so use condoms as well if that matters to you. The risks are small and known: a very rare injury to the womb at fitting (about one or two in a thousand), an infection in the first few weeks, and the chance it comes out by itself, most often in the first months. The hormonal one can also bring headaches, breast tenderness, acne or low mood for the first few months, and those usually settle; the copper one mainly makes periods heavier. Ring the clinic if you get a fever, bad pain or smelly discharge after fitting."; // _the_coil
#[rustfmt::skip]
const S_2622: &str = "Over 99 in 100 for both kinds: fewer than one in a hundred people get pregnant in a year with a coil in, which is better than the pill in everyday use because there is nothing to forget. The copper one works from the moment it is fitted and for its whole life, the hormonal one for three to eight years depending on which, and neither is affected by being sick or by antibiotics the way the pill is. If you are worried it has moved, that is the one thing that drops the protection."; // _the_coil
#[rustfmt::skip]
const S_2629: &str = "Not likely, but not impossible: fewer than one in a hundred people get pregnant in a year with a coil in, which makes it one of the most reliable methods there is - more so than the pill, because there is nothing to forget. If you think you might be pregnant with one in place, take a test and ring the clinic that fitted it the same day rather than pulling it out yourself: they will check it is still where it should be, and a coil can stay in through a pregnancy with a check. Use condoms until they have seen you."; // _the_coil
#[rustfmt::skip]
const S_2638: &str = "Treat it as no protection: use condoms, or the morning after pill if you have had sex since it moved, and get seen at a sexual health clinic or your GP within a few days - they check the strings and fit a new one if it has gone, which is quick and free. Do not try to put it back or pull it out yourself. A coil can come out on its own, most often in the first few months or during a heavy period, and it is more likely if you have not had a baby."; // _the_coil
#[rustfmt::skip]
const S_2645: &str = "A copper coil protects from the moment it is fitted. A hormonal one does too if it goes in during the first seven days of your period; fitted at another point in the cycle, use condoms for the first seven days. Neither needs a wait after that, and neither is affected by being sick, by antibiotics or by a missed pill - the protection does not lapse between coils either, because a new one can go in the same appointment as the old comes out."; // _the_coil
#[rustfmt::skip]
const S_2652: &str = "A nurse takes it out in a few minutes, at any time you want - you do not have to wait for the years to run out, and any sexual health clinic or the place that fitted it will do it. It is free on the NHS, and fertility comes back straight away, so use another method from about a week before if you do not want to be pregnant. If it has moved, or you cannot feel the strings, say so when you ring."; // _the_coil
#[rustfmt::skip]
const S_2658: &str = "Yes, at any age: it is free on the NHS, and under 16 you can have one without your parents being told, as long as the doctor or nurse thinks you understand what you are agreeing to. A young people's clinic is usually the easiest place to ask, and they will talk you through the copper and hormonal ones before you decide. Nothing goes on a record your parents can see."; // _the_coil
#[rustfmt::skip]
const S_2664: &str = "Yes, and often before you leave hospital: a coil can go in straight after a vaginal birth or at a caesarean, and otherwise from six weeks afterwards. It does not affect your milk, and the hormonal one is fine while you are feeding. It is free, and the fitting is the same few minutes at a clinic or your GP surgery."; // _the_coil
#[rustfmt::skip]
const S_2669: &str = "A day or two of period-like cramping, some spotting and feeling a bit rough is normal in the first days after a coil goes in, and it settles on its own: ibuprofen, a hot water bottle and an early night do most of it. Ring the clinic the same day if you have a fever, pain that gets worse rather than better, smelly discharge or heavy bleeding with clots - that is usually an infection, and it clears up quickly with antibiotics. Feeling faint at the fitting itself is common too, so it is worth having someone with you."; // _the_coil
#[rustfmt::skip]
const S_2677: &str = "It hurts, but briefly: fitting is a sharp period-like cramp for about a minute while it goes in, then a dull ache for a day or two and spotting for a few weeks. Take ibuprofen an hour before, eat something first, and arrange a lift home. Most people say it was easier than they feared, and the hormonal ones are usually gentler to fit than the copper coil - so say at the clinic if you are dreading it and ask about a local anaesthetic."; // _the_coil
#[rustfmt::skip]
const S_2684: &str = "Free on the NHS, at any age and whatever your income - you never pay for contraception, and that includes the coil and the fitting. It is done at a sexual health clinic, a young people's clinic or your GP surgery, and under 16 you can have one without your parents being told."; // _the_coil
#[rustfmt::skip]
const S_2689: &str = "The hormonal coil usually makes periods much lighter, and about one in five people stop having them altogether, which is not harmful. The copper one can make them heavier and more painful for the first three months. If you are bleeding between periods or after sex once it has settled in, get it checked."; // _the_coil
#[rustfmt::skip]
const S_2694: &str = "Five years for most copper coils, and three to eight for the hormonal ones - Mirena eight, Kyleena five, Jaydess three. Those are maximums, not a promise, so ask at the clinic which one they are fitting, and you can have it out sooner at any time you want."; // _the_coil

// `_explain(...)`, `_as_written(words)`, `_about_the_word(subject)`, `_both(*guards)`,
// `_name(...)`, `_civics(...)`, `_civics_new(...)` and `_choosing_rows()` build entries and
// tables at import time; the closures they make are ported below and read their cells.

/// The `_explain` guard: not a how-to line, and the row's own guard (`guard` cell) when it has one.
pub fn _explain__lambda(guard: Option<&Value>, understood: &Understood) -> bool {
    _not_a_how_to_ask(understood) && guard.is_some_and(|g| run_guard(g, understood))
}

/// False on a "how to" line, unless it asks how the thing works.
pub fn _not_a_how_to_ask(understood: &Understood) -> bool {
    let said = text::squash(&sh::clean_or_text(understood).to_lowercase());
    if !_HOW_TO_ASK.is_match(&said) {
        return true;
    }
    _ASKS_THE_MECHANISM.is_match(&said)
}

/// False when the person is telling (a word starting "tell", not "telly"), not asking.
pub fn _not_a_telling(understood: &Understood) -> bool {
    !understood.words.iter().any(|w| {
        let w = w.to_lowercase();
        w.starts_with("tell") && !["telly", "tellies", "tellys"].contains(&w.as_str())
    })
}

/// False on a garden line (the topic, or a garden word in the line).
pub fn _not_a_garden_chat(understood: &Understood) -> bool {
    let mut topics: Vec<&str> = Vec::new();
    if let Some(t) = &understood.topic {
        topics.push(t);
    }
    topics.extend(understood.topics.iter().map(String::as_str));
    if topics.contains(&"garden-and-plants") {
        return false;
    }
    let line = format!(" {} ", NOT_PLAIN.sub(" ", &sh::clean_or_text(understood).to_lowercase()));
    !["garden", "gardens", "lawn", "grass", "allotment", "allotments", "plot", "borders", "hedge", "hedges", "flowerbed", "flowerbeds", "mow", "mowing"]
        .iter()
        .any(|w| line.contains(&format!(" {w} ")))
}

/// `_as_written(words)`'s guard: a single word as typed (not inside another word), or a phrase.
pub fn _as_written__guard(single: &[Value], phrased: &[Value], understood: &Understood) -> bool {
    let text = format!(" {} ", sh::clean_or_text(understood).to_lowercase());
    for word in single {
        if sh::re_search(&format!("(?<![a-z]){}(?![a-z])", sh::escape(word.str())), &text, 0) {
            return true;
        }
    }
    phrased.iter().any(|w| text.contains(&format!(" {} ", w.str())))
}

/// `_about_the_word(subject)`'s guard: the word is asked about ("what does X mean", "X stands
/// for"), or is the whole line.
pub fn _about_the_word__guard(pattern: &Re, understood: &Understood) -> bool {
    let joined;
    let mut text = sh::clean_or_text(understood);
    if text.is_empty() {
        joined = understood.words.join(" ");
        text = &joined;
    }
    let text = text::squash(&text.to_lowercase());
    if text.is_empty() {
        return false;
    }
    for found in pattern.finditer(&text) {
        let (before, after) = (&text[..found.start()], &text[found.end()..]);
        if _ASK_BEFORE.is_match(before) {
            return true;
        }
        if _ASK_AFTER.match_(after).is_some() {
            return true;
        }
        if text::strip_chars(before, " .!?,-").is_empty() && text::strip_chars(after, " .!?,-").is_empty() {
            return true;
        }
    }
    false
}

/// `_both(*guards)`'s guard: every one of the guards passes.
pub fn _both__guard(guards: &[Value], understood: &Understood) -> bool {
    guards.iter().all(|g| run_guard(g, understood))
}

/// Run a guard held in a closure cell: this module's closures directly, anything else through
/// the registry.
fn run_guard(g: &Value, understood: &Understood) -> bool {
    match g.as_fn() {
        Some(f) => match f.name() {
            "bot.knowledge.explains:_as_written.<locals>.guard" => g_as_written(f, understood),
            "bot.knowledge.explains:_about_the_word.<locals>.guard" => g_about_the_word(f, understood),
            "bot.knowledge.explains:_both.<locals>.guard" => g_both(f, understood),
            "bot.knowledge.explains:_not_a_telling" => _not_a_telling(understood),
            "bot.knowledge.explains:_not_a_garden_chat" => _not_a_garden_chat(understood),
            "bot.knowledge.explains:_not_a_how_to_ask" => _not_a_how_to_ask(understood),
            _ => sh::call_guard(g, understood),
        },
        None => false,
    }
}

/// False when "space" in the line is room to put something or do a job (the size or storage
/// rows), not space in a relationship.
pub fn not_the_space_in_a_relationship(understood: &Understood) -> bool {
    let joined;
    let mut said = sh::text_or_clean(understood);
    if said.is_empty() {
        joined = understood.words.join(" ");
        said = &joined;
    }
    if _A_SIZE_ASK.is_match(said) {
        return true;
    }
    if _A_DEVICE_IN_THE_LINE.is_match(said) && !_THE_ASK_TO_MAKE_ROOM.is_match(said) {
        return true;
    }
    if _SPACE_AS_A_NEED.is_match(said) {
        return false;
    }
    if _SPACE_FOR_A_THING.is_match(said) || _SPACE_IN_A_PLACE.is_match(said) || _SPACE_TO_DO_A_JOB.is_match(said) {
        return false;
    }
    true
}

/// `_names_in(text, names)`: any of the names in the text as a word.
pub fn _names_in<S: AsRef<str>>(text: &str, names: &[S]) -> bool {
    names.iter().any(|n| sh::re_search(&format!("(?<![a-z]){}(?![a-z])", sh::escape(n.as_ref())), text, 0))
}

/// Whether a line naming both things asks their difference (a strong ask, "which", a question
/// mark, or a short "x or y" / bare pair).
pub fn _is_a_difference_ask(text: &str, raw: &str, left: &[String], right: &[String]) -> bool {
    if sh::any_in(text, &DIFFERENCE_STRONG_ASKS) {
        return true;
    }
    if text.contains("which") || raw.contains('?') {
        return true;
    }
    if DIFFERENCE_MINE.is_match(text) && !raw.contains("or") {
        return false;
    }
    let words: Vec<&str> = WORD.finditer(raw).iter().map(|m| m.as_str()).filter(|w| !DIFFERENCE_FILLER.contains(*w)).collect();
    if raw.contains("or") {
        return words.len() <= 5;
    }
    let names: HashSet<&str> = left.iter().chain(right.iter()).flat_map(|n| text::split_ref(n)).collect();
    words.iter().all(|w| names.contains(w))
}

/// The difference card for the pair of things the line names and asks about.
pub fn _difference(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = sh::clean_or_text(understood).to_lowercase();
    let raw = understood.text.to_lowercase();
    for (left, right, reply) in DIFFERENCES.iter() {
        if !(left.in_text(&text) && right.in_text(&text)) {
            continue;
        }
        let l: Vec<String> = left.0.iter().map(|(n, _)| n.clone()).collect();
        let r: Vec<String> = right.0.iter().map(|(n, _)| n.clone()).collect();
        if !_is_a_difference_ask(&text, &raw, &l, &r) {
            continue;
        }
        return Some(reply.clone());
    }
    None
}

/// The coil card the line asks for (safety, effectiveness, pregnancy, coming out, when it
/// works, removal, age, after a baby, after fitting, pain, cost, bleeding, how long), or the
/// general card.
pub fn _the_coil(_memory: &Memory, understood: &Understood) -> String {
    let said = _coil_line(understood);
    let says = |i: usize| sh::any_in(&said, &_COIL[i]);
    if says(0) {
        return S_2611.into();
    }
    if says(1) {
        return S_2622.into();
    }
    if says(2) {
        return S_2629.into();
    }
    if says(3) && !says(4) {
        return S_2638.into();
    }
    if says(5) {
        return S_2645.into();
    }
    if says(6) {
        return S_2652.into();
    }
    if says(7) {
        return S_2658.into();
    }
    if says(8) {
        return S_2664.into();
    }
    if says(9) {
        return S_2669.into();
    }
    if says(10) {
        return S_2677.into();
    }
    if says(11) {
        return S_2684.into();
    }
    if says(12) {
        return S_2689.into();
    }
    if says(13) {
        return S_2694.into();
    }
    S_2698.into()
}

/// The line as text, plain and clean joined, lower-cased.
pub fn _coil_line(understood: &Understood) -> String {
    format!("{} {} {}", understood.text, understood.plain, understood.clean).to_lowercase()
}

/// The card for a choice between two named options the line asks about (a trade or DIY, ...).
pub fn _choosing(_memory: Option<&Memory>, understood: &Understood) -> Option<String> {
    let text = sh::clean_or_text(understood).to_lowercase();
    for (left, right, asks, reply, needs) in CHOOSING.iter() {
        if !left.in_text(&text) {
            continue;
        }
        if !right.is_empty() && !right.in_text(&text) {
            continue;
        }
        if !asks.in_text(&text) {
            continue;
        }
        if !needs.is_empty() && !needs.in_text(&text) {
            continue;
        }
        return Some(reply.clone());
    }
    None
}

/// The choosing row answers only when one of its options is named, and the line does not ask
/// the order of steps.
pub fn _one_of_them_is_actually_named(understood: &Understood) -> bool {
    _choosing(None, understood).is_some() && dup::not_the_ask_about_the_order(understood)
}

/// False when the line is about another country's government.
pub fn _only_this_country(understood: &Understood) -> bool {
    !_ANOTHER_COUNTRY.is_match(sh::text_or_clean(understood))
}

/// False when the line asks one party's tally of seats or votes.
pub fn _not_one_party(understood: &Understood) -> bool {
    !_A_PARTY_TALLY.is_match(sh::text_or_clean(understood))
}

fn cell_items<'a>(f: &'a FnRef, name: &str) -> &'a [Value] {
    f.cell(name).map(Value::items).unwrap_or_default()
}
fn g_explain(f: &FnRef, u: &Understood) -> bool {
    _explain__lambda(f.cell("guard"), u)
}
fn g_as_written(f: &FnRef, u: &Understood) -> bool {
    _as_written__guard(cell_items(f, "single"), cell_items(f, "phrased"), u)
}
fn g_about_the_word(f: &FnRef, u: &Understood) -> bool {
    f.cell("pattern").and_then(Value::as_re).is_some_and(|p| _about_the_word__guard(p, u))
}
fn g_both(f: &FnRef, u: &Understood) -> bool {
    _both__guard(cell_items(f, "guards"), u)
}
macro_rules! guard {
    ($name:ident, $f:ident) => {
        fn $name(_f: &FnRef, u: &Understood) -> bool {
            $f(u)
        }
    };
}
guard!(g_how_to, _not_a_how_to_ask);
guard!(g_telling, _not_a_telling);
guard!(g_garden, _not_a_garden_chat);
guard!(g_space, not_the_space_in_a_relationship);
guard!(g_named, _one_of_them_is_actually_named);
guard!(g_country, _only_this_country);
guard!(g_party, _not_one_party);
fn r_difference(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _difference(m, u)
}
fn r_coil(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_the_coil(m, u))
}
fn r_choosing(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _choosing(Some(m), u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.explains:_about_the_word.<locals>.guard:251", g_about_the_word),
    ("bot.knowledge.explains:_as_written.<locals>.guard:206", g_as_written),
    ("bot.knowledge.explains:_both.<locals>.guard:277", g_both),
    ("bot.knowledge.explains:_explain.<locals>.<lambda>:133", g_explain),
    ("bot.knowledge.explains:_not_a_garden_chat:175", g_garden),
    ("bot.knowledge.explains:_not_a_how_to_ask:151", g_how_to),
    ("bot.knowledge.explains:_not_a_telling:160", g_telling),
    ("bot.knowledge.explains:_not_one_party:3102", g_party),
    ("bot.knowledge.explains:_one_of_them_is_actually_named:3013", g_named),
    ("bot.knowledge.explains:_only_this_country:3096", g_country),
    ("bot.knowledge.explains:not_the_space_in_a_relationship:354", g_space),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.explains:_choosing:2994", r_choosing),
    ("bot.knowledge.explains:_difference:2515", r_difference),
    ("bot.knowledge.explains:_the_coil:2604", r_coil),
];


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/knowledge/explains.py:130 `_explain(id, subject, reply, bias=1, avoid=(), guard=None)` (never called on the trace corpus)
    pub fn _explain(id: &Value, subject: &str, reply: &str, bias: &Value, avoid: &Value, guard: &Value) -> Value {
        crate::unported!("bot.knowledge.explains:_explain")
    }

    /// Python: bot/knowledge/explains.py:193 `_as_written(words)` (never called on the trace corpus)
    pub fn _as_written(words: &[String]) -> Value {
        crate::unported!("bot.knowledge.explains:_as_written")
    }

    /// Python: bot/knowledge/explains.py:240 `_about_the_word(subject)` (never called on the trace corpus)
    pub fn _about_the_word(subject: &str) -> Value {
        crate::unported!("bot.knowledge.explains:_about_the_word")
    }

    /// Python: bot/knowledge/explains.py:270 `_both()` (never called on the trace corpus)
    pub fn _both(guards: &[Value]) -> Value {
        crate::unported!("bot.knowledge.explains:_both")
    }

    /// Python: bot/knowledge/explains.py:282 `_name(id, subject, reply, bias=2, avoid=())` (never called on the trace corpus)
    pub fn _name(id: &Value, subject: &str, reply: &str, bias: &Value, avoid: &Value) -> Value {
        crate::unported!("bot.knowledge.explains:_name")
    }

    /// Python: bot/knowledge/explains.py:2974 `_choosing_rows()`
    pub fn _choosing_rows() -> Value {
        crate::unported!("bot.knowledge.explains:_choosing_rows")
    }

    /// Python: bot/knowledge/explains.py:3125 `_civics_new(id, subject, reply, also=(), avoid=(), asks=None, guard=None)` (never called on the trace corpus)
    pub fn _civics_new(id: &Value, subject: &str, reply: &str, also: &Value, avoid: &Value, asks: &Value, guard: &Value) -> Value {
        crate::unported!("bot.knowledge.explains:_civics_new")
    }

    /// Python: bot/knowledge/explains.py:3131 `_civics(id, subject, reply, also=(), avoid=(), asks=None, guard=None, blocked=_NOT_CIVICS, ask=None)` (never called on the trace corpus)
    pub fn _civics(id: &Value, subject: &str, reply: &str, also: &Value, avoid: &Value, asks: &Value, guard: &Value, blocked: &Value, ask: &str) -> Value {
        crate::unported!("bot.knowledge.explains:_civics")
    }

}

