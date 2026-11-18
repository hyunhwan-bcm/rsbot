//! Port of PyBot's `bot/knowledge/materials.py`: how much of a thing to buy (gravel, topsoil,
//! compost, sand, turf, paving slabs, underlay, grout, tile adhesive, flooring, concrete), worked
//! out from the size the line gives.
use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g6::{pct_d, py_floordiv, py_round};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

const M: &str = "bot.knowledge.materials.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}

static _NUM: LazyLock<String> = LazyLock::new(|| g("_NUM").str().to_string());
static _PAIR: LazyLock<&'static Re> = LazyLock::new(|| rx("_PAIR"));
static _SQUARE: LazyLock<&'static Re> = LazyLock::new(|| rx("_SQUARE"));
static _FEET: LazyLock<&'static Re> = LazyLock::new(|| rx("_FEET"));
static _UNIT: LazyLock<&'static Re> = LazyLock::new(|| rx("_UNIT"));
static _DEPTH: LazyLock<&'static Re> = LazyLock::new(|| rx("_DEPTH"));
static _A_PERSONS_WEIGHT: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_PERSONS_WEIGHT"));
static _A_MATERIAL_WEIGHT: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_MATERIAL_WEIGHT"));
static _CONCRETE_JOB: LazyLock<&'static Re> = LazyLock::new(|| rx("_CONCRETE_JOB"));
static _CONCRETE_SURE: LazyLock<Vec<String>> = LazyLock::new(|| g("_CONCRETE_SURE").strs());
static _LENGTH_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| g("_LENGTH_NAMES").strs());
static _SLAB: LazyLock<Vec<String>> = LazyLock::new(|| g("_SLAB").strs());
static _LAST_TURNS: LazyLock<usize> = LazyLock::new(|| g("_LAST_TURNS").as_int().unwrap_or(3) as usize);

/// One `_LOOSE` row: names, the depth the rate is quoted at (mm), what a bag covers there (m²),
/// the lead sentence and the extra note.
struct Loose {
    names: Vec<String>,
    depth: f64,
    covers: f64,
    lead: String,
    extra: String,
}

static _LOOSE: LazyLock<Vec<Loose>> = LazyLock::new(|| {
    g("_LOOSE")
        .items()
        .iter()
        .map(|row| {
            let r = row.items();
            Loose {
                names: r[0].strs(),
                depth: r[1].as_f64().unwrap_or(0.0),
                covers: r[2].as_f64().unwrap_or(0.0),
                lead: r[3].str().to_string(),
                extra: r[4].str().to_string(),
            }
        })
        .collect()
});

static _BAG_WORD: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("_BAG_WORD").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});

static _BULK_WEIGHT: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    g("_BULK_WEIGHT").items().iter().map(|row| (row.items()[0].strs(), row.items()[1].str().to_string())).collect()
});

/// `_A_NUMBER[n]`: one to nine as words.
fn a_number(n: i64) -> Option<&'static str> {
    g("_A_NUMBER").pairs().iter().find(|(k, _)| k.as_int() == Some(n)).map(|(_, v)| v.str())
}

/// `_A_FRACTION[part]`: a quarter, a half, three quarters.
fn a_fraction(part: f64) -> Option<&'static str> {
    g("_A_FRACTION").pairs().iter().find(|(k, _)| k.as_f64() == Some(part)).map(|(_, v)| v.str())
}

fn num(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}

/// The area in square metres the line asks about, or None when it names no size.
pub fn _area(text: &str) -> Option<f64> {
    if let Some(pair) = _PAIR.search(text) {
        if _UNIT.is_match(pair.as_str()) {
            let (mut one, mut two) = (num(pair.str(1)), num(pair.str(2)));
            if _FEET.is_match(pair.as_str()) {
                one *= 0.3048;
                two *= 0.3048;
            }
            let area = one * two;
            if (0.2..=20000.0).contains(&area) {
                return Some(py_round(area, 1));
            }
        }
    }
    if let Some(square) = _SQUARE.search(text) {
        let mut area = num(square.str(1));
        if _FEET.is_match(square.as_str()) {
            area *= 0.0929;
        }
        if (0.2..=20000.0).contains(&area) {
            return Some(py_round(area, 1));
        }
    }
    None
}

/// The depth in the line in millimetres, or the material's usual depth.
pub fn _depth_mm(text: &str, default: f64) -> f64 {
    let Some(found) = _DEPTH.search(text) else { return default };
    let (amount, unit) = (num(found.str(1)), found.str(2));
    let depth = if unit.starts_with("mm") {
        amount
    } else if unit.starts_with("cm") {
        amount * 10.0
    } else {
        amount * 25.0
    };
    if (10.0..=400.0).contains(&depth) {
        return depth;
    }
    default
}

/// A number of square metres as a person writes it: 12, not 12.0.
pub fn _sizes(value: f64) -> String {
    if (value - text::round_half_even(value) as f64).abs() < 0.05 {
        return format!("{}", text::round_half_even(value));
    }
    format!("{:.1}", value).trim_end_matches('0').trim_end_matches('.').to_string()
}

/// How many bags to buy, said the way a person would: "about half a bulk bag".
pub fn _a_bag_count(bags: f64, word: &str) -> String {
    let quarters = text::round_half_even(bags * 4.0) as f64 / 4.0;
    let mut whole = quarters.trunc() as i64;
    let part = py_round(quarters - whole as f64, 2);
    if quarters < 0.25 {
        return format!("about a tenth of a {word}");
    }
    if quarters < 1.0 {
        if part == 0.25 {
            return format!("about a quarter of a {word}");
        }
        if part == 0.75 {
            return format!("about three quarters of a {word}");
        }
        return format!("about half a {word}");
    }
    if quarters >= 9.5 {
        let n = if bags < 20.0 { text::round_half_even(bags) as f64 } else { py_round(bags, -1) };
        return format!("about {} {word}s", pct_d(n));
    }
    if part == 0.75 {
        // nobody orders three quarters of a bag: the next whole one is the same errand
        whole += 1;
        if whole == 1 {
            return format!("about one {word}");
        }
        return format!("about {} {word}s", a_number(whole).expect("_A_NUMBER"));
    }
    if part != 0.0 {
        return format!(
            "about {} and {} {word}s",
            a_number(whole).expect("_A_NUMBER"),
            a_fraction(part).expect("_A_FRACTION")
        );
    }
    if whole == 1 {
        return format!("about one {word}");
    }
    format!("about {} {word}s", a_number(whole).expect("_A_NUMBER"))
}

/// How a person would say a number of things: "about two and a half", not "2.5".
pub fn _count(value: f64) -> String {
    if value >= 20.0 {
        return format!("about {}", pct_d(py_round(value, -1)));
    }
    if value >= 4.0 {
        return format!("about {}", text::round_half_even(value));
    }
    let halves = text::round_half_even(value * 2.0) as f64 / 2.0;
    if halves == halves.trunc() {
        if let Some(word) = a_number(halves as i64) {
            return format!("about {word}");
        }
    }
    format!("about {}", format!("{:.1}", halves).trim_end_matches('0').trim_end_matches('.'))
}

/// The bags one loose material needs at the depth the line asks for.
pub fn _loose_reply(name: &str, understood: &Understood) -> String {
    let text = format!(" {} ", understood.clean);
    let row = _LOOSE.iter().find(|r| r.names.iter().any(|n| n == name)).unwrap_or(&_LOOSE[0]);
    let mut lead = row.lead.clone();
    let mut thing = row.names[0].clone();
    let mut by_len: Vec<&String> = row.names.iter().collect();
    by_len.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    if let Some(w) = by_len.into_iter().find(|w| text.contains(w.as_str())) {
        thing = w.clone();
    }
    if thing == "sand" && (text.contains("sandpit") || text.contains("sand pit") || text.contains("play sand")) {
        // a sandpit is filled with play sand, a different sum
        thing = "play sand".to_string();
        lead = "A 25kg bag of play sand covers about 0.6 square metres at 50mm, so a 1.2m square \
                sandpit filled to 150mm takes about 10 bags"
            .to_string();
    }
    let lead = lead.replace("{thing}", &thing);
    let asked = _depth_mm(&text, row.depth);
    let area = _area(&text);
    // the coverage is per bag at the usual depth, so it falls as the layer gets deeper
    let per_bag = row.covers * (row.depth / asked);
    if let Some(area) = area.filter(|a| *a != 0.0) {
        let bags = area / per_bag;
        let word = _BAG_WORD.iter().find(|(k, _)| *k == thing).map(|(_, v)| v.as_str()).unwrap_or("bulk bag");
        return format!(
            "{} square metres at {}mm needs {} of {}. {}, and it is worth a tenth more than the sums say. {}",
            _sizes(area),
            pct_d(asked),
            _a_bag_count(bags, word),
            thing,
            lead,
            row.extra
        );
    }
    format!(
        "{}. Measure the area and work out the volume at the depth you want, then divide by what a bag \
         holds, and order a tenth more than the sums say - a bag never spreads as far as the label claims. {}",
        lead, row.extra
    )
}

/// The slabs a patio of the line's size needs, at the slab size it names.
pub fn _slabs_reply(_memory: &Memory, understood: &Understood) -> String {
    static SIZE: LazyLock<Re> = LazyLock::new(|| {
        Re::new(&format!(r"{}\s*(?:mm)?\s*(?:x|by)\s*{}\s*(?:mm|cm)?", *_NUM, *_NUM), 0)
    });
    let text = format!(" {} ", understood.clean);
    let area = _area(&text);
    let mut per_square = 4.0f64;
    if let Some(size) = SIZE.search(&text) {
        let (mut one, mut two) = (num(size.str(1)), num(size.str(2)));
        if one < 20.0 {
            one *= 1000.0;
        }
        if two < 20.0 {
            two *= 1000.0;
        }
        if (200.0..=1200.0).contains(&one) && (200.0..=1200.0).contains(&two) {
            per_square = 1e6 / (one * two);
        }
    }
    let side = text::round_half_even(1000.0 / per_square.powf(0.5));
    if let Some(area) = area.filter(|a| *a != 0.0) {
        return format!(
            "{} square metres needs {} slabs of {}mm, working at {:.1} to the square metre, with a tenth \
             spare for cuts and breakages. Under them: {}.",
            _sizes(area),
            _count(area * per_square * 1.1),
            side,
            per_square,
            _SLAB[1]
        );
    }
    format!(
        "Work it out at four 500mm slabs to the square metre, six of 400mm and five and a half of 600 x \
         300, then add a tenth for cuts and breakages. Under them: {}.",
        _SLAB[1]
    )
}

/// The rolls of turf for the line's area.
pub fn _turf_reply(_memory: &Memory, understood: &Understood) -> String {
    let text = format!(" {} ", understood.clean);
    if let Some(area) = _area(&text).filter(|a| *a != 0.0) {
        return format!(
            "{} square metres is {} rolls of turf, ordered a few extra for the edges and any that go thin. \
             Lay it within a day of delivery, water it the same evening, and keep off it for a fortnight.",
            _sizes(area),
            _count(area * 1.05)
        );
    }
    "Turf rolls cover about a square metre each, so the area in square metres is the number of rolls, \
     plus a few for the edges and any that go thin. Lay it within a day of delivery, water it the same \
     evening, and keep off it for a fortnight."
        .to_string()
}

/// What one bulk bag weighs, by what is in it.
pub fn _bulk_bag_weight(_memory: &Memory, understood: &Understood) -> String {
    let text = format!(" {} ", sh::clean_or_text(understood));
    let (mut what, mut figures) = ("gravel or sand".to_string(), "about 850kg, which is about half a cubic metre".to_string());
    for (names, line) in _BULK_WEIGHT.iter() {
        let mut by_len: Vec<&String> = names.iter().collect();
        by_len.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
        let hit = by_len
            .into_iter()
            .find(|n| text.contains(&format!(" {n} ")) || text.contains(&format!(" {n}s ")));
        if let Some(hit) = hit {
            what = hit.clone();
            figures = line.clone();
            break;
        }
    }
    format!(
        "A bulk bag of {what} is {figures} - the bags sold as a tonne bag are filled to about 850kg for stone \
         and sand, because that is what the bag and a lorry will take, so buy by the bag rather than by the \
         tonne if the weight matters. That is the same figure the coverage sums use: 850kg of gravel covers \
         about 10 square metres at 50mm. A 25kg bag is the same material in a bag you can lift, so 34 of \
         them make a bulk bag."
    )
}

/// False when the stone in the line is a weight rather than a bag of them.
pub fn _not_a_persons_weight(understood: &Understood) -> bool {
    let text = format!(" {} ", sh::clean_or_text(understood));
    if _A_MATERIAL_WEIGHT.is_match(&text) {
        return true;
    }
    !_A_PERSONS_WEIGHT.is_match(&text)
}

/// Whether the line is about buying sand rather than sanding something down.
pub fn _not_a_sanding_job(understood: &Understood) -> bool {
    !sh::a_sanding_job(understood)
}

/// `lambda memory, understood: _loose_reply("gravel", understood)` (how-much-gravel).
pub fn lambda_372(_memory: &Memory, understood: &Understood) -> String {
    _loose_reply("gravel", understood)
}

/// `lambda memory, understood: _loose_reply("topsoil", understood)` (how-much-topsoil).
pub fn lambda_378(_memory: &Memory, understood: &Understood) -> String {
    _loose_reply("topsoil", understood)
}

/// `lambda memory, understood: _loose_reply("compost", understood)` (how-much-compost).
pub fn lambda_381(_memory: &Memory, understood: &Understood) -> String {
    _loose_reply("compost", understood)
}

/// `lambda memory, understood: _loose_reply("sand", understood)` (how-much-sand).
pub fn lambda_387(_memory: &Memory, understood: &Understood) -> String {
    _loose_reply("sand", understood)
}

/// The area this line is about: from the line, else from the last few turns' lines.
pub fn _asked_area(memory: &Memory, understood: &Understood) -> Option<f64> {
    let area = _area(&format!(" {} ", sh::clean_or_text(understood)));
    if area.is_some_and(|a| a != 0.0) {
        return area;
    }
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(*_LAST_TURNS);
    for turn in turns[start..].iter().rev() {
        // Python reads `turn.get("reply")` first; a turn is {"user", "bot", "module"}, so that is
        // always None, i.e. the text " None ", which holds no size.
        for said in ["None", turn.user.as_str()] {
            let area = _area(&format!(" {said} "));
            if area.is_some_and(|a| a != 0.0) {
                return area;
            }
        }
    }
    None
}

/// How many rolls or bags of `covers` square metres each an area of `area` needs.
pub fn _rolls(area: f64, covers: f64, spare: f64) -> i64 {
    std::cmp::max(1, (-py_floordiv(-(area * spare), covers)) as i64)
}

/// The rolls of underlay for the floor's area.
pub fn _underlay_reply(memory: &Memory, understood: &Understood) -> String {
    let area = _asked_area(memory, understood);
    let how = "Roll it across the room rather than along it, butt the seams without overlapping and tape the \
               joins. Foam crumb is the cheap default; rubber crumb or felt is worth it under a thin carpet and \
               over a hard floor.";
    if let Some(area) = area {
        let rolls = _rolls(area, 15.0, 1.1);
        return format!(
            "{} square metres needs {} roll{} of underlay, a roll being 15 square metres (1.37m by 11m), and \
             the tenth extra covers the cuts round the edges and under the doorway. {}",
            _sizes(area),
            _count(rolls as f64),
            if rolls == 1 { "" } else { "s" },
            how
        );
    }
    format!(
        "Underlay is sold by area: a roll is usually 15 square metres (1.37m by 11m), so measure the floor \
         and add a tenth for the cuts. {how}"
    )
}

/// The bags of grout for the tiled area.
pub fn _grout_reply(memory: &Memory, understood: &Understood) -> String {
    let area = _asked_area(memory, understood);
    let how = "Mix it to the thickness of toothpaste and only as much as you can use in half an hour, because \
               it goes off in the bucket.";
    if let Some(area) = area {
        let bags = _rolls(area, 4.5, 1.1);
        return format!(
            "{} square metres needs {} bag{} of grout, 5kg each: a bag does 4 to 5 square metres of 30cm tiles \
             with 3mm joints, and small tiles or wide joints take about twice as much. {}",
            _sizes(area),
            _count(bags as f64),
            if bags == 1 { "" } else { "s" },
            how
        );
    }
    format!(
        "A 5kg bag of grout does about 4 to 5 square metres of 30cm tiles with 3mm joints, so a bathroom \
         floor is one bag and a kitchen wall two; mosaic or wide joints take about twice as much for the \
         same area. {how}"
    )
}

/// The bags of tile adhesive for the tiled area.
pub fn _adhesive_reply(memory: &Memory, understood: &Understood) -> String {
    let area = _asked_area(memory, understood);
    let how = "Comb it out in one direction, press each tile on across the ridges, and lift one now and then \
               to check the back is covered.";
    if let Some(area) = area {
        let bags = _rolls(area, 4.5, 1.1);
        return format!(
            "{} square metres needs {} bag{} of tile adhesive, 20kg each: a bag covers 4 to 5 square metres \
             with a 6mm notched trowel, and large tiles or a deeper notch take it nearer 3. {}",
            _sizes(area),
            _count(bags as f64),
            if bags == 1 { "" } else { "s" },
            how
        );
    }
    format!(
        "A 20kg bag of tile adhesive covers about 4 to 5 square metres with a 6mm notched trowel, so a small \
         bathroom wall is one bag; large tiles, a 10mm notch or back-buttering take it nearer 3 square \
         metres a bag. {how}"
    )
}

/// Carpet, laminate, vinyl and skirting: something bought by the area or by the length.
pub fn _floor_reply(_memory: &Memory, understood: &Understood) -> String {
    let text = format!(" {} ", understood.clean);
    let pair = _PAIR.search(&text);
    if sh::any_in(&text, &_LENGTH_NAMES) {
        // a skirting, a coving and a dado rail are a length: the room's perimeter, less the
        // doorways, in whole boards
        if let Some(pair) = pair {
            let (mut one, mut two) = (num(pair.str(1)), num(pair.str(2)));
            if _FEET.is_match(pair.as_str()) {
                one *= 0.3048;
                two *= 0.3048;
            }
            let around = 2.0 * (one + two);
            let boards = (around / 2.4).trunc() as i64 + 1;
            return format!(
                "A room {:.1}m by {:.1}m has {:.1} metres round it, so {:.1} metres once a couple of doorways \
                 are taken off - and boards come in 2.4m lengths, so buy {} of them. Cut the joints where two \
                 boards meet over a straight bit of wall, because the joint shows less there than at a corner.",
                one,
                two,
                around,
                around - 1.6,
                boards
            );
        }
        return "Skirting, coving and dado rail are bought by the length, not the area: run a tape right round \
                the room and take off the doorways, about 0.8m each. A doorway takes the skirting away and the \
                coving carries on over it. Boards come in 2.4m lengths, so buy whole boards and join them over \
                a straight run, and add one board for the corners and the cuts."
            .to_string();
    }
    if let Some(area) = _area(&text).filter(|a| *a != 0.0) {
        let waste = py_round(area * 1.1, 1);
        let packs = area * 1.1 / 2.2;
        let count = _count(packs);
        return format!(
            "{} square metres needs {} square metres of carpet or flooring with the tenth spare for the cuts \
             and the doorway, which is {} packs of laminate at about 2.2 square metres a pack. A carpet roll is \
             4m or 5m wide, which is why the shop asks for the longest and widest measurement of each room \
             rather than the area alone: a room under 4m wide comes out of one piece, across the width of the \
             roll, and a wider one needs a join. Keep the offcuts, because a spare pack for a repair later is \
             worth more than the few pounds it costs.",
            _sizes(area),
            _sizes(waste),
            count
        );
    }
    "Measure each room on the floor, not on the plan: length times width in metres, and an L-shaped room as \
     two rectangles added together. Add a tenth for the cuts and the doorway, so 12 square metres of floor \
     takes 13.2 square metres of carpet, or six packs of laminate at 2.2 square metres a pack. A carpet roll \
     is 4m or 5m wide, so the shop wants the longest and the widest measurement of each room as well as the \
     area, and a room wider than the roll needs a join."
        .to_string()
}

/// False when the line is loose ballast or hardcore with no job named, which is the volume row.
pub fn _concrete_is_what_is_asked(understood: &Understood) -> bool {
    let said = format!(" {} {} ", understood.clean.to_lowercase(), understood.text.to_lowercase());
    if sh::any_in(&said, &_CONCRETE_SURE) {
        return true;
    }
    _CONCRETE_JOB.is_match(&said)
}

/// The concrete for a post, or for a base of the line's size.
pub fn _concrete_reply(memory: &Memory, understood: &Understood) -> String {
    static POST: LazyLock<Re> =
        LazyLock::new(|| Re::new(r"\b(?:fence|gate|post|posts|hole|holes|postcrete|post mix|postmix)\b", 0));
    let text = format!(" {} ", understood.clean);
    let mixing = "Concrete is one part cement to six of ballast by volume - six shovels of ballast to one of \
                  cement, mixed dry first, then water until it holds together - and a cubic metre takes about \
                  1.75 tonnes of ballast and six or seven 25kg bags of cement, which is two bulk bags of ballast.";
    if POST.is_match(&text) {
        return format!(
            "One 20kg bag of Postcrete sets one post: the hole wants to be about 250 to 300mm across and 600mm \
             (two foot) deep, with 50 to 75mm of gravel in the bottom for drainage. Stand the post, check it \
             upright with a level on two sides, pour the dry mix in and add the water on top of it. A thicker \
             post, a gate post or a hole much wider than that takes two bags. {mixing}"
        );
    }
    if let Some(area) = _asked_area(memory, understood) {
        let volume = area * (_depth_mm(&text, 100.0) / 1000.0);
        let ballast = volume * 1.75;
        let cement = volume * 6.5;
        return format!(
            "{} square metres at {}mm is {:.2} cubic metres of concrete: about {:.1} tonnes of ballast and {} \
             bags of cement, or {} 25kg bags of ready-mixed concrete. {}",
            _sizes(area),
            pct_d(_depth_mm(&text, 100.0)),
            volume,
            ballast,
            (-py_floordiv(-cement, 1.0)) as i64,
            (-py_floordiv(-(volume * 2400.0 / 25.0), 1.0)) as i64,
            mixing
        );
    }
    format!(
        "Work it out as a volume: length times width times depth in metres, so a base 100mm thick takes 0.1 \
         cubic metres for every square metre, and a 6 by 8 foot shed is about 4.5 square metres - 0.45 cubic \
         metres, which is one 850kg bulk bag of ballast and three 25kg bags of cement. A single fence post is \
         one 20kg bag of postcrete. {mixing}"
    )
}

// --- registry wrappers ------------------------------------------------------------------------

fn g_not_a_persons_weight(_f: &FnRef, u: &Understood) -> bool {
    _not_a_persons_weight(u)
}
fn g_not_a_sanding_job(_f: &FnRef, u: &Understood) -> bool {
    _not_a_sanding_job(u)
}
fn g_concrete_is_what_is_asked(_f: &FnRef, u: &Understood) -> bool {
    _concrete_is_what_is_asked(u)
}

fn r_372(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(lambda_372(m, u))
}
fn r_378(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(lambda_378(m, u))
}
fn r_381(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(lambda_381(m, u))
}
fn r_387(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(lambda_387(m, u))
}
fn r_adhesive(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_adhesive_reply(m, u))
}
fn r_bulk_bag_weight(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_bulk_bag_weight(m, u))
}
fn r_concrete(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_concrete_reply(m, u))
}
fn r_floor(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_floor_reply(m, u))
}
fn r_grout(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_grout_reply(m, u))
}
fn r_slabs(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_slabs_reply(m, u))
}
fn r_turf(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_turf_reply(m, u))
}
fn r_underlay(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_underlay_reply(m, u))
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.materials:_concrete_is_what_is_asked:650", g_concrete_is_what_is_asked),
    ("bot.knowledge.materials:_not_a_persons_weight:350", g_not_a_persons_weight),
    ("bot.knowledge.materials:_not_a_sanding_job:363", g_not_a_sanding_job),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.materials:<lambda>:372", r_372),
    ("bot.knowledge.materials:<lambda>:378", r_378),
    ("bot.knowledge.materials:<lambda>:381", r_381),
    ("bot.knowledge.materials:<lambda>:387", r_387),
    ("bot.knowledge.materials:_adhesive_reply:510", r_adhesive),
    ("bot.knowledge.materials:_bulk_bag_weight:332", r_bulk_bag_weight),
    ("bot.knowledge.materials:_concrete_reply:665", r_concrete),
    ("bot.knowledge.materials:_floor_reply:545", r_floor),
    ("bot.knowledge.materials:_grout_reply:496", r_grout),
    ("bot.knowledge.materials:_slabs_reply:215", r_slabs),
    ("bot.knowledge.materials:_turf_reply:239", r_turf),
    ("bot.knowledge.materials:_underlay_reply:482", r_underlay),
];
