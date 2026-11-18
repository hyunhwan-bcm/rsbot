//! Port of `bot/knowledge/what_damages_what.py`: what a cleaner or a machine does to a material -
//! bleach on bedding, an acid on stone, and what cannot go in the dishwasher.

use crate::data;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

const P: &str = "bot.knowledge.what_damages_what.";

fn words(name: &str) -> Vec<String> {
    data::get(&format!("{P}{name}")).strs()
}

/// `re.search(r"\b%s\b" % re.escape(word), ...)`, compiled once per word.
fn word_re(word: &str) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<String, &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(r) = cache.get(word) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(&format!(r"\b{}\b", pyre::escape(word)), 0)));
    cache.insert(word.to_string(), r);
    r
}

/// Whether the text names one of these words, whole.
pub fn _names<S: AsRef<str>>(text_: &str, words: &[S]) -> bool {
    words.iter().any(|w| word_re(w.as_ref()).is_match(text_))
}

/// Bleach on a mattress, a duvet or bedding: no, and what to use instead.
pub fn _bleach_on_bedding(_memory: &Memory, understood: &Understood) -> String {
    let said = text::lower(&understood.text);
    if _names(&said, &words("_MATTRESS")) {
        return "No: the cover yellows and the foam keeps the smell of it, and bleach does not reach the sweat and the mites you are after. Vacuum both sides, dust it with bicarbonate of soda for an hour and vacuum again, and dab a stain with cool water and a little washing-up liquid. Air it with the bed stripped, and a washable protector saves the next one.".into();
    }
    "Not neat bleach: the cotton weakens and goes yellow, and a synthetic duvet or pillow clogs and holds the smell of it. A white cotton cover takes a laundry bleach in a hot wash at the label's temperature; anything else wants detergent at 60 and a long dry. Pillows and duvets go in a big drum with a couple of towels to balance it.".into()
}

/// An acid (vinegar, lemon, descaler) on a stone surface: which stones it etches.
pub fn _acid_on_stone(_memory: &Memory, understood: &Understood) -> String {
    let said = format!(" {} ", text::lower(&understood.text));
    let acid = if _names(&said, &["descaler", "descaling", "limescale remover"]) {
        "descaler"
    } else if _names(&said, &["lemon juice", "lemon"]) {
        "lemon juice"
    } else {
        "vinegar"
    };
    if _names(&said, &words("_HARD_STONE")) {
        return format!(
            "That one stands it: quartz and a sealed granite or slate surface take an acid splash \u{2014} {acid} \u{2014} so wipe it off rather than leave it standing. Marble, limestone, travertine and onyx are the ones that etch, and the dull patch they leave never comes back. Warm water and a drop of washing-up liquid is all any of them needs."
        );
    }
    format!(
        "That one etches: acid dulls a marble, limestone, travertine or onyx surface and the patch never comes back, so a wipe with {acid} is the wrong cleaner. Granite and quartz stand one now and then but do not need it. Warm water with a drop of washing-up liquid, dried off, and a pH-neutral stone cleaner for the rest."
    )
}

/// The reason the thing in the person's hand cannot go in the dishwasher, by its material.
pub fn _what_the_machine_ruins(_memory: &Memory, understood: &Understood) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let said = format!(" {} ", text::lower(line));
    let named = |name: &str| _names(&said, &words(name));
    if named("_IRON") {
        return "Not a seasoned pan: the wash strips the seasoning and the heat dries it to rust, so it comes out grey and sticks. Scrub it with hot water and a stiff brush while it is warm, dry it on the hob, and wipe a little oil over it. Stainless steel and glass are fine in the machine.".into();
    }
    if named("_WOOD") {
        return "Not a wooden one: it swells, splits along the grain and the glue in a board lets go, and a wooden handle soon feels rough. Wash it by hand in hot soapy water, stand it up to dry rather than flat, and rub food-safe oil into a board or a spoon now and then. Plastic boards are fine in the machine.".into();
    }
    if named("_KNIFE") {
        return "Not a knife: the edge knocks against the basket and the other cutlery, the handle can swell or lift, and the point comes out blunt. Wash it by hand in hot soapy water, dry it at once, and put it back in a block or on a magnetic strip. Stainless steel and everyday china are fine in the machine.".into();
    }
    if named("_SOFT_METAL") {
        return "Not bare aluminium or copper: the salt and the drying cycle leave it grey, pitted and chalky, and a tinned baking tray comes up patchy. By hand in hot soapy water is all it needs, and dry it at once. Stainless steel, glass and china are fine in the machine.".into();
    }
    if named("_TRAPPED") {
        return "Not that one: the water gets into a space it cannot drain out of, and it comes out rusty, chalky or sour, or with the soap still in it. Rinse it by hand in hot water and leave it to drain upside down. Plain crockery, stainless steel, glass and plastic boards are fine in the machine.".into();
    }
    if named("_STONEWARE") {
        return "Not stone: the detergent and the drying cycle take the surface off a marble mortar or a slate board and leave it dull and chalky, and a pizza stone soaks up water and can split in the oven next time. Rinse it in warm water and a stiff brush, no soap if you can, and dry it slowly. Plain crockery, stainless steel and glass are fine in the machine.".into();
    }
    if named("_DELICATE") {
        return "Not that: crystal clouds and can chip, a gold rim or a printed pattern comes off in the hot water, and anything with a trapped space holds water that rusts or smells. Wash it by hand in warm water with a drop of washing-up liquid and dry it with a cloth. Plain glass, stainless steel and everyday china are fine in the machine.".into();
    }
    if named("_COATING") {
        return "Not a coated one: the hot water and the detergent dull the coating and it starts to flake, and the same goes for an air fryer basket or a rice cooker bowl. Wash it with the soft side of a sponge and dry it before it goes back. Stainless steel, glass and china are fine in the machine.".into();
    }
    "Look for the dishwasher-safe mark on the base or the handle first, and wash by hand anything without one: wood swells, thin metal and a sharp edge suffer, and china with a gold rim loses it. Stainless steel, glass and everyday china are fine in the machine.".into()
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.what_damages_what:_bleach_on_bedding:28", |_, m, u| Some(_bleach_on_bedding(m, u))),
    ("bot.knowledge.what_damages_what:_acid_on_stone:158", |_, m, u| Some(_acid_on_stone(m, u))),
    ("bot.knowledge.what_damages_what:_what_the_machine_ruins:178", |_, m, u| Some(_what_the_machine_ruins(m, u))),
];
