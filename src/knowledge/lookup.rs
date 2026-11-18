//! Port of `bot/knowledge/lookup.py`: the shared matcher that finds the knowledge entry a line
//! asks for (`find`), and the `Entry` every domain builds its knowledge from.

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, RwLock};
use crate::understood::Understood;
use std::sync::Arc;

pub use crate::knowledge::entry::{Entry, KeyPart};

/// The alternatives of a key part (`part if isinstance(part, tuple) else (part,)`).
pub fn alternatives(part: &KeyPart) -> &[String] {
    match part {
        KeyPart::Word(s) => std::slice::from_ref(s),
        KeyPart::Any(v) => v,
    }
}

/// Every entry in the knowledge base (`bot.knowledge.entries()`).
pub fn all_entries() -> &'static [Entry] {
    crate::knowledge::entries()
}

/// `guard_passes(item, reading)`: an entry without a guard passes; a guard that raises fails.
pub fn guard_passes(item: &Entry, reading: &Understood) -> bool {
    match &item.guard {
        None => true,
        Some(g) => crate::knowledge::registry::guard(g, reading),
    }
}

/// `find(understood, entries=None, reject=None, use_context=False)`: the entry the line asks
/// for, or None. `entries` None means every entry; `reject` turns an entry down.
pub fn find(understood: &Understood, entries: Option<&[&'static Entry]>, reject: Option<&dyn Fn(&Entry) -> bool>, use_context: bool) -> Option<&'static Entry> {
    let _ = (understood, entries, reject, use_context);
    todo!("lookup::find")
}

/// What `_vocabulary(entries)` works out: each domain's key words with how many entries use
/// them, the same over every domain, and every form of those words.
#[derive(Clone, Debug, Default)]
pub struct Vocabulary {
    pub by_domain: std::collections::HashMap<String, std::collections::HashMap<String, usize>>,
    pub every: std::collections::HashMap<String, usize>,
    pub known: std::collections::HashSet<String>,
}

/// `_vocabulary(entries)`: `(by_domain, every, known)`. Python caches it by the identity of the
/// list; the port caches the result for the whole knowledge base (the only list the engine
/// passes in a turn) by the slice's address and length.
pub fn _vocabulary(entries: &[&'static Entry]) -> Arc<Vocabulary> {
    static CACHE: LazyLock<RwLock<Vec<((usize, usize), Arc<Vocabulary>)>>> = LazyLock::new(|| RwLock::new(Vec::new()));
    let id = (entries.as_ptr() as usize, entries.len());
    let first = entries.first().map(|e| *e as *const Entry as usize).unwrap_or(0);
    let key = (id.1, first);
    if let Some((_, v)) = CACHE.read().unwrap().iter().find(|(k, _)| *k == key) {
        return v.clone();
    }
    let (by_domain, every) = _domain_rarity(entries);
    let slips: HashSet<String> = entries.iter().flat_map(|item| item.slips.iter().map(|s| s.to_lowercase())).collect();
    let every: HashMap<String, usize> = every.into_iter().filter(|(word, _)| !slips.contains(&word.to_lowercase())).collect();
    let mut known = HashSet::new();
    for word in every.keys() {
        known.extend(_forms(word).iter().cloned());
    }
    let v = Arc::new(Vocabulary { by_domain, every, known });
    if entries.iter().all(|item| !item.domain.is_empty()) {
        let mut cache = CACHE.write().unwrap();
        while cache.len() >= 6 {
            cache.remove(0);
        }
        cache.push((key, v.clone()));
    }
    v
}

/// Every entry as references, for the functions that take Python's `entries` list.
pub fn all_entry_refs() -> &'static [&'static Entry] {
    static REFS: LazyLock<Vec<&'static Entry>> = LazyLock::new(|| all_entries().iter().collect());
    &REFS
}

const M: &str = "bot.knowledge.lookup.";

fn set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}

static WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}WORD")));
static MIN_FORM: LazyLock<usize> = LazyLock::new(|| data::get(&format!("{M}MIN_FORM")).as_int().unwrap_or(3) as usize);
static _CONTRACTION: LazyLock<HashSet<String>> = LazyLock::new(|| set("_CONTRACTION"));
static _COMMON: LazyLock<HashSet<String>> = LazyLock::new(|| set("_COMMON"));
static _STEMS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_STEMS"));
static ENDINGS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}ENDINGS")));
static _FUNCTION_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_FUNCTION_WORDS"));
static COLOUR_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("COLOUR_WORDS"));
static IRREGULAR_PLURALS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map(&format!("{M}IRREGULAR_PLURALS")));
static NOT_A_WORD_WITH_AN_S: LazyLock<HashSet<String>> = LazyLock::new(|| set("NOT_A_WORD_WITH_AN_S"));
static NOT_THE_STEM_WITH_E: LazyLock<HashSet<String>> = LazyLock::new(|| set("NOT_THE_STEM_WITH_E"));
static NOT_THE_STEM: LazyLock<HashSet<String>> = LazyLock::new(|| set("NOT_THE_STEM"));
static NOT_A_BASE: LazyLock<HashSet<String>> = LazyLock::new(|| set("NOT_A_BASE"));
static NOT_THE_DOUBLED_STEM: LazyLock<HashSet<String>> = LazyLock::new(|| set("NOT_THE_DOUBLED_STEM"));
static SLIPS: LazyLock<HashMap<String, HashSet<String>>> =
    LazyLock::new(|| data::get(&format!("{M}SLIPS")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect());
static US_SPELLINGS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.understanding.lexicon.US_SPELLINGS"));
static LEX_CHAT: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.understanding.lexicon.CHAT"));
static LEX_EXPAND: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.understanding.lexicon.EXPAND"));
static COMMON_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.normalize.LEXICON"));

/// `text.translate(_CURLY)`: curly apostrophes made straight.
pub fn uncurl(text: &str) -> String {
    text.replace(['\u{2019}', '\u{2018}', '\u{2bc}'], "'")
}

/// `tokens(text)`: the lower-cased words of a text.
pub fn tokens(text: &str) -> Vec<String> {
    WORD.findall_str(&uncurl(text).to_lowercase())
}

/// The forms of one word (`_forms`), shared.
pub type Forms = Arc<HashSet<String>>;

/// `_forms(word)`: every form a key word can be typed in (plural, past, -ing, US spelling),
/// worked out once per word.
pub fn _forms(word: &str) -> Forms {
    static CACHE: LazyLock<RwLock<HashMap<String, Forms>>> = LazyLock::new(|| RwLock::new(HashMap::new()));
    if let Some(f) = CACHE.read().unwrap().get(word) {
        return f.clone();
    }
    let f: Forms = Arc::new(_work_out_forms(word));
    CACHE.write().unwrap().insert(word.to_string(), f.clone());
    f
}

/// `_work_out_forms(word)`.
pub fn _work_out_forms(word: &str) -> HashSet<String> {
    let word = uncurl(word);
    let w: Vec<char> = word.chars().collect();
    let n = w.len();
    let cut = |k: usize| -> String { w[..n - k].iter().collect() };
    let mut forms = HashSet::new();
    forms.insert(word.clone());
    if let Some(other) = US_SPELLINGS.get(&word) {
        if !other.is_empty() {
            forms.insert(other.clone());
        }
    }
    if word.ends_with("ies") && n > 5 {
        forms.insert(cut(3) + "y");
    }
    if word.ends_with('y') && n > 3 && !"aeiou".contains(w[n - 2]) {
        forms.insert(cut(1) + "ies");
    }
    if let Some(p) = IRREGULAR_PLURALS.get(&word) {
        forms.insert(p.clone());
        return forms;
    }
    if word.ends_with("'s") && n > 3 && !_CONTRACTION.contains(&word) {
        forms.insert(cut(2));
    } else if word.ends_with("s'") && n > 3 {
        forms.insert(cut(2));
        forms.insert(cut(1));
    }
    if n > 3 {
        if word.ends_with('s') && !word.ends_with("ss") && !NOT_A_WORD_WITH_AN_S.contains(&word) {
            _add(&mut forms, cut(1));
        }
        if word.ends_with("es") && n > 4 && ("sxzi".contains(w[n - 3]) || matches!(w[n - 4..n - 2].iter().collect::<String>().as_str(), "ch" | "sh") || w[n - 3] == 'o') {
            _add(&mut forms, cut(2));
        }
        if word.ends_with("ed") && n > 4 {
            if _engine_reads(&cut(2)) && _engine_reads(&cut(1)) {
                _add(&mut forms, cut(1));
            } else {
                _add(&mut forms, cut(2));
            }
            if n > 5 && _doubles_for_an_ending(&cut(2)) {
                _add(&mut forms, cut(3));
            }
        }
        if word.ends_with("ing") && n > 5 {
            let stem = cut(3);
            if !NOT_THE_STEM_WITH_E.contains(&word) {
                _add(&mut forms, stem.clone() + "e");
            }
            if !NOT_THE_STEM.contains(&word)
                && !NOT_A_BASE.contains(&stem)
                && (!_cvc(&stem) || (text::len(&stem) > 2 && stem.ends_with('y')) || _bare_is_the_word(&stem))
            {
                _add(&mut forms, stem.clone());
            }
            if _doubles_for_an_ending(&stem) && !NOT_THE_DOUBLED_STEM.contains(&word) {
                let s: Vec<char> = stem.chars().collect();
                _add(&mut forms, s[..s.len() - 1].iter().collect());
            }
        }
    }
    forms
}

/// `_add(forms, form)`: a form long enough to count.
fn _add(forms: &mut HashSet<String>, form: String) {
    if text::len(&form) >= *MIN_FORM {
        forms.insert(form);
    }
}

/// `_doubles_for_an_ending(stem)`: "stopp" in "stopping" doubled its last letter.
pub fn _doubles_for_an_ending(stem: &str) -> bool {
    let s: Vec<char> = stem.chars().collect();
    let n = s.len();
    if n < 4 || s[n - 1] != s[n - 2] || "aeiou".contains(s[n - 1]) || s[n - 1] == 's' {
        return false;
    }
    if s[n - 1] == 'l' {
        return true;
    }
    if !"aeiou".contains(s[n - 3]) {
        return false;
    }
    _is_consonant(&s, n as isize - 4)
}

/// `_is_consonant(word, index)`.
fn _is_consonant(word: &[char], index: isize) -> bool {
    if index < 0 {
        return false;
    }
    let letter = word[index as usize];
    if !"aeiou".contains(letter) {
        return true;
    }
    letter == 'u' && index > 0 && word[index as usize - 1] == 'q'
}

/// `_cvc(stem)`: consonant-vowel-consonant at the end of a short stem ("hop", "stop").
pub fn _cvc(stem: &str) -> bool {
    let s: Vec<char> = stem.chars().collect();
    let n = s.len();
    if !(3..=4).contains(&n) {
        return false;
    }
    let v = "aeiou";
    !v.contains(s[n - 1]) && v.contains(s[n - 2]) && !v.contains(s[n - 3])
}

/// `_bare_is_the_word(stem)`.
pub fn _bare_is_the_word(stem: &str) -> bool {
    _engine_reads(stem) && !_engine_reads(&format!("{stem}e")) && !_FUNCTION_WORDS.contains(stem)
}

/// `_stem(word)`: the word without its commonest ending.
pub fn _stem(word: &str) -> String {
    for ending in ENDINGS.iter() {
        if word.ends_with(ending.as_str()) && text::len(word) >= text::len(ending) + 3 {
            return word[..word.len() - ending.len()].to_string();
        }
    }
    word.to_string()
}

/// `_engine_reads(word)`: a word the understanding engine knows as typed.
pub fn _engine_reads(word: &str) -> bool {
    _COMMON.contains(word)
        || COMMON_WORDS.contains(word)
        || LEX_CHAT.contains_key(word)
        || LEX_EXPAND.contains_key(word)
        || _STEMS.contains(&_stem(word))
        || crate::understanding::lexicon::every_word().contains(word)
        || COLOUR_WORDS.contains(word)
}

/// `_domain_rarity(entries)`: `(by_domain, every)`, how many entries use each key word.
pub fn _domain_rarity(entries: &[&'static Entry]) -> (HashMap<String, HashMap<String, usize>>, HashMap<String, usize>) {
    let mut by_domain: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let mut owners: HashMap<String, HashSet<&str>> = HashMap::new();
    for item in entries {
        let counts = by_domain.entry(item.domain.clone()).or_default();
        let mut words: HashSet<String> = HashSet::new();
        for part in &item.key {
            for alternative in alternatives(part) {
                words.extend(text::split(&alternative.to_lowercase()));
            }
        }
        for word in words {
            *counts.entry(word.clone()).or_default() += 1;
            owners.entry(word).or_default().insert(item.id.as_str());
        }
    }
    let mut every = HashMap::new();
    for (word, owned) in owners {
        if let Some(slips) = SLIPS.get(&word) {
            if owned.iter().all(|id| slips.contains(*id)) {
                continue;
            }
        }
        every.insert(word, owned.len());
    }
    (by_domain, every)
}
