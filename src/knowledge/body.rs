//! Port of PyBot's `bot/knowledge/body.py`: a part of the body that hurts ("my <part> hurts"),
//! with the guards that keep a thing's part, somebody else's part or a figure of speech off the
//! person's own ache, and the replies for the everyday complaints the region rows lead to.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared::{self as sh, ShapeFn};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as re, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Module globals from the bundle, by their Python name.
macro_rules! global {
    ($name:ident: re) => {
        static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.knowledge.body.", stringify!($name))));
    };
    ($name:ident: set) => {
        static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.knowledge.body.", stringify!($name))));
    };
    ($name:ident: strs) => {
        static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.knowledge.body.", stringify!($name))));
    };
    ($name:ident: string) => {
        static $name: LazyLock<String> =
            LazyLock::new(|| data::string(concat!("bot.knowledge.body.", stringify!($name))));
    };
    ($name:ident: map) => {
        static $name: LazyLock<HashMap<String, String>> =
            LazyLock::new(|| data::str_map(concat!("bot.knowledge.body.", stringify!($name))));
    };
}

global!(TEMPERATURE_WORDS: set);
global!(WE_OUR_OURS: set);
global!(POSSESSIVE_OF_A_PERSON: set);
global!(THING_AFTER_THE_WORD: set);
global!(OTHER_PEOPLE: set);
global!(_WORKSHOP: re);
global!(_CANNOT_USE_IT: re);
global!(_THE_PERSONS_OWN_WORDS: set);
global!(_SOMEONE_ELSE: set);
global!(_WEEKS_ALONG: re);
global!(_SWELLING: re);
global!(_PREGNANCY_WORDS: strs);
global!(JOINTS: strs);
global!(PART_ONE: map);
global!(CLOT_AND_CHEST: strs);
global!(HOT_FLAGS: strs);
global!(SEE_TODAY_BACK_AND_CALF: string);
global!(SPELLED_OUT: strs);
global!(BACK_NAMES: strs);
global!(NUMB_ANYWHERE: strs);
global!(NUMB_LOWER: strs);
global!(SADDLE_PARTS: strs);
global!(FLAG_WITH_IT: strs);
global!(SADDLE_CARD: string);
global!(RED_FLAG_CARD: string);
global!(LOWER_HALF: strs);
global!(SOMEWHERE_ELSE: strs);
global!(_PIERCING_TROUBLE: re);
global!(_PIERCING_BUMP: re);
global!(_PIERCING_PIECE: re);
global!(_PIERCING_NOT_TROUBLE: re);
global!(_CARTILAGE_PIERCING: re);
global!(_PIERCING_ROUTINE: string);
global!(_A_BECOMING_A_DENTIST: re);
global!(_A_PRACTICE_ERRAND: re);
global!(_A_LIST_THAT_IS_LONG: re);
global!(_A_LENGTH_OF_A_VISIT: re);
global!(_A_FILLING: string);
global!(_A_ROOT_CANAL: string);
global!(_A_HYGIENIST: string);
global!(_A_CROWN_VISIT: string);
global!(_AN_IMPLANT: string);
global!(_THE_VISIT_IN_ORDER: strs);
global!(_THE_BOOKING: string);
global!(_THE_QUEUE: set);
global!(_REGISTERING_A_DENTIST: string);
global!(TOOTH_WORDS: set);
global!(SEEN_ABOUT_IT: set);
global!(_DECISION_ASK: re);
global!(_A_RETURN_TO: strs);
global!(_THE_BOOKING_WORDS: set);
global!(SEA_WATER_ALARM: re);
global!(SEA_WATER_ALARM_REPLY: string);
global!(OBJECT_OWNERS: set);
global!(THING_PARTS: set);
global!(_DETERMINERS: set);
global!(HURT_WORDS: set);
global!(_POSSESSIVES: set);
global!(_PERSON_OWNER: set);
global!(_ALSO_THE_BACK: set);
global!(_BACK_IS_A_NOUN_AFTER: set);
global!(_BACK_THEIRS: set);
global!(_BACK_VERB_BEFORE: set);
global!(_BACK_DESCRIBED: set);
global!(_BACK_IS_A_RETURN: set);
global!(_BACK_CLAUSE_JOIN: set);
global!(_BACK_TAKING_A_THING: set);
global!(_REPORTED_COMPLAINT: re);
global!(_STANDING_TROUBLE: strs);
global!(_KEPT_AWAKE: re);
global!(_A_PART_IN_THE_TABLE: re);
global!(BREAK_SAID: strs);
global!(A_PART_NO_KNOCK_TEST: strs);
global!(SOFT_TISSUE: set);
global!(_A_WHILE_AGO: re);
global!(_IT_HAPPENED_TO: set);
global!(_TO_HURT_A_PART: set);
global!(_PREGNANT_WORDS: strs);
global!(_WEEKS_DRILL: map);
global!(_BACK_AFTER_A_VERB: re);
global!(_BACK_BEFORE_A_PLACE: re);
global!(_BACK_HOME: re);
global!(_SORENESS_WORDS: set);
global!(EXERCISE_AFTER: strs);
global!(SEE_DOCTOR_PARTS: set);
global!(_DECISION_WORDS: set);
global!(_A_VERB_TOOK_THE_BACK: re);
global!(_THE_BACK_IS_THEIRS: re);
global!(_CARRIES_WEIGHT: strs);
global!(_PROTECT_ALLOWED: set);
global!(_TAKE_A_TEMPERATURE: string);
global!(_FEVER_ADVICE: string);
global!(_CHICKENPOX_TIMELINE: string);
global!(_ARTHRITIS_CURE: set);
global!(_ARTHRITIS_TAIL: string);
global!(IN_YOUR_HANDS: set);
global!(AT_YOUR_SHOULDER: set);
global!(ON_YOUR_FEET: set);
global!(IN_YOUR_HANDS_REPLY: string);
global!(AT_YOUR_SHOULDER_REPLY: string);
global!(FOUR_STEPS_REPLY: string);
global!(_NEW_SUBJECT: strs);
global!(MOLE_REMOVE: strs);
global!(MOLE_HAPPENING: set);
global!(_MOLE_REMOVE_TEXT: string);
global!(_MOLE_CLEAR: set);
global!(_MOLE_ORDINARY_TEXT: string);
global!(_CHECK_HOW_TO: re);
global!(_MOLE_CHANGED_TEXT: string);
global!(_NOT_AN_NSAID: strs);
global!(_KIDNEY_WORDS: strs);
global!(_WAX_WORDS: set);
global!(_WAX_FIRST_REPLY: string);
global!(_SHAVE_FIRST_REPLY: string);
global!(_BLADE_WORDS: set);
global!(_SHAVE_BURN_WORDS: set);
global!(_BEARD_TRIM_WORDS: set);
global!(_BEARD_TRIM_REPLY: string);
global!(_STRAIGHT_SHAVE_REPLY: string);
global!(_RAZOR_BLADE_REPLY: string);
global!(_RAZOR_BURN_REPLY: string);
global!(_A_JOINT_OF_MEAT: re);
global!(_A_PART_NAMED: set);
global!(_A_HEALING_WORD: re);
global!(_A_REST_NOT_A_BONE: re);
global!(_NOT_ON_A_BODY: strs);
global!(_HEAT_OR_ICE_THE_THING: re);
global!(_A_THING_THAT_IS_NOT_A_BODY: set);
global!(ACHE_WORDS: set);
global!(_ON_A_BODY: strs);
global!(_TAMPON_WORD: re);
global!(_NOT_A_TAMPON_LINE: re);

/// `REGIONS`: (name, the words that spell the part, the region's reply), in table order.
static REGIONS: LazyLock<Vec<(String, Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.body.REGIONS")
        .items()
        .iter()
        .map(|row| {
            let r = row.items();
            (r[0].str().to_string(), r[1].strs(), r[2].str().to_string())
        })
        .collect()
});

static WORDS_RE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// `re.findall(r"[a-z']+", line)`.
fn findall_words(line: &str) -> Vec<String> {
    WORDS_RE.findall_str(line)
}

/// "An" before a part that starts with a vowel, else "A" (`part[:1] in "aeiou"`, which is
/// also true for an empty part).
fn article(part: &str) -> &'static str {
    match part.chars().next() {
        None => "An",
        Some(c) if "aeiou".contains(c) => "An",
        Some(_) => "A",
    }
}

/// `str(clean or text or "").lower()`.
fn clean_or_text_lower(u: &Understood) -> String {
    sh::clean_or_text(u).to_lowercase()
}

/// False when the nail in the line is a piece of metal (a workshop noun is in the line), not
/// one on a person.
pub fn _the_nail_is_a_persons(understood: &Understood) -> bool {
    let mut said = understood.clean.to_lowercase();
    if said.is_empty() {
        said = understood.words.join(" ").to_lowercase();
    }
    if !said.contains("nail") {
        return true;
    }
    !_WORKSHOP.is_match(&said)
}

/// False when the body word in the line belongs to a thing (a chair's leg, a sofa's back),
/// not to the person.
pub fn _the_part_is_a_persons(understood: &Understood) -> bool {
    !_a_thing_owns_the_part(understood)
}

/// False when the body the line names is somebody else's: the reading's owner says "other",
/// or a third person is named with no first person on the line.
pub fn the_body_is_the_persons(understood: &Understood) -> bool {
    if sh::field(understood, "state_owner") == "other" {
        return false;
    }
    let words = &understood.words;
    if words.iter().any(|w| _THE_PERSONS_OWN_WORDS.contains(w)) {
        return true;
    }
    !words.iter().any(|w| _SOMEONE_ELSE.contains(w))
}

/// The person's own part, and one they can still use: the standing-condition rows' guard.
pub fn the_part_that_keeps_grumbling(understood: &Understood) -> bool {
    the_body_is_the_persons(understood) && !_CANNOT_USE_IT.is_match(&clean_or_text_lower(understood))
}

/// True when the line says, in words, that someone is pregnant.
pub fn _names_a_pregnancy(understood: &Understood) -> bool {
    let line = sh::squash_lower(sh::clean_or_text(understood));
    sh::any_in(&line, &_PREGNANCY_WORDS)
}

/// True when the line dates a pregnancy in weeks and is about swelling.
pub fn _dates_a_pregnancy_in_weeks(understood: &Understood) -> bool {
    let line = sh::squash_lower(sh::clean_or_text(understood));
    _a_figure_in_weeks(&line)
}

/// A week figure that dates a pregnancy ("im 30 weeks"), on a line that is about swelling.
pub fn _a_figure_in_weeks(words: &str) -> bool {
    let line = words.to_lowercase();
    _WEEKS_ALONG.is_match(&line) && _SWELLING.is_match(&line)
}

/// True when the reading says the temperature in the line is somebody's own (a fever), not a
/// probe's, an oven's or a room's.
pub fn a_persons_temperature(understood: &Understood) -> bool {
    let words: Vec<String> =
        understood.words.iter().map(|w| sh::strip_punct(&w.to_lowercase()).to_string()).collect();
    if words
        .windows(2)
        .any(|p| TEMPERATURE_WORDS.contains(&p[0]) && THING_AFTER_THE_WORD.contains(&p[1]))
    {
        return false;
    }
    if understood.body.as_deref() == Some("temperature") {
        return true;
    }
    if !words.iter().any(|w| TEMPERATURE_WORDS.contains(w)) {
        return false;
    }
    if words.iter().any(|w| OTHER_PEOPLE.contains(w) || WE_OUR_OURS.contains(w)) {
        return true;
    }
    words
        .windows(2)
        .any(|p| POSSESSIVE_OF_A_PERSON.contains(&p[0]) && TEMPERATURE_WORDS.contains(&p[1]))
}

/// True when the reading says the line is about the person's stomach (a stool, green poo),
/// so the furniture and salad rows stand aside.
pub fn the_stomach_is_what_it_is_about(understood: &Understood) -> bool {
    understood.body.as_deref() == Some("stomach") || understood.own_output.is_some()
}

/// The swollen-part card: same day for a hot one, this week for an ordinary swelling, and the
/// back-and-calf card for the parts a clot or the chest can hide behind.
pub fn _see_today(_memory: &Memory, understood: &Understood) -> String {
    let words = _words(understood);
    let part = _part_named(&words, Some(understood));
    if part.is_empty() || CLOT_AND_CHEST.contains(&part) {
        return SEE_TODAY_BACK_AND_CALF.clone();
    }
    if HOT_FLAGS.iter().any(|f| words.contains(f)) {
        return format!(
            "A {part} that has gone hot and swollen wants looking at today rather than iced and \
             waited on: ring 111 this morning, and go to A&E if there is a fever with it or you \
             cannot put weight on it at all."
        );
    }
    format!(
        "A swollen {part} is usually fluid - from a knock or overuse, or from a tablet or the heart \
         when it is on both sides - so have it looked at this week and keep it raised when you sit. \
         Ring 111 today if it is hot, red and spreading, or painful on just one side."
    )
}

/// The part the words name, by the region table then the joints, in its one-word form; "" for
/// none. A "back" that is not a body's back (given the reading) does not count.
pub fn _part_named(words: &HashSet<String>, understood: Option<&Understood>) -> String {
    let dropped;
    let mut words = words;
    if let Some(u) = understood {
        if words.contains("back") && !_the_back_is_a_back(u) {
            let mut w = words.clone();
            w.remove("back");
            dropped = w;
            words = &dropped;
        }
    }
    for (_name, spelled, _reply) in REGIONS.iter() {
        for word in spelled {
            if words.contains(word) {
                return PART_ONE.get(word).cloned().unwrap_or_else(|| word.clone());
            }
        }
    }
    for part in JOINTS.iter() {
        if words.contains(part) {
            return PART_ONE.get(part).cloned().unwrap_or_else(|| part.clone());
        }
    }
    String::new()
}

/// Whether any of the words is in the text as a whole word (`" " + word + " " in text`).
pub fn _names<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    sh::any_word_in(text, words)
}

/// The back red-flag card (999), the saddle-numbness card, or "" for a numb hand, arm or
/// tongue that another entry answers.
pub fn _red_flag(_memory: &Memory, understood: &Understood) -> String {
    static NOT_LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z]+", 0));
    let said = understood.text.to_lowercase().replace('\u{2019}', "").replace('\'', "");
    let mut text = format!(" {} ", NOT_LETTERS.sub(" ", &said));
    for word in understood.words.iter().chain(understood.background.iter()) {
        text.push_str(&word.to_lowercase().replace('\'', ""));
        text.push(' ');
    }
    if _names(&text, &SPELLED_OUT) {
        return RED_FLAG_CARD.clone();
    }
    let own = format!(" {} ", NOT_LETTERS.sub(" ", &understood.text.to_lowercase()));
    let mut ruled_out = own;
    for phrase in [
        "no back pain", "no backache", "no back ache", "no pain in my back",
        "without back pain", "no leg pain", "no pain in my legs", "no weakness",
        "no trouble weeing", "no trouble going for a wee", "no problems weeing",
        "no trouble peeing", "no numbness in my legs", "no pain",
        "my back is fine", "my back is ok", "my back is okay", "back is fine",
        "without a bad back", "no problems with my back",
    ] {
        ruled_out = ruled_out.replace(phrase, " ");
    }
    if _names(&ruled_out, &BACK_NAMES) && _names(&ruled_out, &NUMB_ANYWHERE) && _names(&ruled_out, &NUMB_LOWER) {
        return RED_FLAG_CARD.clone();
    }
    if _names(&ruled_out, &SADDLE_PARTS) && !_names(&ruled_out, &FLAG_WITH_IT) {
        return SADDLE_CARD.clone();
    }
    if _names(&text, &LOWER_HALF) || !_names(&text, &SOMEWHERE_ELSE) {
        return RED_FLAG_CARD.clone();
    }
    String::new()
}

/// The reading's words and background words, lower-cased, as a set.
pub fn _words(understood: &Understood) -> HashSet<String> {
    understood.words.iter().chain(understood.background.iter()).map(|w| w.to_lowercase()).collect()
}

/// The dose card for the painkiller the line names (co-codamol, ibuprofen, or paracetamol by
/// default); None for a painkiller this table has no row for.
pub fn _dose_for_the_drug_named(_memory: &Memory, reading: &Understood) -> Option<String> {
    let has = |set: &[&str]| reading.words.iter().any(|w| set.contains(&w.as_str()));
    if has(&[
        "aspirin", "disprin", "anadin", "naproxen", "diclofenac", "voltarol", "tramadol",
        "solpadeine", "veganin", "feminax", "migraleve", "dydramol", "paracodol", "dihydrocodeine",
    ]) {
        return None;
    }
    let paracetamol = has(&["paracetamol", "acetaminophen", "calpol", "panadol"]);
    let ibuprofen = has(&["ibuprofen", "nurofen", "brufen"]);
    let codeine = has(&["codeine", "co-codamol", "co codamol", "cocodamol", "codamol"]);
    if codeine && !paracetamol {
        return Some(
            "Co-codamol is paracetamol and codeine together, so the paracetamol limit is what \
             caps it: two 500/8mg tablets, four times in 24 hours at most, eight a day. The \
             codeine is what makes it constipating and drowsy, so days rather than weeks, and \
             don't drive until you know how it treats you. Don't add paracetamol on top: over \
             4g a day from any mix of products is a liver emergency."
                .into(),
        );
    }
    if ibuprofen && !paracetamol {
        return Some(
            "Ibuprofen for an adult is 200 to 400mg every four to six hours, up to three \
             times a day, with food, and no more than 1200mg a day without a prescription - \
             six of the 200mg tablets. Skip it \
             if you have a stomach ulcer, kidney trouble or are on a blood thinner, and in the \
             last three months of pregnancy. Use the lowest dose that works, and check a cold \
             remedy does not already contain it."
                .into(),
        );
    }
    Some(
        "Two 500mg paracetamol tablets, four times in 24 hours at most, four hours apart - 4g, \
         or eight tablets, a day, and less if you have liver trouble or drink heavily. \
         Ibuprofen is 200 to 400mg every four to six hours with food, and no more than 1200mg \
         a day without a prescription. The packet is the one to trust, and check a cold remedy \
         does not already contain paracetamol."
            .into(),
    )
}

/// The clean line and the words, joined (the piercing guards' reading of the line).
fn clean_and_words(u: &Understood) -> String {
    format!("{} {}", u.clean, u.words.join(" "))
}

/// True when a piercing is named with a sign of infection, and the line is not a healing
/// question.
pub fn _a_piercing_in_trouble(understood: &Understood) -> bool {
    let said = clean_and_words(understood);
    if !(_PIERCING_PIECE.is_match(&said) && _PIERCING_TROUBLE.is_match(&said)) {
        return false;
    }
    !_PIERCING_NOT_TROUBLE.is_match(&said)
}

/// The infected-piercing card (a fever makes it today's).
pub fn _piercing_infection(_memory: &Memory, understood: &Understood) -> String {
    static FEVER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bfever\b", re::I));
    let said = clean_and_words(understood);
    let mut lead = "A piercing with spreading redness, heat, pus or swelling is infected, and it needs \
                    cleaning and often antibiotics rather than patience.";
    if FEVER.is_match(&said) {
        lead = "An infected piercing with a fever is one to have looked at today - a fever means \
                it is not staying local.";
    }
    format!(
        "{lead} Leave the jewellery in unless a piercer takes it out: the hole closes over \
         the infection and traps it, and a piercer can fit a longer bar if the swelling has \
         outgrown the one you have. Clean it twice a day with sterile saline on a clean pad, \
         let water run over it in the shower, and stop touching or turning it. Warm salty \
         water soaks for ten minutes twice a day ease the soreness. See a piercer or a GP \
         within a day or two if the redness spreads more than a centimetre a day, the skin \
         round it is hot, there is pus that keeps coming, or the swelling swallows the \
         jewellery - and go straight away if the redness streaks towards the body, or you have \
         a fever with it. Cartilage piercings are the ones that go wrong: a hot, swollen \
         helix or nose that is spreading is worth being seen rather than watched."
    )
}

/// True when the piercing line is a healing bump or names no sign of trouble.
pub fn _a_healing_piercing_not_a_troubled_one(understood: &Understood) -> bool {
    let said = clean_and_words(understood);
    if _PIERCING_BUMP.is_match(&said) {
        return true;
    }
    !_PIERCING_TROUBLE.is_match(&said)
}

/// How long a piercing takes to heal (cartilage or lobe) and the aftercare routine.
pub fn _piercing_healing(_memory: &Memory, understood: &Understood) -> String {
    let said = clean_and_words(understood);
    if _CARTILAGE_PIERCING.is_match(&said) {
        return format!(
            "A cartilage piercing - a nose, a nostril, a septum, a helix - takes six to twelve \
             weeks and often months to settle, where an earlobe is six to eight weeks.{}",
            *_PIERCING_ROUTINE
        );
    }
    format!(
        "An earlobe takes six to eight weeks to heal and cartilage six to twelve weeks, though \
         cartilage can take months to settle.{}",
        *_PIERCING_ROUTINE
    )
}

/// True when the dentist line is an errand the dentist card answers (finding a practice, a
/// waiting list, the length of a visit), not a career.
pub fn _an_errand_the_dentist_card_answers(understood: &Understood) -> bool {
    let said = sh::clean_or_text(understood);
    if _A_BECOMING_A_DENTIST.is_match(said) {
        return false;
    }
    if _A_PRACTICE_ERRAND.is_match(said) || _A_LIST_THAT_IS_LONG.is_match(said) {
        return true;
    }
    _A_LENGTH_OF_A_VISIT.is_match(said)
}

/// The lengths of the dental visits, the ones the line names first, then the booking tip.
pub fn _the_lengths_of_the_visit(said: &str, words: &HashSet<String>) -> String {
    let any = |set: &[&str]| set.iter().any(|w| words.contains(*w));
    let mut named: Vec<String> = Vec::new();
    if any(&["filling", "fillings"]) {
        named.push(_A_FILLING.clone());
    }
    if any(&["canal"]) || said.contains("root canal") {
        named.push(_A_ROOT_CANAL.clone());
    }
    if any(&["hygienist", "hygienist's", "clean", "cleaning"]) {
        named.push(_A_HYGIENIST.clone());
    }
    if any(&["crown", "crowns"]) {
        named.push(_A_CROWN_VISIT.clone());
    }
    if any(&["implant", "implants"]) {
        named.push(_AN_IMPLANT.clone());
    }
    let mut parts = named.clone();
    parts.extend(_THE_VISIT_IN_ORDER.iter().filter(|p| !named.contains(p)).cloned());
    parts.push(_THE_BOOKING.clone());
    parts.join(" ")
}

/// The dentist errand: how long a crown lasts, how long a visit takes, how long the wait is,
/// or how to register with a practice.
pub fn _the_dentist_errand(memory: &Memory, understood: &Understood) -> String {
    static HOW_LONG: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow (?:long|many (?:minutes|hours))\b", 0));
    static HOW_MANY: LazyLock<Re> = LazyLock::new(|| Re::new(r"how many (?:minutes|hours)", 0));
    let words = _line_words(understood);
    let said = clean_or_text_lower(understood);
    let any = |set: &[&str]| set.iter().any(|w| words.contains(*w));
    let occasion = any(&["appointment", "appointments", "checkup", "check-up", "check", "visit", "hygienist", "hygienist's"]);
    let done_to_a_tooth = any(&["filling", "fillings", "crown", "crowns", "canal"])
        || said.contains("root canal")
        || said.contains("check up");
    let the_place = any(&["dentist", "dentists", "dental", "orthodontist", "tooth", "teeth", "practice", "surgery"]);
    let how_long = HOW_LONG.is_match(&said);
    let takes = any(&["take", "takes", "taking"]);
    let lasts = any(&["last", "lasts"]);
    let queued = words.iter().any(|w| _THE_QUEUE.contains(w)) || any(&["see", "seen"]);
    let the_wait = queued || said.contains("to get");
    let registering = any(&["register", "registering", "sign up", "join"]);
    let a_crown = any(&["crown", "crowns"]);
    if a_crown && lasts && !takes && !the_wait && !registering {
        return "A crown lasts ten to fifteen years on average, and some go much longer. What wears \
                it out is usually the tooth and the gum under it rather than the crown itself, so \
                keep cleaning round the edge where it meets the gum, and tell the dentist if it \
                starts to feel loose or the gum bleeds when you floss."
            .into();
    }
    let a_visit = (occasion && (how_long || takes || lasts))
        || (done_to_a_tooth && (takes || HOW_MANY.is_match(&said)))
        || (the_place && how_long && (takes || lasts));
    if a_visit && !the_wait && !registering {
        return _the_lengths_of_the_visit(&said, &words);
    }
    if _A_LIST_THAT_IS_LONG.is_match(&said) && !how_long {
        return _REGISTERING_A_DENTIST.clone();
    }
    if queued || said.contains("to get") {
        return _book_the_appointment(memory, understood);
    }
    _REGISTERING_A_DENTIST.clone()
}

/// Painkillers and heartburn remedies in pregnancy and while breastfeeding.
pub fn _what_can_i_take_pregnant_or_feeding(_memory: &Memory, reading: &Understood) -> String {
    let has = |set: &[&str]| reading.words.iter().any(|w| set.contains(&w.as_str()));
    let feeding = has(&["breastfeeding", "breast", "feeding", "nursing", "chestfeeding"]);
    let pregnant = has(&["pregnant", "pregnancy", "expecting", "trimester"]);
    if feeding && !pregnant {
        return "Paracetamol and ibuprofen are both fine while you are breastfeeding; avoid \
                aspirin and go easy on codeine. Heartburn takes an antacid like Gaviscon, \
                own-brand is the same. A pharmacist will check anything else for free, and see \
                your GP if it lasts days rather than hours."
            .into();
    }
    "In pregnancy, paracetamol at the lowest dose that works, for the shortest time, and \
     no ibuprofen - especially after 20 weeks - or aspirin unless a doctor prescribed it. \
     While breastfeeding, paracetamol and ibuprofen are both fine; avoid aspirin and go \
     easy on codeine. Heartburn takes an antacid like Gaviscon, own-brand is the same. A \
     pharmacist will check the rest, and ring your midwife if it lasts days or comes with \
     bleeding or less movement."
        .into()
}

/// The words of a line longer than two letters, lower-cased.
pub fn _bare_words(line: &str) -> Vec<String> {
    findall_words(&line.to_lowercase()).into_iter().filter(|w| text::len(w) > 2).collect()
}

/// How to get an appointment (GP or dentist, by the place named or the turn before), or how
/// long the wait usually is.
pub fn _book_the_appointment(memory: &Memory, understood: &Understood) -> String {
    const PLACES: [&str; 4] = ["dentist", "dentists", "dental", "orthodontist"];
    const DOCTORS: [&str; 8] = ["doctor", "doctors", "gp", "gps", "surgery", "practice", "clinic", "nurse"];
    let words = _line_words(understood);
    let mut named = PLACES.iter().any(|p| words.contains(*p));
    if !named && !DOCTORS.iter().any(|d| words.contains(*d)) {
        let start = memory.turns.len().saturating_sub(1);
        let before: HashSet<String> =
            memory.turns[start..].iter().flat_map(|t| _bare_words(&t.user)).collect();
        if before.iter().any(|w| TOOTH_WORDS.contains(w)) {
            named = true; // `before & places or {"dentist"}`
        } else if PLACES.iter().any(|p| before.contains(*p)) && before.iter().any(|w| SEEN_ABOUT_IT.contains(w)) {
            named = true;
        }
    }
    if ["long", "soon", "quickly", "wait", "waits", "waiting", "weeks", "days"].iter().any(|w| words.contains(*w)) {
        if named {
            return "Ask what the wait is when you ring: a routine dental check-up is usually a few \
                    days to a couple of weeks, depending on the practice, and a tooth that hurts is \
                    seen the same day or the next because most practices keep a few slots back each \
                    morning. Ring at opening time for one of those, and 111 can name an urgent NHS \
                    dentist if yours has nothing for days."
                .into();
        }
        return "Ask what the wait is when you ring: a routine GP appointment is usually a few days to \
                a couple of weeks, and anything urgent is seen the same day because surgeries keep \
                slots back each morning for it. Ring at the moment the lines open for one of those, \
                and 111 can find an urgent dentist or a walk-in centre if your surgery has nothing."
            .into();
    }
    if named {
        return "Ring the dental practice as soon as it opens and ask for an urgent slot: most keep a \
                few back each morning, and saying it hurts, or that a tooth has broken or a filling \
                has come out, is what gets you one of them. Ask what they have sooner if the wait is \
                weeks, and ring 111 for the out-of-hours dental service if your practice is shut or \
                cannot see you within a few days - they can also name an urgent NHS dentist. Until \
                then paracetamol or ibuprofen at the dose on the pack, and a pharmacist can help you \
                hold on."
            .into();
    }
    "Ring at the moment the lines open - 8am for most GP surgeries - and ask for the first \
     appointment of the day: same-day slots go in the first half hour. Say in one sentence \
     what it is for, because \"I need it looked at today\" books faster than \"I was \
     wondering\", and ask to go on the cancellation list if the wait is weeks. Dentists are \
     the same: say it hurts and ask what they have sooner, and 111 can find an urgent NHS \
     dentist if yours cannot see you within a few days. If there is nothing at all, ask which \
     day the next slots are released and whether the duty doctor can ring you back, and \
     remember a pharmacist can treat a lot of everyday things the same day."
        .into()
}

/// True unless the line asks whether to be seen (a decision) with no booking word in it.
pub fn _a_decision_is_not_a_booking(understood: &Understood) -> bool {
    let clean = understood.clean.to_lowercase();
    if !_DECISION_ASK.is_match(&clean) {
        return true;
    }
    if sh::any_in(&clean, &_A_RETURN_TO) {
        return true;
    }
    findall_words(&clean).iter().any(|w| _THE_BOOKING_WORDS.contains(w))
}

/// Whether one of the last `keep` bot turns already said the opening.
pub fn _said_already(opening: &str, memory: &Memory, keep: usize) -> bool {
    let opening = opening.to_lowercase();
    let start = memory.turns.len().saturating_sub(keep);
    memory.turns[start..].iter().any(|t| t.bot.to_lowercase().contains(&opening))
}

/// Swallowed sea water: the pet card when a pet is the one talked about, the 111 card for
/// sickness or a lot of it, else the mouthful-is-harmless answer.
pub fn _sea_water_reply(memory: &Memory, understood: &Understood) -> Option<String> {
    if crate::knowledge::pets::_pet_in_the_talk(memory, understood) {
        if _said_already("ring the vet now", memory, 3) {
            return Some(
                "Same as before, and it still stands: ring the vet now and say how much salt \
                 water and when. Being sick, wobbly or drowsy in the next few hours is a \
                 same-day visit, not a wait and see, and keep the fresh water away until the \
                 vet has said otherwise."
                    .into(),
            );
        }
        return crate::knowledge::pets::_pet_salt_water(memory, understood);
    }
    let line = understood.clean.to_lowercase();
    if SEA_WATER_ALARM.is_match(&line) {
        return Some(SEA_WATER_ALARM_REPLY.clone());
    }
    Some(
        "A mouthful or two of sea water does no harm: the salt in it just makes you thirsty, so \
         rinse your mouth out and drink some fresh water. Boiling it is no help - the water \
         steams off and the salt stays, so it gets saltier. A child or an adult who has swallowed \
         a lot of it - or who is being sick, dizzy or confused - needs 111, and 999 if they are \
         drowsy or hard to wake. Never make anyone sick to get it out."
            .into(),
    )
}

/// Whether a word names a thing that can own a part (a chair, a bag, a car), possessive or
/// plural forms included.
pub fn _owner_word(word: &str) -> bool {
    let mut word = sh::strip_punct(word);
    if let Some(stem) = word.strip_suffix("'s") {
        word = stem;
    }
    OBJECT_OWNERS.contains(word) || word.strip_suffix('s').is_some_and(|w| OBJECT_OWNERS.contains(w))
}

/// True when a thing in the line owns the body word ("the chair's leg", "the back wall", "the
/// leg of the table", "my foot slipping in my shoes").
pub fn _a_thing_owns_the_part(understood: &Understood) -> bool {
    // `words` is always a list on a reading here (Python falls back to the clean line split
    // when the reading's words are not a list).
    a_thing_owns_the_part_in(&understood.words)
}

/// `_a_thing_owns_the_part` on a list of words.
fn a_thing_owns_the_part_in(words: &[String]) -> bool {
    let words: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();
    let n = words.len();
    let strip = |w: &String| sh::strip_punct(w).to_string();
    for (index, word) in words.iter().enumerate() {
        if !THING_PARTS.contains(sh::strip_punct(word)) {
            continue;
        }
        if index > 0 && _owner_word(&words[index - 1]) {
            return true;
        }
        let nxt: Vec<String> = words[(index + 1).min(n)..(index + 4).min(n)]
            .iter()
            .map(strip)
            .filter(|w| !w.is_empty() && !_DETERMINERS.contains(w))
            .collect();
        if let Some(first) = nxt.first() {
            if !THING_PARTS.contains(first) && _owner_word(first) {
                return true;
            }
        }
        let between: Vec<String> = words[(index + 1).min(n)..(index + 5).min(n)].iter().map(strip).collect();
        for preposition in ["in", "inside"] {
            if let Some(cut) = between.iter().position(|w| w == preposition) {
                if !between[..cut].iter().any(|w| HURT_WORDS.contains(w)) {
                    let rest: Vec<&String> = between[cut + 1..]
                        .iter()
                        .filter(|w| !w.is_empty())
                        .filter(|w| !_DETERMINERS.contains(*w))
                        .filter(|w| !_POSSESSIVES.contains(*w))
                        .collect();
                    if let Some(first) = rest.first() {
                        if _owner_word(first) {
                            return true;
                        }
                    }
                }
                break;
            }
        }
        let after = &words[(index + 1).min(n)..(index + 5).min(n)];
        if let Some(first) = after.first() {
            if sh::strip_punct(first) == "of" {
                let mut rest: Vec<String> =
                    after[1..].iter().map(strip).filter(|w| !w.is_empty() && !_DETERMINERS.contains(w)).collect();
                while rest.len() > 1 && _POSSESSIVES.contains(&rest[0]) {
                    rest.remove(0);
                }
                if let Some(first) = rest.first() {
                    if !_PERSON_OWNER.contains(first) && !THING_PARTS.contains(first) {
                        return true;
                    }
                }
                if rest.iter().any(|w| _owner_word(w)) {
                    return true;
                }
            }
        }
    }
    false
}

/// The region a typed line says hurts ("my back is killing me" -> "back"); None when the line
/// has no hurt word, a thing owns the part, or no region is named.
pub fn the_part_that_hurts(text_in: &str) -> Option<String> {
    let line = text::squash(&text_in.to_lowercase());
    let found = findall_words(&line);
    if found.is_empty() {
        return None;
    }
    if !(found.iter().any(|w| HURT_WORDS.contains(w)) || found.iter().any(|w| w == "killing")) {
        return None;
    }
    if a_thing_owns_the_part_in(&found) {
        return None;
    }
    for (name, parts, _reply) in REGIONS.iter() {
        if parts.iter().any(|p| found.contains(p)) {
            return Some(name.clone());
        }
    }
    None
}

/// The words of a reading: its words lower-cased and stripped of punctuation, or the words
/// of its line when it has none.
pub fn _reading_words(reading: &Understood) -> Vec<String> {
    if !reading.words.is_empty() {
        return reading.words.iter().map(|w| sh::strip_punct(&w.to_lowercase()).to_string()).collect();
    }
    findall_words(&sh::clean_or_text(reading).to_lowercase())
}

/// `_reading_words` of the leaned-on line (a dict with "words" and "text", and no "clean").
fn _lean_words(leaned: &crate::understood::LeansOn) -> Vec<String> {
    if !leaned.words.is_empty() {
        return leaned.words.iter().map(|w| sh::strip_punct(&w.to_lowercase()).to_string()).collect();
    }
    findall_words(&leaned.text.to_lowercase())
}

/// False when the line's only "back" is the particle in "come back" or "is back again".
pub fn _the_back_is_a_back(understood: &Understood) -> bool {
    let mut words = _reading_words(understood);
    if let Some(leaned) = &understood.leans_on {
        // a follow-up says "it keeps coming back", so the back a chat was about is in the line
        // this one leans on: "it hurts when i sit" after "my back aches" is still a back.
        words.extend(_lean_words(leaned));
    }
    if words.is_empty() {
        return true;
    }
    if words.iter().any(|w| _ALSO_THE_BACK.contains(w)) {
        return true;
    }
    for (index, word) in words.iter().enumerate() {
        if word != "back" {
            continue;
        }
        let next = words.get(index + 1);
        if next.is_some_and(|w| _BACK_IS_A_NOUN_AFTER.contains(w)) {
            return true;
        }
        if index > 0 && _BACK_THEIRS.contains(&words[index - 1]) {
            return true;
        }
        if index > 0 && _BACK_VERB_BEFORE.contains(&words[index - 1]) {
            continue;
        }
        if index > 0 && _BACK_DESCRIBED.contains(&words[index - 1]) {
            return true;
        }
        if next.is_some_and(|w| _BACK_IS_A_RETURN.contains(w)) {
            continue;
        }
        if _a_verb_earlier_took_it(&words, index) {
            continue;
        }
        return true;
    }
    false
}

/// Whether a verb up to five words before the "back" (in the same clause) took it, as in
/// "give the book back".
pub fn _a_verb_earlier_took_it(words: &[String], index: usize) -> bool {
    let mut seen = 0;
    for at in (0..index).rev() {
        let word = &words[at];
        if _BACK_CLAUSE_JOIN.contains(word) {
            return false;
        }
        if _BACK_TAKING_A_THING.contains(word) {
            return true;
        }
        seen += 1;
        if seen > 4 {
            return false;
        }
    }
    false
}

/// The part is the person's (not a thing's) and the back is a body's back.
pub fn _the_back_is_a_back_that_is_someones(understood: &Understood) -> bool {
    _the_part_is_a_persons(understood) && _the_back_is_a_back(understood)
}

/// False when the line is about a standing trouble ("a bad knee", "strengthen my back") that
/// is not reported as the person's own complaint.
pub fn _the_part_is_not_a_standing_trouble(understood: &Understood) -> bool {
    let line = clean_or_text_lower(understood);
    if _REPORTED_COMPLAINT.is_match(&line) {
        return true;
    }
    !sh::any_in(&line, &_STANDING_TROUBLE)
}

/// The guard a region row gets: the back's own one for "back", the part's for the rest.
pub fn _a_region_guard(name: &str) -> fn(&Understood) -> bool {
    if name == "back" {
        return _a_region_guard__the_back_is_the_persons_and_not_a_standing_trouble;
    }
    _a_region_guard__the_part_is_the_persons_and_not_a_standing_trouble
}

/// The back row's guard: a person's back, not a thing's, and not a standing trouble.
#[allow(non_snake_case)]
pub fn _a_region_guard__the_back_is_the_persons_and_not_a_standing_trouble(understood: &Understood) -> bool {
    _the_back_is_a_back_that_is_someones(understood) && _the_part_is_not_a_standing_trouble(understood)
}

/// A region row's guard: the person's part, not a thing's, and not a standing trouble.
#[allow(non_snake_case)]
pub fn _a_region_guard__the_part_is_the_persons_and_not_a_standing_trouble(understood: &Understood) -> bool {
    _the_part_is_a_persons(understood) && _the_part_is_not_a_standing_trouble(understood)
}

/// True when the line says a part is stopping the person doing something (a shape).
pub fn _a_part_keeps_me_from_something(words: &str) -> bool {
    let line = words.to_lowercase();
    _KEPT_AWAKE.is_match(&line) && _A_PART_IN_THE_TABLE.is_match(&line)
}

/// The break words for a region row whose part the knock card has no test for; else none.
pub fn _the_breaks_it_says(name: &str) -> Vec<String> {
    if A_PART_NO_KNOCK_TEST.iter().any(|p| p == name) { BREAK_SAID.clone() } else { Vec::new() }
}

/// True when the hurt part in the line is somebody else's and something happened to it ("my
/// son fell and hurt his arm").
pub fn _it_is_somebody_elses_part(understood: &Understood) -> bool {
    if !_the_part_is_a_persons(understood) {
        return false;
    }
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let someone = words.iter().any(|w| {
        let person = w.strip_suffix("'s").unwrap_or(w);
        _SOMEONE_ELSE.contains(person)
    });
    if !someone {
        return false;
    }
    let said = clean_or_text_lower(understood);
    if _A_WHILE_AGO.is_match(&said) {
        return false;
    }
    let Some(first_part) = words.iter().position(|w| SOFT_TISSUE.contains(w)) else {
        return false;
    };
    if words.iter().any(|w| _IT_HAPPENED_TO.contains(w)) || said.contains("went over") {
        return true;
    }
    words.iter().enumerate().any(|(at, w)| _TO_HURT_A_PART.contains(w) && at < first_part)
}

/// The reading's words, lower-cased, as a set.
pub fn _line_words(understood: &Understood) -> HashSet<String> {
    understood.words.iter().map(|w| w.to_lowercase()).collect()
}

/// How long the line says it has been sore, as a phrase ("for months", ...).
pub fn _how_long(understood: &Understood) -> &'static str {
    static YEAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\byear", 0));
    static MONTH: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bmonth", 0));
    let said = clean_or_text_lower(understood);
    if YEAR.is_match(&said) {
        return "for months or years";
    }
    if MONTH.is_match(&said) {
        return "for months";
    }
    "for a week or more"
}

/// True when every "back" in the line is a direction ("back home", "came back"), not a body.
pub fn _back_is_a_direction(understood: &Understood) -> bool {
    back_is_a_direction_in(&clean_or_text_lower(understood))
}

/// `_back_is_a_direction` on the lower-cased line.
fn back_is_a_direction_in(text: &str) -> bool {
    static BACK: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bback\b", 0));
    let said = BACK.finditer(text).len();
    let seen = _BACK_AFTER_A_VERB.finditer(text).len()
        + _BACK_BEFORE_A_PLACE.finditer(text).len()
        + _BACK_HOME.finditer(text).len();
    said > 0 && seen >= said
}

/// Whether an earlier user turn talked about a body's back.
pub fn _a_back_in_the_chat(memory: &Memory) -> bool {
    static BACK: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bback\b", 0));
    memory.turns.iter().any(|turn| {
        let said = &turn.user;
        // `_back_is_a_direction({"text": said})`: the turn's text, lower-cased
        BACK.is_match(&said.to_lowercase()) && !back_is_a_direction_in(&said.to_lowercase())
    })
}

/// A part sore for weeks: keep going and build it up, physio if no better in six weeks; None
/// for a pregnancy (that card knows not to offer ibuprofen) or a back that is a direction.
pub fn _sore_weeks_reply(memory: &Memory, understood: &Understood) -> Option<String> {
    let mut parts: Vec<String> = vec![clean_or_text_lower(understood)];
    parts.push(understood.text.to_lowercase());
    parts.push(understood.plain.to_lowercase());
    parts.extend(understood.words.iter().chain(understood.background.iter()).map(|w| w.to_lowercase()));
    if let Some(last) = memory.turns.last() {
        parts.push(last.user.to_lowercase());
    }
    let joined = parts.join(" ");
    if sh::any_in(&joined, &_PREGNANT_WORDS) {
        return None;
    }
    let mut part = _part_named(&_line_words(understood), Some(understood));
    if part.is_empty() {
        part = _part_named(&_words(understood), Some(understood));
    }
    if part.is_empty() {
        part = "part".into();
    }
    if part == "back" && _back_is_a_direction(understood) && !_a_back_in_the_chat(memory) {
        return None;
    }
    let drill = _WEEKS_DRILL.get(&part).map(String::as_str).unwrap_or(
        "two short sessions a week of the movement you want to return to, stopping short of pain",
    );
    Some(format!(
        "{} {} that has been sore {} is usually a tendon or a joint asked for more than it is used \
         to: keep going as far as is comfortable, and add {}. If it is no better after six weeks, \
         wakes you up, or keeps coming back, ask your GP about physio. Ring 111 sooner for \
         numbness, a part that gives way, or pain with a fever or weight loss.",
        article(&part),
        part,
        _how_long(understood),
        drill
    ))
}

/// True when the soreness is the day after new exercise (that is DOMS, not a doctor's).
pub fn _the_day_after_new_exercise(understood: &Understood) -> bool {
    if !_line_words(understood).iter().any(|w| _SORENESS_WORDS.contains(w)) {
        return false;
    }
    let clean = understood.clean.to_lowercase();
    sh::any_in(&clean, &EXERCISE_AFTER)
}

/// Whether to see a doctor about a part: by how long it has been going, and today for the
/// sudden or the numb; None for the day after exercise.
pub fn _see_doctor_reply(_memory: &Memory, understood: &Understood) -> Option<String> {
    if _the_day_after_new_exercise(understood) {
        return None;
    }
    let part = _part_named(&_line_words(understood), Some(understood));
    let opening = if part.is_empty() {
        "Go by how long it has been going: ".to_string()
    } else {
        format!("{} {} like that: ", article(&part), part)
    };
    Some(format!(
        "{opening}give it two to six weeks if it is steady or easing, and book sooner if it is getting \
         worse, waking you at night or stopping you doing normal things. Go today if it came on \
         suddenly or after a fall, or comes with a fever, numbness or pins and needles. Ask \
         about physio when you ring; in most areas you can refer yourself."
    ))
}

/// True unless the line's only part is a "back" that is not a body's back.
pub fn _a_part_in_a_decision_is_a_part(understood: &Understood) -> bool {
    let words: HashSet<String> = _reading_words(understood).into_iter().collect();
    if !words.contains("back") || words.iter().any(|w| w != "back" && SEE_DOCTOR_PARTS.contains(w)) {
        return true;
    }
    _the_back_is_a_back(understood)
}

/// True when every word of the line is a word of the bare "should i see a doctor" decision.
pub fn _a_bare_decision(understood: &Understood) -> bool {
    let words = _line_words(understood);
    if words.is_empty() {
        return false;
    }
    words.iter().all(|w| _DECISION_WORDS.contains(w))
}

/// The see-a-doctor answer for a bare decision, without the physio line when no part is
/// named.
pub fn _see_doctor_decision(memory: &Memory, understood: &Understood) -> Option<String> {
    let reply = _see_doctor_reply(memory, understood)?;
    if !_part_named(&_line_words(understood), Some(understood)).is_empty() {
        return Some(reply);
    }
    Some(reply.replace(" Ask about physio when you ring; in most areas you can refer yourself.", ""))
}

/// False when the line's "back" is a verb's ("ring her back") rather than a body's.
pub fn _the_back_is_the_persons(understood: &Understood) -> bool {
    let clean = understood.clean.to_lowercase().replace('\u{2019}', "'");
    if !_A_VERB_TOOK_THE_BACK.is_match(&clean) {
        return true;
    }
    _THE_BACK_IS_THEIRS.is_match(&_A_VERB_TOOK_THE_BACK.sub("  ", &clean))
}

/// True when the line as typed names a part today (the person's own, usable, and not reported
/// as a standing complaint).
pub fn _the_part_is_named_today(understood: &Understood) -> bool {
    if !the_part_that_keeps_grumbling(understood) {
        return false;
    }
    let said = &understood.text;
    if _REPORTED_COMPLAINT.is_match(said) {
        return false;
    }
    let typed: HashSet<String> = findall_words(&said.to_lowercase().replace('\u{2019}', "'")).into_iter().collect();
    !_part_named(&typed, None).is_empty()
}

/// A part that gives trouble on and off: strengthen it with its drill.
pub fn _bad_part_reply(_memory: &Memory, understood: &Understood) -> String {
    let mut part = _part_named(&_line_words(understood), None);
    if part.is_empty() {
        part = _part_named(&_words(understood), None);
    }
    if part.is_empty() {
        part = "joint".into();
    }
    let drill = _WEEKS_DRILL.get(&part).map(String::as_str).unwrap_or(
        "two short sessions a week of the movement you want to keep, stopping short of pain",
    );
    let instead = if _CARRIES_WEIGHT.contains(&part) {
        "Swimming, cycling and a flat walk are the kind ones on the other days."
    } else {
        "Keep the rest of you moving, and use it as far as is comfortable."
    };
    format!(
        "{} {} that gives trouble on and off is usually weak muscle round a joint that does not \
         like one particular movement, and building it up is what keeps it quiet: {}. {} Get it \
         looked at if it gives way, locks, or swells up after a knock.",
        article(&part),
        part,
        drill,
        instead
    )
}

/// True when every word of the line is one the protect-a-part question is asked with.
pub fn _the_ask_names_nothing_else(understood: &Understood) -> bool {
    let said = sh::clean_or_text(understood);
    let words = findall_words(&said.to_lowercase().replace('\u{2019}', "'"));
    words.iter().all(|w| _in_the_allow_list(w))
}

/// Whether a word (or its stem without -ing, -es, -s, -ed, -d) is in the protect allow list.
pub fn _in_the_allow_list(word: &str) -> bool {
    if _PROTECT_ALLOWED.contains(word) {
        return true;
    }
    for tail in ["ing", "es", "s", "ed", "d"] {
        let stem = word.strip_suffix(tail).unwrap_or("");
        if text::len(stem) >= 3 && _PROTECT_ALLOWED.contains(stem) {
            return true;
        }
    }
    false
}

/// How to protect a part: its strengthening drill and how to use it.
pub fn _protect_reply(_memory: &Memory, understood: &Understood) -> String {
    let mut part = _part_named(&_line_words(understood), Some(understood));
    if part.is_empty() {
        part = _part_named(&_words(understood), Some(understood));
    }
    if part.is_empty() {
        part = "joint".into();
    }
    let drill = _WEEKS_DRILL.get(&part).map(String::as_str).unwrap_or(
        "two short sessions a week of the movement you want to keep, stopping short of pain",
    );
    let instead = if _CARRIES_WEIGHT.contains(&part) {
        "Swimming, cycling and a flat walk are the kind ones on the other days. Warm up before a \
         big job, keep the load close to you when you lift, and change position every half hour \
         rather than holding one."
    } else {
        "Keep it moving every day rather than resting it, and keep it in the middle of where it \
         moves - not locked straight, not at the end of its range."
    };
    format!(
        "{} {} is protected by the muscle round it and by how you use it more than by anything you \
         wear or avoid: {}. {}",
        article(&part),
        part,
        drill,
        instead
    )
}

/// How to take a temperature (or how to judge one without a thermometer).
pub fn _take_a_temperature(_memory: &Memory, understood: &Understood) -> String {
    let said = clean_or_text_lower(understood);
    if said.contains("without") {
        return "Without a thermometer there is no number to take, so go by how they seem: a hot \
                back or neck, flushed cheeks, shivering, and a hand on the skin is a poor guide \
                either way. Don't pile layers on or take them off to chase a guess, and get \
                anyone seen the same day if a baby under three months feels hot, or they are \
                floppy, breathing fast or not drinking."
            .into();
    }
    _TAKE_A_TEMPERATURE.clone()
}

/// The fever card, or how long a fever lasts when that is asked.
pub fn _fever_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = clean_or_text_lower(understood);
    if said.contains("how long") || said.contains("last") || said.contains("how many days") {
        return "Most fevers from a virus settle within two or three days, and the last day is \
                usually lower than the first. Have it looked at if it has been there five days or \
                is still climbing, or if it goes away and comes back. Treat how they seem rather \
                than the number, and get seen the same day for a baby under three months with \
                38C, a rash that will not fade under a glass, a stiff neck, breathing trouble or \
                a fit."
            .into();
    }
    _FEVER_ADVICE.clone()
}

/// Chickenpox: no ibuprofen, and when they stop being infectious.
pub fn _chickenpox_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = clean_or_text_lower(understood);
    if said.contains("ibuprofen") || said.contains("nurofen") || said.contains("brufen") {
        return "No - avoid ibuprofen with chickenpox, because it is linked to a serious skin \
                infection in children who have it. Use paracetamol at the dose on the pack, \
                calamine or a cool bath for the itch, and keep the nails short so the spots do not \
                scar. They are infectious from two days before the rash until every spot has \
                crusted over, usually five days, which is when they can go back to school."
            .into();
    }
    format!(
        "{} Calamine or a cool bath eases the itch, paracetamol helps the temperature and short \
         nails stop the spots scarring; avoid ibuprofen. Ring the GP if a newborn, a pregnant \
         woman or anyone with a weak immune system has been in contact.",
        *_CHICKENPOX_TIMELINE
    )
}

/// Ordinary arthritis: what it is, and that it is managed rather than cured.
pub fn _arthritis_reply(_memory: &Memory, reading: &Understood) -> String {
    if reading.words.iter().any(|w| _ARTHRITIS_CURE.contains(w)) {
        return format!(
            "Arthritis cannot be cured, but the ordinary wear-and-tear kind is managed well: {}",
            *_ARTHRITIS_TAIL
        );
    }
    format!("Ordinary arthritis - osteoarthritis - is the wear-and-tear kind: {}", *_ARTHRITIS_TAIL)
}

/// Broken or sprained: the test for the hand, the shoulder or the foot; None when the part is
/// a thing's.
pub fn _broken_or_sprained(_memory: &Memory, understood: &Understood) -> Option<String> {
    if _a_thing_owns_the_part(understood) {
        return None;
    }
    let words = _words(understood);
    if words.iter().any(|w| IN_YOUR_HANDS.contains(w)) {
        return Some(IN_YOUR_HANDS_REPLY.clone());
    }
    if words.iter().any(|w| AT_YOUR_SHOULDER.contains(w)) {
        return Some(AT_YOUR_SHOULDER_REPLY.clone());
    }
    if words.iter().any(|w| ON_YOUR_FEET.contains(w)) {
        return Some(FOUR_STEPS_REPLY.clone());
    }
    Some(FOUR_STEPS_REPLY.clone())
}

/// A short follow-up ("is that bad") that names no mole and no new subject.
pub fn _a_bare_followup(said: &str) -> bool {
    let words = findall_words(&said.to_lowercase());
    if words.is_empty() || words.len() > 6 {
        return false;
    }
    if ["mole", "moles", "birthmark", "birthmarks", "spot", "spots"].iter().any(|w| words.iter().any(|x| x == w)) {
        return false;
    }
    !_NEW_SUBJECT.iter().any(|w| words.contains(w))
}

/// The mole card: removal, the reassuring hair, how to check, a changed mole (see a GP within
/// two weeks), or the ordinary one.
pub fn _mole_reply(memory: &Memory, understood: &Understood) -> String {
    let words = _words(understood);
    let said_words = sh::text_or_clean(understood);
    if sh::any_in(&said_words.to_lowercase(), &MOLE_REMOVE) && !words.iter().any(|w| MOLE_HAPPENING.contains(w)) {
        return _MOLE_REMOVE_TEXT.clone();
    }
    let said = sh::text_or_clean(understood);
    if (words.contains("hair") || words.contains("hairy")) && !words.iter().any(|w| _MOLE_CLEAR.contains(w)) {
        return _MOLE_ORDINARY_TEXT.clone();
    }
    if _CHECK_HOW_TO.is_match(said) {
        return _MOLE_ORDINARY_TEXT.clone();
    }
    if words.iter().any(|w| MOLE_HAPPENING.contains(w)) || ["check", "checked", "checking"].iter().any(|w| words.contains(*w)) {
        return _MOLE_CHANGED_TEXT.clone();
    }
    if _a_bare_followup(said) && memory.turns.iter().any(|t| t.bot.to_lowercase().contains("get it checked by a gp")) {
        return _MOLE_CHANGED_TEXT.clone();
    }
    _MOLE_ORDINARY_TEXT.clone()
}

/// Which organ a painkiller upsets: paracetamol is the safe one, an anti-inflammatory and the
/// kidneys, or the stomach.
pub fn _the_organ_the_painkiller_upsets(_memory: &Memory, reading: &Understood) -> String {
    let line = text::squash(&reading.text).to_lowercase();
    let holds_paracetamol = sh::any_in(&line, &_NOT_AN_NSAID);
    let holds_nsaid = [
        "ibuprofen", "nurofen", "brufen", "naproxen", "aspirin", "diclofenac", "nsaid",
        "anti inflammatory", "anti-inflammatory", "voltarol", "indomethacin",
    ]
    .iter()
    .any(|w| line.contains(w));
    if holds_paracetamol && !holds_nsaid {
        return "Paracetamol is the painkiller to use here: it is not an anti-inflammatory, so it \
                does not cut the blood flow the kidneys rely on or irritate the stomach the way \
                ibuprofen does, and a blood pressure tablet or a water tablet is no reason to \
                avoid it. Stay within the 4g a day limit (less if a GP has said so, and less if \
                you have liver trouble or drink heavily), and space the doses out. If it is not \
                touching the pain, that is worth a word with a pharmacist rather than more \
                tablets."
            .into();
    }
    if sh::any_in(&line, &_KIDNEY_WORDS) {
        return "An anti-inflammatory cuts the blood flow the kidneys rely on, so it is the wrong \
                choice if yours are already poor, if you are dehydrated, or if you take a water \
                tablet or a blood pressure tablet - that mix is the one pharmacists watch for. \
                Paracetamol is the painkiller to use instead, and it is worth asking a \
                pharmacist or GP before starting ibuprofen at all if you have kidney disease or \
                only one kidney."
            .into();
    }
    "Ibuprofen and the other anti-inflammatories (naproxen, aspirin, diclofenac) can \
     irritate the stomach lining, so take them with or after food rather than on an empty \
     stomach, at the lowest dose that works and for the shortest time. Don't take them at \
     all if you have had a stomach ulcer or a bleed, or if you are on a blood thinner - \
     paracetamol is the one to use instead. Get seen the same day for a pain in the top of \
     the stomach that is severe, for black tarry poo, or for vomiting blood or anything \
     that looks like coffee grounds."
        .into()
}

/// Hair removal: the waxing answer first when the line names waxing, else shaving first.
pub fn _hair_removal(_memory: &Memory, understood: &Understood) -> String {
    if _words(understood).iter().any(|w| _WAX_WORDS.contains(w)) {
        return _WAX_FIRST_REPLY.clone();
    }
    _SHAVE_FIRST_REPLY.clone()
}

/// Face shaving: the blade, razor burn, a beard trim, or the straight shave.
pub fn _face_shaving(_memory: &Memory, understood: &Understood) -> String {
    let words = _words(understood);
    if words.iter().any(|w| _BLADE_WORDS.contains(w)) {
        return _RAZOR_BLADE_REPLY.clone();
    }
    if words.iter().any(|w| _SHAVE_BURN_WORDS.contains(w)) {
        return _RAZOR_BURN_REPLY.clone();
    }
    if words.iter().any(|w| _BEARD_TRIM_WORDS.contains(w)) {
        return _BEARD_TRIM_REPLY.clone();
    }
    _STRAIGHT_SHAVE_REPLY.clone()
}

/// False when the "break" in the line is a rest from work (or the joint is a joint of meat),
/// not a broken bone.
pub fn _not_a_break_from_work(understood: &Understood) -> bool {
    let said = understood.text.to_lowercase();
    if _A_JOINT_OF_MEAT.is_match(&said) {
        return false;
    }
    if _line_words(understood).iter().any(|w| _A_PART_NAMED.contains(w)) {
        return true;
    }
    if _A_HEALING_WORD.is_match(&said) {
        return true;
    }
    !_A_REST_NOT_A_BONE.is_match(&said)
}

/// How long a broken bone takes to heal (a leg's longest).
pub fn _healing(_memory: &Memory, understood: &Understood) -> String {
    const SLOW: [&str; 12] = ["leg", "legs", "shin", "tibia", "fibula", "femur", "ankle", "ankles", "knee", "hip", "foot", "feet"];
    if understood.words.iter().any(|w| SLOW.contains(&w.as_str())) {
        return "Bones in the leg take the longest: about 12 weeks for a shin or ankle, and a \
                thigh bone can be four months or more. The cast usually comes off before the \
                bone is fully strong - it is the X-ray at each clinic visit that decides - and \
                the muscles and joint need a few weeks after that to feel normal, so build back \
                slowly rather than testing it early."
            .into();
    }
    "About six weeks for most bones - six for a wrist or collarbone, six to eight for an \
     arm, and longer for a leg or anything that had to be pinned, up to 12 weeks or more. \
     Children heal faster, often in half the time. The cast comes off before the bone is \
     fully strong and the joint takes a few more weeks to feel normal, and smoking slows \
     bone healing a lot, so it is worth stopping while it knits."
        .into()
}

/// True when the heat or ice in the line is going on a body (a compress), not on food, a room
/// or a road.
pub fn _a_place_you_would_ice(understood: &Understood) -> bool {
    let line = sh::padded(&sh::squash_lower(sh::clean_or_text(understood)));
    if sh::any_word_in(&line, &_NOT_ON_A_BODY) {
        return false;
    }
    for thing in _HEAT_OR_ICE_THE_THING.findall_str(&line) {
        if _A_THING_THAT_IS_NOT_A_BODY.contains(&thing) && !ACHE_WORDS.contains(&thing) {
            return false;
        }
    }
    if sh::any_word_in(&line, &_ON_A_BODY) {
        return true;
    }
    ACHE_WORDS.iter().any(|w| line.contains(&format!(" {w} ")))
}

/// How long a pulled muscle takes: a groin, a torn hamstring or calf, or a pulled thigh.
pub fn _muscle_healing(_memory: &Memory, understood: &Understood) -> String {
    let has = |set: &[&str]| understood.words.iter().chain(understood.background.iter()).any(|w| set.contains(&w.as_str()));
    let slow = has(&["hamstring", "calf", "calves", "tear", "tore", "torn", "badly", "completely", "snapped"]);
    if has(&["groin", "adductor"]) {
        return "A groin strain is usually two to six weeks, and the milder ones are back in a \
                fortnight: the first week is ice and rest from the twisting and kicking, then \
                gentle stretching, and back to football or five-a-side only when you can sprint \
                and change direction without pain. A sudden sharp pain, or a lump that appears in \
                the groin when you cough, wants a physio rather than time."
            .into();
    }
    if slow {
        return "A torn hamstring or calf is four to eight weeks, and the last third of that is \
                the part people rush: run again only when you can move it through its full \
                range without pain and it feels as strong as the other side. A sudden snap, a \
                lot of bruising, or a dent in the muscle wants a physio this week rather than \
                rest alone."
            .into();
    }
    "A pulled thigh muscle is usually two to three weeks: ice it for 15 minutes every few \
     hours for the first two days, then keep it gently moving, because gentle movement \
     heals faster than total rest. Go back to sport when it moves fully without pain and \
     feels as strong as the other side, and see a physio if it is no better after a \
     fortnight."
        .into()
}

/// True when the line is about a tampon left in, not about binning or flushing one.
pub fn _a_tampon_worn_too_long(understood: &Understood) -> bool {
    let said = clean_and_words(understood);
    _TAMPON_WORD.is_match(&said) && !_NOT_A_TAMPON_LINE.is_match(&said)
}

// ---------------------------------------------------------------------------------------------
// Registry tables.

fn g_the_nail_is_a_persons(_f: &FnRef, u: &Understood) -> bool { _the_nail_is_a_persons(u) }
fn g_the_part_is_a_persons(_f: &FnRef, u: &Understood) -> bool { _the_part_is_a_persons(u) }
fn g_names_a_pregnancy(_f: &FnRef, u: &Understood) -> bool { _names_a_pregnancy(u) }
fn g_dates_a_pregnancy_in_weeks(_f: &FnRef, u: &Understood) -> bool { _dates_a_pregnancy_in_weeks(u) }
fn g_the_stomach_is_what_it_is_about(_f: &FnRef, u: &Understood) -> bool { the_stomach_is_what_it_is_about(u) }
fn g_a_piercing_in_trouble(_f: &FnRef, u: &Understood) -> bool { _a_piercing_in_trouble(u) }
fn g_a_healing_piercing_not_a_troubled_one(_f: &FnRef, u: &Understood) -> bool { _a_healing_piercing_not_a_troubled_one(u) }
fn g_an_errand_the_dentist_card_answers(_f: &FnRef, u: &Understood) -> bool { _an_errand_the_dentist_card_answers(u) }
fn g_a_decision_is_not_a_booking(_f: &FnRef, u: &Understood) -> bool { _a_decision_is_not_a_booking(u) }
fn g_a_thing_owns_the_part(_f: &FnRef, u: &Understood) -> bool { _a_thing_owns_the_part(u) }
fn g_the_back_is_a_back(_f: &FnRef, u: &Understood) -> bool { _the_back_is_a_back(u) }
fn g_region_back(_f: &FnRef, u: &Understood) -> bool {
    _a_region_guard__the_back_is_the_persons_and_not_a_standing_trouble(u)
}
fn g_region_part(_f: &FnRef, u: &Understood) -> bool {
    _a_region_guard__the_part_is_the_persons_and_not_a_standing_trouble(u)
}
fn g_it_is_somebody_elses_part(_f: &FnRef, u: &Understood) -> bool { _it_is_somebody_elses_part(u) }
fn g_a_part_in_a_decision_is_a_part(_f: &FnRef, u: &Understood) -> bool { _a_part_in_a_decision_is_a_part(u) }
fn g_a_bare_decision(_f: &FnRef, u: &Understood) -> bool { _a_bare_decision(u) }
fn g_the_back_is_the_persons(_f: &FnRef, u: &Understood) -> bool { _the_back_is_the_persons(u) }
fn g_the_part_is_named_today(_f: &FnRef, u: &Understood) -> bool { _the_part_is_named_today(u) }
fn g_the_ask_names_nothing_else(_f: &FnRef, u: &Understood) -> bool { _the_ask_names_nothing_else(u) }
fn g_not_a_break_from_work(_f: &FnRef, u: &Understood) -> bool { _not_a_break_from_work(u) }
fn g_a_place_you_would_ice(_f: &FnRef, u: &Understood) -> bool { _a_place_you_would_ice(u) }
fn g_a_tampon_worn_too_long(_f: &FnRef, u: &Understood) -> bool { _a_tampon_worn_too_long(u) }

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.body:_a_bare_decision:3037", g_a_bare_decision),
    ("bot.knowledge.body:_a_decision_is_not_a_booking:2007", g_a_decision_is_not_a_booking),
    ("bot.knowledge.body:_a_healing_piercing_not_a_troubled_one:934", g_a_healing_piercing_not_a_troubled_one),
    ("bot.knowledge.body:_a_part_in_a_decision_is_a_part:3006", g_a_part_in_a_decision_is_a_part),
    ("bot.knowledge.body:_a_piercing_in_trouble:901", g_a_piercing_in_trouble),
    ("bot.knowledge.body:_a_place_you_would_ice:4805", g_a_place_you_would_ice),
    (
        "bot.knowledge.body:_a_region_guard.<locals>._the_back_is_the_persons_and_not_a_standing_trouble:2512",
        g_region_back,
    ),
    (
        "bot.knowledge.body:_a_region_guard.<locals>._the_part_is_the_persons_and_not_a_standing_trouble:2518",
        g_region_part,
    ),
    ("bot.knowledge.body:_a_tampon_worn_too_long:4998", g_a_tampon_worn_too_long),
    ("bot.knowledge.body:_a_thing_owns_the_part:2191", g_a_thing_owns_the_part),
    ("bot.knowledge.body:_an_errand_the_dentist_card_answers:1420", g_an_errand_the_dentist_card_answers),
    ("bot.knowledge.body:_dates_a_pregnancy_in_weeks:183", g_dates_a_pregnancy_in_weeks),
    ("bot.knowledge.body:_it_is_somebody_elses_part:2641", g_it_is_somebody_elses_part),
    ("bot.knowledge.body:_names_a_pregnancy:171", g_names_a_pregnancy),
    ("bot.knowledge.body:_not_a_break_from_work:4664", g_not_a_break_from_work),
    ("bot.knowledge.body:_the_ask_names_nothing_else:3378", g_the_ask_names_nothing_else),
    ("bot.knowledge.body:_the_back_is_a_back:2388", g_the_back_is_a_back),
    ("bot.knowledge.body:_the_back_is_the_persons:3182", g_the_back_is_the_persons),
    ("bot.knowledge.body:_the_nail_is_a_persons:70", g_the_nail_is_a_persons),
    ("bot.knowledge.body:_the_part_is_a_persons:88", g_the_part_is_a_persons),
    ("bot.knowledge.body:_the_part_is_named_today:3271", g_the_part_is_named_today),
    ("bot.knowledge.body:the_stomach_is_what_it_is_about:267", g_the_stomach_is_what_it_is_about),
];

fn r_arthritis_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_arthritis_reply(m, u)) }
fn r_bad_part_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_bad_part_reply(m, u)) }
fn r_book_the_appointment(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_book_the_appointment(m, u)) }
fn r_broken_or_sprained(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _broken_or_sprained(m, u) }
fn r_chickenpox_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_chickenpox_reply(m, u)) }
fn r_dose_for_the_drug_named(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _dose_for_the_drug_named(m, u) }
fn r_face_shaving(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_face_shaving(m, u)) }
fn r_fever_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_fever_reply(m, u)) }
fn r_hair_removal(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_hair_removal(m, u)) }
fn r_healing(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_healing(m, u)) }
fn r_mole_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_mole_reply(m, u)) }
fn r_muscle_healing(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_muscle_healing(m, u)) }
fn r_piercing_healing(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_piercing_healing(m, u)) }
fn r_piercing_infection(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_piercing_infection(m, u)) }
fn r_protect_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_protect_reply(m, u)) }
fn r_red_flag(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_red_flag(m, u)) }
fn r_sea_water_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _sea_water_reply(m, u) }
fn r_see_doctor_decision(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _see_doctor_decision(m, u) }
fn r_see_doctor_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _see_doctor_reply(m, u) }
fn r_see_today(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_see_today(m, u)) }
fn r_sore_weeks_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _sore_weeks_reply(m, u) }
fn r_take_a_temperature(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_take_a_temperature(m, u)) }
fn r_the_dentist_errand(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_the_dentist_errand(m, u)) }
fn r_the_organ_the_painkiller_upsets(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_the_organ_the_painkiller_upsets(m, u))
}
fn r_what_can_i_take_pregnant_or_feeding(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_what_can_i_take_pregnant_or_feeding(m, u))
}

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.body:_arthritis_reply:3839", r_arthritis_reply),
    ("bot.knowledge.body:_bad_part_reply:3302", r_bad_part_reply),
    ("bot.knowledge.body:_book_the_appointment:1914", r_book_the_appointment),
    ("bot.knowledge.body:_broken_or_sprained:3963", r_broken_or_sprained),
    ("bot.knowledge.body:_chickenpox_reply:3571", r_chickenpox_reply),
    ("bot.knowledge.body:_dose_for_the_drug_named:835", r_dose_for_the_drug_named),
    ("bot.knowledge.body:_face_shaving:4486", r_face_shaving),
    ("bot.knowledge.body:_fever_reply:3552", r_fever_reply),
    ("bot.knowledge.body:_hair_removal:4428", r_hair_removal),
    ("bot.knowledge.body:_healing:4683", r_healing),
    ("bot.knowledge.body:_mole_reply:4211", r_mole_reply),
    ("bot.knowledge.body:_muscle_healing:4910", r_muscle_healing),
    ("bot.knowledge.body:_piercing_healing:956", r_piercing_healing),
    ("bot.knowledge.body:_piercing_infection:911", r_piercing_infection),
    ("bot.knowledge.body:_protect_reply:3398", r_protect_reply),
    ("bot.knowledge.body:_red_flag:764", r_red_flag),
    ("bot.knowledge.body:_sea_water_reply:2091", r_sea_water_reply),
    ("bot.knowledge.body:_see_doctor_decision:3051", r_see_doctor_decision),
    ("bot.knowledge.body:_see_doctor_reply:2976", r_see_doctor_reply),
    ("bot.knowledge.body:_see_today:627", r_see_today),
    ("bot.knowledge.body:_sore_weeks_reply:2834", r_sore_weeks_reply),
    ("bot.knowledge.body:_take_a_temperature:3539", r_take_a_temperature),
    ("bot.knowledge.body:_the_dentist_errand:1475", r_the_dentist_errand),
    ("bot.knowledge.body:_the_organ_the_painkiller_upsets:4288", r_the_organ_the_painkiller_upsets),
    ("bot.knowledge.body:_what_can_i_take_pregnant_or_feeding:1748", r_what_can_i_take_pregnant_or_feeding),
];

fn s_a_figure_in_weeks(_f: &FnRef, words: &str) -> bool { _a_figure_in_weeks(words) }
fn s_a_part_keeps_me_from_something(_f: &FnRef, words: &str) -> bool { _a_part_keeps_me_from_something(words) }

/// The entries' `shape` callables (a row's figure in the line's words).
pub const SHAPES: &[(&str, ShapeFn)] = &[
    ("bot.knowledge.body:_a_figure_in_weeks:194", s_a_figure_in_weeks),
    ("bot.knowledge.body:_a_part_keeps_me_from_something:2561", s_a_part_keeps_me_from_something),
];
