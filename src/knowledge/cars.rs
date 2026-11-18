//! Port of PyBot's `bot/knowledge/cars.py`: cars and driving, the everyday jobs and warning signs
//! (road charges, tickets, dash lights, oil, tyres, MOTs, repair costs, driving procedures).
#![allow(non_snake_case)] // closures are named `<factory>__<inner>`

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g6 as dup;
use crate::knowledge::d1_shared::{self as sh, ShapeFn};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

const M: &str = "bot.knowledge.cars.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
/// A string global of this module.
fn s(name: &str) -> String {
    g(name).str().to_string()
}

macro_rules! strs {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($name)).strs());)*
    };
}
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> = LazyLock::new(|| rx(stringify!($name)));)*
    };
}

strs!(
    _ROAD_CHARGE_MISS_WORDS,
    _ROAD_CHARGE_OTHER_CITIES,
    _ROAD_CHARGE_PRICE_WORDS,
    _ROAD_CHARGE_PAY_WORDS,
    _ROAD_CHARGE_WHO_WORDS,
    MODELS,
    ALL_ON_THE_ROAD,
    _STRUCK,
    _SELLING_BILL,
    _A_HOUSE_SURFACE,
    _A_CAR_ONLY,
    _THE_LINE_NAMES_THE_TEST,
    LEARNING,
    CAR_WEAR,
    HOME_SURFACE,
    GARAGE_TRADE,
    HOUSE_TRADE,
    _OIL_COST_WORDS,
    _OIL_FILTER_WORDS,
    _OIL_GRADE_WORDS,
    _OIL_JOB_WORDS,
    _OIL_OVERFILL_WORDS,
    _OIL_AMOUNT_WORDS,
    _OIL_CHECK_WORDS,
    _OIL_INTERVAL_WORDS,
    TO_A_TEST,
    LEGAL_ASK,
    _A_BICYCLE,
    A_CAR_IS_NAMED,
    WRONG_WITH_IT,
    _A_DEFINITION_ASK,
);
regexes!(_SELLING_RIVAL, _A_DRIVING_WORD, _SOMEBODY_ELSES_HOME, _A_SEAT_OR_A_CHILD);

static _DASH_LIGHT_TEXTS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    g("_DASH_LIGHT_TEXTS").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
});

/// The text the turns before wrote, by key ("user" or "bot"), for the last `n` turns.
fn recent_turns(memory: &Memory, n: usize) -> &[crate::state::Turn] {
    let turns = &memory.turns;
    &turns[turns.len().saturating_sub(n)..]
}

/// The one road-charge card the line asked for: a price, a rule, a payment, a penalty or a city.
pub fn _road_charge(_memory: &Memory, reading: &Understood) -> String {
    let said = sh::squash_lower(&reading.clean);
    let any = |words: &[&str]| words.iter().any(|w| said.contains(w));
    if any(&["toll", "dart charge", "dartford crossing", "m6 toll"]) {
        return s("TOLL");
    }
    if any(&["motorbike", "motorbikes", "motorcycle", "motorcycles", "moped", "mopeds", "scooter", "scooters"]) {
        return s("ROAD_CHARGE_BIKES");
    }
    if any(&["24 hours", "24 hour", "all day", "all night", "overnight", "at night", "through the night"]) {
        return s("ROAD_CHARGE_NOT_ALL_DAY");
    }
    if sh::any_in(&said, &_ROAD_CHARGE_OTHER_CITIES) {
        return s("ROAD_CHARGE_CAZ");
    }
    if sh::any_in(&said, &_ROAD_CHARGE_MISS_WORDS) {
        return s("ROAD_CHARGE_MISSED");
    }
    if sh::any_in(&said, &_ROAD_CHARGE_PAY_WORDS) {
        return s("ROAD_CHARGE_HOW_TO_PAY");
    }
    if sh::any_in(&said, &_ROAD_CHARGE_PRICE_WORDS) {
        if said.contains("ulez") && !said.contains("congestion charge") {
            return s("ROAD_CHARGE_ULEZ_PRICE");
        }
        return s("ROAD_CHARGE_PRICE");
    }
    if any(&["exempt", "exemption", "exemptions", "exempt from"]) {
        return s("ROAD_CHARGE_EXEMPT");
    }
    if sh::any_in(&said, &_ROAD_CHARGE_WHO_WORDS) {
        return s("ROAD_CHARGE_WHO_PAYS");
    }
    if said.contains("ulez") && !said.contains("congestion charge") {
        return s("ROAD_CHARGE_ULEZ_PRICE");
    }
    s("ROAD_CHARGE")
}

/// The parking ticket, defined first when the line asks what one is.
pub fn _parking_ticket(_memory: &Memory, reading: &Understood) -> String {
    let said = sh::squash_lower(&reading.clean);
    let words = text::split_ref(&said);
    if reading.asks.iter().any(|a| a == "definition") && !["my", "our"].iter().any(|w| words.contains(w)) {
        return s("PARKING_TICKET_IS");
    }
    s("PARKING_TICKET")
}

/// The dash cam card: buying it (the card), what it is, or where it goes and how it is wired.
pub fn _dash_cam(_memory: &Memory, understood: &Understood) -> String {
    let text = format!("{} {}", understood.text, understood.clean).to_lowercase();
    let any = |words: &[&str]| words.iter().any(|w| text.contains(w));
    if any(&["card", "memory", "sd", "size", "buy", "best", "recommend", "which", "worth"]) {
        return s("DASH_CAM_BUY");
    }
    if any(&["what is", "whats a", "what are", "explain", "used for", "do they do", "how does it work"]) {
        return s("DASH_CAM_WHAT");
    }
    s("DASH_CAM_FIT")
}

/// The shape: a road animal and a verb that says it met a car, in the line's words.
pub fn _an_animal_struck(line: &str) -> bool {
    let said = format!(" {} ", line.to_lowercase());
    if !ALL_ON_THE_ROAD.iter().any(|a| said.contains(&format!(" {a} "))) {
        return false;
    }
    _STRUCK.iter().any(|c| said.contains(&format!(" {c} ")))
}

/// False when the conversation is on a bicycle and this line is a follow-up to it.
pub fn _not_a_bike_chat(understood: &Understood) -> bool {
    let above = understood.leans_on.as_ref().map(|l| l.text.to_lowercase()).unwrap_or_default();
    if !above.contains("cable") {
        return true;
    }
    if !["bike", "bicycle", "cycle", "pushbike"].iter().any(|w| above.contains(w)) {
        return true;
    }
    let said = sh::clean_or_text(understood).to_lowercase();
    ["car", "cars", "van", "mot", "motor", "engine", "vehicle", "lorry", "truck"].iter().any(|w| said.contains(w))
}

/// False when the line is choosing between selling the car and keeping or mending it.
pub fn _the_selling_is_the_decision(understood: &Understood) -> bool {
    let text = format!(" {} ", sh::or(&understood.clean, &understood.plain));
    if sh::any_in(&text, &_SELLING_BILL) {
        return false;
    }
    if !(text.contains(" or ") || text.contains("should i")) {
        return true;
    }
    !_SELLING_RIVAL.is_match(&text)
}

/// False when the line is about being out at night rather than about driving in it.
pub fn _a_car_is_in_the_line(understood: &Understood) -> bool {
    let words = understood.words.join(" ");
    // `raw` is not a key of the reading, so it is None and skipped like any empty part.
    let parts: Vec<&str> = [understood.text.as_str(), understood.clean.as_str(), words.as_str()]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect();
    let said = parts.join(" ").to_lowercase();
    if said.trim().is_empty() {
        return true;
    }
    if _A_DRIVING_WORD.is_match(&said) {
        return true;
    }
    let padded = format!(" {said} ");
    _A_CAR_ONLY.iter().chain(MODELS.iter()).any(|part| padded.contains(&format!(" {part} ")))
}

/// True when the line typed itself names the MOT or handing the car over, not just the car.
pub fn _the_line_names_the_test(understood: &Understood) -> bool {
    let mut said = sh::text_or_clean(understood).to_string();
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    let said = format!(" {} ", said.to_lowercase().replace(',', " ").replace('.', " "));
    sh::any_in(&said, &_THE_LINE_NAMES_THE_TEST)
}

/// False when the line is a room that mists up rather than a car that leaks.
pub fn _a_room_not_a_car(understood: &Understood) -> bool {
    let said = format!(" {} ", understood.text.to_lowercase());
    if _A_CAR_ONLY.iter().any(|p| said.contains(&format!(" {p} "))) {
        return true;
    }
    !_A_HOUSE_SURFACE.iter().any(|p| said.contains(&format!(" {p} ")))
}

/// True unless the line's "home" is where somebody lives or is being looked after.
pub fn _a_machine_and_not_a_person(understood: &Understood) -> bool {
    let words = understood.words.join(" ");
    let said = sh::or(&understood.text, &words);
    !_SOMEBODY_ELSES_HOME.is_match(said)
}

/// True when this conversation is someone learning to drive rather than a car fault.
pub fn _learning(memory: &Memory) -> bool {
    recent_turns(memory, 6).iter().any(|t| sh::any_in(&t.user.to_lowercase(), &LEARNING))
}

/// A car that stalls by itself; a learner's stall is a different question.
pub fn _stalling(memory: &Memory, _understood: &Understood) -> Option<String> {
    if _learning(memory) {
        return None;
    }
    Some(
        "A car that stalls when you stop is usually a dirty throttle body, a vacuum leak or an idle control \
         fault, and the engine light often comes on with it. It is not usually dangerous but it is worth a \
         garage soon, because it can leave you without power steering or brakes at a junction."
            .to_string(),
    )
}

/// How long brake pads last, or what they cost, by what the line asked.
pub fn _brakes_cost(memory: &Memory, understood: &Understood) -> String {
    let mut line = format!("{} {} {}", understood.clean, understood.norm_text, understood.text).to_lowercase();
    if line.is_empty() {
        if let Some(last) = memory.turns.last() {
            line = last.user.to_lowercase();
        }
    }
    let asked_life = ["last", "mileage", "life", "how often", "how long", "how many miles"].iter().any(|w| line.contains(w));
    if asked_life {
        return "Front pads usually last 25,000 to 50,000 miles, or about three to five years of ordinary \
                driving, and rears last far longer - up to twice that. Town driving, towing and a heavy right \
                foot wear them sooner, and a full car makes it worse. What ends their life is the friction \
                material, not the miles: a wear indicator light, a squeal that starts when you brake, or a pedal \
                that needs more travel means have them checked within a few hundred miles. Fitted, front pads \
                are usually £100 to £200, and discs add £80 to £150 a pair."
            .to_string();
    }
    "Front pads fitted are usually £100 to £200 and discs add £80 to £150 a pair, so a front set with discs is \
     roughly £200 to £350; rears last longer and cost a bit less. Pads are a wear item, so this is normal \
     upkeep rather than a fault, and any garage can quote you in five minutes."
        .to_string()
}

/// The pressures, or the interval first for a line that asks how often to check them.
pub fn _tyre_pressure_reply(_memory: &Memory, understood: &Understood) -> String {
    if dup::asks_how_often(understood) {
        return s("TYRE_PRESSURE_FREQUENCY");
    }
    s("TYRE_PRESSURE")
}

/// A bare "abs" is the brake system when the line says a light, a dash or a car.
pub fn _the_abs_is_in_a_car(understood: &Understood) -> bool {
    static ABS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\babs\b", 0));
    static CAR_SIDE: LazyLock<Re> = LazyLock::new(|| {
        Re::new(
            r"\b(?:light|lights|lit|warning|symbol|dash|dashboard|cars?|brake|brakes|braking|mot|garage|wheel|wheels|sensor|comes?|came|flashing|flashes|flickering)\b",
            0,
        )
    });
    let text = sh::clean_or_text(understood).to_lowercase();
    if !ABS.is_match(&text) {
        return true;
    }
    CAR_SIDE.is_match(&text)
}

/// True when nothing in the line is the house's job and the garage is the reading left.
pub fn _the_trade_is_named(understood: &Understood) -> bool {
    if understood.topic.as_deref() == Some("cars") {
        return true;
    }
    let said = format!("{} {} {}", understood.clean, understood.norm_text, understood.plain).to_lowercase();
    if _words_name_the_trade(&said) {
        return true;
    }
    let leaned = understood.leans_on.as_ref().map(|l| l.text.as_str()).unwrap_or("");
    if !leaned.is_empty() && _words_name_the_trade(leaned) {
        return true;
    }
    !(_names_the_house(&said) || _names_the_house(leaned))
}

/// True when these words name something in the house that a trade repairs.
pub fn _names_the_house(text: &str) -> bool {
    let said = format!(" {} ", text::squash(text));
    HOUSE_TRADE.iter().any(|w| said.contains(&format!(" {w} ")))
}

/// True when these words on their own name a car, a part, a garage, or a car's damage.
pub fn _words_name_the_trade(text: &str) -> bool {
    let said = format!(" {} ", text::squash(text));
    let has = |w: &String| said.contains(&format!(" {w} "));
    let house = HOME_SURFACE.iter().any(has);
    if !house && CAR_WEAR.iter().any(has) {
        return true;
    }
    GARAGE_TRADE.iter().any(has)
}

/// The light the line named, or the general warning-light answer if none of them match.
pub fn _dash_light_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::the_line(understood);
    for (words, reply) in _DASH_LIGHT_TEXTS.iter() {
        if sh::any_in(&said, words) {
            return reply.clone();
        }
    }
    "Most dash lights are a warning to check something rather than a breakdown: an amber one means get it \
     looked at soon, a red one means stop when it is safe. Tell me which symbol it is, or what shape it has, \
     and I will say what it stands for."
        .to_string()
}

/// The oil answer: the change interval, the amount it takes, or the dipstick, by the ask.
pub fn _oil_reply(memory: &Memory, understood: &Understood) -> String {
    let said = sh::the_line(understood);
    if sh::any_in(&said, &_OIL_COST_WORDS) {
        return s("OIL_COST");
    }
    if sh::any_in(&said, &_OIL_FILTER_WORDS) {
        return s("OIL_FILTER");
    }
    if sh::any_in(&said, &_OIL_GRADE_WORDS) {
        return s("OIL_GRADE");
    }
    if sh::any_in(&said, &_OIL_JOB_WORDS) {
        return s("OIL_JOB");
    }
    if sh::any_in(&said, &_OIL_OVERFILL_WORDS) {
        return s("OIL_OVERFILL");
    }
    if sh::any_in(&said, &_OIL_AMOUNT_WORDS) {
        return s("OIL_AMOUNT");
    }
    if sh::any_in(&said, &_OIL_CHECK_WORDS) {
        if dup::asks_how_often_text(&said) {
            return s("OIL_CHECK_FREQUENCY");
        }
        return s("OIL_LEVEL");
    }
    if sh::any_in(&said, &_OIL_INTERVAL_WORDS) {
        return s("OIL_INTERVAL");
    }
    // A follow-up that only adds a detail is the same question again.
    if let Some(last) = memory.turns.last() {
        for answer in [s("OIL_INTERVAL"), s("OIL_AMOUNT")] {
            if last.bot.contains(answer.split('.').next().unwrap_or("")) {
                return answer;
            }
        }
    }
    s("OIL_LEVEL")
}

/// The seat or the child has to be in the line, or in the line it follows.
pub fn _a_seat_is_what_it_is_about(understood: &Understood) -> bool {
    let line = understood.words.join(" ");
    if _A_SEAT_OR_A_CHILD.is_match(&line) {
        return true;
    }
    let lean = understood.leans_on.as_ref().map(|l| l.words.join(" ")).unwrap_or_default();
    _A_SEAT_OR_A_CHILD.is_match(&lean)
}

/// A lapsed MOT, answered from the half of it the person asked about.
pub fn _no_mot(_memory: &Memory, understood: &Understood) -> String {
    let line = sh::clean_or_text(understood).to_lowercase();
    if sh::any_in(&line, &TO_A_TEST) && sh::any_in(&line, &LEGAL_ASK) {
        return s("NO_MOT_TO_A_TEST");
    }
    s("NO_MOT_FINE")
}

/// What a car tyre costs, or "" when the person has been talking about a bicycle.
pub fn _tyre_price(memory: &Memory, _understood: &Understood) -> String {
    let said = recent_turns(memory, 4).iter().map(|t| t.user.as_str()).collect::<Vec<_>>().join(" ").to_lowercase();
    if sh::any_in(&said, &_A_BICYCLE) {
        return String::new();
    }
    s("_TYRE_PRICE")
}

/// The fail sheet, with the clock in front of it when the line asks how long there is.
pub fn _mot_fail_reply(_memory: &Memory, understood: &Understood) -> String {
    if understood.asks.iter().any(|a| a == "how_long" || a == "how_much") {
        return format!("{}{}", s("MOT_FAIL_CLOCK"), s("MOT_FAIL_REPLY"));
    }
    s("MOT_FAIL_REPLY")
}

/// The repair-or-let-it-go decision, said in the person's own number when they gave one.
pub fn _money_against_value(_memory: &Memory, understood: &Understood) -> String {
    let figures: Vec<i64> = understood
        .numbers
        .iter()
        .filter_map(|n| n.value.as_ref())
        .map(|v| v.as_f64())
        .filter(|v| *v >= 150.0)
        .map(|v| v.trunc() as i64)
        .collect();
    if figures.len() == 1 {
        let figure = figures[0];
        return format!(
            "£{} of work is the number to judge it by: if the car would sell for less than about £{}, the repair \
             is the bigger number, so let it go. Scrap value is only £150 to £400, and a 'dangerous' MOT item like \
             brakes, steering or rust near a mounting is not worth fixing on a cheap car; tyres, a clutch or a \
             cambelt on an otherwise sound car usually are.",
            text::thousands(figure),
            text::thousands(figure * 2)
        );
    }
    "Money against value: if the quote is more than the car is worth, or more than half of it in one go on an \
     old car, let it go. A 'dangerous' MOT item like brakes, steering or rust near a mounting is not worth \
     fixing on a cheap car; tyres, a clutch or a cambelt on an otherwise sound car usually are. Scrap value is \
     only £150 to £400."
        .to_string()
}

/// The car-transfer gate: a vehicle, a model or a part only a car has is named in the line.
pub fn _a_car_is_named(understood: &Understood) -> bool {
    let read = _words_of_a_car_line(understood);
    // Python joins the *set* of words, whose order is per-process; first-seen order is used here
    // (SET ORDER: only the multi-word makes - "land rover", "range rover", "alfa romeo" - can see it).
    let mut words: Vec<&str> = Vec::new();
    for w in &read {
        if !words.contains(&w.as_str()) {
            words.push(w);
        }
    }
    let joined = words.join(" ");
    A_CAR_IS_NAMED.iter().any(|word| words.contains(&word.as_str()) || (word.contains(' ') && joined.contains(word.as_str())))
}

/// The line's words for the car gate: lower-cased, anything but letters, digits and % blanked.
pub fn _words_of_a_car_line(read: &Understood) -> Vec<String> {
    static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9% ]", 0));
    let clean = sh::clean_or_text(read).to_lowercase();
    let clean = NOT_A_WORD.sub(" ", &clean);
    text::split(&clean)
}

/// True when the line names something wrong with the car (a fault, a cost, a crash).
pub fn _says_something_is_wrong(understood: &Understood) -> bool {
    let line = format!(" {}", sh::clean_or_text(understood).to_lowercase());
    sh::any_in(&line, &WRONG_WITH_IT)
}

/// `_how_to(reply)`'s closure: the procedure, unless the line says the car is doing something wrong.
pub fn _how_to__answer(f: &FnRef, _memory: &Memory, understood: &Understood) -> Option<String> {
    if _says_something_is_wrong(understood) {
        return None;
    }
    f.cell("reply").and_then(Value::as_str).map(str::to_string)
}

/// A click on the steering is a CV joint fault, unless the line asks what the part is.
pub fn _the_click_on_a_turn(_memory: &Memory, understood: &Understood) -> String {
    let clean = sh::or(sh::clean_or_text(understood), &understood.plain).to_lowercase();
    if sh::any_in(&clean, &_A_DEFINITION_ASK) {
        return s("_CV_JOINT_DEFINITION");
    }
    s("_THE_CLICK_ON_A_TURN_REPLY")
}

/// The overheating guard: false when a pet, not an engine, is what is hot in the car.
pub fn lambda_2019(understood: &Understood) -> bool {
    let clean = understood.clean.as_str();
    !["dog", "dogs", "puppy", "pup", "cat", "cats", "kitten", "pet", "pets"].iter().any(|w| clean.contains(w))
}

// --- registry wrappers ------------------------------------------------------------------------

fn g_2019(_f: &FnRef, u: &Understood) -> bool {
    lambda_2019(u)
}
fn g_a_car_is_in_the_line(_f: &FnRef, u: &Understood) -> bool {
    _a_car_is_in_the_line(u)
}
fn g_a_car_is_named(_f: &FnRef, u: &Understood) -> bool {
    _a_car_is_named(u)
}
fn g_a_machine_and_not_a_person(_f: &FnRef, u: &Understood) -> bool {
    _a_machine_and_not_a_person(u)
}
fn g_a_room_not_a_car(_f: &FnRef, u: &Understood) -> bool {
    _a_room_not_a_car(u)
}
fn g_a_seat_is_what_it_is_about(_f: &FnRef, u: &Understood) -> bool {
    _a_seat_is_what_it_is_about(u)
}
fn g_not_a_bike_chat(_f: &FnRef, u: &Understood) -> bool {
    _not_a_bike_chat(u)
}
fn g_the_abs_is_in_a_car(_f: &FnRef, u: &Understood) -> bool {
    _the_abs_is_in_a_car(u)
}
fn g_the_line_names_the_test(_f: &FnRef, u: &Understood) -> bool {
    _the_line_names_the_test(u)
}
fn g_the_selling_is_the_decision(_f: &FnRef, u: &Understood) -> bool {
    _the_selling_is_the_decision(u)
}
fn g_the_trade_is_named(_f: &FnRef, u: &Understood) -> bool {
    _the_trade_is_named(u)
}

fn r_brakes_cost(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_brakes_cost(m, u))
}
fn r_dash_cam(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_dash_cam(m, u))
}
fn r_dash_light_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_dash_light_reply(m, u))
}
fn r_how_to(f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _how_to__answer(f, m, u)
}
fn r_money_against_value(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_money_against_value(m, u))
}
fn r_mot_fail_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_mot_fail_reply(m, u))
}
fn r_no_mot(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_no_mot(m, u))
}
fn r_oil_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_oil_reply(m, u))
}
fn r_parking_ticket(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_parking_ticket(m, u))
}
fn r_road_charge(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_road_charge(m, u))
}
fn r_stalling(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _stalling(m, u)
}
fn r_the_click_on_a_turn(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_the_click_on_a_turn(m, u))
}
fn r_tyre_pressure_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_tyre_pressure_reply(m, u))
}
fn r_tyre_price(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_tyre_price(m, u))
}

fn shape_an_animal_struck(_f: &FnRef, line: &str) -> bool {
    _an_animal_struck(line)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.cars:<lambda>:2019", g_2019),
    ("bot.knowledge.cars:_a_car_is_in_the_line:446", g_a_car_is_in_the_line),
    ("bot.knowledge.cars:_a_car_is_named:4511", g_a_car_is_named),
    ("bot.knowledge.cars:_a_machine_and_not_a_person:530", g_a_machine_and_not_a_person),
    ("bot.knowledge.cars:_a_room_not_a_car:487", g_a_room_not_a_car),
    ("bot.knowledge.cars:_a_seat_is_what_it_is_about:990", g_a_seat_is_what_it_is_about),
    ("bot.knowledge.cars:_not_a_bike_chat:355", g_not_a_bike_chat),
    ("bot.knowledge.cars:_the_abs_is_in_a_car:728", g_the_abs_is_in_a_car),
    ("bot.knowledge.cars:_the_line_names_the_test:471", g_the_line_names_the_test),
    ("bot.knowledge.cars:_the_selling_is_the_decision:388", g_the_selling_is_the_decision),
    ("bot.knowledge.cars:_the_trade_is_named:857", g_the_trade_is_named),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.cars:_brakes_cost:559", r_brakes_cost),
    ("bot.knowledge.cars:_dash_cam:297", r_dash_cam),
    ("bot.knowledge.cars:_dash_light_reply:912", r_dash_light_reply),
    ("bot.knowledge.cars:_how_to.<locals>.answer:4730", r_how_to),
    ("bot.knowledge.cars:_money_against_value:1055", r_money_against_value),
    ("bot.knowledge.cars:_mot_fail_reply:1047", r_mot_fail_reply),
    ("bot.knowledge.cars:_no_mot:999", r_no_mot),
    ("bot.knowledge.cars:_oil_reply:924", r_oil_reply),
    ("bot.knowledge.cars:_parking_ticket:184", r_parking_ticket),
    ("bot.knowledge.cars:_road_charge:143", r_road_charge),
    ("bot.knowledge.cars:_stalling:549", r_stalling),
    ("bot.knowledge.cars:_the_click_on_a_turn:4853", r_the_click_on_a_turn),
    ("bot.knowledge.cars:_tyre_pressure_reply:718", r_tyre_pressure_reply),
    ("bot.knowledge.cars:_tyre_price:1022", r_tyre_price),
];

pub const SHAPES: &[(&str, ShapeFn)] = &[("bot.knowledge.cars:_an_animal_struck:348", shape_an_animal_struck)];


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

    /// Python: bot/knowledge/cars.py:398 `_entry(name, keys, reply, avoid=())` (never called on the trace corpus)
    pub fn _entry(name: &str, keys: &Value, reply: &str, avoid: &Value) -> Value {
        crate::unported!("bot.knowledge.cars:_entry")
    }

    /// Python: bot/knowledge/cars.py:4727 `_how_to(reply)` (never called on the trace corpus)
    pub fn _how_to(reply: &str) -> Value {
        crate::unported!("bot.knowledge.cars:_how_to")
    }

}

