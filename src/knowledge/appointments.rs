//! Port of `bot/knowledge/appointments.py`: getting ready for an appointment (an operation, a
//! blood test, a scan, the dentist, the optician), read from the line and the one before it.

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::{globals, last_user};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;

globals!("bot.knowledge.appointments",
    strs { _MEDICINE_NAMED, _A_LIST_TO_CARRY, _APPOINTMENT_KINDS, _NOT_A_MEDICAL_VISIT, _STILL_MEDICAL,
           _ITS_AN_OPERATION, _NOT_THE_FASTING, _ITS_A_BLOOD_TEST, _ITS_A_SCAN, _ITS_THE_DENTIST,
           _ITS_THE_OPTICIAN },
    re { _A_TEST_THAT_TAKES, _A_FAST_LENGTH_ASK });

fn any_in(line: &str, words: &[String]) -> bool {
    words.iter().any(|w| line.contains(w.as_str()))
}

/// A tablet question rather than a fasting one, and not a list to carry in.
pub fn _a_dose_is_the_question(line: &str) -> bool {
    if !any_in(line, &_MEDICINE_NAMED) {
        return false;
    }
    !any_in(line, &_A_LIST_TO_CARRY)
}

/// The user's last line, or "": from the reading's own carried line, else the chat.
/// TYPE: Python first reads `reading["context" | "last_user" | "earlier_line"]`, which the typed
/// reading does not hold (they are never set on a reading PyBot builds), and falls back to
/// `memory["history"]` when there are no turns (never kept by this port).
pub fn _the_line_before(memory: &Memory, _reading: &Understood) -> String {
    let said = last_user(memory);
    if !text::strip(&said).is_empty() {
        return text::squash(&said).to_lowercase();
    }
    for turn in memory.turns.iter().rev() {
        if !text::strip(&turn.user).is_empty() {
            return text::squash(&turn.user).to_lowercase();
        }
    }
    String::new()
}

/// Whether the line asks how long to go without food, which is the hours and not this advice.
pub fn _the_line_wants_a_length(line: &str, reading: &Understood) -> bool {
    if text::strip(reading.wants.as_deref().unwrap_or("")).to_lowercase() != "time" {
        return false;
    }
    if _A_TEST_THAT_TAKES.is_match(line) {
        return false;
    }
    _A_FAST_LENGTH_ASK.is_match(line)
}

/// The card for the kind of appointment the line is about, or the general one.
pub fn _before_an_appointment(memory: &mut Memory, reading: &Understood) -> Option<String> {
    let mut line = text::squash(&reading.text).to_lowercase();
    let mut before = String::new();
    if let Some(leaned) = reading.leans_on.as_ref().map(|l| l.text.as_str()).filter(|t| !t.is_empty()) {
        // The kind is often in the line before: "i have a dentist appointment tomorrow" then
        // "do i need to do anything before I go".
        before = text::squash(leaned).to_lowercase();
    }
    if before.is_empty() || !any_in(&line, &_APPOINTMENT_KINDS) {
        let earlier = _the_line_before(memory, reading);
        if !earlier.is_empty() && !line.contains(&earlier) {
            before = text::strip(&format!("{before} {earlier}")).to_string();
        }
    }
    if !before.is_empty() {
        line = format!("{line} {before}");
    }
    if any_in(&line, &_NOT_A_MEDICAL_VISIT) && !any_in(&line, &_STILL_MEDICAL) {
        return None;
    }
    if any_in(&line, &_ITS_AN_OPERATION) && _a_dose_is_the_question(&line) {
        return None;
    }
    if any_in(&line, &_ITS_AN_OPERATION)
        && !any_in(&line, &_NOT_THE_FASTING)
        && !["gp surgery", "the surgery", "at the surgery", "surgery hours", "surgery number"].iter().any(|w| line.contains(w))
    {
        return Some("The hospital sets this one and you should not guess: usually nothing to eat for six hours before an anaesthetic and nothing clear to drink for two, and the letter or the pre-op call gives you the exact times. Ask about your regular tablets at the pre-op rather than deciding yourself, because most are continued but blood thinners are often stopped for a few days. Arrange for someone to bring you home, as you cannot drive or go home alone after a general anaesthetic.".into());
    }
    if any_in(&line, &_ITS_A_BLOOD_TEST) {
        if _the_line_wants_a_length(&line, reading) {
            return None;
        }
        return Some("Only fast if the letter or the person who booked it told you to: most blood tests need no fasting at all, and water is fine even when they do. Wear something with a short or loose sleeve so the arm is easy to get at, and keep your hands warm on the way in, which makes the vein easier to find. Keep taking your medicines as usual unless the letter says otherwise - thyroid and blood pressure tablets are meant to be taken, so ring the surgery rather than skipping one.".into());
    }
    if any_in(&line, &_ITS_A_SCAN) {
        return Some("Follow the letter rather than a general rule, because what to do differs: a plain x-ray needs nothing, an ultrasound often wants a full bladder, some scans want an empty stomach for a few hours, and a colonoscopy is the bowel prep it gives you plus nothing to eat after the time it names. Leave metal at home for an MRI - jewellery, keys, an underwired bra - and tell them if you have a pacemaker, a metal implant or could be pregnant. Take the letter with you and ring the department if any of it is unclear or you cannot make the time.".into());
    }
    if any_in(&line, &_ITS_THE_DENTIST) {
        return Some("For a check-up there is nothing to prepare: brush as usual and eat normally. Bring the names of the medicines you take, and say if you are on a blood thinner or have had a heart valve replaced, because they may want to check before treatment; tell them early if you are nervous, since they can plan the visit around it. Arrive a few minutes early, as the paperwork at a practice you have not been to before takes longer than the appointment.".into());
    }
    if any_in(&line, &_ITS_THE_OPTICIAN) {
        return Some("Take your current glasses, or the prescription if you have it, and any eye drops you use, and write down what you have noticed - blurring at a distance, trouble reading, glare at night - because the test is quicker when they know what to look for. Say if you drive: dilating drops blur your vision for a few hours, so you may want someone with you or the bus.".into());
    }
    Some("Read the letter or the text reminder first, because it says what to do before this one and most appointments need nothing. Worth doing every time: write down the two or three things you want to raise, most worrying first, and take the list of medicines you are on, or the boxes themselves. Arrive ten minutes early for a place you have not been to, and ring if you cannot make it - a missed slot usually means a long wait for the next one.".into())
}

/// Whether this line is a question about an appointment the line before it named.
pub fn _follows_an_appointment(understood: &Understood) -> bool {
    let Some(leans) = &understood.leans_on else { return false }; // `leans_on` is not a dict
    let before = [leans.text.as_str(), leans.subject.as_deref().unwrap_or(""), leans.topic.as_deref().unwrap_or("")].join(" ");
    let before = text::squash(&before).to_lowercase();
    if before.is_empty() {
        return false;
    }
    if any_in(&before, &_NOT_A_MEDICAL_VISIT) && !any_in(&before, &_STILL_MEDICAL) {
        return false;
    }
    any_in(&before, &_APPOINTMENT_KINDS)
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.appointments:_follows_an_appointment:291", |_: &FnRef, u: &Understood| _follows_an_appointment(u))];

pub const REPLIES: &[(&str, ReplyFn)] = &[(
    "bot.knowledge.appointments:_before_an_appointment:179",
    |_: &FnRef, m: &mut Memory, u: &Understood| _before_an_appointment(m, u),
)];
