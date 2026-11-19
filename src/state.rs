//! The conversation memory a turn reads and writes: Python's `memory` dict. (The Python
//! package `bot/memory` is `crate::memory`; this struct is `crate::state::Memory`.)
//!
//! This port answers one line at a time (as SmolTalkBench asks), so `turns` is empty when a
//! turn starts and the keys that only matter on a later turn are not kept. What a module writes
//! for its own use within the turn (an entry it answered from, a case it recorded) goes in
//! `extra`, under the Python key.

use crate::data::Value;
use crate::understood::Understood;
use indexmap::IndexMap;

#[derive(Clone, Debug, Default)]
pub struct Turn {
    pub user: String,
    pub bot: String,
    pub module: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Memory {
    /// The understanding engine's reading of this line (`memory["understood"]`).
    pub understood: Option<Understood>,
    /// The shape of answer the line asks for (`memory["intent_wants"]`).
    pub intent_wants: Option<String>,
    /// The line as typed, kept by the answers module (`memory["asked_now"]`).
    pub asked_now: Option<String>,
    /// Earlier turns (`memory["turns"]`); empty at the start of a single-turn reply.
    pub turns: Vec<Turn>,
    /// Anything else a module keeps for itself, by its Python key.
    pub extra: IndexMap<String, Value>,
    /// `intent_wants` was set (to None) - Python keeps the key; the plain form shows it.
    pub extra_had_intent: bool,
    /// Set on a `shallow()` copy only: the `extra` keys bound afresh on the copy since it was made
    /// (`memory[key] = ...`, `memory.pop(key)`, a `setdefault` that had to create the key).
    pub rebound: Option<Vec<String>>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory::default()
    }
    /// `len(memory.get("turns") or [])`, the count most seeded pickers use.
    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }
}

impl Memory {
    /// `memory.get(key)` for anything kept in `extra` (the typed fields have their own names).
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }
    /// `memory.get(key)` as a mutable value.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.extra.get_mut(key)
    }
    /// `memory[key] = value` for an `extra` key.
    pub fn set(&mut self, key: &str, value: impl Into<Value>) {
        self.note_rebound(key);
        self.extra.insert(key.to_string(), value.into());
    }
    /// `memory.pop(key, None)`.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.note_rebound(key);
        self.extra.shift_remove(key)
    }
    /// `memory.setdefault(key, default)`.
    pub fn setdefault(&mut self, key: &str, default: impl Into<Value>) -> &mut Value {
        if !self.extra.contains_key(key) {
            self.note_rebound(key);
        }
        self.extra.entry(key.to_string()).or_insert_with(|| default.into())
    }
    fn note_rebound(&mut self, key: &str) {
        if let Some(rebound) = &mut self.rebound {
            if !rebound.iter().any(|k| k == key) {
                rebound.push(key.to_string());
            }
        }
    }

    /// Python's `dict(memory)`: a shallow copy. Its values are the same objects as the original's,
    /// so a write *inside* a value the original already held (`sub["compose"]["taste"] = True`,
    /// `sub["facts_log"].append(...)`) shows in the original, while a key bound on the copy
    /// (`sub[key] = value`) does not. Values here are copy-on-write, so the copy remembers which
    /// keys were bound afresh and `absorb_shallow` hands the rest back when the copy is done with.
    /// (The typed fields - `understood`, `turns`, `intent_wants`, `asked_now` - are not handed
    /// back: no caller of a shallow copy writes inside them.)
    pub fn shallow(&self) -> Memory {
        let mut copy = self.clone();
        copy.rebound = Some(Vec::new());
        copy
    }

    /// The other half of `shallow()`: every value the copy still shares with this memory (a key
    /// both hold that the copy never bound afresh) is taken as the copy left it.
    pub fn absorb_shallow(&mut self, copy: &Memory) {
        let Some(rebound) = &copy.rebound else { return };
        for (key, value) in &copy.extra {
            if rebound.iter().any(|k| k == key) {
                continue;
            }
            if let Some(mine) = self.extra.get_mut(key) {
                *mine = value.clone();
            }
        }
    }

    /// `key in memory`.
    pub fn has(&self, key: &str) -> bool {
        match key {
            "understood" => self.understood.is_some(),
            "intent_wants" => self.intent_wants.is_some(),
            "asked_now" => self.asked_now.is_some(),
            "turns" => true,
            _ => self.extra.contains_key(key),
        }
    }

    /// The memory as plain JSON, in the shape the per-turn oracle records (`harness/turn_oracle.py`).
    pub fn to_plain(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut m = serde_json::Map::new();
        if let Some(u) = &self.understood {
            m.insert("understood".into(), serde_json::to_value(u).unwrap());
        }
        if self.intent_wants.is_some() || self.extra_had_intent {
            m.insert("intent_wants".into(), serde_json::json!(self.intent_wants));
        }
        if let Some(a) = &self.asked_now {
            m.insert("asked_now".into(), serde_json::json!(a));
        }
        for (k, v) in &self.extra {
            m.insert(k.clone(), v.to_plain());
        }
        if !self.turns.is_empty() {
            m.insert(
                "turns".into(),
                serde_json::Value::Array(
                    self.turns.iter().map(|t| serde_json::json!({"user": t.user, "bot": t.bot, "module": t.module})).collect(),
                ),
            );
        }
        m
    }

    /// A memory from the oracle's plain JSON.
    pub fn from_plain(m: &serde_json::Map<String, serde_json::Value>) -> Memory {
        let mut out = Memory::new();
        for (k, v) in m {
            match k.as_str() {
                "understood" => out.understood = serde_json::from_value(v.clone()).ok(),
                "intent_wants" => {
                    out.intent_wants = v.as_str().map(str::to_string);
                    out.extra_had_intent = true;
                }
                "asked_now" if v.is_string() => out.asked_now = v.as_str().map(str::to_string),
                "turns" => {
                    out.turns = v
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .map(|t| Turn {
                                    user: t["user"].as_str().unwrap_or("").to_string(),
                                    bot: t["bot"].as_str().unwrap_or("").to_string(),
                                    module: t["module"].as_str().map(str::to_string),
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                }
                _ => {
                    out.extra.insert(k.clone(), Value::from_plain(v));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `sub = dict(memory)`: a write inside a shared value shows in the original, a key bound on
    /// the copy does not.
    #[test]
    fn a_shallow_copy_shares_its_values() {
        let mut memory = Memory::new();
        memory.setdefault("compose", Value::dict()).set_key("taste", false);
        memory.set("topic", "tea");
        let mut copy = memory.shallow();
        copy.setdefault("compose", Value::dict()).set_key("taste", true); // sub["compose"]["taste"] = True
        copy.set("topic", "coffee"); // sub["topic"] = "coffee"
        copy.setdefault("fresh", Value::dict()).set_key("made", true); // a key the original never had
        memory.absorb_shallow(&copy);
        assert!(memory.get("compose").unwrap().get("taste").unwrap().truthy());
        assert_eq!(memory.get("topic").unwrap().as_str(), Some("tea"));
        assert!(!memory.has("fresh"));
    }
}
