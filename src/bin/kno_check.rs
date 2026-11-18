//! KNO's checks of knowledge helpers against PyBot's values (corpora from harness/kno/).
//!
//!     kno_check units FILE [--show K]
//!
//! `units`: every case of `make_units.py`: `convert` with a memory of the case's earlier turns,
//! and the other functions on the line alone, against PyBot's results.
use rsbot::knowledge::units as u;
use rsbot::state::{Memory, Turn};
use serde_json::{json, Value as J};
use std::collections::BTreeMap;
use std::io::BufRead;

fn opt(v: Option<String>) -> J {
    v.map(J::String).unwrap_or(J::Null)
}

fn units_case(name: &str, clean: &str, memory: Option<&Memory>) -> J {
    match name {
        "convert" => opt(u::convert(clean, memory)),
        "_object_weight" => opt(u::_object_weight(clean)),
        "_oven" => opt(u::_oven(clean)),
        "_temperature" => opt(u::_temperature(clean)),
        "_temperature_rule" => opt(u::_temperature_rule(clean)),
        "_butter" => opt(u::_butter(clean)),
        "_pack" => opt(u::_pack(clean)),
        "_pack_count" => opt(u::_pack_count(clean)),
        "_loose" => opt(u::_loose(clean)),
        "_liquid_weight" => opt(u::_liquid_weight(clean)),
        "_a_dish_for_a_table" => opt(u::_a_dish_for_a_table(clean)),
        "_per_person" => opt(u::_per_person(clean, None)),
        "_portions_in_a_weight" => opt(u::_portions_in_a_weight(clean)),
        "_birds_for_a_table" => opt(u::_birds_for_a_table(clean)),
        "_a_whole_thing_feeds" => opt(u::_a_whole_thing_feeds(clean, None)),
        "_share_of_a_measure" => opt(u::_share_of_a_measure(clean, true)),
        "_a_span_of_time" => opt(u::_a_span_of_time(clean)),
        "_handful" => opt(u::_handful(clean)),
        "_own_height" => opt(u::_own_height(clean)),
        "_cup_of_something" => opt(u::_cup_of_something(clean)),
        "_piece_weight" => opt(u::_piece_weight(clean)),
        "_frac" => json!(u::_frac(clean)),
        "people_in_groups" => u::people_in_groups(clean).map(|n| json!(n)).unwrap_or(J::Null),
        "_people_asked" => u::_people_asked(clean).map(|n| json!(n)).unwrap_or(J::Null),
        "answers_more_than_a_pair" => json!(u::answers_more_than_a_pair(clean)),
        "_count_in_a_line" => opt(u::_count_in_a_line(clean)),
        "_metres_in_feet" => opt(u::_metres_in_feet(clean)),
        "_the_thing_measured_by_itself" => opt(u::_the_thing_measured_by_itself(clean)),
        "_in_a_vessel" => opt(u::_in_a_vessel(clean)),
        "_a_fraction_of_a_time_unit" => opt(u::_a_fraction_of_a_time_unit(clean)),
        "_a_corrected_figure" => u::_a_corrected_figure(clean).map(|n| json!(n)).unwrap_or(J::Null),
        "_a_table_without_people" => json!(u::_a_table_without_people(clean)),
        "_asks_something_else" => json!(u::_asks_something_else(clean)),
        "_asks_for_weight" => json!(u::_asks_for_weight(clean)),
        other => panic!("no case for {other}"),
    }
}

const UNITS_FNS: &[&str] = &[
    "convert", "_object_weight", "_oven", "_temperature", "_temperature_rule", "_butter", "_pack", "_pack_count", "_loose",
    "_liquid_weight", "_a_dish_for_a_table", "_per_person", "_portions_in_a_weight", "_birds_for_a_table",
    "_a_whole_thing_feeds", "_share_of_a_measure", "_a_span_of_time", "_handful", "_own_height", "_cup_of_something",
    "_piece_weight", "_frac", "people_in_groups", "_people_asked", "answers_more_than_a_pair", "_count_in_a_line",
    "_metres_in_feet", "_the_thing_measured_by_itself", "_in_a_vessel", "_a_fraction_of_a_time_unit", "_a_corrected_figure",
    "_a_table_without_people", "_asks_something_else", "_asks_for_weight",
];

/// Python's result as the port reports it: a raised exception is a panic, None/False are absent.
fn python(r: &J, name: &str) -> J {
    match r.get(name) {
        None => match name {
            "_frac" => J::Null,
            "answers_more_than_a_pair" | "_a_table_without_people" | "_asks_something_else" | "_asks_for_weight" => json!(false),
            _ => J::Null,
        },
        Some(v) => v.clone(),
    }
}

fn main() {
    if std::env::var_os("RSBOT_TODAY").is_none() {
        unsafe { std::env::set_var("RSBOT_TODAY", "2026-11-17") };
    }
    rsbot::runtime::quiet_panics();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let show: usize = args.iter().position(|a| a == "--show").map(|i| args[i + 1].parse().unwrap()).unwrap_or(5);
    rsbot::data::load();
    match args[0].as_str() {
        "units" => {
            let mut stats: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
            let mut shown: BTreeMap<&str, usize> = BTreeMap::new();
            for line in std::io::BufReader::new(std::fs::File::open(&args[1]).unwrap()).lines() {
                let c: J = serde_json::from_str(&line.unwrap()).unwrap();
                let clean = c["c"].as_str().unwrap();
                let turns: Vec<Turn> = c["t"].as_array().unwrap().iter().map(|t| Turn { user: t.as_str().unwrap().to_string(), bot: String::new(), module: None }).collect();
                let memory = if turns.is_empty() { None } else { Some(Memory { turns, ..Memory::default() }) };
                for &name in UNITS_FNS {
                    let want = python(&c["r"], name);
                    let got = match rsbot::runtime::guarded(|| units_case(name, clean, memory.as_ref())) {
                        Some(v) => v,
                        None => J::String(format!("!{}", rsbot::runtime::take_last_panic().unwrap_or_default())),
                    };
                    // `_frac` is recorded only when it is not None, which it never is: compare when kept.
                    // make_units.py dropped results equal to False, which in Python is 0 as well
                    let same = if c["r"].get(name).is_none() && got == json!(0) { true } else if name == "_frac" && want.is_null() { true } else if let (J::String(g), J::String(w)) = (&got, &want) { g == w || (w.starts_with('!') && g.starts_with('!')) } else { got == want };
                    let st = stats.entry(name).or_default();
                    if same { st.0 += 1 } else {
                        st.1 += 1;
                        let k = shown.entry(name).or_default();
                        if *k < show {
                            *k += 1;
                            println!("-- {name} {clean:?} turns {}\n   rust {got}\n   py   {want}", c["t"]);
                        }
                    }
                }
            }
            let (mut ok, mut bad) = (0, 0);
            for (name, (o, b)) in &stats {
                ok += o;
                bad += b;
                if *b > 0 { println!("{name}: {o} ok, {b} differ"); }
            }
            println!("units: calls {ok} ok / {bad} differ");
        }
        other => panic!("unknown check {other}"),
    }
}
