//! Port of PyBot's `bot/understanding` package (`bot/understanding/__init__.py` is this file).
//! Each Python file `bot/understanding/<name>.py` is `<name>.rs` here.
//!
//! `understand(text, memory)` is the reading every module looks at first.

use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub mod attempts;
pub mod body;
pub mod breeds;
pub mod car_glass;
pub mod cars;
pub mod confusion;
pub mod kinds;
pub mod lexicon;
pub mod normalize;
pub mod pyuni;
pub mod references;
pub mod senses;
pub mod topics;

/// A regex built from a pattern made at run time (`re.search(r"\b" + re.escape(cue) ...)`),
/// compiled once per distinct pattern and kept for the life of the process (the patterns come
/// from PyBot's own tables, so there are finitely many).
pub fn rx(pattern: &str) -> &'static Re {
    rx_flags(pattern, 0)
}

/// [`rx`] with Python flag bits (`re::I`, ...).
pub fn rx_flags(pattern: &str, flags: u32) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<(String, u32), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let key = (pattern.to_string(), flags);
    if let Some(r) = CACHE.lock().unwrap().get(&key) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(pattern, flags)));
    CACHE.lock().unwrap().insert(key, r);
    r
}

/// `understand(text, memory=None)`: the reading of one line, read against the conversation so
/// far. With `RSBOT_ORACLE_UNDERSTAND` set, PyBot's own recorded reading is returned for a text
/// it has seen (for checking modules before this engine is complete).
pub fn understand(text: &str, memory: Option<&Memory>) -> Understood {
    if crate::oracle::use_oracle_understand() {
        if let Some(u) = crate::oracle::UNDERSTAND_CALLS.get(text) {
            return u.clone();
        }
    }
    let _ = memory;
    todo!("understanding::understand")
}

/// Typed readers for PyBot's module-level tables in the data bundle, by module and Python name:
/// `t::set("bot.understanding.normalize.", "LEADERS")`.
pub mod t {
    use crate::data::{self, Value};
    use crate::py::re::Re;
    use indexmap::IndexMap;
    use std::collections::HashSet;

    fn get(m: &str, name: &str) -> &'static Value {
        data::get(&format!("{m}{name}"))
    }
    pub fn set(m: &str, name: &str) -> HashSet<String> {
        get(m, name).str_set()
    }
    pub fn strs(m: &str, name: &str) -> Vec<String> {
        get(m, name).strs()
    }
    pub fn string(m: &str, name: &str) -> &'static str {
        get(m, name).str()
    }
    pub fn int(m: &str, name: &str) -> i64 {
        get(m, name).as_int().unwrap_or_else(|| panic!("{m}{name} is not an int"))
    }
    pub fn re(m: &str, name: &str) -> &'static Re {
        get(m, name).as_re().unwrap_or_else(|| panic!("{m}{name} is not a regex"))
    }
    pub fn res(m: &str, name: &str) -> Vec<&'static Re> {
        get(m, name).items().iter().map(|v| v.as_re().unwrap()).collect()
    }
    pub fn tuples(m: &str, name: &str) -> Vec<Vec<String>> {
        get(m, name).items().iter().map(Value::strs).collect()
    }
    pub fn re_str_pairs(m: &str, name: &str) -> Vec<(&'static Re, String)> {
        get(m, name).items().iter().map(|p| (p.items()[0].as_re().unwrap(), p.items()[1].str().to_string())).collect()
    }
    pub fn str_re_pairs(m: &str, name: &str) -> Vec<(String, &'static Re)> {
        get(m, name).items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().unwrap())).collect()
    }
    pub fn map(m: &str, name: &str) -> IndexMap<String, String> {
        get(m, name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
    }
    pub fn map_int(m: &str, name: &str) -> IndexMap<String, i64> {
        get(m, name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
    }
    pub fn map_strs(m: &str, name: &str) -> IndexMap<String, Vec<String>> {
        get(m, name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
    }
    pub fn map_set(m: &str, name: &str) -> IndexMap<String, HashSet<String>> {
        get(m, name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect()
    }
    pub fn map_re(m: &str, name: &str) -> IndexMap<String, &'static Re> {
        get(m, name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_re().unwrap())).collect()
    }
    pub fn value(m: &str, name: &str) -> &'static Value {
        get(m, name)
    }
}
