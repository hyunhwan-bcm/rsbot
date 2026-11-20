//! PyBot's module-level data, exported from Python by `harness/export_data.py`.
//!
//! The port hand-writes PyBot's logic and reads its data (word lists, reply pools, regexes,
//! the 8k knowledge entries) from this bundle, keyed by the Python name:
//! `data::strs("bot.knowledge.lookup.RING_AS_A_CALL")`. Each ported module keeps the values it
//! needs in `LazyLock` statics, so a name is looked up once.
//!
//! The bundle is `data/export/bundle.json` next to the crate, or the file named by
//! `RSBOT_DATA`.

use crate::py::re::Re;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// A Python value as exported: enough structure to rebuild any table the bot reads.
#[derive(Clone, Debug)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Arc<str>),
    List(Arc<Vec<Value>>),
    Tuple(Arc<Vec<Value>>),
    Set(Arc<Vec<Value>>),
    Dict(Arc<Vec<(Value, Value)>>),
    Re(Arc<Re>),
    /// A function: which one, and what a closure captured.
    Fn(Arc<FnRef>),
    Obj(Arc<str>, Arc<Vec<(String, Value)>>),
    Module(Arc<str>),
    Opaque(Arc<str>),
}

/// A Python function in the data: its id ("module:qualname:line"), the values a closure
/// captured (by variable name) and its default arguments. A knowledge entry's guard or reply
/// that is a function is one of these; `crate::knowledge::registry` runs the Rust port of it.
#[derive(Clone, Debug)]
pub struct FnRef {
    pub id: String,
    pub cells: Vec<(String, Value)>,
    pub defaults: Vec<Value>,
}

impl FnRef {
    /// "module:qualname" without the line number.
    pub fn name(&self) -> &str {
        // "module:qualname:line" -> "module:qualname"; an id built by hand without the line
        // ("module:qualname") is already the name.
        match self.id.rsplit_once(':') {
            Some((a, line)) if !line.is_empty() && line.bytes().all(|b| b.is_ascii_digit()) => a,
            _ => &self.id,
        }
    }
    /// A captured variable's value.
    pub fn cell(&self, name: &str) -> Option<&Value> {
        self.cells.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }
}

static EMPTY: Vec<Value> = Vec::new();

impl Value {
    pub fn is_none(&self) -> bool {
        matches!(self, Value::None)
    }
    /// Python truthiness.
    pub fn truthy(&self) -> bool {
        match self {
            Value::None => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(v) | Value::Tuple(v) | Value::Set(v) => !v.is_empty(),
            Value::Dict(d) => !d.is_empty(),
            _ => true,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    /// The string, or "" for anything else.
    pub fn str(&self) -> &str {
        self.as_str().unwrap_or("")
    }
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Bool(b) => Some(*b as i64),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    /// Items of a list, tuple or set (sets in sorted order); empty for anything else.
    pub fn items(&self) -> &[Value] {
        match self {
            Value::List(v) | Value::Tuple(v) | Value::Set(v) => v,
            _ => &EMPTY,
        }
    }
    /// The strings in a list, tuple or set.
    pub fn strs(&self) -> Vec<String> {
        self.items().iter().filter_map(|v| v.as_str().map(str::to_string)).collect()
    }
    pub fn str_set(&self) -> HashSet<String> {
        self.items().iter().filter_map(|v| v.as_str().map(str::to_string)).collect()
    }
    pub fn pairs(&self) -> &[(Value, Value)] {
        match self {
            Value::Dict(d) => d,
            _ => &[],
        }
    }
    /// A dict's value for a string key.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.pairs().iter().find(|(k, _)| k.as_str() == Some(key)).map(|(_, v)| v)
    }
    /// An object's attribute (an exported class instance, e.g. a knowledge Entry).
    pub fn attr(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Obj(_, attrs) => attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn class(&self) -> Option<&str> {
        match self {
            Value::Obj(c, _) => Some(c),
            _ => None,
        }
    }
    pub fn as_re(&self) -> Option<&Re> {
        match self {
            Value::Re(r) => Some(r),
            _ => None,
        }
    }
    pub fn as_fn(&self) -> Option<&FnRef> {
        match self {
            Value::Fn(f) => Some(f),
            _ => None,
        }
    }
    /// A `{str: str}` dict as a map.
    pub fn str_map(&self) -> HashMap<String, String> {
        self.pairs()
            .iter()
            .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
            .collect()
    }
    /// A `{str: [str]}` dict (or tuple values) as a map.
    pub fn str_list_map(&self) -> HashMap<String, Vec<String>> {
        self.pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.strs()))).collect()
    }

    /// A value from plain JSON (lists for any sequence, objects for str-keyed dicts), as the
    /// oracle files hold readings and memory.
    pub fn from_plain(j: &serde_json::Value) -> Value {
        use serde_json::Value as J;
        match j {
            J::Null => Value::None,
            J::Bool(b) => Value::Bool(*b),
            J::Number(n) => match n.as_i64() {
                Some(i) => Value::Int(i),
                None => Value::Float(n.as_f64().unwrap_or(0.0)),
            },
            J::String(s) => Value::Str(Arc::from(s.as_str())),
            J::Array(a) => Value::List(Arc::new(a.iter().map(Value::from_plain).collect())),
            J::Object(o) => Value::Dict(Arc::new(
                o.iter().map(|(k, v)| (Value::Str(Arc::from(k.as_str())), Value::from_plain(v))).collect(),
            )),
        }
    }

    fn from_json(j: &serde_json::Value) -> Value {
        use serde_json::Value as J;
        match j {
            J::Null => Value::None,
            J::Bool(b) => Value::Bool(*b),
            J::Number(n) => match n.as_i64() {
                Some(i) => Value::Int(i),
                None => Value::Float(n.as_f64().unwrap_or(0.0)),
            },
            J::String(s) => Value::Str(Arc::from(s.as_str())),
            J::Array(a) => Value::List(Arc::new(a.iter().map(Value::from_json).collect())),
            J::Object(o) => {
                let (tag, body) = o.iter().next().expect("tagged value");
                match tag.as_str() {
                    "l" => Value::List(Arc::new(body.as_array().unwrap().iter().map(Value::from_json).collect())),
                    "t" => Value::Tuple(Arc::new(body.as_array().unwrap().iter().map(Value::from_json).collect())),
                    "s" => Value::Set(Arc::new(body.as_array().unwrap().iter().map(Value::from_json).collect())),
                    "d" => Value::Dict(Arc::new(
                        body.as_array()
                            .unwrap()
                            .iter()
                            .map(|kv| (Value::from_json(&kv[0]), Value::from_json(&kv[1])))
                            .collect(),
                    )),
                    "re" => Value::Re(Arc::new(Re::new(body.as_str().unwrap(), o["flags"].as_u64().unwrap() as u32))),
                    "fn" => Value::Fn(Arc::new(FnRef {
                        id: body.as_str().unwrap().to_string(),
                        cells: o
                            .get("cells")
                            .and_then(|c| c.as_object())
                            .map(|c| c.iter().map(|(k, v)| (k.clone(), Value::from_json(v))).collect())
                            .unwrap_or_default(),
                        defaults: o
                            .get("defaults")
                            .and_then(|d| d.as_array())
                            .map(|d| d.iter().map(Value::from_json).collect())
                            .unwrap_or_default(),
                    })),
                    "obj" => Value::Obj(
                        Arc::from(body.as_str().unwrap()),
                        Arc::new(
                            o["attrs"]
                                .as_object()
                                .unwrap()
                                .iter()
                                .map(|(k, v)| (k.clone(), Value::from_json(v)))
                                .collect(),
                        ),
                    ),
                    "mod" => Value::Module(Arc::from(body.as_str().unwrap())),
                    "float" => Value::Float(body.as_str().unwrap().parse().unwrap_or(f64::NAN)),
                    _ => Value::Opaque(Arc::from(tag.as_str())),
                }
            }
        }
    }
}

/// Where the bundle is read from.
pub fn bundle_path() -> String {
    std::env::var("RSBOT_DATA").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../data/export/bundle.json").to_string())
}

/// The bundle's bytes when the host hands them over ([`supply`]) instead of a file.
static SUPPLIED: std::sync::Mutex<Option<Vec<u8>>> = std::sync::Mutex::new(None);

/// Give the bundle's JSON bytes directly, for builds without a file system (the WebAssembly
/// page downloads the bundle and passes it in). Call before the first lookup; the bytes are
/// parsed on first use and then dropped. Without it the bundle is read from [`bundle_path`].
pub fn supply(bytes: Vec<u8>) {
    *SUPPLIED.lock().unwrap() = Some(bytes);
}

static BUNDLE: LazyLock<HashMap<String, Value>> = LazyLock::new(|| {
    let supplied = SUPPLIED.lock().unwrap().take();
    let bytes = supplied.unwrap_or_else(|| {
        let path = bundle_path();
        std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read PyBot data bundle {path}: {e}"))
    });
    let j: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(&bytes).expect("bundle is JSON");
    drop(bytes);
    j.iter().map(|(k, v)| (k.clone(), Value::from_json(v))).collect()
});

/// Load the bundle now (so the first reply does not pay for it).
pub fn load() -> usize {
    BUNDLE.len()
}

/// A module global by its Python name, e.g. "bot.modules.answers.REFERRING".
pub fn get(name: &str) -> &'static Value {
    BUNDLE.get(name).unwrap_or_else(|| panic!("PyBot data has no global {name}"))
}

/// A module global if it exists.
pub fn try_get(name: &str) -> Option<&'static Value> {
    BUNDLE.get(name)
}

/// A tuple / list / set of strings, in order.
pub fn strs(name: &str) -> Vec<String> {
    get(name).strs()
}

/// A set (or any collection) of strings, for membership tests.
pub fn set(name: &str) -> HashSet<String> {
    get(name).str_set()
}

/// A compiled regex global (`re.compile(...)` at module level).
pub fn re(name: &str) -> &'static Re {
    get(name).as_re().unwrap_or_else(|| panic!("{name} is not a regex"))
}

/// A string global.
pub fn string(name: &str) -> String {
    get(name).str().to_string()
}

/// A `{str: str}` dict global.
pub fn str_map(name: &str) -> HashMap<String, String> {
    get(name).str_map()
}

/// Every global whose name starts with `prefix` (e.g. one module's).
pub fn names_with_prefix(prefix: &str) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = BUNDLE.keys().filter(|k| k.starts_with(prefix)).map(String::as_str).collect();
    v.sort();
    v
}

// ---------------------------------------------------------------------------------------------
// Values as mutable Python objects (conversation memory).
//
// Containers are `Arc`-shared and copied on write (`Arc::make_mut`), so cloning a whole memory
// for every module (Python's deepcopy per module) costs almost nothing until a module writes.

impl From<&str> for Value {
    fn from(s: &str) -> Value {
        Value::Str(Arc::from(s))
    }
}
impl From<String> for Value {
    fn from(s: String) -> Value {
        Value::Str(Arc::from(s.as_str()))
    }
}
impl From<&String> for Value {
    fn from(s: &String) -> Value {
        Value::Str(Arc::from(s.as_str()))
    }
}
impl From<bool> for Value {
    fn from(b: bool) -> Value {
        Value::Bool(b)
    }
}
impl From<i64> for Value {
    fn from(i: i64) -> Value {
        Value::Int(i)
    }
}
impl From<usize> for Value {
    fn from(i: usize) -> Value {
        Value::Int(i as i64)
    }
}
impl From<f64> for Value {
    fn from(f: f64) -> Value {
        Value::Float(f)
    }
}
impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(o: Option<T>) -> Value {
        o.map(Into::into).unwrap_or(Value::None)
    }
}
impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Value {
        Value::List(Arc::new(v))
    }
}
impl From<Vec<String>> for Value {
    fn from(v: Vec<String>) -> Value {
        Value::List(Arc::new(v.into_iter().map(Value::from).collect()))
    }
}

impl PartialEq for Value {
    /// Python `==` on these values (lists and tuples compare by items; a regex by its source).
    fn eq(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (None, None) => true,
            (Bool(a), Bool(b)) => a == b,
            (Int(a), Int(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Int(a), Float(b)) | (Float(b), Int(a)) => (*a as f64) == *b,
            (Bool(a), Int(b)) | (Int(b), Bool(a)) => (*a as i64) == *b,
            (Str(a), Str(b)) => a == b,
            (List(a), List(b)) | (Tuple(a), Tuple(b)) => a == b,
            (Set(a), Set(b)) => a.len() == b.len() && a.iter().all(|x| b.contains(x)),
            (Dict(a), Dict(b)) => a.len() == b.len() && a.iter().all(|(k, v)| b.iter().any(|(k2, v2)| k == k2 && v == v2)),
            (Re(a), Re(b)) => a.pattern() == b.pattern() && a.flags() == b.flags(),
            (Fn(a), Fn(b)) => a.id == b.id,
            (Obj(c, a), Obj(d, b)) => c == d && a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.0 == y.0 && x.1 == y.1),
            (Module(a), Module(b)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    /// An empty dict.
    pub fn dict() -> Value {
        Value::Dict(Arc::new(Vec::new()))
    }
    /// An empty list.
    pub fn list() -> Value {
        Value::List(Arc::new(Vec::new()))
    }
    /// `d[key] = v` (insertion order kept, as in Python). Turns a non-dict into a dict.
    pub fn set_key(&mut self, key: impl Into<Value>, v: impl Into<Value>) {
        let key = key.into();
        let v = v.into();
        if !matches!(self, Value::Dict(_)) {
            *self = Value::dict();
        }
        if let Value::Dict(d) = self {
            let d = Arc::make_mut(d);
            match d.iter_mut().find(|(k, _)| *k == key) {
                Some(slot) => slot.1 = v,
                None => d.push((key, v)),
            }
        }
    }
    /// `d.pop(key, None)`.
    pub fn del_key(&mut self, key: &str) -> Option<Value> {
        if let Value::Dict(d) = self {
            let d = Arc::make_mut(d);
            if let Some(i) = d.iter().position(|(k, _)| k.as_str() == Some(key)) {
                return Some(d.remove(i).1);
            }
        }
        None
    }
    /// A mutable reference to `d[key]` (a str key), if present.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        match self {
            Value::Dict(d) => Arc::make_mut(d).iter_mut().find(|(k, _)| k.as_str() == Some(key)).map(|(_, v)| v),
            _ => None,
        }
    }
    /// `d.setdefault(key, default)`, returning the value now there.
    pub fn setdefault(&mut self, key: &str, default: impl Into<Value>) -> &mut Value {
        if self.get(key).is_none() {
            self.set_key(key, default);
        }
        self.get_mut(key).unwrap()
    }
    /// `lst.append(v)`. Turns a non-list into a list.
    pub fn push(&mut self, v: impl Into<Value>) {
        if !matches!(self, Value::List(_)) {
            *self = Value::list();
        }
        if let Value::List(l) = self {
            Arc::make_mut(l).push(v.into());
        }
    }
    /// The items of a list/tuple/set, mutable (lists only).
    pub fn items_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::List(l) => Some(Arc::make_mut(l)),
            _ => None,
        }
    }
    /// `len(x)` for str (characters), containers; 0 otherwise.
    pub fn len(&self) -> usize {
        match self {
            Value::Str(s) => s.chars().count(),
            Value::List(v) | Value::Tuple(v) | Value::Set(v) => v.len(),
            Value::Dict(d) => d.len(),
            _ => 0,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn is_str(&self) -> bool {
        matches!(self, Value::Str(_))
    }
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_) | Value::Tuple(_))
    }
    pub fn is_dict(&self) -> bool {
        matches!(self, Value::Dict(_))
    }
    /// `x in container` for a str item (list/tuple/set items, dict keys, or a substring of a str).
    pub fn contains_str(&self, item: &str) -> bool {
        match self {
            Value::Str(s) => s.contains(item),
            Value::List(v) | Value::Tuple(v) | Value::Set(v) => v.iter().any(|x| x.as_str() == Some(item)),
            Value::Dict(d) => d.iter().any(|(k, _)| k.as_str() == Some(item)),
            _ => false,
        }
    }

    /// Plain JSON (what the oracle files hold): sequences as arrays, str-keyed dicts as objects,
    /// other dicts as [[k, v], ...], objects/regexes/functions in their export tags.
    pub fn to_plain(&self) -> serde_json::Value {
        use serde_json::{json, Value as J};
        match self {
            Value::None => J::Null,
            Value::Bool(b) => J::Bool(*b),
            Value::Int(i) => json!(i),
            Value::Float(f) => json!(f),
            Value::Str(s) => J::String(s.to_string()),
            Value::List(v) | Value::Tuple(v) | Value::Set(v) => J::Array(v.iter().map(Value::to_plain).collect()),
            Value::Dict(d) => {
                if d.iter().all(|(k, _)| k.is_str()) {
                    J::Object(d.iter().map(|(k, v)| (k.str().to_string(), v.to_plain())).collect())
                } else {
                    J::Array(d.iter().map(|(k, v)| J::Array(vec![k.to_plain(), v.to_plain()])).collect())
                }
            }
            Value::Re(r) => json!({"re": r.pattern(), "flags": r.flags()}),
            Value::Fn(f) => json!({"fn": f.id}),
            Value::Obj(c, a) => json!({"obj": &**c, "attrs": a.iter().map(|(k, v)| (k.clone(), v.to_plain())).collect::<serde_json::Map<_, _>>()}),
            Value::Module(m) => json!({"mod": &**m}),
            Value::Opaque(o) => json!({"opaque": &**o}),
        }
    }
}

/// Mark a function that has not been ported yet. Panics like a Python exception: the runtime
/// skips the module that hit it, and the checkers count it by name.
#[macro_export]
macro_rules! unported {
    ($name:expr) => {
        panic!("UNPORTED {}", $name)
    };
}
