//! Port of PyBot's `bot/knowledge/nutrition.py`: what is in food and how much of it to eat (veg,
//! cereal, daily figures, pregnancy supplements, nutrients, plant milks, drinks), with the guards
//! every nutrition row wears (food gone off, a dose, a cooking question, a keeping question).
#![allow(non_snake_case)] // closures are named `<factory>__<inner>`

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g6 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

const M: &str = "bot.knowledge.nutrition.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
/// A string global of this module.
fn s(name: &str) -> String {
    g(name).str().to_string()
}

macro_rules! strs {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($name)).strs());)*
    };
}
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> = LazyLock::new(|| rx(stringify!($name)));)*
    };
}

strs!(
    _ME_ASKING,
    _A_CHILD,
    _FUSSY,
    _A_GROWN_UP,
    _SOMEONE_ELSE,
    _AGE_WORDS,
    _A_DAY_TEEN,
    _A_DAY_WOMAN,
    _A_BARE_OK_ASK,
    _NOURISHING,
    _WHICH_MILK,
    _MILK_FOR_A_CHILD,
    _THE_PLANET,
    _SWEET_DRINKS,
);
regexes!(
    _A_DOSE,
    _CHILD_AGE,
    _A_THERMOMETER,
    COOKING_VERBS,
    _GOING_OFF,
    _A_STATE_NOT_SPOILAGE,
    _A_VERDICT_ASK,
    _EDIBILITY_ASK,
    _NAMED_TAKEAWAY,
);

/// A `((words...), text)` table global, in order.
fn word_table(name: &str) -> Vec<(Vec<String>, String)> {
    g(name).items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
}

static _CEREAL_BOWLS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_table("_CEREAL_BOWLS"));
static MORE_OF: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_table("MORE_OF"));
static NUTRIENT_ROLE: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_table("NUTRIENT_ROLE"));
static _PLANT_MILKS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_table("_PLANT_MILKS"));
static _A_DAY_WORDS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("_A_DAY_WORDS").items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string())).collect()
});

/// The line (clean, else text) lower-cased and padded with a space each side.
fn padded_line(understood: &Understood) -> String {
    format!(" {} ", sh::clean_or_text(understood).to_lowercase())
}

/// Fresh against frozen, asked as health, as money, as a smoothie, or of fruit.
pub fn _frozen_and_tinned_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = padded_line(understood);
    let any = |words: &[&str]| words.iter().any(|w| said.contains(w));
    let smoothie = any(&["smoothie", "smoothies", "blender", "shake"]);
    let price = any(&[" cheap", " cheaper", " cheapest", " cost", " costs", " price", " prices", " budget", " dearer", " value"]);
    let fruit = any(&[" fruit", " fruits", " peach", " peaches", " mango", " pineapple", " pears", " cherries"])
        && any(&["tinned", "tin of", "canned", "can of", " from a tin"]);
    if smoothie {
        return "Frozen berries and frozen fruit are the better buy for a smoothie: they are picked ripe and frozen \
                within hours, so they keep more vitamin C than soft fruit that has sat in a punnet, and frozen chunks \
                chill the drink and thicken it without ice. Tip them in from frozen with the yoghurt or milk and blend \
                - no need to thaw, and no added sugar, unlike the fruit in syrup."
            .to_string();
    }
    if price {
        return "Almost always yes: frozen vegetables cost less than fresh for the same weight, because nothing has \
                to be sorted, shipped soft or thrown away, and tinned beans, chickpeas and tomatoes are cheaper still. \
                Frozen also wastes nothing, since you pour out what you need and the bag goes back. The one thing \
                worth paying for fresh is a vegetable you are going to eat raw, where the texture is the point."
            .to_string();
    }
    if fruit {
        return "Tinned fruit counts, with one catch: it is picked ripe and tinned within hours, so the vitamins \
                survive, but fruit in syrup is a pudding rather than a portion - a couple of tinned peach halves in \
                syrup carry about 15g of sugar. Buy the ones in juice, or rinse the syrup off, and tinned fruit does \
                the same job as fresh. Frozen berries are the better buy of the two, since nothing is added to them \
                at all."
            .to_string();
    }
    "Frozen and tinned are as good as fresh, and often better: vegetables are frozen within hours of picking, so \
     their vitamins outlast a cabbage that has sat in the fridge all week, and tinned tomatoes are better than the \
     pale ones flown in out of season. Frozen peas, sweetcorn, spinach and berries are the best buys, and tinned \
     beans, chickpeas and sardines are cheap protein with nothing added. The tin or the bag is not the problem - \
     read the label: brine, syrup, oil and sauces are where the salt and sugar come from. Frozen fish is fine too, \
     thawed overnight in the fridge and cooked from there."
        .to_string()
}

/// `entry(...)`'s default guard (`lambda read: not food_has_gone_off(read)`): the row stands
/// down when the line says the food has gone off.
pub fn entry__lambda_83(read: &Understood) -> bool {
    !dup::food_has_gone_off(read)
}

/// The guard an entry passed in, run by its id: this module's own by a direct call, the two
/// from other modules through their DUP ports, anything else through the registry.
fn inner_guard(inner: &Value, read: &Understood) -> bool {
    let Some(f) = inner.as_fn() else { return false };
    match f.id.as_str() {
        "bot.knowledge.nutrition:_not_a_keeping_question:970" => _not_a_keeping_question(read),
        "bot.knowledge.nutrition:not_a_cooking_question:1012" => not_a_cooking_question(read),
        "bot.knowledge.nutrition:not_a_dose:128" => not_a_dose(read),
        "bot.knowledge.nutrition:_not_a_verdict_on_what_is_going_off:3043" => _not_a_verdict_on_what_is_going_off(read),
        "bot.knowledge.nutrition:_not_a_named_takeaway:3214" => _not_a_named_takeaway(read),
        "bot.knowledge.lookup:not_a_count_for_the_box_or_the_dish:4234" => dup::not_a_count_for_the_box_or_the_dish(read),
        "bot.knowledge.shopping:not_a_present:544" => dup::not_a_present(read),
        _ => sh::call_guard(inner, read),
    }
}

/// `entry(...)`'s wrapped guard (`lambda read: inner(read) and not food_has_gone_off(read)`).
pub fn entry__lambda_86(f: &FnRef, read: &Understood) -> bool {
    let inner_ok = f.cell("inner").is_some_and(|inner| inner_guard(inner, read));
    inner_ok && !dup::food_has_gone_off(read)
}

/// True unless the line times a dose rather than a session or a meal.
pub fn not_a_dose(understood: &Understood) -> bool {
    !_A_DOSE.is_match(&sh::or(&understood.clean, &understood.plain).to_lowercase())
}

/// Whole words only (a word padded with spaces is a plain substring test).
pub fn _a_word_in<S: AsRef<str>>(line: &str, words: &[S]) -> bool {
    words.iter().map(AsRef::as_ref).any(|word| {
        if word.starts_with(' ') && word.ends_with(' ') {
            line.contains(word)
        } else {
            sh::re_search(&format!(r"\b{}\b", sh::escape(text::strip(word))), line, 0)
        }
    })
}

/// A child the line names: a son or a 4 year old, but not a 40 year old husband.
pub fn _a_child_in(line: &str) -> bool {
    for word in _A_CHILD.iter() {
        if _AGE_WORDS.contains(word) {
            continue;
        }
        if _a_word_in(line, &[word]) {
            return true;
        }
    }
    _CHILD_AGE.finditer(line).iter().any(|m| m.str(1).parse::<i64>().is_ok_and(|age| age < 18))
}

/// Who a line says the refusal is about: "me", "grown-up", "child", or None.
pub fn _who_in_line(line: &str) -> Option<&'static str> {
    if _a_word_in(line, &_ME_ASKING) {
        return Some("me");
    }
    let a_child = _a_child_in(line);
    if _a_word_in(line, &_A_GROWN_UP) && !a_child {
        return Some("grown-up");
    }
    if a_child || _a_word_in(line, &_FUSSY) {
        return Some("child");
    }
    None
}

/// Who the refusal is about, reading back through the conversation when the line says nobody.
pub fn _who_this_is_about(memory: &Memory, understood: &Understood) -> Option<&'static str> {
    let line = format!(" {} ", understood.clean.to_lowercase());
    if let Some(who) = _who_in_line(&line) {
        return Some(who);
    }
    let somebody_else = _a_word_in(&line, &_SOMEONE_ELSE);
    let said_now = text::strip(&line);
    for turn in memory.turns.iter().rev() {
        let said = format!(" {} ", text::strip(&turn.user).to_lowercase());
        if text::strip(&said).is_empty() || text::strip(&said) == said_now {
            continue;
        }
        let who = _who_in_line(&said);
        if who == Some("me") && somebody_else {
            return None;
        }
        if who.is_some() {
            return who;
        }
    }
    None
}

/// Five a day for the person asking, and a different answer for the person they name.
pub fn _veg_reply(memory: &Memory, understood: &Understood) -> String {
    match _who_this_is_about(memory, understood) {
        Some("me") => s("_VEG_FIVE_A_DAY"),
        Some("grown-up") => s("_VEG_A_GROWN_UP"),
        Some("child") => s("_VEG_A_CHILD"),
        _ => s("_VEG_WHO_IS_IT"),
    }
}

/// Every bowl the line names, in the order the words appear in it: (where, name, verdict).
pub fn _the_bowl_named(understood: &Understood) -> Vec<(usize, String, String)> {
    let line = format!(" {} ", text::squash(&sh::clean_or_text(understood).to_lowercase()));
    let at = |name: &str| -> Option<usize> {
        let needle = format!(" {name}");
        line.find(&needle).map(|b| text::char_index(&line, b))
    };
    let mut found: Vec<(usize, String, String)> = Vec::new();
    for (names, verdict) in _CEREAL_BOWLS.iter() {
        if let Some(first) = names.iter().filter_map(|n| at(n)).min() {
            found.push((first, names[0].clone(), verdict.clone()));
        }
    }
    if let Some(first) = ["porridge", "oat", "oats", "oatmeal"].iter().filter_map(|n| at(n)).min() {
        found.push((first, "porridge".to_string(), s("PORRIDGE_VERDICT")));
    }
    found.sort();
    let mut bowls: Vec<(usize, String, String)> = Vec::new();
    for bowl in found {
        if bowls.iter().any(|b| b.1 == bowl.1) {
            continue;
        }
        bowls.push(bowl);
    }
    bowls
}

/// The verdict on the bowl the person named, and the comparison only when they named two.
pub fn _cereal_verdict(_memory: &Memory, understood: &Understood) -> String {
    let named = _the_bowl_named(understood);
    if named.is_empty() {
        return s("PORRIDGE_VERDICT");
    }
    if named.len() == 1 {
        return named[0].2.clone();
    }
    let (first, second) = (&named[0], &named[1]);
    let pair = |a: &str, b: &str| (first.1 == a && second.1 == b) || (first.1 == b && second.1 == a);
    if pair("porridge", "granola") {
        return "Porridge is the better everyday bowl of the two: oats and water with no added sugar, about 4g of \
                fibre in 40g, and it keeps you full. Granola is the same oats with oil and sugar baked on - 11g to 15g \
                of sugar in a 50g serving - so it is a pudding's worth of sugar with a good fibre story. If you like \
                the crunch, granola on top of plain yoghurt, 20g rather than 50g, is the compromise."
            .to_string();
    }
    if pair("porridge", "muesli") {
        return "Porridge is the plainer of the two and has no added sugar; muesli brings nuts and dried fruit, so it \
                has more fibre and more sugar - 7g to 12g in 50g. Either is a good breakfast; muesli needs measuring, \
                porridge needs nothing but oats."
            .to_string();
    }
    let short = |bowl: &(usize, String, String)| -> String {
        g("_CEREAL_SHORT")
            .get(&bowl.1)
            .map(|v| v.str().to_string())
            .unwrap_or_else(|| format!("{}.", bowl.2.split('.').next().unwrap_or("")))
    };
    format!("{} {}", short(first), short(second))
}

/// The daily figure for the nutrient the line names: the woman's or the teenager's where it differs.
pub fn _a_day(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = padded_line(understood);
    let name = _A_DAY_WORDS.iter().find(|(word, _)| clean.contains(word.as_str())).map(|(_, n)| n.clone())?;
    if dup::_nutrient_food(&clean).is_some() {
        return None;
    }
    let figures = g("_A_DAY").get(&name)?;
    if sh::any_in(&clean, &_A_DAY_TEEN) {
        if let Some(teen) = figures.get("teen") {
            return Some(teen.str().to_string());
        }
    }
    if sh::any_in(&clean, &_A_DAY_WOMAN) {
        if let Some(woman) = figures.get("woman") {
            return Some(woman.str().to_string());
        }
    }
    // Python indexes figures["adult"]; a table without one raises and the reply declines
    figures.get("adult").map(|v| v.str().to_string())
}

/// The supplement answer for a pregnancy, which is not the adult figure.
pub fn _pregnancy_vitamins(_memory: &Memory, understood: &Understood) -> String {
    let line = format!(" {} ", understood.clean.to_lowercase());
    let any = |words: &[&str]| words.iter().any(|w| line.contains(w));
    if any(&["vitamin a", "liver", "cod liver", "retinol"]) {
        return "Skip vitamin A supplements and liver while you are pregnant: too much of it is harmful to the baby, \
                and it is the one vitamin people take by accident, in a multivitamin or a cod liver oil. An ordinary \
                multivitamin is not the one to take - a pregnancy one leaves the vitamin A out."
            .to_string();
    }
    if any(&["folic", "folate"]) {
        return "Take 400 micrograms of folic acid every day from before you conceive until the end of week 12, which \
                is double the ordinary 200: it cuts the risk of spina bifida, and food alone will not get you there. \
                After 12 weeks, and while breastfeeding, the ordinary 200 is enough."
            .to_string();
    }
    if any(&["vitamin d", "vit d"]) {
        return "Take 10 micrograms (400 IU) of vitamin D every day, all year, while you are pregnant and while you \
                breastfeed. The figure is the same as everyone else's, but you cannot count on the sun or on food, \
                so it comes from a supplement."
            .to_string();
    }
    if line.contains("iron") {
        return "You do not need an iron supplement unless a blood test shows you are low: your blood count is \
                checked at your booking appointment and again at about 28 weeks, and low iron is treated then. \
                Vitamin C with a meal helps you take up the iron in food, and tea or coffee with it works against you."
            .to_string();
    }
    if line.contains("calcium") {
        return "You need the same 700mg of calcium a day as anyone else - a pint of milk, or a pot of yoghurt and a \
                matchbox of cheese. The baby takes what it needs from your bones whatever you eat, so the dairy is \
                for your bones, not theirs, and a supplement is only for a low a blood test has found."
            .to_string();
    }
    if line.contains("b12") || line.contains("vegan") {
        return "Vitamin B12 matters in pregnancy and breastfeeding, and it comes from meat, fish, milk and eggs: if \
                you are vegan, take a B12 supplement or a fortified milk, because the baby's supply comes from yours \
                and the stores run down quietly."
            .to_string();
    }
    "Folic acid, 400 micrograms a day until the end of week 12, and vitamin D, 10 micrograms a day all year, are the \
     two worth taking in pregnancy. Leave out vitamin A and liver, and take iron only for a low a blood test has \
     found."
        .to_string()
}

/// The row of a nutrient table whose word the line says on its own or starts with.
fn nutrient_row(line: &str, table: &[(Vec<String>, String)]) -> Option<String> {
    let stripped = text::strip(line);
    table
        .iter()
        .find(|(words, _)| words.iter().any(|w| line.contains(&format!(" {w} ")) || stripped.starts_with(&format!("{w} "))))
        .map(|(_, t)| t.clone())
}

/// Where to get more of the nutrient this line names, or None when it names none.
pub fn _more_of(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = padded_line(understood);
    if dup::_nutrient_food(&line).is_some() {
        return None;
    }
    nutrient_row(&line, &MORE_OF)
}

/// A person who thinks their iron is low wants a blood test before a shopping list.
pub fn _low_iron(_memory: &Memory, _understood: &Understood) -> String {
    "That is worth a blood test rather than a guess: tiredness, breathlessness on the stairs, pallor, brittle nails \
     and hair loss are the usual signs, and your GP can check a full blood count in a few days. Do not start iron \
     tablets on suspicion - too much iron is harmful - but do eat iron-rich food meanwhile: red meat, lentils, beans, \
     fortified cereal and dark green veg, with vitamin C in the same meal and tea or coffee kept an hour away."
        .to_string()
}

/// What the nutrient this line names does, or None when it names none.
pub fn _nutrient_role(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = padded_line(understood);
    if dup::_nutrient_food(&line).is_some() {
        return None;
    }
    nutrient_row(&line, &NUTRIENT_ROLE)
}

/// True when one of `words` is said on its own, at either end of the line as well as inside.
pub fn _a_bare_word<S: AsRef<str>>(said: &str, words: &[S]) -> bool {
    let said = format!(" {} ", text::strip(said));
    let trimmed = text::rstrip(&said);
    words.iter().map(AsRef::as_ref).any(|word| {
        said.contains(&format!(" {word} "))
            || said.starts_with(&format!(" {word} "))
            || trimmed == format!(" {word}")
            || trimmed.ends_with(&format!(" {word}"))
    })
}

/// The carton the line named, or the ranking when it asks which milk to take.
pub fn _a_plant_milk(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", sh::or(&understood.clean, &understood.norm_text).to_lowercase());
    if _MILK_FOR_A_CHILD.iter().any(|w| said.contains(&format!(" {w}"))) {
        return None;
    }
    let trimmed = text::rstrip(&said);
    let names = |w: &String| {
        said.contains(&format!(" {w} ")) || trimmed.ends_with(&format!(" {w}")) || said.contains(&format!(" {w}s "))
    };
    if _THE_PLANET.iter().any(names) {
        return Some(s("_MILK_AND_THE_PLANET"));
    }
    let named: Vec<(&String, &String)> =
        _PLANT_MILKS.iter().filter(|(words, _)| words.iter().any(names)).map(|(words, t)| (&words[0], t)).collect();
    let kept: Vec<&(&String, &String)> =
        named.iter().filter(|(name, _)| !named.iter().any(|(other, _)| name != other && other.contains(name.as_str()))).collect();
    if kept.len() == 1 {
        return Some(kept[0].1.clone());
    }
    if kept.len() > 1 {
        let texts: Vec<&str> = kept.iter().take(2).map(|(_, t)| t.as_str()).collect();
        return Some(format!(
            "{} If it is protein you are after, take the one with more of it; if it is the cup of tea or the coffee, \
             take the one that does not split.",
            texts.join(" ")
        ));
    }
    if sh::any_in(&said, &_WHICH_MILK) {
        return Some(s("_THE_MILK_RANKING"));
    }
    if _a_bare_word(&said, &_A_BARE_OK_ASK) && !sh::any_in(&said, &_NOURISHING) {
        return Some(format!("{} {}", s("_COW_MILK_REPLY"), s("_THE_FRIDGE_CHECK")));
    }
    Some(s("_COW_MILK_REPLY"))
}

/// A verdict on the drink the line named: tea or coffee.
pub fn _coffee_or_tea(_memory: &Memory, understood: &Understood) -> String {
    let line = format!(" {} ", understood.clean.to_lowercase());
    let tea = line.contains(" tea") || line.contains("teas ");
    let coffee = line.contains("coffee") || line.contains("espresso") || line.contains("latte");
    if tea && !coffee {
        return "Tea is fine: four or five cups of ordinary black tea is about half the caffeine of the same number \
                of coffees, it counts towards your fluid, and the tea itself has no calories - the sugar and the \
                biscuits are what add up. Herbal and green teas are caffeine-free or nearly, so they are not the ones \
                to worry about in the evening. If you are anaemic or take iron tablets, keep the cup an hour away \
                from a meal, because the tannins cut how much iron you take up."
            .to_string();
    }
    "Coffee is not the villain it was: up to about four cups a day is linked to no harm for most adults, and it may \
     be mildly protective for the heart and liver. The trouble is what goes in it (a latte is 120 calories) and \
     drinking it late enough to spoil your sleep. Filter coffee raises cholesterol a little more than espresso, \
     which is why paper filters are still worth using."
        .to_string()
}

/// Whether the line is a which-is-better question rather than a keeping one.
pub fn _not_a_keeping_question(understood: &Understood) -> bool {
    !dup::a_food_is_being_kept(understood)
}

/// The wholemeal swap for the grain the line names: rice, or pasta.
pub fn _wholegrain(_memory: &Memory, understood: &Understood) -> String {
    let said = format!(" {} ", understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" "));
    let pasta = [" pasta", " spaghetti", " macaroni", " noodles", " penne", " fusilli", " tagliatelle"]
        .iter()
        .any(|w| said.contains(w));
    let rice = said.contains(" rice");
    let head = if pasta && !rice {
        "Wholewheat pasta keeps the bran and the germ, so it has about three times the fibre of white"
    } else {
        "Brown rice keeps the bran and the germ, so it has about three times the fibre of white"
    };
    if pasta && !rice {
        let rest = "White pasta is not bad - it just gives you less fibre and less staying power. It keeps its \
                    calories, around 130 to 150 per 100g cooked, so the swap is about fibre and fullness, not about \
                    weight. Wholewheat takes a minute or two longer and holds sauce well; the half-and-half bag is an \
                    easy start if the family notices the change.";
        return format!("{head}, and it has a slower rise in blood sugar too. {rest}");
    }
    let rest = "White rice and pasta are not bad - they just give you less fibre and less staying power. Both keep \
                their calories, around 130 to 150 per 100g cooked, so the swap is about fibre and fullness, not about \
                weight.";
    format!("{head} and a slower rise in blood sugar; wholewheat pasta is the same idea against white pasta. {rest}")
}

/// True when the line is about a food rather than about how it is cooked.
pub fn not_a_cooking_question(understood: &Understood) -> bool {
    let said = text::squash(&format!("{} {}", understood.text, understood.clean).to_lowercase());
    if COOKING_VERBS.is_match(&said) {
        return false;
    }
    !_A_THERMOMETER.is_match(&said)
}

/// The sugary-drink verdict, naming the drink the line named.
pub fn _sugary_drink_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = format!(" {} ", sh::text_or_clean(understood).to_lowercase());
    let named = _SWEET_DRINKS.iter().find(|w| said.contains(&format!(" {w} ")));
    let tail = "The sugar is the whole problem: it comes as liquid, so it does not fill you up, and it is what \
                dentists see. Diet and zero versions cut the sugar and are the sensible swap rather than a healthy \
                drink; water, or fizzy water with a slice of lemon, is the free one.";
    if let Some(named) = named {
        return format!(
            "A glass of {named} is about as sugary as cola - roughly 35g in a 330ml glass, seven teaspoons - and an \
             adult should stay under 30g a day in total. {tail}"
        );
    }
    format!(
        "A 330ml can of cola has about 35g of sugar, seven teaspoons, and lemonade, squash and the rest are in the \
         same range - an adult should stay under 30g a day in total. {tail}"
    )
}

/// False when the food the line is about is going off: that is a keeping question.
pub fn _not_a_verdict_on_what_is_going_off(understood: &Understood) -> bool {
    let clean = understood.clean.to_lowercase();
    if _GOING_OFF.is_match(&clean) {
        if _A_STATE_NOT_SPOILAGE.is_match(&clean) && _A_VERDICT_ASK.is_match(&clean) && !_EDIBILITY_ASK.is_match(&clean) {
            return true;
        }
        return false;
    }
    if let Some(leans) = &understood.leans_on {
        let past = leans.text.to_lowercase();
        if _GOING_OFF.is_match(&past) && _EDIBILITY_ASK.is_match(&clean) {
            return false;
        }
    }
    true
}

/// False for a line whose only fast food is a takeaway, which the takeaway card answers.
pub fn _not_a_named_takeaway(understood: &Understood) -> bool {
    static JUNK: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bjunk\b", 0));
    let clean = understood.clean.to_lowercase();
    if !_NAMED_TAKEAWAY.is_match(&clean) {
        return true;
    }
    JUNK.is_match(&clean)
}

// --- registry wrappers ------------------------------------------------------------------------

fn g_83(_f: &FnRef, u: &Understood) -> bool {
    entry__lambda_83(u)
}
fn g_86(f: &FnRef, u: &Understood) -> bool {
    entry__lambda_86(f, u)
}
fn g_not_a_keeping_question(_f: &FnRef, u: &Understood) -> bool {
    _not_a_keeping_question(u)
}
fn g_not_a_named_takeaway(_f: &FnRef, u: &Understood) -> bool {
    _not_a_named_takeaway(u)
}
fn g_not_a_verdict_on_what_is_going_off(_f: &FnRef, u: &Understood) -> bool {
    _not_a_verdict_on_what_is_going_off(u)
}
fn g_not_a_cooking_question(_f: &FnRef, u: &Understood) -> bool {
    not_a_cooking_question(u)
}
fn g_not_a_dose(_f: &FnRef, u: &Understood) -> bool {
    not_a_dose(u)
}

macro_rules! replies {
    ($($w:ident => $f:ident : $kind:tt),* $(,)?) => {
        $(fn $w(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { replies!(@call $kind, $f, m, u) })*
    };
    (@call str, $f:ident, $m:ident, $u:ident) => { Some($f($m, $u)) };
    (@call opt, $f:ident, $m:ident, $u:ident) => { $f($m, $u) };
}
replies!(
    r_a_day => _a_day: opt,
    r_a_plant_milk => _a_plant_milk: opt,
    r_cereal_verdict => _cereal_verdict: str,
    r_coffee_or_tea => _coffee_or_tea: str,
    r_frozen_and_tinned_reply => _frozen_and_tinned_reply: str,
    r_low_iron => _low_iron: str,
    r_more_of => _more_of: opt,
    r_nutrient_role => _nutrient_role: opt,
    r_pregnancy_vitamins => _pregnancy_vitamins: str,
    r_sugary_drink_reply => _sugary_drink_reply: str,
    r_veg_reply => _veg_reply: str,
    r_wholegrain => _wholegrain: str,
);

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.nutrition:entry.<locals>.<lambda>:83", g_83),
    ("bot.knowledge.nutrition:entry.<locals>.<lambda>:86", g_86),
    // held in the `inner` cell of the lambda above; registered so a cell can run them too
    ("bot.knowledge.nutrition:_not_a_keeping_question:970", g_not_a_keeping_question),
    ("bot.knowledge.nutrition:_not_a_named_takeaway:3214", g_not_a_named_takeaway),
    ("bot.knowledge.nutrition:_not_a_verdict_on_what_is_going_off:3043", g_not_a_verdict_on_what_is_going_off),
    ("bot.knowledge.nutrition:not_a_cooking_question:1012", g_not_a_cooking_question),
    ("bot.knowledge.nutrition:not_a_dose:128", g_not_a_dose),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.nutrition:_a_day:535", r_a_day),
    ("bot.knowledge.nutrition:_a_plant_milk:913", r_a_plant_milk),
    ("bot.knowledge.nutrition:_cereal_verdict:511", r_cereal_verdict),
    ("bot.knowledge.nutrition:_coffee_or_tea:948", r_coffee_or_tea),
    ("bot.knowledge.nutrition:_frozen_and_tinned_reply:27", r_frozen_and_tinned_reply),
    ("bot.knowledge.nutrition:_low_iron:704", r_low_iron),
    ("bot.knowledge.nutrition:_more_of:689", r_more_of),
    ("bot.knowledge.nutrition:_nutrient_role:775", r_nutrient_role),
    ("bot.knowledge.nutrition:_pregnancy_vitamins:570", r_pregnancy_vitamins),
    ("bot.knowledge.nutrition:_sugary_drink_reply:1041", r_sugary_drink_reply),
    ("bot.knowledge.nutrition:_veg_reply:315", r_veg_reply),
    ("bot.knowledge.nutrition:_wholegrain:974", r_wholegrain),
];


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/knowledge/nutrition.py:65 `entry(id, key, reply, avoid=(), bias=0, ask=(), slips=(), keep=(), guard=None)` (never called on the trace corpus)
    pub fn entry(id: &Value, key: &str, reply: &str, avoid: &Value, bias: &Value, ask: &str, slips: &Value, keep: i64, guard: &Value) -> Value {
        crate::unported!("bot.knowledge.nutrition:entry")
    }

}

