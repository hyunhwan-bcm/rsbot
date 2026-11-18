//! Port of PyBot's `bot/knowledge/complaints.py`: everyday complaints with one useful answer
//! each (warts and verrucas, a tooth out, a filling, vulval skin, childhood illnesses, an
//! abscess, toothache), with the replies that pick the card's branch from the line.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g4 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Module globals from the bundle, by their Python name.
macro_rules! global {
    ($name:ident: re) => {
        static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.knowledge.complaints.", stringify!($name))));
    };
    ($name:ident: strs) => {
        static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.knowledge.complaints.", stringify!($name))));
    };
    ($name:ident: string) => {
        static $name: LazyLock<String> =
            LazyLock::new(|| data::string(concat!("bot.knowledge.complaints.", stringify!($name))));
    };
}

global!(_WART_SPORT_WORDS: strs);
global!(_WART_CATCHING_WORDS: strs);
global!(_WART_CHANGED_WORDS: strs);
global!(_WART_PREGNANT_WORDS: strs);
global!(_A_MOUTH_ABOUT_IT: strs);
global!(_SOCKET: string);
global!(_VULVAL_ITCH_CARD: string);
global!(_WHITE_SKIN: re);
global!(_LICHEN_CARD: string);
global!(_FILLING_WHEN: re);
global!(FILLING_APPOINTMENT: string);
global!(FILLING_EATING_FIRST: string);
global!(_A_TIME_NAMED: re);
global!(_STILL_TO_GET_THERE: re);
global!(_ABSCESS_CARD: string);
global!(_TOOTHACHE_CARD: string);

/// `CHILDHOOD_ILLNESSES`: (the illness's names, its card), in table order.
static CHILDHOOD_ILLNESSES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.complaints.CHILDHOOD_ILLNESSES")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
});

/// Warts and verrucas: a changed one seen, pregnancy, catching, sport, or the usual treatment.
pub fn _warts_verrucas(_memory: &Memory, understood: &Understood) -> String {
    let words = understood.words.join(" ").to_lowercase();
    if sh::any_in(&words, &_WART_CHANGED_WORDS) {
        return "Get that one seen rather than treating it: a wart or verruca that turns black, \
                bleeds, or weeps is worth a GP or podiatrist appointment, because a spot that \
                changes like that is not always a wart. Keep it clean and covered until you are \
                seen, and don't pick, file or freeze it in the meantime."
            .into();
    }
    if sh::any_in(&words, &_WART_PREGNANT_WORDS) {
        return "A pharmacy salicylic acid paint or patch is usually fine in pregnancy - ask the \
                pharmacist to check it for you - and leave the freeze kits alone for now. Don't \
                pick or file it hard, and see a podiatrist if it is painful."
            .into();
    }
    if sh::any_in(&words, &_WART_CATCHING_WORDS) {
        return "Yes, mildly: a verruca passes on through the flakes of rough skin it sheds on wet \
                floors, so keep shoes on in a changing room and don't share towels or shoes, and \
                it can spread to another spot on your own foot or hand if you pick at it. It does \
                not mean staying out of the pool or off PE - a waterproof plaster or a swim sock \
                covers it, and no pool or school can insist otherwise. Treat it as usual with a \
                pharmacy paint or patch, and see a podiatrist if it is painful or spreading."
            .into();
    }
    if sh::any_in(&words, &_WART_SPORT_WORDS) {
        return "No need to stay out of the water, or off PE and games, with a verruca: cover it \
                with a waterproof plaster or wear a swim sock at the pool, keep shoes on in the \
                changing room, and don't share towels. Keep treating it with a pharmacy paint or \
                patch, and see a podiatrist if it is painful or spreading."
            .into();
    }
    "Most warts and verrucas go on their own within a year or two, and a pharmacy \
     salicylic acid gel or medicated plaster, used every day for up to twelve weeks after \
     a soak and a gentle file, clears most of the rest. Don't cut, pick or freeze them at \
     home, and cover a verruca at the pool. See a podiatrist or GP if it is painful, \
     bleeds, or you have diabetes or poor circulation in your feet."
        .into()
}

/// True unless the line's "socket" is an eye's or a plug's: a mouth word has to be with it.
pub fn _a_socket_in_a_mouth(understood: &Understood) -> bool {
    static EYES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\beyes?\b", 0));
    let mut text = sh::squash_lower(sh::clean_or_text(understood));
    if text.is_empty() {
        text = understood.words.join(" ").to_lowercase();
    }
    if !text.contains("socket") {
        return true;
    }
    if EYES.is_match(&text) {
        return false;
    }
    let padded = sh::padded(&text);
    sh::any_word_in(&padded, &_A_MOUTH_ABOUT_IT)
}

/// After a tooth out: smoking, rinsing, exercise, trapped food, a week on, alcohol, eating,
/// bleeding, a dry socket, numbness, a stiff jaw, how long it takes, healing, or the aftercare.
pub fn _after_a_tooth_out(_memory: &Memory, understood: &Understood) -> String {
    let text = understood.clean.to_lowercase();
    let words: HashSet<String> =
        text::split(&text.replace('?', " ").replace(',', " ").replace('.', " ")).into_iter().collect();
    let says = |options: &[&str]| -> bool { _after_a_tooth_out__says(&text, &words, options) };
    if says(&["smok", "vape", "vaping", "e-cigarette", "ecigarette", "cigarette", "weed", "joint"]) {
        return "No smoking or vaping for at least three days, and longer if you can: the sucking \
                and the heat are the biggest single cause of a dry socket, and it is worth more \
                than the few cigarettes. Patches or gum are fine. Alcohol and straws are the same \
                kind of trouble for the first day, because both pull the clot loose."
            .into();
    }
    if says(&["rins", "mouthwash", "salt water", "saltwater", "salt-water", "swill", "gargl"]) {
        return "Not for the first 24 hours - rinsing that early washes the clot out, and the clot \
                is what protects the bone underneath. From tomorrow, half a teaspoon of salt in a \
                glass of warm water, four times a day after meals, and let it fall out of your \
                mouth gently instead of spitting. Brush the rest of your teeth as normal, just \
                steer clear of the socket for a few days."
            .into();
    }
    if says(&[
        "exercis", "gym", "run", "runn", "lift", "weights", "swim", "football", "cycl",
        "training", "workout", "work out", "sport",
    ]) {
        return format!(
            "Keep it easy for two or three days: walking is fine, but hard training, heavy \
             lifting, swimming and anything that makes you puff can start the socket bleeding \
             again while the clot is still settling. {}When you do go back, build up over a week \
             rather than straight into a best effort.",
            *_SOCKET
        );
    }
    if says(&[
        "collects food", "collect food", "food gets stuck", "gets stuck", "traps food",
        "trapped food", "bit of food", "bits of food", "food stuck",
    ]) {
        return "A gap that keeps trapping food is worth closing even years later: the food sits \
                there and inflames the gum around it. Until it is seen, rinse hard with warm salt \
                water after every meal, and if that is not enough ask the dentist for a plastic \
                syringe with a curved tip to irrigate the socket. Mention it at your next \
                check-up - a bridge, an implant or a partial denture all stop the trapping, and \
                the dentist can tell you which suits the gap you have."
            .into();
    }
    if says(&[
        "after a week", "a week after", "a week later", "week later", "for a week",
        "weeks after", "two weeks", "ten days", "10 days", "fortnight", "last week",
        "last month", "not healing", "not healed", "hasnt healed", "hasn't healed",
        "not getting better", "getting worse", "worse not better",
    ]) {
        return "A socket that is still aching a week or more after the tooth came out, or one that \
                is getting worse rather than better, needs looking at rather than waiting: ring \
                the dentist and say how many days it has been, because a dry socket, a bit of \
                tooth or bone left in the gum and an infection all need something doing. \
                Ibuprofen or paracetamol at the dose on the pack covers the ache until you are \
                seen, and go the same day if there is pus, a fever or swelling in your face."
            .into();
    }
    if says(&["alcohol", "beer", "wine", "pint", "vodka", "spirits", "get drunk"]) {
        return format!(
            "Leave alcohol alone for the first day, and longer if you can: it thins the blood \
             and softens the clot, so the socket is more likely to ooze or to lose the clot and \
             turn into a dry socket. Water, squash and tea that is only warm are all fine, and \
             no straws. {}",
            *_SOCKET
        );
    }
    if says(&["eat", "ate", "eating", "food", "chew", "chewing", "what can i drink", "can i drink", "coffee", "tea", "fizzy"]) {
        return format!(
            "Cool, soft food on the other side for a day or two: yoghurt, soup that is only \
             warm, mashed potato, scrambled egg, a banana, and plenty to drink that is not hot. \
             Skip anything hot, crunchy or seedy that can lodge in the socket for the first few \
             days, and leave the straws out. {}",
            *_SOCKET
        );
    }
    if says(&["bleed", "bled", "blood"]) {
        return "Spotting blood for the first day is normal; a mouthful of fresh blood is not. Bite \
                firmly on the gauze the dentist packed, or a folded clean handkerchief, for 20 \
                minutes without lifting it to look, and sit up rather than lying flat. If it is \
                still bleeding after 20 minutes of steady pressure, or the bleeding restarts later \
                in the week, ring the dentist the same day - and 111 if you cannot reach them."
            .into();
    }
    if says(&[
        "dry socket", "tastes bad", "taste bad", "bad taste", "foul taste", "tastes foul",
        "tastes horrible", "tastes awful", "smells bad", "smell bad", "throbbing", "throb",
        "worse not better", "getting worse", "more painful",
    ]) {
        return "That is how a dry socket reads: a dull throbbing ache that starts around day two \
                or three, often with a foul taste or smell, because the clot has come away and the \
                bone is exposed. It does not settle on its own - ring the dentist, who puts a \
                medicated dressing in the socket and the pain usually eases within minutes. \
                Ibuprofen or paracetamol at the dose on the pack covers it until you are seen."
            .into();
    }
    if says(&[
        "cant taste", "can't taste", "cannot taste", "no taste", "lost my taste", "numb",
        "numbness", "tingling", "tingles", "still asleep",
    ]) {
        return "Numbness that outlasts the appointment, or a tongue that cannot taste properly, \
                is the local anaesthetic wearing off or a small nerve near the socket that has \
                been bruised on the way out. It usually settles over days to a few weeks. Tell \
                the dentist if the lip, chin or tongue is still numb after two or three days, or \
                if it is getting worse rather than better, because a nerve that is known about \
                early is easier to look after."
            .into();
    }
    if says(&[
        "stiff", "cant open", "can't open", "cannot open", "lockjaw", "lock jaw",
        "sore jaw", "jaw ache", "jaw aches", "jaw hurts", "jaw is",
    ]) {
        return "A stiff, sore jaw for a few days after an extraction is normal - the muscle has \
                been held open and the injection leaves it aching - and it eases over about a \
                week. A warm pack on the outside of the cheek, soft food, and opening the mouth \
                gently several times a day all help; do not force it wide. If you cannot open \
                your mouth at all, the swelling is spreading under the jaw, or you have a fever, \
                ring the dentist or 111 the same day."
            .into();
    }
    if says(&["how long", "how soon", "how much time"])
        && says(&["take", "takes", "last", "lasts", "appointment", "procedure", "surgery", "in the chair"])
        && !says(&[
            "heal", "healing", "healed", "ache", "aches", "pain", "sore", "swell", "swelling",
            "bruis", "numb",
        ])
    {
        return format!(
            "A single tooth is out in a few minutes once the gum is numb, and the whole \
             appointment is about 20 to 30 minutes with the numbing and the aftercare talk in \
             it. A wisdom tooth that has to be cut out takes longer: usually 20 to 40 minutes, \
             and up to about an hour if it is sitting deep, so ask the dentist what yours will \
             be. {}",
            *_SOCKET
        );
    }
    if says(&["heal", "how long", "normal", "days", "weeks", "week", "still"]) {
        return format!(
            "The gum closes over in about one to two weeks and the hollow fills in over the \
             weeks after that; a dull ache that eases day by day is normal, and paracetamol or \
             ibuprofen at the dose on the pack is enough for it. {}Ring the dentist if the pain \
             gets worse after day two rather than better, or if there is pus, a fever or swelling \
             in your face.",
            *_SOCKET
        );
    }
    format!(
        "{}Bite on the gauze for 20 minutes, then leave it alone: no rinsing, straws, smoking or \
         alcohol for the first day, and cool soft food on the other side. From tomorrow, warm \
         salt-water rinses four times a day, and paracetamol or ibuprofen at the dose on the \
         pack. Ring the dentist if it gets worse after day two instead of better, or if there \
         is pus, a fever or swelling in your face.",
        *_SOCKET
    )
}

/// `_after_a_tooth_out`'s `says`: a whole word, or the start of one ("smok" covers smoke,
/// smoking and smoker), and a phrase (with a space or a hyphen) read against the text.
#[allow(non_snake_case)]
pub fn _after_a_tooth_out__says(text: &str, words: &HashSet<String>, options: &[&str]) -> bool {
    options.iter().any(|option| {
        if option.contains(' ') || option.contains('-') {
            text.contains(option)
        } else {
            words.iter().any(|w| w.starts_with(option))
        }
    })
}

/// Vulval skin: the lichen sclerosus card for white, thin or tearing skin, else the itch card.
pub fn _vulval_skin(_memory: &Memory, understood: &Understood) -> String {
    static NOT_PLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' -]+", 0));
    let said = sh::padded(&NOT_PLAIN.sub(" ", &understood.text.to_lowercase()));
    if _WHITE_SKIN.is_match(&said) {
        return _LICHEN_CARD.clone();
    }
    _VULVAL_ITCH_CARD.clone()
}

/// A filling: when to eat first if the line asks about food, then what the appointment is like.
pub fn _filling_reply(_memory: &Memory, reading: &Understood) -> String {
    let said = reading.clean.to_lowercase();
    if _FILLING_WHEN.is_match(&said) || said.contains("something to eat") {
        return format!("{}{}", *FILLING_EATING_FIRST, *FILLING_APPOINTMENT);
    }
    FILLING_APPOINTMENT.clone()
}

/// True unless the line names when the visit is and says nothing still to arrange.
pub fn _the_visit_is_not_arranged(understood: &Understood) -> bool {
    let said = sh::clean_or_text(understood);
    if !_A_TIME_NAMED.is_match(said) {
        return true;
    }
    _STILL_TO_GET_THERE.is_match(said)
}

/// The reply of the row whose name comes first in the line; None when none is named.
pub fn _first_named(table: &[(Vec<String>, String)], said: &str) -> Option<String> {
    let mut best: Option<(usize, &String)> = None;
    for (names, reply) in table {
        for name in names {
            if let Some(at) = said.find(name.as_str()) {
                if best.is_none_or(|(b, _)| at < b) {
                    best = Some((at, reply));
                }
            }
        }
    }
    best.map(|(_, reply)| reply.clone())
}

/// The card for the childhood illness the line names first; None when it names none.
pub fn _childhood_illness(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = sh::clean_or_text(understood).to_lowercase();
    _first_named(&CHILDHOOD_ILLNESSES, &said)
}

/// The dental abscess card, with paracetamol alone in pregnancy.
pub fn _abscess_reply(_memory: &Memory, understood: &Understood) -> String {
    dup::health_tablets_for_a_pregnancy(&_ABSCESS_CARD, understood)
}

/// The toothache card, with paracetamol alone in pregnancy.
pub fn _toothache_reply(_memory: &Memory, understood: &Understood) -> String {
    dup::health_tablets_for_a_pregnancy(&_TOOTHACHE_CARD, understood)
}

// ---------------------------------------------------------------------------------------------
// Registry tables.

fn g_a_socket_in_a_mouth(_f: &FnRef, u: &Understood) -> bool { _a_socket_in_a_mouth(u) }
fn g_the_visit_is_not_arranged(_f: &FnRef, u: &Understood) -> bool { _the_visit_is_not_arranged(u) }

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.complaints:_a_socket_in_a_mouth:111", g_a_socket_in_a_mouth),
    ("bot.knowledge.complaints:_the_visit_is_not_arranged:331", g_the_visit_is_not_arranged),
];

fn r_abscess_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_abscess_reply(m, u)) }
fn r_after_a_tooth_out(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_after_a_tooth_out(m, u)) }
fn r_childhood_illness(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _childhood_illness(m, u) }
fn r_filling_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_filling_reply(m, u)) }
fn r_toothache_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_toothache_reply(m, u)) }
fn r_vulval_skin(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_vulval_skin(m, u)) }
fn r_warts_verrucas(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_warts_verrucas(m, u)) }

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.complaints:_abscess_reply:467", r_abscess_reply),
    ("bot.knowledge.complaints:_after_a_tooth_out:125", r_after_a_tooth_out),
    ("bot.knowledge.complaints:_childhood_illness:452", r_childhood_illness),
    ("bot.knowledge.complaints:_filling_reply:307", r_filling_reply),
    ("bot.knowledge.complaints:_toothache_reply:484", r_toothache_reply),
    ("bot.knowledge.complaints:_vulval_skin:277", r_vulval_skin),
    ("bot.knowledge.complaints:_warts_verrucas:60", r_warts_verrucas),
];
