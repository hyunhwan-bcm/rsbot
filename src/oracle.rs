//! PyBot's own per-stage results on the benchmark prompts (`harness/oracle_dump.py`), for
//! checking the port stage by stage and for standing in for a stage that is not ported yet.
//!
//! `data/export/oracle_<split>.plain.jsonl` holds, per prompt: the reading after both observers
//! (`understood`, `intent_wants`), every hot-path module's candidate (or null), the order they
//! were proposed in, the arbiter's pick and the final reply.

use crate::understood::Understood;
use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Clone, Debug, Deserialize)]
pub struct Row {
    pub id: String,
    pub text: String,
    pub understood: Understood,
    pub intent_wants: Option<String>,
    pub candidates: IndexMap<String, Option<String>>,
    pub order: Vec<String>,
    pub chosen: Option<String>,
    pub reply: String,
}

pub fn path(split: &str) -> String {
    format!("{}/../data/export/oracle_{split}.plain.jsonl", env!("CARGO_MANIFEST_DIR"))
}

pub fn load(split: &str) -> Vec<Row> {
    let text = std::fs::read_to_string(path(split)).unwrap_or_else(|e| panic!("{}: {e}", path(split)));
    text.lines()
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("oracle row: {e}")))
        .collect()
}

/// PyBot's `understand(text)` results for every text it was called on while answering the
/// benchmark prompts (`data/export/understand_calls.jsonl`): lets a module be checked before the
/// understanding engine is ported. Only used when `RSBOT_ORACLE_UNDERSTAND` is set.
pub static UNDERSTAND_CALLS: LazyLock<HashMap<String, Understood>> = LazyLock::new(|| {
    let p = format!("{}/../data/export/understand_calls.jsonl", env!("CARGO_MANIFEST_DIR"));
    let Ok(text) = std::fs::read_to_string(&p) else { return HashMap::new() };
    text.lines()
        .filter_map(|l| {
            #[derive(Deserialize)]
            struct C {
                text: String,
                understood: Understood,
            }
            serde_json::from_str::<C>(l).ok().map(|c| (c.text, c.understood))
        })
        .collect()
});

pub fn use_oracle_understand() -> bool {
    std::env::var_os("RSBOT_ORACLE_UNDERSTAND").is_some()
}
