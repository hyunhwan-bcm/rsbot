//! Port of PyBot's `bot/knowledge/garden.py`: gardens and plants - the everyday jobs that decide
//! whether things grow, what goes in when, and what is wrong with a plant that is struggling.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g5 as dup;
use crate::knowledge::d1_shared::{self as sh, LeadFn};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.knowledge.garden.";

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}
fn set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn string(name: &str) -> String {
    data::string(&format!("{M}{name}"))
}
fn get(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}
/// A `{str: str}` dict global, in insertion order.
fn str_pairs(name: &str) -> Vec<(String, String)> {
    get(name)
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
        .collect()
}
/// An `{int: str}` dict global.
fn int_str_map(name: &str) -> HashMap<i64, String> {
    get(name).pairs().iter().filter_map(|(k, v)| Some((k.as_int()?, v.as_str()?.to_string()))).collect()
}
/// A `((words, reply), ...)` table global.
fn word_rows(name: &str) -> Vec<(Vec<String>, String)> {
    get(name)
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            Some((row.first()?.strs(), row.get(1)?.as_str()?.to_string()))
        })
        .collect()
}
/// A `((str, str), ...)` table global.
fn str_rows(name: &str) -> Vec<(String, String)> {
    get(name)
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            Some((row.first()?.as_str()?.to_string(), row.get(1)?.as_str()?.to_string()))
        })
        .collect()
}
fn lookup<'a>(table: &'a [(String, String)], key: &str) -> Option<&'a String> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

static NOT_LINE_CHARS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));
static WORDS_RX: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static TOO_LATE_OR_EARLY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\btoo late\b|\btoo early\b|\blate for\b", 0));

// --- reading the line ---------------------------------------------------------------------------------

/// `_words({"clean": text})`: the text lower-cased, everything but letters, digits, apostrophes
/// and spaces made a space, with a space either side.
pub fn _words_of(text: &str) -> String {
    format!(" {} ", NOT_LINE_CHARS.sub(" ", &text.to_lowercase()))
}

/// The line (clean, else text) as one lower-case string for the small word tests below.
pub fn _words(understood: &Understood) -> String {
    _words_of(sh::clean_or_text(understood))
}

/// Whether the word (or phrase) stands in the line with no letter or digit either side
/// (`(?<![a-z0-9])word(?![a-z0-9])`).
pub fn _has(line: &str, word: &str) -> bool {
    let bytes = line.as_bytes();
    let edge = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    let mut start = 0;
    while start <= line.len() {
        let Some(i) = line[start..].find(word) else {
            return false;
        };
        let at = start + i;
        let end = at + word.len();
        if (at == 0 || !edge(bytes[at - 1])) && (end == line.len() || !edge(bytes[end])) {
            return true;
        }
        start = at + line[at..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

/// `any(_has(line, word) for word in words)`.
fn has_any<S: AsRef<str>>(line: &str, words: &[S]) -> bool {
    words.iter().any(|w| _has(line, w.as_ref()))
}

/// A single word has to be a word of the line ("bed" is not in "bedroom"); a phrase is looked for
/// in the line as it stands.
pub fn _says_one_of<S: AsRef<str>>(said: &str, words: &[S]) -> bool {
    let tokens: HashSet<String> = WORDS_RX.findall_str(said).into_iter().collect();
    words.iter().any(|one| {
        let one = one.as_ref();
        if one.contains(' ') { said.contains(one) } else { tokens.contains(one) }
    })
}

static WHEN_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_WHEN_ASKS"));

/// True when the line wants a date rather than a method: "when do i prune thyme".
pub fn _asks_when(line: &str) -> bool {
    has_any(line, &WHEN_ASKS)
}

// --- a small place, a raised bed, lawns, cuttings ------------------------------------------------------

static SMALL_PLACE: LazyLock<&'static Re> = LazyLock::new(|| rx("_SMALL_PLACE"));

/// True when "small" is about the fruit, not the garden: a small garden is the growing rows'.
pub fn _a_small_place_ask(understood: &Understood) -> bool {
    let line = format!("{} {} {}", understood.clean, understood.norm_text, understood.plain).to_lowercase();
    !SMALL_PLACE.is_match(&line)
}

static A_RAISED_BED_BOTTOM: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_RAISED_BED_BOTTOM"));
static A_RAISED_BED_GRASS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_RAISED_BED_GRASS"));
static A_RAISED_BED_PAVING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_RAISED_BED_PAVING"));
static A_RAISED_BED_GROW: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_RAISED_BED_GROW"));
static A_RAISED_BED_SOIL: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_RAISED_BED_SOIL"));
static A_RAISED_BED_AMOUNT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_RAISED_BED_AMOUNT"));
static A_RAISED_BED_DRY: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_RAISED_BED_DRY"));
static A_RAISED_BED_BUILD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_RAISED_BED_BUILD"));

/// The raised-bed question that was actually asked (how much, drying out, the bottom, on grass,
/// on paving, what to grow, the soil, what to build it from), with the depth and the mix in it.
pub fn _a_raised_bed(_memory: &Memory, understood: &Understood) -> String {
    let said = _words(understood);
    if has_any(&said, &A_RAISED_BED_AMOUNT) {
        return "Work it out as a volume: length times width times depth, all in metres, and add \
                a tenth for settling. Bulk loam-based compost from a builders' merchant is far \
                cheaper than bags, and two parts topsoil to one part compost fills it better \
                than pure compost, which sinks."
            .to_string();
    }
    if has_any(&said, &A_RAISED_BED_DRY) {
        return "A raised bed dries out much faster than the ground, because the roots are above \
                it on all four sides: water when the top 2cm is dry, a good soak twice a week in \
                summer rather than a daily sprinkle, and a mulch of compost or grass clippings \
                holds it. Thirty centimetres of soil holds far longer than fifteen."
            .to_string();
    }
    if has_any(&said, &A_RAISED_BED_BOTTOM) {
        return "Line the bottom with cardboard to smother the weeds, then fill it free: logs, \
                twigs, leaves and any garden waste you have, then two parts topsoil to one part \
                compost on top. Skip rubble, gravel and plastic, and keep it no wider than about \
                1.2m so you can reach the middle from both sides without standing on the soil."
            .to_string();
    }
    if has_any(&said, &A_RAISED_BED_GRASS) {
        return "Straight onto the grass with no digging: mow it short, lay cardboard or four \
                layers of newspaper over the whole footprint, soak it, and fill on top - the \
                grass dies underneath and the worms pull it in. Keep it no wider than about 1.2m \
                so you can reach the middle from both sides."
            .to_string();
    }
    if has_any(&said, &A_RAISED_BED_PAVING) {
        return "On slabs or concrete it needs no cardboard and no digging: stand the frame on \
                the paving, leave a gap at the bottom edge so water runs off, and be ready to \
                water more, because a bed on slabs dries out faster than one on soil. Twenty \
                centimetres of soil is enough for salad, herbs and dwarf beans."
            .to_string();
    }
    if has_any(&said, &A_RAISED_BED_GROW) {
        return "In twenty centimetres of soil: salad, herbs, radishes, dwarf beans, onions and \
                carrots. Potatoes, tomatoes, courgettes and anything tall want forty or more, \
                because their roots and their water need the depth. Keep it no wider than about \
                1.2m so you can reach the middle from both sides without standing on the soil."
            .to_string();
    }
    if has_any(&said, &A_RAISED_BED_SOIL) {
        return "Fill it with a mix of topsoil and compost rather than pure compost, which dries \
                out and sinks: two parts topsoil to one part compost, and a handful of grit if it \
                holds water. Twenty centimetres is enough for salad, herbs and dwarf beans, \
                thirty for carrots, onions and beetroot."
            .to_string();
    }
    if has_any(&said, &A_RAISED_BED_BUILD) {
        return "Untreated timber is fine and cheap and lasts about five years; tanalised timber \
                8 to 10 years; old bricks or paving slabs cost nothing and last longer, and \
                creosoted railway sleepers stay away from anything you eat. Keep it no wider than \
                about 1.2m so you can reach the middle."
            .to_string();
    }
    "Twenty centimetres deep does salad, herbs and dwarf beans; thirty for carrots, onions \
     and beetroot; forty or more for potatoes and tomatoes. Untreated timber is fine and \
     cheap, line the bottom with cardboard to smother the weeds, and fill it with a mix of \
     topsoil and compost rather than pure compost, which dries out and sinks. Keep it no \
     wider than about 1.2m so you can reach the middle from both sides without standing on \
     the soil."
        .to_string()
}

/// The cuttings answer, with the season in front when the line asks when to take one.
pub fn _cutting_reply(_memory: &Memory, understood: &Understood) -> String {
    let body = "Cut a shoot about 8 to 10cm long just below a leaf joint, strip the bottom leaves \
                off, and push it into a small pot of damp, gritty compost, then a clear bag over the \
                top holds the moisture in. Bright light, out of direct sun, and most soft cuttings - \
                geraniums, fuchsias, penstemons, herbs - root in two to four weeks; tug one gently and \
                it holds when there are roots. Geraniums, busy lizzies and pothos also root in a glass \
                of water changed every few days, and a spider plant does the job for you: the \
                plantlets on the long runners are ready to pot up once they have roots 2cm long, \
                still attached to the parent if you like.";
    if _asks_when(&_words(understood)) {
        return format!(
            "Late spring to early summer, when the new growth has firmed at the base but still \
             bends - May and June for most shrubs. {body}"
        );
    }
    body.to_string()
}

static GARDEN_IN_THE_LINE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GARDEN_IN_THE_LINE"));

/// True when this line says the garden, not when a topic from earlier turns does.
pub fn _names_the_garden(understood: &Understood) -> bool {
    let said = _words_of(&understood.text);
    has_any(&said, &GARDEN_IN_THE_LINE)
}

static KITCHEN_ROT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KITCHEN_ROT"));
static GROUND_THING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GROUND_THING"));
static LEAF_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_LEAF_WORDS"));
static A_FRUIT_THAT_ROTS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_FRUIT_THAT_ROTS"));

/// True when the line is food in the kitchen, not a crop still growing or just lifted.
pub fn _a_kitchen_thing(understood: &Understood) -> bool {
    let line = _words(understood);
    if !has_any(&line, &KITCHEN_ROT) {
        return false;
    }
    !has_any(&line, &GROUND_THING)
}

/// True when the line names the leaves and says nothing about the fruit going black.
pub fn _a_leaf_problem_not_the_fruit(understood: &Understood) -> bool {
    let line = _words(understood);
    if !has_any(&line, &LEAF_WORDS) {
        return false;
    }
    !has_any(&line, &A_FRUIT_THAT_ROTS)
}

static LAWN_MOW: LazyLock<String> = LazyLock::new(|| string("_LAWN_MOW"));
static LAWN_DEAD: LazyLock<String> = LazyLock::new(|| string("_LAWN_DEAD"));
static LAWN_COLOUR: LazyLock<String> = LazyLock::new(|| string("_LAWN_COLOUR"));
static LAWN_REST: LazyLock<String> = LazyLock::new(|| string("_LAWN_REST"));
static LAWN_GREEN: LazyLock<String> = LazyLock::new(|| string("_LAWN_GREEN"));
static LAWN_REST_SEED: LazyLock<String> = LazyLock::new(|| string("_LAWN_REST_SEED"));
static LAWN_AERATE: LazyLock<String> = LazyLock::new(|| string("_LAWN_AERATE"));
static LAWN_RESEED: LazyLock<String> = LazyLock::new(|| string("_LAWN_RESEED"));
static LAWN_SCARIFY: LazyLock<String> = LazyLock::new(|| string("_LAWN_SCARIFY"));

/// The lawn answer, in the order the line asks for it: a job named (aerate, scarify, reseed), a
/// greener lawn, patches, a dead lawn, a colour, or mowing.
pub fn _lawn(_memory: &Memory, understood: &Understood) -> String {
    let line = _words(understood);
    const COLOUR: &[&str] = &["yellow", "yellowing", "brown", "browning", "patchy", "patches", "dying", "dead", "died", "killed"];
    const GREEN: &[&str] = &[
        "green", "greener", "greenest", "lush", "thicker", "thick", "dull", "pale", "washed out", "faded", "tired",
        "yellowish", "pale green", "not very green", "thin", "feed", "feeding", "fertiliser", "fertilizer", "starved",
    ];
    if has_any(
        &line,
        &["aerate", "aerating", "aeration", "aerated", "spiking", "spike", "hollow tine", "hollow tines", "compacted", "compaction"],
    ) {
        return LAWN_AERATE.clone();
    }
    if has_any(
        &line,
        &["scarify", "scarifying", "scarified", "scarifier", "dethatch", "dethatching", "thatch", "thatching", "spongy", "raking out"],
    ) {
        return LAWN_SCARIFY.clone();
    }
    if has_any(&line, &["reseed", "reseeding", "re seed", "oversow", "over sow", "sow grass", "grass seed"]) {
        return LAWN_RESEED.clone();
    }
    if has_any(&line, GREEN) && !has_any(&line, COLOUR) {
        return [LAWN_GREEN.as_str(), &LAWN_MOW, &LAWN_REST_SEED].join(" ");
    }
    if has_any(&line, &["patch", "patches", "patchy", "in places", "here and there"]) {
        return [LAWN_COLOUR.as_str(), &LAWN_MOW, &LAWN_REST_SEED].join(" ");
    }
    if has_any(&line, &["dead", "died", "killed"]) {
        return [LAWN_DEAD.as_str(), &LAWN_REST_SEED].join(" ");
    }
    if has_any(&line, COLOUR) {
        return [LAWN_COLOUR.as_str(), &LAWN_MOW, &LAWN_REST_SEED].join(" ");
    }
    [LAWN_MOW.as_str(), &LAWN_REST].join(" ")
}

// --- harvests --------------------------------------------------------------------------------------------

static BRASSICA_HARVEST: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("BRASSICA_HARVEST"));
static HARVEST_MORE: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("HARVEST_MORE"));
static SOWING: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("SOWING"));
/// HARVEST: crop -> its harvest reply, a string or a reply function (`_bean_harvest`, ...).
static HARVEST: LazyLock<Vec<(String, Value)>> = LazyLock::new(|| {
    get("HARVEST").pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.clone()))).collect()
});
static EATEN_RAW: LazyLock<HashSet<String>> = LazyLock::new(|| set("EATEN_RAW"));
static COURGETTE_HARVEST: LazyLock<String> = LazyLock::new(|| string("COURGETTE_HARVEST"));
static PUMPKIN_HARVEST: LazyLock<String> = LazyLock::new(|| string("PUMPKIN_HARVEST"));

/// The harvest sign for the brassica the line names, or cabbage, sprouts and kale together.
pub fn _brassica_harvest(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = _words(understood);
    for crop in ["broccoli", "cauliflower", "sprouts", "sprout", "kale", "cabbage"] {
        if _has(&line, crop) {
            // `BRASSICA_HARVEST[...]`: a KeyError would decline the reply.
            return lookup(&BRASSICA_HARVEST, if crop == "sprout" { "sprouts" } else { crop }).cloned();
        }
    }
    let parts: Option<Vec<&String>> =
        ["cabbage", "sprouts", "kale"].iter().map(|c| lookup(&BRASSICA_HARVEST, c)).collect();
    Some(parts?.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "))
}

/// Every crop that has a harvest row, so a cooking method can stand down for one.
pub fn harvest_crops() -> HashSet<String> {
    let mut names = HashSet::new();
    for (words, _reply) in HARVEST_MORE.iter() {
        names.extend(words.iter().cloned());
    }
    for (words, _sow) in SOWING.iter() {
        let Some(first) = words.first() else { continue };
        if HARVEST.iter().find(|(k, _)| k == first).is_some_and(|(_, reply)| reply.truthy()) {
            names.extend(words.iter().cloned());
        }
    }
    names
}

/// A pumpkin is not a courgette that was left too long.
pub fn _courgette_harvest(_memory: &Memory, understood: &Understood) -> String {
    let line = _words(understood);
    if _has(&line, "pumpkin") || _has(&line, "pumpkins") || _has(&line, "squash") {
        return PUMPKIN_HARVEST.clone();
    }
    COURGETTE_HARVEST.clone()
}

/// Broad beans crop a month before the runners, so the line that names them leads.
pub fn _bean_harvest(_memory: &Memory, understood: &Understood) -> String {
    let line = _words(understood);
    if _has(&line, "broad") || _has(&line, "broad bean") || _has(&line, "broad beans") {
        return "Broad beans are ready from June, when the pods bulge and the beans inside are \
                the size of a thumbnail: pick from the bottom of the plant upwards, and pinch \
                out the tops once the flowers have set to stop blackfly. Runner and french \
                beans follow from July."
            .to_string();
    }
    "Runner and french beans are ready from July: pick them at 15cm while they are slim \
     and snap without a string. Broad beans are a month earlier, when the pods bulge. \
     Pick every couple of days or the plants stop setting."
        .to_string()
}

/// The done-words a harvest row avoids for a crop that is cooked ("done" is a pan word), none for
/// one eaten raw. (Import time: it builds the harvest rows' avoid lists.)
pub fn _done_means_a_pan(crop: &str) -> Vec<&'static str> {
    if EATEN_RAW.contains(crop) { vec![] } else { vec!["done"] }
}

// --- sowing and planting -------------------------------------------------------------------------------

static MONTH_NUMBER: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    get("MONTH_NUMBER").pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_int()?))).collect()
});
static MONTH_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("MONTH_PHRASES"));
static SOW_WINDOW: LazyLock<HashMap<String, (i64, i64)>> = LazyLock::new(|| {
    get("SOW_WINDOW")
        .pairs()
        .iter()
        .filter_map(|(k, v)| {
            let w = v.items();
            Some((k.as_str()?.to_string(), (w.first()?.as_int()?, w.get(1)?.as_int()?)))
        })
        .collect()
});
static SOW_MONTH: LazyLock<HashMap<i64, String>> = LazyLock::new(|| int_str_map("SOW_MONTH"));
static SEASON_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("SEASON_WORDS"));
static SEASON_REPLY: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("SEASON_REPLY"));
static NOW_WORDS: LazyLock<&'static Re> = LazyLock::new(|| rx("NOW_WORDS"));
static NEXT_MONTH_WORDS: LazyLock<&'static Re> = LazyLock::new(|| rx("NEXT_MONTH_WORDS"));
static MONTH_NAMES: LazyLock<HashMap<i64, String>> = LazyLock::new(|| int_str_map("MONTH_NAMES"));
static FOR_WINTER: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\bfor (the )?winter\b|\bover ?winter\b|\bfor christmas\b|\bbefore winter\b", 0));
static PLANT_LATE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("PLANT_LATE"));

/// The month the line names (May by its phrases, since "may" alone is a verb), or None.
pub fn _month_named(understood: &Understood) -> Option<i64> {
    let line = _words(understood);
    for (word, number) in MONTH_NUMBER.iter() {
        if _has(&line, word) {
            return Some(*number);
        }
    }
    if sh::any_in(&line, &MONTH_PHRASES) {
        return Some(5);
    }
    None
}

/// The closure `_sowing_reply(crop, reply)` makes: the row's answer, standing down (None) for a
/// month outside its sowing window, with the reassurance first for "too late" / "too early".
#[allow(non_snake_case)]
pub fn _sowing_reply__answer(early: i64, late: i64, reply: &str, _memory: &Memory, understood: &Understood) -> Option<String> {
    let line = _words(understood);
    if let Some(month) = _month_named(understood) {
        if !(early <= month && month <= late) {
            return None;
        }
    }
    if TOO_LATE_OR_EARLY.is_match(&line) {
        return Some(format!(
            "Later than the usual window it still grows, it just crops later and wants \
             watering while it catches up. {reply}"
        ));
    }
    Some(reply.to_string())
}

/// A sowing row's answer (import time): `_sowing_reply__answer` with the crop's own window.
pub fn _sowing_reply(crop: &str, reply: &str) -> impl Fn(&Memory, &Understood) -> Option<String> + use<> {
    let (early, late) = SOW_WINDOW.get(crop).copied().unwrap_or((1, 12));
    let reply = reply.to_string();
    move |memory: &Memory, understood: &Understood| _sowing_reply__answer(early, late, &reply, memory, understood)
}

/// The month it is where the person is talking to us (the pinned clock).
pub fn _this_month() -> Option<i64> {
    Some(dup::this_month() as i64)
}

/// What goes in the ground in the month the line names, in the season it names, or in the month
/// it is now ("now", "next month"); None for a line that names none of them.
pub fn _sow_month(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = _words(understood);
    if let Some(month) = _month_named(understood) {
        return SOW_MONTH.get(&month).cloned();
    }
    for season in SEASON_WORDS.iter() {
        if _has(&line, season) {
            if season == "winter" && FOR_WINTER.is_match(&line) {
                return lookup(&SEASON_REPLY, "autumn").cloned();
            }
            return lookup(&SEASON_REPLY, season).cloned();
        }
    }
    if !(NOW_WORDS.is_match(&line) || NEXT_MONTH_WORDS.is_match(&line)) {
        return None;
    }
    let mut month = _this_month()?;
    if NEXT_MONTH_WORDS.is_match(&line) {
        month = month % 12 + 1;
    }
    Some(format!("It's {}, so here's what goes in. {}", MONTH_NAMES.get(&month)?, SOW_MONTH.get(&month)?))
}

static CROP_WEEKS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("_CROP_WEEKS"));
static CROP_WEEKS_REST: LazyLock<String> = LazyLock::new(|| string("_CROP_WEEKS_REST"));

/// The weeks for the crop the line named, first, then the rest of the table.
pub fn _crop_weeks_reply(_memory: &Memory, reading: &Understood) -> String {
    let src = sh::or(sh::or(&reading.plain, &reading.norm_text), &reading.text);
    let said = format!(" {} ", src.to_lowercase());
    let words: HashSet<&str> = reading.words.iter().map(String::as_str).collect();
    for (forms, lead) in CROP_WEEKS.iter() {
        if forms.iter().any(|f| said.contains(&format!(" {f} ")) || words.contains(f.as_str())) {
            return format!("{lead} {}", *CROP_WEEKS_REST);
        }
    }
    CROP_WEEKS_REST.clone()
}

static PRUNE_STONE: LazyLock<String> = LazyLock::new(|| string("PRUNE_STONE"));
static PRUNE_APPLE: LazyLock<String> = LazyLock::new(|| string("PRUNE_APPLE"));
static PRUNE_NAMED: LazyLock<Vec<(Re, String)>> =
    LazyLock::new(|| str_rows("PRUNE_NAMED").into_iter().map(|(shape, reply)| (Re::new(&shape, 0), reply)).collect());
static STONE_FRUIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bplum|\bcherr|\bdamson", 0));

/// The winter job or the June one, whichever tree the line names, in front.
pub fn _prune_reply(_memory: &Memory, understood: &Understood) -> String {
    let line = _words(understood);
    if STONE_FRUIT.is_match(&line) {
        return format!("{} {}", *PRUNE_STONE, *PRUNE_APPLE);
    }
    format!("{} {}", *PRUNE_APPLE, *PRUNE_STONE)
}

/// The date and the cut for the shrub the line names, or the rule for any other one.
pub fn _prune_shrub(_memory: &Memory, understood: &Understood) -> String {
    let line = _words(understood);
    for (shape, reply) in PRUNE_NAMED.iter() {
        if shape.is_match(&line) {
            return reply.clone();
        }
    }
    "Shrubs want one of two dates. If it flowers before midsummer, cut it just after \
     flowering, because the buds were made last year. If it flowers later, cut it in \
     March while it is dormant, because the flowers come on the new growth. Take out \
     dead and crossing stems first, then a third of the oldest at the base."
        .to_string()
}

/// The closure `_planting_reply(name, reply)` makes when the crop has a late sentence: that
/// sentence in front for "too late" / "too early", else the sowing answer (`answer`).
#[allow(non_snake_case)]
pub fn _planting_reply__with_late(
    late: &str,
    reply: &str,
    answer: &dyn Fn(&Memory, &Understood) -> Option<String>,
    memory: &Memory,
    understood: &Understood,
) -> Option<String> {
    if TOO_LATE_OR_EARLY.is_match(&_words(understood)) {
        return Some(format!("{late}{reply}"));
    }
    answer(memory, understood)
}

/// A planting row's answer (import time): the crop rows' month window, and a late sentence that
/// fits when the crop has one.
pub fn _planting_reply(name: &str, reply: &str) -> Box<dyn Fn(&Memory, &Understood) -> Option<String> + Send + Sync> {
    let answer = _sowing_reply(name, reply);
    let late = lookup(&PLANT_LATE, name).cloned().unwrap_or_default();
    if late.is_empty() {
        return Box::new(answer);
    }
    let reply = reply.to_string();
    Box::new(move |memory: &Memory, understood: &Understood| {
        _planting_reply__with_late(&late, &reply, &answer, memory, understood)
    })
}

// --- care for a plant, ponds, trees ---------------------------------------------------------------------

static A_PLANT_IN_THE_LINE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_PLANT_IN_THE_LINE"));
static CARE_FOR_A_PERSON: LazyLock<&'static Re> = LazyLock::new(|| rx("CARE_FOR_A_PERSON"));
static CARE_FOR_A_PET: LazyLock<&'static Re> = LazyLock::new(|| rx("CARE_FOR_A_PET"));
static A_PERSON_NEEDING_CARE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_PERSON_NEEDING_CARE"));
static A_GARDEN_THING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_GARDEN_THING"));

/// False when the line is about caring for a person (or a pet) rather than for a plant.
pub fn _the_care_is_for_a_person(understood: &Understood) -> bool {
    let said = sh::clean_or_text(understood).to_lowercase().replace('\u{2019}', "'");
    if A_PLANT_IN_THE_LINE.is_match(&said) {
        return true;
    }
    if CARE_FOR_A_PERSON.is_match(&said) {
        return false;
    }
    if CARE_FOR_A_PET.is_match(&said) {
        return false;
    }
    !A_PERSON_NEEDING_CARE.is_match(&said)
}

const COLD_WORDS: &[&str] = &[
    "freeze", "freezes", "freezing", "frozen", "ice", "iced", "icy", "winter", "winters", "wintertime", "overwinter",
    "overwintering", "winterise", "winterised", "winterising", "winterize", "cold", "snow", "frost",
];
const SUMMER_POND_COLD: &[&str] = &[
    "freeze", "freezes", "freezing", "frozen", "ice", "iced", "icy", "winter", "winters", "overwinter", "winterise",
    "winterize", "cold", "snow", "frost",
];
const CHRISTMAS_TREE: &[&str] = &["christmas", "xmas", "spruce", "fir", "nordmann"];

/// False when the pond's water is not frozen and the season is not named.
pub fn _a_winter_pond_line(understood: &Understood) -> bool {
    has_any(&_words(understood), COLD_WORDS)
}

/// False when the tree in the line is a growing one in the garden.
pub fn _a_christmas_tree(understood: &Understood) -> bool {
    let line = _words(understood);
    if has_any(&line, &["christmas", "xmas", "spruce", "fir", "nordmann", "needles", "needle", "tinsel", "decorations"]) {
        return true;
    }
    !has_any(&line, &A_GARDEN_THING)
}

/// A growing tree: watering, pruning, or the choosing-and-planting that most lines ask.
pub fn _garden_tree(_memory: &Memory, understood: &Understood) -> String {
    let line = _words(understood);
    if _has(&line, "water") || _has(&line, "watering") || _has(&line, "how often") || _has(&line, "soak") {
        return "A tree that is in its first two summers needs a proper soak of about ten litres \
                a week, poured slowly round the roots so it sinks in, and more in a dry spell - \
                a sprinkle from a hose wets the top inch and the roots stay shallow and thirsty. \
                Water in the morning or the evening rather than in the middle of a hot day, and \
                aim at the soil, not the leaves. After that a tree is on its own unless there \
                are months without rain, when the leaves going dull and curling upwards is the \
                sign it wants a drink. A potted tree dries out far faster and needs checking \
                every few days in summer."
            .to_string();
    }
    if _has(&line, "prune") || _has(&line, "pruning") || _has(&line, "cut back") || _has(&line, "shape") {
        return "Prune a tree in late winter while it is dormant, when the sap is down and the \
                wound heals fast: take out the dead, the damaged and the crossing branches \
                first, then any that rub or grow in towards the middle, and leave the rest. Do \
                not top it or cut the leader out of the top, because that is what turns a tree \
                into a bush of water shoots. Take out no more than a quarter of the crown in a \
                year, cut back to a bud or a branch rather than leaving a stub, and leave \
                anything that flowers on old wood - plum, cherry, magnolia - until just after \
                it has flowered."
            .to_string();
    }
    "Choose it for the space it will need in twenty years, not the one it needs now: the \
     label gives a height and a spread, and most disappointment with trees is a tree \
     planted at a third of its final width against a fence. Keep it well away from a \
     house wall, the drains and the path, because roots go as wide as the branches, and \
     check the soil and the light where it is going: a tree that wants acid soil sulks in \
     chalk and one that wants sun goes thin in shade. Plant bare-root between November \
     and March while it is dormant - it is cheaper, and it settles far better than a \
     potted one in leaf. Dig a hole two or three times as wide as the roots and no \
     deeper than the soil mark on the trunk, spread the roots, and put the stake in \
     before the tree so the root ball is not speared. Firm it in, water it well, mulch \
     round the base but leave a clear gap against the trunk, and give it a good soak \
     every week or two through the first two summers, because that is what turns a \
     stick into a tree."
        .to_string()
}

/// True when the line is about a tree that is growing, not the one that comes indoors.
pub fn _a_garden_tree(understood: &Understood) -> bool {
    let line = _words(understood);
    if has_any(&line, CHRISTMAS_TREE) {
        return false;
    }
    has_any(&line, &["tree", "trees", "sapling", "saplings"])
}

/// False when the line is about the pond in the cold (the winter row's question).
pub fn _a_summer_pond_line(understood: &Understood) -> bool {
    !has_any(&_words(understood), SUMMER_POND_COLD)
}

/// False when the line gives the speaker the state as well as the plant ("and so am i").
pub fn _the_person_is_sad_too(understood: &Understood) -> bool {
    sh::field(understood, "state_owner") != "self"
}

// --- dry beds, blight, rot, pests, mildew, roots ------------------------------------------------------------

static DRY_IS_A_VERB: LazyLock<&'static Re> = LazyLock::new(|| rx("_DRY_IS_A_VERB"));
static THE_DRY_WHY: LazyLock<String> = LazyLock::new(|| string("THE_DRY_WHY"));
static THE_DRY_HOW_OFTEN: LazyLock<String> = LazyLock::new(|| string("THE_DRY_HOW_OFTEN"));
static WHY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:why|how come)\b", 0));

/// The reason first when the line asks why a bed dries out, the soak regime otherwise; None when
/// "dry" is something the person wants to do.
pub fn _a_bed_that_dries_out(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = _words(understood);
    if DRY_IS_A_VERB.is_match(&said) {
        return None;
    }
    if WHY.is_match(&said) {
        return Some(THE_DRY_WHY.clone());
    }
    Some(THE_DRY_HOW_OFTEN.clone())
}

/// Blight (or black spot on a rose), in the order that matters for the plant the line names.
pub fn _blight(_memory: &Memory, reading: &Understood) -> String {
    let line = _words(reading);
    if _has(&line, "rose") || _has(&line, "roses") || _has(&line, "spot") || _has(&line, "spots") {
        return "Black spot is a fungus that overwinters on the fallen leaves, so pick every \
                spotted leaf off the plant and off the ground and bin them - never the compost - \
                and it comes back much less next year. Water at the base rather than over the \
                leaves, leave space round the plant for air to move, and feed it: a rose or a \
                tomato that is growing hard shakes it off. A fungicide is the last resort, and it \
                has to go on early, before the spots are everywhere."
            .to_string();
    }
    if _has(&line, "potato") || _has(&line, "potatoes") {
        return "Blight on a potato crop shows as brown-black patches that spread fast in warm wet \
                weather, and the spores wash down to the tubers: cut the tops off at ground level \
                and bin them, then lift the potatoes a fortnight later so the skins set and the \
                spores do not reach them. Do not compost the tops. Next year grow a blight-\
                resistant variety and leave more space between rows, because a crop that dries \
                quickly after rain gets it less."
            .to_string();
    }
    "Brown-black patches with a pale edge that spread fast in warm wet weather are blight, \
     and once it is on a tomato plant the aim is slowing it, not curing it: cut off and bin \
     the affected leaves (not the compost), water at the base in the morning rather than \
     over the leaves, and take out some side shoots so the plants dry quicker. Pick the \
     fruit that is already ripening - a green tomato will ripen on a windowsill indoors - \
     and a copper fungicide slows it if it is caught early. Next year grow a blight-\
     resistant variety with more room between plants."
        .to_string()
}

/// The words of the last `keep` turns (3 by default), for a line that names no crop of its own.
pub fn _earlier_words(memory: &Memory, keep: usize) -> String {
    let turns = &memory.turns;
    let said: Vec<&str> = turns[turns.len().saturating_sub(keep)..].iter().map(|t| t.user.as_str()).collect();
    _words_of(&said.join(" "))
}

static ROT_CROPS: LazyLock<Vec<String>> = LazyLock::new(|| strs("ROT_CROPS"));

/// A fruit rotting is not a root rotting: eating the sound ones, a seedling's damping off,
/// blossom end rot, stored roots, tree fruit, or rot on the plant.
pub fn _rot(memory: &Memory, reading: &Understood) -> String {
    let mut line = _words(reading);
    if has_any(&line, &["eat", "eating", "edible", "safe", "safe to eat", "still eat", "ok to eat", "use the rest", "use them"]) {
        if has_any(&line, &["potato", "potatoes", "onion", "onions", "garlic"]) {
            return "Bin every soft, slimy or smelly one, because rot in a potato or an onion is \
                    not something to cut round: the spores travel through the flesh and the \
                    whole one tastes wrong. A green patch on a potato is solanine, so cut it off \
                    with a wide margin or bin that potato, and never eat one that has sprouted \
                    and gone soft."
                .to_string();
        }
        return "Cut out the brown with a good margin around it and the sound part is fine to eat; \
                anything soft, wet or with grey fuzz on it goes in the bin, not on the plate and \
                not in the compost. Wash what you keep and use it in the next day or two, \
                because fruit that has started to go will not keep."
            .to_string();
    }
    if !has_any(&line, &ROT_CROPS) {
        line = format!("{line} {}", _earlier_words(memory, 3));
    }
    if has_any(&line, &["seedling", "seedlings", "cutting", "cuttings", "stem", "stems"]) {
        return "A seedling that collapses at soil level has damping off: the stem goes thin and \
                water-soaked and the plant keels over. It is a fungus in cold, wet compost, so \
                sow into fresh compost, water from below and let the surface dry between \
                waterings, keep the pots warm and move the survivors into the light. Air moving \
                round them matters more than anything you can spray: take the lid off the \
                propagator as soon as the seedlings are up, and bin the collapsed ones with \
                their compost."
            .to_string();
    }
    if has_any(
        &line,
        &[
            "blossom end", "black at the bottom", "black at the end", "bottom is black", "bottom of the fruit is black",
            "flower end", "rot at the end", "rotting at the end", "end is rotting", "end rot", "rotting at the flower",
            "rot at the flower",
        ],
    ) {
        return "A black sunken patch on the bottom end of a tomato, a pepper or a courgette is \
                blossom end rot, and it is not an infection: the fruit could not get enough \
                calcium because the plant's water swung about. Water deeply and evenly - a good \
                soak twice a week rather than a sprinkle every day - and mulch to hold it, and \
                do not add calcium to the soil, because the calcium is there and the water is \
                what moves it. Take the marked fruit off so the plant stops feeding it, and the \
                later trusses come through sound."
            .to_string();
    }
    if has_any(&line, &["potato", "potatoes", "bulb", "bulbs", "onion", "onions", "garlic"]) {
        return "Sort them out now: bin anything soft, slimy or smelly, because one rotting potato \
                or onion takes the rest with it, and cut the bad bits out of the sound ones and \
                use those first. If they are still in the ground, lift them rather than leaving \
                them in wet soil, and dry them somewhere airy before they go into store. Next \
                year plant in a bed that drains - a raised bed if the plot holds water - and \
                never store a bruised one with the sound ones."
            .to_string();
    }
    if has_any(&line, &["apple", "apples", "pear", "pears", "plum", "plums"]) {
        return "Fruit rotting on the tree is usually brown rot, and the spores come from the \
                mummified fruit left hanging and off the ground from last year. Pick and bin \
                every rotten one - on the tree and under it - and do not compost them, thin the \
                clusters so the fruit has air round it, and prune out crowded branches in winter. \
                Wasps and birds start the holes, so a trap from August saves a lot of fruit."
            .to_string();
    }
    "Rot on the plant is wet and still air, so pick off every affected fruit and bin it - \
     not the compost - and take off the leaves touching the ground so air moves under the \
     plant. Water at the base in the morning rather than over the leaves, and pick what is \
     sound a little earlier than you would otherwise, since a ripe fruit on wet soil is \
     what goes next. Take the fruit with a soft brown bruise at the stalk end off as soon \
     as you see it, because it will not ripen and it spreads."
        .to_string()
}

static PEST_NOUNS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    get("PEST_NOUNS")
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            Some((row.first()?.as_str()?.to_string(), row.get(1)?.strs()))
        })
        .collect()
});

/// The pest a stretch of text names, or "" when it names none.
pub fn _pest_named(text: &str) -> String {
    for (pest, words) in PEST_NOUNS.iter() {
        if has_any(text, words) {
            return pest.clone();
        }
    }
    String::new()
}

/// The pest the person named in the last `keep` turns (3 by default), for a follow-up that
/// drops the noun ("will washing up liquid kill them").
pub fn _pest_from_turns(memory: &Memory, keep: usize) -> String {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(keep)..].iter().rev() {
        let found = _pest_named(&_words_of(&turn.user));
        if !found.is_empty() {
            return found;
        }
    }
    String::new()
}

/// The pest named in the line (or the chat) leads; None when there is none to name.
pub fn _pests(memory: &Memory, reading: &Understood) -> Option<String> {
    let line = _words(reading);
    let mut pest = _pest_named(&line);
    if pest.is_empty() {
        pest = _pest_from_turns(memory, 3);
    }
    let reply = match pest.as_str() {
        "" => return None,
        "caterpillar" => {
            "Caterpillars are easiest picked off by hand in the evening, when they come out to \
             feed, and dropped in a bucket of water - check the undersides of the leaves for the \
             yellow eggs and squash those too. A net over the crop from May to September is the \
             real fix, because the cabbage white butterfly cannot get in to lay: keep it clear \
             of the leaves and peg the edges down. Inspect every few days, since a small \
             caterpillar becomes a big one fast."
        }
        "whitefly" => {
            "Whitefly sits under the leaves and flies up in a cloud when you touch the plant. \
             Yellow sticky traps catch the adults, and a spray of soapy water - a teaspoon of \
             washing-up liquid in a litre - on the undersides of the leaves kills the young \
             ones; repeat every few days, because it does not touch the eggs. Badly hit leaves \
             come off. Check new plants before they go in the greenhouse, which is where \
             whitefly gets in."
        }
        "aphid" => {
            "Greenfly and blackfly are both aphids and get the same treatment: squash them by \
             hand or spray the undersides of the leaves and the soft growing tips with soapy \
             water - a teaspoon of washing-up liquid in a litre. Broad beans are the usual \
             blackfly crop, and the tips are the first place to look: pinch the top few inches \
             out once the flowers set. Ladybirds and their larvae will clear a lot for you, so \
             leave a marigold or a poached egg plant nearby."
        }
        "weevil" => {
            "Vine weevil is a root grub with a beetle above ground, and the beetles notch the \
             leaf edges at night. The grubs are what kills the plant: tip a pot out and look \
             for fat white C-shaped ones in the compost, then bin that compost and repot in \
             fresh. A nematode watered on in late summer or early autumn is the cleanest \
             control, and going out with a torch after dark deals with the beetles."
        }
        "scale" => {
            "Scale insects sit on stems and the undersides of leaves as little brown or white \
             shells you can flick off with a fingernail, and they leave sticky honeydew that \
             turns the leaves black. Wipe them off with a cotton bud dipped in rubbing \
             alcohol, or spray the plant with soapy water - a teaspoon of washing-up liquid in \
             a litre - a few times over a month, because the shells protect the ones \
             underneath. Check the plant that arrived last, which is usually where they came \
             in."
        }
        "mealybug" => {
            "Mealybug is the white cotton-wool fluff in the leaf joints and along the stems, \
             and it hides in the same places as scale. Paint it off with a cotton bud dipped \
             in rubbing alcohol, or spray the plant with soapy water - a teaspoon of \
             washing-up liquid in a litre - every week for a month, and take off the worst \
             leaves. It thrives in warm, still air, so give the plant more room and check the \
             plants either side of it."
        }
        _ => {
            "Squash the aphids by hand or spray them with soapy water - a teaspoon of washing-up \
             liquid in a litre - and get the undersides of the leaves and the soft new tips, which \
             is where they cluster. Ladybirds and their larvae will clear a lot for you, so plant a \
             marigold or a poached egg plant nearby. A strong jet of water knocks them off, and \
             check the plant every few days, because aphids multiply fast and curl the new leaves."
        }
    };
    Some(reply.to_string())
}

/// Powdery mildew: dry roots and still air, whichever plant has the white powder.
pub fn _mildew(_memory: &Memory, _reading: &Understood) -> String {
    "White powder on the leaves is powdery mildew, and it likes dry roots and still air: \
     water thoroughly at the base rather than sprinkling the leaves, take off the worst of \
     them, and open the plants up so the air moves. A spray of one part milk to nine parts \
     water helps a little, and courgettes, cucumbers and roses usually carry on growing \
     through it. Feed the plant and keep the water steady - mildew moves fastest on a \
     plant that has been let dry out and then flooded."
        .to_string()
}

static ROOT_NAMES: LazyLock<Vec<(Re, String)>> = LazyLock::new(|| {
    str_rows("ROOT_NAMES").into_iter().map(|(pattern, said)| (Re::new(&format!(r"\b{pattern}\b"), 0), said)).collect()
});
static BOLTED: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\bbolt(?:ed|ing)?\b|\brun to seed\b|\bgoing to seed\b|\bgone to seed\b|\bflower(?:ing|ed)?\b|\bwent up\b", 0)
});
static FORKED: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bfork(?:ed|ing)?\b|\btwo legs\b|\bdouble root", 0));
static WOODY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwood(?:y|en)\b|\btough\b|\bpithy\b", 0));

/// A root that split, forked, bolted or went woody: the root named decides the first sentence.
pub fn _root_split(_memory: &Memory, understood: &Understood) -> String {
    let text = sh::clean_or_text(understood).to_lowercase();
    let name = ROOT_NAMES.iter().find(|(r, _)| r.is_match(&text)).map(|(_, s)| s.as_str()).unwrap_or("A root crop");
    let first = if BOLTED.is_match(&text) {
        format!(
            "{name} runs to seed when a hot dry spell pushes it up in its first year, and \
             once the stem goes up the root stops swelling and turns woody in the ground."
        )
    } else if FORKED.is_match(&text) {
        format!(
            "{name} forks when the taproot meets a stone, a hard pan or fresh manure, or \
             when the seedlings were left crowded: the root splits in two to get round it."
        )
    } else if WOODY.is_match(&text) {
        format!(
            "{name} goes woody when it is left in the ground too long, and in hot dry soil \
             it happens sooner: the root keeps growing and the middle turns hard and \
             stringy."
        )
    } else {
        format!(
            "{name} splits when it takes a drink after a dry spell: the root swells faster \
             than the skin keeps up, and too much feed does the same."
        )
    };
    format!(
        "{first} Water little and often so the soil stays evenly damp rather than soaking it \
         after a dry week, thin the seedlings so the roots have room, and lift them once they \
         are a good size. The damaged ones are still fine to eat, so use those first."
    )
}

// --- crops that will not set, crops in the shade, herbs ---------------------------------------------------

/// A crop that flowers and sets no fruit (or is bitter, or small): which crop it is decides the
/// first thing to try.
pub fn _crop_setting_reply(_memory: &Memory, understood: &Understood) -> String {
    let text = sh::clean_or_text(understood).to_lowercase();
    let has = |words: &[&str]| sh::any_in(&text, words);
    if text.contains("bitter") {
        if has(&["courgette", "courgettes", "zucchini", "squash", "pumpkin", "pumpkins", "cucumber", "cucumbers"]) {
            return "A bitter cucumber, courgette or squash is stress, not a bad seed. Heat, a dry \
                    spell at the root and a plant left to grow huge all push the bitter compounds up, \
                    and they sit mostly at the stalk end, so cut that inch off and taste the \
                    rest. Water regularly at the base, pick the fruit small and often rather \
                    than leaving it to swell, and shade the plant in a heatwave."
                .to_string();
        }
        if has(&["tomato", "tomatoes"]) {
            return "A bitter tomato is usually one picked before it is fully ripe, or a plant \
                    that ran short of water while the fruit was swelling. Leave the rest on the \
                    vine until it is deep red all over, water evenly at the base, and taste a \
                    ripe one before you blame the variety."
                .to_string();
        }
        if has(&["strawberry", "strawberries"]) {
            return "A bitter strawberry is underripe: the sugar comes last, so pick only fruit \
                    that is red all over and leave the pale shoulders to colour. Water at the \
                    base and net them, because a stressed or bird-pecked fruit turns sharp."
                .to_string();
        }
        return "Bitterness in a home-grown crop is usually stress - a dry spell at the root, or \
                fruit picked before it ripened. Water evenly at the base, pick when it is fully \
                coloured, and taste a ripe one before blaming the seed."
            .to_string();
    }
    if (text.contains("small") || text.contains("tiny") || text.contains("shrivelled"))
        && !has(&["strawberry", "strawberries", "plant", "plants", "seedling", "seedlings", "seed", "seeds", "leaf", "leaves"])
    {
        return "Small fruit on a plant that flowered means it has set more than its roots can \
                fill. Water at the base so the soil stays evenly damp, feed a high-potash tomato \
                feed once the first flowers open, and pick the fruit as soon as it is ready so \
                the rest can swell. A dry spell while the fruit was setting is the commonest \
                cause, and in a pot it can happen within a day or two."
            .to_string();
    }
    if has(&["courgette", "courgettes", "zucchini", "squash", "pumpkin", "cucumber", "cucumbers"]) {
        return "Courgettes, squashes and cucumbers carry male and female flowers on the same \
                plant, and the female one has a tiny fruit behind the flower. If the little fruit \
                goes yellow and drops off, the pollen never got to it - pick a male flower, peel \
                the petals back and dab it into the middle of a female one early in the morning, \
                when the flower is fresh. Cold nights under 10C and a very hot dry spell both stop \
                the pollen working, and a plant fed hard on tomato feed grows leaves instead, so \
                water at the base and wait for the weather to pass."
            .to_string();
    }
    if has(&["runner", "french bean", "french beans", "broad bean", "broad beans", "beans"]) {
        return "Runner beans set nothing when the soil dries out at the root, which is what makes \
                the flowers drop before they are pollinated. Water at the base every evening in a \
                dry spell, a good soaking rather than a sprinkle, and mulch round the plants to \
                hold it. They also need bees, so leave a patch of flowers near the wigwam, and \
                once the pods start pick them every couple of days because a bean left to swell \
                stops the plant flowering."
            .to_string();
    }
    if has(&["chilli", "chillies", "chili", "pepper", "peppers", "aubergine", "aubergines", "eggplant", "eggplants"]) {
        let crop = if text.contains("aubergine") || text.contains("eggplant") {
            "An aubergine"
        } else if text.contains("pepper") {
            "A pepper"
        } else {
            "A chilli"
        };
        return format!(
            "{crop} drops its flowers for one of three reasons, and heat is the first. Pollen \
             goes sterile above about 32C by day or 27C at night, so shade the plant from \
             midday sun in a heatwave and keep the pot off hot paving. The second is feed - a \
             plant given nitrogen grows leaves, so switch to a high-potash tomato feed once \
             the first flowers open. The third is pollination, which barely happens in a \
             greenhouse or on a windowsill, so tap the cane or the pot at midday and move \
             pollen between the flowers with a soft brush. Nights below about 15C stop it \
             setting as well, so do not put it outside before the nights are warm."
        );
    }
    if has(&["tomato", "tomatoes"]) {
        return "A tomato with no flowers at all is still growing rather than fruiting. Too much \
                nitrogen, too little sun, and nights still below about 10C are the three usual \
                reasons, so stop any high-nitrogen feed, keep the plant in full sun, and switch \
                to a tomato feed once the first truss opens. If the flowers open and then drop, \
                the pollen is what failed - tap the cane at midday to shake it loose, water \
                evenly at the base rather than in fits, and damp the path down in a heatwave so \
                the air is not bone dry. A truss that sets one or two fruits and drops the rest is \
                usually a plant carrying more than its roots can fill."
            .to_string();
    }
    if has(&["strawberry", "strawberries"]) {
        return "Tiny strawberries are usually an old plant, and a strawberry bed is worth \
                replanting every three years from fresh runners because the yield drops off after \
                that. The other two causes are easy. Feed the plants with a high-potash feed as \
                the flowers open rather than a nitrogen one, and cut off the runners instead of \
                letting the plant spend its strength on new plants. Net them from the birds as the \
                fruit colours, and water at the base so the fruit is not sitting wet."
            .to_string();
    }
    "A vegetable that makes flowers and no fruit is usually a pollination problem before \
     it is a feeding one. Leave the flowers for the bees, and hand-pollinate under cover \
     with a soft brush or a dabbed male flower. Cold nights and very hot dry days both stop \
     the pollen, and a plant fed on nitrogen grows leaves instead of fruit, so switch to a \
     high-potash feed once the first flowers open. Water at the base and keep it even, \
     because a plant that dries out drops its flowers before they set."
        .to_string()
}

static SHADE_LEAVES: LazyLock<String> = LazyLock::new(|| string("SHADE_LEAVES"));
static SHADE_SUN_CROPS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_rows("SHADE_SUN_CROPS"));

/// A crop in the shade: the ones that ripen want six hours of sun, and say so by name.
pub fn _shade_crop_reply(_memory: &Memory, understood: &Understood) -> String {
    let text = sh::clean_or_text(understood).to_lowercase();
    let leaves = SHADE_LEAVES.as_str();
    if text.contains("potato") || text.contains("potatoes") {
        return format!(
            "Potatoes will crop in half shade, just smaller and later, so they are the one \
             root that earns a place in a bed with morning sun and four hours of it. What \
             they will not do is fill in deep shade, where the tops grow and the tubers stay \
             marble-sized. The crops that do best out of the sun are the leaves: {leaves}."
        );
    }
    let named: Vec<&String> = SHADE_SUN_CROPS.iter().filter(|(w, _)| text.contains(w.as_str())).map(|(w, _)| w).collect();
    if let Some(first) = named.first() {
        // `dict(SHADE_SUN_CROPS)[...]`: the last row with that word wins.
        let crop = SHADE_SUN_CROPS.iter().rev().find(|(w, _)| w == *first).map(|(_, s)| s.as_str()).unwrap_or("");
        return format!(
            "{crop} want six to eight hours of direct sun a day, and in shade they grow \
             leaves and set nothing - so give them the sunniest strip you have, against a \
             south wall or in a pot you can move. What does crop on three to four hours is \
             the leaf and root list: {leaves}. Sow those thinner than you would \
             in the open, and keep the slugs off, because slow growth and still damp air are \
             what slugs and mildew both like."
        );
    }
    format!(
        "For a shady bed, grow the leaf and root crops: {leaves}, and they crop \
         on three to four hours of sun a day, which is what shade gives. Sow them thinner \
         than you would in the open and keep the slugs off, because slow growth and still \
         damp air are what mildew and slugs like best. Anything that has to ripen wants six \
         hours and more, so keep tomatoes, courgettes, peppers, beans, squash and sweetcorn \
         for the sunniest strip you have."
    )
}

static SOFT_HERBS: LazyLock<Vec<String>> = LazyLock::new(|| strs("SOFT_HERBS"));
static HERB_WINTER_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HERB_WINTER_WORDS"));
static HERB_WINTER: LazyLock<String> = LazyLock::new(|| string("_HERB_WINTER"));
static HERB_SPREAD: LazyLock<String> = LazyLock::new(|| string("_HERB_SPREAD"));
static HERB_WOODY: LazyLock<String> = LazyLock::new(|| string("_HERB_WOODY"));
static HERB_WOODY_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HERB_WOODY_NAMES"));
static HERB_WHEN_WOODY: LazyLock<String> = LazyLock::new(|| string("_HERB_WHEN_WOODY"));
static HERB_WHEN_SOFT: LazyLock<String> = LazyLock::new(|| string("_HERB_WHEN_SOFT"));
static HERB_PINCH: LazyLock<String> = LazyLock::new(|| string("_HERB_PINCH"));
static HERB_LOOK: LazyLock<String> = LazyLock::new(|| string("_HERB_LOOK"));

/// Which herb question this is: through the winter, a mint that has taken over, a woody stem,
/// when to cut, how to pinch, or the care; None when no herb is named.
pub fn _herbs(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = _words(understood);
    let named = has_any(&line, &SOFT_HERBS) || _has(&line, "herb") || _has(&line, "herbs");
    let mint = _has(&line, "mint");
    if named && has_any(&line, &HERB_WINTER_WORDS) && !has_any(&line, &["grow", "growing", "plant", "planting", "sow", "sowing"]) {
        return Some(HERB_WINTER.clone());
    }
    if mint
        && has_any(
            &line,
            &[
                "spread", "spreads", "spreading", "spread out", "taken over", "took over", "taking over", "everywhere",
                "invasive", "contain", "control", "stop it", "keep it in", "border",
            ],
        )
    {
        return Some(HERB_SPREAD.clone());
    }
    if named && has_any(&line, &["woody", "wooden", "leggy", "flowered", "flowering", "flower", "flowers", "bolted", "bolting", "gone to seed"]) {
        return Some(HERB_WOODY.clone());
    }
    const CUTTING: &[&str] = &[
        "prune", "pruning", "pinch", "pinching", "tip", "tips", "top", "topping", "trim", "trimming", "cut", "cutting", "snip",
        "snipping", "back", "harvest", "harvesting", "pick", "picking",
    ];
    if named && _asks_when(&line) && has_any(&line, CUTTING) {
        if has_any(&line, &HERB_WOODY_NAMES) {
            return Some(HERB_WHEN_WOODY.clone());
        }
        return Some(HERB_WHEN_SOFT.clone());
    }
    if named && has_any(&line, CUTTING) {
        return Some(HERB_PINCH.clone());
    }
    if named {
        return Some(HERB_LOOK.clone());
    }
    None
}

static SEEDLING_LEGGY: LazyLock<String> = LazyLock::new(|| string("_SEEDLING_LEGGY"));
static CACTUS_SOFT: LazyLock<String> = LazyLock::new(|| string("_CACTUS_SOFT"));

/// Leggy seedlings (not enough light); None for any other seedling line.
pub fn _seedlings(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = _words(understood);
    if has_any(
        &line,
        &["leggy", "spindly", "straggly", "stretched", "stretching", "tall and thin", "fallen over", "falling over", "flopped", "flopping"],
    ) {
        return Some(SEEDLING_LEGGY.clone());
    }
    None
}

/// A cactus gone soft is rotting; None for any other cactus line.
pub fn _cactus_soft(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = _words(understood);
    if has_any(&line, &["soft", "squishy", "soggy", "mushy", "translucent", "rotting", "rotted", "collapse", "collapsed"]) {
        return Some(CACTUS_SOFT.clone());
    }
    None
}

/// The colour answer for a hydrangea colour question, the care answer for everything else.
pub fn _hydrangea(_memory: &Memory, reading: &Understood) -> String {
    let line = sh::clean_or_text(reading).to_lowercase();
    let words = text::split_ref(&line);
    const COLOUR: &[&str] = &[
        "blue", "pink", "purple", "colour", "color", "colourful", "acid", "alkaline", "lime", "limy", "ph", "soil", "ericaceous",
    ];
    if COLOUR.iter().any(|w| words.contains(w)) {
        return "The colour is the soil rather than the plant: acid soil gives blue, limy soil \
                gives pink, and a pot is whatever compost you fill it with. Water in sulphur or an \
                ericaceous feed through the season for blue, or add garden lime for pink, and give \
                it a year, because it is next season's flowers that change. Water it well in dry \
                spells as well - wilting in the afternoon sun is thirst, and the buds for next \
                year dry up if it is left."
            .to_string();
    }
    "Water a hydrangea well in dry spells, because wilting in the afternoon sun is thirst \
     and the buds for next year dry up if it is left. The colour is the soil: acid soil \
     gives blue, limy soil gives pink, so water in a sulphur or ericaceous feed for blue, \
     and add garden lime for pink. Prune a mophead in early spring, taking the old flower \
     heads and about a third of last year's stems, and leave a lacecap and a climbing \
     hydrangea alone apart from the dead heads."
        .to_string()
}

static AVOIDING_A_STING: LazyLock<&'static Re> = LazyLock::new(|| rx("_AVOIDING_A_STING"));
static AVOID_THE_STING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_AVOID_THE_STING"));
static STUNG: LazyLock<Vec<String>> = LazyLock::new(|| strs("STUNG"));
static WASP_AVOID: LazyLock<String> = LazyLock::new(|| string("_WASP_AVOID"));
static WASP_STUNG: LazyLock<String> = LazyLock::new(|| string("_WASP_STUNG"));

/// What to do about a wasp near you, or about the sting it already gave.
pub fn _wasp_near(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::or(&understood.clean, &understood.norm_text).to_lowercase();
    if AVOIDING_A_STING.is_match(&said) {
        return WASP_AVOID.clone();
    }
    if sh::any_in(&said, &AVOID_THE_STING) {
        return WASP_AVOID.clone();
    }
    if sh::any_in(&said, &STUNG) {
        return WASP_STUNG.clone();
    }
    WASP_AVOID.clone()
}

// --- rooms, compost, the cold, bare stems, digging -----------------------------------------------------------

static A_ROOM_OR_WALL: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_ROOM_OR_WALL"));
static SOMETHING_TO_GROW_IN: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SOMETHING_TO_GROW_IN"));
static NO_COOKED_FOOD: LazyLock<String> = LazyLock::new(|| string("NO_COOKED_FOOD"));
static NAMED_A_MATERIAL: LazyLock<&'static Re> = LazyLock::new(|| rx("_NAMED_A_MATERIAL"));

/// True unless the line names a room and nothing in that room to grow a flower in.
pub fn _has_something_to_grow_in(reading: &Understood) -> bool {
    let said = text::squash(sh::or(&reading.clean, &reading.norm_text)).to_lowercase();
    if !_says_one_of(&said, &A_ROOM_OR_WALL) {
        return true;
    }
    _says_one_of(&said, &SOMETHING_TO_GROW_IN)
}

/// A lead: the "no cooked food or meat" sentence for a line that names a material, else "".
pub fn _the_no_cooked_food(understood: &Understood) -> String {
    let said = text::squash(&sh::clean_or_text(understood).to_lowercase());
    if NAMED_A_MATERIAL.is_match(&said) { NO_COOKED_FOOD.clone() } else { String::new() }
}

static COMPOST_WHAT_IS: LazyLock<Vec<String>> = LazyLock::new(|| strs("COMPOST_WHAT_IS"));
static COMPOST_MEANS: LazyLock<String> = LazyLock::new(|| string("COMPOST_MEANS"));
static COMPOST_JOB: LazyLock<String> = LazyLock::new(|| string("COMPOST_JOB"));
static COMPOST_AILING: LazyLock<Vec<String>> = LazyLock::new(|| strs("COMPOST_AILING"));
static COMPOST_FROZEN: LazyLock<Vec<String>> = LazyLock::new(|| strs("COMPOST_FROZEN"));
static COMPOST_IN_THE_COLD: LazyLock<String> = LazyLock::new(|| string("COMPOST_IN_THE_COLD"));
static COMPOST_SMELLY: LazyLock<Vec<String>> = LazyLock::new(|| strs("COMPOST_SMELLY"));
static COMPOST_TOO_WET: LazyLock<String> = LazyLock::new(|| string("COMPOST_TOO_WET"));
static COMPOST_STALLED: LazyLock<String> = LazyLock::new(|| string("COMPOST_STALLED"));

/// What compost is, what is wrong with a heap (frozen, smelly, stalled), or how to make it.
pub fn compost_reply(_memory: &Memory, reading: &Understood) -> String {
    let line = sh::text_or_clean(reading).to_lowercase();
    if sh::any_in(&line, &COMPOST_WHAT_IS) {
        return format!("{}{}", *COMPOST_MEANS, *COMPOST_JOB);
    }
    if _says_one_of(&line, &COMPOST_AILING) {
        if _says_one_of(&line, &COMPOST_FROZEN) {
            return COMPOST_IN_THE_COLD.clone();
        }
        if _says_one_of(&line, &COMPOST_SMELLY) {
            return COMPOST_TOO_WET.clone();
        }
        return COMPOST_STALLED.clone();
    }
    COMPOST_JOB.clone()
}

static THE_GARDEN_IN_THE_COLD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_THE_GARDEN_IN_THE_COLD"));
static BARE_STEMS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BARE_STEMS"));
static BARE_LOSS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BARE_LOSS"));
static A_PART_OF_THE_PLANT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_PART_OF_THE_PLANT"));
static A_COLD_THAT_IS_AN_ILLNESS: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_COLD_THAT_IS_AN_ILLNESS"));
static DIG_OVER_WHEN: LazyLock<String> = LazyLock::new(|| string("_DIG_OVER_WHEN"));
static DIG_OVER_METHOD: LazyLock<String> = LazyLock::new(|| string("_DIG_OVER_METHOD"));

/// True when the line is about what is growing, not a person who is cold.
pub fn _the_cold_is_the_garden_s(reading: &Understood) -> bool {
    let said = text::squash(sh::clean_or_text(reading)).to_lowercase();
    _says_one_of(&said, &THE_GARDEN_IN_THE_COLD)
}

/// True when the line is about the plant's stems being bare, not about a lost thing.
pub fn _the_stems_are_what_is_bare(reading: &Understood) -> bool {
    let said = text::squash(sh::clean_or_text(reading)).to_lowercase();
    if _says_one_of(&said, &BARE_STEMS) {
        return true;
    }
    _says_one_of(&said, &BARE_LOSS) && _says_one_of(&said, &A_PART_OF_THE_PLANT)
}

/// False when the "cold" in the line is a virus rather than the weather.
pub fn _the_cold_is_not_an_illness(understood: &Understood) -> bool {
    let line = text::squash(&sh::clean_or_text(understood).to_lowercase());
    !A_COLD_THAT_IS_AN_ILLNESS.is_match(&line)
}

/// The date first when the line asks when to dig a bed over, the spade first when it asks how.
pub fn _dig_over_bed(_memory: &Memory, understood: &Understood) -> String {
    let line = _words(understood);
    if _asks_when(&line) && _has(&line, "dig") {
        return DIG_OVER_WHEN.clone();
    }
    DIG_OVER_METHOD.clone()
}

// --- roses, the season's jobs, logs, frost ------------------------------------------------------------------

/// The rose answer the line is asking for: how much water, what to feed it, or why it is sulking
/// (both the cut and the planted rose when the line does not say which).
pub fn _rose_care(_memory: &Memory, understood: &Understood) -> String {
    let line = _words(understood);
    if _has(&line, "water") || _has(&line, "watering") || _has(&line, "how often") || _has(&line, "hosepipe") {
        return "A rose in the ground wants a full watering can per plant each week in dry weather, \
                poured at the base rather than over the leaves, and a rose in a pot wants that \
                much every other day in summer, because a pot dries out far faster than the soil. \
                Water in the morning, and give it a proper soak: a light sprinkle every evening \
                only wets the top inch, and the roots stay shallow and thirsty."
            .to_string();
    }
    if _has(&line, "feed") || _has(&line, "feeding") || _has(&line, "fertiliser") || _has(&line, "fertilizer") {
        return "Feed a rose with a rose feed, not a general one, from the first leaves in spring \
                until the end of July: a handful of granules round the base in March and again in \
                June, watered in, or a liquid feed every fortnight while it flowers. Stop by \
                August so the new growth hardens before winter, and never feed a dry plant - \
                water it first, or the feed burns the roots."
            .to_string();
    }
    if !has_any(
        &line,
        &[
            "ground", "garden", "bush", "bushes", "border", "pot", "soil", "beds", "bed", "shrub", "climbing", "patio", "plotted",
            "planted", "allotment", "leaves", "leaf", "stems", "stem", "roots", "root", "buds", "bud", "shoots", "hedge",
        ],
    ) {
        return "Cut roses die in a day or two for a simple reason: trim 2cm off each stem at an \
                angle, strip any leaf that would sit in the water, change the water every other \
                day and keep them out of sun. A rose in the ground that looks dead is usually \
                thirsty or hungry instead: a full can a week at the base, a rose feed while it \
                flowers, and cut the brown stems back to green wood - one dry fortnight drops \
                every leaf before the plant is dead."
            .to_string();
    }
    "A rose in the ground is nearly always thirsty, hungry or short of air: water it at the \
     base, a full can a week in dry spells, more in a pot, and feed it with a rose feed \
     while it flowers. Cut out the yellowing and spotted leaves and bin them rather than \
     compost them, because black spot and rust live on the fallen leaves and splash back up \
     in the rain. If the stems themselves are brown and dry, prune those back to green wood \
     and water: a rose left dry for a fortnight drops its leaves and looks dead before it \
     is."
        .to_string()
}

static GARDEN_SEASON_WORDS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    get("_GARDEN_SEASON_WORDS").pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.strs()))).collect()
});
static GARDEN_NOW: LazyLock<&'static Re> = LazyLock::new(|| rx("_GARDEN_NOW"));
static GARDEN_MONTHS: LazyLock<HashMap<i64, String>> = LazyLock::new(|| int_str_map("_GARDEN_MONTHS"));
static GARDEN_THINGS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GARDEN_THINGS"));
static GARDEN_JOBS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GARDEN_JOBS"));
static JOBS_IN: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("JOBS_IN"));
const SEASONS: [&str; 4] = ["spring", "summer", "autumn", "winter"];

/// `_GARDEN_SEASON_WORDS[season]` (a KeyError would raise).
fn season_words(season: &str) -> &'static [String] {
    GARDEN_SEASON_WORDS
        .iter()
        .find(|(k, _)| k == season)
        .map(|(_, v)| v.as_slice())
        .unwrap_or_else(|| panic!("KeyError: {season}"))
}

/// The season a garden line is about: the one it names, else the one it is now, else None.
pub fn _the_season_named(understood: &Understood) -> Option<String> {
    let line = _words(understood);
    for season in SEASONS {
        if has_any(&line, season_words(season)) {
            return Some(season.to_string());
        }
    }
    if GARDEN_NOW.is_match(&line) {
        if let Some(month) = _this_month() {
            return Some(GARDEN_MONTHS.get(&month).cloned().unwrap_or_else(|| panic!("KeyError: {month}")));
        }
    }
    None
}

/// True when the line, or the topic the reading holds, is about the garden.
pub fn _a_garden_line(understood: &Understood) -> bool {
    let line = _words(understood);
    if has_any(&line, &GARDEN_THINGS) {
        return true;
    }
    understood.topic.as_deref() == Some("garden-and-plants") || understood.topics.iter().any(|t| t == "garden-and-plants")
}

/// What needs doing in the garden in the season the line names (or the season it is now); None
/// for another ask.
pub fn _garden_jobs(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = _words(understood);
    if !has_any(&line, &GARDEN_JOBS) {
        return None;
    }
    if !_a_garden_line(understood) {
        return None;
    }
    let season = _the_season_named(understood)?;
    // `JOBS_IN[season]`: a KeyError would decline the reply.
    lookup(&JOBS_IN, &season).cloned()
}

/// True for a garden job ask that says "now" instead of naming a season.
pub fn _an_ask_about_now(understood: &Understood) -> bool {
    let line = _words(understood);
    if !GARDEN_NOW.is_match(&line) {
        return false;
    }
    if !has_any(&line, &GARDEN_JOBS) {
        return false;
    }
    if !_a_garden_line(understood) {
        return false;
    }
    for season in SEASONS {
        if has_any(&line, season_words(season)) {
            return false;
        }
    }
    _the_season_named(understood).is_some()
}

/// The winter card answers the lines that are about winter; a "now" ask is the season row's.
pub fn _not_about_now(understood: &Understood) -> bool {
    !_an_ask_about_now(understood)
}

static OTHER_WOOD: LazyLock<&'static Re> = LazyLock::new(|| rx("_OTHER_WOOD"));
static RING_IS_NOT_WOOD: LazyLock<&'static Re> = LazyLock::new(|| rx("_RING_IS_NOT_WOOD"));
static WOOD_WORDS: LazyLock<&'static Re> = LazyLock::new(|| rx("_WOOD_WORDS"));
static SPLITTING: LazyLock<&'static Re> = LazyLock::new(|| rx("_SPLITTING"));
static RINGS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\brings?\b", 0));
static ROUNDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\brounds?\b", 0));
static FROST_ALREADY: LazyLock<Vec<String>> = LazyLock::new(|| strs("_FROST_ALREADY"));
static FROST_AFTERMATH: LazyLock<Vec<String>> = LazyLock::new(|| strs("_FROST_AFTERMATH"));

/// True when the line is about a round of wood, not about going round something (or a ring on a
/// hob or a finger).
pub fn _splitting_a_log(understood: &Understood) -> bool {
    let said = format!("{} {} {}", understood.clean, understood.norm_text, understood.text).to_lowercase();
    if RINGS.is_match(&said) && !OTHER_WOOD.is_match(&said) && RING_IS_NOT_WOOD.is_match(&said) {
        return false;
    }
    if WOOD_WORDS.is_match(&said) {
        return true;
    }
    SPLITTING.is_match(&said) && ROUNDS.is_match(&said)
}

/// True when the line is about something growing that the frost has already damaged.
pub fn _the_frost_already_bit_something(understood: &Understood) -> bool {
    let said = sh::clean_or_text(understood).to_lowercase().replace('\u{2019}', "'");
    _says_one_of(&said, &FROST_ALREADY) && _says_one_of(&said, &FROST_AFTERMATH)
}

// --- entry lambdas --------------------------------------------------------------------------------------

/// The crop-rot row's guard: a rot word on kitchen food is the kitchen's question.
pub fn lambda_4168(understood: &Understood) -> bool {
    !_a_kitchen_thing(understood)
}

/// The blossom-end-rot row's guard: a leaf problem is the yellow-leaves row's.
pub fn lambda_4634(understood: &Understood) -> bool {
    !_a_leaf_problem_not_the_fruit(understood)
}

// --- registry wrappers ----------------------------------------------------------------------------------

macro_rules! guard {
    ($name:ident, $f:path) => {
        fn $name(_f: &FnRef, u: &Understood) -> bool {
            $f(u)
        }
    };
}
macro_rules! reply {
    ($name:ident, $f:path) => {
        fn $name(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
            Some($f(m, u))
        }
    };
    ($name:ident, $f:path, opt) => {
        fn $name(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
            $f(m, u)
        }
    };
}

guard!(g_lambda_4168, lambda_4168);
guard!(g_lambda_4634, lambda_4634);
guard!(g_a_christmas_tree, _a_christmas_tree);
guard!(g_a_garden_tree, _a_garden_tree);
guard!(g_a_small_place_ask, _a_small_place_ask);
guard!(g_a_summer_pond_line, _a_summer_pond_line);
guard!(g_a_winter_pond_line, _a_winter_pond_line);
guard!(g_has_something_to_grow_in, _has_something_to_grow_in);
guard!(g_names_the_garden, _names_the_garden);
guard!(g_not_about_now, _not_about_now);
guard!(g_splitting_a_log, _splitting_a_log);
guard!(g_the_care_is_for_a_person, _the_care_is_for_a_person);
guard!(g_the_cold_is_not_an_illness, _the_cold_is_not_an_illness);
guard!(g_the_cold_is_the_garden_s, _the_cold_is_the_garden_s);
guard!(g_the_frost_already_bit_something, _the_frost_already_bit_something);
guard!(g_the_person_is_sad_too, _the_person_is_sad_too);
guard!(g_the_stems_are_what_is_bare, _the_stems_are_what_is_bare);

reply!(r_a_bed_that_dries_out, _a_bed_that_dries_out, opt);
reply!(r_a_raised_bed, _a_raised_bed);
reply!(r_bean_harvest, _bean_harvest);
reply!(r_blight, _blight);
reply!(r_brassica_harvest, _brassica_harvest, opt);
reply!(r_cactus_soft, _cactus_soft, opt);
reply!(r_courgette_harvest, _courgette_harvest);
reply!(r_crop_setting_reply, _crop_setting_reply);
reply!(r_crop_weeks_reply, _crop_weeks_reply);
reply!(r_cutting_reply, _cutting_reply);
reply!(r_dig_over_bed, _dig_over_bed);
reply!(r_garden_jobs, _garden_jobs, opt);
reply!(r_garden_tree, _garden_tree);
reply!(r_herbs, _herbs, opt);
reply!(r_hydrangea, _hydrangea);
reply!(r_lawn, _lawn);
reply!(r_mildew, _mildew);
reply!(r_pests, _pests, opt);
reply!(r_prune_reply, _prune_reply);
reply!(r_prune_shrub, _prune_shrub);
reply!(r_root_split, _root_split);
reply!(r_rose_care, _rose_care);
reply!(r_rot, _rot);
reply!(r_seedlings, _seedlings, opt);
reply!(r_shade_crop_reply, _shade_crop_reply);
reply!(r_sow_month, _sow_month, opt);
reply!(r_wasp_near, _wasp_near);
reply!(r_compost_reply, compost_reply);

/// `_sowing_reply.<locals>.answer` from its captured `early`, `late` and `reply`.
fn sowing_answer_of(f: &FnRef, m: &Memory, u: &Understood) -> Option<String> {
    let early = f.cell("early")?.as_int()?;
    let late = f.cell("late")?.as_int()?;
    let reply = f.cell("reply")?.as_str()?;
    _sowing_reply__answer(early, late, reply, m, u)
}

fn r_sowing_reply_answer(f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    sowing_answer_of(f, m, u)
}

fn r_planting_reply_with_late(f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    let late = f.cell("late")?.as_str()?;
    let reply = f.cell("reply")?.as_str()?;
    let answer = f.cell("answer")?.as_fn()?;
    let run_answer = |memory: &Memory, understood: &Understood| -> Option<String> {
        if answer.name() == "bot.knowledge.garden:_sowing_reply.<locals>.answer" {
            sowing_answer_of(answer, memory, understood)
        } else {
            let mut memory = memory.clone();
            Some(crate::knowledge::registry::reply(answer, &mut memory, understood))
        }
    };
    _planting_reply__with_late(late, reply, &run_answer, m, u)
}

fn l_the_no_cooked_food(_f: &FnRef, u: &Understood) -> String {
    _the_no_cooked_food(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.garden:<lambda>:4168", g_lambda_4168),
    ("bot.knowledge.garden:<lambda>:4634", g_lambda_4634),
    ("bot.knowledge.garden:_a_christmas_tree:1519", g_a_christmas_tree),
    ("bot.knowledge.garden:_a_garden_tree:1572", g_a_garden_tree),
    ("bot.knowledge.garden:_a_small_place_ask:27", g_a_small_place_ask),
    ("bot.knowledge.garden:_a_summer_pond_line:1580", g_a_summer_pond_line),
    ("bot.knowledge.garden:_a_winter_pond_line:1504", g_a_winter_pond_line),
    ("bot.knowledge.garden:_has_something_to_grow_in:2427", g_has_something_to_grow_in),
    ("bot.knowledge.garden:_names_the_garden:229", g_names_the_garden),
    ("bot.knowledge.garden:_not_about_now:6530", g_not_about_now),
    ("bot.knowledge.garden:_splitting_a_log:6822", g_splitting_a_log),
    ("bot.knowledge.garden:_the_care_is_for_a_person:1482", g_the_care_is_for_a_person),
    ("bot.knowledge.garden:_the_cold_is_not_an_illness:2630", g_the_cold_is_not_an_illness),
    ("bot.knowledge.garden:_the_cold_is_the_garden_s:2539", g_the_cold_is_the_garden_s),
    ("bot.knowledge.garden:_the_frost_already_bit_something:6992", g_the_frost_already_bit_something),
    ("bot.knowledge.garden:_the_person_is_sad_too:1593", g_the_person_is_sad_too),
    ("bot.knowledge.garden:_the_stems_are_what_is_bare:2570", g_the_stems_are_what_is_bare),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.garden:_a_bed_that_dries_out:1751", r_a_bed_that_dries_out),
    ("bot.knowledge.garden:_a_raised_bed:127", r_a_raised_bed),
    ("bot.knowledge.garden:_bean_harvest:412", r_bean_harvest),
    ("bot.knowledge.garden:_blight:1767", r_blight),
    ("bot.knowledge.garden:_brassica_harvest:341", r_brassica_harvest),
    ("bot.knowledge.garden:_cactus_soft:2307", r_cactus_soft),
    ("bot.knowledge.garden:_courgette_harvest:404", r_courgette_harvest),
    ("bot.knowledge.garden:_crop_setting_reply:2032", r_crop_setting_reply),
    ("bot.knowledge.garden:_crop_weeks_reply:1007", r_crop_weeks_reply),
    ("bot.knowledge.garden:_cutting_reply:205", r_cutting_reply),
    ("bot.knowledge.garden:_dig_over_bed:2637", r_dig_over_bed),
    ("bot.knowledge.garden:_garden_jobs:6496", r_garden_jobs),
    ("bot.knowledge.garden:_garden_tree:1534", r_garden_tree),
    ("bot.knowledge.garden:_herbs:2256", r_herbs),
    ("bot.knowledge.garden:_hydrangea:2318", r_hydrangea),
    ("bot.knowledge.garden:_lawn:277", r_lawn),
    ("bot.knowledge.garden:_mildew:1981", r_mildew),
    ("bot.knowledge.garden:_pests:1924", r_pests),
    ("bot.knowledge.garden:_planting_reply.<locals>.with_late:1266", r_planting_reply_with_late),
    ("bot.knowledge.garden:_prune_reply:1161", r_prune_reply),
    ("bot.knowledge.garden:_prune_shrub:1227", r_prune_shrub),
    ("bot.knowledge.garden:_root_split:1999", r_root_split),
    ("bot.knowledge.garden:_rose_care:6177", r_rose_care),
    ("bot.knowledge.garden:_rot:1817", r_rot),
    ("bot.knowledge.garden:_seedlings:2298", r_seedlings),
    ("bot.knowledge.garden:_shade_crop_reply:2155", r_shade_crop_reply),
    ("bot.knowledge.garden:_sow_month:701", r_sow_month),
    ("bot.knowledge.garden:_sowing_reply.<locals>.answer:644", r_sowing_reply_answer),
    ("bot.knowledge.garden:_wasp_near:2384", r_wasp_near),
    ("bot.knowledge.garden:compost_reply:2506", r_compost_reply),
];

pub const LEADS: &[(&str, LeadFn)] = &[("bot.knowledge.garden:_the_no_cooked_food:2499", l_the_no_cooked_food)];


// ---------------------------------------------------------------------------------------------
// The row builders (`knowledge::built`).

use crate::knowledge::built;
use crate::knowledge::entry::{Entry, KeyPart, Reply};

fn g(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}

/// `_harvest_entries()`: one row per crop, keyed the same way as the sowing row but only for a
/// harvest question.
pub fn _harvest_entries() -> Vec<Entry> {
    let keys = g("HARVEST_KEYS").strs();
    let ask: Vec<KeyPart> = g("HARVEST_ASK").strs().into_iter().map(KeyPart::Word).collect();
    let row = |words: &[String], reply: &Value| {
        let pan: Vec<String> = _done_means_a_pan(&words[0]).into_iter().map(str::to_string).collect();
        built::entry(
            &format!("harvest-{}", words[0]),
            vec![built::part(words), built::part(&keys)],
            built::reply_of(reply),
            &[g("SOW_NOT").clone(), g("DONE_NOT").clone(), built::words(&pan)],
            3,
            ask.clone(),
            None,
        )
    };
    let harvest = g("HARVEST");
    let mut out = Vec::new();
    for sow in g("SOWING").items() {
        let words = sow.items()[0].strs();
        let Some(reply) = harvest.get(&words[0]).filter(|r| r.truthy()) else { continue };
        out.push(row(&words, reply));
    }
    for more in g("HARVEST_MORE").items() {
        out.push(row(&more.items()[0].strs(), &more.items()[1]));
    }
    out
}

/// `_plant_entries()`: one entry per plant in `PLANT_CARE`.
pub fn _plant_entries() -> Vec<Entry> {
    let words = g("PLANT_CARE_WORDS").strs();
    g("PLANT_CARE")
        .items()
        .iter()
        .map(|row| {
            let r = row.items();
            built::entry(
                &format!("plant-{}", r[0].str()),
                vec![built::part(&r[1].strs()), built::part(&words)],
                built::reply_of(&r[2]),
                &[g("LIGHT_QUESTION").clone()],
                1,
                vec![],
                Some(built::function("bot.knowledge.garden:_the_care_is_for_a_person:1482")),
            )
        })
        .collect()
}

#[cfg(test)]
mod factory_tests {
    #[test]
    fn rows_rebuild() {
        crate::data::load();
        crate::knowledge::built::assert_rows_match(&super::_harvest_entries(), "bot.knowledge.garden.ENTRIES");
        crate::knowledge::built::assert_rows_match(&super::_plant_entries(), "bot.knowledge.garden.ENTRIES");
    }
}
