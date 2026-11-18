//! Port of `bot/knowledge/cooking_gaps.py`: everyday kitchen questions the cooking table had no
//! answer for (a roast for a crowd, measuring, reading a recipe, herbs from the garden, a table
//! of people...). Tables and entries are data from the bundle.

use crate::data;
use crate::data::FnRef;
use crate::knowledge::d2dup_g9::{_crowd, _crowd_meal, _occasion, dishes_ruled_out};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{escape, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.cooking_gaps.{name}"))
}
fn string(name: &str) -> String {
    data::string(&format!("bot.knowledge.cooking_gaps.{name}"))
}
fn re(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.cooking_gaps.{name}"))
}

/// `(?<![a-z0-9])<word>(?![a-z0-9])`: the word standing on its own.
fn spoken_rx(word: &str) -> Re {
    Re::new(&format!("(?<![a-z0-9]){}(?![a-z0-9])", escape(word)), 0)
}

static _COUNT_WORDS: LazyLock<Vec<(String, i64, Re)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking_gaps._COUNT_WORDS")
        .items()
        .iter()
        .map(|row| {
            let word = row.items()[0].str().to_string();
            let rx = spoken_rx(&word);
            (word, row.items()[1].as_int().unwrap_or(0), rx)
        })
        .collect()
});
static _THE_MEAL_NAMED: LazyLock<&'static Re> = LazyLock::new(|| re("_THE_MEAL_NAMED"));
static _THE_TABLE_NAMED: LazyLock<&'static Re> = LazyLock::new(|| re("_THE_TABLE_NAMED"));
static _TABLE_WORDS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking_gaps._TABLE_WORDS")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0)))
        .collect()
});
static _A_TABLE_PUT_RIGHT: LazyLock<&'static Re> = LazyLock::new(|| re("_A_TABLE_PUT_RIGHT"));
static _NOT_THE_CROWD_CARD: LazyLock<Vec<Re>> = LazyLock::new(|| {
    strs("_NOT_THE_CROWD_CARD").iter().map(|w| Re::new(&format!(r"(?<!\w){}(?!\w)", escape(w)), 0)).collect()
});
static _CHILD_DRANK_IT: LazyLock<&'static Re> = LazyLock::new(|| re("_CHILD_DRANK_IT"));
static _ORDER_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("_ORDER_ASK"));
static _HERB_PLANT_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HERB_PLANT_WORDS"));
static _HERB_USE_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HERB_USE_ASK"));
static _WOODY_HERB: LazyLock<Vec<String>> = LazyLock::new(|| strs("_WOODY_HERB"));
static _HERB_OWNED: LazyLock<&'static Re> = LazyLock::new(|| re("_HERB_OWNED"));
/// `_CUP_OF`: each row's words (as `\b<word>\b` patterns) and its reply (the row's last item).
static _CUP_OF: LazyLock<Vec<(Vec<Re>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking_gaps._CUP_OF")
        .items()
        .iter()
        .map(|row| {
            let words = row.strs();
            let (last, rest) = words.split_last().expect("a _CUP_OF row");
            (rest.iter().map(|w| Re::new(&format!(r"\b{}\b", escape(w)), 0)).collect(), last.clone())
        })
        .collect()
});
static _MEASURE_AWAY: LazyLock<Vec<Re>> =
    LazyLock::new(|| strs("_MEASURE_AWAY").iter().map(|w| Re::new(&format!(r"\b{}\b", escape(w)), 0)).collect());
static _MEASURING_TAIL: LazyLock<String> = LazyLock::new(|| string("_MEASURING_TAIL"));
/// `_RECIPE_TERM`: (pattern, reply) per row.
static _RECIPE_TERM: LazyLock<Vec<(Re, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking_gaps._RECIPE_TERM")
        .items()
        .iter()
        .map(|row| (Re::new(row.items()[1].str(), 0), row.items()[2].str().to_string()))
        .collect()
});
static _POWDER_FOR_SODA: LazyLock<&'static Re> = LazyLock::new(|| re("_POWDER_FOR_SODA"));
static _A_MEAL_IS_NAMED: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_MEAL_IS_NAMED"));
static _ALLOWED_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ALLOWED_ASKS"));
static _EATING_VERBS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_EATING_VERBS"));
static MEAL_SITUATIONS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking_gaps.MEAL_SITUATIONS")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
});
static MEAL_ASK_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("MEAL_ASK_WORDS"));
static _BAKE_NAMED: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BAKE_NAMED"));
static _A_TABLE_OF_PEOPLE: LazyLock<&'static Re> = LazyLock::new(|| re("_A_TABLE_OF_PEOPLE"));
static _NOT_A_TABLE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NOT_A_TABLE"));
static _A_DIET: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking_gaps._A_DIET")
        .items()
        .iter()
        .map(|row| (row.items()[0].str().to_string(), row.items()[1].str().to_string()))
        .collect()
});

static TWO_DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<![a-z0-9])(\d{1,2})(?![a-z0-9])", 0));
static NOT_A_WORD_CHAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));

fn any_in<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    words.iter().any(|w| text.contains(w.as_ref()))
}

/// Whether the word stands on its own in the line.
pub fn _spoken(line: &str, word: &str) -> bool {
    spoken_rx(word).is_match(line)
}

/// The number the chat said this meal is for, or 0 when no earlier line set one.
pub fn _the_table_the_chat_set(memory: &Memory) -> i64 {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(8)..].iter().rev() {
        let said = text::squash(&turn.user.to_lowercase());
        if said.is_empty() {
            continue;
        }
        let Some(found) = _THE_TABLE_NAMED.search(&said) else { continue };
        if !(_THE_MEAL_NAMED.is_match(&said) || _A_TABLE_PUT_RIGHT.is_match(&said)) {
            continue;
        }
        let raw = {
            let n = found.name_str("n");
            if n.is_empty() { found.name_str("only") } else { n }
        }
        .to_lowercase();
        let count = if text::is_digit(&raw) {
            raw.parse::<i64>().unwrap_or(0)
        } else {
            _TABLE_WORDS.iter().find(|(k, _)| *k == raw).map(|(_, v)| *v).unwrap_or(0)
        };
        if (2..=30).contains(&count) {
            return count;
        }
    }
    0
}

/// The number of people the line is cooking for, or 0 when it names none.
pub fn _roast_head_count(line: &str) -> i64 {
    for found in TWO_DIGITS.findall_str(line) {
        let count: i64 = found.parse().unwrap_or(0);
        if (4..=30).contains(&count) {
            return count;
        }
    }
    for (_word, count, rx) in _COUNT_WORDS.iter() {
        if rx.is_match(line) {
            return *count;
        }
    }
    0
}

/// The raw weight to buy, said the way a butcher would say it.
pub fn _kilos(count: i64) -> String {
    let grams = count * 250;
    if grams % 1000 == 0 {
        return format!("{}kg", grams / 1000);
    }
    if grams % 500 == 0 {
        return format!("{}.5kg", grams / 1000);
    }
    format!("{:.2}kg", grams as f64 / 1000.0)
}

/// The time this card's own rule gives a joint of this weight: 20 minutes per 500g plus 20.
pub fn _the_time_a_joint_needs(grams: i64) -> String {
    let minutes = (grams as f64 / 25.0) as i64 + 20;
    let (hours, minutes) = (minutes.div_euclid(60), minutes.rem_euclid(60));
    if hours > 1 {
        return if minutes != 0 { format!("{hours} hours {minutes} minutes") } else { format!("{hours} hours") };
    }
    if hours == 1 {
        return if minutes != 0 { format!("an hour and {minutes} minutes") } else { "an hour".to_string() };
    }
    format!("{minutes} minutes")
}

/// True when the line itself names a dish, a joint or a diet the crowd card is not for.
pub fn _the_dish_the_line_names(line: &str) -> bool {
    if _NOT_THE_CROWD_CARD.iter().any(|rx| rx.is_match(line)) {
        return true;
    }
    dishes_ruled_out(line).contains("sunday-roast")
}

/// The joint, the buy weight and the timings for a roast for the table the line names.
pub fn _roast_for_a_crowd(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let line = format!(" {} ", NOT_A_WORD_CHAR.sub(" ", &said.to_lowercase()));
    if _the_dish_the_line_names(&line) {
        return None;
    }
    let mut count = _roast_head_count(&line);
    if count == 0 {
        count = _the_table_the_chat_set(memory);
    }
    if count == 0 {
        return Some(format!(
            "Allow about 250g of raw meat each with the bone in, or 150g boneless: six people \
             want a 2kg chicken or a 1.5kg joint. Work backwards from the time you want to eat: \
             meat in at 190C (170C fan) {} before, because a 2kg bird \
             is 20 minutes per 500g plus 20. Potatoes in an hour before, parboiled and \
             rough in hot fat at 200C for 45 minutes. Stuffing and pigs in blankets 25 minutes \
             before the end, greens 10. The meat comes out to rest 15 minutes under foil, and \
             the gravy is made in the tin while it does.",
            _the_time_a_joint_needs(2000)
        ));
    }
    let kilos = _kilos(count);
    let length = _the_time_a_joint_needs(count * 250);
    Some(format!(
        "For {count} people allow about 250g of raw meat each with the bone in, or 150g boneless, so \
         buy about {kilos} in total: one joint that size, or two good-sized chickens. \
         Work backwards from the time you want to eat: meat in at 190C (170C fan) {length} \
         before, because a joint is 20 minutes per 500g plus 20. Potatoes in an hour \
         before, parboiled and rough in hot fat at 200C \
         for 45 minutes. Stuffing and pigs in blankets 25 minutes before the end, greens 10. \
         The meat comes out to rest 15 minutes under foil, and the gravy is made in the tin \
         while it does."
    ))
}

/// The answer for a doubled, halved or differently tinned recipe, chosen by the words said.
pub fn _recipe_amount(_memory: &mut Memory, understood: &Understood) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let said = format!(" {} ", line.to_lowercase());
    let tin = any_in(&said, &[" tin ", " tins ", " sandwich tin", " cake tin"]);
    let other_size = any_in(
        &said,
        &[
            "bigger", "larger", "smaller", "different size", "different tin", "other tin", "wrong size", "too big",
            "too small", "deeper",
        ],
    );
    if tin && other_size {
        return "Go by the area of the tin rather than the numbers on the page: a 23cm round holds \
                about a third more than a 20cm, so the same batter fills it shallower and bakes \
                5 to 10 minutes faster, and scaling the batter up by that third fills it to the \
                same depth in the same time. Keep the oven temperature, fill to two-thirds, and \
                check with a skewer 5 minutes before the recipe's time either way."
            .to_string();
    }
    let halving = any_in(
        &said,
        &[
            "halve", "halving", "half the recipe", "half a recipe", "half this recipe", "half batch", "smaller batch",
            "only two of us", "just for one", "just me", "for two",
        ],
    );
    if halving {
        return "Halve everything except the egg and the raising agent: beat the egg and weigh \
                half of it, and keep the baking powder or yeast nearer two-thirds, because it \
                does not scale down in a straight line. Use a smaller tin or pan so the depth \
                stays the same, keep the temperature, and start checking 5 minutes earlier - a \
                smaller batch comes up to heat and browns sooner."
            .to_string();
    }
    "No - the time barely changes, the quantity does. Keep the oven or hob where it was \
     and add 10 to 20 minutes for the extra volume to heat through and reduce, checking \
     early rather than trusting the clock. Use a wider pot or two tins rather than a \
     deeper one, swap two trays on their shelves halfway, and cut the liquid by about a \
     tenth because less of it evaporates. Salt, chilli, stock and herbs do not double \
     exactly: add three-quarters, taste, then top up."
        .to_string()
}

fn lower_words(understood: &Understood) -> HashSet<String> {
    understood.words.iter().map(|w| w.to_lowercase()).collect()
}

/// True when the line asks for a use, or says the growing herb is theirs.
pub fn _herb_use_asked(understood: &Understood) -> bool {
    let clean = format!(" {} ", understood.clean.to_lowercase());
    let words = lower_words(understood);
    if _HERB_USE_ASK.iter().any(|w| words.contains(w)) {
        return true;
    }
    if _HERB_USE_ASK.iter().any(|w| clean.contains(&format!(" {w} "))) {
        return true;
    }
    if !_HERB_PLANT_WORDS.iter().any(|w| clean.contains(&format!(" {w} "))) {
        return false;
    }
    _HERB_OWNED.is_match(&clean)
}

/// The picking and the herb's own use - woody at the start of the cooking, soft at the end.
pub fn _herb_plant_reply(_memory: &mut Memory, understood: &Understood) -> String {
    let words = lower_words(understood);
    let clean = format!(" {} ", understood.clean.to_lowercase());
    let woody = _WOODY_HERB.iter().any(|h| words.contains(h)) || _WOODY_HERB.iter().any(|h| clean.contains(&format!(" {h} ")));
    let (r#use, pick) = if woody {
        (
            "Use them at the start of the cooking, because heat is what opens them up: \
             rosemary or thyme with potatoes, lamb, chicken or a tray of vegetables, sage \
             with pork or in a brown butter, and a bay leaf in rice or a stew.",
            "Take a few sprigs from the tips rather than cutting back to the old wood, and a \
             third off the top is the most a bush wants at once - it grows back from there. ",
        )
    } else {
        (
            "Use them at the end, off the heat, so the flavour stays - chopped into a green \
             sauce with oil, lemon and garlic, torn over tomatoes or fish, or stirred into a \
             soup just before it is served.",
            "Pinch the top few leaves and leave the stem, and the plant keeps going: a third \
             off the top is plenty at once, and it grows back from the cuts. ",
        )
    };
    if _herb_use_asked(understood) {
        return format!("{} {}", r#use, text::strip(pick));
    }
    format!("{pick}{}", r#use)
}

/// The words the person typed, cleaned, with a space at each end.
pub fn _lined(_memory: &Memory, understood: &Understood) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    format!(" {} ", text::squash(&line.to_lowercase()))
}

/// How to measure the ingredient the line names in cups, or the general rule.
pub fn _measuring(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _lined(memory, understood);
    if _MEASURE_AWAY.iter().any(|rx| rx.is_match(&line)) {
        return None;
    }
    for (words, reply) in _CUP_OF.iter() {
        if words.iter().any(|rx| rx.is_match(&line)) {
            return Some(format!("{reply} {}", *_MEASURING_TAIL));
        }
    }
    Some(_MEASURING_TAIL.clone())
}

/// What the recipe term the line asks about means, or None.
pub fn _the_recipe_term(memory: &Memory, understood: &Understood) -> Option<String> {
    let line = _lined(memory, understood);
    for (rx, reply) in _RECIPE_TERM.iter() {
        if rx.is_match(&line) {
            return Some(reply.clone());
        }
    }
    None
}

/// The read-a-recipe row's reply (the lambda): the term asked about, or how to read one.
pub fn _read_a_recipe(memory: &mut Memory, understood: &Understood) -> String {
    _the_recipe_term(memory, understood).filter(|s| !s.is_empty()).unwrap_or_else(|| {
        "Read it twice, all the way through, before you touch anything: what it serves, the prep \
         time and the cooking time separately, the oven setting, and whether anything has to chill, \
         soak or rest. Then get every ingredient out and cut up, which is what the French call mise \
         en place - most recipes go wrong in the last five minutes because the onion was not chopped \
         yet. Two phrases to know: 'divided' means the amount is split between stages (half the \
         cheese in the sauce, half on top), and 'until' is a cue rather than a time - until golden, \
         until it comes away from the pan. Times in a recipe are for that oven and that tin; trust \
         your eyes and nose first. Check the tin size too, because a wrong-sized tin changes the \
         depth and with it the time."
            .to_string()
    })
}

/// Whether the line asks to use baking powder in place of soda.
pub fn _powder_goes_in_for_soda(understood: &Understood) -> bool {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    _POWDER_FOR_SODA.is_match(&format!(" {} ", line.to_lowercase()))
}

/// True when the line asks whether a food is allowed rather than asking for a recipe.
pub fn _asks_what_is_allowed(said: &str) -> bool {
    if any_in(said, &_ALLOWED_ASKS) {
        return true;
    }
    if !any_in(said, &_EATING_VERBS) {
        return false;
    }
    any_in(said, &[" can ", " could ", " should ", " is ", " are "])
}

/// The row answers a sweet that is asked for, and stands aside for a meal or a permission.
pub fn _a_bake_is_the_ask(understood: &Understood) -> bool {
    let said = format!(" {} ", understood.clean.to_lowercase());
    if _A_MEAL_IS_NAMED.iter().any(|w| said.contains(&format!(" {w} "))) {
        return false;
    }
    !_asks_what_is_allowed(&said)
}

/// The dinner the situation wants, or "" when the line names no situation.
pub fn _meal_for_the_situation(_memory: &mut Memory, understood: &Understood) -> String {
    let said = understood.clean.to_lowercase();
    if !any_in(&said, &MEAL_ASK_WORDS) {
        return String::new();
    }
    for (words, reply) in MEAL_SITUATIONS.iter() {
        if any_in(&said, words) {
            return reply.clone();
        }
    }
    String::new()
}

/// True when the line names something being baked, so the sugar is in a recipe.
pub fn _a_bake_is_named(said: &str) -> bool {
    let words = format!(" {} ", said.to_lowercase());
    _BAKE_NAMED.iter().any(|name| words.contains(&format!(" {name} ")))
}

/// Whether the line states a table of people, with no meal and no cooking verb in it.
pub fn _a_table_announced(understood: &Understood) -> bool {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let text = format!(" {} ", line.to_lowercase());
    if !_A_TABLE_OF_PEOPLE.is_match(&text) {
        return false;
    }
    !any_in(&text, &_NOT_A_TABLE)
}

/// The one-dish answer for a stated table, sized by the number in the line (and the chat).
pub fn the_table_of_people(memory: &mut Memory, understood: &Understood) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let mut text = line.clone();
    let turns = &memory.turns;
    // Python: `text += " " + " ".join(...)` whenever turns is a list, even an empty one.
    let users: Vec<&str> = turns[turns.len().saturating_sub(4)..].iter().map(|t| t.user.as_str()).collect();
    text = format!("{text} {}", users.join(" "));
    let text = text.to_lowercase();
    let mut banned: HashSet<String> = HashSet::new();
    for (word, food) in _A_DIET.iter() {
        if text.contains(word.as_str()) {
            banned.insert(food.clone());
        }
    }
    _crowd_meal(&banned, _occasion(&text), _crowd(&text))
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.cooking_gaps:<lambda>:1599", |_: &FnRef, u: &Understood| {
        _CHILD_DRANK_IT.is_match(&format!(" {} ", u.clean.to_lowercase()))
    }),
    ("bot.knowledge.cooking_gaps:<lambda>:1891", |_: &FnRef, u: &Understood| {
        _ORDER_ASK.is_match(&format!(" {} ", u.clean.to_lowercase()))
    }),
    ("bot.knowledge.cooking_gaps:<lambda>:2635", |_: &FnRef, u: &Understood| _a_bake_is_named(&u.clean)),
    ("bot.knowledge.cooking_gaps:_a_bake_is_the_ask:2543", |_: &FnRef, u: &Understood| _a_bake_is_the_ask(u)),
    ("bot.knowledge.cooking_gaps:_a_table_announced:2703", |_: &FnRef, u: &Understood| _a_table_announced(u)),
    ("bot.knowledge.cooking_gaps:_herb_use_asked:1932", |_: &FnRef, u: &Understood| _herb_use_asked(u)),
    ("bot.knowledge.cooking_gaps:_powder_goes_in_for_soda:2286", |_: &FnRef, u: &Understood| _powder_goes_in_for_soda(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.cooking_gaps:<lambda>:2172", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_read_a_recipe(m, u))),
    ("bot.knowledge.cooking_gaps:_herb_plant_reply:1950", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_herb_plant_reply(m, u))
    }),
    ("bot.knowledge.cooking_gaps:_meal_for_the_situation:2564", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_meal_for_the_situation(m, u))
    }),
    ("bot.knowledge.cooking_gaps:_measuring:2078", |_: &FnRef, m: &mut Memory, u: &Understood| _measuring(m, u)),
    ("bot.knowledge.cooking_gaps:_recipe_amount:1704", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_recipe_amount(m, u))
    }),
    ("bot.knowledge.cooking_gaps:_roast_for_a_crowd:803", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _roast_for_a_crowd(m, u)
    }),
    ("bot.knowledge.cooking_gaps:the_table_of_people:2717", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(the_table_of_people(m, u))
    }),
];
