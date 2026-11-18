//! Port of `bot/knowledge/fruit.py`: getting a fruit out of its skin, one method per fruit, and
//! the skin, bruise and pip questions. Tables and entries come from the bundle.

use crate::data;
use crate::data::FnRef;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.fruit.{name}"))
}

/// A `((words...), text)` table.
fn word_rows(name: &str) -> Vec<(Vec<String>, String)> {
    data::get(&format!("bot.knowledge.fruit.{name}"))
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
}

static FRUIT_METHODS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("FRUIT_METHODS"));
static JUICE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("JUICE_WORDS"));
static JUICE_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("JUICE_ASKS"));
static CITRUS: LazyLock<Vec<String>> = LazyLock::new(|| strs("CITRUS"));
static JUICING: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.fruit.JUICING"));
static PREP_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("PREP_WORDS"));
static NEED_TO_PEEL: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.fruit.NEED_TO_PEEL"));
static SKIN_IN_A_DISH: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.fruit.SKIN_IN_A_DISH"));
static SHOP_SEED_VEG: LazyLock<Vec<String>> = LazyLock::new(|| strs("SHOP_SEED_VEG"));
static SEED_SOURCE: LazyLock<Vec<String>> = LazyLock::new(|| strs("SEED_SOURCE"));
static TENDER_FRUIT: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("TENDER_FRUIT"));
static CITRUS_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| strs("CITRUS_NAMES"));
static KERNEL_FRUIT: LazyLock<Vec<String>> = LazyLock::new(|| strs("KERNEL_FRUIT"));

static NOT_A_WORD_CHAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));
static HOW_MANY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow many\b", 0));
static A_CONTAINER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:in|inside|does|has|have|are there)\b", 0));

/// The line, with the punctuation taken out, so a word can be looked for on its own.
pub fn _said(understood: &Understood) -> String {
    let text = if understood.clean.is_empty() { &understood.text } else { &understood.clean }.to_lowercase();
    format!(" {} ", NOT_A_WORD_CHAR.sub(" ", &text))
}

/// Whether the word stands on its own in the padded line.
pub fn _holds(line: &str, word: &str) -> bool {
    line.contains(&format!(" {word} "))
}

fn holds_any<S: AsRef<str>>(line: &str, words: &[S]) -> bool {
    words.iter().any(|w| _holds(line, w.as_ref()))
}

/// The way to cut, peel, juice or stone the fruit the line names, one method per fruit.
pub fn _fruit_prep(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _said(understood);
    if holds_any(&line, &JUICE_WORDS) {
        if !holds_any(&line, &JUICE_ASKS) {
            return None;
        }
        return if holds_any(&line, &CITRUS) { Some(JUICING.clone()) } else { None };
    }
    if !holds_any(&line, &PREP_WORDS) {
        return None;
    }
    for (words, method) in FRUIT_METHODS.iter() {
        if holds_any(&line, words) {
            return Some(method.clone());
        }
    }
    None
}

/// The skin is food, and the answer names the fruit the line names.
pub fn _skin_reply(_memory: &mut Memory, reading: &Understood) -> String {
    let said = if reading.clean.is_empty() { &reading.text } else { &reading.clean }.to_lowercase();
    let lead = if NEED_TO_PEEL.is_match(&said) { "No need to peel - " } else { "Yes - " };
    let in_a_dish = SKIN_IN_A_DISH.is_match(&said);
    let dish = if in_a_dish {
        " For a crumble, a pie or a cake, leave it on: the skin softens as it bakes and holds \
         the fruit together, and peel only if you want the filling perfectly smooth."
    } else {
        ""
    };
    if said.contains("kiwi") || said.contains("kiwifruit") {
        let body = "kiwi skin is edible, and it holds more fibre and vitamin C than the green \
                    flesh, so scrub it under the tap and eat it if the fuzz does not put you off: \
                    the skin is thin, and rubbing it under water takes most of the hairiness off. A \
                    ripe kiwi eaten skin and all tastes the same, only tarter near the skin.";
        if lead.starts_with("No need") {
            return format!("{lead}{body}");
        }
        return format!("Yes - {body}");
    }
    if ["grape", "grapes"].iter().any(|w| said.contains(w)) {
        return format!(
            "{lead}grape skins are food - that is where the colour, the tannin and much of \
             the fibre sit - so wash them and eat them, and only peel a grape for a small \
             child or someone who cannot manage the skin. A white bloom on the skin is \
             natural yeast, not spoilage, so rinse it rather than peel it."
        );
    }
    if ["peach", "peaches", "plum", "plums"].iter().any(|w| said.contains(w)) {
        return format!(
            "{lead}a peach or a plum is eaten with the skin on, and washing it is enough: \
             the skin holds the colour and much of the fibre, and peeling only matters for a \
             smooth chutney or for somebody who finds the fuzz unpleasant. Pour boiling water \
             over a peach for 30 seconds and the skin slips off if you do want it off.{dish}"
        );
    }
    format!(
        "{lead}apple and pear skin is the best part of it: the fibre, the colour and the \
         quercetin that gives the skin its job are all in there, so wash the fruit and eat it \
         as it is. Peeling loses about a third of the fibre and most of the vitamin C, and \
         the only ones worth peeling are a waxed one if the wax bothers you, or fruit for a \
         small child who cannot chew the skin.{dish} The one case where the answer is cook it: if raw apple or pear makes your mouth \
         itch or tingle, heat settles that - stewed, baked or in a crumble the protein that \
         does it is broken down, and peeled is usually fine too."
    )
}

/// A bruise on fruit is a knock, not a germ, and the fruit is still food.
pub fn _bruise_reply(_memory: &mut Memory, reading: &Understood) -> String {
    let said = reading.clean.to_lowercase();
    if said.contains("banana") {
        return "A bruised banana is fine to eat: the bruise is the flesh knocked and oxidising, \
                not rot, and a brown-spotted banana is sweeter because the starch has turned to \
                sugar, which is why it is the one for banana bread. Cut out a soft black patch \
                if you do not like the look of it, and bin the fruit only if the skin has split \
                and it smells fermented or has mould on it."
            .to_string();
    }
    "A bruise on fruit is a knock, not spoilage: the cell walls are broken and the flesh \
     browns as it oxidises, which is why an apple or a peach browns where it was dropped. \
     Cut the patch out if the texture bothers you, or eat it - the rest of the fruit is \
     unchanged. Bin it only when the bruise is wet and sour-smelling or there is mould, \
     because a soft bruise is the way in for rot."
        .to_string()
}

/// The first fruit word in the line, as one bare noun: "a grapefruit", "a fig".
pub fn _the_fruit<S: AsRef<str>>(line: &str, words: &[S]) -> String {
    for word in words {
        let word = word.as_ref();
        if _holds(line, word) {
            return if word.ends_with('s') && !word.ends_with("ss") {
                word[..word.len() - 1].to_string()
            } else {
                word.to_string()
            };
        }
    }
    "citrus".to_string()
}

/// Whether a pip, stone or seed will grow, which depends on which fruit it came from.
pub fn _growing_from_a_pip(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _said(understood);
    if holds_any(&line, &["avocado", "avocados"]) {
        return Some(
            "An avocado stone sprouts easily and almost never gives you avocados: push three \
             cocktail sticks into the fat end and sit it over a glass so the base is just in \
             the water, and roots show in four to six weeks. Pot it in free-draining compost \
             when the shoot is 15cm, keep it in bright light and pinch the top out to make it \
             bushy. It is a handsome houseplant that wants a bigger pot each year; fruit \
             needs a mature tree, two of them for cross-pollination, and a warm climate."
                .to_string(),
        );
    }
    if holds_any(&line, &SHOP_SEED_VEG) {
        if !holds_any(&line, &SEED_SOURCE) {
            return None;
        }
        let lead = if holds_any(&line, &["can i", "could i", "is it possible", "will it", "would it", "does it"]) {
            "Yes, and it is worth doing: "
        } else {
            "Save the seed first: "
        };
        return Some(format!(
            "{lead}scoop the seeds and their jelly into a glass, add a little water and leave them \
             three or four days until a film of mould sits on top - that ferment kills the \
             germination inhibitor in the jelly - then rinse them, dry them on paper and sow \
             them indoors from late February at 18-21C, about 6mm deep. A supermarket tomato \
             is often a hybrid, so the fruit may come smaller or uneven, but it grows and it \
             fruits, and dry seed in a paper envelope still comes up three or four years \
             later."
        ));
    }
    if holds_any(&line, &CITRUS) {
        return Some(format!(
            "A {} seed from the fruit bowl does grow, and comes closer to the \
             parent than an apple pip does, but it is slow: expect five to fifteen years \
             before fruit, and a plant that wants a sunny windowsill in winter and a spot \
             outside in summer. Sow it fresh, before the seed dries out, in moist seed \
             compost at about 21C, and pot it on as it grows. A named variety comes as a \
             grafted tree, which fruits in two or three years and is what most people want.",
            _the_fruit(&line, &CITRUS_NAMES)
        ));
    }
    for (words, said) in TENDER_FRUIT.iter() {
        if holds_any(&line, words) {
            return Some(said.clone());
        }
    }
    if holds_any(
        &line,
        &[
            "peach", "peaches", "nectarine", "nectarines", "plum", "plums", "cherry", "cherries", "apricot", "apricots",
            "damson", "damsons", "olive", "olives",
        ],
    ) {
        return Some(
            "A stone will grow, but it needs a winter first: clean the flesh off, mix the stone \
             with damp compost in a sealed bag and keep it in the fridge for six to ten \
             weeks, then pot it and wait - plums and peaches come up in a couple of months \
             after that. Expect four to eight years before fruit, and the fruit will not be \
             the variety you ate, because a stone fruit is cross-pollinated. For fruit you \
             can count on, buy a bare-root tree from November to March and plant it where it \
             gets sun."
                .to_string(),
        );
    }
    if !holds_any(
        &line,
        &["apple", "apples", "pear", "pears", "apple tree", "pear tree", "fruit tree", "fruit pip", "fruit pips"],
    ) {
        return None;
    }
    Some(
        "An apple or pear pip does grow, but it will not give you the apple you ate: pips are \
         crossed with whatever tree pollinated the flower, so the fruit is a lottery, often \
         hard and sour, and it takes five to ten years to find out. If you want to try, keep \
         the pips damp in the fridge for six to eight weeks - they need cold to break \
         dormancy - then sow them in compost on a windowsill, where they come up in three \
         to four weeks, and pot them on. For eating \
         apples, buy a named variety: a bare-root tree planted between November and March \
         fruits in three or four years and tastes of something."
            .to_string(),
    )
}

/// Whether the pips in the fruit are dangerous, which is a dose question, not a yes or no.
pub fn _a_pip_to_eat(_memory: &mut Memory, understood: &Understood) -> String {
    let line = _said(understood);
    if holds_any(&line, &KERNEL_FRUIT) {
        return "Careful with these ones: a bitter apricot kernel holds about forty times the \
                amygdalin of an apple pip, and amygdalin releases cyanide when it is chewed. One \
                or two do most adults no harm; around ten to twenty bitter kernels have caused \
                poisoning and deaths, and a child needs far fewer, so a child should have none \
                at all. Bitter almonds are the same thing under another name, and the 'vitamin \
                B17' sold as a cancer cure is laetrile, which is the same cyanide story and does \
                not treat cancer. If someone has eaten a handful, ring 999 - headache, dizziness, \
                vomiting, a flushed face or confusion after eating them is an emergency - and \
                ring 111 if they feel well but you are unsure. Sweet kernels from a labelled \
                packet are much lower, but an unlabelled bag is not worth the risk."
            .to_string();
    }
    if HOW_MANY.is_match(&line)
        && holds_any(&line, &["pip", "pips", "seed", "seeds", "stone", "stones", "kernel", "kernels"])
        && A_CONTAINER.is_match(&line)
    {
        if _holds(&line, "apple") || _holds(&line, "apples") {
            return "Usually five to ten, in two rows in the core: an apple has five seed \
                    chambers and each holds one or two pips. A big apple has more pips than a \
                    small one, and a pippin is simply an apple grown from a pip. None of them \
                    will grow into the apple you ate, because the tree they came from was \
                    pollinated by another one nearby."
                .to_string();
        }
        if holds_any(&line, &["lemon", "lemons", "lime", "limes", "orange", "oranges"]) {
            return "A lemon or a lime has about eight to ten seeds, an orange about the same, \
                    and they sit in the segments rather than in a core. A citrus seed is more \
                    likely to grow than an apple pip, and it still takes a few years and a \
                    sunny windowsill to become a plant worth keeping."
                .to_string();
        }
    }
    if holds_any(&line, &["dog", "dogs", "puppy", "cat", "cats", "kitten", "pet", "pets", "bird", "birds", "parrot"]) {
        return "The pips and stones of apples, pears, cherries, plums, peaches and apricots are \
                worse for a dog or a cat than for you: they hold amygdalin, which releases \
                cyanide when one is crushed and chewed, and a small animal needs much less than \
                a person does. A couple swallowed whole usually pass straight through, but call \
                the vet if a pet has chewed a handful, and keep the cores and stones out of the \
                bowl you put down."
            .to_string();
    }
    "Fine, in the amounts anyone eats by accident: the pips and kernels of apples, pears, \
     cherries, plums, peaches and apricots hold amygdalin, which releases a little cyanide \
     when the pip is crushed and chewed - but you would have to chew a cupful at once to \
     be in trouble. A pip swallowed whole comes out the other end intact, so a child who \
     ate two or three is not an emergency; watch for a sore tummy and ring 111 if you are \
     worried. The flesh is the part to eat, and the pips are best left in the core."
        .to_string()
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.fruit:_a_pip_to_eat:424", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_a_pip_to_eat(m, u))),
    ("bot.knowledge.fruit:_bruise_reply:218", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_bruise_reply(m, u))),
    ("bot.knowledge.fruit:_fruit_prep:130", |_: &FnRef, m: &mut Memory, u: &Understood| _fruit_prep(m, u)),
    ("bot.knowledge.fruit:_growing_from_a_pip:344", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _growing_from_a_pip(m, u)
    }),
    ("bot.knowledge.fruit:_skin_reply:176", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_skin_reply(m, u))),
];
