//! Port of `bot/knowledge/cleaning.py`: cleaning, laundry and stains. The entries and their
//! tables are data in the bundle; these are the guards and replies (and `landed_on_a_thing` /
//! `not_a_mark_on_a_thing`, which the food rows elsewhere use too).

use crate::data;
use crate::data::FnRef;
use crate::knowledge::d2_shared::{food_has_gone_off, the_line, the_thing_has_gone};
use crate::knowledge::d2dup_g8::has_bounded_word;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! g {
    ($t:ty, $name:ident, $f:path) => {
        static $name: LazyLock<$t> = LazyLock::new(|| $f(concat!("bot.knowledge.cleaning.", stringify!($name))));
    };
}

g!(Vec<String>, A_MARK_WORD, data::strs);
g!(Vec<String>, A_STATE_OF_A_SURFACE, data::strs);
g!(String, BATH_TOWELS, data::string);
g!(String, BBQ_MAIN, data::string);
g!(String, BBQ_RUST, data::string);
g!(String, BBQ_WINTER, data::string);
g!(String, BEDSHEETS_OFTEN, data::string);
g!(Vec<String>, CHEMICAL_LANDED, data::strs);
g!(Vec<String>, COMES_OFF, data::strs);
g!(String, CUPBOARD_CLEAN, data::string);
g!(String, CUPBOARD_FULL, data::string);
g!(String, CUPBOARD_MOTHS, data::string);
g!(Vec<String>, CUPBOARD_PESTS, data::strs);
g!(Vec<String>, CUPBOARD_STATE, data::strs);
g!(Vec<String>, DUST_METHOD_ASK, data::strs);
g!(String, FIRST_WASH, data::string);
g!(String, FOG_FIX, data::string);
g!(String, FOG_REASON, data::string);
g!(HashSet<String>, FOOD_NAMED, data::set);
g!(&'static Re, LANDED_ON, data::re);
g!(HashSet<String>, LANDED_ON_THINGS, data::set);
g!(Vec<String>, MARK_ON_THE_PERSON, data::strs);
g!(&'static Re, NOT_A_CLEANING_ASK, data::re);
g!(Vec<String>, NOT_A_SKIN_MARK, data::strs);
g!(&'static Re, OFF_COMES_OFF, data::re);
g!(String, PET_ACCIDENT_REPLY, data::string);
g!(&'static Re, QUESTION_SHAPE, data::re);
g!(&'static Re, _AN_ACCIDENT, data::re);
g!(Vec<String>, _A_DRINK, data::strs);
g!(&'static Re, _A_DRYER, data::re);
g!(&'static Re, _A_HURTING_JOINT, data::re);
g!(&'static Re, _A_MARK_NAMED, data::re);
g!(&'static Re, _A_PLACE, data::re);
g!(&'static Re, _A_PLACE_TO_WASH, data::re);
g!(&'static Re, _A_PRONOUN_BEFORE_OFF, data::re);
g!(Vec<String>, _A_SCREEN, data::strs);
g!(Vec<String>, _A_VESSEL, data::strs);
g!(&'static Re, _A_WASHING_MACHINE, data::re);
g!(&'static Re, _A_WEAR_WORD, data::re);
g!(Vec<String>, _A_WINDOW_NAMED, data::strs);
g!(&'static Re, _BIN_IS_A_VERB, data::re);
g!(&'static Re, _BODY_JOINT, data::re);
g!(&'static Re, _DUST_IS_ARRIVING, data::re);
g!(&'static Re, _DUST_NOUN, data::re);
g!(&'static Re, _DUST_ON_A_THING, data::re);
g!(Vec<String>, _GLASSWARE, data::strs);
g!(&'static Re, _PET_NAMED, data::re);
g!(Vec<String>, _ROOM_WITH_ITS_OWN_CARD, data::strs);
g!(&'static Re, _SEAL_COST, data::re);
g!(&'static Re, _SEAL_THINGS, data::re);
g!(&'static Re, _SEAL_WHEN, data::re);
g!(&'static Re, _SEAL_WHERE, data::re);
g!(&'static Re, _SEAL_WORTH, data::re);
g!(&'static Re, _STEAM_A_SURFACE, data::re);
g!(&'static Re, _STEAM_LOOK_AFTER, data::re);
g!(&'static Re, _STEAM_RISKY, data::re);
g!(Vec<String>, _STEP_OBJECTOWN, data::strs);
g!(&'static Re, _THE_GARMENT, data::re);
g!(&'static Re, _THE_LOAF_WENT_OFF, data::re);
g!(Vec<String>, _WET_GLASS, data::strs);
g!(Vec<String>, _WIPE_THE_GLASS, data::strs);

/// `_STEP_WORDS`: the step words (dict keys, in order).
static _STEP_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| {
    data::get("bot.knowledge.cleaning._STEP_WORDS").pairs().iter().map(|(k, _)| k.str().to_string()).collect()
});

/// `_SOFT_STAIN_WORDS`: (words, reply) per family of mark, in order.
static _SOFT_STAIN_WORDS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cleaning._SOFT_STAIN_WORDS")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
});

static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

fn any_in(text: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|p| text.contains(p.as_str()))
}

fn any_of(text: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|p| text.contains(p))
}

fn or<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.is_empty() { b } else { a }
}

/// True unless the reading names a room or a fixture that has its own card.
pub fn _not_a_room_with_its_own_card(understood: &Understood) -> bool {
    let words = &understood.words;
    if words.is_empty() {
        return true;
    }
    let lowered: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();
    let said: HashSet<&str> = lowered.iter().map(String::as_str).collect();
    let line = format!(" {} {} ", lowered.join(" "), or(&understood.plain, &understood.text).to_lowercase());
    for word in _ROOM_WITH_ITS_OWN_CARD.iter() {
        if word.contains(' ') {
            if line.contains(word.as_str()) {
                return false;
            }
        } else if said.contains(word.as_str()) {
            return false;
        }
    }
    true
}

/// A capful of thin bleach in a drain is fine; it will not clear a blockage.
pub fn _bleach_down_the_drain(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = or(&understood.clean, &understood.text).to_lowercase();
    if any_of(&text, &["septic", "tank", "soakaway", "cesspit"]) {
        return Some("Not with a septic tank: the bleach kills the bacteria the tank works with, and a capful now and then is the most it will take. Hot water and a plunger clear the blockage, and a bucket of water with a spoonful of bicarbonate of soda and vinegar freshens the drain without the chlorine.".into());
    }
    Some("A capful of thin bleach in the drain once a week is fine and kills the smell, but it will not clear a blockage - hot water and a plunger do that, and mixing bleach with an acid or ammonia cleaner makes chlorine gas, so never two in the same drain on the same day. Regular bleach on old metal pipework and rubber seals does shorten their life, so keep it to a capful rather than a mugful.".into())
}

/// False for dust on a named thing - a lens, a monitor, a dashboard, a keyboard.
pub fn _dust_is_on_a_thing(understood: &Understood) -> bool {
    let clean = understood.clean.to_lowercase();
    if !_DUST_NOUN.is_match(&clean) {
        return true;
    }
    if _DUST_IS_ARRIVING.is_match(&clean) {
        return true;
    }
    for m in _DUST_ON_A_THING.finditer(&clean) {
        let tail = text::strip(m.str(1));
        if !_A_PLACE.is_match(tail) {
            return false;
        }
    }
    true
}

/// False when the line's worn thing is a joint rather than a garment (`mend-hole`).
pub fn a_worn_thing_is_a_garment(understood: &Understood) -> bool {
    let said = or(&understood.clean, &understood.text).to_lowercase();
    if !_BODY_JOINT.is_match(&said) {
        return true;
    }
    if _A_WEAR_WORD.is_match(&said) && !_THE_GARMENT.is_match(&said) {
        return false;
    }
    if _A_HURTING_JOINT.is_match(&said) {
        return false;
    }
    true
}

/// What a steam cleaner is good for, or how to look after one.
pub fn _steam_cleaner(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let joined = understood.words.join(" ");
    let text = or(or(&understood.clean, &understood.text), &joined);
    if _STEAM_RISKY.is_match(text) {
        return Some("Skip it on that one: the heat lifts laminate and engineered boards, opens the seams of unsealed or oiled wood, and drives into a carpet's backing or a sofa's filling instead of out of it, so it comes back damp and can smell. Use it on tiles, grout, hard floors, the shower and the hob - about four seconds with the head on one spot lifts the grease with no chemicals, so move slowly rather than scrubbing. A carpet wants a hire machine once a year, and a wooden floor wants a barely damp mop. Empty the tank after every go and descale it every few months with half water, half white vinegar.".into());
    }
    if _STEAM_LOOK_AFTER.is_match(text) && !_STEAM_A_SURFACE.is_match(text) {
        return Some("Empty the tank after every go and descale it every few months: half water, half white vinegar in the tank for 20 minutes, then rinse. Rinse the pads or the cloth and dry them rather than leaving them damp in the cupboard, and wipe the head and the nozzles with a vinegar-soaked cloth if the steam starts coming out unevenly. Beyond that the machine looks after itself - the water is the cleaning, so the tank and the scale are the whole of the maintenance.".into());
    }
    Some("Hot steam lifts grease and kills most bacteria on tiles, grout, hard floors, the shower and the hob, with no chemicals - about four seconds with the head on one spot is what does it, so move slowly rather than scrubbing. Skip laminate, engineered and unsealed wood and anything with a gap the water can get under, because the heat lifts the surface; skip carpets and upholstery, where a hire machine once a year is cheaper; and keep it off cold glass, which can crack. Empty the tank after every go, and descale it every few months with half water, half white vinegar for 20 minutes.".into())
}

/// One uniform, several questions: wash, stain, whiteness, shrinking, ironing, smell.
pub fn _uniform_care(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = or(&understood.text, &understood.clean).to_lowercase();
    if any_of(&text, &["name tag", "name tags", "name tape", "nametape", "label", "labels", "tag", "tags", "coming off", "comes off"]) {
        return Some("Sew it on rather than ironing it on: a few stitches through the loop of the label and into the inside seam hold through every wash, and an iron-on lifts at the first 40 cycle. Mark the care label inside the collar with a fabric pen and the uniform still comes back if the sewn tag does.".into());
    }
    if any_of(&text, &["tie", "ties"]) {
        return Some("Sponge it instead of washing it: a drop of washing-up liquid on a damp cloth, dab the mark, then hang the tie flat to dry on the back of a chair, and a clip-on tie is the answer to September mornings. Silk and wool ties go to the dry cleaner, and an iron on the front takes the sheen off it for good.".into());
    }
    if any_of(&text, &[
        "30 or 40", "40 or 30", "40 or 60", "60 or 40", "what temperature", "which temperature", "how hot", "what temp",
        "warmer", "hottest", "cooler", "at 60", "at 90", "60 degree", "90 degree", "hot wash", "wash hot", "boil wash",
        "boiling",
    ]) {
        return Some("40 is the setting for uniform: polyester-cotton comes out of a 40 wash clean, and 60 is what fades the colours and sets the creases in. 30 is fine for a normal week's wear and about 10p a load cheaper, but take a stain or a bug going round to 40 or 60 and wash the whole uniform the same way.".into());
    }
    if any_of(&text, &["brand new", "first wear", "before wearing", "before wear", "new uniform", "new school uniform", "just bought"]) {
        return Some("Yes, wash it once before the first wear: new uniform carries dressing and loose dye that can mark the white collar and irritate skin. A plain 40 cycle, no fabric softener, dry it in the air, and wash the dark pieces on their own the first time round.".into());
    }
    if any_of(&text, &["shrank", "shrunk", "shrink", "too small", "came out small"]) {
        return Some("Soak it in lukewarm water with a spoonful of hair conditioner for 15 minutes and stretch it gently back into shape, then dry it flat: cotton-polyester goes back a little and never fully. Warmth and a wash were what shrank it, so keep the next one off the tumble dryer.".into());
    }
    if any_of(&text, &["white", "whiten", "bleach", "grey", "gray", "yellow"]) {
        return Some("Wash white uniform at 40 with a scoop of oxygen bleach (a colour-safe one, not thin bleach) and dry it in daylight: oxygen bleach lifts the grey without eating the fabric, and daylight does what no product does. Wash it inside out and never let black or navy sit damp against it.".into());
    }
    if any_of(&text, &["mud", "muddy", "muddy puddle", "clay", "soil"]) {
        return Some("Let it dry completely, then brush it off with a stiff brush before any water touches it: washing mud while it is wet pushes it into the weave. Then work a drop of washing-up liquid into what is left, wash it at 40 and air-dry it, and keep it off the radiator.".into());
    }
    if any_of(&text, &["stain", "ink", "paint", "mark", "pen"]) {
        return Some("Blot it, do not rub, then work a little washing-up liquid into the mark with your fingers and rinse from the back in cold water before the wash. Wash it at 40 with a scoop of oxygen bleach, and do not tumble it dry while it is still damp, because the dryer sets whatever the wash leaves behind.".into());
    }
    if any_of(&text, &["iron", "crease", "creases", "crinkle"]) {
        return Some("Iron it warm, not hot, under a damp tea towel: polyester scorches and goes shiny at a cotton setting, and the cloth on top is what stops the shine. Wash it at 40, shake it out and hang it straight, and most of it needs no iron at all.".into());
    }
    if any_of(&text, &["smell", "smelly", "stinks", "stink", "odour", "odor", "sweat"]) {
        return Some("Wash it at 40 with a couple of spoonfuls of bicarbonate of soda in the drum and dry it in the air: school uniform smells are trapped sweat or damp, and soda lifts the sweat without a strong detergent. Skip the fabric softener, which holds the smell in, and never leave a damp uniform in a bag or a heap.".into());
    }
    Some("Wash uniform at 40 on a normal cycle and air-dry it: polyester-cotton comes clean at 40, fades in a hot wash and shrinks in the dryer. Treat ink, mud and food before washing with cold water and washing-up liquid on the mark, iron it low under a tea towel, and keep whites white with oxygen bleach rather than thin bleach.".into())
}

/// Why spectacles fog and how to stop it, the reason first when the line asks why.
pub fn _spectacles_fog(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if understood.asks.iter().any(|a| a == "why") {
        return Some(format!("{} {}", *FOG_REASON, *FOG_FIX));
    }
    let mut chars = FOG_REASON.chars();
    let first = chars.next().map(|c| c.to_lowercase().to_string()).unwrap_or_default();
    Some(format!("{} That is because {}{}", *FOG_FIX, first, chars.as_str()))
}

/// Cupboards: moths, a full cupboard, or the plain clean.
pub fn _cupboards(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = or(&understood.clean, &understood.text).to_lowercase();
    if any_in(&text, &CUPBOARD_PESTS) {
        return Some(CUPBOARD_MOTHS.clone());
    }
    if any_in(&text, &CUPBOARD_STATE) {
        return Some(CUPBOARD_FULL.clone());
    }
    Some(CUPBOARD_CLEAN.clone())
}

/// How long a pillow, duvet or mattress lasts, worded for the one the line names.
pub fn _bedding_life(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = or(&understood.text, &understood.clean).to_lowercase();
    if line.contains("pillow") {
        return Some("Two years is a normal life for a pillow and three or four for a good one, and you know it is finished when it no longer springs back: fold it in half, let go, and if it stays folded it has lost its loft. A flat pillow is one of the commonest causes of a stiff neck in the morning, so replace it rather than sleeping on it. Wash it to the label now and then, use a washable protector, and if you sleep on your side you want a fuller one than your partner does.".into());
    }
    Some("About five years is a normal life for a down duvet and two or three for hollowfibre. A lumpy one is filling that has clumped, usually after a wash, so tumble it on low with two or three dryer balls and shake it out holding two corners, and down comes back; if it stays flat and cold, or the filling is in hard clumps, it will not. Air it over the banister each time you change the bed.".into())
}

/// "Should i wash new sheets before using them" is the first wash, not the change rota.
pub fn _first_wash(reading: &Understood) -> String {
    let line = or(or(&reading.text, &reading.clean), &reading.plain).to_lowercase();
    if !line.contains("new") && !line.contains("brand new") {
        return String::new();
    }
    if line.contains("before") || line.contains("first use") || line.contains("first wash") {
        return FIRST_WASH.clone();
    }
    String::new()
}

/// How often to change the sheets, or the first wash of new ones.
pub fn _bedsheets(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _first_wash(understood);
    if !said.is_empty() {
        return Some(said);
    }
    Some(BEDSHEETS_OFTEN.clone())
}

/// How often to wash towels, or the first wash of new ones.
pub fn _bath_towels(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _first_wash(understood);
    if !said.is_empty() {
        return Some(said);
    }
    Some(BATH_TOWELS.clone())
}

/// `" %s " % word in said or word in said`, which is the same as `word in said`.
fn said_with(said: &str, words: &[String]) -> bool {
    words.iter().any(|w| said.contains(&format!(" {w} ")) || said.contains(w.as_str()))
}

/// False when the glass the line names is a pair of spectacles or a drinking glass.
pub fn _glass_in_the_line_is_a_window(understood: &Understood) -> bool {
    let said = format!(" {} ", understood.text.to_lowercase());
    if said_with(&said, &_A_WINDOW_NAMED) {
        return true;
    }
    for words in [&*_GLASSWARE, &*_A_DRINK, &*_A_VESSEL] {
        if said_with(&said, words) {
            return false;
        }
    }
    true
}

/// False when the line is about water on the glass rather than dirt on it.
pub fn _a_dirty_window_not_a_wet_one(understood: &Understood) -> bool {
    let said = format!(" {} ", understood.text.to_lowercase());
    if !said_with(&said, &_WET_GLASS) {
        return true;
    }
    any_in(&said, &_WIPE_THE_GLASS)
}

/// False when the line names the back of a screen, which its own card answers.
pub fn _the_back_of_a_screen(understood: &Understood) -> bool {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if words.is_empty() {
        return true;
    }
    let text = format!(" {} ", words.join(" "));
    if !(text.contains(" back of ") || text.contains(" back of the ")) {
        return true;
    }
    !_A_SCREEN.iter().any(|w| text.contains(&format!(" {w} ")))
}

/// The patch answer for a line that names only where the mess was.
pub fn _the_place_a_pet_used(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = or(&understood.clean, &understood.text).to_lowercase();
    if !_A_PLACE_TO_WASH.is_match(&text) {
        return None;
    }
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(3);
    for turn in turns[start..].iter().rev() {
        let line = turn.user.to_lowercase();
        if _PET_NAMED.is_match(&line) && _AN_ACCIDENT.is_match(&line) {
            return Some(PET_ACCIDENT_REPLY.clone());
        }
    }
    None
}

/// True when the line says the bread went off, rather than the bin it was kept in.
pub fn _the_loaf_went_off(understood: &Understood) -> bool {
    _THE_LOAF_WENT_OFF.is_match(&the_line(understood))
}

/// True when the "bin" in the line is a food being thrown away, not the bin in the kitchen.
pub fn _the_food_was_binned(understood: &Understood) -> bool {
    let said = the_line(understood);
    if !_BIN_IS_A_VERB.is_match(&said) {
        return false;
    }
    LETTERS.findall_str(&said.to_lowercase()).iter().any(|w| FOOD_NAMED.contains(w))
}

/// Whether the line is about a colour that came out in the wash rather than a thing that died.
pub fn _a_colour_can_have_run(understood: &Understood) -> bool {
    !the_thing_has_gone(understood)
}

/// True unless the line names a dryer and no washing machine with it.
pub fn _not_the_dryer_alone(understood: &Understood) -> bool {
    let said = the_line(understood);
    !(_A_DRYER.is_match(&said) && !_A_WASHING_MACHINE.is_match(&said))
}

/// Only a line that names two cleaning steps and asks which comes first.
pub fn _two_steps_in_order(understood: &Understood) -> bool {
    let text = or(&understood.text, &understood.clean).to_lowercase();
    if any_in(&text, &_STEP_OBJECTOWN) {
        return false;
    }
    _STEP_WORDS.iter().filter(|w| text.contains(w.as_str())).count() >= 2
}

/// Rust, winter, or the ordinary clean, whichever the line is about.
pub fn _clean_bbq(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = or(&understood.clean, &understood.text).to_lowercase();
    if said.contains("rust") || said.contains("rusty") {
        return Some(BBQ_RUST.clone());
    }
    if any_of(&said, &["winter", "store", "storing", "put it away", "cover", "covering", "end of the season"]) {
        return Some(BBQ_WINTER.clone());
    }
    Some(BBQ_MAIN.clone())
}

/// The method for the mark the line names, for a carpet, rug, sofa or mattress.
pub fn _a_mark_on_a_soft_surface(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = format!(" {} ", or(&understood.text, &understood.clean).to_lowercase());
    for (words, reply) in _SOFT_STAIN_WORDS.iter() {
        for word in words {
            if word.contains(' ') {
                if text.contains(word.as_str()) {
                    return Some(reply.clone());
                }
            } else if has_bounded_word(&text, word) {
                return Some(reply.clone());
            }
        }
    }
    Some("Blot, don't rub, with kitchen paper, then dab from the outside of the mark in with cool water and a drop of washing-up liquid, blotting dry with a towel and repeating until nothing more comes up. Work in small patches and never soak it, or the backing and underlay stay damp and start to smell.".into())
}

/// True when the line says what the mark is: "get tar off a car".
pub fn _names_a_mark(text: &str) -> bool {
    _A_MARK_NAMED.is_match(text)
}

/// True when the line says what is on a garment or a surface, not what it is made of.
pub fn landed_on_a_thing(understood: &Understood) -> bool {
    let text = format!(" {} ", or(&understood.text, &understood.clean).to_lowercase());
    for m in LANDED_ON.finditer(&text) {
        if m.get(1).is_some_and(|w| LANDED_ON_THINGS.contains(w)) {
            return true;
        }
    }
    if any_of(&text, &[" on it ", " on them ", " on those ", " on that ", " on this "]) {
        return LANDED_ON_THINGS.iter().any(|w| text.contains(w.as_str()));
    }
    false
}

/// The stand-down for a food row: a mark on a shirt is the stain row's line, not a recipe.
pub fn not_a_mark_on_a_thing(understood: &Understood) -> bool {
    !landed_on_a_thing(understood)
}

/// True when the line is about a mark coming off, or says where the mark is.
pub fn _says_how_it_comes_off(understood: &Understood) -> bool {
    let text = format!(" {}", or(&understood.text, &understood.clean).to_lowercase());
    let comes_off = OFF_COMES_OFF.is_match(&text) && (_A_PRONOUN_BEFORE_OFF.is_match(&text) || _names_a_mark(&text));
    any_in(&text, &COMES_OFF) || comes_off || landed_on_a_thing(understood)
}

/// The first move for a mark the base has no card for, by what made it and what it is on.
pub fn _stain_nobody_taught_it(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = or(&understood.text, &understood.clean).to_lowercase();
    let on_skin = any_of(&text, &["fingers", "finger", "hands", "hand", "skin", "arms", "arm", "face", "thumbs", "thumb"]);
    let on_hard = any_of(&text, &[
        "worktop", "work top", "counter", "countertop", "table", "floor", "floors", "wall", "walls", "slab", "slabs",
        "paving", "pavement", "path", "drive", "driveway", "patio", "decking", "brick", "bricks", "concrete", "render",
        "tile", "tiles", "grout", "bath", "sink", "basin", "toilet", "hob", "glass", "mirror", "window", "windows",
        "fridge", "cupboard", "door", "car", "car seat",
    ]);
    let padded = format!(" {text} ");
    if on_skin && !on_hard && A_STATE_OF_A_SURFACE.iter().any(|w| padded.contains(&format!(" {w} "))) {
        return None;
    }
    if any_of(&text, &["chewing gum", "chewed gum", "gum", "bubble gum"]) {
        if on_skin {
            return Some("Do not pull it: hold an ice cube on it for a minute until it goes hard and it lifts off in one piece, and a little cooking oil or peanut butter worked in with a fingertip takes the last of the stickiness off, then wash with soap. Nothing else shifts gum without taking skin with it.".into());
        }
        return Some("Harden it before you touch it: an ice cube in a bag held on the gum for a minute makes it brittle enough to pick off in one piece, and what is left comes away with a little cooking oil or a dab of peanut butter worked in from the outside, then washing-up liquid and a rinse. Do not use heat, which sets it into the fibres.".into());
    }
    if any_of(&text, &[
        "superglue", "super glue", "glue", "glued", "adhesive", "resin", "silicone", "sealant", "blu tack", "blu-tack",
        "blutack", "putty", "plasticine", "play dough", "playdough",
    ]) {
        if on_skin && any_of(&text, &["silicone", "sealant", "putty", "play dough", "playdough", "plasticine"]) {
            return Some("Let it set and then rub it off rather than trying to wipe it: wet silicone smears and takes the skin with it, cured silicone peels off in one lump. A little olive or baby oil on a cloth takes the film off afterwards, and warm soapy water finishes the fingers.".into());
        }
        if on_skin {
            return Some("Soak it in warm soapy water and do not pull the fingers apart: superglue lets go in warm water after ten or fifteen minutes, and a little acetone or nail polish remover on a cotton bud finishes it. Never use a knife or peel it, and if skin is properly bonded, soak rather than force it.".into());
        }
        return Some("Let it harden rather than wiping it while it is tacky: an ice cube on it for a minute makes it brittle enough to pick off in one piece, and a little cooking oil or acetone softens what is left. Then a drop of washing-up liquid on a damp cloth takes the residue, and work from the outside in so the mark does not spread.".into());
    }
    if any_of(&text, &[
        "tar", "tarmac", "bitumen", "creosote", "resin", "paint", "varnish", "dye", "hair dye", "henna", "wood stain",
        "rust", "oil paint", "graffiti",
    ]) {
        if on_skin {
            return Some("Oil first, not water: rub a little olive or baby oil into it, leave a minute and wipe with a cloth, and for hair dye on skin a dab of toothpaste or milk on a cotton bud lifts the last of the colour. Wash with soap after, and give it a couple of days rather than scrubbing the skin.".into());
        }
        if on_hard {
            return Some("Water will not move it, so start with a solvent on a cloth: white spirit or methylated spirit for tar and paint, vinegar or a lemon for a rust mark, and acetone only on a surface you know will take it. Test it on a hidden corner first, dab rather than rub, and wipe the residue off with hot water and a drop of washing-up liquid. Keep the window open, keep the cloths out of the bin indoors, and wash your hands after.".into());
        }
        return Some("Water will not move it, so start with a solvent on a cloth: white spirit or methylated spirit for tar and paint, vinegar or a lemon for a rust mark, and acetone on hard surfaces only. Test it on a hidden patch, dab rather than rub, and change the cloth as it takes the colour up. On fabric, follow with washing-up liquid and a wash, and keep it out of the dryer until the mark has gone.".into());
    }
    if on_skin
        && any_of(&text, &["sauce", "tomato", "ketchup", "curry", "juice", "coffee", "tea", "wine", "gravy", "soup", "chocolate", "jam"])
    {
        return Some("Rinse the hands under cold water first, not hot, then a drop of washing-up liquid worked in with the fingertips: a sauce is mostly water and colour, and hot water is what sets the colour into the skin. Tomato, curry and coffee leave a faint tint that fades in a day or two, so wash once more with soap later rather than scrubbing the skin.".into());
    }
    if any_of(&text, &[
        "grease", "greasy", "oil", "butter", "fat", "sauce", "sunscreen", "sun cream", "suncream", "cream", "lotion",
        "lipstick", "makeup", "make-up", "mayonnaise", "dressing",
    ]) {
        if on_skin {
            return Some("Washing-up liquid first, on dry hands and before any water: work a drop in with your fingertips, leave it a minute, then rinse in warm water - water on its own just spreads a grease. A second round with a soft nail brush takes what is in the creases, and a little hand cream afterwards puts back what the washing-up liquid took off.".into());
        }
        return Some("Washing-up liquid first, on the dry mark, worked in with a fingertip before any water touches it: that is what breaks up a grease. Leave it five minutes, then rinse in the hottest water the label allows and wash as normal. On a hard surface or a sofa, the same drop of washing-up liquid on a barely damp cloth, dabbed from the outside in, then blotted dry.".into());
    }
    if any_of(&text, &["egg", "milk", "blood", "sick", "vomit", "poo", "pee", "urine", "spit", "phlegm"]) {
        if on_skin {
            return Some("Cold water only, and plenty of it: hot water sets a protein mark into the skin and it stains harder. Rinse the hands under a cool tap until the water runs clear, wash with soap, and if any colour is left, a dab of hydrogen peroxide on a cotton bud lifts it - a blood mark on skin is not an emergency, and scrubbing it only makes it spread.".into());
        }
        return Some("Cold water only, and plenty of it: hot water sets a protein mark into the fibres and it will not come out afterwards. Rinse from the back of the fabric so the mark is pushed out rather than through, rub in a little washing-up liquid or biological detergent, leave it fifteen minutes, then wash as the label allows and check before it goes anywhere near a dryer.".into());
    }
    if on_skin {
        return Some("Warm soapy water and a soft nail brush, then a little oil or hand cream to lift what is left: most marks on skin are grease or dye and both come off with those two. Do not scrub the skin raw and do not put a solvent on it, apart from nail polish remover on a mark that will not shift, and give it a day rather than ten minutes.".into());
    }
    if any_of(&text, &["book", "paper", "card", "page", "pages", "letter", "document", "photo", "photograph"]) {
        return Some("On paper you cannot wash it, so lift what you can while it is wet: a clean cotton bud of water dabbed from the edge of the mark outwards, then blot with kitchen paper and leave a weight on it to dry flat. A soft eraser takes a dry mark down, and a highlighter or ink mark will always leave some colour behind, so stop before the paper starts to cockle.".into());
    }
    if any_of(&text, &["carpet", "rug", "mat", "sofa", "couch", "settee", "armchair", "mattress", "upholstery", "cushion", "curtain", "car seat"]) {
        return Some("Blot, do not rub, and work from the outside in so it does not spread: kitchen paper first, then a barely damp cloth with a drop of washing-up liquid, blotting and changing the cloth as it takes the colour up. Dry it with a towel and a weight rather than heat, and do not scrub, because scrubbing spreads the mark and fluffs the pile.".into());
    }
    if on_hard {
        return Some("A hard surface wants lifting and wiping rather than blotting: take the worst off with a plastic scraper or the back of a knife held flat, then wipe with hot water and a drop of washing-up liquid, turning the cloth as it takes the colour up. Anything stronger gets tested on a hidden corner first, because acid eats limestone and concrete and bleach dulls a seal, and a cream cleaner on a damp cloth takes the last of the film off without either.".into());
    }
    Some("Blot up what you can with kitchen paper, then rinse the mark from the back of the fabric in cool water - pushing it out rather than through the cloth - and work in a drop of washing-up liquid or a little biological detergent. Leave it fifteen minutes, wash at the hottest the care label allows, and air-dry it: a mark that has been through a dryer is much harder to shift, so check it before it goes in.".into())
}

/// True when the line says the chemical landed on somebody, not that it exists.
pub fn _the_chemical_landed_on_a_person(understood: &Understood) -> bool {
    let text = format!(" {}", or(&understood.text, &understood.clean).to_lowercase());
    any_in(&text, &CHEMICAL_LANDED)
}

/// True when the line is a mark on a person rather than a condition or a way of getting.
pub fn _a_mark_on_the_person(understood: &Understood) -> bool {
    let text = format!(" {} ", or(&understood.text, &understood.clean).to_lowercase());
    if NOT_A_SKIN_MARK.iter().any(|p| text.contains(&format!(" {p} "))) {
        return false;
    }
    if !MARK_ON_THE_PERSON.iter().any(|p| text.contains(&format!(" {p} "))) {
        return false;
    }
    any_in(&text, &A_MARK_WORD)
}

/// The verdict for "is it worth sealing", the method for "how do i seal".
pub fn _sealing_grout(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = reading.clean.as_str();
    if _SEAL_COST.is_match(said) {
        return Some("A one litre bottle of penetrating sealer is usually \u{a3}15 to \u{a3}30 and does a bathroom's grout twice over; the sealing itself is an afternoon's work.".into());
    }
    if _SEAL_WHEN.is_match(said) {
        return Some("New grout wants about 48 hours to cure before you seal it, then a shower's grout wants resealing every year or two, or sooner if water stops beading.".into());
    }
    if _SEAL_WHERE.is_match(said) {
        return Some("A tile shop or a DIY shed sells penetrating sealer: ask for an impregnating sealer for cement grout, not a film-forming one that peels in a shower.".into());
    }
    if _SEAL_THINGS.is_match(said) {
        return Some("A penetrating sealer for cement grout, a small brush or applicator and a cloth: clean the grout, let it dry for a day, work the sealer into the joints and wipe the tiles after ten minutes.".into());
    }
    let asks_a_verdict = reading.asks.iter().any(|a| a == "yesno" || a == "recommend" || a == "opinion");
    if asks_a_verdict || _SEAL_WORTH.is_match(said) {
        return Some("Worth it on cement grout in a shower or bathroom, less so in a dry hall: a penetrating sealer keeps water and stains out of the joints for a year or two, though it does not make the grout waterproof, and epoxy grout needs none. Clean it and let it dry for a day first, or the sealer locks the dirt in.".into());
    }
    Some("Clean the grout and let it dry for at least a day, then work a penetrating sealer into the joints with a small brush or applicator, not a sponge. Wipe the face of the tiles with a damp cloth after ten minutes so no film dries on them, and keep water off it for 24 hours. Reapply every year or two in a shower.".into())
}

/// False when the line asks whether the standby is good for a person, or what replaces it.
pub fn _not_a_cleaning_ask(understood: &Understood) -> bool {
    let text = or(&understood.clean, &understood.text);
    if NOT_A_CLEANING_ASK.is_match(text) {
        return false;
    }
    text.contains('?') || QUESTION_SHAPE.match_(text).is_some()
}

/// `guard=lambda read: not (_the_loaf_went_off(read) or _the_food_was_binned(read))` (line 1403).
pub fn _not_the_loaf_or_a_binned_food(read: &Understood) -> bool {
    !(_the_loaf_went_off(read) || _the_food_was_binned(read))
}

/// `guard=lambda understood: _a_dirty_window_not_a_wet_one(u) and _glass_in_the_line_is_a_window(u)` (line 1560).
pub fn _a_dirty_window(understood: &Understood) -> bool {
    _a_dirty_window_not_a_wet_one(understood) && _glass_in_the_line_is_a_window(understood)
}

/// `guard=lambda understood: any(word in the_line(understood) for word in DUST_METHOD_ASK)` (line 5373).
pub fn _asks_the_dust_method(understood: &Understood) -> bool {
    let line = the_line(understood);
    any_in(&line, &DUST_METHOD_ASK)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.cleaning:<lambda>:1403", |_: &FnRef, u: &Understood| _not_the_loaf_or_a_binned_food(u)),
    ("bot.knowledge.cleaning:<lambda>:1560", |_: &FnRef, u: &Understood| _a_dirty_window(u)),
    ("bot.knowledge.cleaning:<lambda>:3139", |_: &FnRef, u: &Understood| !food_has_gone_off(u)),
    ("bot.knowledge.cleaning:<lambda>:5373", |_: &FnRef, u: &Understood| _asks_the_dust_method(u)),
    ("bot.knowledge.cleaning:_a_colour_can_have_run:590", |_: &FnRef, u: &Understood| _a_colour_can_have_run(u)),
    ("bot.knowledge.cleaning:_a_mark_on_the_person:4850", |_: &FnRef, u: &Understood| _a_mark_on_the_person(u)),
    ("bot.knowledge.cleaning:_dust_is_on_a_thing:113", |_: &FnRef, u: &Understood| _dust_is_on_a_thing(u)),
    ("bot.knowledge.cleaning:_not_a_cleaning_ask:5415", |_: &FnRef, u: &Understood| _not_a_cleaning_ask(u)),
    ("bot.knowledge.cleaning:_not_a_room_with_its_own_card:38", |_: &FnRef, u: &Understood| {
        _not_a_room_with_its_own_card(u)
    }),
    ("bot.knowledge.cleaning:_not_the_dryer_alone:610", |_: &FnRef, u: &Understood| _not_the_dryer_alone(u)),
    ("bot.knowledge.cleaning:_says_how_it_comes_off:4540", |_: &FnRef, u: &Understood| _says_how_it_comes_off(u)),
    ("bot.knowledge.cleaning:_the_back_of_a_screen:498", |_: &FnRef, u: &Understood| _the_back_of_a_screen(u)),
    ("bot.knowledge.cleaning:_the_chemical_landed_on_a_person:4747", |_: &FnRef, u: &Understood| {
        _the_chemical_landed_on_a_person(u)
    }),
    ("bot.knowledge.cleaning:_two_steps_in_order:669", |_: &FnRef, u: &Understood| _two_steps_in_order(u)),
    ("bot.knowledge.cleaning:a_worn_thing_is_a_garment:159", |_: &FnRef, u: &Understood| a_worn_thing_is_a_garment(u)),
    ("bot.knowledge.cleaning:not_a_mark_on_a_thing:4535", |_: &FnRef, u: &Understood| not_a_mark_on_a_thing(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.cleaning:_a_mark_on_a_soft_surface:4319", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _a_mark_on_a_soft_surface(m, u)
    }),
    ("bot.knowledge.cleaning:_bath_towels:405", |_: &FnRef, m: &mut Memory, u: &Understood| _bath_towels(m, u)),
    ("bot.knowledge.cleaning:_bedding_life:350", |_: &FnRef, m: &mut Memory, u: &Understood| _bedding_life(m, u)),
    ("bot.knowledge.cleaning:_bedsheets:398", |_: &FnRef, m: &mut Memory, u: &Understood| _bedsheets(m, u)),
    ("bot.knowledge.cleaning:_bleach_down_the_drain:61", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _bleach_down_the_drain(m, u)
    }),
    ("bot.knowledge.cleaning:_clean_bbq:3871", |_: &FnRef, m: &mut Memory, u: &Understood| _clean_bbq(m, u)),
    ("bot.knowledge.cleaning:_cupboards:341", |_: &FnRef, m: &mut Memory, u: &Understood| _cupboards(m, u)),
    ("bot.knowledge.cleaning:_sealing_grout:4975", |_: &FnRef, m: &mut Memory, u: &Understood| _sealing_grout(m, u)),
    ("bot.knowledge.cleaning:_spectacles_fog:306", |_: &FnRef, m: &mut Memory, u: &Understood| _spectacles_fog(m, u)),
    ("bot.knowledge.cleaning:_stain_nobody_taught_it:4562", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _stain_nobody_taught_it(m, u)
    }),
    ("bot.knowledge.cleaning:_steam_cleaner:184", |_: &FnRef, m: &mut Memory, u: &Understood| _steam_cleaner(m, u)),
    ("bot.knowledge.cleaning:_the_place_a_pet_used:534", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _the_place_a_pet_used(m, u)
    }),
    ("bot.knowledge.cleaning:_uniform_care:220", |_: &FnRef, m: &mut Memory, u: &Understood| _uniform_care(m, u)),
];
