//! Private ports of helpers the g9 modules (babyfood, opinions, family_more, keeping,
//! cooking_gaps, fruit, costs, devices) need from code other workstreams own. Each is marked
//! `// DUP(<python module>::<name>)` (another workstream) or `// DUP-D2(...)` (another D2 group),
//! so the integrator can swap in the owner's port.

use crate::data;
use crate::py::re::{escape, Re};
use crate::understood::Understood;
use std::sync::LazyLock;

/// Python `str(list_of_str)`, or "" for an empty list (`str(x or "")`).
fn list_or_empty(items: &[String]) -> String {
    if items.is_empty() { String::new() } else { crate::knowledge::d2_shared::py_repr_list(items) }
}

static APPLIANCE_NAMES: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    data::strs("bot.knowledge.lookup.APPLIANCE_NAMES")
        .into_iter()
        .map(|a| {
            let rx = Re::new(&format!("(?<![a-z0-9]){}(?![a-z0-9])", escape(&a)), 0);
            (a, rx)
        })
        .collect()
});

// DUP(bot.knowledge.lookup::the_appliance_swallows_the_name)
/// True when a named appliance contains one of `names` in its own name ("heat pump" inside
/// "heat pump dryer"), so the appliance's own row answers the line.
pub fn the_appliance_swallows_the_name<S: AsRef<str>>(understood: &Understood, names: &[S]) -> bool {
    let parts = [understood.clean.clone(), understood.text.clone(), list_or_empty(&understood.norm)];
    let said = format!(" {} ", parts.join(" ").to_lowercase());
    let names: Vec<String> = names
        .iter()
        .map(|n| n.as_ref())
        .filter(|n| !crate::py::text::strip(n).is_empty())
        .map(|n| n.to_lowercase())
        .collect();
    if names.is_empty() {
        return false;
    }
    for (appliance, rx) in APPLIANCE_NAMES.iter() {
        if !rx.is_match(&said) {
            continue;
        }
        if names.iter().any(|n| appliance.contains(n.as_str()) && n.chars().count() < appliance.chars().count()) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// bot/memory

// DUP(bot.memory::recent)
/// The last n turns, oldest first.
pub fn recent(memory: &crate::state::Memory, n: usize) -> &[crate::state::Turn] {
    let turns = &memory.turns;
    &turns[turns.len().saturating_sub(n)..]
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/family.py

static A_BOY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.family._A_BOY"));
static A_GIRL: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.family._A_GIRL"));

// DUP(bot.knowledge.family::_the_child_pronouns)
/// he/him or she/her for the child the card is about, they/them when nothing says which: read
/// from the line, then the line it leans on, then the chat.
pub fn _the_child_pronouns(
    memory: &crate::state::Memory,
    understood: &Understood,
) -> (&'static str, &'static str, &'static str, &'static str) {
    let mut said: Vec<String> = Vec::new();
    said.push(if understood.text.is_empty() { understood.clean.clone() } else { understood.text.clone() });
    // TYPE(understood.leans_on): Python reads leans_on["text"]; the typed reading keeps no dict.
    said.push(String::new());
    for turn in &memory.turns {
        said.push(turn.user.clone());
    }
    for text in said {
        let low = format!(" {} ", text.to_lowercase());
        let boy = A_BOY.is_match(&low);
        let girl = A_GIRL.is_match(&low);
        if boy && !girl {
            return ("he", "him", "his", "does");
        }
        if girl && !boy {
            return ("she", "her", "hers", "does");
        }
    }
    ("they", "them", "their", "do")
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/units.py

static TABLE_GROUPS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.units._TABLE_GROUPS"));
static GROUP_VALUE: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| int_dict("bot.knowledge.units._GROUP_VALUE"));

fn int_dict(name: &str) -> Vec<(String, i64)> {
    data::get(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
}

fn lookup_int(table: &[(String, i64)], key: &str) -> Option<i64> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| *v)
}

// DUP(bot.knowledge.units::people_in_groups)
/// How many people the groups named in the line add up to, or None when it names none.
pub fn people_in_groups(text: &str) -> Option<i64> {
    let mut total = 0;
    for raw in TABLE_GROUPS.findall_str(&text.to_lowercase()) {
        let count = if crate::py::text::is_digit(&raw) { raw.parse::<i64>().ok() } else { lookup_int(&GROUP_VALUE, &raw) };
        if let Some(count) = count.filter(|c| *c != 0) {
            total += count;
        }
    }
    if total == 0 { None } else { Some(total) }
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/cooking.py (owned by the cooking groups)

static COOKING_NOT_A_TABLE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking._NOT_A_TABLE"));
static CROWD_NUMBERS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| int_dict("bot.knowledge.cooking._CROWD_NUMBERS"));
static CROWD_FOR: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.cooking._CROWD_FOR"));
static CROWD_WITHIN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.cooking._CROWD_WITHIN"));
static CROWD_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking._CROWD_PHRASES"));
static CROWD_WORDS: LazyLock<Vec<(i64, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking._CROWD_WORDS")
        .pairs()
        .iter()
        .map(|(k, v)| (k.as_int().unwrap_or(0), v.str().to_string()))
        .collect()
});
static OCCASIONS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking._OCCASIONS"));
static FOR_A_NUMBER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bfor\s+(\d{1,3})\b", 0));
static A_NUMBER_OF_PEOPLE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d{1,3})\s+(?:people|of us|adults|guests|coming)\b", 0));
static AN_ORDINAL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b\d{1,3}(?:st|nd|rd|th)\b", 0));

// DUP-D2(bot.knowledge.cooking::_people_after)
/// True when what follows the count at `end` (a byte offset) is not a unit, so the number is people.
pub fn _people_after(text: &str, end: usize) -> bool {
    let rest = crate::py::text::split_ref(&text[end..]);
    let after = rest.first().map(|w| crate::py::text::strip_chars(w, ".,!?;:").to_lowercase()).unwrap_or_default();
    !COOKING_NOT_A_TABLE.contains(&after)
}

// DUP-D2(bot.knowledge.cooking::_crowd)
/// How many people the line is cooking for, or 0 when it is an ordinary dinner.
pub fn _crowd(text: &str) -> i64 {
    let text = text.to_lowercase().replace('-', " ");
    let mut numbers: Vec<i64> = Vec::new();
    for m in FOR_A_NUMBER.finditer(&text) {
        if _people_after(&text, m.end()) {
            numbers.push(m.str(1).parse().unwrap_or(0));
        }
    }
    for m in CROWD_FOR.finditer(&text) {
        if _people_after(&text, m.end()) {
            // Python: _CROWD_NUMBERS[...] (every word the pattern takes is a key)
            numbers.push(lookup_int(&CROWD_NUMBERS, m.str(1)).expect("_CROWD_NUMBERS key"));
        }
    }
    for word in CROWD_WITHIN.findall_str(&text) {
        numbers.push(lookup_int(&CROWD_NUMBERS, &word).expect("_CROWD_NUMBERS key"));
    }
    for count in A_NUMBER_OF_PEOPLE.findall_str(&text) {
        numbers.push(count.parse().unwrap_or(0));
    }
    if let Some(grouped) = people_in_groups(&text) {
        numbers.push(grouped);
    }
    if let Some(max) = numbers.iter().max() {
        return *max;
    }
    if CROWD_PHRASES.iter().any(|p| text.contains(p.as_str())) {
        return 6;
    }
    0
}

fn crowd_word(count: i64) -> String {
    CROWD_WORDS.iter().find(|(k, _)| *k == count).map(|(_, v)| v.clone()).unwrap_or_else(|| count.to_string())
}

// DUP-D2(bot.knowledge.cooking::_trays)
/// How many trays the table needs, so a table of twelve is not told about eight.
pub fn _trays(count: i64) -> String {
    if count >= 13 {
        return "two or three trays of it, on the top and middle shelves, or one dish made in \
                the biggest pot you own"
            .to_string();
    }
    if count >= 9 {
        return format!("two trays of it, on the top and middle shelves and swapped halfway, to feed {}", crowd_word(count));
    }
    if count != 0 && count <= 5 {
        return format!("one tray of it is plenty for {}", crowd_word(count));
    }
    if count == 0 {
        return "which does four to six, so count the potatoes yourself".to_string();
    }
    "which feeds eight from one oven".to_string()
}

// DUP-D2(bot.knowledge.cooking::_crowd_meal)
/// One dish that scales, for a birthday or a table of eight, instead of a ten-minute dinner.
pub fn _crowd_meal(banned: &std::collections::HashSet<String>, occasion: bool, count: i64) -> String {
    const MEATS: [&str; 17] = [
        "meat", "chicken", "beef", "pork", "lamb", "mince", "bacon", "sausage", "sausages", "ham", "turkey", "fish",
        "salmon", "cod", "tuna", "prawn", "prawns",
    ];
    let vegetarian = MEATS.iter().any(|m| banned.contains(*m));
    let (dish, amount) = if vegetarian {
        (
            "a big mushroom and lentil lasagne or a chickpea and vegetable traybake, made in \
             the largest dish you own",
            "about 300g of the dish per person",
        )
    } else {
        (
            "a tray of chicken thighs or sausages with halved potatoes, peppers and onions at \
             200C (180C fan) for 45 minutes",
            "about 200g of meat and a big potato each",
        )
    };
    let mut reply = format!(
        "Cook one big oven dish rather than several pans: {dish}, {}. Allow {amount}, add a loaf of bread and a bowl of salad, and write the \
         times down so nothing needs the oven at once.",
        _trays(count)
    );
    if occasion {
        reply.push_str(
            " Anything that can be made that morning - a chilli, a stew, a lasagne - just \
             gets reheated, which leaves the day free.",
        );
    }
    reply
}

// DUP-D2(bot.knowledge.cooking::_occasion)
/// True when the line is a day that matters: a birthday, a wake, or a "70th".
pub fn _occasion(text: &str) -> bool {
    if OCCASIONS.iter().any(|w| text.contains(w.as_str())) {
        return true;
    }
    AN_ORDINAL.is_match(text)
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/dishes.py

static INGREDIENT_AFTER: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes._INGREDIENT_AFTER"));
static RULE_OUT_FILLER: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes._RULE_OUT_FILLER"));
static RULE_OUT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes._RULE_OUT"));
/// `dishes.PLANS`: each plan key with the token lists of its names.
static PLAN_NAMES: LazyLock<Vec<(String, Vec<Vec<String>>)>> = LazyLock::new(|| {
    data::get("bot.knowledge.dishes.PLANS")
        .pairs()
        .iter()
        .map(|(k, plan)| {
            let mut names = Vec::new();
            if let Some(groups) = plan.get("names") {
                for group in groups.items() {
                    match group {
                        crate::data::Value::Tuple(_) => {
                            for name in group.items() {
                                names.push(crate::knowledge::d2_shared::tokens(name.str()));
                            }
                        }
                        other => names.push(crate::knowledge::d2_shared::tokens(other.str())),
                    }
                }
            }
            (k.str().to_string(), names)
        })
        .collect()
});

// DUP(bot.knowledge.dishes::_same_word)
/// eggs/egg, potatoes/potato, roasting/roast: the loose forms cooking.py matches with.
pub fn _same_word(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    for (one, two) in [(a, b), (b, a)] {
        if one.chars().count() > 3 {
            if one.strip_suffix('s') == Some(two) {
                return true;
            }
            if one.strip_suffix("es") == Some(two) {
                return true;
            }
            if let Some(stem) = one.strip_suffix("ing") {
                if stem == two || format!("{stem}e") == two {
                    return true;
                }
            }
            if one.strip_suffix("ed") == Some(two) {
                return true;
            }
        }
    }
    false
}

// DUP(bot.knowledge.dishes::_ruled_out_near)
/// True when the words just before `at` rule the dish that starts there out ("isnt a roast").
pub fn _ruled_out_near(have: &[String], at: usize, width: usize) -> bool {
    let after = at + width;
    if after < have.len() {
        let tail = have[after].replace('\u{2019}', "").replace('\'', "");
        if INGREDIENT_AFTER.contains(&tail) {
            return false;
        }
    }
    for step in 1..=at.min(5) {
        let word = have[at - step].replace('\u{2019}', "").replace('\'', "");
        if RULE_OUT_FILLER.contains(&word) {
            continue;
        }
        return RULE_OUT.contains(&word);
    }
    false
}

// DUP(bot.knowledge.dishes::dishes_ruled_out)
/// The plan keys a text names only to rule them out ("isnt a roast").
pub fn dishes_ruled_out(text: &str) -> std::collections::HashSet<String> {
    let have = crate::knowledge::d2_shared::tokens(text);
    let mut out = std::collections::HashSet::new();
    for (key, names) in PLAN_NAMES.iter() {
        for wanted in names {
            if wanted.is_empty() || wanted.len() > have.len() {
                continue;
            }
            for i in 0..(have.len() - wanted.len() + 1) {
                if (0..wanted.len()).all(|j| _same_word(&have[i + j], &wanted[j])) && _ruled_out_near(&have, i, wanted.len()) {
                    out.insert(key.clone());
                    break;
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/entertainment.py (owned by the entertainment group)

static RESERVING_PHRASES: LazyLock<Vec<Re>> = LazyLock::new(|| {
    data::strs("bot.knowledge.entertainment._RESERVING_PHRASES")
        .iter()
        .map(|p| Re::new(&format!("(?<![a-z]){}(?![a-z])", escape(p)), 0))
        .collect()
});
static BOOK_VERB_BEFORE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.entertainment._BOOK_VERB_BEFORE"));
static BOOK_VERB_AFTER: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.entertainment._BOOK_VERB_AFTER"));
static A_DETERMINER_JUST_BEFORE: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.knowledge.entertainment._A_DETERMINER_JUST_BEFORE"));

// DUP-D2(bot.knowledge.entertainment::_a_reservation)
/// Whether this one reading of the line is reserving something.
pub fn _a_reservation(text: &str) -> bool {
    if crate::py::text::strip(text).is_empty() {
        return false;
    }
    if RESERVING_PHRASES.iter().any(|rx| rx.is_match(text)) {
        return true;
    }
    if BOOK_VERB_BEFORE.is_match(text) {
        return true;
    }
    for m in BOOK_VERB_AFTER.finditer(text) {
        if !A_DETERMINER_JUST_BEFORE.is_match(&text[..m.start()]) {
            return true;
        }
    }
    false
}

// DUP-D2(bot.knowledge.entertainment::the_book_is_a_reservation)
/// True when the line is reserving something, which is the booking rows' question.
pub fn the_book_is_a_reservation(understood: &Understood) -> bool {
    for reading in [&understood.clean, &understood.norm_text] {
        if _a_reservation(&reading.to_lowercase()) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// Python's datetime.date, for the replies that name a date.

/// A calendar date (proleptic Gregorian), as Python's `datetime.date`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: i64, month: u32, day: u32) -> Date {
        Date { year, month, day }
    }

    /// Days since 1970-01-01.
    pub fn ordinal(&self) -> i64 {
        let y = if self.month <= 2 { self.year - 1 } else { self.year };
        let era = (if y >= 0 { y } else { y - 399 }) / 400;
        let yoe = y - era * 400;
        let m = self.month as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    /// `(a - b).days`.
    pub fn days_since(&self, other: &Date) -> i64 {
        self.ordinal() - other.ordinal()
    }

    /// Monday is 0, as `date.weekday()`.
    pub fn weekday(&self) -> u32 {
        // 1970-01-01 was a Thursday (3)
        ((self.ordinal() + 3).rem_euclid(7)) as u32
    }

    /// `strftime("%A")`.
    pub fn weekday_name(&self) -> &'static str {
        ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"][self.weekday() as usize]
    }

    /// `strftime("%B")`.
    pub fn month_name(&self) -> &'static str {
        [
            "January", "February", "March", "April", "May", "June", "July", "August", "September", "October",
            "November", "December",
        ][(self.month - 1) as usize]
    }
}

// NEEDS(shared clock): `datetime.date.today()`. PyBot's oracle runs pin the clock to 2026-11-17;
// RSBOT_TODAY=YYYY-MM-DD overrides it.
pub fn today() -> Date {
    if let Ok(s) = std::env::var("RSBOT_TODAY") {
        let parts: Vec<i64> = s.split('-').filter_map(|p| p.parse().ok()).collect();
        if parts.len() == 3 {
            return Date::new(parts[0], parts[1] as u32, parts[2] as u32);
        }
    }
    Date::new(2026, 11, 17)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dates_match_python() {
        let cases = [
            ((2026, 11, 17), "Tuesday", 20774),
            ((2026, 12, 19), "Saturday", 20806),
            ((2027, 7, 21), "Wednesday", 21020),
            ((2000, 2, 29), "Tuesday", 11016),
            ((1969, 12, 31), "Wednesday", -1),
        ];
        for ((y, m, d), name, days) in cases {
            let date = Date::new(y, m, d);
            assert_eq!(date.weekday_name(), name);
            assert_eq!(date.ordinal(), days);
        }
    }
}
