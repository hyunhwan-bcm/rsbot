//! Port of PyBot's `bot/knowledge/home_craft.py`: the rows of the home-and-DIY domain (moved out
//! of `home.py` for size) and the guards and replies they name - a fixture that has stopped, a
//! boiler's own service, a fly that travels, a gauge, a wash, a washer that is a tap's, a machine
//! that will not empty, a chair and not a body, paint and wallpaper for a room, nails.
//!
//! `entries()` builds the rows at import time and its result is in the bundle (`home.ENTRIES`);
//! the lambdas inside it that serve as guards are ported here as `entries__lambda_<line>`.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g1 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::home;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn g(name: &str) -> &'static crate::data::Value {
    data::get(&format!("bot.knowledge.home_craft.{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.home_craft.{name}"))
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

/// `" ".join(words).lower()`, or the clean (or typed) line when the reading has no words.
fn words_or_line(u: &Understood) -> String {
    let text = u.words.join(" ").to_lowercase();
    if text.is_empty() { sh::clean_or_text(u).to_lowercase() } else { text }
}

fn lower_words(u: &Understood) -> HashSet<String> {
    u.words.iter().map(|w| w.to_lowercase()).collect()
}

// ---------------------------------------------------------------------------------------------
// A deposit, food in the fridge, a fixture that has stopped.

static _A_DEPOSIT_COMING_BACK: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_DEPOSIT_COMING_BACK"));

/// True when the deposit line is a tenancy one, not somebody saving towards a deposit.
pub fn _a_deposit_coming_back(understood: &Understood) -> bool {
    _A_DEPOSIT_COMING_BACK.is_match(&sh::squash_lower(sh::clean_or_text(understood)))
}

static _FOOD_OWN_SMELL: LazyLock<&'static Re> = LazyLock::new(|| rx("_FOOD_OWN_SMELL"));

/// False when the line is a food that has turned, with the fridge named as where it is.
pub fn _not_the_food_in_the_fridge(understood: &Understood) -> bool {
    !_FOOD_OWN_SMELL.is_match(&words_or_line(understood))
}

static STOPPED_ALTOGETHER: LazyLock<Vec<String>> = LazyLock::new(|| strs("STOPPED_ALTOGETHER"));
/// FIXTURE_FIRST_CHECKS with each name's `\b<name>s?\b` compiled once.
static FIXTURE_FIRST_CHECKS: LazyLock<Vec<(Vec<Re>, String)>> = LazyLock::new(|| {
    g("FIXTURE_FIRST_CHECKS")
        .items()
        .iter()
        .map(|r| {
            let names = r.items()[0].strs().iter().map(|n| Re::new(&format!(r"\b{}s?\b", sh::escape(n)), 0)).collect();
            (names, r.items()[1].str().to_string())
        })
        .collect()
});

/// The first check for the fixture a line says has stopped, or None.
pub fn _the_fixture_that_stopped(understood: &Understood) -> Option<String> {
    let said = words_or_line(understood);
    if said.is_empty() || !STOPPED_ALTOGETHER.iter().any(|w| said.contains(w.as_str())) {
        return None;
    }
    for (names, check) in FIXTURE_FIRST_CHECKS.iter() {
        if names.iter().any(|name| name.is_match(&said)) {
            return Some(check.clone());
        }
    }
    None
}

/// The first check for the fixture the line names, or None for a line that names none.
pub fn _a_fixture_that_stopped_was_the_question(understood: &Understood) -> bool {
    _the_fixture_that_stopped(understood).is_some()
}

/// The reply of every row `_the_stopped_fixture_entries` builds: the first check for the
/// fixture that has stopped.
#[allow(non_snake_case)]
pub fn _the_stopped_fixture_entries__lambda_171(_memory: &Memory, reading: &Understood) -> Option<String> {
    _the_fixture_that_stopped(reading)
}

static SERVICE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("SERVICE_WORDS"));

/// True when the line asks whether the yearly service is theirs to do.
pub fn _an_ask_about_servicing_my_own(understood: &Understood) -> bool {
    let words = lower_words(understood);
    let text = sh::clean_or_text(understood).to_lowercase();
    SERVICE_WORDS.iter().any(|w| words.contains(w)) || SERVICE_WORDS.iter().any(|w| text.contains(w.as_str()))
}

static _A_FLY_THAT_TRAVELS: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_FLY_THAT_TRAVELS"));

/// False when the line's "fly" is the verb of travelling, which is not a fly in the house.
pub fn _the_fly_is_not_a_verb(understood: &Understood) -> bool {
    !_A_FLY_THAT_TRAVELS.is_match(&words_or_line(understood))
}

static _A_BAR_WITH_A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_BAR_WITH_A_FIGURE"));
static _PRESSURE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_PRESSURE_WORDS"));

/// False for a bare "bar" that is not a measuring unit.
pub fn _a_gauge_reading(understood: &Understood) -> bool {
    let text = sh::clean_or_text(understood).to_lowercase();
    if lower_words(understood).iter().any(|w| _PRESSURE_WORDS.contains(w)) {
        return true;
    }
    _A_BAR_WITH_A_FIGURE.is_match(&text)
}

// ---------------------------------------------------------------------------------------------
// A wash, water leaving, a washer that is a tap's.

static _WILL_NOT_EMPTY: LazyLock<&'static Re> = LazyLock::new(|| rx("_WILL_NOT_EMPTY"));
static _WATER_LEAVING: LazyLock<&'static Re> = LazyLock::new(|| rx("_WATER_LEAVING"));
static A_WASH_NAMED: LazyLock<HashSet<String>> = LazyLock::new(|| set("A_WASH_NAMED"));
static A_TEMPERATURE_SAID: LazyLock<&'static Re> = LazyLock::new(|| rx("A_TEMPERATURE_SAID"));
static A_BARE_TEMPERATURE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_BARE_TEMPERATURE"));

/// True when the line is about a wash, which is the subject the temperature row is about.
pub fn _a_wash_is_named(understood: &Understood) -> bool {
    let text = words_or_line(understood);
    let flat = text.replace(',', " ");
    if text::split_ref(&flat).iter().any(|w| A_WASH_NAMED.contains(*w)) {
        return true;
    }
    if A_TEMPERATURE_SAID.is_match(&text) {
        return true;
    }
    let figures: HashSet<String> = A_BARE_TEMPERATURE.findall_str(&text).into_iter().collect();
    figures.len() > 1
}

/// True when the line has water somewhere it should not be, which is what the leak row answers.
pub fn _it_is_water_leaving(understood: &Understood) -> bool {
    _WATER_LEAVING.is_match(&words_or_line(understood))
}

static _A_WASHER: LazyLock<HashSet<String>> = LazyLock::new(|| set("_A_WASHER"));
static _ANOTHER_KIND_OF_WASHER: LazyLock<HashSet<String>> = LazyLock::new(|| set("_ANOTHER_KIND_OF_WASHER"));
static _THE_MACHINE_ITSELF: LazyLock<HashSet<String>> = LazyLock::new(|| set("_THE_MACHINE_ITSELF"));
static _A_WASHER_ALREADY_CHANGED: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_WASHER_ALREADY_CHANGED"));
static _A_WASHER_DONE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_WASHER_DONE"));
static _A_TAP_TALK: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_TAP_TALK"));

fn words_meet(u: &Understood, table: &HashSet<String>) -> bool {
    u.words.iter().any(|w| table.contains(w))
}

/// The washer is the answer, unless they have already changed it: then it is the seat.
pub fn _a_dripping_tap(_memory: &Memory, understood: &Understood) -> String {
    if _A_WASHER_ALREADY_CHANGED.is_match(sh::clean_or_text(understood)) {
        return string("_THE_NEXT_TAP_STEP");
    }
    string("_THE_DRIPPING_TAP")
}

/// False for a washer that is a tap's or a bolt's, which no machine row can answer.
pub fn _not_a_washer_of_another_kind(understood: &Understood) -> bool {
    if !words_meet(understood, &_A_WASHER) {
        return true;
    }
    if words_meet(understood, &_THE_MACHINE_ITSELF) {
        return true;
    }
    !words_meet(understood, &_ANOTHER_KIND_OF_WASHER)
}

/// True when "washer" is the whole subject of the line, so the answer has to hold both senses.
pub fn _a_washer_with_no_other_subject(understood: &Understood) -> bool {
    if !words_meet(understood, &_A_WASHER) {
        return false;
    }
    if words_meet(understood, &_THE_MACHINE_ITSELF) || words_meet(understood, &_ANOTHER_KIND_OF_WASHER) {
        return false;
    }
    !_A_WASHER_DONE.is_match(sh::clean_or_text(understood))
}

/// True when the last turn or two named a tap and nothing named the machine.
pub fn _the_chat_is_about_a_tap(memory: &Memory, understood: &Understood) -> bool {
    let said_now = text::strip(sh::text_or_clean(understood)).to_lowercase();
    let mut seen = 0;
    for turn in memory.turns.iter().rev() {
        let line = text::strip(&turn.user).to_lowercase();
        if line.is_empty() || line == said_now {
            continue;
        }
        if text::split_ref(&line).iter().any(|w| _THE_MACHINE_ITSELF.contains(*w)) {
            return false;
        }
        if _A_TAP_TALK.is_match(&line) {
            return true;
        }
        seen += 1;
        if seen >= 2 {
            break;
        }
    }
    false
}

/// The washer that a chat has already identified, or the two senses when it has not.
pub fn _a_washer_two_things(memory: &Memory, understood: &Understood) -> String {
    if _the_chat_is_about_a_tap(memory, understood) {
        return string("_THE_TAP_WASHER");
    }
    string("_WASHER_TWO_THINGS")
}

/// The leak row's guard, plus the other sense of "washer": a tap's, not the machine's.
pub fn _the_leak_is_a_machines(understood: &Understood) -> bool {
    _it_is_water_leaving(understood) && _not_a_washer_of_another_kind(understood)
}

/// True when the line names a machine that will not empty, which is what the drain row answers.
pub fn _a_machine_that_will_not_empty(understood: &Understood) -> bool {
    _WILL_NOT_EMPTY.is_match(&words_or_line(understood))
}

/// The bed for blocks (sharp sand alone) or for patio slabs (a 5:1 mix).
pub fn _the_bed_for_paving_or_blocks(_memory: &Memory, reading: &Understood) -> String {
    let line = sh::clean_or_text(reading).to_lowercase();
    let blocks = reading.words.iter().any(|w| matches!(w.as_str(), "block" | "blocks" | "setts" | "pavers"))
        || ["block paving", "block paved", "block paver", "block paving drive", "paved drive", "paving the drive"]
            .iter()
            .any(|p| line.contains(p));
    if blocks {
        return "Blocks are not bedded on a sand-and-cement mix: they go down on 30 to 40mm \
                of sharp sand alone, compacted with a wacker plate, and the joints are \
                brushed full of kiln-dried sand afterwards - so there is no cement to \
                measure. For 20 square metres reckon on about one tonne of sharp sand for \
                the bed and two or three 25kg bags of kiln-dried sand for the joints, plus \
                an edge restraint on any open side, which is what stops the blocks \
                spreading."
            .into();
    }
    "Patio slabs go on a full bed of 5:1 sharp sand and cement, 30 to 40mm thick, \
     never on five dabs: a 25kg bag of cement with five 25kg bags of sharp sand \
     beds about 2 to 2.5 square metres, so a 20 square metre patio wants nine or \
     ten bags of cement and two bulk bags of sand. The mix should be crumbly-damp \
     - squeeze a ball and no water runs - and the joints are pointed afterwards \
     with a 4:1 mix, not the bed mix."
        .into()
}

// ---------------------------------------------------------------------------------------------
// A chair, a fan, the kitchen stuff, the heat pump explainer, a hair dryer.

static _LOOSE_STOOL: LazyLock<&'static Re> = LazyLock::new(|| rx("_LOOSE_STOOL"));
static _FURNITURE_WORDS: LazyLock<&'static Re> = LazyLock::new(|| rx("_FURNITURE_WORDS"));

/// False when the line is the looseness of what a body passes, rather than a wobbly chair.
pub fn _a_chair_and_not_a_body(understood: &Understood) -> bool {
    if crate::knowledge::body::the_stomach_is_what_it_is_about(understood) {
        return false;
    }
    let text = sh::clean_or_text(understood).to_lowercase();
    !(_LOOSE_STOOL.is_match(&text) && !_FURNITURE_WORDS.is_match(&text))
}

static FAN_OR_BLADE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:fan|fans|blade|blades)\b", 0));

/// The answer for the fan that will not sit still, or for a plain appliance that rocks.
pub fn _the_fan_stopped(_memory: &Memory, understood: &Understood) -> String {
    let text = sh::clean_or_text(understood).to_lowercase();
    if FAN_OR_BLADE.is_match(&text) {
        return "A fan that rocks is usually the base or a bent blade: tighten the nut under the \
                base, then take the blade off and sight along it, because one blade out of line \
                is the whole wobble and a new blade is a few pounds. Stand it on a hard, level \
                floor rather than a rug, and wind out the feet if it has them."
            .into();
    }
    "A wobble is a loose base or an uneven floor rather than a fault: wind the feet out \
     until it sits flat on all of them, tighten what holds the base to the body, and take \
     it off a rug or a soft board, which is what most of them are standing on. If it is \
     still rocking it wants shimming on one corner."
        .into()
}

static _PACKING_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| set("_PACKING_VERB"));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", 0));

/// The bare "kitchen stuff" line answers only when it takes up a line before it.
pub fn _the_kitchen_stuff_is_a_take_up(understood: &Understood) -> bool {
    let said = sh::clean_or_text(understood).to_lowercase();
    if LETTERS.findall_str(&said).iter().any(|w| _PACKING_VERB.contains(w)) {
        return true;
    }
    understood.followup
}

static _HEAT_PUMP_USE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HEAT_PUMP_USE"));
static _HEAT_PUMP_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HEAT_PUMP_ASK"));

/// False when the line is about a heat pump appliance, not about what a heat pump is.
pub fn _the_explainer_keeps_the_explainer(understood: &Understood) -> bool {
    let text = format!(" {} ", sh::clean_or_text(understood).to_lowercase());
    if !_HEAT_PUMP_USE.iter().any(|w| text.contains(w.as_str())) {
        return true;
    }
    !_HEAT_PUMP_ASK.iter().any(|w| text.contains(w.as_str()))
}

/// The grille for a dryer that blows cold, the flex for one that is dead.
pub fn _hair_dryer(_memory: &Memory, reading: &Understood) -> String {
    const DEAD: &[&str] = &[
        "dead", "stopped", "stop", "wont", "won't", "wont.", "turn", "turns", "start", "starts", "switch",
        "switches", "power", "fuse", "buzzing", "buzzes", "nothing",
    ];
    if lower_words(reading).iter().any(|w| DEAD.contains(&w.as_str())) {
        return "A hair dryer that is completely dead is nearly always the plug or the flex, not \
                the element: try another socket, check the fuse in the plug, and look at the \
                point where the flex meets the handle, because a cable that is wound up every \
                day is what breaks. If a fresh fuse blows again, stop using it - that is a short \
                in the element, and a new dryer is cheaper than the repair."
            .into();
    }
    "Check the air intake first: a hair dryer that blows cold usually has the grille at \
     the back furred up with hair and fluff, and the thermal cut-out trips before the \
     element ever fails. Unplug it, pull the hair off the grille and out of the vent, let \
     it cool for ten minutes and try it again on the high setting. If it still runs cold \
     it is the element or the cut-out, and on a cheap dryer that is a new one rather than \
     a repair - do not open the case, the element is a live part."
        .into()
}

// ---------------------------------------------------------------------------------------------
// Paint and wallpaper for the room in the line.

/// A figure a person would say out loud: 4.46 -> 4.5, 2.0 -> 2, 1 -> 1.
pub fn _to_a_figure(value: f64) -> f64 {
    dup::py_round(value, 1)
}

/// The figure with its noun agreed with it: "1 litre", "2.5 litres", "1 square metre".
pub fn _counted(value: f64, unit: &str) -> String {
    let noun = if _to_a_figure(value) == 1.0 { unit.to_string() } else { format!("{unit}s") };
    format!("{} {noun}", dup::py_g(value))
}

/// The litres for the wall or room in the line, or the method when the line has no size.
pub fn _paint_for_the_room(memory: &Memory, understood: &Understood) -> String {
    let (area, ceiling, what) = home::_wall_area(understood, Some(memory));
    let Some(area) = area.filter(|a| *a != 0.0) else { return string("PAINT_WITH_NO_SIZE") };
    // rounded up to the next half litre, never down
    let litres = f64::max(1.0, (area * 2.0 / 11.0 * 2.0 + 0.999).trunc() / 2.0);
    let area = _to_a_figure(area);
    let coats = dup::py_g(_to_a_figure(area * 2.0));
    let count = if what == Some("ceiling") {
        format!(
            "A ceiling that size is {}, two coats makes {coats}, and a litre of \
             emulsion covers 10 to 12.",
            _counted(area, "square metre")
        )
    } else if let Some(ceiling) = ceiling {
        format!(
            "A room that size is {} round and {} to the ceiling, so about {} of wall, and \
             two coats makes {coats}.",
            _counted(_to_a_figure(area / ceiling), "metre"),
            _counted(ceiling, "metre"),
            _counted(area, "square metre")
        )
    } else {
        format!(
            "That's {}, two coats makes {coats}, and a litre of emulsion \
             covers 10 to 12.",
            _counted(area, "square metre")
        )
    };
    let tail = if what == Some("ceiling") {
        "Give the walls their own sum, and go a coat further when a white ceiling is \
         going over a colour rather than over white."
    } else if ceiling.is_none() {
        "Allow a third coat when a dark colour goes over a light one, and give the ceiling \
         and any second colour their own sum."
    } else {
        "Take off roughly 1.7 square metres for a door and 1.2 for a window if they are in \
         it, allow a third coat when a dark colour goes over a light one, and give the \
         ceiling and any second colour their own sum."
    };
    format!("About {}: {count} {tail}", _counted(litres, "litre"))
}

/// The rolls for the room or the wall in the line, or the method when the line has no size.
pub fn _wallpaper_for_the_room(memory: &Memory, understood: &Understood) -> String {
    let (area, ceiling, what) = home::_wall_area(understood, Some(memory));
    if !area.is_some_and(|a| a != 0.0) {
        return string("WALLPAPER_WITH_NO_SIZE");
    }
    let area = area.unwrap();
    let int_floordiv = |a: f64, b: f64| dup::py_floordiv(a, b) as i64;
    if what == Some("wall") {
        let (width, height) = home::_wall_width_and_height(understood, Some(memory));
        let (Some(width), Some(height)) = (width.filter(|w| *w != 0.0), height.filter(|h| *h != 0.0)) else {
            return string("WALLPAPER_WITH_NO_SIZE");
        };
        let across = dup::py_ceil(width / 0.52);
        let plain = i64::max(1, int_floordiv(10.0, height));
        let patterned = i64::max(1, int_floordiv(10.0, height + 0.3));
        let plain_rolls = dup::py_ceil(across as f64 / plain as f64);
        let patterned_rolls = dup::py_ceil(across as f64 / patterned as f64);
        return format!(
            "About {}: a wall {} wide and {} high takes {} at the 52cm width of a standard \
             roll, and a 10m roll gives {} of a plain paper or {} of one with a big repeat - \
             so {} plain and {} patterned. Buy one extra for a mistake and matching, and keep \
             the same batch number on every roll.",
            _counted(plain_rolls as f64, "roll"),
            _counted(_to_a_figure(width), "metre"),
            _counted(_to_a_figure(height), "metre"),
            _counted(across as f64, "drop"),
            _counted(plain as f64, "drop"),
            _counted(patterned as f64, "drop"),
            _counted(plain_rolls as f64, "roll"),
            _counted(patterned_rolls as f64, "roll")
        );
    }
    let Some(ceiling) = ceiling.filter(|c| *c != 0.0 && what == Some("room")) else {
        return string("WALLPAPER_WITH_NO_SIZE");
    };
    let round_the_room = area / ceiling;
    let drops = dup::py_ceil(round_the_room / 0.52);
    let plain = i64::max(1, int_floordiv(10.0, ceiling));
    let patterned = i64::max(1, int_floordiv(10.0, ceiling + 0.3));
    let rolls_plain = dup::py_ceil(drops as f64 / plain as f64);
    let rolls_patterned = dup::py_ceil(drops as f64 / patterned as f64);
    format!(
        "About {rolls_plain} rolls: a room that size is {} metres round and {} metres to the ceiling, so \
         {drops} drops at the 52cm width of a standard roll, and a 10m roll gives {plain} drops of a \
         plain paper or {patterned} of one with a big repeat - so {rolls_plain} rolls plain and {rolls_patterned} patterned. Buy \
         one extra for a mistake and matching, and keep the same batch number on every roll.",
        dup::py_g(round_the_room),
        dup::py_g(ceiling)
    )
}

// ---------------------------------------------------------------------------------------------
// Nails, a child in a move, drying the washing.

static _A_NAIL_COMES_OUT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_NAIL_COMES_OUT"));
static _A_SIZE_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_SIZE_ASK"));
static _A_SPLIT_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_SPLIT_ASK"));

/// How to get an old nail out of a wall, or how to put a new one in so it holds.
pub fn _a_nail_in_the_wall(_memory: &Memory, reading: &Understood) -> String {
    let words = lower_words(reading);
    if _A_NAIL_COMES_OUT.iter().any(|w| words.contains(w)) {
        return "Grip the head with pincers and slide a scrap of wood under them so the plaster \
                does not crush, then ease it out with a slight twist rather than levering \
                against the wall. A nail that turns but will not come has lost its grip: punch \
                it below the surface with another nail, fill the dent with filler, sand it flat \
                and touch up the paint."
            .into();
    }
    "Tap it in at a slight downward angle so the weight hangs on the wall instead of \
     pulling the head out, and put it into something solid - a stud, found by tapping, \
     or brick under the plaster. On plasterboard with nothing behind it a bare nail or \
     a picture hook pulls out: use a plasterboard plug or a self-drilling fixing, and \
     keep the load light."
        .into()
}

/// What size of nail to buy, or how to drive one without splitting the board.
pub fn _the_wood_and_the_nail(_memory: &Memory, reading: &Understood) -> String {
    let words = lower_words(reading);
    let said = format!(" {} ", reading.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" "));
    if _A_SIZE_ASK.iter().any(|a| said.contains(a.as_str())) && !_A_SPLIT_ASK.iter().any(|w| words.contains(w)) {
        return "Size the nail to the job, not to the room: 40mm to 50mm lost-head or oval nails \
                for skirting and architrave, 50mm to 65mm for a floorboard, 30mm for a 6mm \
                batten, and thin panel pins for beading and a back panel. As a rule the nail \
                wants to be about two and a half times as long as the piece it is holding, and \
                the top third of it has to bite into something solid behind."
            .into();
    }
    "Drill a pilot hole first, about three quarters as wide as the nail, and keep it at \
     least 25mm in from the end of the board, because that is where a split starts. \
     Blunt the point by tapping it once on a hard surface so it cuts the grain instead \
     of wedging it apart, and clamp a scrap of wood behind the far face when the board \
     is thin. A split that has already opened wants wood glue worked into it and a \
     clamp overnight, not another nail."
        .into()
}

static _A_CHILD_IN_A_MOVE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_CHILD_IN_A_MOVE"));
static _A_TELLING_THE_LINE_ASKS_FOR: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_TELLING_THE_LINE_ASKS_FOR"));

/// True when the line names a child, or asks for the words to say about the move.
pub fn _a_child_or_a_telling(understood: &Understood) -> bool {
    let line = sh::or(sh::or(&understood.norm_text, &understood.plain), &understood.text);
    let text = sh::squash_lower(line);
    _A_CHILD_IN_A_MOVE.is_match(&text) || _A_TELLING_THE_LINE_ASKS_FOR.is_match(&text)
}

static _DRY_IT_INDOORS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DRY_IT_INDOORS"));
static _DRYING_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("_DRYING_WORD"));
static _THE_WEATHER: LazyLock<&'static Re> = LazyLock::new(|| rx("_THE_WEATHER"));
static _A_LOAD_NAMED: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_LOAD_NAMED"));
static _A_ROOM: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_ROOM"));
static _WEATHER_NOT_A_ROOM: LazyLock<&'static Re> = LazyLock::new(|| rx("_WEATHER_NOT_A_ROOM"));

/// False when the line is about drying the washing indoors rather than in the weather.
pub fn _not_a_drying_inside_question(understood: &Understood) -> bool {
    let text = format!(" {} ", sh::clean_or_text(understood).to_lowercase());
    if !_DRYING_WORD.is_match(&text) {
        return true;
    }
    if _A_LOAD_NAMED.is_match(&text) && _A_ROOM.is_match(&text) {
        return _WEATHER_NOT_A_ROOM.is_match(&text);
    }
    if !_DRY_IT_INDOORS.iter().any(|w| text.contains(w.as_str())) {
        return true;
    }
    _THE_WEATHER.is_match(&text)
}

// ---------------------------------------------------------------------------------------------
// The guards written inline in `entries()`.

static OR_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bor\b[^.?]*\bfirst\b|\bfirst\b[^.?]*\bor\b", crate::py::re::I));

/// The volume-of-fill row stands down for a sanding job ("do i need to sand the wall").
#[allow(non_snake_case)]
pub fn entries__lambda_4841(u: &Understood) -> bool {
    !sh::a_sanding_job(u)
}

/// The sand-before-painting row stands down for "which comes first" ("sand or fill first").
#[allow(non_snake_case)]
pub fn entries__lambda_4889(u: &Understood) -> bool {
    !OR_FIRST.is_match(sh::clean_or_text(u))
}

/// The furniture-repair row stands down when the stomach is what the line is about.
#[allow(non_snake_case)]
pub fn entries__lambda_6500(understood: &Understood) -> bool {
    !crate::knowledge::body::the_stomach_is_what_it_is_about(understood)
}

/// A thing's part that hurts: the part is a thing's, and the line is not the stomach.
#[allow(non_snake_case)]
pub fn entries__lambda_6518(understood: &Understood) -> bool {
    crate::knowledge::body::_a_thing_owns_the_part(understood) && !crate::knowledge::body::the_stomach_is_what_it_is_about(understood)
}

/// The night-tariff row stands down for a line that asks where to get something.
#[allow(non_snake_case)]
pub fn entries__lambda_6932(understood: &Understood) -> bool {
    !dup::asks_where(understood)
}

/// The heat-a-room row: a room (not a body part) being heated, and not a heated argument.
#[allow(non_snake_case)]
pub fn entries__lambda_7175(u: &Understood) -> bool {
    home::_a_room_being_heated(u) && sh::not_the_heat_of_an_argument(u)
}

/// The first-night-box row stands down when the stomach is what the line is about.
#[allow(non_snake_case)]
pub fn entries__lambda_8289(understood: &Understood) -> bool {
    !crate::knowledge::body::the_stomach_is_what_it_is_about(understood)
}

/// The kitchen-bin row stands down for the recycle bin on a computer.
#[allow(non_snake_case)]
pub fn entries__lambda_10042(read: &Understood) -> bool {
    !dup::the_bin_is_on_a_machine(read)
}

// ---------------------------------------------------------------------------------------------
// The registry's tables.

macro_rules! guards {
    ($($wrapper:ident => $f:path),* $(,)?) => {
        $(fn $wrapper(_f: &FnRef, u: &Understood) -> bool { $f(u) })*
    };
}

guards! {
    g_a_chair_and_not_a_body => _a_chair_and_not_a_body,
    g_a_child_or_a_telling => _a_child_or_a_telling,
    g_a_deposit_coming_back => _a_deposit_coming_back,
    g_a_fixture_that_stopped_was_the_question => _a_fixture_that_stopped_was_the_question,
    g_a_gauge_reading => _a_gauge_reading,
    g_a_machine_that_will_not_empty => _a_machine_that_will_not_empty,
    g_a_wash_is_named => _a_wash_is_named,
    g_a_washer_with_no_other_subject => _a_washer_with_no_other_subject,
    g_an_ask_about_servicing_my_own => _an_ask_about_servicing_my_own,
    g_not_a_drying_inside_question => _not_a_drying_inside_question,
    g_not_a_washer_of_another_kind => _not_a_washer_of_another_kind,
    g_not_the_food_in_the_fridge => _not_the_food_in_the_fridge,
    g_the_explainer_keeps_the_explainer => _the_explainer_keeps_the_explainer,
    g_the_fly_is_not_a_verb => _the_fly_is_not_a_verb,
    g_the_kitchen_stuff_is_a_take_up => _the_kitchen_stuff_is_a_take_up,
    g_the_leak_is_a_machines => _the_leak_is_a_machines,
    g_lambda_4841 => entries__lambda_4841,
    g_lambda_4889 => entries__lambda_4889,
    g_lambda_6500 => entries__lambda_6500,
    g_lambda_6518 => entries__lambda_6518,
    g_lambda_6932 => entries__lambda_6932,
    g_lambda_7175 => entries__lambda_7175,
    g_lambda_8289 => entries__lambda_8289,
    g_lambda_10042 => entries__lambda_10042,
}

fn r_a_dripping_tap(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_a_dripping_tap(m, u))
}
fn r_a_nail_in_the_wall(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_a_nail_in_the_wall(m, u))
}
fn r_a_washer_two_things(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_a_washer_two_things(m, u))
}
fn r_hair_dryer(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_hair_dryer(m, u))
}
fn r_paint_for_the_room(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_paint_for_the_room(m, u))
}
fn r_the_fan_stopped(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_the_fan_stopped(m, u))
}
fn r_stopped_fixture_lambda_171(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _the_stopped_fixture_entries__lambda_171(m, u)
}
fn r_the_wood_and_the_nail(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_the_wood_and_the_nail(m, u))
}
fn r_wallpaper_for_the_room(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_wallpaper_for_the_room(m, u))
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.home_craft:_a_chair_and_not_a_body:527", g_a_chair_and_not_a_body),
    ("bot.knowledge.home_craft:_a_child_or_a_telling:823", g_a_child_or_a_telling),
    ("bot.knowledge.home_craft:_a_deposit_coming_back:23", g_a_deposit_coming_back),
    ("bot.knowledge.home_craft:_a_fixture_that_stopped_was_the_question:155", g_a_fixture_that_stopped_was_the_question),
    ("bot.knowledge.home_craft:_a_gauge_reading:242", g_a_gauge_reading),
    ("bot.knowledge.home_craft:_a_machine_that_will_not_empty:472", g_a_machine_that_will_not_empty),
    ("bot.knowledge.home_craft:_a_wash_is_named:318", g_a_wash_is_named),
    ("bot.knowledge.home_craft:_a_washer_with_no_other_subject:406", g_a_washer_with_no_other_subject),
    ("bot.knowledge.home_craft:_an_ask_about_servicing_my_own:187", g_an_ask_about_servicing_my_own),
    ("bot.knowledge.home_craft:_not_a_drying_inside_question:873", g_not_a_drying_inside_question),
    ("bot.knowledge.home_craft:_not_a_washer_of_another_kind:388", g_not_a_washer_of_another_kind),
    ("bot.knowledge.home_craft:_not_the_food_in_the_fridge:46", g_not_the_food_in_the_fridge),
    ("bot.knowledge.home_craft:_the_explainer_keeps_the_explainer:589", g_the_explainer_keeps_the_explainer),
    ("bot.knowledge.home_craft:_the_fly_is_not_a_verb:223", g_the_fly_is_not_a_verb),
    ("bot.knowledge.home_craft:_the_kitchen_stuff_is_a_take_up:564", g_the_kitchen_stuff_is_a_take_up),
    ("bot.knowledge.home_craft:_the_leak_is_a_machines:467", g_the_leak_is_a_machines),
    ("bot.knowledge.home_craft:entries.<locals>.<lambda>:4841", g_lambda_4841),
    ("bot.knowledge.home_craft:entries.<locals>.<lambda>:4889", g_lambda_4889),
    ("bot.knowledge.home_craft:entries.<locals>.<lambda>:6500", g_lambda_6500),
    ("bot.knowledge.home_craft:entries.<locals>.<lambda>:6518", g_lambda_6518),
    ("bot.knowledge.home_craft:entries.<locals>.<lambda>:6932", g_lambda_6932),
    ("bot.knowledge.home_craft:entries.<locals>.<lambda>:7175", g_lambda_7175),
    ("bot.knowledge.home_craft:entries.<locals>.<lambda>:8289", g_lambda_8289),
    ("bot.knowledge.home_craft:entries.<locals>.<lambda>:10042", g_lambda_10042),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.home_craft:_a_dripping_tap:380", r_a_dripping_tap),
    ("bot.knowledge.home_craft:_a_nail_in_the_wall:756", r_a_nail_in_the_wall),
    ("bot.knowledge.home_craft:_a_washer_two_things:460", r_a_washer_two_things),
    ("bot.knowledge.home_craft:_hair_dryer:597", r_hair_dryer),
    ("bot.knowledge.home_craft:_paint_for_the_room:646", r_paint_for_the_room),
    ("bot.knowledge.home_craft:_the_fan_stopped:540", r_the_fan_stopped),
    ("bot.knowledge.home_craft:_the_stopped_fixture_entries.<locals>.<lambda>:171", r_stopped_fixture_lambda_171),
    ("bot.knowledge.home_craft:_the_wood_and_the_nail:781", r_the_wood_and_the_nail),
    ("bot.knowledge.home_craft:_wallpaper_for_the_room:699", r_wallpaper_for_the_room),
];


// ---------------------------------------------------------------------------------------------
// The row builders.

use crate::data::Value;
use crate::knowledge::built;
use crate::knowledge::entry::{Entry, Reply};

/// `_the_stopped_fixture_entries(entry)`: one row per fixture, not one row for the family.
pub fn _the_stopped_fixture_entries() -> Vec<Entry> {
    let stopped = data::get("bot.knowledge.home_craft.STOPPED_ALTOGETHER").strs();
    let not_decorating = data::get("bot.knowledge.home_craft.A_FIXTURE_IS_NOT_A_DECORATING_JOB").clone();
    data::get("bot.knowledge.home_craft.FIXTURE_FIRST_CHECKS")
        .items()
        .iter()
        .map(|row| {
            let names = row.items()[0].strs();
            built::entry(
                &format!("a-fixture-that-stopped-{}", names[0].replace(' ', "-")),
                vec![built::part(&names), built::part(&stopped)],
                Reply::Fn(built::function("bot.knowledge.home_craft:_the_stopped_fixture_entries.<locals>.<lambda>:171")),
                &[not_decorating.clone()],
                3,
                vec![],
                Some(built::function("bot.knowledge.home_craft:_a_fixture_that_stopped_was_the_question:155")),
            )
        })
        .collect()
}

/// `entries()`: the rows `home.py` builds its list from, in their original order. Python's
/// function is the ten-thousand-line list of row literals home.py puts in front of its own rows
/// (`ENTRIES[:] = _craft_entries() + ENTRIES`); the rows are data, so they are read from the
/// exported `home.ENTRIES` - every row in front of home.py's own first row, "sewage-coming-up".
pub fn entries() -> Vec<Entry> {
    let rows: &Value = data::get("bot.knowledge.home.ENTRIES");
    rows.items()
        .iter()
        .enumerate()
        .map(|(i, v)| Entry::from_value(i, v))
        .take_while(|row| row.id != "sewage-coming-up")
        .collect()
}

#[cfg(test)]
mod factory_tests {
    use super::*;

    #[test]
    fn rows_rebuild() {
        data::load();
        let craft = entries();
        // PyBot: len(home_craft.entries()) == 355, from "stair-gate" to "a-nail-in-a-wall"
        assert_eq!(craft.len(), 355);
        assert_eq!(craft[0].id, "stair-gate");
        assert_eq!(craft[craft.len() - 1].id, "a-nail-in-a-wall");
        let ours = _the_stopped_fixture_entries();
        for mine in &ours {
            let theirs = craft.iter().find(|r| r.id == mine.id).expect("a fixture row");
            assert_eq!(built::same_entry(mine, theirs), None);
        }
        assert!(!ours.is_empty());
    }
}
