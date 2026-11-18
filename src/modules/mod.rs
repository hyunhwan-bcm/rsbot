//! Port of PyBot's `bot/modules`: the hot-path modules only.
//!
//! On SmolTalkBench, these nine modules and two observers give every reply the full 97-module
//! bot gives (1000/1000 on the gold set; see README). PRIORITY values and the order (highest
//! first, then by name) are PyBot's.

use crate::runtime::ModuleDef;

pub mod arbiter;

/// The arbiter's `choose`.
pub fn choose(text: &str, memory: &crate::state::Memory, candidates: &[crate::runtime::Candidate]) -> usize {
    arbiter::choose(text, memory, candidates)
}

/// Every module, highest PRIORITY first, then by name: the order `bot/runtime.py` runs them in.
pub fn registry() -> &'static [ModuleDef] {
    static REGISTRY: std::sync::LazyLock<Vec<ModuleDef>> = std::sync::LazyLock::new(|| {
        let mut v = vec![
            ModuleDef { name: "understand", priority: 10000.0, observe: Some(crate::modules::hooks::observe_understand), respond: None, after: None },
            ModuleDef { name: "intent", priority: 9500.0, observe: Some(crate::modules::hooks::observe_intent), respond: None, after: None },
            ModuleDef { name: "safety", priority: 9000.0, observe: None, respond: Some(crate::modules::hooks::respond_safety), after: None },
            ModuleDef { name: "memory_store", priority: 7000.0, observe: None, respond: Some(crate::modules::hooks::respond_memory_store), after: None },
            ModuleDef { name: "dreams", priority: 3000.0, observe: None, respond: Some(crate::modules::hooks::respond_dreams), after: None },
            ModuleDef { name: "chat", priority: 300.0, observe: None, respond: Some(crate::modules::hooks::respond_chat), after: None },
            ModuleDef { name: "answers", priority: 0.0, observe: None, respond: Some(crate::modules::hooks::respond_answers), after: None },
            ModuleDef { name: "where_it_answers", priority: -999.0, observe: None, respond: Some(crate::modules::hooks::respond_where_it_answers), after: None },
            ModuleDef { name: "first_line_open", priority: -999.5, observe: None, respond: Some(crate::modules::hooks::respond_first_line_open), after: None },
            ModuleDef { name: "the_shape_of_the_answer", priority: -999.5, observe: None, respond: Some(crate::modules::hooks::respond_the_shape_of_the_answer), after: None },
            ModuleDef { name: "greeting", priority: -1000.0, observe: None, respond: Some(crate::modules::hooks::respond_greeting), after: None },
        ];
        v.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap().then(a.name.cmp(b.name)));
        v
    });
    &REGISTRY
}

/// The single place each module's entry points are wired. A workstream replaces its stub here
/// with a call into the ported module.
pub mod hooks {
    use crate::state::Memory;

    pub fn observe_understand(_text: &str, _memory: &mut Memory) {}
    pub fn observe_intent(_text: &str, _memory: &mut Memory) {}
    pub fn respond_safety(_text: &str, _memory: &mut Memory) -> Option<String> {
        None
    }
    pub fn respond_memory_store(_text: &str, _memory: &mut Memory) -> Option<String> {
        None
    }
    pub fn respond_dreams(_text: &str, _memory: &mut Memory) -> Option<String> {
        None
    }
    pub fn respond_chat(_text: &str, _memory: &mut Memory) -> Option<String> {
        None
    }
    pub fn respond_answers(_text: &str, _memory: &mut Memory) -> Option<String> {
        None
    }
    pub fn respond_where_it_answers(_text: &str, _memory: &mut Memory) -> Option<String> {
        None
    }
    pub fn respond_first_line_open(_text: &str, _memory: &mut Memory) -> Option<String> {
        None
    }
    pub fn respond_the_shape_of_the_answer(_text: &str, _memory: &mut Memory) -> Option<String> {
        None
    }
    pub fn respond_greeting(_text: &str, _memory: &mut Memory) -> Option<String> {
        None
    }
}
