//! Port of `bot/knowledge/clothing.py`: clothes, shoes, washing, drying, ironing, hair and what
//! to wear. The entries and tables are data from the bundle; this is the logic their guards and
//! replies run.

use crate::data::{self, FnRef};
use crate::knowledge::d2_shared::{self, any_in};
use crate::knowledge::d2dup_g7::{self, cached_re, py_field, recent, user_lines};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as pyre, Re};
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn name(n: &str) -> String {
    format!("bot.knowledge.clothing.{n}")
}
fn re(n: &str) -> &'static Re {
    data::re(&name(n))
}
fn strs(n: &str) -> Vec<String> {
    data::strs(&name(n))
}
fn string(n: &str) -> String {
    data::string(&name(n))
}
fn set(n: &str) -> HashSet<String> {
    data::set(&name(n))
}
/// `(?<![a-z0-9])word(?![a-z0-9])` for each word of a table.
fn bounded(words: &[String]) -> Vec<&'static Re> {
    words.iter().map(|w| cached_re(&format!("(?<![a-z0-9]){}(?![a-z0-9])", pyre::escape(w)), 0)).collect()
}

static _ZIP_FAULT: LazyLock<&'static Re> = LazyLock::new(|| re("_ZIP_FAULT"));
static _ZIP_NEEDLE: LazyLock<&'static Re> = LazyLock::new(|| re("_ZIP_NEEDLE"));
static _ZIP_FITTING: LazyLock<&'static Re> = LazyLock::new(|| re("_ZIP_FITTING"));
static _MEAL_FOR_THE_NIGHT: LazyLock<HashSet<String>> = LazyLock::new(|| set("_MEAL_FOR_THE_NIGHT"));
static AN_INTERVIEW_IN_A_DRESS_LINE: LazyLock<Vec<&'static Re>> =
    LazyLock::new(|| bounded(&strs("AN_INTERVIEW_IN_A_DRESS_LINE")));
static _DEFINITELY_CLOTHES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DEFINITELY_CLOTHES"));
static PUT_ON_OTHER: LazyLock<HashSet<String>> = LazyLock::new(|| set("PUT_ON_OTHER"));
static _HAIR_LINE: LazyLock<&'static Re> = LazyLock::new(|| re("_HAIR_LINE"));
static _A_COLOUR_CHOICE: LazyLock<&'static Re> = LazyLock::new(|| re("_A_COLOUR_CHOICE"));
static _A_CHOICE_FRAME: LazyLock<&'static Re> = LazyLock::new(|| re("_A_CHOICE_FRAME"));
static _NOT_SPECTACLES: LazyLock<&'static Re> = LazyLock::new(|| re("_NOT_SPECTACLES"));
static _LENS_HELP: LazyLock<String> = LazyLock::new(|| string("_LENS_HELP"));
static _GARMENT: LazyLock<&'static Re> = LazyLock::new(|| re("_GARMENT"));
static _IRON_REPLY: LazyLock<String> = LazyLock::new(|| string("_IRON_REPLY"));
static _BURN_BEFORE: LazyLock<&'static Re> = LazyLock::new(|| re("_BURN_BEFORE"));
static _BLISTER_SORE: LazyLock<&'static Re> = LazyLock::new(|| re("_BLISTER_SORE"));
static _BLISTER_BURN: LazyLock<String> = LazyLock::new(|| string("_BLISTER_BURN"));
static _BLISTER_CARE: LazyLock<String> = LazyLock::new(|| string("_BLISTER_CARE"));
static _BLISTER_POP: LazyLock<String> = LazyLock::new(|| string("_BLISTER_POP"));
static _BLISTER_STOP: LazyLock<String> = LazyLock::new(|| string("_BLISTER_STOP"));
static _BLISTER_FIRST: LazyLock<String> = LazyLock::new(|| string("_BLISTER_FIRST"));
static _PREVENT_BLISTER: LazyLock<&'static Re> = LazyLock::new(|| re("_PREVENT_BLISTER"));
static _RUBBING: LazyLock<&'static Re> = LazyLock::new(|| re("_RUBBING"));
static _BREAKING_IN: LazyLock<&'static Re> = LazyLock::new(|| re("_BREAKING_IN"));
static _BLISTER_NAMED: LazyLock<&'static Re> = LazyLock::new(|| re("_BLISTER_NAMED"));
static _A_GLASS_OF: LazyLock<&'static Re> = LazyLock::new(|| re("_A_GLASS_OF"));
static _A_DRINKING_GLASS: LazyLock<&'static Re> = LazyLock::new(|| re("_A_DRINKING_GLASS"));
static _GLUE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GLUE_WORDS"));
static _GLASSES_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GLASSES_WORDS"));
static GLASSES_GLUE: LazyLock<String> = LazyLock::new(|| string("GLASSES_GLUE"));
static GLASSES_REPAIR: LazyLock<String> = LazyLock::new(|| string("GLASSES_REPAIR"));
static _SOCKS_SPORT: LazyLock<&'static Re> = LazyLock::new(|| re("_SOCKS_SPORT"));
static _SOCKS_BLACK_AND_BROWN: LazyLock<&'static Re> = LazyLock::new(|| re("_SOCKS_BLACK_AND_BROWN"));
static _SOCKS_FORMAL: LazyLock<&'static Re> = LazyLock::new(|| re("_SOCKS_FORMAL"));
static _SOCKS_CASUAL: LazyLock<&'static Re> = LazyLock::new(|| re("_SOCKS_CASUAL"));
static _SOCKS_COLOUR: LazyLock<&'static Re> = LazyLock::new(|| re("_SOCKS_COLOUR"));
static _MOULD_SOFT: LazyLock<Vec<&'static Re>> = LazyLock::new(|| bounded(&strs("_MOULD_SOFT")));
static _SAID: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SAID"));
static A_CHILD_REFUSING_A_GARMENT: LazyLock<&'static Re> = LazyLock::new(|| re("A_CHILD_REFUSING_A_GARMENT"));
static _COAT_FAULT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_COAT_FAULT"));
static _COAT_BUYING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_COAT_BUYING"));
static _A_FAULTY_MACHINE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_FAULTY_MACHINE"));
static NEEDS_SIXTY: LazyLock<&'static Re> = LazyLock::new(|| re("NEEDS_SIXTY"));
static _STAIN_SAID: LazyLock<Vec<String>> = LazyLock::new(|| strs("_STAIN_SAID"));
static _NO_IRON: LazyLock<&'static Re> = LazyLock::new(|| re("_NO_IRON"));
static _NO_IRON_TEXT: LazyLock<String> = LazyLock::new(|| string("_NO_IRON_TEXT"));
static _NO_BOARD: LazyLock<&'static Re> = LazyLock::new(|| re("_NO_BOARD"));
static _NO_BOARD_TEXT: LazyLock<String> = LazyLock::new(|| string("_NO_BOARD_TEXT"));
static _ACNE_PRODUCT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_ACNE_PRODUCT_WORDS"));
static _A_MARK_NOT_A_STAIN: LazyLock<HashSet<String>> = LazyLock::new(|| set("_A_MARK_NOT_A_STAIN"));
static _ANTIPERSPIRANT: LazyLock<HashSet<String>> = LazyLock::new(|| set("_ANTIPERSPIRANT"));
static _A_SMELL_ON_THE_GARMENT: LazyLock<HashSet<String>> = LazyLock::new(|| set("_A_SMELL_ON_THE_GARMENT"));
static _A_KIT_GARMENT: LazyLock<HashSet<String>> = LazyLock::new(|| set("_A_KIT_GARMENT"));
static LAUNDRY_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("LAUNDRY_WORDS"));
static BUILDING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("BUILDING_WORDS"));
static _A_DRYER: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_DRYER"));
static A_JOB_LEFT_UNDONE: LazyLock<Vec<String>> = LazyLock::new(|| strs("A_JOB_LEFT_UNDONE"));
static _DRY_DOWN_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DRY_DOWN_WORDS"));
static _DRY_HANGER_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DRY_HANGER_WORDS"));
static _DRY_FLAT_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DRY_FLAT_WORDS"));
static _DRY_FLUFFY_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DRY_FLUFFY_WORDS"));
static _BAD_COLOUR_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BAD_COLOUR_WORDS"));
static _THE_BAD_COLOUR: LazyLock<String> = LazyLock::new(|| string("_THE_BAD_COLOUR"));
static _THE_BAD_CUT: LazyLock<String> = LazyLock::new(|| string("_THE_BAD_CUT"));
static _HAIR_HOME_LENGTH: LazyLock<String> = LazyLock::new(|| string("_HAIR_HOME_LENGTH"));
static _HAIR_HOME_GREY: LazyLock<String> = LazyLock::new(|| string("_HAIR_HOME_GREY"));
static _HAIR_HOME_TAIL: LazyLock<String> = LazyLock::new(|| string("_HAIR_HOME_TAIL"));
static _HAIR_HOME_DARK_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HAIR_HOME_DARK_WORDS"));
static _HAIR_HOME_DARKER: LazyLock<String> = LazyLock::new(|| string("_HAIR_HOME_DARKER"));
static _HAIR_HOME_LIGHTER: LazyLock<String> = LazyLock::new(|| string("_HAIR_HOME_LIGHTER"));
static KEPT_IN: LazyLock<Vec<String>> = LazyLock::new(|| strs("KEPT_IN"));
static _LEATHER_PEELING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_LEATHER_PEELING"));
static _A_BODY_PART: LazyLock<&'static Re> = LazyLock::new(|| re("_A_BODY_PART"));
static _ONTO_A_PERSON: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ONTO_A_PERSON"));
static _A_NAMED_DRESS_CODE: LazyLock<&'static Re> = LazyLock::new(|| re("_A_NAMED_DRESS_CODE"));
static _A_WEDDING_SHORTAGE: LazyLock<&'static Re> = LazyLock::new(|| re("_A_WEDDING_SHORTAGE"));
static _A_FIT_OR_A_SIZE: LazyLock<&'static Re> = LazyLock::new(|| re("_A_FIT_OR_A_SIZE"));
static _A_WEIGHT_COMING_OFF: LazyLock<&'static Re> = LazyLock::new(|| re("_A_WEIGHT_COMING_OFF"));
static _A_WARDROBE_ADVICE: LazyLock<String> = LazyLock::new(|| string("_A_WARDROBE_ADVICE"));
static _A_HEIGHT: LazyLock<&'static Re> = LazyLock::new(|| re("_A_HEIGHT"));
static _ON_THE_FEET: LazyLock<&'static Re> = LazyLock::new(|| re("_ON_THE_FEET"));
static _A_USE_OF_A_GARMENT: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&name("_A_USE_OF_A_GARMENT"))
        .items()
        .iter()
        .map(|r| (r.items()[0].strs(), r.items()[1].str().to_string()))
        .collect()
});
static _A_COLD_YOU_HAVE: LazyLock<&'static Re> = LazyLock::new(|| re("_A_COLD_YOU_HAVE"));
static _BOILING_SOMETHING: LazyLock<&'static Re> = LazyLock::new(|| re("_BOILING_SOMETHING"));
static _A_TIME_ASKED_FOR: LazyLock<&'static Re> = LazyLock::new(|| re("_A_TIME_ASKED_FOR"));
static _A_PAN_WORD: LazyLock<&'static Re> = LazyLock::new(|| re("_A_PAN_WORD"));
static _TAKING_AND_NOT_DRESSING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_TAKING_AND_NOT_DRESSING"));
static _DRESSING_WORD: LazyLock<&'static Re> = LazyLock::new(|| re("_DRESSING_WORD"));
static _ENOUGH_FOR_A_GARMENT: LazyLock<&'static Re> = LazyLock::new(|| re("_ENOUGH_FOR_A_GARMENT"));
static _A_TEMPERATURE: LazyLock<&'static Re> = LazyLock::new(|| re("_A_TEMPERATURE"));
static _A_SIZE_SAID: LazyLock<&'static Re> = LazyLock::new(|| re("_A_SIZE_SAID"));
static _A_SHOE_SIZE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_SHOE_SIZE"));
static _WOMENS_UK_TO_EU: LazyLock<Vec<(i64, i64)>> = LazyLock::new(|| int_map("_WOMENS_UK_TO_EU"));
static _WOMENS_UK_TO_US: LazyLock<Vec<(i64, i64)>> = LazyLock::new(|| int_map("_WOMENS_UK_TO_US"));
static _LACE_TEACH: LazyLock<Vec<String>> = LazyLock::new(|| strs("_LACE_TEACH"));
static _LACE_SKATES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_LACE_SKATES"));
static _LACE_BOW: LazyLock<Vec<String>> = LazyLock::new(|| strs("_LACE_BOW"));
static _LACE_UNDONE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_LACE_UNDONE"));
static _LACE_TEACHING: LazyLock<String> = LazyLock::new(|| string("_LACE_TEACHING"));
static _LACE_BOW_REPLY: LazyLock<String> = LazyLock::new(|| string("_LACE_BOW_REPLY"));
static _LACE_SKATES_REPLY: LazyLock<String> = LazyLock::new(|| string("_LACE_SKATES_REPLY"));
static _LACE_UNDONE_REPLY: LazyLock<String> = LazyLock::new(|| string("_LACE_UNDONE_REPLY"));
static _LACE_KNOT: LazyLock<String> = LazyLock::new(|| string("_LACE_KNOT"));

static NOT_WORD_CHARS: LazyLock<Re> = LazyLock::new(|| Re::new("[^a-z0-9' ]+", 0));

fn int_map(n: &str) -> Vec<(i64, i64)> {
    data::get(&name(n)).pairs().iter().map(|(k, v)| (k.as_int().unwrap(), v.as_int().unwrap())).collect()
}

fn lookup(map: &[(i64, i64)], key: i64) -> Option<i64> {
    map.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// The reading's lower-case words.
fn lower_words(u: &Understood) -> Vec<String> {
    u.words.iter().map(|w| w.to_lowercase()).collect()
}

/// `" ".join(str(u.get(k) or "") for k in keys)` (+ the words), lower-cased: the wide line the
/// outfit guards read.
fn wide_line(u: &Understood) -> String {
    let text: Vec<String> =
        ["clean", "norm_text", "text", "raw", "rewritten", "plain"].iter().map(|k| py_field(u, k)).collect();
    format!("{} {}", text.join(" "), u.words.join(" ")).to_lowercase()
}

/// Guard: a zip that will not do up is not one being sewn in.
pub fn _not_fitting_a_zip(understood: &Understood) -> bool {
    let said = format!("{} {} {}", understood.text, understood.clean, understood.norm_text).to_lowercase();
    if _ZIP_FAULT.is_match(&said) {
        return true;
    }
    !(_ZIP_NEEDLE.is_match(&said) || _ZIP_FITTING.is_match(&said))
}

/// Guard: "date night" is what to wear, unless the line is about the meal.
pub fn _the_date_night_is_a_meal(understood: &Understood) -> bool {
    let said = lower_words(understood);
    let mut flat = said.join(" ");
    if flat.is_empty() {
        flat = d2_shared::or(&understood.clean, &understood.text).to_lowercase();
    }
    if said.iter().any(|w| _MEAL_FOR_THE_NIGHT.contains(w)) {
        return false;
    }
    for shape in [
        "dinner ideas", "dinner for", "dinner at home", "dinner tonight", "dinner for two", "dinner to cook", "meal ideas",
    ] {
        if flat.contains(shape) {
            return false;
        }
    }
    true
}

/// Guard: a dress code line that names an interview belongs to the interview rows.
pub fn _the_code_is_not_an_interview(understood: &Understood) -> bool {
    let clean = format!(" {} ", d2_shared::or(&understood.clean, &understood.text).to_lowercase());
    let clean = NOT_WORD_CHARS.sub(" ", &clean);
    !AN_INTERVIEW_IN_A_DRESS_LINE.iter().any(|r| r.is_match(&clean))
}

/// Guard: "put on" is about clothes unless the line puts on something else.
pub fn _put_on_is_clothes(understood: &Understood) -> bool {
    let said = lower_words(understood);
    let flat = said.join(" ");
    if !flat.contains("put on") && !flat.contains("puts on") && !flat.contains("putting on") {
        return true;
    }
    if _DEFINITELY_CLOTHES.iter().any(|w| said.contains(w)) {
        return true;
    }
    !PUT_ON_OTHER.iter().any(|w| said.contains(w))
}

/// Guard: the line is about hair.
pub fn _a_hair_line(understood: &Understood) -> bool {
    let text = d2_shared::or3(&understood.norm_text, &understood.clean, &understood.text).to_lowercase();
    _HAIR_LINE.is_match(&format!(" {text} "))
}

/// Guard: "which colour should i choose" is not this row's.
pub fn _not_a_colour_choice(understood: &Understood) -> bool {
    let said = d2_shared::or3(&understood.norm_text, &understood.clean, &understood.text);
    !(_A_COLOUR_CHOICE.is_match(said) && _A_CHOICE_FRAME.is_match(said))
}

/// Guard: the glass is a lens, not a window pane.
pub fn _not_a_pane(understood: &Understood) -> bool {
    _spectacle_glass(&Memory::default(), understood).is_some()
}

/// A lens to help with, unless this line or the two before are about a pane of glass.
pub fn _spectacle_glass(memory: &Memory, understood: &Understood) -> Option<String> {
    let pane = |text: &str| -> bool {
        let text = NOT_WORD_CHARS.sub(" ", &format!(" {} ", text.to_lowercase()));
        _NOT_SPECTACLES.is_match(&text)
    };
    if pane(d2_shared::or(&understood.clean, &understood.text)) {
        return None;
    }
    let turns = &memory.turns;
    for turn in &turns[turns.len().saturating_sub(2)..] {
        if pane(&turn.user) {
            return None;
        }
    }
    Some(_LENS_HELP.clone())
}

/// Ironing a garment, or "" for anything else.
pub fn _iron_shirt(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    static IRONING: LazyLock<Re> = LazyLock::new(|| Re::new("\\b(?:iron|ironing|steam|crease)\\b", 0));
    let said = &understood.clean;
    if _GARMENT.is_match(said) || IRONING.is_match(said) {
        return Some(_IRON_REPLY.clone());
    }
    Some(String::new())
}

/// Blisters from new shoes: a burn, a sore one, popping one, stopping them, or the first aid.
pub fn _new_shoe_blisters(memory: &mut Memory, understood: &Understood) -> Option<String> {
    static POP: LazyLock<Re> = LazyLock::new(|| Re::new("\\b(?:pop|burst|drain|bust)\\b", pyre::I));
    static STOP_HURTING: LazyLock<Re> = LazyLock::new(|| {
        Re::new("\\bstop\\s+(?:it|them|the\\s+blister)\\s+(?:from\\s+)?(?:hurting|aching|pain)\\b", pyre::I)
    });
    let text = d2_shared::or(&understood.clean, &understood.text);
    let mut burn = _BURN_BEFORE.is_match(text);
    let mut sore = _BLISTER_SORE.is_match(text);
    if !burn {
        let turns = &memory.turns;
        for turn in &turns[turns.len().saturating_sub(2)..] {
            if _BURN_BEFORE.is_match(&turn.user) {
                burn = true;
            }
            if _BLISTER_SORE.is_match(&turn.user) {
                sore = true;
            }
        }
    }
    if burn {
        return Some(_BLISTER_BURN.clone());
    }
    if sore {
        return Some(_BLISTER_CARE.clone());
    }
    if POP.is_match(text) || STOP_HURTING.is_match(text) {
        return Some(_BLISTER_POP.clone());
    }
    if _PREVENT_BLISTER.is_match(text) {
        return Some(_BLISTER_STOP.clone());
    }
    if (_RUBBING.is_match(text) || _BREAKING_IN.is_match(text)) && !_BLISTER_NAMED.is_match(text) {
        return Some(_BLISTER_STOP.clone());
    }
    Some(_BLISTER_FIRST.clone())
}

/// Guard: "a glass of" is a drink; a drinking glass named otherwise is not the spectacles row's.
pub fn _the_glass_is_a_drink(understood: &Understood) -> bool {
    let said = format!("{} {} {}", understood.clean, understood.norm_text, understood.text).to_lowercase();
    if _A_GLASS_OF.is_match(&said) {
        return true;
    }
    !_A_DRINKING_GLASS.is_match(&said)
}

/// Mending glasses: glue for a frame without the glasses named, else the optician.
pub fn _glasses_repair(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!("{} {} {}", understood.clean, understood.norm_text, understood.text).to_lowercase();
    let mentioned = _GLUE_WORDS.iter().any(|w| said.contains(w.as_str()));
    if mentioned && !any_in(&said, &_GLASSES_WORDS) {
        return Some(GLASSES_GLUE.clone());
    }
    Some(GLASSES_REPAIR.clone())
}

/// Which socks: for sport, with brown shoes, formal, white, casual, or the general rule.
pub fn _socks(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    static WHITE: LazyLock<Re> = LazyLock::new(|| Re::new("\\bwhite\\b", pyre::I));
    let said = _the_line(understood);
    if _SOCKS_SPORT.is_match(&said) {
        return Some(
            "For running, walking or the gym, wear a thin synthetic or merino-wool sock with a flat seam across the \
toe, in the same shoe you train in: cotton holds sweat against the skin and that is what blisters come from. Two thin \
pairs are warmer and rub less than one thick pair, and the sock should be tight enough not to bunch under the arch."
                .to_string(),
        );
    }
    if _SOCKS_BLACK_AND_BROWN.is_match(&said) {
        return Some(
            "Black socks with brown shoes is the pairing to avoid: the sock should match the trousers, so with brown \
shoes wear a dark brown, navy or charcoal sock, or keep the shoes for a lighter suit. Nobody notices a sock that \
matches the trousers and everybody notices one that fights the shoes."
                .to_string(),
        );
    }
    if _SOCKS_FORMAL.is_match(&said) {
        return Some(
            "Dark socks that match the trousers rather than the shoes, long enough to cover the calf so no bare leg \
shows when you sit down: navy, charcoal or black in wool or a wool blend, plain unless the suit is patterned. Black \
socks with brown shoes is the one pairing that looks wrong, so match the trousers and let the shoes be their own colour."
                .to_string(),
        );
    }
    if WHITE.is_match(&said) && _SOCKS_CASUAL.is_match(&said) {
        return Some(
            "White socks are fine with trainers, shorts and gym kit, and wrong with a suit or smart trousers: with \
those, wear a dark sock that matches the trousers, long enough that the calf stays covered when you sit down."
                .to_string(),
        );
    }
    if _SOCKS_CASUAL.is_match(&said) || _SOCKS_COLOUR.is_match(&said) {
        return Some(
            "Go by the trousers rather than the shoes: a plain sock a shade darker than the trousers is the safe \
pairing, which means dark grey or navy with jeans and charcoal or black with smart trousers. Cotton for everyday, wool \
or a wool blend when the foot will be in a shoe all day, because cotton stays wet against the skin."
                .to_string(),
        );
    }
    Some(
        "Match the trousers rather than the shoes, dark navy, charcoal or black, and long enough that no bare leg shows \
when you sit down. Cotton for everyday and wool or a wool blend for a shoe you are in all day, and thin wool rather \
than thick cotton with anything smart, because a thick sock makes a well-fitted shoe tight."
            .to_string(),
    )
}

/// Mould on something washable.
pub fn _mould_wash(_memory: &mut Memory, _understood: &Understood) -> Option<String> {
    Some(
        "Brush the dry spores off outdoors and wash it at the hottest temperature the care label allows with ordinary \
detergent: that alone clears cotton, linen and synthetics, and drying it in the sun finishes the job. Don't bleach \
wool, silk or anything dyed - use a colour-safe oxygen bleach instead. Mould that has gone through the padding of a \
pillow, a pram mattress or a padded jacket is not worth saving. Damp is what feeds it, so dry the piece right through \
and ventilate where it is kept, or the spots come back."
            .to_string(),
    )
}

/// Mould on leather, suede or anything that must not be washed.
pub fn _mould_wipe(_memory: &mut Memory, _understood: &Understood) -> Option<String> {
    Some(
        "Don't wash it: brush the spores off outdoors with a soft brush, then wipe it over with a cloth wrung out in \
a little white vinegar and let it dry slowly, away from heat - a radiator or a hot dryer hardens and cracks leather \
and shrinks suede. Finished leather likes a conditioner afterwards; suede that has gone stiff or black all through \
wants a leather specialist, and faux leather and patent wipe clean with the same vinegar cloth. Damp is what grew it, \
so air the piece and keep it dry from now on."
            .to_string(),
    )
}

/// Mould on a thing: wiped for leather and the like, washed for the rest.
pub fn _mould_on_it(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line =
        format!(" {} ", NOT_WORD_CHARS.sub(" ", &d2_shared::or(&understood.clean, &understood.text).to_lowercase()));
    if _MOULD_SOFT.iter().any(|r| r.is_match(&line)) {
        return _mould_wipe(memory, understood);
    }
    _mould_wash(memory, understood)
}

/// Everything the user has said so far, lower-cased.
pub fn _conversation_so_far(memory: &Memory) -> String {
    memory.turns.iter().map(|t| t.user.as_str()).collect::<Vec<_>>().join(" ").to_lowercase()
}

/// The line read three ways (clean, normalised, typed), lower-cased.
pub fn _the_line(understood: &Understood) -> String {
    _SAID.iter().map(|k| py_field(understood, k)).collect::<Vec<_>>().join(" ").to_lowercase()
}

/// `any(word in text for word in words)`.
pub fn _any_word<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    any_in(text, words)
}

/// True when one of the last few lines said one of the words.
pub fn _said_lately<S: AsRef<str>>(memory: &Memory, words: &[S], turns: usize) -> bool {
    recent(memory, turns).iter().any(|t| _any_word(&t.user.to_lowercase(), words))
}

/// Boots to buy: what to spend, wellies, walking boots, winter boots, or the fit.
pub fn _boots_to_buy(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if _said_lately(
        memory,
        &[
            "football", "rugby", "five a side", "5 a side", "netball", "hockey", "match", "pitch", "training session",
            "studs", "astro", "moulded",
        ],
        3,
    ) {
        return None;
    }
    let said = _the_line(understood);
    if _any_word(&said, &["how much", "spend", "price", "cost", "cheap", "budget", "afford"]) {
        return Some(
            "Spend \u{a3}90 to \u{a3}160 on a walking boot with a waterproof membrane: it lasts about a decade and can \
be resoled, where a \u{a3}40 boot is usually a sole that splits in a year and a lining that lets water in. Everyday \
and winter boots are \u{a3}60 to \u{a3}120, and at any price the fit matters more than the badge."
                .to_string(),
        );
    }
    if _any_word(&said, &["wellies", "welly", "wellington", "wellington boots"]) {
        return Some(
            "For a muddy path a wellington beats a walking boot: buy rubber with a deep lugged sole and a real heel, \
and check the top of the boot does not rub your calf when you walk, which is what puts most people off a long walk in \
them. A neoprene lining keeps your feet warm in winter, and half a size up leaves room for thick socks."
                .to_string(),
        );
    }
    if _any_word(
        &said,
        &["walking", "hiking", "hill", "mountain", "fell", "rambl", "trek", "lake district", "muddy", "bog", "ridge"],
    ) {
        return Some(
            "For walking, buy a boot with a waterproof membrane (Gore-Tex or similar), a fairly stiff sole with deep \
lugs and enough of a cuff to hold your ankle on rough ground. Try them on late in the day in the socks you will walk \
in, with a thumb's width at the toe and no lift at the heel, and wear them around the house an evening at a time \
before a long walk."
                .to_string(),
        );
    }
    if _any_word(&said, &["winter", "snow", "ice", "icy", "slush", "grit", "frost", "cold"]) {
        return Some(
            "For winter the tread matters more than the lining: deep soft rubber lugs grip ice and packed snow, where \
a hard plastic sole just skates. Buy them roomy enough for a thick wool sock with the heel still snug, and check the \
upper is waterproof rather than only shower resistant."
                .to_string(),
        );
    }
    Some(
        "Fit first: try them on in the afternoon in the socks you will wear, with a thumb's width at the toe and no \
lift at the heel, because a boot will not stretch to fit. Then look at the sole - deep rubber tread, a waterproof \
upper - and at lacing that lets you tighten the ankle without pinching across the foot."
            .to_string(),
    )
}

/// Trainers to buy: what to spend, with a suit, or when to replace them.
pub fn _trainers_to_buy(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _the_line(understood);
    if _any_word(&said, &["how much", "spend", "price", "cost", "cheap", "budget", "afford", "worth", "expensive"]) {
        return Some(
            "Spend \u{a3}70 to \u{a3}120 on everyday trainers and \u{a3}90 to \u{a3}130 on running shoes: past about \
\u{a3}130 you are paying for a logo or a limited colourway, and below \u{a3}50 the midsole is the part that gives up \
first. Replace them when the sole is flat under the ball of the foot, not when they look scuffed."
                .to_string(),
        );
    }
    if _any_word(
        &said,
        &[
            "suit", "smart", "navy", "office", "interview", "wedding", "go with", "wear with", "pair with", "trousers",
            "chinos", "shirt",
        ],
    ) {
        return Some(
            "With a suit, keep the trainer plain and clean: a low-profile white or black leather pair, no visible \
logo, a thin sole, and spotless matters more than the brand. Chunky soles, mesh and colours read as weekend, and suede \
with a suit looks like you forgot your shoes."
                .to_string(),
        );
    }
    Some(
        "Replace them once the midsole has gone flat or the heel is worn through: the cushioning is what has gone, no \
repair brings it back, and it starts to show as heel or knee pain. Buy at the end of the day in the socks you will \
wear them with, leave a thumb's width at the toe, and check the heel does not lift when you walk."
            .to_string(),
    )
}

/// Taking up a hem: jeans, curtains and the like, or a skirt or trousers.
pub fn _take_up_hem(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _the_line(understood);
    if _any_word(&said, &["jeans", "jean", "denim"]) {
        return Some(
            "Put them on with the shoes you will wear them with, turn the leg up inside out to the length you want - \
creasing the shoe, not puddling on it - and stitch 1cm from the fold. Trim the spare to 1.5cm, zigzag that raw edge \
and press it flat. To keep the faded original hem, cut the leg 3cm above it, pin that hem back on and stitch round \
twice: that is the trick that makes a shortened pair look factory."
                .to_string(),
        );
    }
    if _any_word(
        &said,
        &[
            "curtain", "curtains", "drape", "drapes", "blind", "blinds", "tablecloth", "tablecloths", "napkin",
            "napkins", "tea towel", "tea towels",
        ],
    ) {
        return Some(
            "Hang them and let them drop for a few days before you cut: fabric stretches under its own weight, and a \
hem measured flat ends up too short. Then measure from the rail at three points across each panel, because floors and \
rails are rarely level: aim for 1cm below the sill for a sill-length pair, or 1cm above the floor for floor length. \
Turn up a double hem about 5cm deep, press it with a damp cloth, and use iron-on hemming tape if you would rather not \
sew - an even hem is what makes curtains look made, more than the stitches do."
                .to_string(),
        );
    }
    Some(
        "Try it on inside out and pin the hem where it should fall, then press and stitch round with a simple running \
stitch, just catching the fabric. Iron the fold first with a damp cloth, and a few dots of hemming tape hold it \
without any sewing. Keep the cut edge 1.5cm long and press the seam flat, and the hem sits as flat as a shop's."
            .to_string(),
    )
}

/// Jeans to buy: the fit, and what to spend.
pub fn _jeans_to_buy(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _the_line(understood);
    let fit = "the waist should sit snug with no gap at the back, the rise about an inch below your navel, and the \
hem break once on the shoe";
    if _any_word(
        &said,
        &["cheap", "cheapest", "budget", "afford", "under", "quid", "price", "brand", "brands", "who makes", "value", "money", "shop"],
    ) {
        return Some(format!(
            "Fit is what makes jeans look good, so spend your time on the cut rather than the label: {fit}. A shop's \
own brand at \u{a3}25 to \u{a3}45 in 98% cotton with a little elastane wears as well as a designer pair, so try one \
size either side of what you think you are, because the same size varies by inches between shops."
        ));
    }
    Some(format!(
        "Jeans are fitted by three numbers, so try a size either side of the one you think you are: {fit}. Look for \
98% cotton with a little elastane, which holds its shape without clinging, and a leg that is long enough not to ride \
up when you sit."
    ))
}

/// Guard: a child refusing to wear something is a parenting line, not a clothes one.
pub fn not_a_fight_about_clothes(understood: &Understood) -> bool {
    !A_CHILD_REFUSING_A_GARMENT.is_match(&_the_line(understood))
}

/// A child's coat: in a car seat or pram, sizing, or what to look for.
pub fn _child_coat(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _the_line(understood);
    if _any_word(&said, &["car seat", "carseat", "car", "pram", "pushchair", "buggy", "stroller", "travel system"]) {
        return Some(
            "Never put a thick coat under a car-seat harness: the straps cannot tighten on it, and in a crash a child \
can slip out of a loose seat. Strap them in a thin fleece or jumper and put the coat on once they are out, or lay it \
over them, and warm the car before you set off. In a pram a pramsuit or a coat over a tucked-in blanket is warmer than \
a stiff coat they have to be folded into."
                .to_string(),
        );
    }
    if _any_word(&said, &["size", "sizing", "grow", "growing", "age", "year old", "fit"]) {
        return Some(
            "Buy the size that fits their height now rather than one to grow into: an oversized coat leaves a gap at \
the neck and wrists where the warmth goes, and the sleeves hang over their hands. Check there is room for a jumper \
underneath, that the hood stays up in wind, and that the cuffs and the hem can be adjusted."
                .to_string(),
        );
    }
    Some(
        "For a child, look for a hood that stays up, a waterproof outer, room for a jumper underneath and no cords \
round the neck; a size that fits now beats one to grow into. A baby in a pram is warmer in a pramsuit than in a coat \
they have to be folded into."
            .to_string(),
    )
}

/// Guard: a fault with a coat, not buying one.
pub fn _a_faulty_coat(understood: &Understood) -> bool {
    let said = _the_line(understood);
    if _any_word(&said, &_COAT_FAULT) {
        return true;
    }
    !_any_word(&said, &_COAT_BUYING)
}

/// What temperature to wash at, or "" when the chat is about a faulty machine or a radiator.
pub fn _wash_temperature(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _conversation_so_far(memory);
    if any_in(&said, &_A_FAULTY_MACHINE) {
        return Some(String::new());
    }
    let line = _the_line(understood);
    if any_in(&line, &["towel rail", "towel radiator", "heated towel rail", "ladder rail", "radiator", "radiators"]) {
        return Some(String::new());
    }
    if line.contains("90") || line.contains("95") {
        return Some(
            "90 is only for illness, nappies and bedding that has been through a bug: it shrinks cotton and fades \
colour, so nothing you want to keep should go in it. 60 does towels, bedding, underwear, socks and anything greasy or \
stained, and 30 or 40 covers shirts and jeans worn once."
                .to_string(),
        );
    }
    if line.contains("60") {
        return Some(
            "60 is not too hot for towels, bedding, underwear, socks and anything greasy or stained - it is what those \
need, along with a full dose of detergent. Keep shirts and jeans worn once at 30 or 40, and save 90 for illness and \
nappies: heat is what shrinks and fades things."
                .to_string(),
        );
    }
    if !any_in(&line, &["30", "40", "60", "90", "95"]) && NEEDS_SIXTY.is_match(&line) {
        return Some(
            "60C for bedding, towels, underwear and socks: that is the heat that kills dust mites and the bacteria \
that make a towel smell, along with a full dose of detergent, and cotton bedding takes it well. Keep shirts, jeans and \
anything worn once at 30 or 40, and save 90 for illness and nappies."
                .to_string(),
        );
    }
    Some(
        "Treat 30C as the everyday setting: it is about 10p a load cheaper than 40C and gets shirts, jeans and \
normally-worn clothes just as clean, as long as the drum is not crammed and you use the full dose of detergent. Save 40 \
for heavy soiling, 60 for towels, bedding, underwear, socks and anything greasy or stained, and 90 only for illness or \
nappies."
            .to_string(),
    )
}

/// Engine oil on clothes, when this line or an earlier one names the stain.
pub fn _engine_oil_stain(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut said = ["clean", "norm", "plain"].iter().map(|k| py_field(understood, k)).collect::<Vec<_>>().join(" ");
    if !any_in(&said, &_STAIN_SAID) {
        said = user_lines(memory).iter().map(|t| t.to_lowercase()).collect::<Vec<_>>().join(" ");
    }
    if !any_in(&said, &_STAIN_SAID) {
        return None;
    }
    Some(
        "Do not wash it first: water and detergent only push engine oil further in. Blot what will come off, work \
neat washing-up liquid into the mark with your fingers and leave it 15 minutes, then wash it on its own at the hottest \
the label allows. On a mark that has dried, a drop of white spirit or a motor-shop degreaser does the lifting first, \
rinsed out before it goes in the machine. Check it before the dryer, because heat sets it."
            .to_string(),
    )
}

/// No iron, or no ironing board; "" otherwise.
pub fn _no_iron_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = d2_shared::or(&understood.clean, &understood.text);
    if _NO_IRON.is_match(said) {
        return Some(_NO_IRON_TEXT.clone());
    }
    if _NO_BOARD.is_match(said) {
        return Some(_NO_BOARD_TEXT.clone());
    }
    Some(String::new())
}

/// Guard: an acne line is this row's only when it asks about a product.
pub fn _acne_product_question(understood: &Understood) -> bool {
    let words: HashSet<String> = lower_words(understood).into_iter().collect();
    if !words.contains("acne") {
        return true;
    }
    words.iter().any(|w| _ACNE_PRODUCT_WORDS.contains(w))
}

/// Sweat marks: deodorant marks, a smelly kit, or yellow underarm stains.
pub fn _sweat_mark(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said: HashSet<String> = lower_words(reading).into_iter().collect();
    let clean = reading.clean.to_lowercase();
    let has = |set: &HashSet<String>| said.iter().any(|w| set.contains(w));
    if has(&_A_MARK_NOT_A_STAIN) && has(&_ANTIPERSPIRANT) && !has(&_A_SMELL_ON_THE_GARMENT) {
        return Some(
            "Deodorant marks on a dark top are the antiperspirant sitting on the fibres, not a stain: rub the fabric \
against itself, or run a dry cloth or a pair of tights over it, and they lift off without washing. Let the deodorant \
dry properly before you dress and the marks stop appearing."
                .to_string(),
        );
    }
    if has(&_A_SMELL_ON_THE_GARMENT) || has(&_A_KIT_GARMENT) || clean.contains("work out") || clean.contains("sports kit") {
        return Some(
            "Wash it at 40C with a biological detergent and no fabric softener, which coats the fibres and traps the \
smell, and use the extra-rinse setting: the bacteria that make old sweat smell live in the fibres and survive a cool \
wash. Leave it to dry in the air rather than in a heap, and never put a sweaty kit back in the bag. A splash of white \
vinegar in the softener drawer instead of conditioner shifts the last of it."
                .to_string(),
        );
    }
    Some(
        "Soak the armpits in a paste of bicarbonate of soda and water with a squeeze of lemon for an hour, or use \
oxygen bleach on white shirts. Wash at 40C and don't iron over the stain until it's gone."
            .to_string(),
    )
}

/// Guard: drying is the washing, not a building drying out.
pub fn _drying_the_washing(understood: &Understood) -> bool {
    let mut text = understood.words.join(" ").to_lowercase();
    if text.is_empty() {
        text = d2_shared::or(&understood.clean, &understood.text).to_lowercase();
    }
    if any_in(&text, &LAUNDRY_WORDS) {
        return true;
    }
    !any_in(&text, &BUILDING_WORDS)
}

/// True when the line is a tumble dryer that has stopped doing its job (not a hair dryer).
pub fn _the_dryer_at_fault(reading_said: &str) -> bool {
    if !any_in(reading_said, &_A_DRYER) {
        return false;
    }
    if reading_said.contains("hair dryer") || reading_said.contains("hairdryer") {
        return false;
    }
    any_in(reading_said, &A_JOB_LEFT_UNDONE)
}

/// How to dry it: a faulty dryer, down, a waterproof, wool, towels and duvets, or the washing.
pub fn _drying_it(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = understood.words.join(" ").to_lowercase();
    if _the_dryer_at_fault(&words) {
        return d2dup_g7::_dryer_heat(memory, understood);
    }
    let named_down = any_in(&words, &_DRY_DOWN_WORDS)
        || (crate::py::text::split_ref(&words).contains(&"down") && any_in(&words, &["jacket", "coat", "gilet", "bag"]));
    if named_down {
        return Some(
            "Tumble a down jacket on low with two or three dryer balls and stop every twenty minutes to pull the \
clumps apart, until it is completely dry - hanging it wet leaves the down in lumps that never loft again. If it is \
dirty, wash it first on its own with a down wash and no fabric conditioner."
                .to_string(),
        );
    }
    if any_in(&words, &_DRY_HANGER_WORDS) {
        return Some(
            "Hang it on a wide hanger somewhere airy with the pockets empty and the hood out, and let it drip dry. \
Keep a waterproof away from a radiator and a tumble dryer, because the heat softens the coating that keeps the rain out."
                .to_string(),
        );
    }
    if any_in(&words, &_DRY_FLAT_WORDS) {
        return Some(
            "Dry wool flat, never on a hanger: press the water out in a towel, reshape it on a dry towel or a mesh \
rack and leave it somewhere airy, away from a radiator. Hanging it wet is what stretches a jumper out of shape."
                .to_string(),
        );
    }
    if any_in(&words, &_DRY_FLUFFY_WORDS) {
        return Some(
            "Shake it out, then hang it with space on both sides - a towel or a duvet bunched up stays damp in the \
middle. Finish it in a tumble dryer on low if you have one, and do not fold it away until it is dry to the touch, or it \
comes out musty."
                .to_string(),
        );
    }
    Some(
        "Hang the washing on a rack in the warmest room with a window cracked open, spaced out so air moves round it, \
and turn thicker things after a few hours. A dehumidifier dries a full load overnight indoors and stops the damp smell."
            .to_string(),
    )
}

/// A haircut gone wrong: the colour, or the cut.
pub fn _the_bad_haircut(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if _any_word(&_the_line(understood), &_BAD_COLOUR_WORDS) {
        return Some(_THE_BAD_COLOUR.clone());
    }
    Some(_THE_BAD_CUT.clone())
}

/// Dyeing hair at home: how long to leave it, grey, darker, or lighter.
pub fn _hair_colour_home(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _the_line(understood);
    if (said.contains("how long") || said.contains("how many minutes") || said.contains("leave it on") || said.contains("leave on for"))
        && !_any_word(&said, &["last", "lasts", "keep", "keeps", "shelf", "expire", "go off"])
    {
        return Some(_HAIR_HOME_LENGTH.clone());
    }
    if _any_word(&said, &["grey", "gray", "greys", "grays", "white hair", "going grey"]) {
        return Some(format!("{}{}", *_HAIR_HOME_GREY, *_HAIR_HOME_TAIL));
    }
    if _any_word(&said, &_HAIR_HOME_DARK_WORDS) && !_any_word(&said, &["lighter", "blonde", "blond", "bleach", "lift"]) {
        return Some(format!("{}{}", *_HAIR_HOME_DARKER, *_HAIR_HOME_TAIL));
    }
    Some(format!("{}{}", *_HAIR_HOME_LIGHTER, *_HAIR_HOME_TAIL))
}

/// Guard: mould on a kept food in a bag or box is the food's question, not the garment's.
pub fn _the_mould_is_not_on_a_kept_food(understood: &Understood) -> bool {
    let mut names = KEPT_IN.clone();
    names.extend(["leather", "suede", "fabric"].iter().map(|s| s.to_string()));
    !(d2_shared::a_food_is_being_kept(understood) && d2dup_g7::only_a_container(understood, &names))
}

/// A mark on leather: peeling, or a scuff.
pub fn _leather_mark(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let line = d2_shared::or(&reading.clean, &reading.text).to_lowercase();
    if any_in(&line, &_LEATHER_PEELING) {
        return Some(
            "Peeling or flaking leather is the finish itself lifting, and no cream or polish brings it back: a leather \
repair kit with a filler and a colour matched to the piece hides a small patch, and a whole panel that is going means \
a re-cover. On bonded leather, which is a coating on a fibre backing, once it starts to peel it keeps going, because \
the layers are separating."
                .to_string(),
        );
    }
    Some(
        "A light scuff on finished leather usually lifts with a little leather conditioner or a matching leather cream \
worked in with a cloth, in circles, then buffed with a dry one; a scratch that has gone grey wants a leather dye or a \
colour-matched pen from a furniture shop. Keep the piece out of direct sun and away from radiators, because heat is \
what makes the finish crack."
            .to_string(),
    )
}

/// Guard: grease on a garment, unless it landed on a person (a burn).
pub fn _grease_is_not_a_burn(understood: &Understood) -> bool {
    let text = d2_shared::or(&understood.clean, &understood.text).to_lowercase();
    if !_A_BODY_PART.is_match(&text) {
        return true;
    }
    !any_in(&text, &_ONTO_A_PERSON)
}

/// Guard: nothing to wear to a wedding, unless the line names a dress code.
pub fn _an_outfit_shortage(understood: &Understood) -> bool {
    let text = wide_line(understood);
    if _A_NAMED_DRESS_CODE.is_match(&text) {
        return false;
    }
    _A_WEDDING_SHORTAGE.is_match(&text)
}

/// Guard: the line is not about how something fits.
pub fn _the_line_is_not_about_fit(understood: &Understood) -> bool {
    !_A_FIT_OR_A_SIZE.is_match(&wide_line(understood))
}

/// A wardrobe after a size change, with a check-up first when the weight came off by itself.
pub fn _the_size_you_are(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _the_line(understood);
    if _A_WEIGHT_COMING_OFF.is_match(&format!(" {said} ")) {
        let advice: &str = &_A_WARDROBE_ADVICE;
        let mut chars = advice.chars();
        let first: String = chars.next().map(|c| c.to_lowercase().collect()).unwrap_or_default();
        return Some(format!(
            "If the weight came off without you trying, get it checked first: that is worth one appointment, and \
weight that comes off by itself shouldn't be ignored. Then, when you restock, {first}{}",
            chars.as_str()
        ));
    }
    Some(_A_WARDROBE_ADVICE.clone())
}

/// Guard: "5 feet" is a height unless the line is about what is on the feet.
pub fn _the_feet_are_not_a_height(understood: &Understood) -> bool {
    let text = wide_line(understood);
    if !_A_HEIGHT.is_match(&text) {
        return true;
    }
    _ON_THE_FEET.is_match(&text)
}

/// `_a_garment_to_choose(fit)`'s reply: the advice for a use the chat named, else the fit.
fn _a_garment_to_choose_answer(f: &FnRef, memory: &mut Memory, _understood: &Understood) -> Option<String> {
    for (words, reply) in _A_USE_OF_A_GARMENT.iter() {
        if _said_lately(memory, words, 4) {
            return Some(reply.clone());
        }
    }
    Some(f.cell("fit").map(|v| v.str().to_string()).unwrap_or_default())
}

/// Guard: the line asks what to wear for the weather.
pub fn _the_weather_is_the_question(understood: &Understood) -> bool {
    if !_put_on_is_clothes(understood) {
        return false;
    }
    let said = lower_words(understood).join(" ");
    if _A_COLD_YOU_HAVE.is_match(&said) {
        return false;
    }
    if _BOILING_SOMETHING.is_match(&said) {
        return false;
    }
    if _A_TIME_ASKED_FOR.is_match(&said) && _A_PAN_WORD.is_match(&said) {
        return false;
    }
    if !_TAKING_AND_NOT_DRESSING.iter().any(|t| t == d2_shared::opt(&understood.topic)) {
        return true;
    }
    _DRESSING_WORD.is_match(&said)
}

/// True when "enough" is about a garment ("is a jumper enough").
pub fn _the_enough_is_about_a_garment(understood: &Understood) -> bool {
    _ENOUGH_FOR_A_GARMENT.is_match(&lower_words(understood).join(" "))
}

/// The temperature the line gives in degrees, or None.
pub fn _the_temperature_said(understood: &Understood) -> Option<i64> {
    let line = d2_shared::or(&understood.clean, &understood.text).to_lowercase();
    let found = _A_TEMPERATURE.search(&line)?;
    let figure = found.get(1).expect("int(None)");
    crate::py::text::strip(figure).parse().ok()
}

/// What to wear for the temperature given: heat, cold, or the layers in between.
pub fn _dress_for_the_temperature(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let degrees = _the_temperature_said(understood);
    match degrees {
        None => {}
        Some(d) if d >= 24 => {}
        Some(d) if d <= 6 => {
            return Some(
                "Three thin layers beat one thick one: a base layer, a jumper and a windproof coat, so you can take the \
middle off indoors. Cover the neck, ears, wrists and ankles, which is where the cold gets in, and a hat is worth more \
than it looks. Wool or a synthetic base layer stays warm where cotton goes cold and damp."
                    .to_string(),
            );
        }
        Some(_) => {
            return Some(
                "Layers you can take off, and one of them waterproof: a thin base layer, a jumper and a shell over the \
top sees you through a day that starts cold and warms up. Shoes with a sole rather than canvas, and a bag with a zip, \
because the British version of this number comes with rain."
                    .to_string(),
            );
        }
    }
    Some(
        "In real heat, loose light cotton in a pale colour is the coolest thing you can wear, because it lets air \
through and reflects the sun. A wide hat and sunglasses are worth more than another coat of sunscreen, and shoes you can \
walk in matter on hot tarmac. Dark, tight and synthetic is what makes it feel worse."
            .to_string(),
    )
}

/// Guard: "what should i put on today" is what to wear.
pub fn _today_is_the_question(understood: &Understood) -> bool {
    _put_on_is_clothes(understood)
}

/// A UK size from the last few lines in EU and US sizes (women's), or None.
pub fn _the_size_carried(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _the_line(understood);
    let said = recent(memory, 3).iter().map(|t| t.user.as_str()).collect::<Vec<_>>().join(" ").to_lowercase();
    let sizes = _A_SIZE_SAID.findall_str(&said);
    let last = sizes.last()?;
    let number: i64 = crate::py::text::strip(last).parse().expect("int()");
    if _any_word(&said, &_A_SHOE_SIZE) {
        return None;
    }
    let eu = lookup(&_WOMENS_UK_TO_EU, number)?;
    let us = lookup(&_WOMENS_UK_TO_US, number).expect("KeyError");
    if _any_word(&line, &["in the us", "in us", "in usa", "in america", "american"]) {
        return Some(format!(
            "A UK {number} is a US {us} - that is the women's conversion, and men's clothes go by chest size instead. \
A size is only a starting point either way: it drifts a whole step between brands, so go by the brand's own size guide \
and your measurements rather than the label."
        ));
    }
    Some(format!(
        "A UK {number} is a European {eu} in women's clothes, and a US {us}. Men's clothes go by chest size instead: 36 \
in chest is EU 46, 38 is 48, 40 is 50, 42 is 52. Sizes drift a whole step between brands, so check the brand's own \
size guide as well."
    ))
}

/// Laces: teaching a child, a bow, skates, ones that come undone, or the knot.
pub fn _a_lace(understood: &Understood) -> String {
    let words = lower_words(understood);
    let text = words.join(" ");
    let has = |options: &[String]| options.iter().any(|o| words.contains(o) || text.contains(o.as_str()));
    if has(&_LACE_TEACH) && !has(&_LACE_SKATES) {
        return _LACE_TEACHING.clone();
    }
    if has(&_LACE_BOW) {
        return _LACE_BOW_REPLY.clone();
    }
    if has(&_LACE_SKATES) {
        return _LACE_SKATES_REPLY.clone();
    }
    if has(&_LACE_UNDONE) {
        return _LACE_UNDONE_REPLY.clone();
    }
    _LACE_KNOT.clone()
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.clothing:<lambda>:2356", |_: &FnRef, u: &Understood| !_the_enough_is_about_a_garment(u)),
    ("bot.knowledge.clothing:<lambda>:5235", |_: &FnRef, u: &Understood| {
        _the_temperature_said(u).is_some() && _the_weather_is_the_question(u)
    }),
    ("bot.knowledge.clothing:_a_faulty_coat:733", |_: &FnRef, u: &Understood| _a_faulty_coat(u)),
    ("bot.knowledge.clothing:_a_hair_line:182", |_: &FnRef, u: &Understood| _a_hair_line(u)),
    ("bot.knowledge.clothing:_acne_product_question:862", |_: &FnRef, u: &Understood| _acne_product_question(u)),
    ("bot.knowledge.clothing:_an_outfit_shortage:1250", |_: &FnRef, u: &Understood| _an_outfit_shortage(u)),
    ("bot.knowledge.clothing:_drying_the_washing:987", |_: &FnRef, u: &Understood| _drying_the_washing(u)),
    ("bot.knowledge.clothing:_grease_is_not_a_burn:1200", |_: &FnRef, u: &Understood| _grease_is_not_a_burn(u)),
    ("bot.knowledge.clothing:_not_a_colour_choice:206", |_: &FnRef, u: &Understood| _not_a_colour_choice(u)),
    ("bot.knowledge.clothing:_not_a_pane:215", |_: &FnRef, u: &Understood| _not_a_pane(u)),
    ("bot.knowledge.clothing:_not_fitting_a_zip:111", |_: &FnRef, u: &Understood| _not_fitting_a_zip(u)),
    ("bot.knowledge.clothing:_put_on_is_clothes:154", |_: &FnRef, u: &Understood| _put_on_is_clothes(u)),
    ("bot.knowledge.clothing:_the_code_is_not_an_interview:146", |_: &FnRef, u: &Understood| {
        _the_code_is_not_an_interview(u)
    }),
    ("bot.knowledge.clothing:_the_date_night_is_a_meal:119", |_: &FnRef, u: &Understood| _the_date_night_is_a_meal(u)),
    ("bot.knowledge.clothing:_the_enough_is_about_a_garment:5179", |_: &FnRef, u: &Understood| {
        _the_enough_is_about_a_garment(u)
    }),
    ("bot.knowledge.clothing:_the_feet_are_not_a_height:1313", |_: &FnRef, u: &Understood| _the_feet_are_not_a_height(u)),
    ("bot.knowledge.clothing:_the_glass_is_a_drink:395", |_: &FnRef, u: &Understood| _the_glass_is_a_drink(u)),
    ("bot.knowledge.clothing:_the_mould_is_not_on_a_kept_food:1164", |_: &FnRef, u: &Understood| {
        _the_mould_is_not_on_a_kept_food(u)
    }),
    ("bot.knowledge.clothing:_the_weather_is_the_question:5130", |_: &FnRef, u: &Understood| {
        _the_weather_is_the_question(u)
    }),
    ("bot.knowledge.clothing:_today_is_the_question:5249", |_: &FnRef, u: &Understood| _today_is_the_question(u)),
    ("bot.knowledge.clothing:not_a_fight_about_clothes:690", |_: &FnRef, u: &Understood| not_a_fight_about_clothes(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.clothing:<lambda>:5615", |_: &FnRef, _: &mut Memory, u: &Understood| Some(_a_lace(u))),
    ("bot.knowledge.clothing:_a_garment_to_choose.<locals>.answer:4861", _a_garment_to_choose_answer),
    ("bot.knowledge.clothing:_boots_to_buy:575", |_: &FnRef, m: &mut Memory, u: &Understood| _boots_to_buy(m, u)),
    ("bot.knowledge.clothing:_child_coat:701", |_: &FnRef, m: &mut Memory, u: &Understood| _child_coat(m, u)),
    ("bot.knowledge.clothing:_dress_for_the_temperature:5202", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _dress_for_the_temperature(m, u)
    }),
    ("bot.knowledge.clothing:_drying_it:1032", |_: &FnRef, m: &mut Memory, u: &Understood| _drying_it(m, u)),
    ("bot.knowledge.clothing:_engine_oil_stain:792", |_: &FnRef, m: &mut Memory, u: &Understood| _engine_oil_stain(m, u)),
    ("bot.knowledge.clothing:_glasses_repair:404", |_: &FnRef, m: &mut Memory, u: &Understood| _glasses_repair(m, u)),
    ("bot.knowledge.clothing:_hair_colour_home:1137", |_: &FnRef, m: &mut Memory, u: &Understood| _hair_colour_home(m, u)),
    ("bot.knowledge.clothing:_iron_shirt:255", |_: &FnRef, m: &mut Memory, u: &Understood| _iron_shirt(m, u)),
    ("bot.knowledge.clothing:_jeans_to_buy:662", |_: &FnRef, m: &mut Memory, u: &Understood| _jeans_to_buy(m, u)),
    ("bot.knowledge.clothing:_leather_mark:1173", |_: &FnRef, m: &mut Memory, u: &Understood| _leather_mark(m, u)),
    ("bot.knowledge.clothing:_mould_on_it:517", |_: &FnRef, m: &mut Memory, u: &Understood| _mould_on_it(m, u)),
    ("bot.knowledge.clothing:_new_shoe_blisters:318", |_: &FnRef, m: &mut Memory, u: &Understood| _new_shoe_blisters(m, u)),
    ("bot.knowledge.clothing:_no_iron_reply:824", |_: &FnRef, m: &mut Memory, u: &Understood| _no_iron_reply(m, u)),
    ("bot.knowledge.clothing:_socks:447", |_: &FnRef, m: &mut Memory, u: &Understood| _socks(m, u)),
    ("bot.knowledge.clothing:_spectacle_glass:224", |_: &FnRef, m: &mut Memory, u: &Understood| _spectacle_glass(m, u)),
    ("bot.knowledge.clothing:_sweat_mark:928", |_: &FnRef, m: &mut Memory, u: &Understood| _sweat_mark(m, u)),
    ("bot.knowledge.clothing:_take_up_hem:637", |_: &FnRef, m: &mut Memory, u: &Understood| _take_up_hem(m, u)),
    ("bot.knowledge.clothing:_the_bad_haircut:1130", |_: &FnRef, m: &mut Memory, u: &Understood| _the_bad_haircut(m, u)),
    ("bot.knowledge.clothing:_the_size_carried:5417", |_: &FnRef, m: &mut Memory, u: &Understood| _the_size_carried(m, u)),
    ("bot.knowledge.clothing:_the_size_you_are:1302", |_: &FnRef, m: &mut Memory, u: &Understood| _the_size_you_are(m, u)),
    ("bot.knowledge.clothing:_trainers_to_buy:615", |_: &FnRef, m: &mut Memory, u: &Understood| _trainers_to_buy(m, u)),
    ("bot.knowledge.clothing:_wash_temperature:741", |_: &FnRef, m: &mut Memory, u: &Understood| _wash_temperature(m, u)),
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

    /// Python: bot/knowledge/clothing.py:4859 `_a_garment_to_choose(fit)` (never called on the trace corpus)
    pub fn _a_garment_to_choose(fit: &Value) -> Value {
        crate::unported!("bot.knowledge.clothing:_a_garment_to_choose")
    }

}

