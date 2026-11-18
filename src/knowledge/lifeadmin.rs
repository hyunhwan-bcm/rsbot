//! Port of `bot/knowledge/lifeadmin.py`: councils, complaints, names, orders, theft, tips,
//! post, passports, tenancy papers, fines, insurance excess, scams, legal words and libraries.
//! The entries and their tables are data in the bundle; these are the guards and replies.
//! (`costs.time_entries` is only used at import time to build entries.)

use crate::data;
use crate::data::FnRef;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! g {
    ($t:ty, $name:ident, $f:path) => {
        static $name: LazyLock<$t> = LazyLock::new(|| $f(concat!("bot.knowledge.lifeadmin.", stringify!($name))));
    };
}

g!(&'static Re, _THE_DRAIN, data::re);
g!(&'static Re, _A_STREET_PLACE, data::re);
g!(&'static Re, _ASKS_WHO, data::re);
g!(&'static Re, _ONLY_THE_ASK, data::re);
g!(Vec<String>, _NAME_CHILD, data::strs);
g!(Vec<String>, _NAME_DIVORCE, data::strs);
g!(Vec<String>, _CANCEL_SENT, data::strs);
g!(Vec<String>, _CANCEL_RETURN, data::strs);
g!(String, _CANCEL_WINDOW, data::string);
g!(String, _CANCEL_REFUSE, data::string);
g!(String, _CANCEL_REFUND, data::string);
g!(String, _CANCEL_AT_WAREHOUSE, data::string);
g!(Vec<String>, AWAY, data::strs);
g!(String, STOLEN_BAG, data::string);
g!(String, STOLEN_BAG_AWAY, data::string);
g!(String, STOLEN_THING_GEAR, data::string);
g!(&'static Re, _THE_SUM_IN_A_TIP_LINE, data::re);
g!(Vec<String>, _TIP_LEASEHOLD, data::strs);
g!(Vec<String>, _TIP_ABROAD, data::strs);
g!(Vec<String>, _TIP_HAIR, data::strs);
g!(Vec<String>, _TIP_TAXI, data::strs);
g!(Vec<String>, _TIP_DELIVERY, data::strs);
g!(Vec<String>, _TIP_ROUND, data::strs);
g!(Vec<String>, _TIP_TRADES, data::strs);
g!(Vec<String>, _TIP_TATTOO, data::strs);
g!(Vec<String>, _TIP_MOVERS, data::strs);
g!(Vec<String>, _TIP_GUIDE, data::strs);
g!(Vec<String>, _TIP_RESTAURANT, data::strs);
g!(Vec<String>, _TIP_PUB, data::strs);
g!(Vec<String>, _TIP_CAFE, data::strs);
g!(Vec<String>, _TIP_HOTEL, data::strs);
g!(Vec<String>, _TIP_CLEANER, data::strs);
g!(Vec<String>, _NOT_POSTED, data::strs);
g!(String, _PASSPORT_CLAIM, data::string);
g!(String, _PARCEL_CLAIM, data::string);
g!(&'static Re, _PHOTO_PRICE, data::re);
g!(&'static Re, _PHOTO_WHERE, data::re);
g!(&'static Re, _PHOTO_AT_HOME, data::re);
g!(&'static Re, _PHOTO_CHEAP, data::re);
g!(String, _PASSPORT_PHOTO_PRICE, data::string);
g!(String, _PASSPORT_PHOTO_PLACES, data::string);
g!(String, _PASSPORT_PHOTO_AT_HOME, data::string);
g!(String, _PASSPORT_PHOTO_RULES, data::string);
g!(String, _TENANCY_DEFINITION, data::string);
g!(HashSet<String>, _FINE_ADJECTIVE_AFTER, data::set);
g!(&'static Re, _A_FEELING_ABOUT_IT, data::re);
g!(&'static Re, _AN_ASK_ABOUT_A_PLACE, data::re);
g!(&'static Re, _FINE_AS_A_MOOD, data::re);
g!(Vec<String>, _FINE_IS_THE_NOUN, data::strs);
g!(&'static Re, _EXCESS, data::re);
g!(&'static Re, _EXCESS_PRICE_ASK, data::re);
g!(&'static Re, _A_CLAIM_OR_LOSS, data::re);
g!(&'static Re, _EXCESS_DICTIONARY_PHRASE, data::re);
g!(HashSet<String>, _EXCESS_DEFINITION_WORDS, data::set);
g!(&'static Re, _CON_IS_ANOTHER_WORD, data::re);
g!(&'static Re, _SOMEONE_ELSE_WAS_SCAMMED, data::re);
g!(&'static Re, _LIBRARY_RENEW, data::re);
g!(&'static Re, _LIBRARY_COUNT, data::re);
g!(&'static Re, _LIBRARY_RESERVE, data::re);
g!(&'static Re, _LIBRARY_LOST, data::re);
g!(&'static Re, _LIBRARY_LATE, data::re);
g!(&'static Re, _LIBRARY_RETURN, data::re);

/// `_STOLEN_THINGS`: (names, reply) in order.
static _STOLEN_THINGS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.lifeadmin._STOLEN_THINGS")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
});

/// `_LEGAL_WORDS`: (keys, reply) in order.
static _LEGAL_WORDS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.lifeadmin._LEGAL_WORDS")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
});

fn any_in(text: &str, words: &[String]) -> bool {
    words.iter().any(|w| text.contains(w.as_str()))
}

fn or<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.is_empty() { b } else { a }
}

/// `" ".join(str(reading.get(key) or "") for key in keys)` for clean / norm_text / text.
fn joined(understood: &Understood, keys: &[&str]) -> String {
    keys.iter()
        .map(|k| match *k {
            "clean" => understood.clean.as_str(),
            "norm_text" => understood.norm_text.as_str(),
            "text" => understood.text.as_str(),
            "plain" => understood.plain.as_str(),
            _ => "",
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// True unless the line asks who owns a drain on a street: then the council does.
pub fn _council_owns_the_drain(understood: &Understood) -> bool {
    let line = text::squash(&or(&understood.clean, &understood.text).to_lowercase());
    if !_THE_DRAIN.is_match(&line) {
        return true;
    }
    if _A_STREET_PLACE.is_match(&line) {
        return true;
    }
    !_ASKS_WHO.is_match(&line)
}

static COMPLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bcomplain", 0));

/// True only for a complaint that names nothing to complain about: "who do i complain to".
pub fn _a_complaint_with_no_subject(understood: &Understood) -> bool {
    let line = text::squash(&or(&understood.clean, &understood.text).to_lowercase());
    if !COMPLAIN.is_match(&line) {
        return false;
    }
    text::strip(&_ONLY_THE_ASK.sub(" ", &line)).is_empty()
}

/// A marriage, a divorce and a child are three different routes to a new surname.
pub fn _change_name(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = or(&understood.norm_text, &understood.text);
    if any_in(said, &_NAME_CHILD) {
        return Some(
            "To change a child's name, everyone with parental responsibility has to agree, or a court decides if they will not: one parent's deed poll is not enough while the child is under 16. Ask the other parent in writing first, keep the reply, and get advice before you pay anyone. At 16 and 17 they can make their own deed poll."
                .into(),
        );
    }
    if any_in(said, &_NAME_DIVORCE) {
        return Some(
            "A divorce does not change your name by itself: the decree absolute proves the divorce, and you use it with a deed poll - the free one, signed by two witnesses - to go back to your old name. Keeping the married name is allowed too. Then tell the passport office, the DVLA, your bank, HMRC, the council, your GP and your employer."
                .into(),
        );
    }
    Some(
        "After a marriage or civil partnership the certificate is your proof, and you can use the new name straight away: send it to the passport office, the DVLA, your bank, HMRC, the council, your GP and your employer, because nothing updates itself. Any other change needs a deed poll - the free one, signed by two witnesses, is accepted everywhere, so skip the firms that charge to print it."
            .into(),
    )
}

/// Lead with the half the line is about: the return window once it is out, the cancellation
/// while it is not, and the refund only as the last sentence.
pub fn _cancel_order_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = or(&understood.norm_text, &understood.text);
    if any_in(said, &_CANCEL_SENT) {
        return Some([_CANCEL_WINDOW.as_str(), _CANCEL_REFUSE.as_str(), _CANCEL_REFUND.as_str()].join(" "));
    }
    if any_in(said, &_CANCEL_RETURN) {
        return Some([_CANCEL_WINDOW.as_str(), _CANCEL_REFUND.as_str()].join(" "));
    }
    Some([_CANCEL_AT_WAREHOUSE.as_str(), _CANCEL_REFUSE.as_str(), _CANCEL_REFUND.as_str()].join(" "))
}

/// The same theft, but a bag taken abroad is reported to a different police force.
pub fn _stolen_bag(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = or(&understood.clean, &understood.text).to_lowercase();
    Some(if any_in(&said, &AWAY) { STOLEN_BAG_AWAY.clone() } else { STOLEN_BAG.clone() })
}

/// What was taken decides the reply: a bike, a tool, a parcel, or the general one.
pub fn _the_stolen_thing(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", or(&understood.clean, &understood.text).to_lowercase());
    for (names, reply) in _STOLEN_THINGS.iter() {
        if names.iter().any(|name| said.contains(&format!(" {name} ")) || said.contains(&format!(" {name}s "))) {
            return Some(reply.clone());
        }
    }
    Some(STOLEN_THING_GEAR.clone())
}

/// A pound figure said the way a person would write it: 84, 8.40, 1,020.
pub fn _a_sum(value: f64) -> String {
    if value >= 100.0 || (value - text::round_half_even(value) as f64).abs() < 0.005 {
        return format!("\u{a3}{}", text::round_half_even(value));
    }
    format!("\u{a3}{:.2}", value)
}

/// The whole pounds inside a range, said the way a person would: "9 or 10", "10".
pub fn _whole_pounds(low: f64, high: f64) -> String {
    let first = low.trunc() as i64 + if low > low.trunc() { 1 } else { 0 };
    let last = high.trunc() as i64;
    if first > last {
        return format!("about {}", _a_sum(text::round_half_even((low + high) / 2.0) as f64));
    }
    if first == last {
        return _a_sum(first as f64);
    }
    format!("{} or {}", _a_sum(first as f64), _a_sum(last as f64))
}

static NOT_A_SUM_AFTER: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"(?:percent|%|people|of us|pints|drinks|hours?|oclock|o.clock)\b", 0));

/// The sum a tip line is worked out on, or None when it names none.
pub fn _the_bill_amount(said: &str) -> Option<f64> {
    for m in _THE_SUM_IN_A_TIP_LINE.finditer(said) {
        let Ok(value) = m.str(1).parse::<f64>() else { continue };
        let at = text::char_index(said, m.end()) as isize;
        let after_raw = text::slice(said, at, Some(at + 12));
        let after = text::strip(&after_raw);
        if NOT_A_SUM_AFTER.match_(after).is_some() {
            continue;
        }
        if value < 5.0 {
            continue;
        }
        return Some(value);
    }
    None
}

static TIP_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// The tip for the thing the line names, in the first sentence.
pub fn _tipping(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = or(&understood.norm_text, &understood.text).to_lowercase();
    let words: HashSet<String> = TIP_WORDS.findall_str(&said).into_iter().collect();
    let has = |group: &[String]| -> bool {
        group.iter().any(|phrase| if phrase.contains(' ') { said.contains(phrase.as_str()) } else { words.contains(phrase) })
    };
    if has(&_TIP_LEASEHOLD) {
        return Some(String::new());
    }
    if has(&_TIP_ABROAD) {
        return Some(String::new());
    }
    let hair_taxi_delivery: Vec<String> =
        _TIP_HAIR.iter().chain(_TIP_TAXI.iter()).chain(_TIP_DELIVERY.iter()).cloned().collect();
    if said.contains("service charge") && !has(&hair_taxi_delivery) {
        return Some("A service charge is a tip that the restaurant has already added to the bill, usually 10 to 15 percent, and it is discretionary rather than law: you can ask for it to be taken off or reduced, and they have to show it as a separate line. If it is on there, that is the tip and you do not leave another.".into());
    }
    if has(&_TIP_HAIR) {
        return Some("A pound or two, or round it up: a hairdresser or barber usually keeps the change and a salon is fine with nothing, so ten percent is generous rather than expected. It is the same on card or in cash, and nobody will chase you for it.".into());
    }
    if has(&_TIP_TAXI) {
        return Some("Round up, or ten percent on a fare: a couple of pounds on a twelve pound ride is plenty, and paying the meter exactly is fine too. Taxi drivers here are not paid on tips the way they are in the States.".into());
    }
    if has(&_TIP_ROUND) {
        return Some("Not the rest of the year: a postman, a milkman or a bin crew is paid a wage and nothing is expected. At Christmas a card with a note, or a fiver in it for a crew you know, is the usual thank you and is optional - and for a postman a card matters more than the money.".into());
    }
    if has(&_TIP_DELIVERY) {
        return Some("Nothing is expected: the delivery fee is not a tip, and most drivers see none of it, so a pound in the app or in cash is a nice extra rather than something you owe. Round up if the weather is filthy.".into());
    }
    if has(&_TIP_TRADES) {
        return Some("No: a plumber, an electrician or a builder quotes a price for the job, and that price is the whole of it. A tip is not expected and can read as odd, so the thing worth doing instead is offering a tea and paying on time.".into());
    }
    if has(&_TIP_TATTOO) {
        return Some("Ten to fifteen percent is the going rate, and more on a big piece that ran over its time: the artist's time is the product, and a tip is the usual thank you. Cash is still welcome even where the studio takes cards.".into());
    }
    if has(&_TIP_MOVERS) {
        return Some("Ten to twenty pounds each for a hard day, a long carry or a lot of stairs, and a round of cold drinks if they will not take cash. Agree it between you before they start rather than handing money over at the door.".into());
    }
    if has(&_TIP_GUIDE) {
        return Some("Ten percent on a booked tour or transfer, and the same on a coach excursion where the guide is not the driver. On a free walking tour the tip is the pay, so ten to fifteen is the norm and a fiver is not an insult.".into());
    }
    if ["wedding", "reception", "hen do", "stag do"].iter().any(|w| said.contains(w)) {
        return Some("Nothing is owed to the venue: a service charge on a wedding bill is worth asking about when you book, and it is often 10 to 12 percent. The people who do get something are the band, the photographer and the bar staff on the night, and the venue usually passes a bar tip on if you ask.".into());
    }
    if has(&_TIP_RESTAURANT) {
        return Some("Ten to twelve and a half percent is the norm, and check the bill first: if a service charge is already on it, that is the tip and you add nothing. If the service was poor, ask for it to come off before you pay.".into());
    }
    if has(&_TIP_PUB) {
        return Some("In a pub, nothing: you order and pay at the bar, so there is no table service to tip. Round up a pound if food was brought to your table, and never feel you have to.".into());
    }
    if has(&_TIP_CAFE) {
        return Some("A pound in the jar or nothing at all: a cafe is the one place a tip is entirely optional, and the person behind the counter does not expect it either way.".into());
    }
    if has(&_TIP_HOTEL) {
        return Some("A pound or two a night for the cleaner, left with a note or at reception, and a pound or two for a porter who carries your bags. It is a thank you rather than part of the price, so nothing is fine too.".into());
    }
    if has(&_TIP_CLEANER) {
        return Some("Not a tip: pay the hourly rate you agreed, on time, and give a Christmas bonus or a paid day off rather than loose change each visit. The same goes for a nanny or a babysitter - round the evening up to the next hour instead.".into());
    }
    if let Some(bill) = _the_bill_amount(&said) {
        let (low, high) = (bill * 0.10, bill * 0.125);
        let mut answer = format!(
            "On {}, ten percent is {} and twelve and a half is {}, so {} to {}",
            _a_sum(bill),
            _a_sum(low),
            _a_sum(high),
            _a_sum(low),
            _a_sum(high)
        );
        if high - low >= 2.0 {
            answer += &format!(", and {} is what people leave", _whole_pounds(low, high));
        }
        return Some(answer + ". Check the bill for a service charge first, because that is the tip and you add nothing on top.");
    }
    Some("It is fine not to tip in the UK: wages do not depend on it the way they do in the States. Ten to twelve and a half percent in a restaurant, once you have checked the service charge, a couple of pounds to a taxi, and a pound or nothing in a cafe, a pub or a hairdresser.".into())
}

/// True unless the thing sent is a medium that is never posted.
pub fn _a_posted_thing(understood: &Understood) -> bool {
    let said = joined(understood, &["clean", "norm_text", "text"]).to_lowercase();
    if said.is_empty() {
        return true;
    }
    !any_in(&said, &_NOT_POSTED)
}

/// The claim, or the document route when the thing that went missing is a passport.
pub fn _claim_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = joined(understood, &["clean", "norm_text", "text"]).to_lowercase();
    if said.contains("passport") || said.contains("passport office") || said.contains("travel document") {
        return Some(_PASSPORT_CLAIM.clone());
    }
    Some(_PARCEL_CLAIM.clone())
}

/// The passport photo line answered in the form it was asked: where, how much, or the rules.
pub fn _the_photo_answer(text: &str) -> String {
    if _PHOTO_PRICE.is_match(text) {
        return _PASSPORT_PHOTO_PRICE.clone();
    }
    if _PHOTO_WHERE.is_match(text) {
        return _PASSPORT_PHOTO_PLACES.clone();
    }
    if _PHOTO_AT_HOME.is_match(text) {
        return _PASSPORT_PHOTO_AT_HOME.clone();
    }
    if _PHOTO_CHEAP.is_match(text) {
        return _PASSPORT_PHOTO_PRICE.clone();
    }
    _PASSPORT_PHOTO_RULES.clone()
}

static PQ_PHOTO: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bphotos?\b|\bphoto rules\b|\bpictures?\b", 0));
static PQ_EXPIRED: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\bhas expired\b|\bexpired\b|\bout of date\b|\bran out\b|\bhas run out\b", 0));
static PQ_RUNS_OUT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\bexpires? (?:in|on|at|next)\b|\bexpires? soon\b|\bexpiring\b|\bruns out\b|",
            r"\bout in (?:january|february|march|april|may|june|july|august|september|",
            r"october|november|december|jan|feb|mar|apr|jun|jul|aug|sep|sept|oct|nov|dec)\b"
        ),
        0,
    )
});
static PQ_CHILD: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\b(?:my|our) (?:son|daughter|child|boy|girl|baby|kid|kids|children)\b|",
            r"\bchild'?s passport\b|\bbaby'?s passport\b"
        ),
        0,
    )
});
static PQ_VALID: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\bvalid (?:for|until|till)\b|\bhow long (?:is|are)\b[^?]*\bvalid\b|",
            r"\bhow many years\b|",
            r"\blast(?:s|ed)?\b(?!\s(?:year|night|week|weekend|month|summer|winter|spring|",
            r"autumn|christmas|time|sunday|monday|tuesday|wednesday|thursday|friday|",
            r"saturday))|\bexpires? after\b|",
            r"\bwhen (?:does|do|will|did)\b[^?]*\bexpire\b|\bexpiry date\b|",
            r"\bexpires?\b|\bexpiring\b"
        ),
        0,
    )
});
static PQ_URGENT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\burgent(?:ly)?\b|\bquickly\b|\bquick\b|\basap\b|\bhurry\b|",
            r"\brush\b|\bfast track\b|\bfast\b|\btomorrow\b|\bnext week\b|",
            r"\bthis week\b|\bin a (?:few )?days\b|\bemergency\b"
        ),
        0,
    )
});
static PQ_HOW_LONG: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow long\b|\bwait(?:ing)?\b|\bsoon\b|\bprocessing\b", 0));

/// Applying for, renewing or photographing a passport: what it takes and how long to allow.
pub fn _passport_queue(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = understood.clean.as_str();
    if PQ_PHOTO.is_match(text) {
        return Some(_the_photo_answer(text));
    }
    if PQ_EXPIRED.is_match(text) {
        return Some("An expired passport cannot be extended, so that is a renewal: apply now rather than at the airport, because the old one is cancelled before the new one is issued. You need the old passport, a recent digital photo and payment, and it is up to ten weeks, so do not book anything you cannot cancel until the new one is in your hand.".into());
    }
    if PQ_RUNS_OUT.is_match(text) {
        return Some("Start the renewal now rather than waiting for it to lapse: allow up to ten weeks, and check the rule for where you are going, because most countries want six months left on your passport beyond the day you come back. Do not book anything you cannot move until the new one is in your hand.".into());
    }
    if PQ_CHILD.is_match(text) {
        return Some("A child's first passport is its own application: you need their birth certificate and the other parent's details as well as a photo and payment, and both people with parental responsibility usually have to be accounted for. Apply once you have the certificate, because that is the thing that holds it up, and allow up to ten weeks.".into());
    }
    if PQ_VALID.is_match(text) {
        return Some("Ten years for an adult passport and five for a child's, counted from the issue date, and a renewal keeps the same number. That is how long it lasts, not how long it takes: most countries also want six months left on it beyond the day you come back, so check the dates against the trip before you book anything.".into());
    }
    if PQ_URGENT.is_match(text) {
        return Some("The fast routes are a one-week fast-track application and a premium appointment at a passport office for a same-day decision, and both cost more than the standard fee. Take your old passport, a photo and proof of the travel date to an appointment, or send them with a fast-track application, and check the fee and the wait on your government's passport site first rather than ringing round.".into());
    }
    if PQ_HOW_LONG.is_match(text) {
        return Some("Allow up to ten weeks for a standard application, and treat that as the planning figure rather than a promise: check the current wait on your government's passport site, because it moves with the season. If you need it sooner, that site sells a faster service and a premium appointment - pay for it, keep the reference, and do not book anything you cannot move until the passport is in your hand.".into());
    }
    Some("Apply or renew through your government's passport service, online if it allows it: you need your current or most recent passport, a recent digital photo and payment, and the old passport goes back by post if they ask for it. Allow up to ten weeks, and do not book travel until the new one is in your hand - the fee and the current wait are on that site, because both change.".into())
}

/// The tenancy agreement: what it is, or how to get a copy of it.
pub fn _tenancy_copy(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!(
        " {} ",
        [&understood.clean, &understood.text, &understood.norm_text].iter().map(|s| s.to_lowercase()).collect::<Vec<_>>().join(" ")
    );
    if [" what is ", " whats ", " what's ", " explain ", " what does ", " mean ", " means "].iter().any(|m| said.contains(m)) {
        return Some(_TENANCY_DEFINITION.clone());
    }
    Some("Your landlord has to give you a copy of the tenancy agreement, so ask in writing - a text or email is enough - and keep it. Ask in the same request for the deposit protection certificate, the gas safety certificate and the energy performance certificate: none of those are optional. If they still will not, your council's tenancy relations officer can put pressure on, and the deposit scheme will confirm whether it was protected.".into())
}

static FINE_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bfine\b(?:\s+([a-z0-9']+))?", 0));

/// True when at least one "fine" in the line is the noun, not a known adjective.
pub fn _a_fine_is_a_paper(line: &str) -> bool {
    for m in FINE_WORD.finditer(line) {
        match m.get(1) {
            None => return true,
            Some(word) if !_FINE_ADJECTIVE_AFTER.contains(word) => return true,
            _ => {}
        }
    }
    false
}

/// The line the user typed, from whichever field the reading carries it in.
pub fn _the_line(understood: &Understood) -> String {
    let line = or(&understood.clean, &understood.text);
    if !line.is_empty() {
        return line.to_string();
    }
    understood.words.join(" ")
}

/// Whether the line asks about a place rather than saying something about a school.
pub fn _an_ask_about_a_place(understood: &Understood) -> bool {
    let text = _the_line(understood);
    if _A_FEELING_ABOUT_IT.is_match(&text) {
        return false;
    }
    _AN_ASK_ABOUT_A_PLACE.is_match(&text)
}

static NOT_FINE_CHARS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));
static FINED: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bfined\b|\bfines\b", 0));

/// True unless the line is using "fine" as an adjective with nothing about a court in it.
pub fn _the_fine_is_the_noun(understood: &Understood) -> bool {
    let line = format!(" {} ", NOT_FINE_CHARS.sub(" ", &or(&understood.clean, &understood.text).to_lowercase()));
    if FINED.is_match(&line) {
        return true;
    }
    if _FINE_AS_A_MOOD.is_match(&line) {
        return any_in(&line, &_FINE_IS_THE_NOUN);
    }
    _a_fine_is_a_paper(&line) || any_in(&line, &_FINE_IS_THE_NOUN)
}

/// False when the line asks what an insurance excess costs, which the price row answers.
pub fn _the_excess_is_a_price_question(understood: &Understood) -> bool {
    let text = or(&understood.clean, &understood.text);
    !(_EXCESS.is_match(text) && _EXCESS_PRICE_ASK.is_match(text) && !_A_CLAIM_OR_LOSS.is_match(text))
}

static WORDS_09: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", 0));

/// True when the line asks what a voluntary or compulsory excess is, and not the dictionary row's.
pub fn _the_excess_is_what_it_means(understood: &Understood) -> bool {
    let text = or(&understood.clean, &understood.text);
    if _EXCESS_DICTIONARY_PHRASE.is_match(text) {
        return false;
    }
    if _EXCESS_PRICE_ASK.is_match(text) || _A_CLAIM_OR_LOSS.is_match(text) {
        return false;
    }
    let words = WORDS_09.findall_str(&text.to_lowercase());
    if words.is_empty() || words.iter().any(|w| w == "my" || w == "our") {
        return false;
    }
    words.iter().all(|w| _EXCESS_DEFINITION_WORDS.contains(w))
}

static CONS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bcons?\b|\bconned\b", 0));
static CONNED: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bconned\b", 0));

/// False when "con" in the line belongs to air con, con carne or a list of pros and cons.
pub fn _con_is_the_question(understood: &Understood) -> bool {
    let text = text::squash(&understood.clean.to_lowercase());
    if !CONS.is_match(&text) {
        return true;
    }
    if CONNED.is_match(&text) {
        return true;
    }
    !_CON_IS_ANOTHER_WORD.is_match(&text)
}

/// Whoever was caught out is the one who rings the bank.
pub fn _scammed_already(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if _SOMEONE_ELSE_WAS_SCAMMED.is_match(&understood.clean) {
        return Some("Get them to ring their bank today, on the number on the back of their card, and say it was a scam: a transfer can sometimes be stopped or recalled, and nobody is in trouble for reporting it. Report it as well - Action Fraud in the UK, or your country's fraud service - and keep every message, number and receipt, because the report needs them. Then change the password on any account they touched, and expect a follow-up call from someone pretending to be the bank: no real bank asks for a code, a password or a transfer, so hang up and ring back on the number they already know.".into());
    }
    Some("Ring your bank today, on the number on the back of your card, and tell them it was a scam: a transfer can sometimes be stopped or recalled, and you are not in trouble for reporting it. Then report it - Action Fraud in the UK, or your country's fraud service - and keep every message, number and receipt, because the report needs them. Change the password on any account they touched, and expect a follow-up call from someone pretending to be the bank: no real bank asks for a code, a password or a transfer, so hang up and ring back on the number you know.".into())
}

/// The word the line asks about, defined in the first sentence.
///
/// Python takes `(first=None, second=None)` and reads whichever carries the reading: called as
/// `reply(memory, reading)` that is the reading (the memory's own `understood` is the fallback).
pub fn _legal_word(memory: Option<&Memory>, understood: Option<&Understood>) -> Option<String> {
    let reading = understood.or_else(|| memory.and_then(|m| m.understood.as_ref()));
    let words: Vec<String> = reading.map(|u| u.words.iter().map(|w| w.to_lowercase()).collect()).unwrap_or_default();
    let text = words.join(" ");
    for (keys, reply) in _LEGAL_WORDS.iter() {
        for key in keys {
            if (key.contains(' ') && text.contains(key.as_str())) || words.contains(key) {
                return Some(reply.clone());
            }
        }
    }
    None
}

/// The library's answer to the question the line asked about its loan.
pub fn _the_library_loan(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = format!(" {} ", text::squash(&or(&reading.clean, &reading.text).to_lowercase()));
    if _LIBRARY_RENEW.is_match(&said) {
        return Some("Renew it online, in the library's app or by phone, before the date falls: most libraries allow two or three renewals unless somebody has reserved the book, and the renewal is what to try first because it costs nothing and stops a fine. Where the renewals have run out, ask at the desk - a librarian can usually override it for one more if nobody is waiting.".into());
    }
    if _LIBRARY_COUNT.is_match(&said) {
        return Some("About a dozen at a time at most libraries, twenty at the big city ones, and a child's card is usually half that - the number is on the card's page or at the desk, and it counts everything, so a DVD or a jigsaw uses one of the dozen. Reference books and anything in the local-history room never leave the building.".into());
    }
    if _LIBRARY_RESERVE.is_match(&said) {
        return Some("Reserve it from the library's own catalogue or at the desk and it comes to your branch when it is free - nothing at most libraries and a small charge at some. You get an email or a text when it arrives and a week or so to collect it, so choose the branch you actually use, and ask for a book that is on the shelf elsewhere in the county now rather than waiting for a copy to come back.".into());
    }
    if _LIBRARY_LOST.is_match(&said) {
        return Some("Tell the desk as soon as you know: a lost or damaged book is charged at the replacement price, not the cover price and not a fine, and staff can often renew it once more to give you time to find it. If it turns up, the charge is refunded at most libraries, and a book damaged by a child or a dog is the same conversation - say what happened, because a librarian decides, not a rule.".into());
    }
    if _LIBRARY_LATE.is_match(&said) {
        return Some("Renew it first, online or by phone, because that costs nothing and clears most of it. Where a book is late, most councils have dropped the daily fine altogether and simply block the card until it comes back; where there is one it is pennies a day and capped, and a child's card is never charged. Take it back and tell the desk - they can usually unfreeze the card there and then.".into());
    }
    if _LIBRARY_RETURN.is_match(&said) {
        return Some("Returns go in the drop box inside or outside the branch, so a book can go back at midnight, and the date it is checked in is the date it came back, not the date somebody got round to it. Where there is no drop box, any branch in the same council will take it - it does not have to go back where it came from.".into());
    }
    Some("Three weeks is the usual loan, and most libraries renew it two or three times online or by phone unless somebody has reserved it, which is the difference between a three-week book and one you can keep for two months.".into())
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.lifeadmin:_a_complaint_with_no_subject:69", |_: &FnRef, u: &Understood| _a_complaint_with_no_subject(u)),
    ("bot.knowledge.lifeadmin:_a_posted_thing:457", |_: &FnRef, u: &Understood| _a_posted_thing(u)),
    ("bot.knowledge.lifeadmin:_an_ask_about_a_place:764", |_: &FnRef, u: &Understood| _an_ask_about_a_place(u)),
    ("bot.knowledge.lifeadmin:_con_is_the_question:4460", |_: &FnRef, u: &Understood| _con_is_the_question(u)),
    ("bot.knowledge.lifeadmin:_council_owns_the_drain:45", |_: &FnRef, u: &Understood| _council_owns_the_drain(u)),
    ("bot.knowledge.lifeadmin:_the_excess_is_a_price_question:804", |_: &FnRef, u: &Understood| {
        _the_excess_is_a_price_question(u)
    }),
    ("bot.knowledge.lifeadmin:_the_excess_is_what_it_means:827", |_: &FnRef, u: &Understood| {
        _the_excess_is_what_it_means(u)
    }),
    ("bot.knowledge.lifeadmin:_the_fine_is_the_noun:772", |_: &FnRef, u: &Understood| _the_fine_is_the_noun(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.lifeadmin:_cancel_order_reply:126", |_: &FnRef, m: &mut Memory, u: &Understood| _cancel_order_reply(m, u)),
    ("bot.knowledge.lifeadmin:_change_name:85", |_: &FnRef, m: &mut Memory, u: &Understood| _change_name(m, u)),
    ("bot.knowledge.lifeadmin:_claim_reply:494", |_: &FnRef, m: &mut Memory, u: &Understood| _claim_reply(m, u)),
    ("bot.knowledge.lifeadmin:_legal_word:4702", |_: &FnRef, m: &mut Memory, u: &Understood| _legal_word(Some(m), Some(u))),
    ("bot.knowledge.lifeadmin:_passport_queue:575", |_: &FnRef, m: &mut Memory, u: &Understood| _passport_queue(m, u)),
    ("bot.knowledge.lifeadmin:_scammed_already:4525", |_: &FnRef, m: &mut Memory, u: &Understood| _scammed_already(m, u)),
    ("bot.knowledge.lifeadmin:_stolen_bag:231", |_: &FnRef, m: &mut Memory, u: &Understood| _stolen_bag(m, u)),
    ("bot.knowledge.lifeadmin:_tenancy_copy:671", |_: &FnRef, m: &mut Memory, u: &Understood| _tenancy_copy(m, u)),
    ("bot.knowledge.lifeadmin:_the_library_loan:4879", |_: &FnRef, m: &mut Memory, u: &Understood| _the_library_loan(m, u)),
    ("bot.knowledge.lifeadmin:_the_stolen_thing:283", |_: &FnRef, m: &mut Memory, u: &Understood| _the_stolen_thing(m, u)),
    ("bot.knowledge.lifeadmin:_tipping:334", |_: &FnRef, m: &mut Memory, u: &Understood| _tipping(m, u)),
];
