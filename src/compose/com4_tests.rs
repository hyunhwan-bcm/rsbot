//! Values from PyBot (`harness/pycall.py`, clock pinned, random.seed(0)) for branches of the COM4
//! files that the recorded corpora do not reach, or reach only through a stub elsewhere.
#![cfg(test)]

use crate::compose as c;
use crate::state::Memory;

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn compose_helpers() {
    assert_eq!(c::greet_with("Sam", "Bristol \u{2014} nice. How long have you been there?"), "Hi Sam. Bristol \u{2014} nice. How long have you been there?");
    assert_eq!(c::greet_with("Jess", "most people who cannot sleep are wound up."), "Hi Jess \u{2014} most people who cannot sleep are wound up.");
    assert_eq!(c::greet_with("Jess", "I'm not sure about that one, sorry"), "Hi Jess \u{2014} I'm not sure about that one, sorry");
    assert_eq!(c::greet_with("Jess", "Hello there"), "Hello there");
    assert!(c::_opens_with_a_name("Leeds? Good to know"));
    assert!(!c::_opens_with_a_name("Sorry - that is rough"));
    assert!(c::negated("im not feeling happy today", "happy", 3));
    assert!(!c::negated("i cant believe im so happy", "happy", 3));
    assert!(c::nearly("i almost won the race", "won", 2));
    assert!(c::doubted("i dont think i passed", 4));
    assert!(!c::doubted("i cant believe i passed", 4));
    assert!(!c::doubted("i dont believe it, i passed", 4));
    assert_eq!(c::said_again("Stay with him  and keep HER warm"), "stay with @ and keep @ warm");
    assert!(c::says_it_has_lingered("a cough for three weeks", false));
    assert!(!c::says_it_has_lingered("a cough for two days", false));
}

#[test]
fn occasionplan_limit_taste() {
    use c::{limit, occasionplan as o, taste as t};
    assert_eq!(o::_ordinal(21), "21st");
    assert_eq!(o::_ordinal(112), "112th");
    assert_eq!(o::_ordinal(33), "33rd");
    assert_eq!(o::_whose("sister"), "your sister's");
    assert_eq!(o::_whose("Jess"), "Jess's");
    assert_eq!(o::_whose("my sons"), "your sons'");
    assert_eq!(o::_the_day(Some("my daughters birthday is next month she turns 7")), "next month");
    assert_eq!(o::_the_day(Some("3rd may")), "on 3rd May");
    assert_eq!(limit::forms_of("walking up hills"), s(&["hill", "hills", "up", "upes", "ups", "walk", "walke", "walking", "walkinges", "walkings"]));
    assert_eq!(limit::forms_of("berries"), s(&["berries", "berry"]));
    assert_eq!(limit::_joined(&["and the fountains are free,", "the walk takes an hour"]), "The fountains are free, the walk takes an hour.");
    for (verb, doing) in [("swim", "swimming"), ("cook", "cooking"), ("tie", "tying"), ("see", "seeing")] {
        assert_eq!(t::_a_doing(verb), doing);
    }
    assert_eq!(t::_split_manner("my tea really weak"), ("my tea really".to_string(), "weak".to_string()));
    assert_eq!(t::_split_manner("coffee strong"), ("coffee".to_string(), "strong".to_string()));
    assert_eq!(t::_the_doing("to go swimming"), "Going swimming");
    assert_eq!(t::_as_a_name("paris in spring"), "paris in spring");
    assert_eq!(t::_in_a_sentence("The rain"), "the rain");
}

#[test]
fn tone_and_pace() {
    use c::{pace as p, tone_again as t};
    assert_eq!(
        t::_firm("Hi, I was just wondering if you could fix the boiler whenever you get a chance. Thanks"),
        "Hi, I need to know you could fix the boiler whenever you get a chance. Please confirm when it will be done."
    );
    assert_eq!(t::_polite("Hi, the boiler is broken. Can you fix it?"), "Hi, the boiler is broken. Can you fix it, please? Thank you.");
    assert_eq!(t::_short("The boiler is broken. It has been a week. Can you fix it by Friday? Thanks."), "The boiler is broken. Can you fix it by Friday?");
    assert_eq!(t::_formal("Hi, I can't get the boiler going. Thanks"), "Dear Sir or Madam, I cannot get the boiler going. Yours faithfully.");
    assert_eq!(t::_casual("Hello, the boiler is broken. Thanks"), "Hi, the boiler is broken. Cheers.");
    let mut tones = t::_tones("make it less formal and shorter");
    tones.sort();
    assert_eq!(tones, s(&["casual", "formal", "short"]));
    assert_eq!(p::_trim(&"a ".repeat(60), 96), "a ".repeat(48).trim_end());
    assert_eq!(p::_as_clause("Once a fortnight is plenty."), "once a fortnight is plenty");
    assert_eq!(p::_the_doing("of doing it the same way every single time, which is what helps"), "doing it the same way every single time");
    let mut stems: Vec<String> = p::_stems("feeding feeds feed walked").into_iter().collect();
    stems.sort();
    assert_eq!(stems, s(&["feed", "walk"]));
}

#[test]
fn harm_reaction_badnews() {
    use c::{badnews as b, harm, reaction};
    assert!(!harm::reads_as_our_reply("nothing"));
    assert_eq!(harm::_said_back("i want to punch my brother"), "your brother");
    assert_eq!(harm::_said_back("i want to kill her"), "");
    assert_eq!(reaction::_bare("wow"), "awe");
    assert_eq!(reaction::_bare("oh wow seriously"), "");
    assert!(b::_one_edit("sezure", "seizure"));
    assert!(!b::_one_edit("cooking", "choking"));
    assert!(b::_one_edit("hospitel", "hospital"));
    assert!(b::_nearly("hospitel"));
    assert!(b::reads_as_someones_death("my dad died last week"));
    assert!(!b::reads_as_someones_death("my phone died"));
    assert!(b::reads_as_a_bare_ailment("ugh toothache"));
}

#[test]
fn who_and_friends() {
    use c::{a_thing_being_weighed as w, missing_half, plan_ask, plans, who};
    assert!(who::_says_it_again("ring them and they will talk", "you can ring them and they will"));
    assert_eq!(who::_tidy("\"see a gp if it lasts,"), "See a gp if it lasts.");
    assert_eq!(who::_lead_of("vet"), "A vet");
    assert_eq!(missing_half::_the_known_list(&[("wife".into(), 32), ("brother".into(), 40)]), "your wife at 32 and your brother at 40");
    assert_eq!(plan_ask::_quoted_back(&["a", "b", "c"]), "\"a\", \"b\" and \"c\"");
    assert_eq!(
        plans::_the_line("time", "health", "", "scan").unwrap(),
        "What time do you need to be at the surgery? Work back from the journey, and add half an hour for the queue."
    );
    assert_eq!(plans::_the_line("who", "trip", "rome", "").unwrap(), "The booking lists everyone who is travelling to Rome on it. Who are you going with?");
    assert_eq!(w::_with_article("electric blanket"), "an electric blanket");
    assert_eq!(w::_with_article("used car"), "a used car");
    assert_eq!(w::_as_said("the bikes", "bikes"), "bikes");
    assert_eq!(
        w::_head_and_name("a second hand bike for commuting"),
        ("bike".to_string(), "a second hand bike for commuting".to_string(), "second hand bike".to_string())
    );
    assert_eq!(w::pool("im thinking about getting a new laptop"), None);
}

#[test]
fn stretches_threads_saidby_urgent() {
    use c::{saidby, stretches as st, thread_detail as td, urgent, what_decides_it as wd};
    assert_eq!(st::_the_stretch_said_back("since monday morning"), "Since Monday morning");
    assert_eq!(st::reads_as_a_long_stretch("ive been on hold for two hours"), Some(("waiting".into(), "For two hours".into())));
    assert_eq!(st::reads_as_a_long_stretch("ive spent all day on this form"), Some(("effort".into(), "All day".into())));
    assert_eq!(td::_second_person("I can't do my mornings"), "You can't do your mornings");
    assert_eq!(td::the_clause_said_back("and im not allowed to drive"), "I'm not allowed to drive");
    assert!(saidby::said_by_the_person("so i texted him \"i want to see you\""));
    assert!(saidby::reads_as_somebody_elses_wish("my wife said \"i want a divorce\""));
    assert!(urgent::_not_a_body("to my texts", "my son is unresponsive"));
    assert!(urgent::_not_a_body("nicely", "my son is fitting"));
    assert!(urgent::reads_as_a_story("he had a fit last year"));
    assert_eq!(
        wd::_the_test("Rest it for a few days. See a GP if it is no better in a fortnight or if it swells badly."),
        "see a GP if it is no better in a fortnight or if it swells badly"
    );
}

#[test]
fn leaving_draft_gap_and_the_rest() {
    use c::{draft, gap, leaving, no_eyes, the_course_of_it as tc};
    let lines = leaving::lines();
    assert!(lines.contains_str("All the best \u{2014} talk soon."));
    assert!(lines.contains_str("Ah, that matters more than me. Tell me about it."));
    assert!(leaving::reads_as_leaving("right im off to bed, night", None));
    assert!(leaving::reads_as_leaving("got to go, the kids are up", None));
    assert_eq!(tc::_the_first_bit("Water little and often. It helps.", 150, false), "Water little and often");
    assert!(no_eyes::handed_you_something("heres a screenshot of it").unwrap()[0].starts_with("I can't see your screen or your photos, sorry"));
    assert!(no_eyes::cannot_see_your_thing("can you listen to this audio clip").unwrap()[0].starts_with("I can't hear a voice note or a recording"));
    assert_eq!(draft::_tidy("ill tell him im off sick monday"), "I'll tell him I'm off sick Monday");
    assert_eq!(draft::_content("saying im off sick tomorrow"), Some(("I'm off sick tomorrow".into(), "said".into())));
    assert!(draft::reads_as_a_writing_ask("can you help me write a text to my landlord"));
    assert!(gap::sum_lines("how much do i spend a year on a coffee every day").unwrap()[0]
        .starts_with("Every day is 365 a year. Put your own price on one - £5 each is £1,825 -"));
    assert_eq!(gap::times_a_year("how much is two takeaways a week over a year"), Some((104, "two takeaways a week".into())));
    let _ = Memory::new();
}
