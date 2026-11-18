//! Port of PyBot's `bot/knowledge/kitchen_prep.py`: whether a prep step (sift, peel, prick,
//! score, blanch, parboil, rinse, pat dry, trim) is needed.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn key(name: &str) -> String {
    format!("bot.knowledge.kitchen_prep.{name}")
}
static _FOOD: LazyLock<&'static Re> = LazyLock::new(|| data::re(&key("_FOOD")));
static _NOT_FOOD: LazyLock<&'static Re> = LazyLock::new(|| data::re(&key("_NOT_FOOD")));
static _BLANCHABLE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&key("_BLANCHABLE")));
static _FREEZING: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&key("_FREEZING")));
static _STEP_VERBS: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| data::get(&key("_STEP_VERBS")).items().iter().map(|v| v.strs()).collect());
const POTATO_WORDS: &[&str] = &["potato", "potatoes", "chips", "chip", "fries"];

/// The line, lower-cased and padded with spaces (clean, else text, else the words).
pub fn _said(understood: &Understood) -> String {
    let text = sh::clean_or_text(understood);
    if text.is_empty() {
        return format!(" {} ", understood.words.join(" ").to_lowercase());
    }
    format!(" {} ", text.to_lowercase())
}

/// Whether the line asks if a prep step on a food is needed ("do I need to", "is it worth").
pub fn _asks_whether_the_step_is_needed(understood: &Understood) -> bool {
    let said = _said(understood);
    if _NOT_FOOD.is_match(&said) || !_FOOD.is_match(&said) {
        return false;
    }
    ["need", "have to", "has to", "should", "must", "necessary", "worth", "point", "bother", "essential", "important"]
        .iter()
        .any(|w| said.contains(w))
}

/// Whether the line asks how to do a prep step on a food, or whether it is needed.
pub fn _how_or_whether(understood: &Understood) -> bool {
    if _asks_whether_the_step_is_needed(understood) {
        return true;
    }
    let said = _said(understood);
    if _NOT_FOOD.is_match(&said) || !_FOOD.is_match(&said) {
        return false;
    }
    ["how do i", "how to ", "how should i", "how do you"].iter().any(|p| said.contains(p))
}

/// Whether the line asks if patting something dry is needed.
pub fn _patting_it_dry(understood: &Understood) -> bool {
    _asks_whether_the_step_is_needed(understood) && _said(understood).contains(" dry")
}

/// Whether the person's previous line named a prep step on a food.
pub fn _a_step_the_turn_before(memory: Option<&Memory>) -> bool {
    let earlier = memory.map(sh::last_user).unwrap_or_default();
    let earlier = format!(" {} ", text::squash(&earlier).to_lowercase());
    if text::strip(&earlier).is_empty() || _NOT_FOOD.is_match(&earlier) {
        return false;
    }
    if !_FOOD.is_match(&earlier) {
        return false;
    }
    _STEP_VERBS.iter().any(|forms| _says(&earlier, forms))
}

/// Whether the line asks about freezing vegetables (the blanching question), not chips.
pub fn _a_freezing_question(understood: &Understood) -> bool {
    let said = _said(understood);
    if _NOT_FOOD.is_match(&said) || !_says(&said, &_FREEZING) {
        return false;
    }
    if _says(&said, POTATO_WORDS) {
        return false;
    }
    if _BLANCHABLE.is_match(&said) {
        return true;
    }
    !_FOOD.is_match(&said) && [" if ", " and ", "what about", "then ", "when "].iter().any(|p| said.contains(p))
}

/// The blanch row's guard: a "do I need to" line, or a freezing question.
pub fn _blanching_or_freezing(understood: &Understood) -> bool {
    _asks_whether_the_step_is_needed(understood) || _a_freezing_question(understood)
}

/// Whether any of the words is in the text as a word (`\bword\b`; the words are patterns).
pub fn _says<S: AsRef<str>>(said: &str, words: &[S]) -> bool {
    words.iter().any(|w| sh::re_search(&format!(r"\b{}\b", w.as_ref()), said, 0))
}

/// Whether to blanch: before freezing vegetables (not chips), nuts to skin them, chips before
/// frying, tomatoes to peel, or greens to keep for later.
pub fn _blanch_reply(memory: &Memory, understood: &Understood) -> Option<String> {
    let said = _said(understood);
    if _says(&said, &_FREEZING) {
        if _says(&said, POTATO_WORDS) {
            return None;
        }
        if !(_BLANCHABLE.is_match(&said) || _a_step_the_turn_before(Some(memory))) {
            return None;
        }
        return Some("Yes before freezing: two minutes in boiling water, then straight into iced water, dry them and bag them. That is what stops the enzymes that turn them grey and soft, so they keep about a year instead of going limp and grassy in a couple of months, and they go into the pan straight from frozen. Cooked leftover vegetables are fine to freeze without blanching.".into());
    }
    if _says(&said, &["almond", "almonds", "nut", "nuts", "hazelnut", "hazelnuts", "walnut", "walnuts", "pistachio", "pistachios"]) {
        return Some("Only if you want the skins off: a minute in boiling water and then straight into cold, and each one slips out of its skin between your fingers. For flaked or ground almonds there is nothing to do.".into());
    }
    if _says(&said, POTATO_WORDS) {
        return Some("For chips, yes: three minutes in boiling water, then drain and dry them completely before frying. That softens the outside, and the roughened surface is what turns crisp.".into());
    }
    if _says(&said, &["tomato", "tomatoes"]) {
        return Some("That is the peeling step: 30 seconds in boiling water and straight into cold, and the skins slip off. Nothing else needs it.".into());
    }
    Some("Only if you are eating them later: two minutes in boiling salted water and then straight into iced water keeps green vegetables bright and just-cooked for a salad or a reheat. Into the freezer, it is the step that matters most - a minute or two blanched, then dried and bagged, and it keeps a year instead of going limp and grassy. Eaten straight away, steam them instead - one pan, and they keep more of the vitamins.".into())
}

/// The _sift_reply card for the food the line names.
pub fn _sift_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = _said(understood);
    if _says(&said, &["icing sugar", "powdered sugar", "icing"]) {
        return "Yes: icing sugar clumps as soon as the bag is open, and a lump shows in icing or sits in a dusting like grit. A sieve is quicker than a whisk for a smooth glacé icing, and a fine one gives the even dusting you want on a cake.".into();
    }
    if _says(&said, &["cocoa"]) {
        return "Yes, always: cocoa is the one that clumps hardest, and lumps will not beat out of a batter or a sauce. Sift it with the flour or whisk it into hot water first.".into();
    }
    "Not from a fresh bag: modern plain and self-raising flour are already fine and loose. Sift it if the bag has been open for months, if there are lumps, or if you are making a light sponge or folding flour into beaten egg whites - the air sifting adds is what makes those rise. Cocoa powder always wants a sieve.".into()
}

/// The _peel_reply card for the food the line names.
pub fn _peel_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = _said(understood);
    if _says(&said, &["mash", "mashed"]) {
        return "Not for mash: scrub them and leave the skin on if you like the flecks - most of the fibre sits there - and peel after boiling if you want the mash white and smooth. Peeling raw potatoes takes a third of the flesh off with the skin, so if you peel first, boil them in their own water.".into();
    }
    if _says(&said, &["roast", "roasting", "roastie", "roasties", "oven"]) {
        return "Yes for roast potatoes: peel them, cut them the same size, parboil 8 minutes, drain, shake in the pan to rough up the edges, then roast at 200C (180C fan) in hot fat for 40 to 45 minutes. Skins left on make a good, quicker wedge - just not a crisp roastie.".into();
    }
    if _says(&said, &["boil", "boiling", "boiled", "salad", "salad"]) {
        return "Not unless you want them white: scrub new potatoes and boil them with the skins on - that is where the fibre and a good part of the flavour sit - and peel only for mash, for a smooth soup, or if the skin has gone green or damaged.".into();
    }
    if _says(&said, &["butternut", "squash", "pumpkin"]) {
        return "Only if you want it in cubes: roast the halves cut side down in their skins - the flesh scoops out clean - but peel it for a soup or a tray of cubes, and mind your hands, because the skin is hard and slippy.".into();
    }
    if _says(&said, &["beetroot"]) {
        return "Roast them whole in their skins and the skin slips off afterwards: foil at 200C (180C fan) for an hour, then rub it away with kitchen paper. Peel raw beetroot only if you are grating it raw - but wear gloves, because it stains.".into();
    }
    if _says(&said, &["sweet potato", "sweet potatoes"]) {
        return "No need: the skin of a sweet potato is thin, it crisps in the oven, and it is where much of the fibre sits - scrub it and roast or bake it whole. Peel it only for a mash or a smooth soup.".into();
    }
    if _says(&said, &["cucumber", "cucumbers"]) {
        return "Only if the skin is thick, waxy or bitter at the stem end: a supermarket cucumber is fine unpeeled, and scoring it with a fork lengthways gives the ridged look without losing anything.".into();
    }
    if _says(&said, &["carrot", "carrots", "parsnip", "parsnips", "swede", "turnip"]) {
        return "Only if they are old: a young carrot or parsnip wants a scrub with a brush, while an old one tastes soapy and woody with the skin on. Peel a thin layer rather than down to the core, because the sweetest part is just under it.".into();
    }
    if _says(&said, &["tomato", "tomatoes"]) {
        return "Only for a smooth sauce or a clear soup: cut a small cross in the base, drop them in boiling water for 30 seconds and then into cold, and the skins slip off. For a chunky sauce or a tray of roasted tomatoes leave them on - they soften and disappear anyway.".into();
    }
    "Only if the skin is thick, muddy or bitter. Scrubbing does the same job on anything thin-skinned and a good part of the flavour sits right under it, so peel as thinly as you can when you do.".into()
}

/// The _prick_reply card for the food the line names.
pub fn _prick_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = _said(understood);
    if _says(&said, &["potato", "potatoes", "jacket", "jackets", "spud", "spuds"]) {
        return "Yes, 4 or 5 deep pricks with a fork: without them the steam has nowhere to go and a jacket potato bursts in the oven. Rub the skin with oil and salt first and it crisps while it bakes.".into();
    }
    "Yes for anything with a skin going into a hot oven or a microwave - a tomato, a potato, an aubergine - so the steam has somewhere to go instead of splitting it open.".into()
}

/// The _score_reply card for the food the line names.
pub fn _score_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = _said(understood);
    if _says(&said, &["pork", "crackling", "belly", "shoulder"]) {
        return "Yes for crackling: score the skin about a centimetre deep in a diamond pattern, dry it well, rub in plenty of salt and start the joint hot. The scores are what let the fat render out and the skin bubble.".into();
    }
    if _says(&said, &["bread", "dough", "loaf", "sourdough"]) {
        return "Yes: one slash about a centimetre deep just before it goes in the oven lets the loaf open where you chose rather than where it wants to. A wet blade and a quick, confident cut are what make it clean.".into();
    }
    if _says(&said, &["sausage", "sausages", "chipolata", "chipolatas", "black pudding"]) {
        return "Not scored, and usually not even pricked: a good butcher's sausage pricked loses its juice, so cook it gently instead - a moderate pan or a 180C oven - and it will not split. A cheap, tight-skinned one splits whatever you do to it.".into();
    }
    if _says(&said, &["fish", "salmon", "cod", "bass", "trout", "skin"]) {
        return "Yes for a crisp pan skin: three shallow slashes through the skin stop the fillet curling and let the heat through. Score it before it goes in the pan, and dry the skin well.".into();
    }
    "Yes when the skin has to let something out: a joint of pork for crackling, a loaf that has to open where you want it, a fish fillet that would curl. Score about a centimetre deep and dry the surface first.".into()
}

/// The _parboil_reply card for the food the line names.
pub fn _parboil_reply(_memory: &Memory, _understood: &Understood) -> String {
    "Yes for roast potatoes and chips: about 8 minutes in boiling salted water until a knife goes in and the outside gives, then drain and shake them in the pan to roughen the edges, and roast at 200C (180C fan) in hot fat for 40 to 45 minutes. The roughened edges are what go crisp. Potatoes for mashing need the full boil, not this.".into()
}

/// The _rinse_reply card for the food the line names.
pub fn _rinse_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = _said(understood);
    if _says(&said, &["quinoa"]) {
        return "Yes, quinoa wants it: 30 seconds under cold water in a sieve takes off the saponin on the surface, which tastes soapy and bitter. It is the one grain that really needs it - and toast it dry in the pan afterwards and it smells nutty.".into();
    }
    if _says(&said, &["barley", "pearl barley", "bulgur", "couscous", "lentil", "lentils"]) {
        return "A quick rinse is enough, and it is mostly to check for grit and bits of husk: run cold water through it in a sieve until it runs clear. No soaking, no scrubbing - that is for dried beans, and it is a different question.".into();
    }
    "A quick rinse under cold water is enough for most grains and pulses - it clears out dust and loose husk, and takes no more than a few seconds in a sieve. Rice is the one where rinsing is a real choice: rinse for separate grains, leave it for a risotto or a rice pudding.".into()
}

/// The _pat_dry_reply card for the food the line names.
pub fn _pat_dry_reply(_memory: &Memory, _understood: &Understood) -> String {
    "Yes, and it is the step people skip: a wet surface steams instead of browning, so pat meat, fish and potatoes dry with kitchen paper and salt them just before they go in the pan. It matters most for a steak, a chicken breast with the skin on and anything going into a hot oven.".into()
}

/// The _trim_reply card for the food the line names.
pub fn _trim_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = _said(understood);
    if _says(&said, &["steak", "beef", "pork", "lamb", "chicken", "meat", "joint", "gammon"]) {
        return "Only if you want it leaner: a rim of fat bastes the meat and is what you are paying for, so leave it on and cut it away on the plate. Silverskin - the shiny, tough membrane - is the one that must come off, or the meat curls.".into();
    }
    if _says(&said, &["asparagus"]) {
        return "Yes, and the woody end knows where it is: bend each spear until it snaps, and throw the stub away - it is fibrous and never softens. Do not cut them all at the same length by eye, or you will keep the tough part on the thin spears.".into();
    }
    if _says(&said, &["leek", "leeks"]) {
        return "Yes: cut off the dark green top for stock, and slit the leek lengthways down the white so the grit between the layers can rinse out. A leek that looks clean is usually the one with sand in it.".into();
    }
    if _says(&said, &["celery"]) {
        return "A rinse and a trim of the base is enough. If the strings bother you, peel them off the outside of the stalk with a vegetable peeler - the flavour is in the stalk, not the strings.".into();
    }
    "Only the tough bits: the stalk end of a bean, the woody end of an asparagus spear, the base of a sprout, the stringy root of a spring onion. Trimming more than that wastes the vegetable, and most of what people cut away cooks down to nothing anyway.".into()
}

fn g_needed(_f: &FnRef, u: &Understood) -> bool {
    _asks_whether_the_step_is_needed(u)
}
fn g_how(_f: &FnRef, u: &Understood) -> bool {
    _how_or_whether(u)
}
fn g_pat(_f: &FnRef, u: &Understood) -> bool {
    _patting_it_dry(u)
}
fn g_blanch(_f: &FnRef, u: &Understood) -> bool {
    _blanching_or_freezing(u)
}
fn r_blanch(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _blanch_reply(m, u)
}
macro_rules! reply {
    ($name:ident, $f:ident) => {
        fn $name(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
            Some($f(m, u))
        }
    };
}
reply!(r_sift, _sift_reply);
reply!(r_peel, _peel_reply);
reply!(r_prick, _prick_reply);
reply!(r_score, _score_reply);
reply!(r_parboil, _parboil_reply);
reply!(r_rinse, _rinse_reply);
reply!(r_pat, _pat_dry_reply);
reply!(r_trim, _trim_reply);

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.kitchen_prep:_asks_whether_the_step_is_needed:121", g_needed),
    ("bot.knowledge.kitchen_prep:_how_or_whether:133", g_how),
    ("bot.knowledge.kitchen_prep:_patting_it_dry:145", g_pat),
    ("bot.knowledge.kitchen_prep:_blanching_or_freezing:188", g_blanch),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.kitchen_prep:_sift_reply:198", r_sift),
    ("bot.knowledge.kitchen_prep:_peel_reply:213", r_peel),
    ("bot.knowledge.kitchen_prep:_prick_reply:259", r_prick),
    ("bot.knowledge.kitchen_prep:_score_reply:270", r_score),
    ("bot.knowledge.kitchen_prep:_blanch_reply:293", r_blanch),
    ("bot.knowledge.kitchen_prep:_parboil_reply:331", r_parboil),
    ("bot.knowledge.kitchen_prep:_rinse_reply:339", r_rinse),
    ("bot.knowledge.kitchen_prep:_pat_dry_reply:355", r_pat),
    ("bot.knowledge.kitchen_prep:_trim_reply:362", r_trim),
];
