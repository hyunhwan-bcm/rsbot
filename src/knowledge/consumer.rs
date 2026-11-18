//! Port of `bot/knowledge/consumer.py`: refunds, faulty goods, bookings, deposits, fees,
//! renewals, returns and the other consumer-rights rows. The entries and their word tables are
//! data in the bundle; these are the guards, replies and the readers they share.
//! (`_age_in_months` reads the word list; `_change_of_mind_verdict` is an entry's `lead` and
//! `_a_stretch_of_waiting` an entry's `shape`, both called by the lookup.)

use crate::data;
use crate::data::FnRef;
use crate::knowledge::d2dup_g8::cooking_without;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

macro_rules! g {
    ($t:ty, $name:ident, $f:path) => {
        static $name: LazyLock<$t> = LazyLock::new(|| $f(concat!("bot.knowledge.consumer.", stringify!($name))));
    };
}

g!(Vec<String>, CAME_OFF_THE_BELT, data::strs);
g!(Vec<String>, EMPLOYER_WORDS, data::strs);
g!(String, FAULTY_REPLY, data::string);
g!(Vec<String>, FRIDGE_IS_THE_QUESTION, data::strs);
g!(Vec<String>, GONE_MISSING, data::strs);
g!(Vec<String>, HOME_ROOM_WORDS, data::strs);
g!(Vec<String>, HOTEL_ROOM_WORDS, data::strs);
g!(Vec<String>, KITCHEN_QUIET, data::strs);
g!(Vec<String>, MACHINES, data::strs);
g!(String, SIX_MONTH_PASSED, data::string);
g!(String, SIX_MONTH_REVERSAL, data::string);
g!(Vec<String>, THE_MONEY, data::strs);
g!(Vec<String>, THINGS, data::strs);
g!(Vec<String>, TRAVEL_SAID, data::strs);
g!(&'static Re, _APPLIANCE_TROUBLE, data::re);
g!(&'static Re, _A_A_SHOP_OR_A_THING, data::re);
g!(&'static Re, _A_BICYCLE_CABLE, data::re);
g!(&'static Re, _A_BUYING_LINE, data::re);
g!(&'static Re, _A_CLAIM_AGAINST_A_SELLER, data::re);
g!(&'static Re, _A_CLAIM_ON_IT, data::re);
g!(&'static Re, _A_COUNT_FOLLOW_UP, data::re);
g!(Vec<String>, _A_DAMP, data::strs);
g!(&'static Re, _A_DEPOSIT_PAID, data::re);
g!(&'static Re, _A_FIGURE_JUDGED, data::re);
g!(Vec<String>, _A_FIX, data::strs);
g!(&'static Re, _A_HOW_TO_ASK, data::re);
g!(&'static Re, _A_MACHINE, data::re);
g!(&'static Re, _A_MACHINE_THAT_WEARS_OUT, data::re);
g!(&'static Re, _A_MARK_ON_ANYTHING, data::re);
g!(&'static Re, _A_MARK_THAT_CAME_WITH_IT, data::re);
g!(&'static Re, _A_PLACE_THAT_TAKES_BOOKINGS, data::re);
g!(&'static Re, _A_PURCHASE_TRACE, data::re);
g!(&'static Re, _A_REPAIR_ASK, data::re);
g!(&'static Re, _A_REPAIR_OR_REPLACE, data::re);
g!(&'static Re, _A_SAVING_ACT, data::re);
g!(&'static Re, _A_SHOP_CLAIM, data::re);
g!(&'static Re, _A_STAKE_IN_A_COMPANY, data::re);
g!(&'static Re, _A_WAIT, data::re);
g!(HashSet<String>, _BEFORE_THE_OBJECT, data::set);
g!(&'static Re, _BUY_OR_MEND, data::re);
g!(Vec<String>, _CANCEL_FEE_WORDS, data::strs);
g!(Vec<String>, _CANCEL_WORDS, data::strs);
g!(Vec<String>, _DELIVERY_FEE_WORDS, data::strs);
g!(Vec<String>, _DESCRIBED_PHRASES, data::strs);
g!(HashSet<String>, _DESCRIBED_WORDS, data::set);
g!(&'static Re, _FITTED_TO_THE_FLAT, data::re);
g!(Vec<String>, _FLAT_FEE_WORDS, data::strs);
g!(&'static Re, _FOOD_NAMED, data::re);
g!(&'static Re, _GIVES_UP, data::re);
g!(Vec<String>, _GRIEVANCE, data::strs);
g!(String, _HOLDING_KEPT, data::string);
g!(String, _HOLDING_WHAT, data::string);
g!(&'static Re, _IT_ARRIVED, data::re);
g!(&'static Re, _I_AM_THE_ONE_CANCELLING, data::re);
g!(Vec<String>, _LATE_FEE_WORDS, data::strs);
g!(&'static Re, _MIND_LIKE, data::re);
g!(&'static Re, _NAMES_THE_LANDLORD, data::re);
g!(Vec<String>, _NEVER_HAPPENED_WORDS, data::strs);
g!(Vec<String>, _OTHER_FEE_WORDS, data::strs);
g!(HashSet<String>, _PENSION_COMBINE_WORDS, data::set);
g!(Vec<String>, _PENSION_SEVERAL_WORDS, data::strs);
g!(Vec<String>, _PENSION_STOPPED_WORDS, data::strs);
g!(Vec<String>, _PHONE_WORDS, data::strs);
g!(HashSet<String>, _PURCHASE_IN_THE_LINE, data::set);
g!(&'static Re, _RENTED_HOME, data::re);
g!(Vec<String>, _SALE_TAG_WORDS, data::strs);
g!(&'static Re, _SELLING_WORDS, data::re);
g!(Vec<String>, _SERVICES_HAIR_WORDS, data::strs);
g!(&'static Re, _SOMEONE_PUT_THE_PHONE_DOWN, data::re);
g!(&'static Re, _STILL_TIME, data::re);
g!(&'static Re, _THE_NOTICE_IS_GIVEN, data::re);
g!(&'static Re, _TOO_LATE, data::re);
g!(Vec<String>, _TRIAL_WORDS, data::strs);
g!(Vec<String>, _WAS_USED_WORDS, data::strs);
g!(HashSet<String>, _WHAT_PEOPLE_QUIT, data::set);

/// `_AGE_COUNTS`: a number word -> its count ("six" -> 6).
static _AGE_COUNTS: LazyLock<HashMap<String, i64>> = LazyLock::new(|| {
    data::get("bot.knowledge.consumer._AGE_COUNTS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_int()?)))
        .collect()
});

/// `_CONDITION_LEAD`: condition -> the returns card's lead.
static _CONDITION_LEAD: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| data::str_map("bot.knowledge.consumer._CONDITION_LEAD"));

/// `_SERVICES_KINDS`: (tokens, phrases, sentence) per kind of job.
static _SERVICES_KINDS: LazyLock<Vec<(Vec<String>, Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.consumer._SERVICES_KINDS")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].strs(), row.items()[2].str().to_string()))
        .collect()
});

static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

fn any_in(text: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|p| text.contains(p.as_str()))
}

fn or<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.is_empty() { b } else { a }
}

/// `" ".join(words).lower()`, or the clean (else typed) line lowered when there are no words.
fn words_or_line(understood: &Understood) -> String {
    let text = understood.words.join(" ").to_lowercase();
    if !text.is_empty() {
        return text;
    }
    or(&understood.clean, &understood.text).to_lowercase()
}

/// `" ".join((clean, text, raw)).lower() + " " + " ".join(lowered words)`; a reading has no
/// `raw`, so that slot is "".
fn line_and_words(understood: &Understood) -> String {
    format!(
        "{} {} {} {}",
        understood.clean.to_lowercase(),
        understood.text.to_lowercase(),
        "",
        understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ")
    )
}

/// How old the line says the thing is, in months, or None if it does not say.
pub fn _age_in_months(reading: &Understood) -> Option<i64> {
    let words = &reading.words;
    for (i, word) in words.iter().enumerate() {
        if !["month", "months", "year", "years", "yr", "yrs"].contains(&word.as_str()) {
            continue;
        }
        let before = if i > 0 { words[i - 1].as_str() } else { "" };
        let mut count = _AGE_COUNTS.get(before).copied();
        if count.is_none() {
            // TYPE: Python's isdigit/int also take non-ASCII digits; ASCII here.
            let digits: String = before.chars().filter(|c| c.is_ascii_digit()).collect();
            count = if digits.is_empty() { None } else { Some(digits.parse::<i64>().unwrap_or(i64::MAX / 12)) };
        }
        let Some(count) = count else { continue };
        return Some(if word.starts_with("yr") || word.starts_with("year") { count * 12 } else { count });
    }
    None
}

/// Whether the line is asking whether a figure they have is a normal one.
pub fn _not_a_figure_judged(understood: &Understood) -> bool {
    let mut text = understood.words.join(" ").to_lowercase();
    if text.is_empty() {
        text = or(&understood.clean, &understood.text).to_string();
    }
    !_A_FIGURE_JUDGED.is_match(&text)
}

/// Whether the mark is the delivery's rather than a fault found later.
pub fn _a_mark_that_came_with_it(understood: &Understood) -> bool {
    let text = words_or_line(understood);
    !(_IT_ARRIVED.is_match(&text) && _A_MARK_THAT_CAME_WITH_IT.is_match(&text))
}

/// Whether a line with a stain in it is about the thing being bought rather than cleaned.
pub fn _a_stain_a_shop_answers_for(understood: &Understood) -> bool {
    let text = words_or_line(understood);
    if !_A_MARK_ON_ANYTHING.is_match(&text) {
        return true;
    }
    _A_BUYING_LINE.is_match(&text)
}

/// False for a blame with no shop and no thing in it: that is not a sale yet.
pub fn _the_shop_or_the_thing_is_named(understood: &Understood) -> bool {
    _A_A_SHOP_OR_A_THING.is_match(&words_or_line(understood))
}

/// False for a how-to that says nothing about buying, selling or the rights.
pub fn _not_a_how_to_about_their_own_thing(understood: &Understood) -> bool {
    let text = words_or_line(understood);
    if !_A_HOW_TO_ASK.is_match(&text) {
        return true;
    }
    _A_BUYING_LINE.is_match(&text)
}

/// False when the deposit is a savings goal rather than a payment already made.
pub fn _not_saving_for_a_deposit(understood: &Understood) -> bool {
    let text = words_or_line(understood);
    if !_A_SAVING_ACT.is_match(&text) {
        return true;
    }
    _A_DEPOSIT_PAID.is_match(&text)
}

/// False when the line is deciding whether to mend the thing or buy a new one.
pub fn _not_a_repair_or_replace_decision(understood: &Understood) -> bool {
    !_A_REPAIR_OR_REPLACE.is_match(&words_or_line(understood))
}

/// False when an old machine in the house is being weighed up for repair or replacement.
pub fn _not_an_old_machines_repair_decision(understood: &Understood) -> bool {
    if _not_a_repair_or_replace_decision(understood) {
        return true;
    }
    let text = words_or_line(understood);
    if _A_CLAIM_ON_IT.is_match(&text) {
        return true;
    }
    !_A_MACHINE_THAT_WEARS_OUT.is_match(&text)
}

/// Whether the line names something a booking is taken for, rather than money held on a home.
pub fn _a_place_that_takes_bookings(understood: &Understood) -> bool {
    _A_PLACE_THAT_TAKES_BOOKINGS.is_match(&words_or_line(understood))
}

/// Whether the line is about the place calling the booking off rather than the person.
pub fn _they_are_the_ones_cancelling(understood: &Understood) -> bool {
    !_I_AM_THE_ONE_CANCELLING.is_match(&words_or_line(understood))
}

/// False when the line's dying phrase is a habit the person gave up instead.
pub fn _the_device_gave_up(understood: &Understood) -> bool {
    let mut words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if words.is_empty() {
        words = text::split(&or(&understood.clean, &understood.text).to_lowercase());
    }
    if words.is_empty() {
        return true;
    }
    if words.iter().any(|w| THINGS.contains(w)) {
        return true;
    }
    let joined = words.join(" ");
    let Some(hit) = _GIVES_UP.search(&joined) else { return true };
    let mut after: Vec<&str> = text::split_ref(&joined[hit.end()..]);
    while !after.is_empty() && _BEFORE_THE_OBJECT.contains(after[0]) {
        after.remove(0);
    }
    let Some(object_word) = after.first() else { return true };
    if *object_word == "ghost" {
        return true; // "the tv gave up the ghost"
    }
    if object_word.ends_with("ing") && object_word.chars().count() > 4 {
        return false; // smoking, drinking, eating, biting, vaping
    }
    !_WHAT_PEOPLE_QUIT.contains(*object_word)
}

/// The guards on the faulty-goods entry: any one of them hands the line back.
pub fn _not_a_kitchen_or_a_stopped_machine(understood: &Understood) -> bool {
    _not_a_kitchen_ask(understood)
        && _a_machine_that_stopped(understood)
        && _the_device_gave_up(understood)
        && _a_stain_a_shop_answers_for(understood)
        && _not_the_flats_own_appliance(understood)
        && _a_mark_that_came_with_it(understood)
        && _not_a_how_to_about_their_own_thing(understood)
        && _not_a_repair_or_replace_decision(understood)
        && _a_brake_cable_repair_ask(understood)
}

/// Whether a bad-room line is a room that came with a booking.
pub fn _a_hotel_room_or_no_home(understood: &Understood) -> bool {
    let text = words_or_line(understood);
    if ["en suite", "ensuite", "wet room", "shower room"].iter().any(|p| text.contains(p)) {
        return false;
    }
    if any_in(&text, &HOTEL_ROOM_WORDS) {
        return true;
    }
    if any_in(&text, &HOME_ROOM_WORDS) {
        return false;
    }
    if any_in(&text, &_A_DAMP) && any_in(&text, &_A_FIX) {
        return false;
    }
    true
}

/// False when the line is about the fridge being warm rather than the food in it.
pub fn _not_the_fridge_asking_why(understood: &Understood) -> bool {
    let text = words_or_line(understood);
    if any_in(&text, &THE_MONEY) {
        return true;
    }
    if _FOOD_NAMED.is_match(&text) && !_APPLIANCE_TROUBLE.is_match(&text) {
        return false;
    }
    !any_in(&text, &FRIDGE_IS_THE_QUESTION)
}

/// Whether this is the refund answer rather than the what-can-i-cook-now answer.
pub fn _not_a_kitchen_ask(understood: &Understood) -> bool {
    let text = words_or_line(understood);
    if any_in(&text, &KITCHEN_QUIET) {
        return true;
    }
    // cooking::without (re-exported by d2dup_g8 as cooking_without)
    text::strip(&cooking_without(understood).unwrap_or_default()).is_empty()
}

/// The refund ladder, with the six-month reversal only while it is still the rule.
pub fn faulty_goods_reply(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    match _age_in_months(reading) {
        Some(months) if months > 6 => Some(format!("{}{}", *FAULTY_REPLY, *SIX_MONTH_PASSED)),
        _ => Some(format!("{}{}", *FAULTY_REPLY, *SIX_MONTH_REVERSAL)),
    }
}

const REAL_HOLIDAY: &[&str] = &[
    "flight", "flights", "airline", "airport", "hotel", "resort", "package", "abroad", "tour operator", "travel agent",
    "booking reference", "atol",
];

/// Whether the leave here is booked with an employer rather than a travel company.
pub fn _the_employer_is_who_stopped_it(understood: &Understood) -> bool {
    // `str(understood.get("raw"))` is "None" on a reading, and Python keeps it in the line.
    let text = format!("{} {} None {}", understood.clean, understood.text, understood.words.join(" "));
    let text = format!(" {} ", text::squash(&text.to_lowercase().replace('\'', "").replace(',', " ")));
    if REAL_HOLIDAY.iter().any(|w| text.contains(&format!(" {w} "))) {
        return false;
    }
    EMPLOYER_WORDS.iter().any(|w| text.contains(&format!(" {w} ")))
}

/// False when the line is the notice the tenant writes, which is the leaving-letter row.
pub fn _the_notice_is_theirs_to_give(understood: &Understood) -> bool {
    let text = or(&understood.clean, &understood.text);
    !(_THE_NOTICE_IS_GIVEN.is_match(text) && _NAMES_THE_LANDLORD.is_match(text))
}

/// False when it is leave from work: the work rows answer that one.
pub fn _not_a_leave_question(understood: &Understood) -> bool {
    !_the_employer_is_who_stopped_it(understood)
}

/// Whether the airline's baggage desk is the right place to send this bag.
pub fn _a_bag_at_the_airport(understood: &Understood) -> bool {
    let text = words_or_line(understood);
    let text = format!(" {} ", text::squash(&text.replace(',', " ").replace('.', " ")));
    if TRAVEL_SAID.iter().any(|w| text.contains(&format!(" {w} "))) {
        return true;
    }
    if CAME_OFF_THE_BELT.iter().any(|p| text.contains(&format!(" {p} "))) {
        return true;
    }
    if text.contains("baggage") || text.contains("luggage") {
        return true;
    }
    GONE_MISSING.iter().any(|w| text.contains(&format!(" {w} ")))
}

/// Who pays to send it back, unless a flat fee is the question.
pub fn _return_costs_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = format!("{} {} {} {}", understood.clean, understood.text, "", understood.words.join(" ")).to_lowercase();
    if any_in(&text, &_FLAT_FEE_WORDS) {
        return Some("A flat restocking or handling fee on a return is not allowed - they can only deduct for damage you caused - and on a faulty or misdescribed item the retailer pays the postage too. Write asking for the full price back, say it is going back as faulty, and ask for a prepaid label or a collection; if they keep the fee, the card provider can charge back that amount.".into());
    }
    Some("Faulty, misdescribed or simply not what you ordered: the retailer pays to get it back, so ask for a prepaid label or a courier collection rather than posting it yourself, and say in the email that it is going back as faulty. On a change of mind bought online you pay the return delivery unless their terms promised free returns, and a restocking fee is not allowed.".into())
}

/// True unless the only mind in the line is a taste, with nothing bought named beside it.
pub fn _mind_is_about_goods(understood: &Understood) -> bool {
    let text = format!(" {} ", or(&understood.clean, &understood.text).to_lowercase());
    if !_MIND_LIKE.is_match(&text) {
        return true;
    }
    _A_PURCHASE_TRACE.is_match(&text)
}

/// Goods that are not what they were said to be, in the two shapes people send.
pub fn _described_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = line_and_words(understood);
    if any_in(&text, &_WAS_USED_WORDS) {
        return Some("Sold as new but used, opened, a shop's display model or a refurb is a misdescription, so you can reject it for a full refund and you do not have to take a discount or a credit note: write to the shop saying what you were sold and what arrived, with photos of the wear, the serial number and the box. Do it inside 30 days, keep the listing that said new, and if they refuse, a card chargeback or Section 75 over \u{a3}100.".into());
    }
    Some("Goods have to match their description: a sofa sold as leather that is vinyl, a jumper labelled 100% cotton that is not, a colour nothing like the photo. That is a misdescription under the Consumer Rights Act 2015, so you can reject it for a full refund and do not have to accept a credit note. Say in writing what you were told and what arrived, with photos, and give them 14 days; if they refuse, a card chargeback or Section 75 over \u{a3}100.".into())
}

/// False when the line's "stock" is a share in a company rather than goods on a shelf.
pub fn _stock_is_not_a_parcel(understood: &Understood) -> bool {
    let text = format!("{} {} {}", understood.clean, understood.text, "").to_lowercase();
    !_A_STAKE_IN_A_COMPANY.is_match(&text)
}

/// Whether the line names what the goods should have been, not just a pronoun and a product.
pub fn _a_description_is_named(understood: &Understood) -> bool {
    // Python joins a set of the words here, in hash order; the words keep their line order
    // (first time each is said) in this port.
    let mut seen = HashSet::new();
    let words: Vec<String> =
        understood.words.iter().map(|w| w.to_lowercase()).filter(|w| seen.insert(w.clone())).collect();
    if words.iter().any(|w| _DESCRIBED_WORDS.contains(w)) {
        return true;
    }
    let text = format!("{} {} {} {}", understood.clean, understood.text, "", words.join(" ")).to_lowercase();
    any_in(&text, &_DESCRIBED_PHRASES)
}

/// Bringing something back to a shop, with the sale-tag excuse only when they raise it.
pub fn _shop_return_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = line_and_words(understood);
    let mut reply = String::from("In a shop, bringing something back because you changed your mind is the shop's own policy, not a right: 28 days for an exchange is common, and a bank statement usually does instead of a receipt. Faulty is different - 30 days to reject it for a full refund, and no returns sign can take that away.");
    if any_in(&text, &_SALE_TAG_WORDS) {
        reply.push_str(" A sale tag changes none of it, so \"it was reduced\" is not a reason to refuse a faulty one.");
    }
    Some(reply)
}

/// A job done badly, with the sentence that job needs.
pub fn _services_ruined_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = line_and_words(understood);
    let words: HashSet<String> = LETTERS.findall_str(&text).into_iter().collect();
    let reply = "Photograph it straight away and ask for a refund the same day: you paid for the job done with reasonable care, and a botched one breaks that. Put the request in writing with 14 days to answer; if they refuse, Money Claims Online costs about \u{a3}35.";
    for (tokens, phrases, sentence) in _SERVICES_KINDS.iter() {
        if tokens.iter().any(|t| words.contains(t)) || any_in(&text, phrases) {
            return Some(format!("{reply} {sentence}"));
        }
    }
    if _SERVICES_HAIR_WORDS.iter().any(|t| words.contains(t)) {
        return Some(format!("{reply} Get a second salon to write down what is wrong and what putting it right would cost, because a cut, a colour or a tattoo cannot be undone."));
    }
    Some(format!("{reply} A second opinion in writing - another salon, another repairer or the vet - is the evidence that wins it."))
}

/// A payment that has stopped is not a lost scheme.
pub fn _pension_reply(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = format!("{} {} {} {}", understood.clean, understood.text, "", understood.words.join(" ")).to_lowercase();
    if any_in(&text, &_PENSION_STOPPED_WORDS) {
        return Some("Ring the scheme, or the payroll if you are still working, and ask why the payment stopped or came up short: they have to tell you and put it right, and most are sorted on that call. Ask for it in writing, give them eight weeks, then the Pensions Ombudsman is free and can order the money paid. If it is the State Pension, ring the Pension Service instead, and check your forecast on gov.uk for the date and amount it should be.".into());
    }
    let asked = format!("{} {} {} {}", understood.clean, understood.text, understood.norm_text, understood.plain).to_lowercase();
    if _A_COUNT_FOLLOW_UP.is_match(&asked) {
        let last = memory.turns.last().map(|t| t.bot.to_lowercase()).unwrap_or_default();
        if last.contains("old pots") || last.contains("old pensions") {
            return Some("Three or four is normal - most people build one up at every job they leave. What is worth a look is the fee each pot charges and whether any of them has a guaranteed benefit, so ask each provider for a statement before you decide to move anything.".into());
        }
    }
    let words: HashSet<String> = LETTERS.findall_str(&text).into_iter().collect();
    if any_in(&text, &_PENSION_SEVERAL_WORDS) || words.iter().any(|w| _PENSION_COMBINE_WORDS.contains(w)) {
        return Some("Three or four old pots is normal, and none of it is lost money: give each employer's name to the free Pension Tracing Service, then ask every provider for a statement - the value, the fees and any guaranteed benefits. Check those guarantees before you move anything, because a final-salary promise is usually lost in a transfer.".into());
    }
    Some("Start with the free Pension Tracing Service if you have lost track of an old scheme: give it the employer's name and it returns the provider's details, then ask that provider in writing for a statement. For the State Pension, your forecast on gov.uk shows the date and the amount, and if a payment is missing complain to the scheme first and then to the free Pensions Ombudsman.".into())
}

/// A seller who will not answer, in a letter or on the phone.
pub fn _seller_ignoring_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = line_and_words(understood);
    if _SOMEONE_PUT_THE_PHONE_DOWN.is_match(&text) && !_SELLING_WORDS.is_match(&text) {
        return None;
    }
    let mut reply = String::from("Write one email that sets out the order, the fault and what you want, and give them 14 days to answer: that email is what a card company or a court reads. If the silence continues, ask your card provider for a chargeback, or Section 75 over \u{a3}100 on a credit card.");
    let words: HashSet<String> = LETTERS.findall_str(&text).into_iter().collect();
    if _PHONE_WORDS.iter().any(|w| words.contains(w)) || text.contains("customer service") {
        reply.push_str(" Keep a note of every call - the date, the number and who you spoke to - because that log is what the ombudsman reads next.");
    }
    Some(reply)
}

/// A fee sprung at the checkout, or a fee for a booking that never happened.
pub fn _fee_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = line_and_words(understood);
    if any_in(&text, &_DELIVERY_FEE_WORDS) {
        return Some("Delivery it said was free is not a charge you agreed to, so ask for it back: quote the page or the email that said free delivery, give them 14 days, and keep the order confirmation. If they will not refund it, the card provider can charge back that amount, and the Citizens Advice consumer helpline passes it to Trading Standards.".into());
    }
    if any_in(&text, &_NEVER_HAPPENED_WORDS) {
        return Some("A fee for something that never happened is not owed: write to them and say when you cancelled or that you never booked it, ask for the charge back and for any no-show marker on your account to be removed. Keep the email and the date. If it came off a card, the card provider can charge back a fee you did not agree to, and for a gym or a salon the next step is their complaints procedure and then the trade body that covers them.".into());
    }
    Some("The price you were shown has to include anything you have to pay, so a booking, admin or card fee sprung at the last screen is not something you owed: ask the company for it back in writing, say what total you were quoted, and point out the fee was not part of it. If they will not refund it, dispute that amount on the card, and report the drip pricing to Trading Standards through the Citizens Advice consumer helpline, which passes it on.".into())
}

/// The plain ask about a named fee: what it is, whether it comes back, whether it is allowed.
pub fn _fee_kind_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = line_and_words(understood);
    if any_in(&text, &_LATE_FEE_WORDS) {
        return Some("A late fee is what you agreed to pay for paying after the due date, and it has to be in the terms you signed up to and no more than a fair estimate of the cost to them. If it was never in the terms, or it is far bigger than the debt, ask for it back in writing; for a bill or a loan the next step is the company's complaints then the energy or financial ombudsman, which is free.".into());
    }
    if any_in(&text, &_CANCEL_FEE_WORDS) {
        return Some("A cancellation or no-show fee is what the terms said you would pay for cancelling late or not turning up, and it has to be a fair estimate of what they lost - so the same fee whatever notice you gave, or a fee when you gave plenty, is the kind to challenge: ask which term it comes from, then go to their complaints, Trading Standards through Citizens Advice, or the card provider.".into());
    }
    for phrase in _OTHER_FEE_WORDS.iter() {
        if text.contains(phrase.as_str()) {
            let article = if phrase.chars().next().is_some_and(|c| "aeiou".contains(c)) { "an" } else { "a" };
            return Some(format!(
                "{} {phrase} is a charge added on top of the thing you are buying rather than payment for the thing itself, and it has to be inside the total price you are shown before you pay - not added at the last screen. Ask what it covers before you agree to it, and if you were quoted a price without it, ask them to take it off: a fee that was never disclosed is not one you owed, and the card provider can charge it back if they will not refund it.",
                text::capitalize(article)
            ));
        }
    }
    if text.contains("cleaning fee") {
        return Some("A cleaning fee is a fixed charge a host adds for the changeover between guests, and it belongs in the price you are shown before you book rather than arriving at checkout. It is not a fine: ordinary mess after a normal stay is what it covers, and extra can only be asked for damage or for something the listing warned about.".into());
    }
    Some("A booking fee is a charge for arranging the booking rather than for the thing itself - a venue's or an agent's cut, or a platform's fee for running the site - and it has to be inside the total price you are shown before you pay, not added at the last screen. It is usually kept when you cancel even where the rest is refundable, so read what the terms say it covers before you pay; a fee that only appeared at checkout, or one charged for a booking that never happened, you can ask back and then dispute on the card.".into())
}

/// A renewal nobody warned about, or a payment taken after the person cancelled.
pub fn _renewed_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let lowered: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let mut words: HashSet<String> = lowered.iter().cloned().collect();
    words.extend(text::split(&understood.clean.to_lowercase()));
    let text = format!("{} {}", understood.clean, lowered.join(" "));
    if any_in(&text.to_lowercase(), &_TRIAL_WORDS) {
        return Some("A free trial that charges you at the end has to tell you so before you sign up, with the price and the date it ends: if it did not, say the charge was not agreed, cancel in writing the same day and ask for the money back. A subscription bought at a distance can be cancelled within 14 days for a full refund, and if they will not pay, the card provider can charge it back.".into());
    }
    if _CANCEL_WORDS.iter().any(|w| words.contains(w)) {
        return Some("A cancellation stands from the day you gave it, so a payment taken after that date has to come back: send the cancellation again in writing, ask for the confirmation and the date they logged it, and ask for the money taken since. Give them 14 days, then your card provider can charge it back with the email as proof. If they keep charging, ask them to remove your card details.".into());
    }
    Some("A company that renews you has to warn you before it does, and a policy sold at a distance can be cancelled within 14 days of the renewal date, so ask for the renewal notice and the money back in writing, quoting the date it renewed. If they never sent a reminder, say so, cancel in writing the same day and keep the email: a charge you did not agree to is also a card chargeback, and over \u{a3}100 on a credit card it is Section 75.".into())
}

/// False when the line is deciding what to spend on a machine they already own.
pub fn _not_a_buy_or_repair_choice(understood: &Understood) -> bool {
    let text = words_or_line(understood);
    if !_A_MACHINE.is_match(&text) || !_BUY_OR_MEND.is_match(&text) {
        return true;
    }
    _A_CLAIM_AGAINST_A_SELLER.is_match(&text)
}

/// False for a machine the line says has stopped, with nothing about buying it in the line.
pub fn _a_machine_that_stopped(reading: &Understood) -> bool {
    let words: Vec<String> = reading.words.iter().map(|w| w.to_lowercase()).collect();
    let joined = words.join(" ");
    let line = if joined.is_empty() { reading.clean.to_lowercase() } else { joined };
    if !any_in(&line, &MACHINES) {
        return true;
    }
    words.iter().any(|w| _PURCHASE_IN_THE_LINE.contains(w))
}

/// False when the thing that broke came with a home the person rents.
pub fn _not_the_flats_own_appliance(understood: &Understood) -> bool {
    let text = words_or_line(understood);
    if !_RENTED_HOME.is_match(&text) {
        return true;
    }
    if _A_BUYING_LINE.is_match(&text) {
        return true;
    }
    !_FITTED_TO_THE_FLAT.is_match(&text)
}

/// True when the line is about the treatment, false when it is about the tooth.
pub fn _dentist_grievance(understood: &Understood) -> bool {
    let said = understood.text.to_lowercase();
    any_in(&said, &_GRIEVANCE)
}

/// A holding deposit: whether it can be kept, or what it is.
pub fn _holding_deposit(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!(
        " {} ",
        [&understood.clean, &understood.text, &understood.norm_text].iter().map(|s| s.to_lowercase()).collect::<Vec<_>>().join(" ")
    );
    let kept = [
        " keep ", " keeps ", " kept ", " keeping ", " withholding ", " refuse ", " not back ", " not return ", " won't return ",
    ];
    if kept.iter().any(|w| said.contains(w)) {
        return Some(_HOLDING_KEPT.clone());
    }
    Some(_HOLDING_WHAT.clone())
}

/// Which condition of a thing the line names: tags, opened, slept, or worn.
pub fn _which_condition(said: &str) -> &'static str {
    let said = format!(" {} ", text::split_ref(said).join(" "));
    let has = |items: &[&str]| items.iter().any(|item| said.contains(&format!(" {item} ")));
    if has(&["tag", "tags", "label", "labels", "tagged"])
        && has(&["without", "no", "missing", "lost", "off", "ripped", "removed", "cut"])
    {
        return "tags";
    }
    if has(&["tag", "tags"]) && !has(&["worn", "wore", "slept"]) {
        return "tags";
    }
    if has(&[
        "opened", "opening", "unopened", "box", "packaging", "packet", "seal", "sealed", "unwrapped", "wrapper",
        "shrink wrap", "shrinkwrap",
    ]) {
        return "opened";
    }
    if has(&["slept", "slept", "mattress", "bed", "duvet"]) {
        return "slept";
    }
    "worn"
}

/// The returns card for a thing that has been worn, opened, slept on or has no tags.
pub fn _condition_reply(_memory: &mut Memory, understanding: &Understood) -> Option<String> {
    let said = or(&understanding.clean, &understanding.text).to_lowercase();
    // a KeyError in Python, caught by the registry like any raise
    Some(_CONDITION_LEAD.get(_which_condition(&said)).expect("condition lead").clone())
}

/// The sentence the change-of-mind row owes, read off which way round the line asks.
pub fn _change_of_mind_verdict(understood: &Understood) -> String {
    let text = understood.text.to_lowercase();
    if _TOO_LATE.is_match(&text) {
        return "No, not if it is within 14 days".into();
    }
    if _STILL_TIME.is_match(&text) {
        return "Yes, if it is within 14 days".into();
    }
    "Yes".into()
}

/// Whether a bicycle brake cable line is a repair question rather than a shop claim.
pub fn _a_brake_cable_repair_ask(understood: &Understood) -> bool {
    // TYPE: Python reads `(leans_on or {}).get("text")`; `leans_on` is a dict on a follow-up
    // turn, which the typed reading does not carry yet, so the line before is "" here.
    let above = "";
    let said = or(&understood.clean, &understood.text);
    let text = format!("{above} {said}").to_lowercase();
    if !_A_BICYCLE_CABLE.is_match(&text) {
        return true;
    }
    if !["bike", "bicycle", "cycle", "pushbike"].iter().any(|w| text.contains(w)) {
        return true;
    }
    _A_SHOP_CLAIM.is_match(&text) || !_A_REPAIR_ASK.is_match(&text)
}

/// True when the line says how long the parcel has been sitting where it is.
pub fn _a_stretch_of_waiting(said: &str) -> bool {
    _A_WAIT.is_match(said)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.consumer:_a_bag_at_the_airport:686", |_: &FnRef, u: &Understood| _a_bag_at_the_airport(u)),
    ("bot.knowledge.consumer:_a_description_is_named:836", |_: &FnRef, u: &Understood| _a_description_is_named(u)),
    ("bot.knowledge.consumer:_a_hotel_room_or_no_home:463", |_: &FnRef, u: &Understood| _a_hotel_room_or_no_home(u)),
    ("bot.knowledge.consumer:_a_place_that_takes_bookings:322", |_: &FnRef, u: &Understood| {
        _a_place_that_takes_bookings(u)
    }),
    ("bot.knowledge.consumer:_dentist_grievance:1356", |_: &FnRef, u: &Understood| _dentist_grievance(u)),
    ("bot.knowledge.consumer:_mind_is_about_goods:748", |_: &FnRef, u: &Understood| _mind_is_about_goods(u)),
    ("bot.knowledge.consumer:_not_a_buy_or_repair_choice:1256", |_: &FnRef, u: &Understood| {
        _not_a_buy_or_repair_choice(u)
    }),
    ("bot.knowledge.consumer:_not_a_figure_judged:147", |_: &FnRef, u: &Understood| _not_a_figure_judged(u)),
    ("bot.knowledge.consumer:_not_a_kitchen_or_a_stopped_machine:414", |_: &FnRef, u: &Understood| {
        _not_a_kitchen_or_a_stopped_machine(u)
    }),
    ("bot.knowledge.consumer:_not_a_leave_question:681", |_: &FnRef, u: &Understood| _not_a_leave_question(u)),
    ("bot.knowledge.consumer:_not_an_old_machines_repair_decision:303", |_: &FnRef, u: &Understood| {
        _not_an_old_machines_repair_decision(u)
    }),
    ("bot.knowledge.consumer:_not_saving_for_a_deposit:268", |_: &FnRef, u: &Understood| _not_saving_for_a_deposit(u)),
    ("bot.knowledge.consumer:_not_the_fridge_asking_why:540", |_: &FnRef, u: &Understood| _not_the_fridge_asking_why(u)),
    ("bot.knowledge.consumer:_stock_is_not_a_parcel:827", |_: &FnRef, u: &Understood| _stock_is_not_a_parcel(u)),
    ("bot.knowledge.consumer:_the_notice_is_theirs_to_give:673", |_: &FnRef, u: &Understood| {
        _the_notice_is_theirs_to_give(u)
    }),
    ("bot.knowledge.consumer:_the_shop_or_the_thing_is_named:234", |_: &FnRef, u: &Understood| {
        _the_shop_or_the_thing_is_named(u)
    }),
    ("bot.knowledge.consumer:_they_are_the_ones_cancelling:346", |_: &FnRef, u: &Understood| {
        _they_are_the_ones_cancelling(u)
    }),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.consumer:_condition_reply:1431", |_: &FnRef, m: &mut Memory, u: &Understood| _condition_reply(m, u)),
    ("bot.knowledge.consumer:_described_reply:756", |_: &FnRef, m: &mut Memory, u: &Understood| _described_reply(m, u)),
    ("bot.knowledge.consumer:_fee_kind_reply:1089", |_: &FnRef, m: &mut Memory, u: &Understood| _fee_kind_reply(m, u)),
    ("bot.knowledge.consumer:_fee_reply:1042", |_: &FnRef, m: &mut Memory, u: &Understood| _fee_reply(m, u)),
    ("bot.knowledge.consumer:_holding_deposit:1378", |_: &FnRef, m: &mut Memory, u: &Understood| _holding_deposit(m, u)),
    ("bot.knowledge.consumer:_pension_reply:956", |_: &FnRef, m: &mut Memory, u: &Understood| _pension_reply(m, u)),
    ("bot.knowledge.consumer:_renewed_reply:1152", |_: &FnRef, m: &mut Memory, u: &Understood| _renewed_reply(m, u)),
    ("bot.knowledge.consumer:_return_costs_reply:707", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _return_costs_reply(m, u)
    }),
    ("bot.knowledge.consumer:_seller_ignoring_reply:1013", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _seller_ignoring_reply(m, u)
    }),
    ("bot.knowledge.consumer:_services_ruined_reply:897", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _services_ruined_reply(m, u)
    }),
    ("bot.knowledge.consumer:_shop_return_reply:852", |_: &FnRef, m: &mut Memory, u: &Understood| _shop_return_reply(m, u)),
    ("bot.knowledge.consumer:faulty_goods_reply:578", |_: &FnRef, m: &mut Memory, u: &Understood| faulty_goods_reply(m, u)),
];
