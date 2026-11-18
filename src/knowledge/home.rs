//! Port of PyBot's `bot/knowledge/home.py`: home and DIY - the jobs around a flat or a house
//! (appliance faults and noises, what a machine costs to run and how long it lasts, mould, pipes,
//! the stopcock and the fuse box, a trade to call, radiators, fittings, crafts, and the counts a
//! job with a tape measure needs).
//!
//! The module's tables, reply texts held in globals and its compiled regexes come from the
//! bundle; the reply texts written inside functions are kept here word for word. `ENTRIES` is
//! data (built at import time, with the rows of `home_craft.entries()` in front).

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g1 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// Bundle access.

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.home.{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.home.{name}"))
}
fn strs(name: &str) -> Vec<String> {
    g(name).strs()
}
fn set(name: &str) -> HashSet<String> {
    g(name).str_set()
}
fn string(name: &str) -> String {
    g(name).str().to_string()
}
/// A tuple of `(word, name)` pairs.
fn pairs_of(v: &Value) -> Vec<(String, String)> {
    v.items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
}
/// A tuple of `(words, text)` rows.
fn word_rows(v: &Value) -> Vec<(Vec<String>, String)> {
    v.items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
}
/// A `{str: str}` dict in insertion order.
fn str_dict(v: &Value) -> Vec<(String, String)> {
    v.pairs().iter().map(|(k, x)| (k.str().to_string(), x.str().to_string())).collect()
}
fn lookup<'a>(table: &'a [(String, String)], key: &str) -> Option<&'a str> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}
/// `r"\b" + re.escape(word) + suffix` built once per word.
fn word_re(word: &str, suffix: &str) -> &'static Re {
    sh::re_cached(&format!(r"\b{}{}", sh::escape(word), suffix), 0)
}
/// The first character of a name is a vowel: "an" rather than "a".
fn starts_with_vowel(s: &str) -> bool {
    s.chars().next().is_some_and(|c| "aeiou".contains(c))
}
/// `str(understood.get(key) or "")` for the keys the reading has, in a joined line.
fn joined(parts: &[&str]) -> String {
    parts.join(" ")
}

static SMALL_APPLIANCES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs_of(g("SMALL_APPLIANCES")));
static GARDEN_MACHINES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs_of(g("GARDEN_MACHINES")));
static WHITE_GOODS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs_of(g("WHITE_GOODS")));
/// `WHITE_GOODS + SMALL_APPLIANCES`.
static WHITE_AND_SMALL: LazyLock<Vec<(String, String)>> =
    LazyLock::new(|| WHITE_GOODS.iter().chain(SMALL_APPLIANCES.iter()).cloned().collect());

// ---------------------------------------------------------------------------------------------
// The space in the house, chemicals, smells.

/// False when the "room" in the line is a bed for the night rather than space at home.
pub fn _the_space_in_the_house(understood: &Understood) -> bool {
    understood.sense.get("room").map(String::as_str) != Some("booking")
}

static _CHEMICAL_CLASSES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows(g("_CHEMICAL_CLASSES")));
static _CHEMICAL_CHILD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_CHEMICAL_CHILD"));
static _CHEMICAL_PET: LazyLock<Vec<String>> = LazyLock::new(|| strs("_CHEMICAL_PET"));
static _CHEMICAL_EXPECTING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_CHEMICAL_EXPECTING"));

/// A poison, a powder or a cleaner, answered by who is in the house (None when nobody at risk
/// is named).
pub fn _chemical_safety(_memory: &Memory, understood: &Understood) -> Option<String> {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if words.is_empty() {
        return None;
    }
    let said = words.join(" ");
    let named = |names: &[String]| names.iter().any(|name| word_re(name, r"\b").is_match(&said));
    let mut hazard: Option<&str> = None;
    let mut best = 0;
    for (names, answer) in _CHEMICAL_CLASSES.iter() {
        for name in names {
            let n = name.chars().count();
            if n > best && named(std::slice::from_ref(name)) {
                best = n;
                hazard = Some(answer);
            }
        }
    }
    let cleaner = string("_CHEMICAL_CLEANER");
    let mut parts: Vec<String> = vec![hazard.map(str::to_string).unwrap_or(cleaner)];
    let child = named(&_CHEMICAL_CHILD);
    let pet = named(&_CHEMICAL_PET);
    let expecting = named(&_CHEMICAL_EXPECTING);
    if child {
        parts.push(string("_CHEMICAL_CHILD_RING"));
    }
    if pet {
        parts.push(string("_CHEMICAL_PET_RING"));
    }
    if expecting {
        parts.push(string("_CHEMICAL_EXPECTING_RING"));
    }
    if !(child || pet || expecting) {
        return None;
    }
    Some(parts.join(" "))
}

static PLACE_MICE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(mice|mouse|rodent|rodents|rats|rat)\b", 0));
static PLACE_WEE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(wee|urine|peed|peeing|pee|piss)\b", 0));
static PLACE_COOKING: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(chips|chip pan|fried|frying|fish|bacon|curry|onions?|garlic)\b", 0));

/// A smell in a place: the source first, then the cleaner that works on that source.
pub fn _place_smell(_memory: &Memory, understood: &Understood) -> String {
    let line = sh::squash_lower(sh::clean_or_text(understood));
    if PLACE_MICE.is_match(&line) {
        return "Look for droppings first, along the joists and behind anything stored, because in \
                a loft, a shed or an empty room that is usually mice: clear them up in gloves and \
                a mask, wet the area before sweeping so the dust does not rise, then wash it with \
                a disinfectant. Block the gaps round pipes and cables and set traps along the \
                wall."
            .into();
    }
    if PLACE_WEE.is_match(&line) {
        return "Something has soaked it up, so find that first: a hard floor or the skirting comes \
                clean with a biological (enzyme) cleaner, and bleach only hides it for a day. \
                Sniff low down and in the corners to find the patch, soak the cleaner in, press a \
                towel hard onto it, then air the room while it dries."
            .into();
    }
    if PLACE_COOKING.is_match(&line) {
        return "Open a window, then simmer a pan of water with a sliced lemon or a splash of \
                vinegar for 10 minutes: that clears cooking smells better than any spray. Check \
                the bin, the drain and the fridge as well, because fat that has gone down them \
                keeps giving the smell back."
            .into();
    }
    "Find the source before you treat the air: a damp patch, a dead mouse, a spilt bottle or \
     a blocked drain keeps giving off until it is dealt with, and a smell that comes and goes \
     after rain usually means water is getting in somewhere. Then air it hard - the door and \
     a window open with a fan pushing air through for an hour or two - and wash the hard \
     surfaces with hot soapy water. A bowl of bicarbonate of soda or vinegar left in the \
     corner takes out what is left; masking it with air freshener only works for an hour."
        .into()
}

/// A soft thing or a car seat that has taken a smell up: the cleaner, then how to dry it.
pub fn _soft_smell(_memory: &Memory, _understood: &Understood) -> String {
    "Soak a biological (enzyme) cleaner into it rather than scrubbing the surface: that is \
     what breaks down what has soaked in, and bleach or a normal carpet shampoo only \
     freshens the top. Press a towel hard onto the patch to pull the moisture out, leave \
     bicarbonate of soda on it overnight, then vacuum it and air the thing with a window \
     open until it is properly dry. A car seat wants the same job with the doors open, and \
     if the smell comes back the leak is under it, so check the floor or the foam."
        .into()
}

// ---------------------------------------------------------------------------------------------
// Ages, machine names, the repair-or-replace rule.

static YEARS_OLD: LazyLock<&'static Re> = LazyLock::new(|| rx("YEARS_OLD"));
static AGE_YEARS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d{1,2})\s*(?:year|yr)s?\b", 0));

/// The age the line gives, or None: "my 15 year old washing machine", "it is 12 years old".
pub fn _age_in(line: &str) -> Option<i64> {
    let m = YEARS_OLD.search(line).or_else(|| AGE_YEARS.search(line))?;
    m.str(1).parse().ok()
}

/// An age said as words: "eight years", "a year", "25 years".
pub fn _words_of_age(age: i64) -> String {
    let words = [
        "", "a year", "two years", "three years", "four years", "five years", "six years", "seven years",
        "eight years", "nine years", "ten years", "eleven years", "twelve years", "thirteen years",
        "fourteen years", "fifteen years", "sixteen years", "seventeen years", "eighteen years",
        "nineteen years", "twenty years",
    ];
    if (1..=20).contains(&age) {
        words[age as usize].to_string()
    } else {
        format!("{age} years")
    }
}

/// The machine the line names, read as whole words so "iron" is not "ironing" (None where
/// Python's caller passes `default=None`; callers apply their own default).
pub fn _named<'t>(said: &str, table: &'t [(String, String)]) -> Option<&'t str> {
    table.iter().find(|(word, _)| word_re(word, r"s?\b").is_match(said)).map(|(_, name)| name.as_str())
}

/// The new-beats-repair rule for a small appliance, worded for the one named.
pub fn _small_appliance(thing: &str, memory: &Memory) -> String {
    let article = if starts_with_vowel(thing) { "an" } else { "a" };
    dup::pick(
        &[
            format!(
                "{} {thing} is not worth repairing: a new one costs less than \
                 the call-out, so replace it. Buy the next one from a shop with a two-year guarantee and \
                 keep the receipt, because that is the only cover a cheap appliance is worth having.",
                text::capitalize(article)
            ),
            format!(
                "Replace the {thing} rather than repair it: the call-out alone usually costs more \
                 than a new one, and the parts are not made to be swapped. A two-year guarantee from the \
                 shop is worth more than any repair here."
            ),
        ],
        memory,
        "",
    )
}

/// The service-it rule for a garden machine, worded for the one named.
pub fn _garden_machine(thing: &str, memory: &Memory) -> String {
    let article = if starts_with_vowel(thing) { "an" } else { "a" };
    dup::pick(
        &[
            format!(
                "{} {thing} is worth a service if the engine is sound: a blade, \
                 oil and a spark plug cost a fraction of a new one, and a service is the whole of the \
                 repair most of them ever need. If the engine is seized or the body has rusted through, \
                 replace it instead.",
                text::capitalize(article)
            ),
            format!(
                "Service the {thing} if it starts and runs: a new blade, oil and a plug is a small \
                 fraction of the price of a new one. If it will not start at all, or the body has rusted \
                 through, put the money towards a new one."
            ),
        ],
        memory,
        "",
    )
}

static TROUBLE_NOW: LazyLock<&'static Re> = LazyLock::new(|| rx("TROUBLE_NOW"));
pub static CALLING_SOMEONE: LazyLock<&'static Re> = LazyLock::new(|| rx("CALLING_SOMEONE"));
static WHO_TO_CALL: LazyLock<&'static Re> = LazyLock::new(|| rx("WHO_TO_CALL"));
pub static FOLLOWING_IT: LazyLock<&'static Re> = LazyLock::new(|| rx("FOLLOWING_IT"));
static SELF_JOB_NOW: LazyLock<&'static Re> = LazyLock::new(|| rx("SELF_JOB_NOW"));
static WORTH_NOW: LazyLock<&'static Re> = LazyLock::new(|| rx("WORTH_NOW"));
static DIY_PARTS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_dict(g("DIY_PARTS")));
static MACHINE_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("MACHINE_WORD"));
pub static DECIDING_NOW: LazyLock<&'static Re> = LazyLock::new(|| rx("DECIDING_NOW"));
static SHOPPING_NOW: LazyLock<&'static Re> = LazyLock::new(|| rx("SHOPPING_NOW"));
static KEEPING_NOW: LazyLock<&'static Re> = LazyLock::new(|| rx("KEEPING_NOW"));
static NEED_A_NEW: LazyLock<&'static Re> = LazyLock::new(|| rx("NEED_A_NEW"));
static CHOOSING: LazyLock<&'static Re> = LazyLock::new(|| rx("CHOOSING"));
static OWNED_NOW: LazyLock<&'static Re> = LazyLock::new(|| rx("OWNED_NOW"));
static FAULT_NOW: LazyLock<&'static Re> = LazyLock::new(|| rx("FAULT_NOW"));

/// The machine a line names, or "" -- "washing machine", "coffee machine", "lawnmower".
pub fn machine_named(said: &str) -> String {
    let said = said.to_lowercase();
    _named(&said, &SMALL_APPLIANCES)
        .or_else(|| _named(&said, &GARDEN_MACHINES))
        .or_else(|| _named(&said, &WHITE_GOODS))
        .unwrap_or("")
        .to_string()
}

/// The line before, when it named a machine and said something was wrong with it ("" if not).
pub fn machine_before(memory: &Memory) -> String {
    let Some(last) = memory.turns.last() else { return String::new() };
    let earlier = text::strip(&last.user).to_lowercase();
    if !_machine_turn(&earlier) {
        return String::new();
    }
    earlier
}

/// True when the line says something is wrong with a machine ("the kettle is leaking").
pub fn _machine_turn(line: &str) -> bool {
    let line = line.to_lowercase();
    if !TROUBLE_NOW.is_match(&line) {
        return false;
    }
    !machine_named(&line).is_empty() || MACHINE_WORD.is_match(&line)
}

/// The repair rule for the machine the line before named, when this line asks about it ("" when
/// the rule does not apply).
pub fn the_machine_decision(memory: &Memory, earlier: &str, line: &str) -> String {
    if !_machine_turn(earlier) {
        return String::new();
    }
    let low_line = line.to_lowercase();
    if WHO_TO_CALL.is_match(&low_line) {
        if !_trade_for(&low_line).is_empty() {
            return String::new();
        }
        let heard = earlier.to_lowercase();
        if matches!(_named(&heard, &WHITE_GOODS), Some("boiler") | Some("gas boiler")) {
            return "A Gas Safe engineer, not a general plumber: gas work is theirs by law. Ask \
                    for a fixed call-out price and say what the boiler is doing, and check the \
                    quote against half the price of a new one before you agree to a repair."
                .into();
        }
        let named = machine_named(&heard);
        let maker = if named.is_empty() { "the machine".to_string() } else { format!("the {named}") };
        return format!(
            "Look for a repair engineer who covers {maker} - the maker's own service \
             while it is under warranty, otherwise a local white-goods engineer with good \
             reviews. Ask for a fixed call-out price before they come, and weigh the quote \
             against half the price of a new one: past that, replace it."
        );
    }
    let reading = RepairReading {
        text: low_line.clone(),
        clean: text::strip(&format!("{earlier} {line}")).to_lowercase(),
        earlier: text::strip(earlier).to_lowercase(),
        fault: true,
        calling: CALLING_SOMEONE.is_match(&low_line),
    };
    _repair_judgement_of(memory, &reading)
}

/// The skill question ("can i fix it myself") answered for the machine the conversation is about.
pub fn _diy_decision(said: &str, memory: Option<&Memory>) -> String {
    let said = said.to_lowercase();
    let mut thing = machine_named(&said);
    if thing.is_empty() {
        thing = _named(&said, &SMALL_APPLIANCES).or_else(|| _named(&said, &GARDEN_MACHINES)).unwrap_or("").to_string();
    }
    if thing.is_empty() {
        let (before, last) = match memory {
            Some(m) => (machine_before(m), m.turns.last().map(|t| t.user.clone()).unwrap_or_default()),
            None => (String::new(), String::new()),
        };
        thing = machine_named(&before);
        if thing.is_empty() {
            thing = machine_named(&last);
        }
    }
    if thing == "boiler" {
        return "Partly: the pressure gauge (1 to 1.5 bar), the thermostat and the reset button \
                are yours, and anything inside the case is the engineer's, because a gas boiler \
                is Gas Safe work by law. If it locks out twice, or you smell gas, stop and call a \
                Gas Safe engineer."
            .into();
    }
    if let Some(parts) = lookup(&DIY_PARTS, &thing) {
        let article = if starts_with_vowel(&thing) { "an " } else { "a " };
        return format!(
            "For {article}{thing}, {parts}. Take the model number and the part \
             number off the manual before you order anything, and photograph how it came \
             apart."
        );
    }
    if !thing.is_empty() {
        let article = if starts_with_vowel(&thing) { "an " } else { "a " };
        return format!(
            "On {article}{thing}, the parts you can see - a filter, a seal, a hose, a \
             belt, a plug fuse - are swaps with the water off and the plug out, and the \
             motor, the drum and the wiring are an engineer's job."
        );
    }
    "Do the parts you can see: a filter, a door seal, an inlet hose, a belt or a plug fuse \
     is a swap with the water off and the plug out, and the manual gives the part number. \
     Leave the motor, the drum, the wiring and anything with gas in it to an engineer."
        .into()
}

/// The reading `_repair_judgement` works from: the line, the reading's `clean`, and what
/// `the_machine_decision` hands over (the line before, a fault already said, a call-out ask).
#[derive(Clone, Debug, Default)]
pub struct RepairReading {
    pub text: String,
    pub clean: String,
    pub earlier: String,
    pub fault: bool,
    pub calling: bool,
}

/// The money rule for a broken appliance, worded for the machine the line names ("" when the
/// line is not that question).
pub fn _repair_judgement(memory: &Memory, understood: &Understood) -> String {
    let reading = RepairReading {
        text: understood.text.clone(),
        clean: understood.clean.clone(),
        ..Default::default()
    };
    _repair_judgement_of(memory, &reading)
}

/// `_repair_judgement` on a reading built by hand (see [`RepairReading`]).
pub fn _repair_judgement_of(memory: &Memory, reading: &RepairReading) -> String {
    let line = sh::or(&reading.text, &reading.clean).to_lowercase();
    let choosing = CHOOSING.is_match(&line);
    let need_new = NEED_A_NEW.is_match(&line);
    let asking = DECIDING_NOW.is_match(&line)
        || SELF_JOB_NOW.is_match(&line)
        || need_new
        || (reading.calling && CALLING_SOMEONE.is_match(&line));
    let shopping = SHOPPING_NOW.is_match(&line) && !CALLING_SOMEONE.is_match(&line);
    if !asking || (shopping && !choosing && !FAULT_NOW.is_match(&line)) {
        return String::new();
    }
    let earlier = text::strip(&reading.earlier).to_lowercase();
    if !(OWNED_NOW.is_match(&line) || need_new)
        && !(FAULT_NOW.is_match(&line) || (!earlier.is_empty() && FAULT_NOW.is_match(&earlier)))
    {
        return String::new();
    }
    let mut said = reading.clean.to_lowercase();
    let skill = SELF_JOB_NOW.is_match(&line) && !WORTH_NOW.is_match(&line);
    if !earlier.is_empty() {
        said = format!("{earlier} {said}");
    }
    if !(FAULT_NOW.is_match(&line)
        || FAULT_NOW.is_match(&said)
        || KEEPING_NOW.is_match(&line)
        || reading.fault
        || skill
        || need_new)
    {
        return String::new();
    }
    if let Some(small) = _named(&line, &SMALL_APPLIANCES).or_else(|| _named(&said, &SMALL_APPLIANCES)) {
        return _small_appliance(small, memory);
    }
    if let Some(garden) = _named(&line, &GARDEN_MACHINES).or_else(|| _named(&said, &GARDEN_MACHINES)) {
        return _garden_machine(garden, memory);
    }
    if skill {
        return _diy_decision(&said, Some(memory));
    }
    let thing = _named(&said, &WHITE_GOODS);
    let name = thing.unwrap_or("machine");
    let article = if starts_with_vowel(name) { "an" } else { "a" };
    if thing == Some("boiler") {
        return "A gas boiler is Gas Safe work by law: the repair itself is an engineer's job, not \
                a DIY one, so the question is what to spend on it. Repair it while it is under \
                about eight years old and the quote is less than half a new one; past that, or \
                with a failed heat exchanger, put the money towards a new boiler, which will also \
                be cheaper to run."
            .into();
    }
    let age = _age_in(&line).filter(|a| *a != 0).or_else(|| _age_in(&said));
    if let Some(age) = age.filter(|a| *a != 0 && *a >= 8) {
        let years = _words_of_age(age);
        return dup::pick(
            &[
                format!(
                    "At {years} the {name} is at the end of its life, so replace \
                     it rather than repair it: a quote for a seal or a pump on a machine that age is money \
                     towards a new one, and the next fault is already on its way."
                ),
                format!(
                    "Replace the {name}: at {years} old the repair is throwing \
                     money after the next breakdown. A new one comes with a guarantee, and the running \
                     costs of a modern {name} are lower too."
                ),
            ],
            memory,
            "",
        );
    }
    dup::pick(
        &[
            format!(
                "{} {name} is worth repairing while the quote is under about \
                 half the price of a new one and it is under six or seven years old: a seal, a pump, a \
                 belt or a heating element is usually money well spent, while a failed drum bearing or \
                 control board on an old one is not. Ask how long the parts are guaranteed before you \
                 agree, and take a quote near half the price of new as the signal to replace it: an \
                 engineer who wants £80 before touching it is already half a new machine.",
                text::capitalize(article)
            ),
            format!(
                "Repair the {name} if it is under about six years old and the quote is less than \
                 half a new one; replace it if the repair costs more than that or it is past eight years. \
                 The wear and tear repairs (seals, belts, pumps, elements) are worth doing on a good \
                 machine, and a control board or a drum on an old one is not. Get the quote in writing \
                 with a guarantee on the parts, because that is what separates a repair from throwing \
                 money at it."
            ),
            format!(
                "The rule of thumb is half and six: repair the {name} while the quote is under \
                 half the price of a new one and it is under six years old, and replace it after eight \
                 years or when the repair passes that half. A seal, a pump or a heating element on a good \
                 machine is money well spent; a control board or a drum bearing on an old one is not. \
                 Ask how long the parts are guaranteed before you say yes."
            ),
        ],
        memory,
        "",
    )
}

// ---------------------------------------------------------------------------------------------
// How long things last.

/// A LIFESPAN row: (names, how it is called in a sentence, the span, the detail).
pub struct LifeRow {
    pub names: Vec<String>,
    pub called: String,
    pub span: String,
    pub detail: String,
}

static LIFESPAN: LazyLock<Vec<LifeRow>> = LazyLock::new(|| {
    g("LIFESPAN")
        .items()
        .iter()
        .map(|r| {
            let x = r.items();
            LifeRow {
                names: x[0].strs(),
                called: x[1].str().to_string(),
                span: x[2].str().to_string(),
                detail: x[3].str().to_string(),
            }
        })
        .collect()
});
static LIFE_RULE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_dict(g("LIFE_RULE")));
static _FIRST_PERSON: LazyLock<&'static Re> = LazyLock::new(|| rx("_FIRST_PERSON"));
static _HOLDS: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOLDS"));
static _HOLDS_A_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOLDS_A_THING"));
static _A_LIFE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_LIFE"));
static _WHEN_TO_REPLACE: LazyLock<&'static Re> = LazyLock::new(|| rx("_WHEN_TO_REPLACE"));
static BE_AT_THE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bbe\b\s*\??\s*$", 0));

/// `_spans`, but "washing machines" names a washing machine.
pub fn _spans_plural(text: &str, names: &[&str]) -> Option<(usize, usize)> {
    let all: Vec<String> =
        names.iter().flat_map(|base| [base.to_string(), format!("{base}s"), format!("{base}es")]).collect();
    _spans(text, &all)
}

/// The row whose name covers the most of the line (longest name wins, a tie keeps table order).
pub fn _longest_row<'t>(text: &str, table: &'t [(Vec<String>, usize)]) -> Option<usize> {
    let mut best = 0usize;
    let mut found = None;
    for (names, index) in table {
        for name in names {
            let n = name.chars().count();
            if n > best && _spans_plural(text, &[name.as_str()]).is_some() {
                best = n;
                found = Some(*index);
            }
        }
    }
    found
}

/// The names of a table's rows with each row's index, for `_longest_row`.
fn row_names<T>(rows: &[T], names: impl Fn(&T) -> &Vec<String>) -> Vec<(Vec<String>, usize)> {
    rows.iter().enumerate().map(|(i, r)| (names(r).clone(), i)).collect()
}
static LIFESPAN_NAMES: LazyLock<Vec<(Vec<String>, usize)>> = LazyLock::new(|| row_names(&LIFESPAN, |r| &r.names));

/// How long the appliance lasts, from the one table ("" when the line is not that question).
pub fn _lifespan(memory: &Memory, understood: &Understood) -> String {
    // The typed reading has no "earlier" key (only `the_machine_decision`'s own reading did).
    _lifespan_of(memory, sh::text_or_clean(understood), "")
}

/// `_lifespan` on the line and the line before it.
pub fn _lifespan_of(memory: &Memory, line: &str, earlier: &str) -> String {
    if _FIRST_PERSON.is_match(line)
        && !_A_LIFE.is_match(line)
        && !_WHEN_TO_REPLACE.is_match(line)
        && _spans_plural(line, &["white goods", "appliances", "appliance"]).is_none()
    {
        return String::new();
    }
    let names: Vec<&str> = WHITE_AND_SMALL.iter().map(|(w, _)| w.as_str()).collect();
    if _named_as_a_place(&line.to_lowercase(), &names) {
        return String::new();
    }
    if BE_AT_THE_END.is_match(&line.to_lowercase()) && !_A_LIFE.is_match(line) {
        return String::new();
    }
    if _HOLDS.is_match(line) && _HOLDS_A_THING.is_match(line) && !_A_LIFE.is_match(line) {
        return String::new();
    }
    let mut row = _longest_row(line, &LIFESPAN_NAMES);
    if row.is_none() && !earlier.is_empty() {
        row = _longest_row(earlier, &LIFESPAN_NAMES);
    }
    if row.is_none() && _spans_plural(line, &["white goods", "appliances", "appliance", "kitchen gadgets"]).is_none() {
        return String::new();
    }
    let mut rule = "Repair it while a fix is well under half the price of a new one, and replace it once \
                    it needs two repairs in a year."
        .to_string();
    if let Some(i) = row {
        let r = &LIFESPAN[i];
        if let Some(own) = lookup(&LIFE_RULE, &r.called) {
            rule = own.to_string();
        }
        return dup::pick(
            &[
                format!("{} lasts about {} if it is looked after: {}. ", text::capitalize(&r.called), r.span, r.detail)
                    + &rule,
                format!("Expect {} to last about {} - {}. ", r.called, r.span, r.detail) + &rule,
            ],
            memory,
            "",
        );
    }
    "Washing machines and dishwashers last about eight to ten years, a fridge freezer, \
     oven or tumble dryer ten to fifteen, and a kettle or a toaster three to five. "
        .to_string()
        + &rule
}

// ---------------------------------------------------------------------------------------------
// What things cost to run.

/// A RUNNING row: (names, pence, label, detail).
pub struct RunRow {
    pub names: Vec<String>,
    pub pence: f64,
    pub label: String,
    pub detail: String,
}

static RUNNING: LazyLock<Vec<RunRow>> = LazyLock::new(|| {
    g("RUNNING")
        .items()
        .iter()
        .map(|r| {
            let x = r.items();
            RunRow {
                names: x[0].strs(),
                pence: x[1].as_f64().unwrap_or(0.0),
                label: x[2].str().to_string(),
                detail: x[3].str().to_string(),
            }
        })
        .collect()
});
static PRICE_OF: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_dict(g("PRICE_OF")));
static RUNNING_TAIL: LazyLock<String> = LazyLock::new(|| string("RUNNING_TAIL"));
static COOKING: LazyLock<Vec<String>> = LazyLock::new(|| strs("COOKING"));

/// (start, end) of the longest name in the line, or None.
pub fn _spans(text: &str, names: &[String]) -> Option<(usize, usize)> {
    // (start, end, length in characters); the match is the name itself, so its length is the
    // name's
    let mut best: Option<(usize, usize, usize)> = None;
    for name in names {
        let pattern = format!(r"(?<![a-z]){}(?![a-z])", sh::escape(name));
        if let Some(m) = sh::re_cached(&pattern, 0).search(text) {
            let len = text::char_index(text, m.end()) - text::char_index(text, m.start());
            if best.is_none_or(|(_, _, n)| len > n) {
                best = Some((m.start(), m.end(), len));
            }
        }
    }
    best.map(|(s, e, _)| (s, e))
}

/// The RUNNING rows the line names, in the order it names them (indices into RUNNING).
pub fn running_costs(text: &str) -> Vec<usize> {
    let mut found: Vec<((usize, usize), usize)> = Vec::new();
    for (i, row) in RUNNING.iter().enumerate() {
        if let Some(span) = _spans(text, &row.names) {
            found.push((span, i));
        }
    }
    let mut kept: Vec<((usize, usize), usize)> = found
        .iter()
        .filter(|item| {
            !found.iter().any(|other| other.0 .0 <= item.0 .0 && item.0 .1 <= other.0 .1 && other.0 != item.0)
        })
        .cloned()
        .collect();
    kept.sort_by_key(|item| item.0 .0);
    kept.into_iter().map(|(_, i)| i).collect()
}

static _GOING_TO_THE_LOO: LazyLock<&'static Re> = LazyLock::new(|| rx("_GOING_TO_THE_LOO"));
static _IN_AND_OUT_OF_THE_LOO: LazyLock<&'static Re> = LazyLock::new(|| rx("_IN_AND_OUT_OF_THE_LOO"));
static _EVERY_FEW_MINUTES: LazyLock<&'static Re> = LazyLock::new(|| rx("_EVERY_FEW_MINUTES"));

/// False when the line is about the person going to the loo, not about the cistern.
pub fn _going_to_the_loo(understood: &Understood) -> bool {
    let clean = sh::clean_or_text(understood);
    !(_GOING_TO_THE_LOO.is_match(clean) || _IN_AND_OUT_OF_THE_LOO.is_match(clean) || _EVERY_FEW_MINUTES.is_match(clean))
}

static _A_COMPARISON_SAID: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_COMPARISON_SAID"));
static _WHAT_IT_IS_FOR_SAID: LazyLock<&'static Re> = LazyLock::new(|| rx("_WHAT_IT_IS_FOR_SAID"));
static _THE_PRICE_OF: LazyLock<&'static Re> = LazyLock::new(|| rx("_THE_PRICE_OF"));

/// True when the line says the thing the band is for, every word of its name.
pub fn _names_the_thing(clean: &str, label: &str) -> bool {
    let low = label.to_lowercase();
    let words: Vec<&str> = text::split_ref(&low).into_iter().filter(|w| !matches!(*w, "a" | "an" | "the")).collect();
    if words.is_empty() {
        return false;
    }
    let text = format!(" {} ", clean.to_lowercase());
    words.iter().all(|w| sh::re_cached(&format!(r"(?<![a-z]){}", sh::escape(w)), 0).is_match(&text))
}

/// True when the line is the price of the thing, not the price of running it.
pub fn _asks_the_price(text: &str) -> bool {
    let clean = format!(" {} ", text.to_lowercase());
    if !_THE_PRICE_OF.is_match(&clean) {
        return false;
    }
    if _A_COMPARISON_SAID.is_match(&clean) {
        return false;
    }
    !_WHAT_IT_IS_FOR_SAID.is_match(&clean)
}

/// Compare the running costs of the things the line names, or give one thing its figure.
pub fn _running(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.as_str();
    let rows = running_costs(clean);
    let first = &RUNNING[*rows.first()?];
    if rows.len() == 1 {
        if let Some(band) = lookup(&PRICE_OF, &first.label) {
            if _asks_the_price(clean) && _names_the_thing(clean, &first.label) {
                return Some(format!("{band} To run it, it is {}.", first.detail));
            }
        }
        return Some(format!("{} is {}.", text::capitalize(&first.label), first.detail));
    }
    let second = &RUNNING[rows[1]];
    let (cheaper, dearer) = if first.pence <= second.pence { (first, second) } else { (second, first) };
    let tail = if COOKING.contains(&cheaper.label) && COOKING.contains(&dearer.label) {
        format!(" {}", *RUNNING_TAIL)
    } else {
        String::new()
    };
    Some(format!(
        "{} is the cheaper one, {}; {} is {}.{tail}",
        text::capitalize(&cheaper.label),
        cheaper.detail,
        dearer.label,
        dearer.detail
    ))
}

// ---------------------------------------------------------------------------------------------
// Noises.

/// A NOISE_TABLE record.
pub struct NoiseRecord {
    pub words: Vec<String>,
    pub normal: String,
    /// `record.get("first", ())`: sounds this machine reads before the shared table.
    pub first: Vec<(String, Vec<String>)>,
    pub kinds: Vec<(String, String)>,
    /// None where the record has no "urgent" (Python raises a KeyError there).
    pub urgent: Option<String>,
}

fn kind_stems(v: &Value) -> Vec<(String, Vec<String>)> {
    v.items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].strs())).collect()
}

static NOISE_KINDS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| kind_stems(g("NOISE_KINDS")));
static NOISE_TABLE: LazyLock<Vec<NoiseRecord>> = LazyLock::new(|| {
    g("NOISE_TABLE")
        .items()
        .iter()
        .map(|r| NoiseRecord {
            words: r.get("words").map(|v| v.strs()).unwrap_or_default(),
            normal: r.get("normal").map(|v| v.str().to_string()).unwrap_or_default(),
            first: r.get("first").map(kind_stems).unwrap_or_default(),
            kinds: r.get("kinds").map(str_dict).unwrap_or_default(),
            urgent: r.get("urgent").map(|v| v.str().to_string()),
        })
        .collect()
});
static _WASHER_SOUND_ANSWERS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_dict(g("_WASHER_SOUND_ANSWERS")));
static _WASHER_SOUND_STEMS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| kind_stems(g("_WASHER_SOUND_STEMS")));
static _A_BANG_IS_A_FAULT: LazyLock<String> = LazyLock::new(|| string("_A_BANG_IS_A_FAULT"));

/// `re.search(r"\b" + word + r"\b", said)`: the word said as a whole word.
pub fn _says(said: &str, word: &str) -> bool {
    word_re(word, r"\b").is_match(said)
}

/// `re.search(r"\b" + stem, said)` with the stem as written (not escaped).
fn stem_in(said: &str, stem: &str) -> bool {
    sh::re_cached(&format!(r"\b{stem}"), 0).is_match(said)
}

/// The kind of sound the line names ("grind", "squeak", ... "loud"), or "".
pub fn _noise_kind(said: &str, record: Option<&NoiseRecord>) -> String {
    if let Some(record) = record {
        for (kind, stems) in &record.first {
            if stems.iter().any(|stem| stem_in(said, stem)) {
                return kind.clone();
            }
        }
    }
    for (kind, stems) in NOISE_KINDS.iter() {
        if stems.iter().any(|stem| stem_in(said, stem)) {
            return kind.clone();
        }
    }
    String::new()
}

/// Which fault a washing machine's sound names, or the grind when it names none of them.
pub fn _washer_sound_fault(_memory: &Memory, understood: &Understood) -> String {
    let said = understood.clean.to_lowercase();
    for (kind, stems) in _WASHER_SOUND_STEMS.iter() {
        if stems.iter().any(|stem| stem_in(&said, stem)) {
            return lookup(&_WASHER_SOUND_ANSWERS, kind).unwrap_or("").to_string();
        }
    }
    lookup(&_WASHER_SOUND_ANSWERS, "grind").unwrap_or("").to_string()
}

static WASHER_OR_WASHING: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(washer|washing)\b", 0));

/// True when the machine whose drum is making the sound is a washing machine.
pub fn _a_dryer_is_not_a_washer(understood: &Understood) -> bool {
    let said = format!(" {} ", understood.clean.to_lowercase());
    if WASHER_OR_WASHING.is_match(&said) {
        return true;
    }
    !["dryer", "tumble", "dishwasher"].iter().any(|w| said.contains(w))
}

/// What the noise means for the appliance the line names (None when it names none).
pub fn _appliance_noise(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = understood.clean.to_lowercase();
    let record = NOISE_TABLE.iter().find(|r| r.words.iter().any(|w| _says(&said, w)))?;
    let kind = _noise_kind(&said, Some(record));
    let mut advice: Option<String> =
        if kind.is_empty() { None } else { lookup(&record.kinds, &kind).map(str::to_string) };
    if advice.as_deref().is_none_or(str::is_empty) && kind == "knock" {
        advice = Some(_A_BANG_IS_A_FAULT.clone());
    }
    let advice = advice.filter(|a| !a.is_empty());
    let mut parts: Vec<String> = match &advice {
        Some(a) if kind != "loud" => vec![a.clone(), record.normal.clone()],
        _ => {
            let mut p = vec![record.normal.clone()];
            p.extend(advice.clone());
            p
        }
    };
    parts.push(record.urgent.clone()?);
    Some(parts.join(" "))
}

// ---------------------------------------------------------------------------------------------
// Keys and locks.

static _ELSEWHERE_KEY: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ELSEWHERE_KEY"));
static _NAMES_A_LOCK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NAMES_A_LOCK"));

/// A Python value as `str()` gives it, for the strings and numbers memory holds.
fn value_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::None => "None".into(),
        _ => String::new(),
    }
}

/// Everything the last few turns and the running topic said, lower case, as one string.
pub fn _recent_said(memory: &Memory) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(topic @ Value::Dict(_)) = memory.extra.get("topic") {
        parts.push(topic.get("text").filter(|t| t.truthy()).map(value_str).unwrap_or_default());
        if let Some(words) = topic.get("words") {
            parts.extend(words.items().iter().map(value_str));
        }
    }
    let start = memory.turns.len().saturating_sub(4);
    for one in &memory.turns[start..] {
        parts.push(one.user.clone());
    }
    parts.join(" ").to_lowercase()
}

static LOCK_STIFF_REPLY: LazyLock<String> = LazyLock::new(|| string("LOCK_STIFF_REPLY"));
static CAR_KEY_LOST_RE: LazyLock<&'static Re> = LazyLock::new(|| rx("CAR_KEY_LOST_RE"));
static KEYS_AGAIN_RE: LazyLock<&'static Re> = LazyLock::new(|| rx("KEYS_AGAIN_RE"));
static KEYS_KEPT_SOMEWHERE_RE: LazyLock<&'static Re> = LazyLock::new(|| rx("KEYS_KEPT_SOMEWHERE_RE"));

/// The keys answer for the thing they open: a house barrel or a car's coding.
pub fn _lost_keys(_memory: &Memory, understood: &Understood) -> String {
    let text = sh::clean_or_text(understood);
    let car = CAR_KEY_LOST_RE.is_match(text);
    if KEYS_AGAIN_RE.is_match(text) && !KEYS_KEPT_SOMEWHERE_RE.is_match(text) {
        return string(if car { "CAR_KEYS_AGAIN_REPLY" } else { "KEYS_AGAIN_REPLY" });
    }
    if car {
        return string("LOST_CAR_KEYS_REPLY");
    }
    string("LOST_HOUSE_KEYS_REPLY")
}

/// The graphite answer, unless the key being talked about belongs to something else ("").
pub fn _door_lock_stiff(memory: &Memory, understood: &Understood) -> String {
    let joined = understood.words.join(" ").to_lowercase();
    let words = text::split_ref(&joined);
    if !_NAMES_A_LOCK.iter().any(|w| words.contains(&w.as_str())) {
        let recent_line = _recent_said(memory);
        let recent = text::split_ref(&recent_line);
        if _ELSEWHERE_KEY.iter().any(|w| recent.contains(&w.as_str())) {
            return String::new();
        }
    }
    LOCK_STIFF_REPLY.clone()
}

// ---------------------------------------------------------------------------------------------
// A machine that has stopped, smokes, leaks or cuts out.

static MACHINE_BURNING: LazyLock<Vec<String>> = LazyLock::new(|| strs("MACHINE_BURNING"));
static MACHINE_LEAKING: LazyLock<Vec<String>> = LazyLock::new(|| strs("MACHINE_LEAKING"));
static LEAK_CHECKS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows(g("LEAK_CHECKS")));
static MACHINE_CHECKS: LazyLock<Vec<(Vec<String>, String, String)>> = LazyLock::new(|| {
    g("MACHINE_CHECKS")
        .items()
        .iter()
        .map(|r| {
            let x = r.items();
            (x[0].strs(), x[1].str().to_string(), x[2].str().to_string())
        })
        .collect()
});
static DEAD_ALREADY_ANSWERED: LazyLock<Vec<String>> = LazyLock::new(|| strs("DEAD_ALREADY_ANSWERED"));
static BURNING_ALREADY_ANSWERED: LazyLock<Vec<String>> = LazyLock::new(|| strs("BURNING_ALREADY_ANSWERED"));
static CUTTING_OUT: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows(g("CUTTING_OUT")));

fn says_any(said: &str, words: &[String]) -> bool {
    words.iter().any(|w| _says(said, w))
}

/// The named machine's own answer for the kind of fault the line describes ("" when another
/// row already answers it).
pub fn _machine_fault(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::or(&understood.clean, &understood.norm_text).to_lowercase();
    let burning = says_any(&said, &MACHINE_BURNING);
    if says_any(&said, &MACHINE_LEAKING) && !burning {
        return LEAK_CHECKS
            .iter()
            .find(|(words, _)| says_any(&said, words))
            .map(|(_, text)| text.clone())
            .unwrap_or_else(|| string("FALLBACK_LEAK"));
    }
    if burning && says_any(&said, &BURNING_ALREADY_ANSWERED) {
        return String::new();
    }
    if !burning && says_any(&said, &DEAD_ALREADY_ANSWERED) {
        return String::new();
    }
    for (words, hot, dead) in MACHINE_CHECKS.iter() {
        if says_any(&said, words) {
            return if burning { hot.clone() } else { dead.clone() };
        }
    }
    string(if burning { "FALLBACK_BURNING" } else { "FALLBACK_DEAD" })
}

/// The sentence for the appliance the line names, then the two checks that fit most of them.
pub fn _appliance_cutting_out(_memory: &Memory, understood: &Understood) -> String {
    let said = understood.clean.to_lowercase();
    let lead = CUTTING_OUT
        .iter()
        .find(|(words, _)| says_any(&said, words))
        .map(|(_, text)| text.clone())
        .unwrap_or_else(|| string("FALLBACK_CUT_OUT"));
    let close = "If it only ever happens on one socket, feel the plug: a warm plug is a loose \
                 connection, so stop using that socket and get an electrician in.";
    format!("{lead} {close}")
}

/// A kettle that will not switch off: the steam switch has failed.
pub fn _kettle_will_not_stop(_memory: &Memory, _understood: &Understood) -> String {
    string("_KETTLE_WILL_NOT_STOP")
}

// ---------------------------------------------------------------------------------------------
// Mould and pipes.

static MOULD_DEVICE: LazyLock<Vec<String>> = LazyLock::new(|| strs("MOULD_DEVICE"));
static MOULD_BODY: LazyLock<Vec<String>> = LazyLock::new(|| strs("MOULD_BODY"));
static MOULD_SURFACE: LazyLock<Vec<String>> = LazyLock::new(|| strs("MOULD_SURFACE"));
static MOULD_SAID: LazyLock<Vec<String>> = LazyLock::new(|| strs("MOULD_SAID"));
static MOULD_IN_THE_BATHROOM: LazyLock<Vec<String>> = LazyLock::new(|| strs("MOULD_IN_THE_BATHROOM"));

/// A colour a body has changed to is the body's question, not a wall's.
pub fn _the_black_is_not_a_body(understood: &Understood) -> bool {
    let said = sh::or(sh::clean_or_text(understood), &understood.norm_text).to_lowercase();
    if !says_any(&said, &MOULD_BODY) {
        return true;
    }
    if says_any(&said, &MOULD_SAID) {
        return true;
    }
    says_any(&said, &MOULD_SURFACE)
}

/// The bathroom cure where the steam is made, the room cure everywhere else ("" for a dead
/// screen or a body's colour).
pub fn _mould(memory: &Memory, understood: &Understood) -> String {
    let said = sh::or(&understood.norm_text, &understood.text);
    let device = says_any(said, &MOULD_DEVICE);
    let surface = says_any(said, &MOULD_SURFACE);
    let mould = says_any(said, &MOULD_SAID);
    if device && !surface && !mould {
        return String::new();
    }
    if device && !surface {
        return string("_MOULD_DEVICE_REPLY");
    }
    if !mould && !surface && says_any(&_recent_said(memory), &MOULD_BODY) {
        return String::new();
    }
    if says_any(said, &MOULD_IN_THE_BATHROOM) {
        return "Wipe the mould off with a mould spray, or a dilute bleach solution wearing gloves, \
                then stop it coming back: run the extractor fan during and for 20 minutes after a \
                shower, and wipe the condensation off the walls. If it returns, the cause is damp, \
                not the paint."
            .into();
    }
    string("_MOULD_ROOM")
}

/// A pipe that has already frozen needs the thaw first; a cold snap needs the prevention.
pub fn _pipes(_memory: &Memory, understood: &Understood) -> String {
    let text = sh::or(&understood.clean, &understood.words.join(" ")).to_lowercase();
    let thaw = "Turn the water off at the stopcock first, then thaw the pipe gently from the tap end \
                with a hot water bottle, warm towels or a hairdryer on low: a frozen pipe is not a \
                burst one yet, and being gentle is what keeps it that way. Never use a blowtorch. If \
                the pipe has already split, keep the water off and call a plumber.";
    let prevent = "Lag the pipes in the loft and under the floor, and in cold snaps leave the heating \
                   on a low setting, 12 to 15C, rather than off: that costs far less than a burst \
                   pipe.";
    if ["frozen", "froze", "thaw", "burst", "split", "no water", "already", "freeze up", "froze up", "solid"]
        .iter()
        .any(|w| text.contains(w))
    {
        return format!("{thaw} {prevent}");
    }
    format!("{prevent} {thaw}")
}

// ---------------------------------------------------------------------------------------------
// The consumer unit and the stopcock.

static UNIT_WHERE: LazyLock<&'static Re> = LazyLock::new(|| rx("UNIT_WHERE"));
static UNIT_RESET: LazyLock<&'static Re> = LazyLock::new(|| rx("UNIT_RESET"));
static UNIT_GONE: LazyLock<&'static Re> = LazyLock::new(|| rx("UNIT_GONE"));
static UNIT_WET: LazyLock<&'static Re> = LazyLock::new(|| rx("UNIT_WET"));

/// Where the consumer unit is, how to reset it, and how to turn the power off at it.
pub fn _consumer_unit(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::or(&understood.clean, &understood.words.join(" ")).to_lowercase();
    if UNIT_WHERE.is_match(&said) {
        return "The consumer unit is a white or grey plastic box with a row of switches and a test \
                button, usually in the hall near the front door, under the stairs, in the garage or \
                next to the electricity meter; an older house may still have a wooden board with \
                fuses instead. Look for the box with the switches on the inside of the front wall, \
                and take a photo of it so you can see which switch was up."
            .into();
    }
    if UNIT_WET.is_match(&said) {
        return "A switch that trips only when it rains is water getting into an outside socket, a light \
                or a cable: leave that circuit off, dry out and reseal the outside fitting, and get an \
                electrician to look at it, because it will keep tripping until that water is kept out. \
                Don't keep resetting it into a fault."
            .into();
    }
    if UNIT_RESET.is_match(&said) {
        return "Push the switch that is down, or the button that has popped out, firmly to off and \
                then back on - a tripped switch only half moves, so it has to go fully off first. If \
                it trips again straight away, unplug everything on that circuit, reset it, then plug \
                things back one at a time to find the culprit; if it trips with nothing plugged in, \
                the wiring is at fault and that needs an electrician."
            .into();
    }
    if UNIT_GONE.is_match(&said) {
        return "Look at the consumer unit first: if every switch is up, the fault is outside the \
                house - check whether a neighbour has power too, then ring 105 (free, 24 hours) or \
                your supplier, and leave a light switch on so you know when it is back. If one switch \
                is down, only that circuit went off, so unplug what is on it before you reset it."
            .into();
    }
    "The big switch at one end of the consumer unit turns off everything in the house, and \
     it is the right thing to do before you work on a socket or a light. Don't pull the \
     fuse out of the meter or cut the seal on it - that is the supplier's, and it is live."
        .into()
}

static SHUTOFF_WHERE: LazyLock<&'static Re> = LazyLock::new(|| rx("SHUTOFF_WHERE"));
static SHUTOFF_ON: LazyLock<&'static Re> = LazyLock::new(|| rx("SHUTOFF_ON"));
static SHUTOFF_FAULT: LazyLock<&'static Re> = LazyLock::new(|| rx("SHUTOFF_FAULT"));

/// Where the stopcock is, how to shut the water off, and what to do when it will not stop.
pub fn _water_off(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::or(&understood.clean, &understood.words.join(" ")).to_lowercase();
    if SHUTOFF_ON.is_match(&said) && !SHUTOFF_WHERE.is_match(&said) {
        return "Open it anticlockwise, a quarter turn at a time, and listen for the water coming back \
                through the pipes. Close the taps you left open first, or the water will run straight \
                out of them, then run the cold tap at the kitchen sink for a minute to clear the air \
                before you use the boiler or the washing machine again."
            .into();
    }
    if SHUTOFF_FAULT.is_match(&said) {
        return "Shut it off at the stopcock first and open the cold taps to take the pressure off; if \
                the water keeps coming, the inside valve has failed, so use the outside one at the \
                boundary, or ring the water company and they can shut it in the road. Water near a \
                socket or a light fitting means don't touch the switches - turn the power off at the \
                consumer unit first."
            .into();
    }
    if SHUTOFF_WHERE.is_match(&said) {
        return "The inside stopcock is usually under the kitchen sink, and sometimes under the stairs, \
                in a downstairs loo or in the airing cupboard: it is a brass tap on the pipe where the \
                mains comes in, and it closes with a quarter turn clockwise. If there is not one \
                indoors, look for a small metal cover just outside the front wall or at the boundary \
                of the property, which needs a stopcock key or a spanner."
            .into();
    }
    "Turn it clockwise until it stops - a quarter turn is usually enough - then open the cold \
     taps downstairs to run the water out of the pipes before you start work, and open the \
     lowest tap if you are draining the system properly. Combi boilers lose their pressure \
     this way, so note the gauge and top it back up afterwards."
        .into()
}

// ---------------------------------------------------------------------------------------------
// Colour.

static COLOUR_PARTNERS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_dict(g("COLOUR_PARTNERS")));
static COLOUR_NEUTRAL: LazyLock<HashSet<String>> = LazyLock::new(|| set("COLOUR_NEUTRAL"));
static COLOUR_WARM: LazyLock<HashSet<String>> = LazyLock::new(|| set("COLOUR_WARM"));
static LETTERS_AND_APOSTROPHES: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// Every colour the line names, in the order it names them, without repeats.
pub fn _colours_in(line: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for word in LETTERS_AND_APOSTROPHES.findall_str(&line.to_lowercase()) {
        if lookup(&COLOUR_PARTNERS, &word).is_some() && !found.contains(&word) {
            found.push(word);
        }
    }
    found
}

/// What to put a named colour with, or the rule when the line names none.
pub fn _colour_advice(_memory: &Memory, understood: &Understood) -> String {
    let line = sh::text_or_clean(understood);
    let colours = _colours_in(line);
    if colours.len() >= 2 {
        let (first, second) = (&colours[0], &colours[1]);
        let both_neutral = COLOUR_NEUTRAL.contains(first) && COLOUR_NEUTRAL.contains(second);
        let mixed = COLOUR_WARM.contains(first) != COLOUR_WARM.contains(second);
        if both_neutral && mixed {
            return format!(
                "{} and {second} work together, and that is the point: one is \
                 warm and one is cool, so the room has depth. Anchor them with a wood, a \
                 metal and texture - a rug, a throw - because two neutrals alone look \
                 unfinished, and put one deeper tone in the cushions.",
                text::title(first)
            );
        }
        if both_neutral {
            return format!(
                "{} and {second} are both quiet, so nothing in the room is doing \
                 the work: add a third colour you actually like - green, mustard or rust - \
                 plus a wood and a texture, or it will look like a show home.",
                text::title(first)
            );
        }
        return format!(
            "They go together if one stays big and the other stays small: let {first} be the \
             walls or the sofa, and keep {second} to cushions, a rug or a picture. Two strong \
             colours at the same size fight, and a cream or white between them settles it."
        );
    }
    if let Some(name) = colours.first() {
        return format!(
            "{} Sample it before you buy with a patch on the wall or a \
             cushion taken home, because {name} changes with the light, and keep the rest of \
             the room quiet so the one colour has somewhere to sit.",
            lookup(&COLOUR_PARTNERS, name).unwrap_or("")
        );
    }
    "Pick the thing you cannot change - the sofa, the floor, the tiles - and pull one colour \
     out of it for the walls in a paler version of itself, then one deeper tone for a rug or \
     cushions and a wood or metal to warm it. Three colours, one of them a neutral, and a \
     sample pot painted on a patch of wall you can see in the morning and at night: that is \
     the whole trick."
        .into()
}

// ---------------------------------------------------------------------------------------------
// Roofs, ceilings, water pressure.

static _OUTBUILDING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_OUTBUILDING"));

/// The steps for a live leak, shorter when the roof is not over anybody's head.
pub fn _roof_leak(_memory: &Memory, understood: &Understood) -> String {
    let line = sh::clean_or_text(understood).to_lowercase();
    if _OUTBUILDING.iter().any(|w| sh::re_cached(&format!(r"\b{w}\b"), 0).is_match(&line)) {
        return "Get a tarpaulin over it and weigh it down rather than standing underneath to sort \
                it: a shed, a garage or a greenhouse roof is not worth a fall. Move anything \
                electric or valuable out of it, photograph the damage for insurance, then get a \
                roofer - a felt or corrugated roof is a same-day patch for one. If the water is \
                running near the house or the consumer unit, treat it as the house's leak and \
                call an electrician before anything else."
            .into();
    }
    "Catch it and cut the power before anything else: a bucket under the drip, and turn off \
     the light circuit for that room at the fuse board if water is near a fitting, because \
     water and ceiling lights are the dangerous pair. Do not go up on a wet roof or into a \
     loft with wet boards - a fall from a roof kills more people than a leak ever does. \
     Absorb what you can, move furniture and photos, and photograph it all with the date: \
     the ceiling may sag, so stay out of the room if the plaster bulges. If \
     you rent, tell the landlord in writing now and keep the message, because a leak they \
     were told about is theirs to fix. Then get a roofer out - most will tarp a live leak \
     the same day - and check your insurance covers the escape of water."
        .into()
}

static A_MARK_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(&string("A_MARK_WORD"), 0));
static THE_WATER_IS_MOVING: LazyLock<Re> = LazyLock::new(|| Re::new(&string("THE_WATER_IS_MOVING"), 0));

/// The live-leak row keeps a mark that is wet, and leaves a dried one to the ceiling patch.
pub fn _a_mark_that_is_live(understood: &Understood) -> bool {
    let line = sh::clean_or_text(understood).to_lowercase();
    if !A_MARK_WORD.is_match(&line) {
        return true;
    }
    THE_WATER_IS_MOVING.is_match(&line)
}

/// Weak water at a shower (the head first) and weak water at a tap.
pub fn _pressure_weak(_memory: &Memory, reading: &Understood) -> String {
    if reading.words.iter().any(|w| matches!(w.as_str(), "shower" | "showers" | "showerhead" | "showering")) {
        return "Scale in the shower head is the first thing to check: unscrew it, pick the grit \
                out of the holes and soak it in vinegar or citric acid overnight, and clean the \
                small filter where the hose joins the outlet. Check the head is not clicked onto \
                an economy or pause stop too. If the other taps are weak as well it is the \
                supply instead: on a combi it is whatever the mains gives, and on a tank it is \
                the pump or the head of water, so look at the boiler gauge (1 to 1.5 bar cold) \
                and the stopcock before you buy a new head."
            .into();
    }
    "If it is one tap, it is nearly always scale in the spout: the mesh aerator unscrews by \
     hand, so soak it in vinegar and the flow usually comes back. If it is every tap, \
     check the stopcock under the kitchen sink is fully open, then ask a neighbour - if \
     theirs is weak too it is the mains and you ring the water company. A combi boiler \
     reads its pressure on the gauge under the front panel, 1 to 1.5 bar when cold, and \
     low pressure there means weak hot water everywhere, which you top up with the filling \
     loop. A shower that is only weak when hot has a scaled head or a tired hose."
        .into()
}

// ---------------------------------------------------------------------------------------------
// Smoke, alarms, hobs, irons, small appliances that will not heat.

static _SMOKE_COMPUTER: LazyLock<&'static Re> = LazyLock::new(|| rx("_SMOKE_COMPUTER"));
static _SMOKE_MACHINE: LazyLock<&'static Re> = LazyLock::new(|| rx("_SMOKE_MACHINE"));
static _SMOKE_CRUMBS: LazyLock<&'static Re> = LazyLock::new(|| rx("_SMOKE_CRUMBS"));
static _SMOKE_HEATER: LazyLock<&'static Re> = LazyLock::new(|| rx("_SMOKE_HEATER"));

/// Smoke out of one thing, with the cause that thing actually gives.
pub fn _smoking_appliance(_memory: &Memory, reading: &Understood) -> String {
    let clean = reading.clean.as_str();
    let lead = "Switch it off at the wall and unplug it, and keep the door shut - opening it feeds \
                the smoke and can flare a fire up. Let it cool, open the windows, and take it outside \
                if it is small enough to lift safely. ";
    let end = " Don't use it again until it has been checked, because a smouldering element or a \
               trapped wire is a fire waiting to happen - and if you see flames or the smoke spreads \
               past the appliance, get out and ring 999 or 911.";
    let middle = if _SMOKE_COMPUTER.is_match(clean) {
        "Smoke from a laptop, a phone or a plug is usually a failing battery or a \
         hot joint rather than anything burning inside the case: pull the charger out \
         too, and do not shut it in a cupboard or a bag while it cools."
    } else if _SMOKE_MACHINE.is_match(clean) {
        "On a washer or a dryer it is usually the motor, a belt, or fluff against \
         the heater, and a dryer full of fluff can go up on its own: take the washing out \
         before you look, and clear the filter once it is cold."
    } else if _SMOKE_HEATER.is_match(clean) {
        "On something that heats, a burning smell is often dust or scale on the \
         element after a spell unused, but check the plug and the cable for a hot or \
         blackened end, because that is the part that catches."
    } else if _SMOKE_CRUMBS.is_match(clean) {
        "On a toaster or a microwave it is nearly always crumbs or a burnt lining: \
         once it is cold, tip the crumbs out over the bin, wipe the inside with a damp \
         cloth and leave it open to dry before you use it again."
    } else {
        "On an oven or a grill it is nearly always old grease or a spill burning off: \
         once it is cold, a paste of bicarbonate of soda and water on the inside (not the \
         elements) gets rid of that."
    };
    format!("{lead}{middle}{end}")
}

static _ALARM_STEAM: LazyLock<&'static Re> = LazyLock::new(|| rx("_ALARM_STEAM"));
static _ALARM_NO_CAUSE: LazyLock<&'static Re> = LazyLock::new(|| rx("_ALARM_NO_CAUSE"));

/// An alarm that goes off for something that is not a fire, blamed on the right cause.
pub fn _alarm_nuisance(_memory: &Memory, reading: &Understood) -> String {
    let clean = reading.clean.as_str();
    let steam = _ALARM_STEAM.is_match(clean);
    if _ALARM_NO_CAUSE.is_match(clean) && !steam {
        return "If nothing was cooking and there was no smoke, the usual causes are in the unit \
                and the room: dust or a spider in the sensing chamber, steam from a bathroom or a \
                damp day, and a sensor that is past its tenth year. Vacuum the vents, press the \
                test button, and replace the whole unit if it is old or if it keeps doing it - the \
                sensor wears out before the battery does. Never take the battery out to silence it, \
                because that is how a real fire goes unnoticed."
            .into();
    }
    let cause = if steam {
        "It cannot tell steam from smoke, so a hot shower with the door open will set it \
         off. Keep the door shut with the window open while you shower, or use the fan, \
         and if that is not enough, move the alarm a couple of metres further from the \
         bathroom door - not out of the landing, because that is where a real fire has \
         to wake you."
    } else {
        "It cannot tell toast or frying from a fire. Open a window and put the extractor \
         on before the pan gets hot, and if it still goes off, move the alarm a couple of \
         metres further from the hob - not out of the room, because that is where a real \
         fire has to wake you."
    };
    let quiet = if steam { "nothing is running at all" } else { "nothing is cooking" };
    format!(
        "{cause} Never take the battery out to silence it, because that is how a real fire \
         goes unnoticed. If it goes off when {quiet}, vacuum the vents, press the test button, and replace the whole unit once \
         it is ten years old: the sensor wears out before the battery does."
    )
}

static _HOB_INDUCTION: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOB_INDUCTION"));
static _HOB_GAS: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOB_GAS"));
static _IRON_COLD: LazyLock<&'static Re> = LazyLock::new(|| rx("_IRON_COLD"));
static _IRON_SCALE: LazyLock<&'static Re> = LazyLock::new(|| rx("_IRON_SCALE"));
static _HOB_PERSON: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOB_PERSON"));
static _HOB_THE_PHONE: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOB_THE_PHONE"));

/// False for "my mum wont stop ringing me", true for "the ring takes ages to heat".
pub fn _a_person_not_a_hob(understood: &Understood) -> bool {
    let clean = sh::squash_lower(&understood.clean);
    !(_HOB_PERSON.is_match(&clean) && _HOB_THE_PHONE.is_match(&clean))
}

/// A hob that is slow, or a ring that has stopped, answered for the hob the person has.
pub fn _hob_heat(_memory: &Memory, reading: &Understood) -> String {
    let clean = reading.clean.as_str();
    if _HOB_INDUCTION.is_match(clean) {
        return "Check the pan before the hob: an induction ring only heats a magnetic base, so a \
                fridge magnet should stick to the bottom of the pan. If it does not, the pan is \
                the problem, not the ring - and a pan narrower than the ring, or one lifted a \
                centimetre off it, will not heat either. If the magnet sticks and the ring stays \
                cold, switch it off at the wall for a minute and back on, and if that does not \
                bring it back, the ring or its board has gone and it is a repair."
            .into();
    }
    if _HOB_GAS.is_match(clean) {
        return "Take the cap and the burner head off and check the holes first: they block with \
                spill and grease, and a damp cap after cleaning will not light until it dries. \
                Scrub the holes with a pin or a brush, dry the parts properly and put them back \
                flat. If the igniter clicks but nothing lights, it is the spark jumping to the \
                wrong place or a blocked jet, and a jet is an engineer's job because it has to \
                be the right size for the gas."
            .into();
    }
    "On a ceramic or solid-plate hob a ring that takes ten minutes to glow is the element \
     on its way out, and one ring that never heats at all is either that element or the \
     switch behind the knob: turn the knob and listen for the click, because a switch that \
     clicks but heats nothing points at the element. Both parts are cheap, and neither is \
     a job for anyone who is not happy turning the power off at the fuse box first."
        .into()
}

/// An iron that spits, drips, leaves marks or will not heat, told apart by what it does.
pub fn _iron_fault(_memory: &Memory, reading: &Understood) -> String {
    let clean = reading.clean.as_str();
    if _IRON_SCALE.is_match(clean) {
        return "That is scale, not a fault: fill the tank with a mix of half water and half white \
                vinegar, stand the iron upright and let it steam itself empty over an old towel, \
                then fill with clean water and press the steam burst a few times to rinse it out. \
                Use the self-clean button if it has one. Once it runs clear it will stop marking, \
                and if it keeps marking after that, the soleplate wants a wipe with a cloth and \
                bicarbonate of soda while it is warm but unplugged."
            .into();
    }
    if _IRON_COLD.is_match(clean) {
        return "Check the fuse in the plug first, then the flex where it goes into the iron at \
                the heel: a wire broken there is the commonest reason an iron goes dead, because \
                that is the part that bends every time you use it. Wiggle the flex with it \
                plugged in and see if the light flickers. If the light comes on and the soleplate \
                still stays cold, it is the thermostat or the thermal fuse, and on a cheap iron \
                the part costs more to fit than a new one."
            .into();
    }
    "The iron is not hot enough for the steam setting: water only turns to steam at the \
     top of the range, so below it the pump pushes water at a soleplate that just spits it \
     out. Turn the dial to the steam or max setting, wait for the light to go out, and only \
     then press the steam button. Also check the tank is not over the max line, and empty \
     it after each use, because water left standing in it is what turns to scale."
        .into()
}

/// A microwave, a kettle or a toaster that will not heat, each with its own first check.
pub fn _small_appliance_not_heating(_memory: &Memory, reading: &Understood) -> String {
    let said = reading.clean.to_lowercase();
    if ["microwave", "microwaves", "microwave oven"].iter().any(|w| _says(&said, w)) {
        return "Check the plug fuse and the door first: a microwave that thinks the door is \
                open runs the fan and the light and no microwaves. If it lights up and turns \
                but the food stays cold, the magnetron or its high-voltage fuse has gone, and \
                that is an engineer's job, not a DIY one - the capacitor holds a lethal charge \
                after the plug is out. Most repairs cost more than a new microwave."
            .into();
    }
    if ["kettle", "kettles"].iter().any(|w| _says(&said, w)) {
        return "Check the fuse in the plug and try another socket first, then look at the \
                contacts in the base: the ring of metal there corrodes and scales and stops \
                passing current, which is the usual reason a kettle stops heating rather than \
                dying. If the switch clicks and nothing happens on another socket, the element \
                has gone, and a kettle is cheaper to replace than to repair."
            .into();
    }
    "Check the fuse in the plug and try another socket first, then the timer or the \
     switch: on a toaster the element and the timer are what fail, and neither is worth \
     replacing on a machine that costs what they cost. Look along the element in the \
     dark with it switched on for a gap or a section that never glows."
        .into()
}

/// A dryer that will not heat, said as cold air or as clothes that take three goes.
pub fn _dryer_heat(_memory: &Memory, reading: &Understood) -> String {
    const SLOW: &[&str] = &[
        "ages", "forever", "slow", "slower", "slowly", "long", "longer", "hours", "again", "twice", "ages.",
        "slowly.", "goes", "go", "cycles", "cycle", "damp", "damp.", "wet", "wet.",
    ];
    if reading.words.iter().any(|w| SLOW.contains(&w.as_str())) {
        return "Slow drying is nearly always the air: empty the fluff filter, rinse the condenser \
                if it has one, and check the vent hose is not kinked. A drum stuffed full will not \
                dry either, so try half a load. If the air still comes out cold, it is the fuse on \
                the element - an engineer's job, cheap part."
            .into();
    }
    "Nearly always the air or the setting, not a part: check the dial first, since most \
     machines finish on a cool-down phase that heats nothing, then empty the fluff filter \
     and check the vent hose is not kinked. If the drum turns and the air still comes out \
     cold, it is the thermal fuse on the element - an engineer's job, cheap part."
        .into()
}

static _DIY_EARLIER: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DIY_EARLIER"));

/// The do-it-yourself answer, unless the turn before named a job with an answer of its own.
pub fn _diy(memory: &Memory, _reading: &Understood) -> Option<String> {
    let start = memory.turns.len().saturating_sub(2);
    for turn in &memory.turns[start..] {
        let line = format!(" {} ", text::squash(&turn.user.to_lowercase()));
        if _DIY_EARLIER.iter().any(|w| line.contains(w.as_str())) {
            return None;
        }
    }
    Some(string("_DIY_ANSWER"))
}

// ---------------------------------------------------------------------------------------------
// How long a machine lasts, and how long it takes.

static APPLIANCE_LIFE: LazyLock<Vec<(String, (String, String))>> = LazyLock::new(|| {
    g("APPLIANCE_LIFE")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].str().to_string())))
        .collect()
});
static MACHINE_TAKES: LazyLock<Vec<(String, (String, String))>> = LazyLock::new(|| {
    g("MACHINE_TAKES")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].str().to_string())))
        .collect()
});
static _TAKE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_TAKE_WORDS"));
static A_JOB_LEFT_UNDONE: LazyLock<Vec<String>> = LazyLock::new(|| strs("A_JOB_LEFT_UNDONE"));
static PLACE_SHAPE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\b(?:in|into|inside|out of|from|kept in|stored in|keep in|store in)\s+(?:the|a|an|my|our|your|his|her|their|this|that)\s+([a-z]+(?: [a-z]+)?)\b",
        0,
    )
});
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", 0));

/// The longest name the line holds, so a short alias cannot stand in front of the thing.
pub fn _longest_named<'t>(said: &str, table: &'t [(String, String)]) -> Option<&'t str> {
    let mut best = 0usize;
    let mut name = None;
    for (word, called) in table {
        let n = word.chars().count();
        if n > best && word_re(word, r"s?\b").is_match(said) {
            best = n;
            name = Some(called.as_str());
        }
    }
    name
}

/// True when one of `names` follows "in the"/"out of the"/"from the": the fridge as a place.
pub fn _named_as_a_place(said: &str, names: &[&str]) -> bool {
    for m in PLACE_SHAPE.finditer(said) {
        let after = m.str(1);
        for name in names {
            if after == *name || after.starts_with(&format!("{name} ")) || name.starts_with(&format!("{after} ")) {
                return true;
            }
        }
    }
    false
}

/// The machine whose turn-round time this line asks for, or "" - the food in it is not it.
pub fn _how_long_a_machine_takes(said: &str) -> String {
    let mut best = 0usize;
    let mut found = String::new();
    for (word, called) in WHITE_AND_SMALL.iter() {
        let n = word.chars().count();
        if n <= best {
            continue;
        }
        for m in word_re(word, r"\b").finditer(said) {
            let words = LETTERS.findall_str(&said[m.end()..]);
            let ahead = LETTERS.findall_str(&said[..m.start()]);
            let all: Vec<&String> = words.iter().chain(ahead.iter()).collect();
            if !all.is_empty()
                && all.iter().any(|w| *w == "take" || *w == "takes")
                && all.iter().all(|w| _TAKE_WORDS.contains(*w))
            {
                best = n;
                found = called.clone();
                break;
            }
        }
    }
    if MACHINE_TAKES.iter().any(|(k, _)| *k == found) { found } else { String::new() }
}

/// The fault answer for a machine that had to do the job twice, or left it undone ("" if not).
pub fn _the_job_needed_twice(memory: &Memory, reading: &Understood, said: &str) -> String {
    if !A_JOB_LEFT_UNDONE.iter().any(|p| said.contains(p.as_str())) {
        return String::new();
    }
    if _longest_named(said, &WHITE_AND_SMALL) == Some("tumble dryer") {
        return _dryer_heat(memory, reading);
    }
    String::new()
}

/// How long a machine takes to get cold, heat up or run a load.
pub fn _machine_takes(memory: &Memory, understood: &Understood) -> String {
    let said = sh::or(&understood.clean, &understood.plain).to_lowercase();
    let thing = _how_long_a_machine_takes(&said);
    if thing.is_empty() {
        return _the_job_needed_twice(memory, understood, &said);
    }
    let (takes, tip) = &MACHINE_TAKES.iter().find(|(k, _)| *k == thing).unwrap().1;
    let article = if starts_with_vowel(&thing) { "An" } else { "A" };
    format!("{article} {thing} takes {takes}. {}.", text::upper_first(tip))
}

/// The years the named machine is good for, and the habit that decides which end of them.
pub fn _appliance_life(memory: &Memory, understood: &Understood) -> String {
    let said = sh::or(&understood.clean, &understood.plain).to_lowercase();
    let mut names: Vec<&str> = WHITE_AND_SMALL.iter().map(|(w, _)| w.as_str()).collect();
    names.push("mattress");
    if _named_as_a_place(&said, &names) {
        return String::new();
    }
    if _FIRST_PERSON.is_match(&said)
        && !_A_LIFE.is_match(&said)
        && _spans_plural(&said, &["white goods", "appliances", "appliance"]).is_none()
    {
        return String::new();
    }
    if _HOLDS.is_match(&said) && _HOLDS_A_THING.is_match(&said) && !_A_LIFE.is_match(&said) {
        return String::new();
    }
    if _longest_row(&said, &LIFESPAN_NAMES).is_some() {
        return _lifespan(memory, understood);
    }
    let mut table: Vec<(String, String)> = WHITE_AND_SMALL.clone();
    table.push(("mattress".into(), "mattress".into()));
    let Some(thing) = _longest_named(&said, &table) else { return String::new() };
    let Some((_, (years, habit))) = APPLIANCE_LIFE.iter().find(|(k, _)| k == thing) else { return String::new() };
    let article = if starts_with_vowel(thing) { "An" } else { "A" };
    format!(
        "{article} {thing} usually lasts about {years} if it is looked after, and {habit}. Repair it while a fix is \
         well under half the price of a new one; replace it once it needs two repairs in a \
         year, or the parts are no longer made."
    )
}

// ---------------------------------------------------------------------------------------------
// A fitting that has come loose, come off or stopped shutting.

static SCREW_HOLE: LazyLock<String> = LazyLock::new(|| string("SCREW_HOLE"));
static FITTING_TEXT: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_dict(g("FITTING_TEXT")));
static _OVEN_PART: LazyLock<Vec<String>> = LazyLock::new(|| strs("_OVEN_PART"));
static FITTING_STANDDOWN: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    g("FITTING_STANDDOWN").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static FITTING_SHARED_STANDDOWN: LazyLock<Vec<String>> = LazyLock::new(|| strs("FITTING_SHARED_STANDDOWN"));
static _CUPBOARD_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_CUPBOARD_WORDS"));
static _FITTING_ORDER: LazyLock<Vec<String>> = LazyLock::new(|| strs("_FITTING_ORDER"));
static _FITTING_WORDS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    g("_FITTING_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static _STAY_SHUT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_STAY_SHUT"));
static _CROOKED: LazyLock<Vec<String>> = LazyLock::new(|| strs("_CROOKED"));

fn fitting_words(name: &str) -> &'static [String] {
    _FITTING_WORDS.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_slice()).unwrap_or(&[])
}

/// Whether the line has a word in it that a cupboard can be the answer to.
pub fn _names_a_cupboard(said: &str) -> bool {
    let said = said.to_lowercase();
    _CUPBOARD_WORDS.iter().any(|w| word_re(w, r"\b").is_match(&said))
}

/// The part behind the fitting the line names, or None when that job is another entry's.
pub fn _fitting_repair(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = joined(&[&understood.clean, &understood.text, &understood.words.join(" ")]).to_lowercase();
    let mut kind: Option<String> = None;
    for name in _FITTING_ORDER.iter() {
        if fitting_words(name).iter().any(|w| word_re(w, r"\b").is_match(&said)) {
            kind = Some(name.clone());
            break;
        }
    }
    if kind.is_none() {
        let subject = understood.subject.as_deref().unwrap_or("").to_lowercase();
        let named: HashSet<String> = LETTERS.findall_str(&subject).into_iter().collect();
        for name in _FITTING_ORDER.iter() {
            if fitting_words(name).iter().any(|w| named.contains(w)) {
                kind = Some(name.clone());
                break;
            }
        }
    }
    let kind = match kind {
        Some(k) => k,
        None => {
            if _STAY_SHUT.iter().any(|p| said.contains(p.as_str())) {
                "catch".to_string()
            } else if _CROOKED.iter().any(|p| said.contains(p.as_str())) {
                "crooked".to_string()
            } else if _names_a_cupboard(&said) {
                "cupboard".to_string()
            } else {
                return None;
            }
        }
    };
    if TRADE_ASKS.is_match(&said) {
        return None;
    }
    if kind == "oven" && !_OVEN_PART.iter().any(|w| said.contains(w.as_str())) {
        return None;
    }
    if let Some((_, words)) = FITTING_STANDDOWN.iter().find(|(k, _)| *k == kind) {
        if words.iter().any(|w| said.contains(w.as_str())) {
            return None;
        }
    }
    if FITTING_SHARED_STANDDOWN.iter().any(|p| said.contains(p.as_str())) {
        return None;
    }
    // FITTING_TEXT[kind] (a KeyError would make the reply decline)
    let text = lookup(&FITTING_TEXT, &kind)?.to_string();
    if kind == "cupboard" {
        return Some(format!("{text} {}", *SCREW_HOLE));
    }
    Some(text)
}

// ---------------------------------------------------------------------------------------------
// Baths and showers, windows.

static _BATH_EVERYDAY: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BATH_EVERYDAY"));

/// How long a bath or a shower should be, or whether a shower every day is fine.
pub fn _bath_length(_memory: &Memory, understood: &Understood) -> String {
    let text = sh::or(&understood.norm_text, &understood.text);
    if _BATH_EVERYDAY.iter().any(|w| text.contains(w.as_str())) {
        return "Every day is fine for most people: it is long hot showers and harsh soap that \
                leave skin tight and itchy, not the water itself. A five-minute warm shower \
                daily is plenty, and every other day is enough if you wash the face, armpits \
                and groin in between, unless you have been sweating or it is hot. If your skin \
                still feels dry, shorten the shower and turn the heat down before you cut one \
                out."
            .into();
    }
    if text.contains("shower") && !text.contains("bath") {
        return "Five to ten minutes is long enough for a shower, and heat matters more than \
                length: keep it warm rather than hot and pat dry instead of rubbing, because \
                long hot water is what leaves skin tight and itchy. A bath wants 15 to 20 \
                minutes; past that the water is cooling and you can come out light-headed."
            .into();
    }
    "A bath is at its best around 15 to 20 minutes: past that the water is cooling, your \
     skin dries out and you can come out light-headed, and a shower is long enough at \
     five to ten minutes. Heat matters more than length, so keep it warm rather than hot \
     and get out slowly if you feel the room spin."
        .into()
}

static _WINDOW_COST_RE: LazyLock<&'static Re> = LazyLock::new(|| rx("_WINDOW_COST_RE"));
static _WINDOW_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_WINDOW_ASKS"));

/// The make-safe answer for a broken window, with the price first when the line asks the cost
/// (None for a who-to-call line).
pub fn _broken_window(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = sh::squash_lower(&understood.clean);
    if TRADE_ASKS.is_match(&clean) {
        return None;
    }
    let asked_price = _WINDOW_COST_RE.is_match(&clean) || _WINDOW_ASKS.iter().any(|p| clean.contains(p.as_str()));
    let safe = string("_WINDOW_SAFE");
    Some(if asked_price { string("_WINDOW_COST") + &safe } else { safe })
}

// ---------------------------------------------------------------------------------------------
// Who to call.

pub static TRADE_ASKS: LazyLock<&'static Re> = LazyLock::new(|| rx("TRADE_ASKS"));
static TRADES: LazyLock<Vec<(Re, String)>> = LazyLock::new(|| {
    word_rows(g("TRADES"))
        .into_iter()
        .map(|(names, reply)| {
            let alt: Vec<String> = names.iter().map(|n| sh::escape(n)).collect();
            (Re::new(&format!(r"\b(?:{})\b", alt.join("|")), 0), reply)
        })
        .collect()
});
static NOT_A_TRADE: LazyLock<&'static Re> = LazyLock::new(|| rx("NOT_A_TRADE"));

/// The trade for a "who do i call about X" line, or "" when the line does not ask that.
pub fn the_trade_to_call(memory: &Memory, understanding: &Understood) -> String {
    let line = sh::squash_lower(sh::clean_or_text(understanding));
    if !TRADE_ASKS.is_match(&line) {
        return String::new();
    }
    if NOT_A_TRADE.is_match(&line) {
        return String::new();
    }
    let reply = _trade_for(&line);
    if !reply.is_empty() {
        return reply;
    }
    let turns = &memory.turns;
    if turns.is_empty() {
        return String::new();
    }
    let earlier = sh::squash_lower(&turns[turns.len() - 1].user);
    if !earlier.is_empty() && !NOT_A_TRADE.is_match(&earlier) {
        let reply = _trade_for(&earlier);
        if !reply.is_empty() {
            return reply;
        }
    }
    let last_turn = turns.len() as i64 - 1;
    if let Some(Value::List(topics)) = memory.extra.get("topics") {
        if topics.iter().any(|one| matches!(one, Value::Dict(_)) && one.get("turn").and_then(Value::as_int) == Some(last_turn)) {
            return String::new();
        }
    }
    // LEANS_ON: Python reads the trade off the earlier line the reading leans on (a dict with
    // "text"); the typed reading's `leans_on` is never that dict on a single-turn reply.
    if turns.len() >= 2 {
        let prior = sh::squash_lower(&turns[turns.len() - 2].user);
        if !prior.is_empty() && !NOT_A_TRADE.is_match(&prior) {
            return _trade_for(&prior);
        }
    }
    String::new()
}

/// The reply of the first trade row whose words are in the line, or "".
pub fn _trade_for(line: &str) -> String {
    TRADES.iter().find(|(re, _)| re.is_match(line)).map(|(_, reply)| reply.clone()).unwrap_or_default()
}

// ---------------------------------------------------------------------------------------------
// A visit or a purchase, the boiler, the size of a home, heating a room, a fence.

static A_THING_TO_BUY: LazyLock<&'static Re> = LazyLock::new(|| rx("A_THING_TO_BUY"));
static A_THING_TO_BUY_NAMED_FIRST: LazyLock<&'static Re> = LazyLock::new(|| rx("A_THING_TO_BUY_NAMED_FIRST"));
static THE_THING_IN_HAND: LazyLock<&'static Re> = LazyLock::new(|| rx("THE_THING_IN_HAND"));
static A_VISITOR: LazyLock<&'static Re> = LazyLock::new(|| rx("A_VISITOR"));
static A_FAULT_TO_CALL_SOMEONE_ABOUT: LazyLock<&'static Re> = LazyLock::new(|| rx("A_FAULT_TO_CALL_SOMEONE_ABOUT"));
static A_WORK_DONE_TO_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("A_WORK_DONE_TO_THE_THING"));
static BOILER_IS_WRONG: LazyLock<&'static Re> = LazyLock::new(|| rx("BOILER_IS_WRONG"));
static WORTH_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bworth\b", 0));
static BOILER_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:boiler|boilers|combi)\b", 0));

/// `[str(understood.get(key) or "") for key in ("clean", "text", "raw", "rewritten", "plain")]`
/// (the typed reading has no "raw" or "rewritten", so those are "").
fn five_bits(u: &Understood) -> [&str; 5] {
    [&u.clean, &u.text, "", "", &u.plain]
}

/// The callout row answers a visit: "is it worth getting a heat pump" is a purchase.
pub fn _a_visit_not_a_purchase(understood: &Understood) -> bool {
    let text = joined(&five_bits(understood)).to_lowercase();
    if !WORTH_WORD.is_match(&text) {
        return true;
    }
    if THE_THING_IN_HAND.is_match(&text) {
        return true;
    }
    if A_VISITOR.is_match(&text) || A_FAULT_TO_CALL_SOMEONE_ABOUT.is_match(&text) {
        return true;
    }
    if A_WORK_DONE_TO_THE_THING.is_match(&text) {
        return true;
    }
    !(A_THING_TO_BUY.is_match(&text) || A_THING_TO_BUY_NAMED_FIRST.is_match(&text))
}

/// A boiler named as the thing that is fine is not a reason to drop the radiator's answer.
pub fn _the_boiler_is_not_the_fault(understood: &Understood) -> bool {
    let mut bits: Vec<&str> = five_bits(understood).to_vec();
    bits.extend(understood.words.iter().map(String::as_str));
    let text = joined(&bits).to_lowercase();
    if !BOILER_WORD.is_match(&text) {
        return true;
    }
    !BOILER_IS_WRONG.is_match(&text)
}

static HOME_SIZE: LazyLock<&'static Re> = LazyLock::new(|| rx("HOME_SIZE"));
static HOME_SIZE_OPENER: LazyLock<&'static Re> = LazyLock::new(|| rx("HOME_SIZE_OPENER"));
static HOME_SIZE_CARRIED: LazyLock<&'static Re> = LazyLock::new(|| rx("HOME_SIZE_CARRIED"));
static _HEAT_OR_WARM_A_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("_HEAT_OR_WARM_A_THING"));
static _A_BODY_PART_BEING_WARMED: LazyLock<HashSet<String>> = LazyLock::new(|| set("_A_BODY_PART_BEING_WARMED"));

/// True unless the line heats a body part, which is the compress card's question.
pub fn _a_room_being_heated(understood: &Understood) -> bool {
    let line = sh::clean_or_text(understood).to_lowercase();
    !_HEAT_OR_WARM_A_THING.findall_str(&line).iter().any(|thing| _A_BODY_PART_BEING_WARMED.contains(thing))
}

/// A line that tells the bot the home's size is not a line about carrying a bed upstairs.
pub fn _not_a_size_of_a_home(understood: &Understood) -> bool {
    let text = format!(" {} ", understood.clean.to_lowercase());
    let Some(size) = HOME_SIZE.search(&text) else { return true };
    let (start, end) = (size.start(), size.end());
    if HOME_SIZE_CARRIED.is_match(&format!("{} {}", &text[..start], &text[end..])) {
        return true;
    }
    !HOME_SIZE_OPENER.is_match(&text[..start])
}

static _A_FENCE_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_FENCE_WORD"));

/// "where do i start" is a way of asking, not a thing, so the fence row wants a fence word.
pub fn _a_fence_is_named(understood: &Understood) -> bool {
    let mut bits: Vec<&str> = five_bits(understood).to_vec();
    bits.extend(understood.words.iter().map(String::as_str));
    _A_FENCE_WORD.is_match(&joined(&bits))
}

static _WASHER_NEW: LazyLock<Vec<String>> = LazyLock::new(|| strs("_WASHER_NEW"));

/// What a washer costs, led by the kind the line asks about.
pub fn _washer_price(_memory: &Memory, reading: &Understood) -> String {
    let clean = reading.clean.as_str();
    if _WASHER_NEW.iter().any(|w| clean.contains(w.as_str()))
        && !["second hand", "second-hand", "used", "refurbished", "reconditioned", "nearly new", "cheapest way"]
            .iter()
            .any(|w| clean.contains(w))
    {
        return "A new basic washing machine is about 300 to 400 pounds, and 450 to 600 buys a \
                quieter one with a bigger drum and a longer guarantee. Refurbished from a dealer \
                with a six-month guarantee is 180 to 250, and a private sale is 80 to 150 with no \
                guarantee at all. Measure the gap and the door before you order, and check the \
                warranty length: two years is worth having on a machine that will last eight."
            .into();
    }
    "Private sale, 80 to 150 pounds; refurbished from a dealer with a six-month guarantee, \
     180 to 250; a new basic one about 300 to 400. See it run a spin before you pay, \
     listen for a rumble from the bearings, check the door seal for black mould and the \
     drum for rust, and ask why it is going. Add 20 to 40 pounds for delivery and \
     plumbing, and run a hot wash with soda crystals before your first load."
        .into()
}

// ---------------------------------------------------------------------------------------------
// A radiator that will not heat.

static RADIATOR_WHOLE_HOUSE: LazyLock<Vec<String>> = LazyLock::new(|| strs("RADIATOR_WHOLE_HOUSE"));
static RADIATOR_ONE: LazyLock<Vec<String>> = LazyLock::new(|| strs("RADIATOR_ONE"));
static RADIATOR_ALREADY_BLED: LazyLock<Vec<String>> = LazyLock::new(|| strs("RADIATOR_ALREADY_BLED"));
static RADIATOR_UP: LazyLock<Vec<String>> = LazyLock::new(|| strs("RADIATOR_UP"));
static RADIATOR_DOWN: LazyLock<Vec<String>> = LazyLock::new(|| strs("RADIATOR_DOWN"));
static RADIATOR_COLD: LazyLock<Vec<String>> = LazyLock::new(|| strs("RADIATOR_COLD"));
static RADIATOR_WARM: LazyLock<Vec<String>> = LazyLock::new(|| strs("RADIATOR_WARM"));
static RADIATOR_TOO_HOT: LazyLock<Vec<String>> = LazyLock::new(|| strs("RADIATOR_TOO_HOT"));
static RADIATOR_SAYS_COLD: LazyLock<Vec<String>> = LazyLock::new(|| strs("RADIATOR_SAYS_COLD"));

/// Which floor is the cold one: "up", "down", "both" or None.
pub fn _radiator_floor(said: &str) -> Option<&'static str> {
    let flat = said.replace(',', " ").replace('.', " ");
    let words = text::split_ref(&flat);
    if words.is_empty() {
        return None;
    }
    let nearest = |index: usize, marks: &[String]| -> Option<(usize, u8)> {
        let is_mark = |w: &str| marks.iter().any(|m| m == w);
        let after = words.iter().enumerate().filter(|(i, w)| is_mark(w) && *i > index).map(|(i, _)| (i - index, 0u8)).min();
        if after.is_some() {
            return after;
        }
        words.iter().enumerate().filter(|(i, w)| is_mark(w) && *i < index).map(|(i, _)| (index - i, 1u8)).min()
    };
    let (mut cold_up, mut cold_down) = (0, 0);
    for (i, word) in words.iter().enumerate() {
        let up = RADIATOR_UP.iter().any(|w| w == word);
        let down = RADIATOR_DOWN.iter().any(|w| w == word);
        if !up && !down {
            continue;
        }
        let (cold, warm) = (nearest(i, &RADIATOR_COLD), nearest(i, &RADIATOR_WARM));
        if cold.is_none() && warm.is_none() {
            continue;
        }
        let is_cold = cold.is_some() && (warm.is_none() || cold < warm);
        let is_warm = warm.is_some() && (cold.is_none() || warm < cold);
        if up {
            cold_up += is_cold as i32;
            cold_down += is_warm as i32;
        } else {
            cold_down += is_cold as i32;
            cold_up += is_warm as i32;
        }
    }
    match (cold_up > 0, cold_down > 0) {
        (true, true) => Some("both"),
        (true, false) => Some("up"),
        (false, true) => Some("down"),
        _ => None,
    }
}

/// The one cold radiator, the radiator already bled, or the whole house (None for a radiator
/// that is too hot).
pub fn _radiator_not_hot(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = sh::or(&understood.clean, &understood.words.join(" ")).to_lowercase();
    // `_says(said, phrase) or phrase in said` is `phrase in said`: a whole-word match is a
    // substring too.
    let in_said = |list: &[String]| list.iter().any(|p| said.contains(p.as_str()));
    let too_hot = in_said(&RADIATOR_TOO_HOT);
    if too_hot && !in_said(&RADIATOR_SAYS_COLD) {
        return None;
    }
    let one = in_said(&RADIATOR_ONE);
    let bled = in_said(&RADIATOR_ALREADY_BLED);
    let floor = _radiator_floor(&said);
    if let Some(floor) = floor {
        if !one && !bled && floor != "both" {
            if floor == "down" {
                return Some(
                    "Warm upstairs and cold downstairs is the bottom of the system not getting \
                     its water: the pump speed, a zone valve stuck shut or a downstairs circuit \
                     that was never balanced, and the first two are an engineer's job. Check the \
                     room thermostat downstairs is calling for heat and that the downstairs \
                     radiators have their valves open before you ring anyone."
                        .into(),
                );
            }
            return Some(
                "Cold upstairs and warm downstairs is the top of the system not getting its \
                 share: bleed the upstairs radiators first, because air rises and collects at \
                 the highest point, then balance them - open the lockshield at the far end of \
                 the coldest one a quarter turn and turn the downstairs ones down a little, \
                 since water takes the easiest path. A stuck thermostatic valve on one of them \
                 is the other common cause, so check the pins too."
                    .into(),
            );
        }
    }
    let house = !one && !bled && (in_said(&RADIATOR_WHOLE_HOUSE) || ["radiators", "rads"].iter().any(|w| _says(&said, w)));
    if house {
        return Some(
            "If every radiator is cold it is the boiler or the pump, not a radiator: check the \
             pressure gauge reads 1 to 1.5 bar cold and top it up at the filling loop, make sure \
             the room thermostat and the programmer are calling for heat, and press reset once. \
             A pump that runs but moves no heat is an engineer's job."
                .into(),
        );
    }
    if bled {
        return Some(
            "If it stays cold after bleeding, air was not the problem: the thermostatic valve is \
             stuck shut, the bottom of the radiator is sludged up, or the system needs balancing. \
             Take the valve head off and press the pin under it - a pin that will not spring back \
             is stuck - then open both valves and turn the other radiators down so the water is \
             pushed its way."
                .into(),
        );
    }
    Some(
        "One radiator cold while the others are hot is usually its own valve, not the boiler: \
         take the thermostatic head off and press the little pin underneath - a pin that does not \
         spring back is stuck, and a few taps with a screwdriver handle usually free it. If the pin \
         moves, the valve at the other end is closed or the system needs balancing."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// Furniture upstairs, a cook's question, paint brushes.

static UPSTAIRS_ITEMS: LazyLock<Vec<(Vec<String>, String, String)>> = LazyLock::new(|| {
    g("UPSTAIRS_ITEMS")
        .items()
        .iter()
        .map(|r| {
            let x = r.items();
            (x[0].strs(), x[1].str().to_string(), x[2].str().to_string())
        })
        .collect()
});

/// Name the thing being moved and its own first move, then the measuring advice.
pub fn _furniture_upstairs(_memory: &Memory, understood: &Understood) -> String {
    let text = format!(" {} ", understood.clean.to_lowercase());
    let mut kind = "furniture".to_string();
    let mut first = UPSTAIRS_ITEMS.last().map(|r| r.2.clone()).unwrap_or_default();
    for (words, _name, mv) in UPSTAIRS_ITEMS.iter() {
        let said: Vec<&String> = words.iter().filter(|w| text.contains(&format!(" {w}"))).collect();
        if !said.is_empty() {
            // max(said, key=len): the first of the longest
            let mut best = said[0];
            for w in &said[1..] {
                if w.chars().count() > best.chars().count() {
                    best = w;
                }
            }
            kind = best.clone();
            first = mv.clone();
            break;
        }
    }
    format!(
        "Measure first: the stairwell's width and the tightest height under the turn, then \
         the narrowest side of the {kind}. {first}. Two people and a strap, not \
         one: a man with a van is 40 to 80 pounds and cheaper than a surgeon. If the numbers \
         say no, a two-piece or flat-pack version that comes up in boxes is the honest \
         answer."
    )
}

/// False when the line asks what to cook with a food it names, so this row stands down.
pub fn _not_a_cook_ask(understood: &Understood) -> bool {
    !dup::cook_ask(understood)
}

/// Keeping a brush usable: between coats, overnight, or after it has gone hard.
pub fn _paint_brush_care(_memory: &Memory, understood: &Understood) -> String {
    let line = format!(" {} ", sh::clean_or_text(understood).to_lowercase());
    let any = |words: &[&str]| words.iter().any(|w| line.contains(w));
    let oil = any(&[" gloss", " oil based", " oil-based", " white spirit", " varnish", " oil paint", " turps"]);
    if any(&[" gone hard", " went hard", " hard", " stiff", " dried out", " dry out", " save", " saved", " rescue"]) {
        if any(&[" roller", " sleeve", " sleeves"]) {
            return "A sleeve that has gone hard is not worth rescuing: the paint has set through \
                    the pile and no amount of white spirit brings the nap back, so bin it and buy \
                    another - they are two or three pounds. What saves you the next one is not \
                    leaving it to dry: wrap it in cling film between coats, and wash it out the \
                    moment you finish."
                .into();
        }
        return "Soak it before you bin it: water-based paint softens in an hour or two in hot \
                water with a good squeeze of washing-up liquid, worked through the bristles with \
                your fingers and a comb, and gloss wants white spirit or a brush restorer first \
                and the soapy wash after. Once it is soft, comb it straight and dry it bristles \
                down - and if the bristles have set bent, it will paint bent, so keep it for \
                priming and undercoat."
            .into();
    }
    if any(&[
        " losing bristles", " lose bristles", " loses bristles", " lost bristles", " sheds", " shedding",
        " bristles are falling", " bristles falling", " bristles falling out", " bristles coming out",
        " loose bristles", " moulting", " falling apart", " falls apart", " come apart", " comes apart",
        " coming apart",
    ]) {
        return "Bristles that come out in the paint are usually a cheap brush rather than a \
                fault you can fix: the ferrule is crimped onto a bundle that was never glued \
                properly, and no amount of washing holds it. Comb it out dry before you start, \
                keep that brush for undercoat and primer where a stray bristle does not show, \
                and buy a decent one for the topcoat - a bristle left in gloss shows for years."
            .into();
    }
    let roller = any(&[" roller", " sleeve", " sleeves"]);
    let brush = line.contains("brush") || line.contains("bristle");
    if roller && !brush {
        if any(&[
            " between coats", " overnight", " over night", " store", " storing", " keep", " keeping", " tonight",
            " tomorrow", " next day", " wrap", " wrapped", " cling film", " clingfilm", " carrier bag",
            " plastic bag", " lunchtime", " short break",
        ]) && !any(&[" clean", " wash", " rinse"])
        {
            return "Wrap the sleeve, do not wash it: cling film or a carrier bag pressed round \
                    the pile with the air out keeps it soft between coats, and the same goes \
                    overnight with the roller in the fridge rather than the freezer. Standing \
                    it in the tray with a little paint on it works for an hour or so. A \
                    sleeve that is left to dry out is a sleeve you throw away."
                .into();
        }
        if oil {
            return "A sleeve that has had gloss on it wants white spirit worked through it, then \
                    a wash in warm soapy water: oil paint skins over in a sleeve the same way it \
                    does in a brush, and a skinned sleeve sheds fibres on the next wall. Spin it \
                    out, stand it on end to drain, and let it dry standing up so the pile does \
                    not flatten. A sleeve that has gone hard is not worth saving - they are two \
                    or three pounds."
                .into();
        }
        return "Wash a roller sleeve under warm water with a little washing-up liquid, working it \
                with your fingers until the water runs clear, then spin it out - a roller on its \
                frame spun in a bucket or against the side of the sink - and stand it on end to \
                drain so the pile does not flatten. For a short stop, wrap the sleeve in cling \
                film or stand it in the tray with a little paint on it, and never leave it \
                soaking in water: the glue that holds the pile lets go."
            .into();
    }
    if any(&[
        " clean", " wash", " rinse", " after ", " finished", " done with", " out of", " get out", " got out",
        " remove", " removing", " off a brush", " off my brush", " off the brush", " off a paintbrush",
        " off my paintbrush", " out of a brush", " out of my brush", " out of a paintbrush",
        " out of my paintbrush", " out of the bristles", " off a roller", " out of a roller",
        " out of the sleeve", " get it off", " get the paint off",
    ]) {
        if oil {
            return "Gloss and oil-based paint come out with white spirit, not water: work it \
                    through the bristles, wipe them on a rag, then wash the brush in warm soapy \
                    water and comb it straight. Dry it bristles down so water does not sit in \
                    the ferrule, and never leave it standing on its bristles in a jar."
                .into();
        }
        return "Warm water and washing-up liquid, worked right down to the ferrule until the \
                water runs clear, then comb the bristles straight and dry the brush bristles \
                down - never standing on them in a jar, which sets them splayed. Emulsion comes \
                out easily while it is wet and hardly at all once it has dried."
            .into();
    }
    if any(&[" overnight", " over night", " tomorrow", " next day", " fridge", " until the morning"]) {
        return "Wrap it and put it in the fridge: cling film or a carrier bag pressed round the \
                bristles with the air out keeps a brush soft overnight, and a day or two at a \
                push, as long as the bag is sealed so the solvent cannot dry out. Take it out an \
                hour before you paint again, and never put a brush in the freezer - the cold \
                cracks the paint in the bristles."
            .into();
    }
    "Wrap it in cling film or a carrier bag with the air squeezed out, or in a damp cloth, \
     and it will still be soft at the end of the day: that is the whole trick between \
     coats, and a brush left out on the tin dries in about an hour. For an overnight stop, \
     wrap it and put it in the fridge rather than the freezer, and for a roller, wrap the \
     sleeve the same way or stand it in the tray with a little paint on it."
        .into()
}

// ---------------------------------------------------------------------------------------------
// A machine's filter, the ceiling's colour, wallpaper.

static FILTER_TURNS: LazyLock<usize> = LazyLock::new(|| g("FILTER_TURNS").as_int().unwrap_or(3) as usize);

/// `the_filter_of(words, text)`'s reply: the filter text when the line, or the topics of the
/// last turns, name the machine ("" otherwise).
#[allow(non_snake_case)]
pub fn the_filter_of__reply(patterns: &[&Re], text: &str, memory: &Memory, reading: &Understood) -> String {
    let line = sh::text_or_clean(reading).to_lowercase();
    if patterns.iter().any(|p| p.is_match(&line)) {
        return text.to_string();
    }
    let heard = dup::topics_context(memory, *FILTER_TURNS).join(" ").to_lowercase();
    if patterns.iter().any(|p| p.is_match(&heard)) {
        return text.to_string();
    }
    String::new()
}

static _CEILING_COLOUR_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_CEILING_COLOUR_WORDS"));
static _CEILING_HOW_TO: LazyLock<Vec<String>> = LazyLock::new(|| strs("_CEILING_HOW_TO"));

/// True for a ceiling question that is about the colour, not about how to paint it.
pub fn _asks_the_ceiling_colour(understood: &Understood) -> bool {
    let line = format!(" {} ", understood.clean.to_lowercase());
    if _CEILING_HOW_TO.iter().any(|h| line.contains(h.as_str())) {
        return false;
    }
    // `" %s " % word in line or " %s" % word in line` is the second test
    _CEILING_COLOUR_WORDS.iter().any(|w| line.contains(&format!(" {w}")))
}

/// True when the line names the paper this job hangs, and not just the hanging.
pub fn _the_paper_is_in_the_line(understood: &Understood) -> bool {
    let line = sh::text_or_clean(understood).to_lowercase();
    ["wallpaper", "wall paper", "lining paper", "paper", "paste"].iter().any(|w| line.contains(w))
}

// ---------------------------------------------------------------------------------------------
// Sounds, boilers, heating.

static A_SOUND_NAMED: LazyLock<HashSet<String>> = LazyLock::new(|| set("A_SOUND_NAMED"));
static _A_SOUND_STEM: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_SOUND_STEM"));

/// True when the line names the sound a thing is making, not what it costs or how it is set.
pub fn a_sound_is_named(understood: &Understood) -> bool {
    if understood.words.iter().any(|w| A_SOUND_NAMED.contains(&w.to_lowercase())) {
        return true;
    }
    let said = sh::squash_lower(sh::clean_or_text(understood));
    _A_SOUND_STEM.iter().any(|s| said.contains(s.as_str()))
}

/// The guard for a row about a cost, a setting or a temperature: a named sound stands it down.
pub fn not_a_sound(understood: &Understood) -> bool {
    !a_sound_is_named(understood)
}

/// A noise in a boiler, with the cause that noise actually has.
pub fn _boiler_noise(_memory: &Memory, understood: &Understood) -> String {
    let line = sh::squash_lower(sh::or(sh::or(&understood.plain, &understood.text), &understood.clean));
    let words: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let said = |names: &[&str]| names.iter().any(|n| line.contains(n));
    let has = |set: &[&str]| set.iter().any(|w| words.contains(*w));
    if said(&["humming", "hummed", "buzzing", "buzzed", "vibrating", "droning"])
        || has(&[
            "hum", "hums", "humming", "hummed", "buzz", "buzzes", "buzzing", "buzzed", "vibrate", "vibrates",
            "vibrating", "vibrated", "drone", "drones", "droning",
        ])
    {
        return "A steady hum is vibration rather than anything burning: the casing or a pipe \
                touching a wall or a floorboard, the pump left on its highest speed, or a fan \
                bearing on the way out. Press a hand on the pipes and the casing while it runs to \
                feel which one is ringing, put a cloth or a bit of foam where metal touches \
                plaster, and turn the pump down a speed if it has a dial. A hum that changes pitch \
                with the firing, or comes with a hot smell or a fault code, is the fan or the \
                pump itself and wants an engineer."
            .into();
    }
    if said(&["whistl", "whine", "whining", "squeal", "squeak", "screech", "hiss", "shriek"])
        || has(&[
            "squeak", "squeaks", "squeaking", "squeaked", "screech", "screeches", "screeching", "screeched",
            "squeal", "squeals", "squealing", "squealed",
        ])
    {
        return "A whistle, a whine or a hiss is a restriction or air rather than the flame: a \
                radiator valve left nearly shut, the boiler's own isolation valve not fully open, \
                low pressure, or scale on the heat exchanger. Check the gauge is between 1 and 1.5 \
                bar cold, open every radiator valve fully, and if the noise only comes with hot \
                water it is usually the diverter valve, which is an engineer's job. Do not run it \
                with the pressure below 1 bar."
            .into();
    }
    if said(&["gurgl", "gurgling", "glug"]) {
        return "Gurgling is air: bleed every radiator from the top, with the heating off until the \
                water runs steady, then check the pressure is 1 to 1.5 bar and top it up if it has \
                dropped. If it gurgles again within a week, air is getting in somewhere and that \
                is an engineer's visit rather than a bleeding."
            .into();
    }
    if said(&["tap", "tapping", "tick", "ticking", "click"]) {
        return "A tick or a tap is usually pipes expanding as they heat, and it is harmless: slide \
                a bit of felt or plastic where the pipe passes through a joist or a wall, and \
                leave the heating on a steady setting rather than switching it on and off. A \
                ticking that turns into a knock when the pump starts is air, so bleed the \
                radiators as well."
            .into();
    }
    if said(&["grind", "grinding", "scraping"])
        || has(&["grind", "grinds", "grinding", "ground", "scrape", "scrapes", "scraping", "scraped"])
    {
        return "Grinding means stop running it and get an engineer: on a boiler it is the pump or a \
                seized fan rather than air, and a pump that has gone on grinding has often taken \
                the shaft with it. Turn it off at the programmer, leave the heating off, and say \
                what it sounds like and how long ago it started - a grind that stops when the \
                boiler does is the pump, and one that carries on is the fan."
            .into();
    }
    "Banging or rumbling when the heating starts is usually air or a pump set too fast: bleed \
     the radiators, check the pressure is 1 to 1.5 bar, and if it carries on get a heating \
     engineer to look at the pump. A knocking that comes at the end of a burn is limescale \
     on the heat exchanger, and a knock with a hot smell or a fault code is one to switch \
     off and report."
        .into()
}

/// The answer to a heating or a hot water that keeps going cold, by which one was named.
pub fn _heating_goes_cold(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::clean_or_text(understood).to_lowercase();
    if ["hot water", "hot tap", "tap", "bath", "shower", "cylinder", "immersion", "boiler"].iter().any(|w| said.contains(w)) {
        return "Hot water that runs cold after a few minutes is nearly always the flow rate: a \
                combi heats only the water you draw, so a hot tap opened full, or two taps at \
                once, outruns it - run it slower and see whether it holds. A tank with an \
                immersion is the other case: the hot runs out when the cylinder empties, so a \
                longer draw needs the immersion on for longer or the cylinder thermostat turned \
                up. If it goes cold at random, check the boiler pressure is between 1 and 1.5 \
                bar."
            .into();
    }
    "A heating that comes on and then goes cold is usually the room thermostat reaching \
     its set temperature and shutting the boiler off, which is it working, not a fault - \
     turn the thermostat up a degree and see whether the radiators stay hot. If it goes \
     off long before the room is warm, check the boiler pressure is between 1 and 1.5 bar, \
     that the programmer or timer is not cutting out early, and whether the thermostat \
     sits somewhere warm like a hallway by a radiator, which makes it read the wrong room."
        .into()
}

static SAFETY_BOILER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(boiler|combi|back boiler|gas fire|gas boiler|flue)\b", 0));
static SAFETY_ASK: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(safe|safely|unsafe|dangerous|danger|risk|alright to leave|ok to leave|okay to leave)\b", 0)
});

/// A boiler, or a line that asks whether something is safe.
pub fn _a_safety_ask(understood: &Understood) -> bool {
    let text = format!(" {} ", sh::or(&understood.clean, &understood.norm_text).to_lowercase());
    if SAFETY_BOILER.is_match(&text) {
        return true;
    }
    SAFETY_ASK.is_match(&text)
}

static _A_BOILER_TOO: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_BOILER_TOO"));
static _WEIGHING_THE_TWO: LazyLock<&'static Re> = LazyLock::new(|| rx("_WEIGHING_THE_TWO"));

/// The heat pump's own answer, or the pick when the line weighs it against a boiler.
pub fn _heat_pump_answer(_memory: &Memory, reading: &Understood) -> String {
    let words = reading.words.join(" ");
    let said = if words.is_empty() { sh::clean_or_text(reading).to_string() } else { words };
    if _A_BOILER_TOO.is_match(&said) && _WEIGHING_THE_TWO.is_match(&said) {
        return string("_HEAT_PUMP_OR_BOILER");
    }
    string("_HEAT_PUMP_MECHANICS")
}

static _OWN_BOUNDARY: LazyLock<&'static Re> = LazyLock::new(|| rx("_OWN_BOUNDARY"));
static _PUBLIC_WAY: LazyLock<&'static Re> = LazyLock::new(|| rx("_PUBLIC_WAY"));
static _BLOCKED_DRAIN: LazyLock<&'static Re> = LazyLock::new(|| rx("_BLOCKED_DRAIN"));

/// True when the line is a blocked drain on the person's own side of the boundary.
pub fn _a_blocked_drain_of_your_own(understood: &Understood) -> bool {
    let said = sh::squash_lower(&understood.clean);
    _BLOCKED_DRAIN.is_match(&said) && _OWN_BOUNDARY.is_match(&said) && !_PUBLIC_WAY.is_match(&said)
}

static _CAME_DOWN: LazyLock<&'static Re> = LazyLock::new(|| rx("_CAME_DOWN"));
static _CAME_DOWN_ON: LazyLock<&'static Re> = LazyLock::new(|| rx("_CAME_DOWN_ON"));
static _STORM_LANDED: LazyLock<&'static Re> = LazyLock::new(|| rx("_STORM_LANDED"));
static _STORM_TAKEN_OFF: LazyLock<&'static Re> = LazyLock::new(|| rx("_STORM_TAKEN_OFF"));
static _STORM_FELL_IN: LazyLock<&'static Re> = LazyLock::new(|| rx("_STORM_FELL_IN"));
static _STORM_WEATHER: LazyLock<&'static Re> = LazyLock::new(|| rx("_STORM_WEATHER"));

/// True when the line says something came down on the person's roof, fence, shed or car.
pub fn _storm_damage(understood: &Understood) -> bool {
    let line = sh::or(&understood.clean, &understood.plain);
    if !(_CAME_DOWN.is_match(line) && _CAME_DOWN_ON.is_match(line)) {
        return false;
    }
    _STORM_WEATHER.is_match(line)
        || _STORM_LANDED.is_match(line)
        || _STORM_TAKEN_OFF.is_match(line)
        || _STORM_FELL_IN.is_match(line)
}

/// The sewage row stands down for a blocked drain inside the person's own boundary.
pub fn lambda_4069(understood: &Understood) -> bool {
    !_a_blocked_drain_of_your_own(understood)
}

// ---------------------------------------------------------------------------------------------
// A machine's consumables, and how much water a job takes.

static _A_DISHWASHER: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_DISHWASHER"));
static _A_WASHER: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_WASHER"));
static _ONLY_A_MACHINE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ONLY_A_MACHINE"));
static _A_RESTOCK: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_RESTOCK"));
static _A_WHERE_IT_GOES: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_WHERE_IT_GOES"));
static _CONSUMABLE_WORDS: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| g("_CONSUMABLE_WORDS").items().iter().map(|v| v.strs()).collect());
static SENTENCE_BREAK: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=\.)\s+", 0));

/// True when the line asks where the salt, powder, tablet or rinse aid of a machine goes.
pub fn _a_machine_consumable(understood: &Understood) -> bool {
    let said = sh::or(&understood.plain, &understood.clean);
    if _A_RESTOCK.is_match(said) && !_A_WHERE_IT_GOES.is_match(said) {
        return false;
    }
    if _A_DISHWASHER.is_match(said) || _A_WASHER.is_match(said) {
        return true;
    }
    _ONLY_A_MACHINE.iter().any(|w| said.contains(w.as_str()))
}

/// The card's sentence that names the consumable asked about, then the rest of the card.
pub fn _leading_sentence(text: &str, said: &str) -> String {
    let asked = said.to_lowercase();
    let sentences: Vec<String> = SENTENCE_BREAK
        .split(text)
        .into_iter()
        .map(|p| text::strip(&p).to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if sentences.len() < 2 {
        return text.to_string();
    }
    let (mut best, mut score) = (0usize, 0usize);
    for (index, sentence) in sentences.iter().enumerate() {
        let here = sentence.to_lowercase();
        let hits = _CONSUMABLE_WORDS
            .iter()
            .flatten()
            .filter(|w| asked.contains(w.as_str()) && here.contains(w.as_str()))
            .count();
        if hits > score {
            best = index;
            score = hits;
        }
    }
    if score == 0 {
        return text.to_string();
    }
    let mut out = vec![sentences[best].clone()];
    out.extend(sentences.iter().enumerate().filter(|(i, _)| *i != best).map(|(_, s)| s.clone()));
    out.join(" ")
}

/// The compartment each consumable belongs in, for the machine the line named.
pub fn _where_the_consumable_goes(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::or(&understood.plain, &understood.clean);
    let drawer = string("_WASHER_DRAWER");
    if _A_WASHER.is_match(said) && !_A_DISHWASHER.is_match(said) {
        return _leading_sentence(&drawer, said);
    }
    if _A_DISHWASHER.is_match(said) || said.contains("rinse aid") || said.contains("rinse-aid") {
        return _leading_sentence(&string("_DISHWASHER_COMPARTMENTS"), said);
    }
    _leading_sentence(&drawer, said)
}

static WATER_USE: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows(g("WATER_USE")));
static WATER_USE_NAMES: LazyLock<Vec<(Vec<String>, usize)>> = LazyLock::new(|| row_names(&WATER_USE, |r| &r.0));

/// Litres for the thing that holds or uses the water, read from one table.
pub fn _water_use(_memory: &Memory, understood: &Understood) -> String {
    let line = sh::text_or_clean(understood);
    if let Some(i) = _longest_row(line, &WATER_USE_NAMES) {
        return WATER_USE[i].1.clone();
    }
    "Measure it and multiply: length times width times depth in metres, times 1,000, \
     gives litres, and a cubic metre is 1,000 litres. A full bath is about 80 litres and \
     five minutes in the shower about 50."
        .into()
}

static _A_LENGTH_OF_STAY_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_LENGTH_OF_STAY_ASK"));

/// False when the line is a length of stay in a city the travel table knows.
pub fn _not_a_length_of_stay(understood: &Understood) -> bool {
    let line = sh::the_line(understood);
    if !_A_LENGTH_OF_STAY_ASK.is_match(&line) {
        return true;
    }
    dup::_find_in(&line, &dup::CITY_DAYS).is_none()
}

// ---------------------------------------------------------------------------------------------
// The kit that wears out, the basics that run out.

/// A WEARS_OUT row.
pub struct WearRow {
    pub names: Vec<String>,
    pub called: String,
    pub span: String,
    pub sign: String,
    pub avoid: Vec<String>,
}

static WEARS_OUT: LazyLock<Vec<WearRow>> = LazyLock::new(|| {
    g("WEARS_OUT")
        .items()
        .iter()
        .map(|r| {
            let s = |k: &str| r.get(k).map(|v| v.str().to_string()).unwrap_or_default();
            WearRow {
                names: r.get("names").map(|v| v.strs()).unwrap_or_default(),
                called: s("called"),
                span: s("span"),
                sign: s("sign"),
                avoid: r.get("avoid").map(|v| v.strs()).unwrap_or_default(),
            }
        })
        .collect()
});
static _NOT_A_WEAR: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NOT_A_WEAR"));

/// The interval and the sign, for the thing the line names ("" when it names none).
pub fn _a_wear(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::or(&understood.clean, &understood.plain).to_lowercase();
    let mut best = 0usize;
    let mut row: Option<&WearRow> = None;
    for candidate in WEARS_OUT.iter() {
        for name in &candidate.names {
            let n = name.chars().count();
            if n > best && word_re(name, r"\b").is_match(&said) {
                best = n;
                row = Some(candidate);
            }
        }
    }
    match row {
        None => String::new(),
        Some(r) => format!("Replace {} {}: {}.", r.called, r.span, r.sign),
    }
}

/// The words that stand the whole worn-out table down, from the shared list and each row's own
/// (the entry's `avoid`, built at import time).
pub fn _worn_out_avoid() -> Vec<String> {
    let names: HashSet<&String> = WEARS_OUT.iter().flat_map(|r| r.names.iter()).collect();
    let mut words: Vec<String> = _NOT_A_WEAR.clone();
    for row in WEARS_OUT.iter() {
        for word in &row.avoid {
            if !names.contains(word) && !words.contains(word) {
                words.push(word.clone());
            }
        }
    }
    words
}

static RUN_OUT_ROWS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows(g("RUN_OUT_ROWS")));

/// The row for the household thing the line has run out of, or None when it names none.
pub fn _run_out_basics(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = joined(&[&understood.clean, &understood.text, &understood.words.join(" ")]).to_lowercase();
    let mut best: Option<&String> = None;
    let mut longest: i64 = -1;
    for (names, reply) in RUN_OUT_ROWS.iter() {
        for name in names {
            let n = name.chars().count() as i64;
            if n > longest && word_re(name, r"\b").is_match(&said) {
                best = Some(reply);
                longest = n;
            }
        }
    }
    best.cloned()
}

// ---------------------------------------------------------------------------------------------
// A machine that has broken, and the part that failed.

static _NOT_A_MACHINE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NOT_A_MACHINE"));
static _COLD_STOP: LazyLock<&'static Re> = LazyLock::new(|| rx("_COLD_STOP"));
static _OTHER_FAULT: LazyLock<&'static Re> = LazyLock::new(|| rx("_OTHER_FAULT"));
static _AN_ANIMAL_IN_THE_MACHINE: LazyLock<&'static Re> = LazyLock::new(|| rx("_AN_ANIMAL_IN_THE_MACHINE"));
static _A_SYMPTOM: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_SYMPTOM"));
static _ASKS_HOW: LazyLock<&'static Re> = LazyLock::new(|| rx("_ASKS_HOW"));
static _A_PART_OF_A_PERSON: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_PART_OF_A_PERSON"));
static MACHINE_OR_APPLIANCE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(machine|machines|appliance|appliances)\b", 0));

/// Whether this is any line but "the machine has gone", which is a repair and not a how-to.
pub fn _not_a_machine_that_has_gone(understood: &Understood) -> bool {
    let line = format!("{} {}", understood.text, understood.clean).to_lowercase();
    if _AN_ANIMAL_IN_THE_MACHINE.is_match(&line) {
        return false;
    }
    if _A_PART_OF_A_PERSON.is_match(&line) {
        return false;
    }
    if (_OTHER_FAULT.is_match(&line) || _A_SYMPTOM.is_match(&line)) && !_ASKS_HOW.is_match(&line) {
        return false;
    }
    !dup::the_thing_has_gone(understood)
}

/// The machine named in the line, if this is a fault statement rather than a decision ("" if
/// not).
pub fn _machine_broke(memory: &Memory, understood: &Understood) -> String {
    let line = format!("{} {}", understood.text, understood.clean).to_lowercase();
    if _NOT_A_MACHINE.iter().any(|w| sh::re_cached(&format!(r"\b{w}\b"), 0).is_match(&line)) {
        return String::new();
    }
    if !_OTHER_FAULT.is_match(&line) && !dup::the_thing_has_gone(understood) {
        return String::new();
    }
    let small = _named(&line, &SMALL_APPLIANCES);
    let cold = small.or_else(|| _named(&line, &WHITE_GOODS)).unwrap_or("");
    if (cold.contains("fridge") || cold.contains("freezer")) && _COLD_STOP.is_match(&line) {
        return String::new();
    }
    if cold.is_empty() && !MACHINE_OR_APPLIANCE.is_match(&line) {
        return String::new();
    }
    if let Some(small) = small {
        let article = if starts_with_vowel(small) { "an" } else { "a" };
        return dup::pick(
            &[
                format!(
                    "Check the fuse in the plug and the socket before anything else, because on {article} \
                     {small} that is what has failed nine times out of ten, then move the plug to \
                     another socket to be sure. If it is still dead, replace it: a {small} is not \
                     worth repairing, since the call-out costs more than a new one and the parts are not \
                     made to be swapped. Buy the next from a shop with a two-year guarantee and keep the \
                     receipt."
                ),
                format!(
                    "On {article} {small} the fault is usually the fuse or the socket, so try \
                     both of those first. Past that it is not worth mending: a new {small} costs less \
                     than the call-out, so replace it, buy it from a shop with a two-year guarantee and \
                     keep the receipt - that guarantee is the only cover worth having on a cheap appliance."
                ),
            ],
            memory,
            "",
        );
    }
    let thing = _named(&line, &WHITE_GOODS);
    let label = match thing {
        Some(t) => format!("{t}s"),
        None => "machines".to_string(),
    };
    dup::pick(
        &[
            format!(
                "Check the cheap things before you pay anyone: the fuse in the plug and the socket, the \
                 switch on the wall, the door shut properly, and any filter or drain it has - most {label} \
                 that will not start are one of those. If it has really \
                 failed, go to the shop you bought it from, not the maker: they owe you a repair, \
                 replacement or refund for up to six years in England and Wales (five in Scotland), and a \
                 manufacturer's guarantee is in addition to that. Get a written quote with a guarantee on \
                 the parts, and treat a repair costing more than half a new one as the signal to replace."
            ),
            format!(
                "Start with the plug fuse, the wall switch and the door or any filter: those account for \
                 most {label} that stop dead, and they cost nothing to check. Then ring the retailer \
                 with your receipt, because your claim is against the shop rather than the maker - up to \
                 six years in England and Wales, five in Scotland, for a repair, a replacement or a \
                 refund - and any guarantee you were given sits on top of that right. Before agreeing to \
                 work, ask for the quote in writing and for how long the parts are guaranteed."
            ),
        ],
        memory,
        "",
    )
}

static _A_DRAIN_FAULT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_DRAIN_FAULT"));
static _MACHINE_PARTS: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| g("_MACHINE_PARTS").items().iter().map(|v| v.strs()).collect());
static _DRYER_KINDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DRYER_KINDS"));
static _MACHINE_PART_TEXT: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_dict(g("_MACHINE_PART_TEXT")));
static VACUUM_AND_MACHINE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    [
        ("hoover", "vacuum cleaner"),
        ("vacuum", "vacuum cleaner"),
        ("dyson", "vacuum cleaner"),
        ("henry", "vacuum cleaner"),
        ("machine", "machine"),
        ("machines", "machine"),
        ("appliance", "machine"),
        ("appliances", "machine"),
    ]
    .iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
});
static PUMP_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bpumps?\b", 0));

/// Which part of which machine failed, and what to do about the part (None when no part is
/// named, "" when the part has no machine).
pub fn _machine_part(_memory: &Memory, understood: &Understood) -> Option<String> {
    let lowered: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let line = format!(" {} ", lowered.join(" "));
    let mut part = "";
    for names in _MACHINE_PARTS.iter() {
        if names.iter().any(|n| line.contains(&format!(" {n} ")) || line.contains(&format!(" {n}s "))) {
            part = names[0].as_str();
            break;
        }
    }
    if part.is_empty() {
        return None;
    }
    let machine = _named(&line, &WHITE_GOODS)
        .filter(|m| !m.is_empty())
        .or_else(|| _named(&line, &VACUUM_AND_MACHINE))
        .unwrap_or("");
    if machine.is_empty() {
        return Some(String::new());
    }
    if part == "pump" && _DRYER_KINDS.iter().any(|k| line.contains(k.as_str())) {
        let mut rest = line.clone();
        for kind in _DRYER_KINDS.iter() {
            rest = rest.replace(kind.as_str(), " ");
        }
        let said_the_pump = PUMP_WORD.is_match(&rest);
        if !said_the_pump && !_A_DRAIN_FAULT.iter().any(|w| line.contains(w.as_str())) {
            return Some(String::new());
        }
    }
    let mut text = lookup(&_MACHINE_PART_TEXT, part)?.to_string();
    if machine == "machine" {
        text.push_str(" The model number on the rating plate is what the parts shop needs.");
    } else {
        text.push_str(&format!(
            " Yours is a {machine}, so the model number on the rating plate is what \
             the parts shop needs."
        ));
    }
    Some(text)
}

/// A washer's drain fault keeps the washer's own filter card.
pub fn _the_washers_drain_has_its_own_answer(understood: &Understood) -> bool {
    let line = sh::squash_lower(sh::clean_or_text(understood));
    if !_A_DRAIN_FAULT.iter().any(|w| line.contains(w.as_str())) {
        return true;
    }
    !(line.contains("washing machine") || line.contains("washer"))
}

// ---------------------------------------------------------------------------------------------
// The mower.

static _MOWER_PETROL: LazyLock<Vec<String>> = LazyLock::new(|| strs("_MOWER_PETROL"));
static _MOWER_BLADE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_MOWER_BLADE"));
static _MOWER_CORDLESS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_MOWER_CORDLESS"));
static _MOWER_BATTERY_BUY: LazyLock<Vec<String>> = LazyLock::new(|| strs("_MOWER_BATTERY_BUY"));
static _MOWER_BATTERY: LazyLock<Vec<String>> = LazyLock::new(|| strs("_MOWER_BATTERY"));

/// A mower that will not start, a blunt blade, a battery, a filter or an oil change.
pub fn _mower(_memory: &Memory, understood: &Understood) -> String {
    let said: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let words: HashSet<&String> = said.iter().collect();
    let line = format!(" {} ", said.join(" "));
    let phrase_in = |list: &[String]| list.iter().any(|p| line.contains(&format!(" {p} ")));
    if phrase_in(&_MOWER_CORDLESS) {
        return "Let the battery come to room temperature first: a mower battery left in a cold \
                shed or a hot car will not start charging, and most chargers flash a fault light \
                instead. Then seat it fully in the charger and look for the charge light - if \
                there is nothing, try the charger on another tool of the same make, which tells \
                you whether it is the battery or the charger. One stored flat over the winter \
                often needs two or three charge-and-run cycles before it holds again, and check \
                the mower's safety key or handle switch is seated, because the mower will not run \
                and often will not charge with it out."
            .into();
    }
    if phrase_in(&_MOWER_BATTERY_BUY) {
        return "Take the old battery out and match the label: 12V and the amp-hour figure (12Ah, \
                18Ah and so on) for a ride-on, and the same shape and terminal layout, because a \
                different case size will not fit the tray or reach the leads. About 40 to 60 \
                pounds for a ride-on, more for one with a higher amp-hour rating, and a mower \
                shop or a battery place will match it from the old one."
            .into();
    }
    if _MOWER_BATTERY.iter().any(|w| words.contains(w)) || phrase_in(&_MOWER_BATTERY) {
        return "Charge it first, properly: a 12V charger on a ride-on's battery for four to six \
                hours, or overnight on a trickle setting, and clean the terminals with a wire \
                brush and tighten them while it is off - a loose or furred terminal behaves \
                exactly like a flat battery. If it spins over slowly and will not catch, the \
                battery is doing its job and the fuel or the spark plug is the fault. A jump \
                start from a car works, with the car's engine off and positive to positive, \
                negative to negative, but a battery over three or four years old that goes flat \
                twice is finished: a new one is about 40 to 60 pounds and it is a five-minute \
                swap."
            .into();
    }
    if phrase_in(&_MOWER_PETROL) {
        return "Nine times in ten a petrol mower that will not start is the fuel: petrol goes off \
                in about a month, so last autumn's half tank is the first thing to drain and \
                replace. Then work through it in this order - the spark plug out and checked \
                (clean it, set the gap, or fit a new one for a fiver), the air filter shaken out \
                or washed and dried, the oil level, and the blade for a jammed stick or a build-up \
                of wet grass under the deck, which drags the engine down and makes it stall. An \
                electric mower that will not start is the cable, the plug fuse and the cut-out \
                switch in the handle, in that order."
            .into();
    }
    if phrase_in(&_MOWER_BLADE) {
        return "Take the blade off - disconnect the spark plug lead first, or pull the electric \
                one out - and file the cutting edge back to the angle it already has, about 30 to \
                45 degrees, keeping the file flat against the bevel and taking a dozen strokes a \
                side. A blunt blade tears grass rather than cutting it, which is what leaves white \
                shredded tips and a lawn that yellows. Any deep nick or a bent blade means a new \
                one, about 15 to 25 pounds, and a bent blade is also a vibration you can feel. \
                Balance it on a nail afterwards and take a little more off the heavy side."
            .into();
    }
    if words.iter().any(|w| *w == "oil" || *w == "oili") || line.contains("change the oil") {
        return "Warm the engine for five minutes so the dirt comes out with the oil, put a tray \
                under the drain plug, tip the mower to drain it and refill with about half a litre \
                of SAE 30 four-stroke oil to the mark on the dipstick. Once a season, or every \
                25 hours of cutting. Four-stroke mowers take straight oil in a separate sump; a \
                two-stroke takes a petrol-and-oil mix and has no oil to change, so check which \
                yours is before you buy anything."
            .into();
    }
    "The air filter is the part that gets missed: a foam one comes out, washes in warm \
     soapy water, dries completely and goes back with a few drops of oil worked through \
     it, and a paper one is tapped clean or replaced. Do it every 25 hours or whenever \
     the grass is dusty, and change the oil once a season too - a mower that runs badly \
     or smokes is usually a choked filter and old fuel rather than anything broken."
        .into()
}

// ---------------------------------------------------------------------------------------------
// Crafts: reading a pattern, the first hour of a craft.

static CRAFT_KINDS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs_of(g("CRAFT_KINDS")));
static KNITTING_MEANINGS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs_of(g("KNITTING_MEANINGS")));
static CAST_ON: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bcast(?:ing)?\s+on\b", 0));

/// The craft named in the line, so a knitter is not read the sewing answer.
pub fn _read_pattern(memory: &Memory, understood: &Understood) -> String {
    let line = format!("{} {}", understood.text, understood.clean).to_lowercase();
    let kind = _named(&line, &CRAFT_KINDS).unwrap_or("knitting");
    let meaning = _named(&line, &KNITTING_MEANINGS).unwrap_or("");
    if !meaning.is_empty() {
        return meaning.to_string();
    }
    if kind == "knitting" && CAST_ON.is_match(&line) {
        return dup::pick_strs(
            &[
                "Cast on with the long tail, which is the one most patterns assume: pull out a tail \
                 about three times the width of the knitting, make a slip knot on the needle, then \
                 scoop the thumb strand through the finger strand and put that loop on the needle, \
                 over and over until you have the number the pattern asks for. Count them once at \
                 the end and, if you are one out, it matters - a stitch lost here shows up as a \
                 lop-sided edge fifty rows later.",
                "The long-tail cast on does nearly every job and gives a neat, stretchy edge: leave a \
                 tail three times the width of what you are making, knot it on, and use your thumb \
                 and finger to lift each new loop over the needle. The number you cast on is the \
                 pattern's first number, so if it lists sizes take the one you circled.",
            ],
            memory,
        );
    }
    if kind == "crochet" {
        return dup::pick_strs(
            &[
                "Crochet reads a row at a time, but check whose stitch names the pattern uses before \
                 you start: a US single crochet is a UK double crochet and a US double is a UK \
                 treble, and most patterns online are American. Then 'ch 3, turn' starts the next \
                 row, brackets and asterisks are a repeat worked as many times as the number after \
                 them says, and the number in brackets at the end of a row is how many stitches you \
                 should have - count it, because that is what tells you a stitch went missing ten \
                 rows earlier.",
                "Work down the page one row at a time and keep the count at the end of each row: the \
                 pattern tells you how many stitches you should have, and that number is the only \
                 thing that catches a mistake before it becomes a shape. Brackets and asterisks mean \
                 repeat, a turning chain is the 'ch 1' or 'ch 3' at the start of a row, and if the \
                 stitch names look wrong for the stitch you know, it is the country: US single \
                 crochet is UK double, US double is UK treble.",
            ],
            memory,
        );
    }
    if kind == "sewing" {
        return dup::pick_strs(
            &[
                "Circle your size on every piece before you cut anything: the pattern holds several \
                 sizes on the same lines, and the seam allowance is usually not included (1.5 cm, or \
                 5/8 inch, is printed on the envelope). Lay the pieces out with the arrow on the \
                 straight grain, match the notches, and fold the fabric wherever a piece says 'cut on \
                 the fold'. Read the instructions end to end once first - the order matters more than \
                 the sewing does.",
                "Read a paper pattern as a list of pieces and a list of steps, not as a diagram: the \
                 notches and the grain arrow line the fabric up, the same line carries every size so \
                 you trace yours with a pencil, and anything marked 'cut on the fold' is laid on a \
                 folded edge. Check the envelope for whether the seam allowance is included - it \
                 usually is not - and press each seam as you sew it, which is most of what makes the \
                 finished thing look right.",
            ],
            memory,
        );
    }
    dup::pick_strs(
        &[
            "Read a knitting pattern a line at a time: the number at the start is the row. k is knit, \
             p is purl, k2tog knits two stitches together (a decrease), kfb knits into the front and \
             back of one (an increase) and yo is a yarn over (a hole and an increase). Brackets or \
             asterisks are a repeat, worked as many times as the number after them says, and when a \
             line lists sizes the first number is the smallest - circle your own before you cast on. \
             Knit a 10 cm square in the pattern stitch first: if it comes out narrow, go up a needle \
             size.",
            "Start with the abbreviations key, then work a row at a time from the row number. Three \
             things trip people up: brackets with a number after them are a repeat, not a note; the \
             sizes are stacked in the same line with the smallest first, so ring yours in pencil; and \
             the tension at the top of the pattern is the size check, so knit a 10 cm square and \
             change the needle rather than the pattern if it is out. k2tog loses a stitch, kfb and yo \
             add one, and the stitch count at the end of a row should still match the pattern.",
        ],
        memory,
    )
}

static _A_PURL: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_PURL"));
static _A_SEWING_MACHINE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_SEWING_MACHINE"));
static _A_PATCH: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_PATCH"));
static _A_THREAD_NEEDLE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_THREAD_NEEDLE"));
static _A_POM_POM: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_POM_POM"));
static _A_IN_THE_ROUND: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_IN_THE_ROUND"));
static _A_A_SOCK: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_A_SOCK"));
static _A_JOIN_WOOL: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_JOIN_WOOL"));
static _A_NEEDLE_SIZE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_NEEDLE_SIZE"));
static _A_DEFINITION_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_DEFINITION_ASK"));
static _A_GETTING_INTO: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_GETTING_INTO"));

/// Not for a definition ("what is double knit wool") or a wish to take a craft up.
pub fn _start_craft_applies(reading: &Understood) -> bool {
    let line = sh::text_or_clean(reading);
    if _A_DEFINITION_ASK.is_match(line) {
        return false;
    }
    !_A_GETTING_INTO.is_match(line)
}

/// The first small thing that finishes today, in the craft the line names.
pub fn _start_craft(memory: &Memory, understood: &Understood) -> String {
    let line = format!("{} {}", understood.text, understood.clean).to_lowercase();
    let kind = _named(&line, &CRAFT_KINDS).unwrap_or("knitting");
    let pool: [&str; 2] = if _A_A_SOCK.is_match(&line) {
        [
            "A sock is the second project, not the first, and the reason is the heel: it is a \
             turning point you knit back and forth on half the stitches, and nothing before it \
             prepares you for it - but it is four small steps in order, not one hard one. For an \
             adult sock in 4-ply, cast on 60 stitches on 2.5 mm needles, join in the round \
             without a twist, and work 2x2 rib for 5 cm, then knit every round for 15 cm. Then \
             the heel flap over 30 stitches for about 30 rows, turn the heel, pick up the \
             gusset, and decrease back to 60 before the foot.",
            "Sock yarn wants nylon in it - about 75 per cent wool and 25 per cent nylon, sold as \
             4-ply or sock yarn - because pure wool wears through at the heel inside a season, \
             and 60 stitches on 2.5 mm needles is the usual adult size. Knit the leg in the \
             round until it is 15 cm, then the heel: half the stitches back and forth for 30 \
             rows, turn the heel, pick up the gusset, and decrease back to 60. Write the count \
             down as you go, because a sock is the one thing where losing count shows.",
        ]
    } else if _A_JOIN_WOOL.is_match(&line) {
        [
            "Join a new ball at the end of a row and leave 15 cm tails: knit three or four \
             stitches holding the old and new strands together, drop the old, and carry on. \
             Then darn the two tails into the back of the work at the end rather than knotting \
             them - a knot works loose in the wash and leaves a hard lump in the fabric. Same \
             for a colour change: swap the yarn at the start of a row, and the jog where the \
             colours meet sits at the side seam if you plan it there.",
            "The join that does not show is a spit splice on pure wool: fray the two ends, \
             overlap them 5 cm, wet them and rub them between your palms until the fibres felt \
             together. On anything else, leave 15 cm tails, knit a few stitches with both \
             strands, and weave the tails in afterwards with a blunt needle. Never knot them - \
             the knot is what comes undone and what you feel through the sole of a sock.",
        ]
    } else if _A_IN_THE_ROUND.is_match(&line) {
        [
            "Circular needles, one length that fits the job: 40 cm for a hat, 60 cm for a \
             child's jumper, 80 cm for an adult's. Cast on the whole round at once, check the \
             cast-on edge is not twisted before you join, put a marker at the join and knit \
             every stitch - no turning - and you get stocking stitch without ever purling. A \
             hat is 90 stitches on a 4 mm needle, knit for 20 cm, then decrease 10 stitches \
             every other round until 10 are left and pull the yarn through them.",
            "The join is the only tricky moment: the cast-on edge has to run straight round \
             without a twist in it, or the whole piece is a Mobius and you find out 10 cm in. \
             Lay it flat on the table, line the bumps up along the inside, then knit the first \
             stitch and put a marker there. On a 40 cm needle a hat works in one piece; for a \
             sleeve or a jumper neck that is too small for the needle, pull a loop of cable out \
             at the side - the magic loop - rather than buying double-pointed needles.",
        ]
    } else if _A_SEWING_MACHINE.is_match(&line) {
        [
            "Thread it in this order and it will sew: spool, the arm guide, the tension dial, \
             the take-up lever, then the needle from front to back. Wind a bobbin, drop it in \
             with the thread trailing to the left, and turn the handwheel towards you with the \
             needle down to bring the bobbin thread up. Then sew on a scrap before the real \
             thing: stitch length about 2.5 mm, the presser foot down before the pedal, and let \
             the machine pull the fabric - pulling it bends the needle and skips stitches.",
            "The order is what matters: the spool, round the arm guide, through the tension \
             dial, up and through the take-up lever, down to the needle, threaded front to back. \
             Then the bobbin, wound and dropped in with its thread to the left, and one turn of \
             the handwheel brings the bobbin thread up through the plate. Set the stitch length \
             to 2.5 mm, put the foot down, and sew a line on scrap: if the stitches loop on the \
             underside it is the top tension, and if it puckers the stitch is too long.",
        ]
    } else if _A_THREAD_NEEDLE.is_match(&line) {
        [
            "Cut the thread at a slant rather than straight across, wet it and pinch it flat, \
             then push it through the eye against something white so you can see the hole. A \
             needle threader costs pennies and does it in a second. Knot the two ends together \
             so you are working with a double thread, and cut it to about a forearm's length - \
             longer than that tangles and snarls with every stitch.",
            "The thread goes through the eye far more easily if it is cut at an angle and \
             pressed flat between wet fingers, and a needle threader turns the whole thing into \
             a two-second job. Use a length about as long as your forearm, doubled and knotted \
             at the end, because a long thread twists, knots and wears thin before it runs out.",
        ]
    } else if _A_POM_POM.is_match(&line) {
        [
            "Cut two circles of card the size you want, each with a hole in the middle the size \
             of a 10p piece, and wind yarn round both together until the hole is filled. Cut \
             the yarn round the outside between the two circles, tie a long thread tight round \
             the middle and knot it twice, then take the card away and trim it round. Tie it \
             hard - a loose tie is the one thing that makes a pom pom fall apart.",
            "Two cardboard rings, a hole in the middle, yarn wound through the hole until it \
             is full and the middle is closed: then cut between the rings all the way round, \
             slip a doubled thread round the waist of the bundle, and pull it as tight as you \
             can before you knot it. Trim it while it is still on the card if you want a tidy \
             ball, and use a plate or a coin as the template so both rings match.",
        ]
    } else if _A_PATCH.is_match(&line) {
        [
            "Patch it from both sides: put a piece of similar-weight fabric or iron-on denim \
             over the hole, press it with a hot iron through a damp cloth, then sew round the \
             edge with a straight or zigzag stitch so it cannot lift. On a knee, put the patch \
             inside and stitch the shape of the hole closed first - a patch on the outside of a \
             knee rubs through in weeks, and on the seat it looks like a repair from the bus.",
            "The fix is a patch as close to the weight of the denim as you can find, ironed on \
             with a hot iron through a cloth and then stitched round the edge; the stitching is \
             what survives the washing. Put it inside the leg for a knee or a seat and outside \
             for a rip at the hem, and stitch the frayed edges flat first so the patch has \
             something to hold.",
        ]
    } else if kind == "sewing" {
        [
            "Hand-sewing is three stitches: running stitch, in and out in an even line; back \
             stitch, one back for every two forward, which is the one that holds a seam that \
             takes a pull; and slip stitch, worked inside a fold so it barely shows, which is \
             what a hem wants. Start and finish with two tiny back stitches on the spot instead \
             of a knot, and keep the thread to a forearm's length.",
            "Learn running stitch first - in and out in an even line, which is gathering, \
             basting and half the repairs you will ever do - then back stitch, which is the \
             strong one and goes one stitch back for every two forward. A hem is slip stitch, \
             worked along the inside of the fold. An iron between steps does more for how the \
             finished thing looks than neat stitches do.",
        ]
    } else if _A_NEEDLE_SIZE.is_match(&line) {
        [
            "Match the needle to the yarn and the pattern does the rest: 4 mm for double knit, \
             5 mm for aran, 6 mm for chunky, and the ball band prints the size that suits it. \
             For a scarf, 5 mm with aran and 30 stitches gives about 20 cm wide, and two 100 g \
             balls make one about 1.5 m long; a hat is 5 mm and about 80 stitches worked in the \
             round.",
            "The ball band is the answer - it prints the needle size the yarn was made for, \
             and 4 mm for double knit, 5 mm for aran and 6 mm for chunky are the usual. Then \
             the pattern's tension square is the check: knit 10 cm, and change the needle rather \
             than the pattern if the square comes out the wrong size. Thirty stitches on 5 mm is \
             a scarf about 20 cm wide.",
        ]
    } else if kind == "crochet" {
        [
            "Start with a 4 mm hook and a smooth double knit: a slip knot, then chain 12, then \
             one double crochet into the second chain from the hook and into each chain after \
             it, turning at the end of every row. That is a square, and the same stitch in rows \
             is a blanket. Check which country the pattern is from before you learn its names: \
             a US double crochet is a UK treble.",
            "A 4 mm hook, a smooth double knit and one stitch gets you going: chain 12, then \
             double crochet into the second chain and every one after it, turn, chain one, and \
             work back along the row. Count at the end of each row - the count is the only thing \
             that catches a dropped stitch before it becomes a sloping edge.",
        ]
    } else if _A_PURL.is_match(&line) {
        [
            "A purl is a knit made with the yarn in front: keep the yarn towards you, put the \
             right needle into the front of the stitch from right to left, wrap the yarn over \
             the needle and pull the loop through to the back. Knit one and purl one and you \
             have ribbing, which is what makes a cuff or a hat brim pull in and stay up.",
            "For a purl the yarn goes in front of the work, the needle goes into the front of \
             the stitch from right to left, and the new loop is brought through to the back. \
             Ribbing - knit one, purl one - is the reason to learn it: it stretches and pulls \
             back, so it is the cuff, the neckband and the bottom of a hat.",
        ]
    } else {
        [
            "Start with one square you can finish today: 5 mm needles and a smooth aran yarn, cast \
             on about 30 stitches and knit every stitch of every row. That is garter stitch, it is \
             flat and it does not curl, and 30 stitches is a scarf about 20 cm wide - two 100 g \
             balls make one about 1.5 m long. Your tension evens out after ten rows, so the first \
             rows looking uneven is not a mistake.",
            "The way in is the smallest finished thing: cast on 30 stitches on 5 mm needles, knit \
             every row until it looks like a scarf, and cast off. Knit every row is garter stitch \
             and needs no pattern at all. When a row goes wrong, undo it stitch by stitch rather \
             than pulling it all out - a dropped stitch is easy to pick back up with a crochet hook.",
        ]
    };
    dup::pick_strs(&pool, memory)
}

/// What the marks on a tape mean, and the one thing that makes the number wrong.
pub fn _read_tape(memory: &Memory, _understood: &Understood) -> String {
    dup::pick_strs(
        &[
            "On a metric tape the big numbers are centimetres and the small marks between them are \
             millimetres, ten to the centimetre, so three marks past the 7 is 73 mm. The imperial \
             side runs the other way up: between two inches the longest small mark is the half, the \
             next longest the quarter, then eighths, then sixteenths. Lay the tape flat and read it \
             with your eye straight above the mark, and add the width of the hook if you are \
             measuring to an outside edge.",
            "The numbers on a tape are centimetres with millimetres marked between them, and the \
             hook at the end is loose on purpose - about a millimetre of play, so it works hooked \
             over an edge and butted against one. For anything inside a frame or a recess, measure \
             from the 100 mm mark instead and take 100 mm off the reading, and the hook never \
             matters. On a ruler the zero is at the very end, so check a new ruler against a tape \
             once: some are trimmed a millimetre short.",
        ],
        memory,
    )
}

static LASER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\blaser\b", 0));

/// The bubble, the check in both directions, and the check on the tool itself.
pub fn _read_level(memory: &Memory, understood: &Understood) -> String {
    let line = format!("{} {}", understood.text, understood.clean).to_lowercase();
    if LASER.is_match(&line) {
        return dup::pick_strs(
            &[
                "A laser level throws a line or a dot along the wall instead of a bubble: stand it on \
                 something solid, switch it on, and wait for the line to stop blinking before you \
                 trust it - that blink is it levelling itself. Once it is steady, work to the line, \
                 and check it by turning the unit round and seeing whether the line lands in the same \
                 place; a laser that has been dropped can be a few millimetres out over a room.",
                "Set the laser on a tripod or a firm surface and give it a moment to settle - most \
                 self-level within a few degrees, and the light flashes until they have. The line is \
                 your reference at whatever height you set it, so mark both ends of a shelf from the \
                 line rather than measuring from one end, and if the room is bright, or the run is \
                 longer than about five metres, use the detector rather than your eye.",
            ],
            memory,
        );
    }
    dup::pick_strs(
        &[
            "The bubble sits between the two lines when the surface is level, and if it is touching a \
             line the surface is off by about 2 mm over a metre. Check along the shelf and then \
             across its depth - a shelf can be level one way and tilted the other - and turn the level \
             end for end over the same spot: if the bubble moves, the level is out, not the shelf. Use \
             the longest one that fits, because a 600 mm level shows four times the gap a 150 mm one \
             does on the same slope.",
            "Put the level on the surface and read the bubble: in the middle, between the two lines, \
             is level, and the further it drifts towards a line the steeper the surface. Then test \
             the other way across the same spot, and swap the level's ends to check the tool itself - \
             a dropped level is out by more than the shelf is. A shelf wants to be within a couple of \
             millimetres over its length, so mark the fixings from the level rather than trusting the \
             wall.",
        ],
        memory,
    )
}

// ---------------------------------------------------------------------------------------------
// Marks on a table, a water softener, a chip in a surface, hot water faults.

static WHITE_ONLY: LazyLock<Vec<String>> = LazyLock::new(|| strs("WHITE_ONLY"));
static SCRATCH_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("SCRATCH_WORDS"));

/// A white ring or a water mark on its own is the stain card's question, not the scratch.
pub fn _not_only_a_white_mark(reading: &Understood) -> bool {
    let line = sh::or(sh::text_or_clean(reading), &reading.plain).to_lowercase();
    if SCRATCH_WORDS.iter().any(|w| line.contains(w.as_str())) {
        return true;
    }
    !WHITE_ONLY.iter().any(|w| line.contains(w.as_str()))
}

static A_SOFTENER_SALT_WORD: LazyLock<Vec<String>> = LazyLock::new(|| strs("A_SOFTENER_SALT_WORD"));
static SOFTENER_BUYING: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\bworth\b|\bshould (?:i|we) (?:get|buy|have|install|bother|invest)\b|\bdo (?:i|we) (?:want|need)\b|\bis it necessary\b|\bgood idea\b|\bworth the money\b",
        0,
    )
});

/// True unless the line asks whether to have a softener rather than how to fill one.
pub fn _not_the_buying_question(understood: &Understood) -> bool {
    let said = understood.text.to_lowercase();
    if A_SOFTENER_SALT_WORD.iter().any(|w| said.contains(w.as_str())) {
        return true;
    }
    !SOFTENER_BUYING.is_match(&said)
}

/// True when one of the words is said as a word, or the phrase as a phrase.
pub fn _chip_says(said: &str, words: &[&str]) -> bool {
    words.iter().any(|w| if w.contains(' ') { said.contains(w) } else { word_re(w, r"\b").is_match(said) })
}

/// The repair for a chip, by what has been chipped.
pub fn _a_chip_in_it(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::text_or_clean(understood).to_lowercase();
    let s = said.as_str();
    if _chip_says(s, &["laminate", "veneer", "formica", "melamine", "chipboard", "ikea"]) {
        return "Seal a laminate chip rather than fill it: a repair wax or a touch-up kit in the \
                same colour goes in with a fingertip and a clear top coat stops water reaching \
                the chipboard core, which is the part that swells and lifts the surface. If the \
                edge of the chip has lifted, press it down with a warm iron through a cloth \
                first. A chip along a front edge that keeps catching wants a new worktop strip or \
                a whole top, because a filler there takes the knocks for a few months only."
            .into();
    }
    if _chip_says(s, &["stone", "granite", "quartz", "marble", "composite", "corian", "silestone", "solid surface"]) {
        return "A stone worktop chip is a two-part epoxy job, tinted to the stone: clean the \
                chip out, build the epoxy slightly proud, let it cure hard and then flat it back \
                with a fine diamond pad and polish. Granite and quartz are a fitter's call if the \
                chip is bigger than a fingertip or on a joint, and most fitters will do a small \
                one for a call-out fee. Never use a filler that shrinks or a heat gun near the \
                stone, because a thermal shock cracks it."
            .into();
    }
    if _chip_says(s, &["worktop", "worktops", "counter", "countertop", "counter top", "breakfast bar"]) {
        return "Find out what the top is made of before you buy anything, because the three \
                repairs do not swap. A laminate or veneered top takes a repair wax in the \
                matching colour and a clear seal over it, so water cannot reach the chipboard \
                core and swell it; solid wood is sanded back and filled with a wax or wood filler \
                and then re-oiled; stone or quartz takes a tinted two-part epoxy, built proud, \
                flattened and polished. A chip on a front edge that keeps catching a sleeve is \
                the one worth a new edge strip instead of a filler."
            .into();
    }
    if _chip_says(s, &["sink", "sinks", "basin", "ceramic", "porcelain", "glazed", "glaze"]) {
        return "Match the kit to the sink: a glazed ceramic one takes an enamel repair kit, and \
                an acrylic or composite one takes an acrylic filler matched to the colour, \
                because the enamel kit will not bond to plastic. Clean the chip right out, let \
                it dry, and build two or three thin coats rather than one thick one. A chip \
                through the glaze on a ceramic sink keeps dirt in the crater, so it is worth \
                doing even though it will still show."
            .into();
    }
    if _chip_says(s, &["enamel", "bath", "cast iron", "steel bath", "shower tray", "cooker", "hob"]) {
        return "An enamel chip wants an enamel repair kit: clean it, de-grease it, then build two \
                or three thin coats of the enamel touch-up rather than one thick one, letting \
                each go off. It is a cosmetic repair that stops the chip rusting and picking up \
                dirt, and it will show a little; a bath chip through to the cast iron has to be \
                done or the enamel round it starts to flake. A chip on a hob ring or a shower \
                tray is the same kit, and a crack rather than a chip is a replacement."
            .into();
    }
    if _chip_says(
        s,
        &[
            "plate", "plates", "mug", "mugs", "bowl", "bowls", "cup", "cups", "teapot", "jug", "dish", "dishes",
            "glass", "glasses", "tumbler", "wine glass", "pint glass",
        ],
    ) {
        return "Retire a chipped plate, mug or glass: the chip is a sharp edge that will cut a \
                lip, the glaze round it is no longer sealed, and there is no repair worth making. \
                A chip on the foot or the underside is the one that is safe to keep for a plant \
                pot or a pet bowl, and if it is glass, a chip anywhere near the rim is a break \
                waiting to happen with hot water in it."
            .into();
    }
    if _chip_says(s, &["tile", "tiles", "grout", "splashback"]) {
        return "A small chip in a tile hides with a dab of the same-colour grout, a touch-up \
                pen or a drop of enamel, worked into the chip with a fingertip and wiped off the \
                face at once. A chip out of a corner or an edge is the tile coming away from the \
                wall, so press it back and check for a hollow sound; if it is hollow, the tile \
                wants replacing rather than filling, and buy a spare now because the shade will \
                not match later."
            .into();
    }
    if _chip_says(s, &["gap", "gaps", "crack between", "cracks between", "opening up", "draught", "draft"]) {
        return "A gap in a floor or under a skirting wants a flexible filler, not a hard one: rake \
                the dust out, push in a foam backer rod if it is deep, then run a paintable \
                acrylic sealant or a sliver of matching wood glued in and planed flat, because a \
                hard filler in a floor cracks out with the first bit of movement. Between \
                floorboards on a suspended floor, fill from above with a sliver of pine and glue \
                rather than sealing it, so the boards can still move."
            .into();
    }
    if _chip_says(
        s,
        &["wood", "wooden", "oak", "pine", "beech", "walnut", "table", "tabletop", "floorboard", "floor", "door", "skirting", "furniture"],
    ) {
        return "Sand the chip back to sound wood, then fill it with a wood filler or a wax stick \
                in the right colour, building it a little proud and paring it flat once it has \
                gone off. Wipe a drop of oil or the same finish over it afterwards, or the bare \
                patch shows as a light mark and dries out again. A chip on a corner that keeps \
                being knocked wants a rounded edge or a corner trim rather than another fill."
            .into();
    }
    "Match the repair to the surface, because a filler that is right for one is wrong for \
     the next. A laminate chip takes a repair wax and a clear seal so water cannot reach \
     the core; a stone top takes tinted two-part epoxy, flattened and polished; an enamel \
     bath or basin takes an enamel touch-up kit in thin coats; a chipped plate or glass is \
     one to retire, and a chipped tile takes colour-matched grout or enamel, or a \
     replacement if the corner has lifted. Photograph it and take the chip's colour to a \
     hardware shop rather than trusting the name on the tin."
        .into()
}

static _HOT_FAULT_LEAK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HOT_FAULT_LEAK"));
static _HOT_FAULT_TEST: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HOT_FAULT_TEST"));

/// A leak, no heat, or an element that trips the fuse: three faults with three answers.
pub fn _hot_water_fault(_memory: &Memory, reading: &Understood) -> String {
    let said = format!(" {} ", sh::clean_or_text(reading).to_lowercase());
    let padded_in = |w: &str| said.contains(&format!(" {w} "));
    if _HOT_FAULT_LEAK.iter().any(|w| padded_in(w)) {
        return string("_HOT_FAULT_LEAK_ANSWER");
    }
    if _HOT_FAULT_TEST.iter().any(|w| padded_in(w)) {
        return string("_HOT_FAULT_TEST_ANSWER");
    }
    if ["tripping", "trips", "trip", "fuse", "fuse box", "breaker", "rcd", "mcb", "cut out", "cutting out"]
        .iter()
        .any(|w| padded_in(w))
    {
        return string("_HOT_FAULT_TRIP_ANSWER");
    }
    string("_HOT_FAULT_COLD_ANSWER")
}

// ---------------------------------------------------------------------------------------------
// Servicing a machine, the words on a lease, a pair of glasses, paper.

/// The machine a service question is about, or "" when it names none of ours.
pub fn _service_machine(reading: &Understood) -> String {
    let said = format!(" {} ", text::squash(&reading.clean));
    for name in ["washing machine", "washer", "dishwasher", "tumble dryer", "dryer", "oven", "fridge freezer", "fridge", "freezer"] {
        if !word_re(name, r"s?\b").is_match(&said) {
            continue;
        }
        return match name {
            "washer" => "washing machine".into(),
            "dryer" => "tumble dryer".into(),
            _ => name.into(),
        };
    }
    String::new()
}

/// The verdict, or the interval, for servicing the machine the line names (None for a price or a
/// how-to, or a machine that is not ours).
pub fn _a_service_worth(_memory: &Memory, reading: &Understood) -> Option<String> {
    let asks: HashSet<&str> = reading.asks.iter().map(String::as_str).collect();
    let words: HashSet<&str> = reading.words.iter().map(String::as_str).collect();
    let said = reading.clean.as_str();
    if asks.contains("how_much") || said.contains("how do i") || said.contains("how can i") {
        return None;
    }
    if !(["yesno", "how_often", "recommend"].iter().any(|a| asks.contains(a))
        || words.contains("worth")
        || words.contains("worthwhile")
        || words.contains("point"))
    {
        return None;
    }
    let machine = _service_machine(reading);
    let often = asks.contains("how_often") || said.contains("how often");
    let text = match machine.as_str() {
        "washing machine" => {
            if often {
                "There is no yearly service to book: a washing machine has no service \
                 interval like a boiler's. What keeps it going is cleaning the filter, \
                 leaving the door open to dry, and a hot 60C maintenance wash once a month, \
                 with a descaler if your water is hard. Pay someone only when it is noisy, \
                 leaking or not draining, and that is a repair rather than a service."
            } else {
                "No, a washing machine has no yearly service like a boiler's. What keeps it going \
                 is cleaning the filter, leaving the door open to dry, and a hot 60C maintenance \
                 wash once a month, with a descaler if your water is hard. Pay someone only when \
                 it is noisy, leaking or not draining, and that is a repair rather than a service."
            }
        }
        "dishwasher" => {
            if often {
                "There is no yearly service to book: a dishwasher looks after itself if you \
                 clean the filter once a month, keep the salt topped up in hard water and run \
                 the hottest cycle now and then. A paid visit is only worth it if it smells \
                 or will not drain, and then it is a repair."
            } else {
                "No, a dishwasher does not need a yearly service. Clean the filter once a month, \
                 keep the salt topped up if your water is hard, and run the hottest cycle now and \
                 then; a paid visit is only worth it if it smells or will not drain, and then it is \
                 a repair."
            }
        }
        "tumble dryer" => {
            if often {
                "There is no yearly service to book: a dryer needs its lint filter emptied \
                 after every load, its vent pipe cleared, and the condenser filter rinsed \
                 once a month on a heat-pump model. Pay someone only if it takes two cycles \
                 to dry, which is usually a blocked vent or a worn belt."
            } else {
                "No, a dryer does not need a yearly service. Empty the lint filter after every \
                 load, clear the vent pipe, and rinse the condenser filter on a heat-pump dryer \
                 once a month. Pay someone only if it takes two cycles to dry, which is usually a \
                 blocked vent or a worn belt."
            }
        }
        "oven" => {
            if often {
                "There is no yearly service to book: clean the shelves and the door glass, \
                 run the pyrolytic setting if it has one, and leave the rest alone unless \
                 it stops heating evenly. A paid oven clean is cleaning, not servicing."
            } else {
                "No, an oven does not need a yearly service. Clean the shelves and the door \
                 glass, run the pyrolytic setting if it has one, and leave the rest alone \
                 unless it stops heating evenly; a paid oven clean is cleaning, not servicing."
            }
        }
        "fridge freezer" | "fridge" | "freezer" => {
            if often {
                "There is no yearly service to book: dust the coils at the back once a year, \
                 keep the door seal clean, and leave a gap round it for the heat to escape. \
                 Pay someone only when it stops holding temperature."
            } else {
                "No, a fridge or freezer does not need a yearly service. Dust the coils at the \
                 back once a year, keep the door seal clean, and leave a gap round it for the heat \
                 to escape; pay someone only when it stops holding temperature."
            }
        }
        _ => return None,
    };
    Some(text.into())
}

static _PROPERTY_WORDS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows(g("_PROPERTY_WORDS")));
static PROPERTY_SERVICE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("PROPERTY_SERVICE_WORDS"));
static PROPERTY_SERVICE_CONTEXT: LazyLock<Vec<String>> = LazyLock::new(|| strs("PROPERTY_SERVICE_CONTEXT"));

/// The word the line asks about, answered in the first sentence (None when it names none).
pub fn _property_word(understood: &Understood) -> Option<String> {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let text = words.join(" ");
    for (keys, reply) in _PROPERTY_WORDS.iter() {
        for key in keys {
            if !(key.contains(' ') && text.contains(key.as_str())) && !words.contains(key) {
                continue;
            }
            if PROPERTY_SERVICE_WORDS.contains(key)
                && !PROPERTY_SERVICE_CONTEXT.iter().any(|w| words.contains(w) || (w.contains(' ') && text.contains(w.as_str())))
            {
                continue;
            }
            return Some(reply.clone());
        }
    }
    None
}

static _A_PAIR_OF_GLASSES: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_PAIR_OF_GLASSES"));

/// False when the line is about spectacles rather than the glass somebody was drinking from.
pub fn _not_a_pair_of_glasses(understood: &Understood) -> bool {
    let said = joined(&[&understood.clean, &understood.norm_text, &understood.text]).to_lowercase();
    !_A_PAIR_OF_GLASSES.is_match(&said)
}

static _PAPER_PHOTOS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PAPER_PHOTOS"));
static _PAPER_SMELL: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PAPER_SMELL"));

/// Paper that is yellowing, fading or smelling musty: damp first, then the light, then the acid.
pub fn _keeping_paper(_memory: &Memory, understood: &Understood) -> String {
    let line = format!(" {} ", sh::clean_or_text(understood).to_lowercase());
    let ends = text::rstrip(&line);
    let said = |word: &String| line.contains(&format!(" {word} ")) || ends.ends_with(&format!(" {word}"));
    if _PAPER_SMELL.iter().any(said) {
        return "Musty paper is damp paper, and the smell is mould growing in it: stand the books \
                or letters up with the pages fanned in a dry room for a day or two - not in \
                sunlight, which yellows them faster - then keep them somewhere dry and steady. A \
                sachet of silica gel in the box, a cupboard inside the house rather than the \
                loft, garage or cellar, and a dehumidifier in a room that has had a leak will \
                stop it coming back. Don't wipe them with bleach or a wet cloth: it takes the \
                print with it."
            .into();
    }
    if _PAPER_PHOTOS.iter().any(said) {
        return "Prints fade from light and from the plastic in old albums, so keep them out of \
                sunlight, in acid-free sleeves or a proper album rather than a PVC one, and scan \
                the ones that matter - a digital copy is the only version that cannot fade. A \
                framed photo wants UV-filtering glass and a wall that never gets direct sun."
            .into();
    }
    "Cheap paper is made from wood pulp, and the lignin in it turns acidic and brown as it \
     sits - light, heat and damp speed it up, which is why the box in the loft goes first. \
     So keep paper cool, dry, dark and steady: a cupboard inside the house, not the loft, \
     shed or cellar; acid-free folders and boxes rather than polythene bags, which trap \
     damp; no sellotape and no lamination, both of which stain and cannot be undone. \
     Yellowing that has already started can be slowed, not reversed."
        .into()
}

/// The property-word row's reply: the word the line asks about.
pub fn lambda_8271(_memory: &Memory, understood: &Understood) -> Option<String> {
    _property_word(understood)
}

static _A_BOUNDARY_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_BOUNDARY_THING"));

/// False when the line is about a fence, a hedge or a garden wall rather than a party wall.
pub fn _not_a_boundary(understood: &Understood) -> bool {
    !_A_BOUNDARY_THING.is_match(sh::clean_or_text(understood))
}

static _HOME_WORDS_2: LazyLock<Vec<(Vec<String>, String, Vec<String>)>> = LazyLock::new(|| {
    g("_HOME_WORDS_2")
        .items()
        .iter()
        .map(|r| {
            let x = r.items();
            (x[0].strs(), x[1].str().to_string(), x[2].strs())
        })
        .collect()
});

/// The words on a tenancy and on the money round a flat (None when none is named).
pub fn _home_word_2(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = understood.clean.as_str();
    let words = &understood.words;
    for (keys, reply, avoid) in _HOME_WORDS_2.iter() {
        if !keys.iter().any(|k| text.contains(k.as_str()) || words.contains(k)) {
            continue;
        }
        if avoid.iter().any(|w| text.contains(w.as_str()) || words.contains(w)) {
            continue;
        }
        return Some(reply.clone());
    }
    None
}

// ---------------------------------------------------------------------------------------------
// A count of what the job needs: reading a size off the line.

static _METRE: LazyLock<Vec<(String, f64)>> = LazyLock::new(|| {
    g("_METRE").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_f64().unwrap_or(1.0))).collect()
});
static _SAID_NUMBER: LazyLock<Vec<(String, f64)>> = LazyLock::new(|| {
    g("_SAID_NUMBER").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_f64().unwrap_or(0.0))).collect()
});
static _SIDE_BY_SIDE: LazyLock<&'static Re> = LazyLock::new(|| rx("_SIDE_BY_SIDE"));
static _A_SQUARE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_SQUARE"));
static _A_LENGTH: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_LENGTH"));
static _A_LONG_AND_HIGH: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_LONG_AND_HIGH"));
static _A_HIGH_AND_LONG: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_HIGH_AND_LONG"));
static _A_CEILING_BEFORE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_CEILING_BEFORE"));
static _A_CEILING_AFTER: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_CEILING_AFTER"));
static _A_CEILING_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_CEILING_THING"));
static _DEFAULT_CEILING: LazyLock<f64> = LazyLock::new(|| g("_DEFAULT_CEILING").as_f64().unwrap_or(2.4));
static _ONE_SURFACE: LazyLock<&'static Re> = LazyLock::new(|| rx("_ONE_SURFACE"));
static _A_ROOM: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_ROOM"));
static ROOM_OR_WALL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\brooms?\b|\bwalls?\b", 0));

/// Python truthiness of an optional figure (None and 0.0 are false).
fn t(x: Option<f64>) -> bool {
    x.is_some_and(|v| v != 0.0)
}

/// The value of a number as typed or as said ("3", "three"), or None.
pub fn _said_number(raw: &str) -> Option<f64> {
    let raw = text::strip(raw).to_lowercase();
    if let Some((_, v)) = _SAID_NUMBER.iter().find(|(k, _)| *k == raw) {
        return Some(*v);
    }
    // float(raw.replace(",", "")); a ValueError is None
    let cleaned = raw.replace(',', "");
    text::strip(&cleaned).parse::<f64>().ok()
}

/// A value in metres: the unit as a person writes it ("metres", "m", "ft"), default metres.
pub fn _in_metres(raw: &str, unit: Option<&str>) -> Option<f64> {
    let value = _said_number(raw)?;
    let unit = text::strip(sh::or(unit.unwrap_or(""), "m")).to_lowercase();
    let factor = _METRE
        .iter()
        .find(|(k, _)| *k == unit)
        .or_else(|| {
            let bare = text::rstrip_chars(&unit, "s");
            _METRE.iter().find(|(k, _)| k == bare)
        })
        .map(|(_, f)| *f)
        .unwrap_or(1.0);
    Some(value * factor)
}

/// A group that did not take part is None; `found.group(4) or found.group(2)`.
fn group_or<'t>(m: &crate::py::re::Match<'t>, a: usize, b: usize) -> Option<&'t str> {
    m.get(a).filter(|s| !s.is_empty()).or_else(|| m.get(b))
}

/// (area in square metres or None, one length or perimeter in metres or None) from the line.
pub fn _measurements(clean: &str) -> (Option<f64>, Option<f64>) {
    for found in _SIDE_BY_SIDE.finditer(clean) {
        let first = _in_metres(found.str(1), found.get(2));
        let second = _in_metres(found.str(3), group_or(&found, 4, 2));
        let (Some(first), Some(second)) = (first.filter(|v| *v != 0.0), second.filter(|v| *v != 0.0)) else { continue };
        if (0.3..=40.0).contains(&first) && (0.3..=40.0).contains(&second) {
            return (Some(dup::py_round(first * second, 2)), Some(dup::py_round(2.0 * (first + second), 2)));
        }
    }
    for (pattern, height_first) in [(&**_A_LONG_AND_HIGH, false), (&**_A_HIGH_AND_LONG, true)] {
        let Some(found) = pattern.search(clean) else { continue };
        let first = _in_metres(found.str(1), found.get(2));
        let second = _in_metres(found.str(3), found.get(4));
        let (Some(first), Some(second)) = (first.filter(|v| *v != 0.0), second.filter(|v| *v != 0.0)) else { continue };
        let (run, height) = if height_first { (second, first) } else { (first, second) };
        if (0.3..=40.0).contains(&run) && (0.3..=40.0).contains(&height) {
            return (Some(dup::py_round(run * height, 2)), Some(dup::py_round(run, 2)));
        }
    }
    if let Some(found) = _A_SQUARE.search(clean) {
        let value = _said_number(found.str(1));
        let unit = found.get(2).filter(|s| !s.is_empty()).or_else(|| found.get(3).filter(|s| !s.is_empty())).unwrap_or("m");
        if let Some(value) = value.filter(|v| *v != 0.0) {
            let m = _in_metres("1", Some(unit)).unwrap_or(1.0);
            return (Some(dup::py_round(value * m.powi(2), 2)), None);
        }
    }
    if let Some(found) = _A_LENGTH.search(clean) {
        if let Some(value) = _in_metres(found.str(1), found.get(2)).filter(|v| *v != 0.0) {
            return (None, Some(dup::py_round(value, 2)));
        }
    }
    (None, None)
}

/// The ceiling height the line names in metres, or None.
pub fn _the_ceiling(clean: &str) -> Option<f64> {
    for pattern in [&**_A_CEILING_BEFORE, &**_A_CEILING_AFTER] {
        let Some(found) = pattern.search(clean) else { continue };
        if let Some(value) = _in_metres(found.str(1), found.get(2)) {
            if value != 0.0 && (1.8..=5.0).contains(&value) {
                return Some(dup::py_round(value, 2));
            }
        }
    }
    None
}

/// (square metres to paint, the ceiling height used or None, what is painted) from the line, or
/// from the turn that gave the size.
pub fn _wall_area(understood: &Understood, memory: Option<&Memory>) -> (Option<f64>, Option<f64>, Option<&'static str>) {
    let clean = _the_line(understood);
    let mut earlier: Option<String> = None;
    let (a, r) = _measurements(&clean);
    if !(t(a) || t(r)) {
        let remembered = memory.and_then(|m| _the_size_said_before(Some(m), &clean));
        earlier = remembered.map(|(_, _, said)| said.to_lowercase());
    }
    for text in [Some(clean.clone()), earlier] {
        let Some(text) = text.filter(|s| !s.is_empty()) else { continue };
        let (area, run) = _measurements(&text);
        if !t(area) && !t(run) {
            continue;
        }
        if _A_LONG_AND_HIGH.is_match(&text) || _A_HIGH_AND_LONG.is_match(&text) {
            return if t(area) { (area, None, Some("wall")) } else { (None, None, None) };
        }
        if t(area) && run.is_none() {
            return (area, None, Some("wall"));
        }
        if !t(area) || !t(run) {
            return (None, None, None);
        }
        if _A_CEILING_THING.is_match(&text) && _the_ceiling(&text).is_none() && !ROOM_OR_WALL.is_match(&text) {
            return (area, None, Some("ceiling"));
        }
        if _ONE_SURFACE.is_match(&text) && !_A_ROOM.is_match(&text) {
            return (area, None, Some("wall"));
        }
        let ceiling = _the_ceiling(&text).filter(|c| *c != 0.0).unwrap_or(*_DEFAULT_CEILING);
        return (Some(dup::py_round(run.unwrap() * ceiling, 2)), Some(ceiling), Some("room"));
    }
    (None, None, None)
}

/// The line as typed, or as tidied when there is nothing typed, lower-cased.
pub fn _the_line(understood: &Understood) -> String {
    sh::or(sh::or(&understood.clean, &understood.plain), &understood.text).to_lowercase()
}

/// (width, height) of the one wall the line gives, or (None, None).
pub fn _wall_width_and_height(understood: &Understood, memory: Option<&Memory>) -> (Option<f64>, Option<f64>) {
    let clean = _the_line(understood);
    let (mut text, mut carried) = (clean.clone(), false);
    let (a, r) = _measurements(&clean);
    if !(t(a) || t(r)) {
        let remembered = memory.and_then(|m| _the_size_said_before(Some(m), &clean));
        text = remembered.map(|(_, _, said)| said.to_lowercase()).unwrap_or_default();
        carried = true;
    }
    if text.is_empty() {
        return (None, None);
    }
    if !carried {
        if _A_CEILING_THING.is_match(&text) && _the_ceiling(&text).is_none() {
            return (None, None);
        }
        if !_ONE_SURFACE.is_match(&text) || _A_ROOM.is_match(&text) {
            return (None, None);
        }
    }
    for found in _SIDE_BY_SIDE.finditer(&text) {
        let first = _in_metres(found.str(1), found.get(2));
        let second = _in_metres(found.str(3), group_or(&found, 4, 2));
        if let (Some(first), Some(second)) = (first.filter(|v| *v != 0.0), second.filter(|v| *v != 0.0)) {
            if (0.3..=40.0).contains(&first) && (0.3..=40.0).contains(&second) {
                return (Some(dup::py_round(first, 2)), Some(dup::py_round(second, 2)));
            }
        }
    }
    for (pattern, height_first) in [(&**_A_LONG_AND_HIGH, false), (&**_A_HIGH_AND_LONG, true)] {
        let Some(found) = pattern.search(&text) else { continue };
        let first = _in_metres(found.str(1), found.get(2));
        let second = _in_metres(found.str(3), found.get(4));
        let (Some(first), Some(second)) = (first.filter(|v| *v != 0.0), second.filter(|v| *v != 0.0)) else { continue };
        let (width, height) = if height_first { (second, first) } else { (first, second) };
        if (0.3..=40.0).contains(&width) && (0.3..=40.0).contains(&height) {
            return (Some(dup::py_round(width, 2)), Some(dup::py_round(height, 2)));
        }
    }
    (None, None)
}

/// (area, run, the earlier line) when this line leaves the size out and the chat gave it.
pub fn _the_size_said_before(memory: Option<&Memory>, clean: &str) -> Option<(Option<f64>, Option<f64>, String)> {
    let (area, run) = _measurements(clean);
    if t(area) || t(run) {
        return None;
    }
    let turns = memory.map(|m| m.turns.as_slice()).unwrap_or(&[]);
    let start = turns.len().saturating_sub(4);
    for turn in turns[start..].iter().rev() {
        let said = &turn.user;
        if said.is_empty() {
            continue;
        }
        let (before, before_run) = _measurements(&said.to_lowercase());
        if t(before) || t(before_run) {
            return Some((before, before_run, text::strip(said).to_string()));
        }
    }
    None
}

/// A count row's reply.
pub type CountFn = fn(&Memory, &Understood) -> String;

/// `_sized_by_the_chat_before(reply)`'s wrapper: a follow-up that leaves the size out is sized
/// by the line before it.
#[allow(non_snake_case)]
pub fn _sized_by_the_chat_before__answered(reply: CountFn, memory: &Memory, understood: &Understood) -> String {
    if let Some((_, _, said)) = _the_size_said_before(Some(memory), &_the_line(understood)) {
        let mut under = understood.clone();
        under.clean = text::strip(&format!("{} {}", _the_line(understood), said)).to_string();
        return reply(memory, &under);
    }
    reply(memory, understood)
}

static _SURFACE_WORD: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs_of(g("_SURFACE_WORD")));

/// The word for the thing being measured, as the line named it, or the row's own default.
pub fn _the_surface(clean: &str, default: &str) -> String {
    for (word, said) in _SURFACE_WORD.iter() {
        if sh::re_cached(&format!(r"\b{word}\b"), 0).is_match(clean) {
            return said.clone();
        }
    }
    default.to_string()
}

/// The size of the tile or slab being laid, when the line names one: "600 by 600 tiles".
pub fn _a_lay_size(clean: &str) -> Option<(f64, f64)> {
    for found in _SIDE_BY_SIDE.finditer(clean) {
        let first = _in_metres(found.str(1), found.get(2));
        let second = _in_metres(found.str(3), group_or(&found, 4, 2));
        let (Some(mut first), Some(mut second)) = (first.filter(|v| *v != 0.0), second.filter(|v| *v != 0.0)) else { continue };
        let unit_said = found.get(2).is_some_and(|s| !s.is_empty()) || found.get(4).is_some_and(|s| !s.is_empty());
        if !unit_said && 1.5 < first && first <= 3000.0 {
            first /= 1000.0;
            second /= 1000.0;
        }
        if (0.05..=1.5).contains(&first) && (0.05..=1.5).contains(&second) {
            return Some((first, second));
        }
    }
    None
}

/// A tile or slab size said the way it is sold: 300mm by 300mm, 4.8m by 145mm.
pub fn _a_piece(size: (f64, f64)) -> String {
    [size.0, size.1]
        .iter()
        .map(|value| {
            if *value < 1.0 {
                format!("{}mm", _said(text::round_half_even(value * 1000.0) as f64))
            } else {
                format!("{}m", dup::py_g(*value))
            }
        })
        .collect::<Vec<_>>()
        .join(" by ")
}

/// A whole number of things to buy: you cannot buy part of a tile.
pub fn _up(value: f64) -> i64 {
    value.ceil() as i64
}

/// A whole count the way a person says it, without a needless ".0".
pub fn _said(value: f64) -> String {
    text::thousands(text::round_half_even(value))
}

/// The boxes of a dozen the tiles come to: "around 7 boxes of a dozen".
pub fn _boxes(tiles: i64) -> String {
    let boxes = _up(tiles as f64 / 12.0);
    let around = if boxes > 1 { "around" } else { "" };
    text::strip(&format!("{around} {} of a dozen", _counted(boxes as f64, "box", Some("boxes")))).to_string()
}

/// A count with its thing, in the number the sentence needs: "1 tile", "74 tiles".
pub fn _counted(count: f64, thing: &str, many: Option<&str>) -> String {
    if text::round_half_even(count) == 1 {
        return format!("1 {thing}");
    }
    let plural = match many {
        Some(m) if !m.is_empty() => m.to_string(),
        _ => format!("{thing}s"),
    };
    format!("{} {plural}", _said(count))
}

/// "Six square metres" - the area said back, so the arithmetic is the person's own.
pub fn _about_the_area(area: f64) -> String {
    if dup::py_round(area, 1) == 1.0 {
        return "One square metre".into();
    }
    if area != area.trunc() {
        return format!("{} square metres", dup::py_g(area));
    }
    format!("{} square metres", _said(area))
}

static IN_BOXES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:in|per|a|one) box\b|\bboxes\b", 0));

/// How many tiles a floor or a wall takes, and in how many boxes.
pub fn _how_many_tiles(_memory: &Memory, understood: &Understood) -> String {
    let clean = _the_line(understood);
    let (area, _) = _measurements(&clean);
    let size = _a_lay_size(&clean);
    let in_boxes = IN_BOXES.is_match(&clean);
    let area = area.filter(|a| *a != 0.0);
    if let (Some(area), Some(size)) = (area, size) {
        let count = area / (size.0 * size.1);
        let buy = _up(count * 1.1);
        return format!(
            "{} of {} tiles is {}, so buy {} for the cuts - {}. Line the cuts up in the corner nobody looks at and check a spare \
             tile is left over for a repair.",
            text::capitalize(&_about_the_area(area)),
            _a_piece(size),
            _counted(text::round_half_even(count) as f64, "tile", None),
            _counted(buy as f64, "tile", None),
            _boxes(buy)
        );
    }
    if let (Some(area), true) = (area, in_boxes) {
        return format!(
            "A box says the square metres it covers on the label, and most cover about one, \
             so {} is about {} boxes, or {} with the cuts. A 600mm tile comes three to the square \
             metre, so a box of those covers more and the count drops: read the m2 on the \
             box rather than the number of tiles in it.",
            _about_the_area(area),
            _said(_up(area) as f64),
            _said(_up(area * 1.1) as f64)
        );
    }
    let Some(area) = area else {
        if let Some(size) = size {
            let per = _up(1.0 / (size.0 * size.1));
            return format!(
                "A {} tile is about {} to the square metre, so add \
                 10% to the area for the cuts. How big is the wall or floor - length and \
                 width in metres?",
                _a_piece(size),
                _said(per as f64)
            );
        }
        if in_boxes {
            return "A box of tiles covers about a square metre - it says the m2 on the box - so \
                    the count follows from the tile size: a 300mm tile is about 11 to the square \
                    metre, and a 600mm one just under three. Buy 10% more than the area needs \
                    for the cuts at the edges. How big is the wall or floor?"
                .into();
        }
        return "A 300mm tile covers about a tenth of a square metre, so count about 11 to the \
                square metre and add 10% for cuts. How big is the wall or floor - length and \
                width in metres?"
            .into();
    };
    let count = area * 11.1;
    let buy = _up(count * 1.1);
    format!(
        "{} at about 11 tiles to the square metre is \
         {}, so buy {} to allow for cuts \
         - {}. A 600mm tile is just under three to the square metre, so that size \
         needs about a quarter of the count.",
        text::capitalize(&_about_the_area(area)),
        _counted(text::round_half_even(count) as f64, "tile", None),
        _counted(buy as f64, "tile", None),
        _boxes(buy)
    )
}

/// How many decking boards a deck takes.
pub fn _how_many_decking(_memory: &Memory, understood: &Understood) -> String {
    let clean = _the_line(understood);
    let (area, _) = _measurements(&clean);
    let Some(area) = area.filter(|a| *a != 0.0) else {
        return format!(
            "A 4.8m board of 145mm decking covers about 0.7 of a square metre, so a square \
             metre is about a board and a half. How big is the \
             {} - length and width in metres?",
            _the_surface(&clean, "deck")
        );
    };
    let count = area / 0.696;
    let buy = _up(count * 1.1);
    format!(
        "{} at about 0.7 of a square metre a board is \
         {} boards of 4.8m, so buy {} with the cuts. The joists \
         are the other count: at 400mm centres a 4m by 3m deck needs about 9 runs, and the \
         boards run across them with a 5mm gap.",
        text::capitalize(&_about_the_area(area)),
        _said(text::round_half_even(count) as f64),
        _said(buy as f64)
    )
}

static SAID_PANELS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d+)\s+(?:fence\s+|6ft\s+)?panels?\b", 0));

/// How many fence panels and posts a run takes.
pub fn _how_many_fence(_memory: &Memory, understood: &Understood) -> String {
    let clean = _the_line(understood);
    let (_, length) = _measurements(&clean);
    let length = length.filter(|l| *l != 0.0);
    if let (Some(found), None) = (SAID_PANELS.search(&clean), length) {
        let mut panels: i64 = found.str(1).parse().unwrap_or(0);
        if panels == 0 {
            panels = 1;
        }
        let run = panels as f64 * 1.83;
        return format!(
            "{} panels at 1.83m is {} metres of run, so you \
             want {} posts if it starts and ends on one. Set each post \
             600mm into the ground in concrete and put a gravel board under every panel so \
             the wood does not sit in the wet.",
            _said(panels as f64),
            _said(dup::py_round(run, 1)),
            _said((panels + 1) as f64)
        );
    }
    let Some(length) = length else {
        return "A standard 6ft fence panel is 1.83m wide, so 20 metres needs 11 panels and 12 \
                posts. How long is the run in metres?"
            .into();
    };
    let panels = _up(length / 1.83);
    format!(
        "{} metres at 1.83m for a standard 6ft panel is {} \
         panels, and {} posts if the run starts and ends on one. Put a \
         gravel board under each panel so the wood does not sit in the wet, and set the \
         posts 600mm into the ground in concrete.",
        _said(length),
        _said(panels as f64),
        _said((panels + 1) as f64)
    )
}

static BLOCKS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bblocks?\b", 0));

/// How many bricks or blocks a wall takes.
pub fn _how_many_bricks(_memory: &Memory, understood: &Understood) -> String {
    let clean = _the_line(understood);
    let (area, _) = _measurements(&clean);
    let blocks = BLOCKS.is_match(&clean);
    let Some(area) = area.filter(|a| *a != 0.0) else {
        if blocks {
            return "A concrete block is 440 by 215mm, so a square metre of single-skin wall is \
                    about 10 blocks. How big is the wall - length and height in metres?"
                .into();
        }
        return "A standard brick with its mortar joint covers about 60 to the square metre of \
                single-skin wall. How big is the wall - length and height in metres?"
            .into();
    };
    if blocks {
        let count = area * 10.0;
        return format!(
            "{} of single-skin wall at about 10 blocks to \
             the square metre is {} blocks, so order \
             {} with the waste. A cavity wall takes two skins, so \
             double it and add wall ties.",
            text::capitalize(&_about_the_area(area)),
            _said(text::round_half_even(count) as f64),
            _said(_up(count * 1.05) as f64)
        );
    }
    let count = area * 60.0;
    format!(
        "{} of single-skin wall at about 60 bricks to the \
         square metre is {} bricks, so order {} \
         with the waste - and double it for a cavity wall. A 25kg bag of mortar mix lays \
         about 25 bricks.",
        text::capitalize(&_about_the_area(area)),
        _said(text::round_half_even(count) as f64),
        _said(_up(count * 1.05) as f64)
    )
}

/// How many floorboards a room takes.
pub fn _how_many_floorboards(_memory: &Memory, understood: &Understood) -> String {
    let clean = _the_line(understood);
    let (area, _) = _measurements(&clean);
    let Some(area) = area.filter(|a| *a != 0.0) else {
        return "A 2.4m floorboard 120mm wide covers about 0.29 of a square metre, so a square \
                metre is about three and a half boards. How big is the room - length and width \
                in metres?"
            .into();
    };
    let count = area / 0.288;
    format!(
        "{} at about 0.29 of a square metre a board is \
         {} boards of 2.4m by 120mm, so buy {} for \
         cuts. Run them across the joists and leave an 8 to 10mm gap at each wall for \
         movement.",
        text::capitalize(&_about_the_area(area)),
        _said(text::round_half_even(count) as f64),
        _said(_up(count * 1.1) as f64)
    )
}

/// How many lengths of skirting a room takes.
pub fn _how_many_skirting(_memory: &Memory, understood: &Understood) -> String {
    let clean = _the_line(understood);
    let (_, length) = _measurements(&clean);
    let Some(length) = length.filter(|l| *l != 0.0) else {
        return "Skirting comes in 2.4m lengths, so add up the room's perimeter and divide: a 4m \
                by 3m room is 14m round, which is six lengths. How big is the room - length and \
                width in metres?"
            .into();
    };
    let lengths = _up(length / 2.4);
    format!(
        "{} metres round at 2.4m a length is {} lengths, \
         so buy {} to allow for the mitres and a mistake. Work the \
         perimeter out as twice (length + width), and add a length for each doorway you have \
         to cut round.",
        _said(length),
        _said(lengths as f64),
        _said((lengths + 1) as f64)
    )
}

static CEILING_OR_ROOF: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bceilings?\b|\broof\b", 0));

/// How many plasterboard sheets a wall or ceiling takes.
pub fn _how_many_plasterboard(_memory: &Memory, understood: &Understood) -> String {
    let clean = _the_line(understood);
    let (area, _) = _measurements(&clean);
    let Some(area) = area.filter(|a| *a != 0.0) else {
        return "A plasterboard sheet is 2.4m by 1.2m, so just under three square metres, and \
                you want about 10% more than the wall for the cuts and the joints. How big is \
                the wall or ceiling - length and width in metres?"
            .into();
    };
    let count = area / 2.88;
    let thick = if CEILING_OR_ROOF.is_match(&clean) { "12.5mm" } else { "9.5mm" };
    format!(
        "{} is {} sheets of 2.4m by 1.2m \
         {thick} plasterboard once it is covered, so buy {} \
         with the cuts at the edges and keep the offcut for a patch. Stagger the joints, \
         leave a 3mm gap between the sheets, and screwing it into the joists or studs holds \
         better than gluing it on its own.",
        text::capitalize(&_about_the_area(area)),
        _said(count),
        _said(_up(area * 1.1 / 2.88) as f64)
    )
}

/// The count reply a `_sized_by_the_chat_before` wrapper holds, by its id.
pub fn count_reply_by_id(id: &str) -> Option<CountFn> {
    Some(match id {
        "bot.knowledge.home:_how_many_tiles:8884" => _how_many_tiles,
        "bot.knowledge.home:_how_many_decking:8930" => _how_many_decking,
        "bot.knowledge.home:_how_many_fence:8946" => _how_many_fence,
        "bot.knowledge.home:_how_many_bricks:8972" => _how_many_bricks,
        "bot.knowledge.home:_how_many_floorboards:8996" => _how_many_floorboards,
        "bot.knowledge.home:_how_many_skirting:9011" => _how_many_skirting,
        "bot.knowledge.home:_how_many_plasterboard:9027" => _how_many_plasterboard,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------------------------
// The material is the subject of a count.

static _DOUBLE_DUTY: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    strs("_DOUBLE_DUTY").into_iter().map(|w| { let re = Re::new(&format!(r"\b{w}(?:s|es|ed|ing)?\b"), 0); (w, re) }).collect()
});
static _A_SUBJECT_OR_MODAL: LazyLock<HashSet<String>> = LazyLock::new(|| set("_A_SUBJECT_OR_MODAL"));
static _OURS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_OURS"));
static _A_JOB_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| set("_A_JOB_WORD"));
static _A_UNIT_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| set("_A_UNIT_WORD"));
static _A_CAME_AWAY: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_CAME_AWAY"));
static _A_LOOKING_BACK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_LOOKING_BACK"));
static HOLDER_OF_HELD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b([a-z]+)\s+of\s+([a-z]+)", 0));
static IN_A_HOLDER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bin\s+(?:a|an|the|one)\s+([a-z]+)", 0));

/// `ours(word)` inside `_the_material_is_the_subject`: the word is the material, allowing for a
/// plural.
#[allow(non_snake_case)]
pub fn _the_material_is_the_subject__ours(word: &str) -> bool {
    if _OURS.contains(word) {
        return true;
    }
    let n = word.chars().count();
    if word.ends_with('s') && n > 3 && _OURS.contains(&word[..word.len() - 1]) {
        return true;
    }
    word.ends_with("es") && n > 4 && _OURS.contains(&word[..word.len() - 2])
}

/// False when the material word in the line is doing another word's job.
pub fn _the_material_is_the_subject(understood: &Understood) -> bool {
    let line = sh::clean_or_text(understood).to_lowercase();
    if line.is_empty() {
        return true;
    }
    if _A_CAME_AWAY.iter().any(|p| line.contains(p.as_str())) {
        return false;
    }
    if _A_LOOKING_BACK.iter().any(|p| line.contains(p.as_str())) {
        return false;
    }
    for (_word, family) in _DOUBLE_DUTY.iter() {
        for found in family.finditer(&line) {
            let before = text::split_ref(&line[..found.start()]);
            match before.last() {
                Some(last) if _A_SUBJECT_OR_MODAL.contains(*last) => {}
                _ => continue,
            }
            let rest = format!("{} {}", &line[..found.start()], &line[found.end()..]);
            if !family.is_match(&rest) {
                return false;
            }
        }
    }
    let ours = _the_material_is_the_subject__ours;
    for found in HOLDER_OF_HELD.finditer(&line) {
        let (holder, held) = (found.str(1), found.str(2));
        if (_A_UNIT_WORD.contains(holder) || ours(holder)) && !ours(held) {
            return false;
        }
    }
    for found in IN_A_HOLDER.finditer(&line) {
        let holder = found.str(1);
        if _A_UNIT_WORD.contains(holder) || _A_JOB_WORD.contains(holder) || _OURS.contains(holder) {
            continue;
        }
        return false;
    }
    true
}

// ---------------------------------------------------------------------------------------------
// A fixing, and the bit that goes with it; a bed's size.

static PLUG_ANSWER: LazyLock<String> = LazyLock::new(|| string("PLUG_ANSWER"));
static SHELF_ANSWER: LazyLock<String> = LazyLock::new(|| string("SHELF_ANSWER"));
static NUT_ANSWER: LazyLock<String> = LazyLock::new(|| string("NUT_ANSWER"));
static _A_REPAIR_VERB: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_REPAIR_VERB"));
static _A_NEEDS_MENDING: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_NEEDS_MENDING"));
static _WASHER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_WASHER_WORDS"));
static _A_FIXING_TO_PUT_IT_ON: LazyLock<HashSet<String>> = LazyLock::new(|| set("_A_FIXING_TO_PUT_IT_ON"));
static _A_WASHER_AT_A_JOINT: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_WASHER_AT_A_JOINT"));

/// False for a washer that is a machine or a tap, which this row cannot answer.
pub fn _a_washer_is_the_fixing(understood: &Understood) -> bool {
    if !understood.words.iter().any(|w| _WASHER_WORDS.contains(w)) {
        return true;
    }
    understood.words.iter().any(|w| _A_FIXING_TO_PUT_IT_ON.contains(w))
}

/// False for a line about mending a thing, which is a job rather than a wall plug.
pub fn _a_repair_job_is_not_a_fixing(understood: &Understood) -> bool {
    let said = sh::clean_or_text(understood).to_lowercase();
    if _A_REPAIR_VERB.is_match(&said) || _A_NEEDS_MENDING.is_match(&said) {
        return false;
    }
    _a_washer_is_the_fixing(understood)
}

/// True unless the line says "washer" and never says what the washer goes into.
pub fn _a_washer_names_its_joint(understood: &Understood) -> bool {
    let said = sh::clean_or_text(understood).to_lowercase();
    let named = understood.words.iter().any(|w| w == "washer" || w == "washers");
    if !(named || said.contains("washer")) {
        return true;
    }
    _A_WASHER_AT_A_JOINT.is_match(&said)
}

/// The fixing row's guard: not a repair job, and a washer only with its joint.
pub fn _a_fixing_is_the_question(understood: &Understood) -> bool {
    _a_repair_job_is_not_a_fixing(understood) && _a_washer_names_its_joint(understood)
}

/// A plug and a drill bit first; a screw asked about for a shelf gets the shelf's answer.
pub fn _a_fixing(_memory: &Memory, understood: &Understood) -> String {
    let said = understood.clean.to_lowercase();
    let has = |set: &[&str]| understood.words.iter().any(|w| set.contains(&w.as_str()));
    let shelf = has(&["shelf", "shelves", "bookcase", "bookcase shelf"]) || said.contains("a shelf");
    let plug = ["plug", "drill bit", "masonry bit", "drill"].iter().any(|p| said.contains(p));
    if shelf && !plug {
        return SHELF_ANSWER.clone();
    }
    if has(&["nut", "nuts", "bolt", "bolts", "thread", "threads", "spanner", "spanners"]) && !plug {
        return NUT_ANSWER.clone();
    }
    PLUG_ANSWER.clone()
}

/// The four bed sizes, which is the whole question.
pub fn _a_bed_size(_memory: &Memory, _understood: &Understood) -> String {
    string("_BED_SIZES")
}

// ---------------------------------------------------------------------------------------------
// The registry's tables.

macro_rules! guards {
    ($($wrapper:ident => $f:path),* $(,)?) => {
        $(fn $wrapper(_f: &FnRef, u: &Understood) -> bool { $f(u) })*
    };
}
macro_rules! replies_str {
    ($($wrapper:ident => $f:path),* $(,)?) => {
        $(fn $wrapper(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some($f(m, u)) })*
    };
}
macro_rules! replies_opt {
    ($($wrapper:ident => $f:path),* $(,)?) => {
        $(fn $wrapper(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { $f(m, u) })*
    };
}

guards! {
    g_lambda_4069 => lambda_4069,
    g_a_dryer_is_not_a_washer => _a_dryer_is_not_a_washer,
    g_a_fence_is_named => _a_fence_is_named,
    g_a_fixing_is_the_question => _a_fixing_is_the_question,
    g_a_machine_consumable => _a_machine_consumable,
    g_a_mark_that_is_live => _a_mark_that_is_live,
    g_a_person_not_a_hob => _a_person_not_a_hob,
    g_a_room_being_heated => _a_room_being_heated,
    g_a_visit_not_a_purchase => _a_visit_not_a_purchase,
    g_asks_the_ceiling_colour => _asks_the_ceiling_colour,
    g_going_to_the_loo => _going_to_the_loo,
    g_not_a_boundary => _not_a_boundary,
    g_not_a_cook_ask => _not_a_cook_ask,
    g_not_a_length_of_stay => _not_a_length_of_stay,
    g_not_a_machine_that_has_gone => _not_a_machine_that_has_gone,
    g_not_a_pair_of_glasses => _not_a_pair_of_glasses,
    g_not_a_size_of_a_home => _not_a_size_of_a_home,
    g_not_only_a_white_mark => _not_only_a_white_mark,
    g_not_the_buying_question => _not_the_buying_question,
    g_start_craft_applies => _start_craft_applies,
    g_storm_damage => _storm_damage,
    g_the_black_is_not_a_body => _the_black_is_not_a_body,
    g_the_boiler_is_not_the_fault => _the_boiler_is_not_the_fault,
    g_the_material_is_the_subject => _the_material_is_the_subject,
    g_the_paper_is_in_the_line => _the_paper_is_in_the_line,
    g_the_space_in_the_house => _the_space_in_the_house,
    g_the_washers_drain_has_its_own_answer => _the_washers_drain_has_its_own_answer,
    g_not_a_sound => not_a_sound,
}

replies_str! {
    r_a_bed_size => _a_bed_size,
    r_a_chip_in_it => _a_chip_in_it,
    r_a_fixing => _a_fixing,
    r_a_wear => _a_wear,
    r_alarm_nuisance => _alarm_nuisance,
    r_appliance_cutting_out => _appliance_cutting_out,
    r_bath_length => _bath_length,
    r_boiler_noise => _boiler_noise,
    r_colour_advice => _colour_advice,
    r_consumer_unit => _consumer_unit,
    r_door_lock_stiff => _door_lock_stiff,
    r_dryer_heat => _dryer_heat,
    r_furniture_upstairs => _furniture_upstairs,
    r_heat_pump_answer => _heat_pump_answer,
    r_heating_goes_cold => _heating_goes_cold,
    r_hob_heat => _hob_heat,
    r_hot_water_fault => _hot_water_fault,
    r_how_many_tiles => _how_many_tiles,
    r_how_many_decking => _how_many_decking,
    r_how_many_fence => _how_many_fence,
    r_how_many_bricks => _how_many_bricks,
    r_how_many_floorboards => _how_many_floorboards,
    r_how_many_skirting => _how_many_skirting,
    r_how_many_plasterboard => _how_many_plasterboard,
    r_iron_fault => _iron_fault,
    r_keeping_paper => _keeping_paper,
    r_kettle_will_not_stop => _kettle_will_not_stop,
    r_lifespan => _lifespan,
    r_lost_keys => _lost_keys,
    r_machine_broke => _machine_broke,
    r_machine_fault => _machine_fault,
    r_machine_takes => _machine_takes,
    r_mould => _mould,
    r_mower => _mower,
    r_paint_brush_care => _paint_brush_care,
    r_pipes => _pipes,
    r_place_smell => _place_smell,
    r_pressure_weak => _pressure_weak,
    r_read_level => _read_level,
    r_read_pattern => _read_pattern,
    r_read_tape => _read_tape,
    r_repair_judgement => _repair_judgement,
    r_roof_leak => _roof_leak,
    r_small_appliance_not_heating => _small_appliance_not_heating,
    r_smoking_appliance => _smoking_appliance,
    r_soft_smell => _soft_smell,
    r_start_craft => _start_craft,
    r_washer_price => _washer_price,
    r_washer_sound_fault => _washer_sound_fault,
    r_water_off => _water_off,
    r_water_use => _water_use,
    r_where_the_consumable_goes => _where_the_consumable_goes,
    r_the_trade_to_call => the_trade_to_call,
}

replies_opt! {
    r_lambda_8271 => lambda_8271,
    r_a_service_worth => _a_service_worth,
    r_appliance_noise => _appliance_noise,
    r_broken_window => _broken_window,
    r_chemical_safety => _chemical_safety,
    r_diy => _diy,
    r_fitting_repair => _fitting_repair,
    r_home_word_2 => _home_word_2,
    r_machine_part => _machine_part,
    r_radiator_not_hot => _radiator_not_hot,
    r_run_out_basics => _run_out_basics,
    r_running => _running,
}

/// `_sized_by_the_chat_before(reply)`'s closure: the count reply held in the `reply` cell, run on
/// the line sized by the chat before when it leaves the size out.
fn r_sized_answered(f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    let cell = f.cell("reply")?.as_fn()?;
    if let Some(reply) = count_reply_by_id(&cell.id) {
        return Some(_sized_by_the_chat_before__answered(reply, m, u));
    }
    // a wrapped reply this module does not own: the registry runs it on the same reading
    let mut under = u.clone();
    if let Some((_, _, said)) = _the_size_said_before(Some(m), &_the_line(u)) {
        under.clean = text::strip(&format!("{} {}", _the_line(u), said)).to_string();
    }
    Some(crate::knowledge::registry::reply(cell, m, &under))
}

/// `the_filter_of(words, text)`'s closure, with its compiled patterns and text from the cells.
fn r_filter_of_reply(f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    let patterns: Vec<&Re> = f.cell("patterns")?.items().iter().filter_map(Value::as_re).collect();
    let text = f.cell("text")?.as_str()?;
    Some(the_filter_of__reply(&patterns, text, m, u))
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.home:<lambda>:4069", g_lambda_4069),
    ("bot.knowledge.home:_a_dryer_is_not_a_washer:1380", g_a_dryer_is_not_a_washer),
    ("bot.knowledge.home:_a_fence_is_named:3335", g_a_fence_is_named),
    ("bot.knowledge.home:_a_fixing_is_the_question:9373", g_a_fixing_is_the_question),
    ("bot.knowledge.home:_a_machine_consumable:4092", g_a_machine_consumable),
    ("bot.knowledge.home:_a_mark_that_is_live:2150", g_a_mark_that_is_live),
    ("bot.knowledge.home:_a_person_not_a_hob:2307", g_a_person_not_a_hob),
    ("bot.knowledge.home:_a_room_being_heated:3308", g_a_room_being_heated),
    ("bot.knowledge.home:_a_visit_not_a_purchase:3234", g_a_visit_not_a_purchase),
    ("bot.knowledge.home:_asks_the_ceiling_colour:3770", g_asks_the_ceiling_colour),
    ("bot.knowledge.home:_going_to_the_loo:1111", g_going_to_the_loo),
    ("bot.knowledge.home:_not_a_boundary:8343", g_not_a_boundary),
    ("bot.knowledge.home:_not_a_cook_ask:3571", g_not_a_cook_ask),
    ("bot.knowledge.home:_not_a_length_of_stay:4239", g_not_a_length_of_stay),
    ("bot.knowledge.home:_not_a_machine_that_has_gone:5181", g_not_a_machine_that_has_gone),
    ("bot.knowledge.home:_not_a_pair_of_glasses:8084", g_not_a_pair_of_glasses),
    ("bot.knowledge.home:_not_a_size_of_a_home:3316", g_not_a_size_of_a_home),
    ("bot.knowledge.home:_not_only_a_white_mark:6830", g_not_only_a_white_mark),
    ("bot.knowledge.home:_not_the_buying_question:7404", g_not_the_buying_question),
    ("bot.knowledge.home:_start_craft_applies:6287", g_start_craft_applies),
    ("bot.knowledge.home:_storm_damage:4034", g_storm_damage),
    ("bot.knowledge.home:_the_black_is_not_a_body:1835", g_the_black_is_not_a_body),
    ("bot.knowledge.home:_the_boiler_is_not_the_fault:3258", g_the_boiler_is_not_the_fault),
    ("bot.knowledge.home:_the_material_is_the_subject:9105", g_the_material_is_the_subject),
    ("bot.knowledge.home:_the_paper_is_in_the_line:3780", g_the_paper_is_in_the_line),
    ("bot.knowledge.home:_the_space_in_the_house:216", g_the_space_in_the_house),
    ("bot.knowledge.home:_the_washers_drain_has_its_own_answer:5407", g_the_washers_drain_has_its_own_answer),
    ("bot.knowledge.home:not_a_sound:3835", g_not_a_sound),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.home:<lambda>:8271", r_lambda_8271),
    ("bot.knowledge.home:_a_bed_size:9401", r_a_bed_size),
    ("bot.knowledge.home:_a_chip_in_it:7511", r_a_chip_in_it),
    ("bot.knowledge.home:_a_fixing:9377", r_a_fixing),
    ("bot.knowledge.home:_a_service_worth:7798", r_a_service_worth),
    ("bot.knowledge.home:_a_wear:4654", r_a_wear),
    ("bot.knowledge.home:_alarm_nuisance:2245", r_alarm_nuisance),
    ("bot.knowledge.home:_appliance_cutting_out:1751", r_appliance_cutting_out),
    ("bot.knowledge.home:_appliance_noise:1407", r_appliance_noise),
    ("bot.knowledge.home:_bath_length:2963", r_bath_length),
    ("bot.knowledge.home:_boiler_noise:3840", r_boiler_noise),
    ("bot.knowledge.home:_broken_window:3009", r_broken_window),
    ("bot.knowledge.home:_chemical_safety:229", r_chemical_safety),
    ("bot.knowledge.home:_colour_advice:2081", r_colour_advice),
    ("bot.knowledge.home:_consumer_unit:1942", r_consumer_unit),
    ("bot.knowledge.home:_diy:2508", r_diy),
    ("bot.knowledge.home:_door_lock_stiff:1535", r_door_lock_stiff),
    ("bot.knowledge.home:_dryer_heat:2407", r_dryer_heat),
    ("bot.knowledge.home:_fitting_repair:2901", r_fitting_repair),
    ("bot.knowledge.home:_furniture_upstairs:3552", r_furniture_upstairs),
    ("bot.knowledge.home:_heat_pump_answer:3969", r_heat_pump_answer),
    ("bot.knowledge.home:_heating_goes_cold:3901", r_heating_goes_cold),
    ("bot.knowledge.home:_hob_heat:2313", r_hob_heat),
    ("bot.knowledge.home:_home_word_2:8446", r_home_word_2),
    ("bot.knowledge.home:_hot_water_fault:7736", r_hot_water_fault),
    ("bot.knowledge.home:_how_many_bricks:8972", r_how_many_bricks),
    ("bot.knowledge.home:_how_many_decking:8930", r_how_many_decking),
    ("bot.knowledge.home:_how_many_fence:8946", r_how_many_fence),
    ("bot.knowledge.home:_how_many_floorboards:8996", r_how_many_floorboards),
    ("bot.knowledge.home:_how_many_plasterboard:9027", r_how_many_plasterboard),
    ("bot.knowledge.home:_how_many_skirting:9011", r_how_many_skirting),
    ("bot.knowledge.home:_how_many_tiles:8884", r_how_many_tiles),
    ("bot.knowledge.home:_iron_fault:2344", r_iron_fault),
    ("bot.knowledge.home:_keeping_paper:8204", r_keeping_paper),
    ("bot.knowledge.home:_kettle_will_not_stop:1784", r_kettle_will_not_stop),
    ("bot.knowledge.home:_lifespan:862", r_lifespan),
    ("bot.knowledge.home:_lost_keys:1524", r_lost_keys),
    ("bot.knowledge.home:_machine_broke:5209", r_machine_broke),
    ("bot.knowledge.home:_machine_fault:1728", r_machine_fault),
    ("bot.knowledge.home:_machine_part:5351", r_machine_part),
    ("bot.knowledge.home:_machine_takes:2721", r_machine_takes),
    ("bot.knowledge.home:_mould:1867", r_mould),
    ("bot.knowledge.home:_mower:5514", r_mower),
    ("bot.knowledge.home:_paint_brush_care:3603", r_paint_brush_care),
    ("bot.knowledge.home:_pipes:1896", r_pipes),
    ("bot.knowledge.home:_place_smell:266", r_place_smell),
    ("bot.knowledge.home:_pressure_weak:2158", r_pressure_weak),
    ("bot.knowledge.home:_radiator_not_hot:3475", r_radiator_not_hot),
    ("bot.knowledge.home:_read_level:6526", r_read_level),
    ("bot.knowledge.home:_read_pattern:6141", r_read_pattern),
    ("bot.knowledge.home:_read_tape:6508", r_read_tape),
    ("bot.knowledge.home:_repair_judgement:543", r_repair_judgement),
    ("bot.knowledge.home:_roof_leak:2117", r_roof_leak),
    ("bot.knowledge.home:_run_out_basics:4901", r_run_out_basics),
    ("bot.knowledge.home:_running:1172", r_running),
    ("bot.knowledge.home:_sized_by_the_chat_before.<locals>.answered:8799", r_sized_answered),
    ("bot.knowledge.home:_small_appliance_not_heating:2378", r_small_appliance_not_heating),
    ("bot.knowledge.home:_smoking_appliance:2195", r_smoking_appliance),
    ("bot.knowledge.home:_soft_smell:294", r_soft_smell),
    ("bot.knowledge.home:_start_craft:6295", r_start_craft),
    ("bot.knowledge.home:_washer_price:3366", r_washer_price),
    ("bot.knowledge.home:_washer_sound_fault:1371", r_washer_sound_fault),
    ("bot.knowledge.home:_water_off:1988", r_water_off),
    ("bot.knowledge.home:_water_use:4216", r_water_use),
    ("bot.knowledge.home:_where_the_consumable_goes:4159", r_where_the_consumable_goes),
    ("bot.knowledge.home:the_filter_of.<locals>.reply:3744", r_filter_of_reply),
    ("bot.knowledge.home:the_trade_to_call:3113", r_the_trade_to_call),
];
