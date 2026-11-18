//! DUP ports (group 7 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

static GOING_OFF_ASK: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking.GOING_OFF_ASK"));
static KEEPING_VERB: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking.KEEPING_VERB"));
static IN_THE_OVEN: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking.IN_THE_OVEN"));
static JOINT_BEEF: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.cooking.JOINT_BEEF"));
static JOINT_PORK: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.cooking.JOINT_PORK"));

/// DUP(bot.knowledge.cooking::_same): the word is the wanted one or starts with it.
pub fn _same(word: &str, wanted: &str) -> bool {
    !word.is_empty() && !wanted.is_empty() && (word == wanted || word.starts_with(wanted))
}

/// DUP(bot.knowledge.cooking::_roast_joint): the oven time for the joint the line names (both
/// when it names none); None on a keeping or going-off line.
pub fn _roast_joint(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = text::squash(&understood.text).to_lowercase();
    if GOING_OFF_ASK.iter().any(|p| line.contains(p.as_str())) {
        return None;
    }
    if KEEPING_VERB.iter().any(|p| line.contains(p.as_str())) && !IN_THE_OVEN.iter().any(|w| line.contains(w.as_str())) {
        return None;
    }
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let names = |wanted: &[&str]| words.iter().any(|w| wanted.iter().any(|x| _same(w, x)));
    if names(&["pork", "gammon", "ham"]) {
        return Some(JOINT_PORK.clone());
    }
    if names(&["beef", "lamb", "mutton"]) {
        return Some(JOINT_BEEF.clone());
    }
    Some(format!("{} {}", *JOINT_BEEF, *JOINT_PORK))
}

static STARTING_OUT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.STARTING_OUT"));
static STARTING_INTO: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.repeat.STARTING_INTO"));
static STARTING_AT: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.repeat.STARTING_AT"));
static STARTING_BACK: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.repeat.STARTING_BACK"));

/// DUP(bot.memory.repeat::_starting_out_names_a_thing): "starting out <thing>" names a thing
/// (not "starting out again", "starting out slowly").
pub fn _starting_out_names_a_thing(line: &str) -> bool {
    let Some(found) = STARTING_OUT.search(line) else { return false };
    let (first, second) = (found.str(1), found.str(2));
    if STARTING_INTO.contains(first) || STARTING_AT.contains(first) {
        return !second.is_empty() && !STARTING_BACK.contains(second);
    }
    !STARTING_BACK.contains(first)
}

static KEEPING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KEEPING_WORDS"));
static KEPT_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KEPT_FOODS"));
static CONTAINERS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.CONTAINERS"));

/// DUP(bot.knowledge.lookup::a_food_is_being_kept): the line is about storing a food (a keeping
/// word and a kept food both named).
pub fn a_food_is_being_kept(understood: &Understood) -> bool {
    let text = format!(" {} ", crate::knowledge::d1_shared::or(&understood.norm_text, &understood.clean).to_lowercase());
    if !KEEPING_WORDS.iter().any(|w| text.contains(w.as_str())) {
        return false;
    }
    let said: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    for food in KEPT_FOODS.iter() {
        if food.contains(' ') {
            if text.contains(food.as_str()) {
                return true;
            }
        } else if said.contains(food) || text.contains(&format!(" {food} ")) {
            return true;
        }
    }
    false
}

/// DUP(bot.knowledge.lookup::only_a_container): every one of `names` the line holds is a
/// container (a bag, a tub), and it holds at least one.
pub fn only_a_container<S: AsRef<str>>(understood: &Understood, names: &[S]) -> bool {
    let text = format!(" {} ", understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" "));
    let found: Vec<&str> = names.iter().map(|n| n.as_ref()).filter(|one| text.contains(&format!(" {one} "))).collect();
    !found.is_empty() && found.iter().all(|one| CONTAINERS.contains(*one))
}

/// Python's `fmt % args` for `%s` (and `%%`) placeholders, in order.
pub fn pct(fmt: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(fmt.len() + 32);
    let mut args = args.iter();
    let mut chars = fmt.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.peek() {
                Some('s') => {
                    chars.next();
                    out.push_str(args.next().copied().unwrap_or(""));
                    continue;
                }
                Some('%') => {
                    chars.next();
                    out.push('%');
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }
    out
}

/// Python's `str.isdigit()` for one character: decimal digits and the superscript digits
/// (not fractions or roman numerals, which Python counts as numeric but not digits).
pub fn is_digit_char(c: char) -> bool {
    c.is_ascii_digit()
        || (c.is_numeric() && !('\u{00BC}'..='\u{00BE}').contains(&c) && !('\u{2150}'..='\u{218F}').contains(&c))
}
