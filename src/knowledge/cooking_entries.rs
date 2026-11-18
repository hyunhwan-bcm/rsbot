//! Port of `bot/knowledge/cooking_entries.py`: the how-to-cook entries (kept apart from cooking.py
//! for size). The entries are data in the bundle; this is every guard and reply function they
//! use, with the helpers those read.
//!
//! Build-time only (not ported): `_make_it` and `_swap` build keys / entries at import time.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d2_shared::{amends_to, line_of};
use crate::knowledge::d2dup_g6 as dup;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use indexmap::IndexMap;
use std::collections::HashSet;
use std::sync::LazyLock;

fn name(n: &str) -> String {
    format!("bot.knowledge.cooking_entries.{n}")
}
fn s(n: &str) -> String {
    data::string(&name(n))
}
fn strs(n: &str) -> Vec<String> {
    data::strs(&name(n))
}
fn set(n: &str) -> HashSet<String> {
    data::set(&name(n))
}
fn rx(n: &str) -> &'static Re {
    data::re(&name(n))
}
fn pairs(n: &str) -> Vec<(String, String)> {
    data::get(&name(n)).items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string())).collect()
}
fn any_in(text: &str, words: &[String]) -> bool {
    words.iter().any(|w| text.contains(w.as_str()))
}
fn set_hits(words: &HashSet<String>, of: &[String]) -> bool {
    of.iter().any(|w| words.contains(w))
}
fn words_set(u: &Understood) -> HashSet<String> {
    u.words.iter().cloned().collect()
}
fn lower_words(u: &Understood) -> Vec<String> {
    u.words.iter().map(|w| w.to_lowercase()).collect()
}

static _RICE_WAS_COOKED: LazyLock<&'static Re> = LazyLock::new(|| rx("_RICE_WAS_COOKED"));
static _TAKING_SOMEBODY: LazyLock<&'static Re> = LazyLock::new(|| rx("_TAKING_SOMEBODY"));
static _OTHER_PEOPLE_AT_LUNCH: LazyLock<&'static Re> = LazyLock::new(|| rx("_OTHER_PEOPLE_AT_LUNCH"));
static _PACKED_LUNCH_NOUNS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PACKED_LUNCH_NOUNS"));
static STOCK_BROWN: LazyLock<Vec<String>> = LazyLock::new(|| strs("STOCK_BROWN"));
static STOCK_FISH: LazyLock<Vec<String>> = LazyLock::new(|| strs("STOCK_FISH"));
static STOCK_VEG: LazyLock<Vec<String>> = LazyLock::new(|| strs("STOCK_VEG"));
static STOCK_METHODS: LazyLock<IndexMap<String, (String, String)>> = LazyLock::new(|| {
    data::get(&name("STOCK_METHODS"))
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), (v.items()[0].str().to_string(), v.items()[1].str().to_string())))
        .collect()
});
static _SMOKE_THINGS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SMOKE_THINGS"));
static _SMOKE_REPLIES: LazyLock<std::collections::HashMap<String, String>> =
    LazyLock::new(|| data::str_map(&name("_SMOKE_REPLIES")));
static _SMOKING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_SMOKING_WORDS"));
static _SMOKING_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SMOKING_PHRASES"));
static _CHEESE_TROUBLE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_CHEESE_TROUBLE_WORDS"));
static _CHEESE_KIND_KEEP: LazyLock<std::collections::HashMap<String, String>> =
    LazyLock::new(|| data::str_map(&name("_CHEESE_KIND_KEEP")));
static BLUE_CHEESES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.keeping.BLUE_CHEESES"));
static SOFT_CHEESES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.keeping.SOFT_CHEESES"));
static CHEESE_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.keeping.CHEESE_NAMES"));
static _OATS_HEALTH: LazyLock<Vec<String>> = LazyLock::new(|| strs("_OATS_HEALTH"));
static _TEA_MILK_BREW: LazyLock<Vec<String>> = LazyLock::new(|| strs("_TEA_MILK_BREW"));
static _TEA_MILK_PAN: LazyLock<Vec<String>> = LazyLock::new(|| strs("_TEA_MILK_PAN"));
static TEA_MILK_AMOUNT_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("TEA_MILK_AMOUNT_ASKS"));
static TEA_MILK_KIND_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("TEA_MILK_KIND_ASKS"));
static TEA_MILK_PLANT_SAID: LazyLock<Vec<String>> = LazyLock::new(|| strs("TEA_MILK_PLANT_SAID"));
static _A_ITS_OWN_FAULT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_ITS_OWN_FAULT"));
static _A_SUPPLY_VERB: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_SUPPLY_VERB"));
static _A_WITHOUT_SUBJECT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_WITHOUT_SUBJECT"));
static _A_SEASON_OF_A_SHOW: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_SEASON_OF_A_SHOW"));
static PACK_DATE: LazyLock<Vec<String>> = LazyLock::new(|| strs("PACK_DATE"));
static BREAD_HOW_LONG: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\bhow long\b|\bhow many days\b|\bhow many weeks\b|\bshelf[- ]?life\b|\blasts? for\b", 0)
});
static CHILD_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| set("CHILD_NAMES"));
static CHILDS_LUNCH_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("CHILDS_LUNCH_WORDS"));
static NO_SANDWICH: LazyLock<Vec<String>> = LazyLock::new(|| strs("NO_SANDWICH"));
static _SNACK_FAMILY_OF: LazyLock<std::collections::HashMap<String, String>> =
    LazyLock::new(|| data::str_map(&name("_SNACK_FAMILY_OF")));
static _NO_SANDWICH_KID_ITEMS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs("_NO_SANDWICH_KID_ITEMS"));
static _NO_SANDWICH_ADULT_ITEMS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs("_NO_SANDWICH_ADULT_ITEMS"));
static COFFEE_STOP_ASK: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\bhow long\b|\bhow many hours\b|\bwhat time\b|\bwhen should i stop\b|\bstop drinking\b|\bcut off\b|\blast (?:cup|coffee)\b|\bhow late\b|\bbefore bed\b|\bbedtime\b|\bbefore sleep\b",
        0,
    )
});
static COFFEE_MAY_I: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\b(?:should|shall|can|could|may)\s+(?:i|we)\b|\b(?:is|would)\s+it\s+(?:ok|okay|alright|fine|bad|a\s+bad\s+idea)\b|\bdo\s+i\s+dare\b",
        0,
    )
});
static _MAKE_IT_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("_MAKE_IT_ASK"));
static _COOKING_CLOCK: LazyLock<&'static Re> = LazyLock::new(|| rx("_COOKING_CLOCK"));
static _A_KEPT_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_KEPT_THING"));
static _BREAST_MILK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BREAST_MILK"));
static STORE_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("STORE_ASKS"));
static _DRINK_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DRINK_NAMES"));
static _COOKING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_COOKING_WORDS"));
static _ASK_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ASK_WORDS"));
static _DOSE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DOSE_WORDS"));
static _SNACK_OPTIONS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs("_SNACK_OPTIONS"));
static _COFFEE_LEADS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs("_COFFEE_LEADS"));
static _THE_MILK_ONLY: LazyLock<Vec<String>> = LazyLock::new(|| strs("_THE_MILK_ONLY"));
static FREEZE_DISH_TIMES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&name("FREEZE_DISH_TIMES"))
        .items()
        .iter()
        .map(|r| (r.items()[0].strs(), r.items()[1].str().to_string()))
        .collect()
});
static _HOT_SAID: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOT_SAID"));
static _LOAF: LazyLock<Vec<String>> = LazyLock::new(|| strs("_LOAF"));
static _BATTER_IN_AN_IRON: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BATTER_IN_AN_IRON"));
static _BATTER_IN_A_TIN: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BATTER_IN_A_TIN"));
static _FROZEN_SAID: LazyLock<HashSet<String>> = LazyLock::new(|| set("_FROZEN_SAID"));
static _A_PORTION_OF_SLICES: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_PORTION_OF_SLICES"));
static _STICKING_A_CUT: LazyLock<&'static Re> = LazyLock::new(|| rx("_STICKING_A_CUT"));
static INGREDIENT_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut named = HashSet::new();
    for (k, forms) in data::get("bot.knowledge.meals.INGREDIENTS").pairs() {
        if k.str() == "veg" {
            continue;
        }
        named.extend(forms.strs());
    }
    named
});
static _PORRIDGE_SALT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PORRIDGE_SALT"));
static _PORRIDGE_BLAND: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PORRIDGE_BLAND"));
static _HOT_PAN_REFUSES: LazyLock<Vec<(String, Re, String)>> = LazyLock::new(|| {
    data::get(&name("_HOT_PAN_REFUSES"))
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            (r[0].str().to_string(), Re::new(&format!("(?:{})", r[1].str()), 0), r[2].str().to_string())
        })
        .collect()
});
static _HOT_PAN_TAKES: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    data::get(&name("_HOT_PAN_TAKES"))
        .items()
        .iter()
        .map(|r| (r.items()[0].str().to_string(), Re::new(&format!("(?:{})", r.items()[1].str()), 0)))
        .collect()
});
static _JOINT_WORDS: LazyLock<Vec<Re>> =
    LazyLock::new(|| strs("_JOINT_WORDS").iter().map(|w| Re::new(&format!(r"\b{w}\b"), 0)).collect());
static _A_FAT_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_FAT_WORD"));
static _A_JOINT_TIMING: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_JOINT_TIMING"));
static _ROAST_STUCK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ROAST_STUCK"));
static _ROAST_HARD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ROAST_HARD"));
static _ROAST_BURNT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ROAST_BURNT"));
static KEEP_QUALITIES: LazyLock<Vec<String>> = LazyLock::new(|| strs("KEEP_QUALITIES"));
static _KEEP_FOOD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_FOOD"));
static _KEEP_STICK_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_STICK_WORDS"));
static _KEEP_BOILED: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_BOILED"));
static _KEEP_HOT_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_HOT_WORDS"));
static _KEEP_GRAIN_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_GRAIN_WORDS"));
static _KEEP_POUR_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_POUR_WORDS"));
static _KEEP_COVERED_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_COVERED_WORDS"));
static _KEEP_FLATBREAD_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_FLATBREAD_WORDS"));
static _KEEP_STACK_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_STACK_WORDS"));
static _KEEP_CRISP_HELD_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_CRISP_HELD_WORDS"));
static _KEEP_SOFT_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_SOFT_WORDS"));
static _KEEP_MEAT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_MEAT"));
static _SMOKED_FISH: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SMOKED_FISH"));
static DECIDING: LazyLock<Vec<Re>> = LazyLock::new(|| {
    data::strs("bot.knowledge.cooking.DECIDING")
        .iter()
        .map(|w| Re::new(&format!(r"\b{}\b", crate::py::re::escape(w)), 0))
        .collect()
});
static WORDS_ONLY: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// Whether the rice in the line was already cooked (leftover, takeaway, in the fridge).
pub fn _the_rice_was_cooked(understood: &Understood) -> bool {
    let said = dup_or(&understood.clean, &understood.text).to_lowercase();
    _RICE_WAS_COOKED.is_match(&said)
}

fn dup_or<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.is_empty() { b } else { a }
}

/// True unless the line is taking somebody somewhere, which is a day out and not a buffet.
pub fn _not_a_trip(understood: &Understood) -> bool {
    let said = dup_or(&understood.norm_text, &understood.text).to_lowercase();
    !_TAKING_SOMEBODY.is_match(&said)
}

/// Whether the line is one person's lunch rather than a meal for other people.
pub fn _one_persons_lunch(understood: &Understood) -> bool {
    let mut lines = vec![understood.text.to_lowercase(), understood.clean.to_lowercase()];
    if lines.iter().all(|l| l.is_empty()) {
        lines = vec![understood.words.join(" ").to_lowercase()];
    }
    for said in &lines {
        if any_in(said, &_PACKED_LUNCH_NOUNS) {
            return true;
        }
        if _OTHER_PEOPLE_AT_LUNCH.is_match(said) {
            return false;
        }
        let table = dup::_crowd(said); // cooking::_crowd (re-exported by d2dup_g6)
        if table >= 6 || (table >= 3 && said.contains("of us")) {
            return false;
        }
    }
    true
}

/// The three bottles people open: a cork, a screw cap, and a sparkling one.
pub fn _open_wine(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = format!(" {} ", understood.clean);
    if [" screw cap", " screwcap", " cap ", " caps ", " cap,", " cap.", " cap?"].iter().any(|w| line.contains(w)) {
        return Some("A screw cap just twists off with a crack, and the ring stays on the neck - no corkscrew, and the cap goes back on between glasses. If it will not turn, hold the top still and turn the bottle.".into());
    }
    if [" prosecco", " champagne", " sparkling", " cava", " fizz", " bubbly", " sekt", " cremant"]
        .iter()
        .any(|w| line.contains(w))
    {
        return Some("Chill it, then hold the cork and turn the bottle rather than the cork, pointing it away from people and anything breakable, and let it sigh out rather than bang. Sparkling wine is opened cold and slowly because the gas is the wine.".into());
    }
    Some("Cut the foil below the rim, then stand the bottle on the table and turn the corkscrew into the middle of the cork; lever it out in two stages on the hinge and ease the last bit by hand so it does not pop. If the cork is dry and stubborn, wrap the neck in a warm cloth for a minute first. A screw cap simply twists off.".into())
}

/// Porridge on the hob, or in the microwave when the line says so.
pub fn _porridge(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = dup_or(&understood.clean, &understood.text).to_lowercase();
    if said.contains("microwave") || said.contains("microwaving") || text::split_ref(&said).contains(&"micro") {
        return Some("Put 40g of porridge oats in a big bowl with 250ml of milk or water - a big bowl, because it climbs the sides - and microwave it on high for 2 minutes. Stir it, then give it another 30 seconds to a minute until it is as thick as you like; it thickens more as it stands, so stop while it still looks a little loose. A pinch of salt and a spoon of honey or jam at the end.".into());
    }
    Some("Simmer 1 part oats to 3 parts liquid for 5 minutes, stirring now and then, and add a pinch of salt; that gives creamy porridge rather than lumps. Milk added at the end keeps it creamier than cooking it all in milk. In a microwave, a single bowl takes 2 minutes on high, stirred halfway; stop early, as it thickens as it stands.".into())
}

/// The stock pot this line is about: the kind of bone from the line, the ask from the line.
pub fn _stock_answer(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = lower_words(understood);
    if said.is_empty() {
        return None;
    }
    let line = format!(" {} ", said.join(" "));
    let says = |words: &[&str]| words.iter().any(|w| line.contains(&format!(" {w} ")));
    let says_v = |words: &[String]| words.iter().any(|w| line.contains(&format!(" {w} ")));
    let kind = if says_v(&STOCK_VEG) {
        "veg"
    } else if says_v(&STOCK_FISH) {
        "fish"
    } else if says_v(&STOCK_BROWN) {
        "brown"
    } else {
        "chicken"
    };
    let (time_reply, method) = &STOCK_METHODS[kind];
    let cubes = says(&["cube", "cubes", "granule", "granules", "bouillon"]);
    let keeps = says(&[
        "keep", "keeps", "keeping", "keep it", "store", "storing", "last", "lasts", "freeze", "freezes", "freezing",
        "frozen", "freezer", "fridge", "use by", "go off", "gone off", "out of date", "expire", "expired", "best before",
    ]);
    if cubes && keeps {
        return Some(s("STOCK_CUBE_KEEP"));
    }
    if cubes {
        return Some(s("STOCK_CUBES"));
    }
    if keeps {
        return Some(s("STOCK_KEEP"));
    }
    if says(&["slow cooker", "crockpot", "crock pot", "pressure cooker", "slow"]) {
        return Some(s("STOCK_SLOW"));
    }
    if says(&["skim", "skimming", "scum", "foam", "froth"]) {
        return Some(s("STOCK_SKIM"));
    }
    if says(&["healthy", "healthier", "good for", "benefit", "benefits", "collagen", "nutritious", "nutrition"]) {
        return Some(s("STOCK_HEALTH"));
    }
    if says(&["flavour", "flavor", "taste", "tastes", "bland", "better", "stronger", "strong", "rich", "richer"]) {
        return Some(s("STOCK_FLAVOUR"));
    }
    if says(&["how much", "how many"]) {
        return Some(s("STOCK_AMOUNT"));
    }
    if says(&[
        "how long", "long", "hours", "hour", "minutes", "minute", "time", "simmer", "simmering", "boil", "boiling",
        "cook", "cooking",
    ]) {
        return Some(time_reply.clone());
    }
    Some(method.clone())
}

/// A guard: a food that is already smoked is a recipe, not a question about a smoker.
pub fn _asks_about_a_smoker(reading: &Understood) -> bool {
    let said = reading.text.to_lowercase();
    let words: HashSet<String> = LETTERS.findall_str(&said).into_iter().collect();
    if !words.contains("smoked") {
        return true;
    }
    if _SMOKING_WORDS.iter().any(|w| w != "smoked" && words.contains(w)) {
        return true;
    }
    any_in(&said, &_SMOKING_PHRASES)
}

/// How to smoke the food the line names.
pub fn _smoke_food(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let words: HashSet<String> = lower_words(reading).into_iter().collect();
    for thing in _SMOKE_THINGS.iter() {
        if words.contains(thing) {
            let key = match thing.as_str() {
                "rib" => "ribs",
                "pulled pork" | "pork" | "shoulder" => {
                    if words.contains("rib") {
                        "ribs"
                    } else {
                        "default"
                    }
                }
                "turkey" | "wings" => "chicken",
                "trout" | "mackerel" | "haddock" | "fish" => "salmon",
                "sausage" | "sausages" | "tofu" | "beef" | "steak" | "lamb" | "duck" => "default",
                other => other,
            };
            return Some(_SMOKE_REPLIES[key].clone());
        }
    }
    Some(_SMOKE_REPLIES["default"].clone())
}

/// `_not_short_of(*foods).<locals>.guard`: stand the row down when the line says the house has
/// none of these.
fn _not_short_of_guard(f: &FnRef, understood: &Understood) -> bool {
    let pattern = f.cell("pattern").and_then(Value::as_re).expect("captured pattern");
    !pattern.is_match(&understood.text.to_lowercase())
}

/// Which kind of cheese the line names, or "" when it names none.
pub fn _the_kind_of_cheese_named(said: &str) -> &'static str {
    for (kind, names) in [("blue", &*BLUE_CHEESES), ("soft", &*SOFT_CHEESES)] {
        if names.iter().any(|n| said.contains(&format!(" {n}"))) {
            return kind;
        }
    }
    if CHEESE_NAMES.iter().any(|n| said.contains(&format!(" {n}"))) {
        return "hard";
    }
    ""
}

/// True unless the line asks what to put beside something.
pub fn _not_a_serving_frame(reading: &Understood) -> bool {
    let text = dup_or(&reading.clean, &reading.text).to_lowercase();
    !dup::asks_for_a_side_of_a_food(&text) // DUP(bot.knowledge.dishes::asks_for_a_side_of_a_food)
}

/// The mould verdict when the line is about the wedge going wrong, storage when it is not.
pub fn _cheese_keep(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = format!(" {} ", reading.clean.to_lowercase());
    let stripped = text::strip(&said);
    if _CHEESE_TROUBLE_WORDS.iter().any(|w| said.contains(&format!(" {w} ")) || stripped.starts_with(w.as_str())) {
        return Some("Cut the mould off hard cheese with a good inch to spare and it is fine to eat; bin soft cheese, cream cheese and anything grated that has mould on it. To keep the rest, wrap it in wax paper or baking paper and then loosely in a bag, and keep it in the warmest part of the fridge: cling film makes it sweat and go slimy.".into());
    }
    let kind = _the_kind_of_cheese_named(&said);
    if !kind.is_empty() {
        return Some(_CHEESE_KIND_KEEP[kind].clone());
    }
    Some("To keep cheese, wrap it in wax paper or baking paper and then loosely in a bag, in the warmest part of the fridge. A hard wedge keeps about two weeks once opened. Soft cheese keeps about a week once opened, and cling film sweats it and makes it slimy, so leave it in the tub it came in.".into())
}

/// The recipe, or the health answer when the health is what was asked.
pub fn _overnight_oats(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = reading.clean.to_lowercase();
    if any_in(&said, &_OATS_HEALTH) {
        return Some("As healthy as what you put in the jar: oats are a slow-carb breakfast and the milk or yoghurt adds protein, so a portion with fruit keeps you full till lunch. The sugar comes from the honey, syrup and dried fruit, so go easy on those - and a shop-bought pot can carry a lot of it. Make it yourself and it is one of the better breakfasts going.".into());
    }
    Some(s("_OVERNIGHT_OATS_RECIPE"))
}

/// Whether the last turns or the plan are about something cooked in a pan (eggs, batter...).
pub fn _a_thread_of_a_pan(memory: &Memory) -> bool {
    let mut words: Vec<String> = Vec::new();
    let topics = dup::mem(memory, "topics").map(|v| v.items()).unwrap_or(&[]);
    let start = topics.len().saturating_sub(2);
    for turn in &topics[start..] {
        if !matches!(turn, Value::Dict(_)) {
            continue;
        }
        if let Some(ws) = turn.get("words") {
            words.extend(ws.items().iter().map(dup::py_str));
        }
        let t = turn.get("text").filter(|v| v.truthy()).map(dup::py_str).unwrap_or_default();
        words.extend(text::split(&t));
    }
    if let Some(plan) = dup::user_facts(memory).and_then(|u| u.get("plan")).and_then(Value::as_str) {
        words.extend(text::split(plan));
    }
    let low = format!(" {} ", words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" "));
    _TEA_MILK_PAN.iter().any(|dish| low.contains(&format!(" {dish} ")))
}

/// Milk in coffee: the split, the amount, or the kind.
pub fn _the_milk_for_a_coffee(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = format!(" {} ", understood.clean.to_lowercase());
    if ["curdl", " split ", " split,", " splits ", " splitting ", " separated ", " separates ", " gone lumpy ", " lumps in "]
        .iter()
        .any(|w| t.contains(w))
    {
        return Some(s("MILK_FOR_COFFEE_SPLIT"));
    }
    if ["how much", "how many ml", "how many mls", "how milky", "much milk", "much of it"].iter().any(|w| t.contains(w)) {
        return Some(s("MILK_FOR_COFFEE_AMOUNT"));
    }
    Some(s("MILK_FOR_COFFEE_KIND"))
}

/// Milk in tea: the order, the amount, the kind or the plant milk the line asks about.
pub fn _tea_milk_reply(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = format!(" {} ", understood.clean.to_lowercase());
    if !_TEA_MILK_BREW.iter().any(|w| t.contains(&format!(" {w} "))) && _a_thread_of_a_pan(memory) {
        return None;
    }
    let amount = any_in(&t, &TEA_MILK_AMOUNT_ASKS);
    let kind = any_in(&t, &TEA_MILK_KIND_ASKS);
    let plant = TEA_MILK_PLANT_SAID.iter().any(|w| t.contains(&format!(" {w} ")));
    if amount {
        return Some(format!("{} {}", s("TEA_MILK_AMOUNT"), if plant { s("TEA_MILK_PLANT") } else { s("TEA_MILK_KIND") }));
    }
    if kind {
        return Some(if plant { format!("{} {}", s("TEA_MILK_KIND"), s("TEA_MILK_PLANT")) } else { s("TEA_MILK_KIND") });
    }
    if plant {
        return Some(s("TEA_MILK_PLANT"));
    }
    Some(s("TEA_MILK_ORDER"))
}

/// Whether a word after "without" is an outcome ("without breaking"), not a thing left out.
pub fn _an_outcome(word: &str) -> bool {
    if _A_ITS_OWN_FAULT.iter().any(|w| w == word) {
        return false;
    }
    word.ends_with("ing") && !_A_SUPPLY_VERB.iter().any(|w| w == word)
}

/// Whether "without" in the line leaves a thing out (a swap), not a step.
pub fn _a_swap_and_not_a_step(understood: &Understood) -> bool {
    let cleaned = WORDS_ONLY.sub(" ", &understood.clean.to_lowercase());
    let words = text::split(&cleaned);
    let mut seen = false;
    for (index, word) in words.iter().enumerate() {
        if word != "without" {
            continue;
        }
        seen = true;
        let after = &words[(index + 1).min(words.len())..(index + 4).min(words.len())];
        if after.is_empty() {
            continue;
        }
        if _A_WITHOUT_SUBJECT.iter().any(|w| *w == after[0]) {
            if after[1..].iter().any(|w| _an_outcome(w)) {
                continue;
            }
            return true;
        }
        if _an_outcome(&after[0]) {
            continue;
        }
        return true;
    }
    !seen
}

/// The words the line says it is without ("without eggs or butter").
pub fn _said_to_be_missing(clean: &str) -> HashSet<String> {
    let words = text::split(&WORDS_ONLY.sub(" ", &clean.to_lowercase()));
    let mut missing = HashSet::new();
    for (index, word) in words.iter().enumerate() {
        if word == "without" {
            let end = (index + 4).min(words.len());
            missing.extend(words[index + 1..end].iter().cloned());
        }
    }
    missing
}

/// `_leaves_out(*ingredients).<locals>.guard`: the row stands down when the line goes without
/// the ingredient it needs.
fn _leaves_out_guard(f: &FnRef, understood: &Understood) -> bool {
    let stems = f.cell("stems").map(|v| v.str_set()).unwrap_or_default();
    let said = _said_to_be_missing(&understood.clean);
    !said.iter().any(|w| stems.contains(w))
}

/// True unless the "season" in the line is a series of a show.
pub fn _not_a_season_of_a_show(understood: &Understood) -> bool {
    let said = format!(" {} ", dup::clean_norm_text_norm(understood).to_lowercase());
    !_A_SEASON_OF_A_SHOW.is_match(&said)
}

/// Keeping bread: past its date, how long it lasts, or how to keep it.
pub fn _bread_kept(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = dup::clean_norm_text_norm(understood).to_lowercase();
    if any_in(&said, &PACK_DATE) || said.contains("the date") {
        return Some(s("BREAD_DATED"));
    }
    if BREAD_HOW_LONG.is_match(&said) {
        return Some(s("BREAD_LIFE"));
    }
    Some(s("BREAD_KEEP"))
}

/// The lunchbox items none of whose words are ruled out.
pub fn _a_lunchbox_without(items: &[(String, String)], banned: &HashSet<String>) -> Vec<String> {
    items
        .iter()
        .filter(|(_, words)| !text::split_ref(words).iter().any(|w| banned.contains(*w)))
        .map(|(food, _)| food.clone())
        .collect()
}

/// Whether the lunch is a child's: the line or the person talked about says so.
pub fn _lunch_is_for_a_child(memory: &Memory, said: &str) -> bool {
    if text::split_ref(said).iter().any(|w| CHILD_NAMES.contains(*w)) {
        return true;
    }
    if any_in(said, &CHILDS_LUNCH_WORDS) {
        return true;
    }
    if let Some(person @ Value::Dict(_)) = dup::mem(memory, "person") {
        let who = person.get("who").filter(|v| v.truthy()).map(dup::py_str).unwrap_or_default();
        if CHILD_NAMES.contains(&text::strip(&who).to_lowercase()) {
            return true;
        }
    }
    if let Some(user) = dup::user_facts(memory) {
        for one in user.get("people").map(|v| v.items()).unwrap_or(&[]) {
            if CHILD_NAMES.contains(&text::strip(&dup::py_str(one)).to_lowercase()) {
                return true;
            }
        }
    }
    false
}

/// Whether one of the last four lines said no sandwich.
pub fn _said_no_sandwich(memory: &Memory) -> bool {
    let start = memory.turns.len().saturating_sub(4);
    memory.turns[start..].iter().any(|t| any_in(&t.user.to_lowercase(), &NO_SANDWICH))
}

/// The lunch ideas, unless they offer a sandwich to someone who said no to one.
pub fn _lunch_ideas_after_a_no(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let reply = crate::knowledge::cooking::_lunch_ideas(memory, understood).unwrap_or_default();
    let low = reply.to_lowercase();
    let offers_one = low.contains("sandwich") && !low.contains("not a sandwich");
    if offers_one && _said_no_sandwich(memory) {
        return _no_sandwich_lunch(memory, understood);
    }
    Some(reply)
}

/// A packed lunch with no sandwich in it, for a child or an adult, less what is ruled out.
pub fn _no_sandwich_lunch(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = understood.words.join(" ").to_lowercase();
    let kid = _lunch_is_for_a_child(memory, &said)
        || ["year old", "nursery", "school", "fussy", "picky"].iter().any(|w| said.contains(w));
    let user = dup::user_facts(memory);
    let mut banned = dup::_bans_of(user); // cooking::_bans_of (re-exported by d2dup_g6)
    banned.extend(dup::avoids_for_the_one_talked_about(memory));
    if !banned.is_empty() {
        for said_allergy in user.and_then(|u| u.get("allergies")).map(|v| v.items()).unwrap_or(&[]) {
            for word in text::split(&dup::py_str(said_allergy).to_lowercase().replace(',', " ")) {
                if let Some(tag) = _SNACK_FAMILY_OF.get(&word) {
                    if !tag.is_empty() {
                        banned.insert(tag.clone());
                    }
                }
            }
        }
        let kept = _a_lunchbox_without(if kid { &_NO_SANDWICH_KID_ITEMS } else { &_NO_SANDWICH_ADULT_ITEMS }, &banned);
        if kept.is_empty() {
            let who = if kid { "he" } else { "you" };
            return Some(format!(
                "With that much ruled out there is nothing on the usual list to name: build the lunch from what {who} already eats at home, a tub of dinner kept cold and a drink, and check the label on every packet you use. {}",
                if kid { "A flask of soup or leftovers does the same job and is often the easiest with a fussy eater." } else { "" }
            ));
        }
        let listed = if kept.len() > 1 {
            format!("{} and {}", kept[..kept.len() - 1].join(", "), kept[kept.len() - 1])
        } else {
            kept[0].clone()
        };
        let how_many = if kept.len() >= 3 {
            "any three of those and a drink is a lunch"
        } else if kept.len() == 2 {
            "both of those and a drink is a lunch"
        } else {
            "that and a drink is a lunch"
        };
        if kid {
            return Some(format!(
                "For a lunchbox with no sandwich: {listed} - {how_many}. A flask of soup or leftovers does the same job and is often the easiest with a fussy eater, because it is food they already eat at home."
            ));
        }
        return Some(format!(
            "For a packed lunch with no sandwich: {listed} travel{} well. {}, and the tub of dinner is the one that keeps you full until tea.",
            if kept.len() == 1 { "s" } else { "" },
            text::capitalize(how_many)
        ));
    }
    if kid {
        return Some("For a lunchbox with no sandwich: cooked pasta or rice from last night's dinner in a tub, cheese cubes or a boiled egg with crackers or oatcakes, veg sticks with hummus, a yoghurt and some fruit - any three of those and a drink is a lunch. A flask of soup or leftovers does the same job and is often the easiest with a fussy eater, because it is food they already eat at home.".into());
    }
    Some("For a packed lunch with no sandwich: last night's dinner in a tub is the easiest, and pasta or rice salad, cold chicken or cheese with oatcakes, a boiled egg, veg sticks and hummus, soup in a flask, yoghurt and fruit all travel well. Any three of those plus a drink is a lunch, and the tub of dinner is the one that keeps you full until tea.".into())
}

/// Coffee late in the day: when to stop, whether this one is too late, or why it keeps them up.
pub fn _coffee_late(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let clean = reading.clean.to_lowercase();
    if COFFEE_STOP_ASK.is_match(&clean) {
        return Some("Stop by early afternoon, and count back: caffeine's half-life is about five hours, so half a 4pm cup is still in you at 9pm and a quarter of it at 2am. If you are in bed at eleven, a last full cup around 2pm leaves almost nothing by then; after that make it decaf, which has about 2mg a cup against 80.".into());
    }
    let late = [
        "late", "tonight", "this evening", "in the evening", "this afternoon", "in the afternoon", "before bed",
        "bedtime", "pm", "at night", "night",
    ];
    if late.iter().any(|w| clean.contains(w)) {
        if COFFEE_MAY_I.is_match(&clean) {
            return Some("No, not if you want to sleep: half the caffeine is still there five or six hours later, so a 4pm cup can still be working at midnight. Keep to one or two cups a day, stop by early afternoon, make the afternoon one decaf, and eat something with it. Palpitations, shakiness or panic that come with caffeine are worth mentioning to a GP, especially with chest pain or breathlessness.".into());
        }
        return Some("Yes, too late if you want to sleep at a normal hour: half the caffeine is still there five or six hours later, so a 4pm coffee can still be working at midnight. Keep to one or two cups a day, stop by early afternoon, make the afternoon one decaf, and eat something with it. Palpitations, shakiness or panic that come with caffeine are worth mentioning to a GP, especially with chest pain or breathlessness.".into());
    }
    Some("It is the caffeine: it peaks about half an hour after the cup and takes five or six hours to leave your system, so a 4pm coffee can still be in you at midnight. Keep to one or two cups, stop by early afternoon, make the afternoon one decaf, and eat something with it. Palpitations, shakiness or panic that come with caffeine are worth mentioning to a GP, especially with chest pain or breathlessness.".into())
}

/// True unless the line asks what to make or cook.
pub fn _not_a_make_it_ask(understood: &Understood) -> bool {
    !_MAKE_IT_ASK.is_match(&understood.clean.to_lowercase())
}

/// True unless the line asks how long something takes to cook (a keeping line is not that).
pub fn _not_a_cooking_time(understood: &Understood) -> bool {
    if !_not_a_make_it_ask(understood) {
        return false;
    }
    let words = lower_words(understood);
    if words.is_empty() {
        return true;
    }
    let joined = words.join(" ");
    if !_COOKING_CLOCK.is_match(&joined) {
        return true;
    }
    _A_KEPT_THING.is_match(&joined)
}

/// Keeping milk: formula, breast milk, or a carton.
pub fn _milk_keep(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", dup::clean_norm_text_norm(understood).to_lowercase());
    if said.contains("formula") || said.contains("made up") || said.contains("made-up") {
        return Some("Made-up formula is good for 2 hours at room temperature and 24 hours in the fridge, and the clock starts when the powder goes in - throw away whatever is left in the bottle after a feed.".into());
    }
    if any_in(&said, &_BREAST_MILK) {
        return Some("Expressed breast milk keeps about 4 hours at room temperature, up to 4 days at the back of the fridge, and 6 months in the freezer; once thawed it is good for 24 hours in the fridge and must not be refrozen.".into());
    }
    let keeping = "Once opened, keep milk in the fridge and use it within about 5 days; the date on the carton is about quality, not safety, so milk that has been kept cold is fine a day or two past it.";
    let spoilt = "If milk smells or tastes sour, bin it, whatever the date says.";
    if said.contains("fresh") || any_in(&said, &STORE_ASKS) {
        return Some(format!("{keeping} {spoilt}"));
    }
    Some(format!("{spoilt} {keeping}"))
}

/// True unless the line asks how much of a drink is too much (a dose question).
pub fn not_a_cup_of_tea(understood: &Understood) -> bool {
    let words = lower_words(understood);
    if words.is_empty() {
        return true;
    }
    let joined = format!(" {} ", words.join(" "));
    let has = |list: &[String]| list.iter().any(|w| joined.contains(&format!(" {w} ")));
    if !has(&_DRINK_NAMES) {
        return true;
    }
    if has(&_COOKING_WORDS) {
        return true;
    }
    if !has(&_ASK_WORDS) {
        return true;
    }
    if has(&_DOSE_WORDS) {
        return false;
    }
    true
}

/// An afternoon snack that fits what the person cannot eat.
pub fn _a_snack(memory: &mut Memory, _understood: &Understood) -> Option<String> {
    let user = dup::user_facts(memory);
    let mut banned = dup::_bans_of(user); // cooking::_bans_of (re-exported by d2dup_g6)
    banned.extend(dup::avoids_for_the_one_talked_about(memory));
    for said in user.and_then(|u| u.get("allergies")).map(|v| v.items()).unwrap_or(&[]) {
        for word in text::split(&dup::py_str(said).to_lowercase().replace(',', " ")) {
            if let Some(tag) = _SNACK_FAMILY_OF.get(&word) {
                if !tag.is_empty() {
                    banned.insert(tag.clone());
                }
            }
        }
    }
    let mut chosen = _a_lunchbox_without(&_SNACK_OPTIONS, &banned);
    if chosen.is_empty() {
        chosen = vec!["whatever protein and fibre you can eat safely".to_string()];
    }
    let mut listed = chosen[..chosen.len().min(2)].join(", ");
    if chosen.len() >= 3 {
        listed.push_str(" or ");
        listed.push_str(&chosen[2]);
    }
    Some(format!(
        "Go for protein plus fibre at about four in the afternoon, 150 to 250 calories: {listed}. It holds you a couple of hours where a biscuit holds you twenty minutes, so you are not starving at dinner."
    ))
}

/// How to make the coffee the line names, with the milk tail.
pub fn _a_coffee(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = dup_or(&understood.norm_text, &understood.text).to_lowercase();
    let tail = s("_COFFEE_MILK_TAIL");
    for (n, lead) in _COFFEE_LEADS.iter() {
        if said.contains(n.as_str()) {
            return Some(format!("{lead} {tail}"));
        }
    }
    if any_in(&said, &_THE_MILK_ONLY) {
        return Some(s("_THE_FROTH_CARD"));
    }
    Some(format!("A flat white is a double espresso (about 60ml) with about 100ml of milk heated to 60-65C and frothed only to a thin, glossy microfoam, poured with the jug low so the foam sits in a thin layer rather than a cap on top, while a latte takes twice the milk and less foam, and a cappuccino more air again. {tail}"))
}

/// How long tins keep.
pub fn _tins_keep(_memory: &mut Memory, _understood: &Understood) -> Option<String> {
    Some("Tins keep for years in a cool cupboard, though anything acidic - tomatoes, fruit, pickles - is best inside 18 months, and a dented or rusty tin is worth using soon. The date on a tin is a best-before for quality rather than a use-by. Bin any tin that is bulging, smells sharp when you open it, or fizzes. Once it is open, what is left goes into a bowl with a lid, not back in the tin: 2 days in the fridge for tuna or beans, and eat anything acidic within a couple of days.".into())
}

/// A tomato sauce, or how long to simmer it when the line asks the time.
pub fn _tomato_sauce(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", dup_or(&understood.clean, &understood.text)).to_lowercase();
    if ["how long", "simmer", "minutes", "how many minutes", "time", "takes", "take"].iter().any(|w| said.contains(w)) {
        return Some("Twenty to thirty minutes on a low simmer, and it is better at forty: soften the garlic in olive oil, add a tin of tomatoes and a pinch of salt, and let it go uncovered so it thickens. Tear in basil at the end and loosen it with a splash of pasta water. A long simmer is what turns tinned tomatoes from sharp into sweet, so if it tastes sour give it another ten minutes rather than more sugar.".into());
    }
    Some("A good 15-minute tomato sauce: soften sliced garlic in olive oil, add a tin of tomatoes and a pinch of salt, and simmer it while the pasta cooks. Tear in a few basil leaves at the end and loosen it with pasta water.".into())
}

/// The freezing answer for a cooked dish (kept for callers outside the entries).
pub fn freeze_cooked_answer(memory: &mut Memory, understood: &Understood) -> Option<String> {
    _freeze_cooked(memory, understood)
}

/// How long the cooked dish the line names keeps frozen, and how to freeze it.
pub fn _freeze_cooked(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", lower_words(understood).join(" "));
    let step = s("FREEZE_COOKED_STEP");
    for (names, keeps) in FREEZE_DISH_TIMES.iter() {
        if names.iter().any(|n| said.contains(&format!(" {n} ")) || said.contains(&format!(" {n}s "))) {
            return Some(format!(
                "Yes - it keeps {keeps} in the freezer, so cool it within an hour, then pack it flat in bags or tubs so it freezes fast. {step}"
            ));
        }
    }
    Some(format!("Yes - cooked dishes keep about three months frozen, and a month for mash, so cool it within an hour, then pack it flat in bags or tubs so it freezes fast. {step}"))
}

/// Whether the line says the thing being set down is hot.
pub fn _a_hot_thing_is_being_set_down(understood: &Understood) -> bool {
    let mut said = dup_or(&understood.text, &understood.clean).to_string();
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    _HOT_SAID.is_match(&said)
}

/// A loaf or a cake stuck in its tin.
pub fn _out_of_the_tin(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    if set_hits(&words_set(reading), &_LOAF) {
        return Some("A loaf sticks to the tin when the tin was bare or the dough was wet: oil the tin all over, and line the base and the two long sides with a strip of baking paper that overhangs the top, so you lift the loaf out by the paper. Run a knife round the edges while it is still warm, wait ten minutes, then turn it out and tap the base - and let it cool on its side so the crust does not go soft.".into());
    }
    Some("A cake sticks to the tin when it was greased unevenly, turned out before it had firmed, or baked in a tin that was not lined. Line it with a strip of baking paper across the base and up two sides and it lifts out whole. Run a thin knife round the edge while it is still warm, wait ten minutes, then turn it out onto a rack - and if it does crack, stick it back together with a little jam or buttercream and nobody will know.".into())
}

/// A batter that sticks: in a waffle iron, a tin, or a pan.
pub fn _a_batter_that_sticks(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let words = words_set(reading);
    if set_hits(&words, &_BATTER_IN_AN_IRON) {
        return Some("A waffle iron that grips wants to be properly hot before the batter goes in, and brushed with a little oil right into the grooves: heat it five minutes, oil it, and fill to the edge of the pattern only. Leave the lid shut for a minute and let the waffle come away by itself - one pulled open early tears.".into());
    }
    if set_hits(&words, &_BATTER_IN_A_TIN) {
        return Some("A batter that sticks to the tin wants the fat hotter than the batter: put a little oil or dripping in each hole and heat the tin in the oven at 220C (200C fan) for 10 minutes, until it shimmers, then pour the batter in and shut the door. Do not open it for 20 minutes - the cold air drops them - and if one still sticks, give it 5 minutes more, because a batter that comes away rough was not done.".into());
    }
    Some("A batter that sticks wants a hotter pan and less fat than you would think: heat a non-stick or well-seasoned pan on a medium heat for a couple of minutes, wipe it with a little oil on kitchen paper, and pour when a drop of batter sizzles at once. Wait until the top bubbles and the edges look dry before you turn it, because lifting one early is what tears it, and thin a thick batter with a splash of milk, or it sits in the middle of the pan and grips.".into())
}

/// Whether the line says the pizza is frozen.
pub fn _a_frozen_pizza(understood: &Understood) -> bool {
    understood.words.iter().any(|w| _FROZEN_SAID.contains(w))
}

/// Slicing bread: stale, thickness, warm, or how; None for a count of slices.
pub fn _bread_sliced(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", dup_or(&understood.clean, &understood.text).to_lowercase());
    if _A_PORTION_OF_SLICES.is_match(&said) && !_STICKING_A_CUT.is_match(&said) {
        return None;
    }
    if [" stale", " stales", " staling", " hard", " dry", " drying", " why does", " why do", " goes off", " gone hard"]
        .iter()
        .any(|w| said.contains(w))
    {
        return Some(s("STALE_FAST"));
    }
    if [" thick", " thickness", " thin", " wide", " how wide", " size of"].iter().any(|w| said.contains(w)) {
        return Some(s("SLICE_THICK"));
    }
    if [" warm", " hot", " just baked", " fresh out", " still warm", " straight out"].iter().any(|w| said.contains(w)) {
        return Some(s("SLICE_WARM"));
    }
    Some(s("SLICE_METHOD"))
}

/// True when the line names no ingredient the meal planner knows (other than "veg").
pub fn _no_named_food(understood: &Understood) -> bool {
    !understood.words.iter().any(|w| INGREDIENT_FORMS.contains(w))
}

/// Porridge gone wrong: too salty, bland, an overnight jar, or too sweet.
pub fn _porridge_fault(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = dup_or(&understood.clean, &understood.text).to_lowercase();
    if any_in(&said, &_PORRIDGE_SALT) {
        return Some("A bowl of porridge with too much salt in it takes more milk and more oats, not water: stir in a good splash of milk and a spoon more oats and give it two minutes on the heat, then a spoon of plain yoghurt or a little honey at the end. Water alone just leaves you with a thin bowl that is still salty.".into());
    }
    if any_in(&said, &_PORRIDGE_BLAND) {
        return Some("Porridge is bland for want of salt and something on top, not for want of skill: put a pinch of salt in the pan, cook it in milk rather than water, and finish it with honey, jam, brown sugar, banana, berries or a spoon of peanut butter. Salt is the one that matters - without it everything else tastes flat.".into());
    }
    if said.contains("overnight") || said.contains("soak") || said.contains("soaked") || said.contains("in a jar") {
        return Some("Stir a pinch of salt and a splash more milk or yoghurt into the jar, because salt and milk are what turn the sweetness down, and leave it ten minutes to mellow. If it is still too sweet, add a spoon more oats and a little milk rather than more sugar in future: the sweetness sits on top of soaked oats rather than in them.".into());
    }
    Some("Stir in a splash more milk or plain yoghurt and a pinch of salt, because salt is what turns the sweetness down, and leave it a minute to loosen. If it is still too sweet, a handful of berries or a spoon of yoghurt does more than adding more oats. Next time, leave the sugar out of the pan or the jar and put it on the table: porridge takes very little, and the milk brings out the sweetness of the oats.".into())
}

/// The surface the line would put a hot pan on: ("no", name, reason), ("yes", name, "") or none.
pub fn _a_hot_pan_surface(t: &str) -> (Option<&'static str>, Option<String>, String) {
    let said = format!(" {} ", t.to_lowercase());
    for (n, pattern, reason) in _HOT_PAN_REFUSES.iter() {
        if pattern.is_match(&said) {
            return (Some("no"), Some(n.clone()), reason.clone());
        }
    }
    for (n, pattern) in _HOT_PAN_TAKES.iter() {
        if pattern.is_match(&said) {
            return (Some("yes"), Some(n.clone()), String::new());
        }
    }
    (None, None, String::new())
}

/// Where a hot pan can go down: the surface the line names, or the general rule.
pub fn _the_hot_pan_place(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let (verdict, surface, reason) = _a_hot_pan_surface(&dup::_line(understood));
    let (can, not, tip) = (s("_HOT_PAN_CAN"), s("_HOT_PAN_NOT"), s("_HOT_PAN_TIP"));
    match verdict {
        Some("no") => Some(format!("No - not on {}: {reason}. {can} {tip}", surface.unwrap_or_default())),
        Some("yes") => Some(format!("Yes - {} takes it straight off the ring. {not} {tip}", surface.unwrap_or_default())),
        _ => Some(format!("{can} {not} {tip}")),
    }
}

/// Whether the line names a dish to bake (so "what temperature" is the oven's, not a fever).
pub fn something_to_bake_in_the_oven(understood: &Understood) -> bool {
    let t = dup_or(&understood.clean, &understood.text);
    if t.is_empty() {
        return false;
    }
    dup::dish_in(t).is_some_and(|d| !d.is_empty())
}

/// True unless the line asks how long a roast joint takes (that is the joint-timing row's).
pub fn _not_the_joint_timing(understood: &Understood) -> bool {
    let said = format!(" {} ", dup::clean_norm_text_norm(understood).to_lowercase());
    if !_JOINT_WORDS.iter().any(|rx| rx.is_match(&said)) {
        return true;
    }
    if _A_FAT_WORD.is_match(&said) {
        return true;
    }
    !_A_JOINT_TIMING.is_match(&said)
}

/// Roast potatoes: stuck, hard, burnt, or the method.
pub fn _roast_potatoes(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let clean = reading.clean.to_lowercase();
    if any_in(&clean, &_ROAST_STUCK) {
        return Some("Sticking is the tray and the wet, not the heat: put the oiled tray in to get hot before the potatoes go on, dry them on a tea towel once they are drained, and keep them in one layer with space between them so the steam can get out. Turn them at 25 minutes with a fish slice rather than a spoon, and whatever is already browned comes away on its own.".into());
    }
    if any_in(&clean, &_ROAST_HARD) {
        return Some("Hard in the middle is a size and time problem, not a heat problem: cut them no bigger than a golf ball, parboil them a full 8 minutes so the middle starts to cook before the outside crisps, and give the tray 10 to 15 minutes more at 200C (180C fan) in one layer. Test the biggest one with the point of a knife: it should slide in with no resistance at all.".into());
    }
    if any_in(&clean, &_ROAST_BURNT) {
        return Some("Too hot, or the potatoes went in wet: turn the oven down to 190C (170C fan), spread them in one layer with space between them and turn them at 25 minutes. Dry them on a tea towel after the parboil and shake them rough, because a wet potato steams and then blackens at the edges before the middle is done.".into());
    }
    Some("Parboil peeled potatoes for 8 minutes, drain them, shake the pan to rough up the edges, then roast at 200C (180C fan) in a thin layer of hot oil for 40 to 45 minutes. The rough edges are what go crunchy. If they came out of a freezer bag, none of that applies: 200C (180C fan) for 20 to 25 minutes straight from frozen, in one layer.".into())
}

/// Keeping cooked food in a state: not sticking, held hot, juicy, or crisp.
pub fn _keep_how(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let mut words = words_set(reading);
    if !set_hits(&words, &KEEP_QUALITIES) || !set_hits(&words, &_KEEP_FOOD) {
        return None;
    }
    if set_hits(&words, &_KEEP_STICK_WORDS) {
        if words.contains("rice") {
            return None;
        }
        if !set_hits(&words, &_KEEP_BOILED) {
            return None;
        }
        return Some("Stir it once as it goes in and once more a minute later, keep the water at a rolling boil with room for the pieces to move, and lift it out with tongs or a spider rather than tipping it into a colander. Oil in the water does nothing - it floats - but a splash of oil through it after draining stops it setting into one lump.".into());
    }
    if set_hits(&words, &_KEEP_HOT_WORDS) {
        if words.contains("garlic") && words.contains("bread") {
            words.insert("garlic bread".to_string());
        }
        if words.contains("rice") {
            return Some("Lid on and off the heat, with a folded tea towel over the pan under the lid: rice holds warm that way for 20 to 30 minutes, and an hour or two in a covered dish in an oven on 80C with a splash of water is the limit. That limit is not fussiness - cooked rice left standing warm grows bacillus cereus, so past two hours cool it fast and reheat it once, thoroughly.".into());
        }
        let held = "Hold it hot rather than hotting it up later: an oven on its lowest, a slow cooker on low, or a bowl over a pan of simmering water, all covered, which keeps food above 63C where bacteria stop multiplying. ";
        if set_hits(&words, &_KEEP_GRAIN_WORDS) {
            return Some(format!("{held}Keep back a cup of the cooking water and stir it through so nothing sets into a block, and do not leave it warm for more than an hour or two."));
        }
        if set_hits(&words, &_KEEP_POUR_WORDS) {
            return Some("Stand the jug in a pan of simmering water on the lowest ring, or pour it into a flask, and give it a stir every so often: a sauce left on the hob catches, and one left standing still grows a skin. Loose it with a splash of stock or milk before it goes out, and an hour or two is as long as it holds.".into());
        }
        if set_hits(&words, &_KEEP_COVERED_WORDS) {
            return Some("A covered dish in the oven on its lowest, with a splash of stock or water in the bottom so the edges do not dry: in one layer rather than piled up, and uncovered if it is meant to stay crusty. An hour or two is the limit - after that it is a leftover, and a leftover is cooled fast and reheated once.".into());
        }
        if set_hits(&words, &_KEEP_FLATBREAD_WORDS) {
            return Some("Keep them wrapped and close together - a clean tea towel or foil - in the low oven, or give each one a minute a side in a warm dry pan. A lid or a covered dish steams a flatbread soft, and it dries out in the open, so ten to fifteen minutes in the cloth is all it needs.".into());
        }
        if set_hits(&words, &_KEEP_STACK_WORDS) {
            return Some(format!("{held}Stack them on a plate under a clean tea towel, rather than under a lid or foil, which traps the steam and leaves them limp, and do not hold them warm for more than an hour or two."));
        }
        if set_hits(&words, &_KEEP_CRISP_HELD_WORDS) {
            return Some(format!("{held}One layer on a tray, uncovered, not piled in a bowl where they steam and go soft, and do not leave them warm for more than an hour or two."));
        }
        return Some(format!("{held}Cover it and do not leave it warm for more than an hour or two."));
    }
    if set_hits(&words, &_KEEP_SOFT_WORDS) {
        if !set_hits(&words, &_KEEP_MEAT) {
            return None;
        }
        return Some("Juiciness is decided before the pan and lost at the end: salt the meat 40 minutes ahead or soak it in brine, cook it to 74C in the middle for chicken and pork (54C for beef you want pink), and rest it under loose foil for 10 minutes so the juices settle back into the fibres. Then slice across the grain, not along it.".into());
    }
    Some("Crisp survives only in the open: cool anything fried or roasted in one layer on a rack or a paper-lined tray, never in a pile or a closed tub, where the steam turns it soft within minutes. Then reheat it 4 to 5 minutes at 200C on a bare tray, not in a microwave. A dry surface before it goes in the fat is the other half of it.".into())
}

/// Rice pudding in a pressure cooker, a microwave, or a slow cooker.
pub fn _rice_pudding_pot(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = if reading.clean.is_empty() { reading.words.join(" ") } else { reading.clean.clone() }.to_lowercase();
    if ["pressure cooker", "instant pot", "multicooker"].iter().any(|w| said.contains(w)) {
        return Some(s("_RICE_PUDDING_PRESSURE"));
    }
    if ["microwave", "microwaved", "microwaving"].iter().any(|w| said.contains(w)) {
        return Some(s("_RICE_PUDDING_MICROWAVE"));
    }
    Some(s("_RICE_PUDDING_SLOW"))
}

/// What to cook with the kitchen kit the line has left, unless it is deciding about a repair or
/// the last reply already gave the no-kit answer.
pub fn _no_appliance(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let gone = dup::missing(understood); // cooking::missing (re-exported by d2dup_g6)
    if gone.is_empty() {
        return None;
    }
    let said = dup::_line(understood);
    if DECIDING.iter().any(|rx| rx.is_match(&said)) {
        return None;
    }
    let before = dup::last_bot(memory).to_lowercase();
    if [
        "you cannot bake without an oven",
        "you can still eat well",
        "you can still eat properly",
        "a kettle still makes",
        "with an oven and no hob",
    ]
    .iter()
    .any(|came| before.contains(came))
    {
        return None;
    }
    dup::without(understood) // cooking::without (re-exported by d2dup_g6)
}

/// Smoked fish: already cooked, so heat it through (smoked salmon wants no cooking).
pub fn _smoked_fish_reply(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = reading.text.to_lowercase();
    if said.contains("smoked salmon") || said.contains("lox") {
        return Some(s("SMOKED_SALMON_ALREADY"));
    }
    Some(s("_SMOKED_FISH_REPLY").replace("{fish}", &_smoked_fish(reading)))
}

/// The smoked fish the line names, "haddock" when it names none.
pub fn _smoked_fish(reading: &Understood) -> String {
    let said = reading.text.to_lowercase();
    for n in _SMOKED_FISH.iter() {
        if said.contains(n.as_str()) {
            if matches!(n.as_str(), "kipper" | "kippers" | "smoked herring") {
                return "kipper".into();
            }
            return n.strip_prefix("smoked ").unwrap_or(n).to_string();
        }
    }
    "haddock".into()
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.cooking_entries:<lambda>:2918", |_: &FnRef, u: &Understood| {
        u.kept.is_none() && dup::a_cooling_question_not_a_keeping_one(u)
    }),
    ("bot.knowledge.cooking_entries:<lambda>:4870", |_: &FnRef, u: &Understood| {
        !dup::a_persons_temperature(u) && !something_to_bake_in_the_oven(u)
    }),
    ("bot.knowledge.cooking_entries:<lambda>:5428", |_: &FnRef, u: &Understood| !amends_to(line_of(u).unwrap_or(""))),
    ("bot.knowledge.cooking_entries:_a_frozen_pizza:1971", |_: &FnRef, u: &Understood| _a_frozen_pizza(u)),
    ("bot.knowledge.cooking_entries:_a_hot_thing_is_being_set_down:1885", |_: &FnRef, u: &Understood| {
        _a_hot_thing_is_being_set_down(u)
    }),
    ("bot.knowledge.cooking_entries:_a_swap_and_not_a_step:1143", |_: &FnRef, u: &Understood| _a_swap_and_not_a_step(u)),
    ("bot.knowledge.cooking_entries:_asks_about_a_smoker:736", |_: &FnRef, u: &Understood| _asks_about_a_smoker(u)),
    ("bot.knowledge.cooking_entries:_leaves_out.<locals>.guard:1214", _leaves_out_guard),
    ("bot.knowledge.cooking_entries:_no_named_food:2028", |_: &FnRef, u: &Understood| _no_named_food(u)),
    ("bot.knowledge.cooking_entries:_not_a_cooking_time:1551", |_: &FnRef, u: &Understood| _not_a_cooking_time(u)),
    ("bot.knowledge.cooking_entries:_not_a_season_of_a_show:1256", |_: &FnRef, u: &Understood| {
        _not_a_season_of_a_show(u)
    }),
    ("bot.knowledge.cooking_entries:_not_a_serving_frame:847", |_: &FnRef, u: &Understood| _not_a_serving_frame(u)),
    ("bot.knowledge.cooking_entries:_not_a_trip:328", |_: &FnRef, u: &Understood| _not_a_trip(u)),
    ("bot.knowledge.cooking_entries:_not_short_of.<locals>.guard:790", _not_short_of_guard),
    ("bot.knowledge.cooking_entries:_not_the_joint_timing:2195", |_: &FnRef, u: &Understood| _not_the_joint_timing(u)),
    ("bot.knowledge.cooking_entries:_one_persons_lunch:384", |_: &FnRef, u: &Understood| _one_persons_lunch(u)),
    ("bot.knowledge.cooking_entries:_the_rice_was_cooked:314", |_: &FnRef, u: &Understood| _the_rice_was_cooked(u)),
    ("bot.knowledge.cooking_entries:not_a_cup_of_tea:1650", |_: &FnRef, u: &Understood| not_a_cup_of_tea(u)),
];

macro_rules! r {
    ($id:literal, $f:path) => {
        ($id, |_: &FnRef, m: &mut Memory, u: &Understood| $f(m, u))
    };
}

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.cooking_entries:<lambda>:3994", |_: &FnRef, _: &mut Memory, u: &Understood| {
        Some(dup::_mushroom_wash(u))
    }),
    ("bot.knowledge.cooking_entries:<lambda>:4013", |_: &FnRef, _: &mut Memory, u: &Understood| {
        Some(dup::_poach_fish(u))
    }),
    ("bot.knowledge.cooking_entries:<lambda>:4036", |_: &FnRef, _: &mut Memory, u: &Understood| {
        Some(dup::_dip_fault(u))
    }),
    ("bot.knowledge.cooking_entries:<lambda>:5608", |_: &FnRef, _: &mut Memory, u: &Understood| {
        Some(dup::_jam_make(u))
    }),
    ("bot.knowledge.cooking_entries:<lambda>:7376", |_: &FnRef, _: &mut Memory, u: &Understood| {
        Some(dup::_jacket(u))
    }),
    r!("bot.knowledge.cooking_entries:_a_batter_that_sticks:1928", _a_batter_that_sticks),
    r!("bot.knowledge.cooking_entries:_a_coffee:1760", _a_coffee),
    r!("bot.knowledge.cooking_entries:_a_snack:1707", _a_snack),
    r!("bot.knowledge.cooking_entries:_bread_kept:1263", _bread_kept),
    r!("bot.knowledge.cooking_entries:_bread_sliced:2005", _bread_sliced),
    r!("bot.knowledge.cooking_entries:_cheese_keep:859", _cheese_keep),
    r!("bot.knowledge.cooking_entries:_coffee_late:1457", _coffee_late),
    r!("bot.knowledge.cooking_entries:_freeze_cooked:1854", _freeze_cooked),
    r!("bot.knowledge.cooking_entries:_keep_how:11451", _keep_how),
    r!("bot.knowledge.cooking_entries:_lunch_ideas_after_a_no:1389", _lunch_ideas_after_a_no),
    r!("bot.knowledge.cooking_entries:_milk_keep:1598", _milk_keep),
    r!("bot.knowledge.cooking_entries:_no_appliance:11950", _no_appliance),
    r!("bot.knowledge.cooking_entries:_no_sandwich_lunch:1407", _no_sandwich_lunch),
    r!("bot.knowledge.cooking_entries:_open_wine:463", _open_wine),
    r!("bot.knowledge.cooking_entries:_out_of_the_tin:1907", _out_of_the_tin),
    r!("bot.knowledge.cooking_entries:_overnight_oats:895", _overnight_oats),
    r!("bot.knowledge.cooking_entries:_porridge:511", _porridge),
    r!("bot.knowledge.cooking_entries:_porridge_fault:2049", _porridge_fault),
    r!("bot.knowledge.cooking_entries:_rice_pudding_pot:11751", _rice_pudding_pot),
    r!("bot.knowledge.cooking_entries:_roast_potatoes:2207", _roast_potatoes),
    r!("bot.knowledge.cooking_entries:_smoke_food:747", _smoke_food),
    r!("bot.knowledge.cooking_entries:_smoked_fish_reply:12042", _smoked_fish_reply),
    r!("bot.knowledge.cooking_entries:_stock_answer:642", _stock_answer),
    r!("bot.knowledge.cooking_entries:_tea_milk_reply:1063", _tea_milk_reply),
    r!("bot.knowledge.cooking_entries:_the_hot_pan_place:2129", _the_hot_pan_place),
    r!("bot.knowledge.cooking_entries:_the_milk_for_a_coffee:1047", _the_milk_for_a_coffee),
    r!("bot.knowledge.cooking_entries:_tins_keep:1791", _tins_keep),
    r!("bot.knowledge.cooking_entries:_tomato_sauce:1806", _tomato_sauce),
];


// ---------------------------------------------------------------------------------------------
// The row factories, building what PyBot built at import (`knowledge::built`).

use crate::knowledge::built;
use crate::knowledge::entry::{Entry, KeyPart, Reply};

/// `_not_short_of(*foods)`: a guard that stands the row down when the line says the house has
/// none of these (`_not_short_of_guard`).
pub fn _not_short_of(foods: &[String]) -> FnRef {
    let template = data::get("bot.knowledge.cooking_entries.SHORT_OF_THE_HOUSE").as_re().expect("a regex");
    let pattern = template.pattern().replacen("%s", &foods.join("|"), 1);
    let re = Re::new(&pattern, template.flags());
    built::closure("bot.knowledge.cooking_entries:_not_short_of.<locals>.guard:790", vec![("pattern", Value::Re(std::sync::Arc::new(re)))])
}

/// `_make_it(nouns, props)`: "make <noun> <prop>" and "make my <noun> <prop>" for each pair.
pub fn _make_it(nouns: &[String], props: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for noun in nouns {
        for prop in props {
            out.push(format!("make {noun} {prop}"));
            out.push(format!("make my {noun} {prop}"));
        }
    }
    out
}

/// `_swap(entry_id, key, reply, **rest)`: a swap entry that keeps out of the meals and out of the
/// steps - `SWAP_NOT_A_MEAL` in front of its own avoid words, and `_a_swap_and_not_a_step`
/// folded into its own guard.
#[allow(clippy::too_many_arguments)]
pub fn _swap(entry_id: &str, key: Vec<KeyPart>, reply: Reply, avoid: &[String], bias: i64, ask: Vec<KeyPart>, guard: Option<FnRef>) -> Entry {
    let guard = match guard {
        Some(g) => built::closure("bot.knowledge.cooking_entries:_swap.<locals>.<lambda>:1177", vec![("guard", Value::Fn(std::sync::Arc::new(g)))]),
        None => built::function("bot.knowledge.cooking_entries:_a_swap_and_not_a_step:1143"),
    };
    let not_a_meal = data::get("bot.knowledge.cooking_entries.SWAP_NOT_A_MEAL").strs();
    built::entry(entry_id, key, reply, &[built::words(&not_a_meal), built::words(avoid)], bias, ask, Some(guard))
}

/// `_swap(...).<locals>.<lambda>`: a swap and not a step, and the row's own guard.
fn _swap_guard(f: &FnRef, understood: &Understood) -> bool {
    _a_swap_and_not_a_step(understood) && f.cell("guard").and_then(Value::as_fn).is_some_and(|g| crate::knowledge::registry::guard(g, understood))
}

/// `_leaves_out(*ingredients)`: a dish card stands down when the line says an ingredient it
/// needs is not there; the guard reads the stems (each word and its plural or singular).
pub fn _leaves_out(ingredients: &[String]) -> FnRef {
    let mut stems: Vec<Value> = Vec::new();
    for one in ingredients {
        let other = match one.strip_suffix('s') {
            Some(stem) => stem.to_string(),
            None => format!("{one}s"),
        };
        for word in [one.clone(), other] {
            let v = Value::from(word.as_str());
            if !stems.contains(&v) {
                stems.push(v);
            }
        }
    }
    built::closure("bot.knowledge.cooking_entries:_leaves_out.<locals>.guard:1214", vec![("stems", Value::Set(std::sync::Arc::new(stems)))])
}

pub const FACTORY_GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.cooking_entries:_swap.<locals>.<lambda>:1177", _swap_guard)];

#[cfg(test)]
mod factory_tests {
    use super::*;

    #[test]
    fn guards_rebuild() {
        data::load();
        let mut n = 0;
        for row in crate::knowledge::entries() {
            let Some(g) = &row.guard else { continue };
            if g.id.starts_with("bot.knowledge.cooking_entries:_leaves_out") {
                let stems = g.cell("stems").unwrap().strs();
                let singles: Vec<String> = stems.iter().filter(|s| !s.ends_with('s')).cloned().collect();
                let built = _leaves_out(&singles);
                assert_eq!(built.cell("stems"), g.cell("stems"), "{}", row.id);
                n += 1;
            }
        }
        assert!(n > 0);
        let pattern = _not_short_of(&["eggs".to_string(), "egg".to_string()]);
        assert!(pattern.cell("pattern").unwrap().as_re().unwrap().pattern().contains("(?:eggs|egg)"));
    }
}
