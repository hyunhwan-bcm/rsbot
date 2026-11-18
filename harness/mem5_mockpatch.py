"""Make a copy of the crate in which the functions the MEM5 files call and other workstreams have
not ported yet answer from a mem5_record.py recording (its "dep" rows), so the MEM5 ports can be
checked alone.

    rsync -a --delete src Cargo.toml Cargo.lock COPY/
    python3 harness/mem5_mockpatch.py COPY
    cd COPY && CARGO_TARGET_DIR=COPY_TARGET cargo build --profile fast --bin mem5_check
    RSBOT_DATA=../data/export/bundle.json RSBOT_TODAY=2026-11-17 MEM5_MOCK=REC.jsonl.gz \\
        COPY_TARGET/fast/mem5_check REC.jsonl.gz

Test-only: the copy is never merged. A dependency's call is looked up by its arguments with any
memory (a dict, or a list of turns) left out, as the recorder keys it. `last_bot` is written out
(it only reads the last turn)."""
import sys
root = sys.argv[1]

MOCK = r'''//! Test-only: answers for functions other workstreams have not ported, read from the "dep"
//! rows of a mem5_record.py recording (MEM5_MOCK=path,...). Never part of the real crate.
use crate::data::Value;
use serde_json::Value as J;
use std::collections::HashMap;
use std::io::BufRead;
use std::sync::LazyLock;

static TABLE: LazyLock<HashMap<String, J>> = LazyLock::new(|| {
    let mut out = HashMap::new();
    let Ok(paths) = std::env::var("MEM5_MOCK") else { return out };
    for p in paths.split(',') {
        let file = std::fs::File::open(p).unwrap();
        for line in std::io::BufReader::new(flate2::read::GzDecoder::new(file)).lines() {
            let Ok(line) = line else { break };
            let Ok(row) = serde_json::from_str::<J>(&line) else { break };
            let Some(name) = row["dep"].as_str() else { continue };
            out.insert(format!("{}|{}", name, serde_json::to_string(&row["key"]).unwrap()), row["result"].clone());
        }
    }
    out
});

/// The first of several argument shapes the recording has.
pub fn get(name: &str, shapes: Vec<J>) -> J {
    for args in &shapes {
        let key = format!("{}|{}", name, serde_json::to_string(args).unwrap());
        if let Some(j) = TABLE.get(&key) {
            return j.clone();
        }
    }
    panic!("UNPORTED mock miss {name} {}", serde_json::to_string(&shapes).unwrap())
}
'''

def patch(path, old, new):
    p = f"{root}/{path}"
    s = open(p).read()
    assert old in s, (path, old)
    s = s.replace(old, new, 1)
    open(p, "w").write(s)

j = "serde_json::json!"
open(f"{root}/src/mem5_mock.rs", "w").write(MOCK)
patch("src/lib.rs", "pub mod memory;", "pub mod memory;\npub mod mem5_mock;")
patch("src/compose/mod.rs", 'crate::unported!("bot.compose:pick")',
      f'let _ = memory; return crate::mem5_mock::get("bot.compose:pick", vec![{j}([options, null, seed_text]), {j}([options, null])]).as_str().unwrap_or("").to_string()')
patch("src/memory/detail.rs", 'crate::compose::trouble::reads_as_trouble(clean, None, "").py_true()',
      f'crate::mem5_mock::get("bot.compose.trouble:reads_as_trouble", vec![{j}([clean])]).as_bool().unwrap_or(false)')
patch("src/memory/not_a_trip.rs", 'crate::unported!("bot.memory.not_a_trip:not_a_destination")',
      f'crate::mem5_mock::get("bot.memory.not_a_trip:not_a_destination", vec![{j}([going_words, place])]).as_bool().unwrap_or(false)')
patch("src/memory/not_a_trip.rs", 'crate::unported!("bot.memory.not_a_trip:the_visit")',
      f'crate::mem5_mock::get("bot.memory.not_a_trip:the_visit", vec![{j}([line])]).as_str().unwrap_or("").to_string()')
patch("src/memory/mod.rs", 'crate::unported!("bot.memory:last_bot")',
      'memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default()')
patch("src/memory/mod.rs", 'crate::unported!("bot.memory:plan_line")',
      f'let _ = memory; crate::mem5_mock::get("bot.memory:plan_line", vec![{j}([null, plan])]).as_str().unwrap_or("").to_string()')
patch("src/memory/condition.rs", 'crate::unported!("bot.memory.condition:conditions")',
      f'Value::from_plain(&crate::mem5_mock::get("bot.memory.condition:conditions", vec![{j}([said])]))')
patch("src/memory/history.rs", 'crate::unported!("bot.memory.history:is_their_news")',
      f'let _ = words; crate::mem5_mock::get("bot.memory.history:is_their_news", vec![{j}([line])]).as_bool().unwrap_or(false)')
patch("src/memory/spoken.rs", 'crate::unported!("bot.memory.spoken:_spoken_as_numbers")',
      f'crate::mem5_mock::get("bot.memory.spoken:_spoken_as_numbers", vec![{j}([text])]).as_str().unwrap_or("").to_string()')
patch("src/memory/enough.rs", 'crate::memory::purse::against_the_bands(text, &turns_value(turns)).reply()',
      f'{{ let _ = turns_value(turns); crate::mem5_mock::get("bot.memory.purse:against_the_bands", vec![{j}([text, null])]).as_str().map(str::to_string).filter(|s| !s.is_empty()) }}')
patch("src/memory/places.rs", 'crate::unported!("bot.memory.places:_a_place")',
      f'crate::mem5_mock::get("bot.memory.places:_a_place", vec![{j}([value.to_plain()])]).as_str().map(str::to_string)')
print("patched")
