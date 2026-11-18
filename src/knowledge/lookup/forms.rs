//! lookup.py, the word forms and the phrase finder: `_forms` and how a key word is typed,
//! `_wanted` / `_phrase_span*` (where a phrase sits in a line of word forms), `_part_matches`
//! (every way a key position is said), with the per-line memo Python keeps in `seen`.

use super::*;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

/// The forms of one word (`_forms`): Python's frozenset, shared.
pub type Forms = Arc<HashSet<String>>;

/// `frozenset()`.
pub fn no_forms() -> Forms {
    static EMPTY: LazyLock<Forms> = LazyLock::new(|| Arc::new(HashSet::default()));
    EMPTY.clone()
}

/// `a & b` is not empty.
#[inline]
pub fn meets<S1: std::hash::BuildHasher, S2: std::hash::BuildHasher>(a: &std::collections::HashSet<String, S1>, b: &std::collections::HashSet<String, S2>) -> bool {
    if a.len() <= b.len() {
        a.iter().any(|x| b.contains(x))
    } else {
        b.iter().any(|x| a.contains(x))
    }
}

/// A line read as word forms: Python's `forms` list. It carries an identity (`id`) because
/// Python's per-line memo (`seen`) checks `got[0] is forms`.
#[derive(Clone, Debug)]
pub struct FormLine {
    pub id: u64,
    pub forms: Vec<Forms>,
}

static NEXT_LINE: AtomicU64 = AtomicU64::new(1);

impl FormLine {
    pub fn new(forms: Vec<Forms>) -> FormLine {
        FormLine { id: NEXT_LINE.fetch_add(1, Ordering::Relaxed), forms }
    }
    /// `[_forms(word) for word in words]`.
    pub fn of<S: AsRef<str>>(words: &[S]) -> FormLine {
        FormLine::new(words.iter().map(|w| _forms(w.as_ref())).collect())
    }
}

impl Deref for FormLine {
    type Target = Vec<Forms>;
    fn deref(&self) -> &Vec<Forms> {
        &self.forms
    }
}

impl DerefMut for FormLine {
    fn deref_mut(&mut self) -> &mut Vec<Forms> {
        &mut self.forms
    }
}

/// One key position as Python passes it to `_part_matches`: a phrase (str) or a tuple of them.
#[derive(Clone, Copy, Debug)]
pub enum Part<'a> {
    Str(&'a str),
    Tuple(&'a [String]),
}

/// Python's `id(part)` for the memo: where the part's data lives.
pub type PartKey = (usize, usize, u8);

impl<'a> Part<'a> {
    /// `part if isinstance(part, tuple) else (part,)`.
    pub fn alternatives(&self) -> Vec<&'a str> {
        match *self {
            Part::Str(s) => vec![s],
            Part::Tuple(v) => v.iter().map(String::as_str).collect(),
        }
    }
    pub fn key(&self) -> PartKey {
        match *self {
            Part::Str(s) => (s.as_ptr() as usize, s.len(), 0),
            Part::Tuple(v) => (v.as_ptr() as usize, v.len(), 1),
        }
    }
}

impl<'a> From<&'a KeyPart> for Part<'a> {
    fn from(p: &'a KeyPart) -> Part<'a> {
        match p {
            KeyPart::Word(s) => Part::Str(s),
            KeyPart::Any(v) => Part::Tuple(v),
        }
    }
}

/// One way a part is said: `(weight, words, span)`.
#[derive(Clone, Debug)]
pub struct Hit {
    pub weight: i64,
    pub words: Arc<[String]>,
    pub span: Vec<usize>,
}

/// `_where`: every form's positions, and its first position.
pub type Where = (HashMap<String, Vec<usize>>, HashMap<String, usize>);

/// Python's `seen` dict: what one line has already worked out. Shared and aliased as Python's
/// dict is (`memo = seen`), so it is an `Rc<RefCell<..>>`.
#[derive(Default)]
pub struct Seen {
    /// `seen[id(part)]` (not checked against the line, as in Python).
    pub parts: HashMap<PartKey, Rc<Vec<Hit>>>,
    pub line_forms: Option<(u64, Arc<HashSet<String>>)>,
    pub where_: Option<(u64, Rc<Where>)>,
    /// `seen[_PHRASE_SPANS]`: (forms, {phrase: span}, {('between', phrase): span}).
    pub spans: Option<(u64, HashMap<String, Vec<usize>>, HashMap<String, Vec<usize>>)>,
    pub wanting: Option<(u64, bool)>,
    pub about_someone: HashMap<PartKey, (u64, bool)>,
    /// `seen[_FRAME_MASK]`: (forms, masked and its memo when masked is not forms, frame words).
    pub frame_mask: Option<(u64, Option<(Rc<FormLine>, SeenRef)>, Arc<HashSet<String>>)>,
    /// `seen[_ASK_PHRASE_MASK]`: (forms, masked, memo).
    pub ask_phrase_mask: Option<(u64, Rc<FormLine>, SeenRef)>,
    /// `seen['keeps-failing-at']`.
    pub keeps_failing_at: Option<Option<usize>>,
}

pub type SeenRef = Rc<RefCell<Seen>>;

/// `{}`: a new memo.
pub fn new_seen() -> SeenRef {
    Rc::new(RefCell::new(Seen::default()))
}

/// `text.translate(_CURLY)`: curly apostrophes made straight.
pub fn uncurl(text: &str) -> String {
    text.replace(['\u{2019}', '\u{2018}', '\u{2bc}'], "'")
}

/// `tokens(text)`: the lower-cased words of a text.
pub fn tokens(text: &str) -> Vec<String> {
    WORD.findall_str(&text::lower(&uncurl(text)))
}

/// `_forms(word)`: every form a key word can be typed in (plural, past, -ing, US spelling),
/// worked out once per word.
pub fn _forms(word: &str) -> Forms {
    static CACHE: LazyLock<RwLock<HashMap<String, Forms>>> = LazyLock::new(|| RwLock::new(HashMap::default()));
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
    let mut forms = HashSet::default();
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

/// `_one_edit(shorter, longer)`: the two words are one typing slip apart.
pub fn _one_edit(shorter: &str, longer: &str) -> bool {
    if shorter == longer {
        return true;
    }
    let mut s: Vec<char> = shorter.chars().collect();
    let mut l: Vec<char> = longer.chars().collect();
    if s.len().abs_diff(l.len()) > 1 {
        return false;
    }
    if s.len() > l.len() {
        std::mem::swap(&mut s, &mut l);
    }
    let mut i = 0;
    while i < s.len() && s[i] == l[i] {
        i += 1;
    }
    if s.len() < l.len() {
        return s[i..] == l[i + 1..];
    }
    let (rest, tail) = (&s[i..], &l[i..]);
    if rest.get(1..).unwrap_or(&[]) == tail.get(1..).unwrap_or(&[]) {
        return true;
    }
    rest.len() > 1 && rest[0] == tail[1] && rest[1] == tail[0] && rest[2..] == tail[2..]
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

/// `_typo_forms(word, vocabulary)`: the forms of the one key word a misspelt word is a slip of.
pub fn _typo_forms(word: &str, vocabulary: &HashMap<String, usize>) -> HashSet<String> {
    if text::len(word) < *MIN_TYPO_LENGTH || _engine_reads(word) {
        return HashSet::default();
    }
    let first = word.chars().next();
    let n = text::len(word);
    let close: Vec<&String> = vocabulary
        .keys()
        .filter(|other| other.chars().next() == first && text::len(other).abs_diff(n) <= 1 && _one_edit(word, other))
        .collect();
    let families: HashSet<String> = close.iter().map(|other| _stem(other)).collect();
    if families.len() != 1 {
        return HashSet::default();
    }
    let mut forms = HashSet::default();
    for other in close {
        forms.extend(_forms(other).iter().cloned());
    }
    forms
}

/// `_wanted(phrase)`: the forms of each word of a phrase.
/// `_wanted(phrase)`: the forms of each word of a phrase.
pub fn _wanted(phrase: &str) -> Arc<Vec<Forms>> {
    if let Some(info) = static_phrase(phrase) {
        return info.wanted.clone();
    }
    cached_wanted(phrase)
}

fn cached_wanted(phrase: &str) -> Arc<Vec<Forms>> {
    static CACHE: LazyLock<RwLock<HashMap<String, Arc<Vec<Forms>>>>> = LazyLock::new(|| RwLock::new(HashMap::default()));
    if let Some(got) = CACHE.read().unwrap().get(phrase) {
        return got.clone();
    }
    let low = text::lower(phrase);
    let got: Vec<Forms> = if PLURAL_ONLY_PHRASES.contains(&low) {
        text::split(&low).into_iter().map(|w| Arc::new(std::iter::once(w).collect::<HashSet<String>>())).collect()
    } else {
        text::split(&low).iter().map(|w| _forms(w)).collect()
    };
    let got = Arc::new(got);
    CACHE.write().unwrap().insert(phrase.to_string(), got.clone());
    got
}

/// `_wanted_bare(phrase)`: the phrase's forms without its articles, when that leaves enough.
/// `_wanted_bare(phrase)`: the phrase's forms without its articles, when that leaves enough.
pub fn _wanted_bare(phrase: &str) -> Option<Arc<Vec<Forms>>> {
    if let Some(info) = static_phrase(phrase) {
        return info.bare.clone();
    }
    cached_wanted_bare(phrase)
}

fn cached_wanted_bare(phrase: &str) -> Option<Arc<Vec<Forms>>> {
    static CACHE: LazyLock<RwLock<HashMap<String, Option<Arc<Vec<Forms>>>>>> = LazyLock::new(|| RwLock::new(HashMap::default()));
    if let Some(got) = CACHE.read().unwrap().get(phrase) {
        return got.clone();
    }
    let words = text::split(&text::lower(phrase));
    let kept: Vec<&String> = words.iter().filter(|w| !_ARTICLE_WORDS.contains(*w)).collect();
    let bare = if kept.len() as i64 >= *_MIN_BARE_WORDS && kept.len() < words.len() {
        Some(Arc::new(kept.iter().map(|w| _forms(w)).collect::<Vec<_>>()))
    } else {
        None
    };
    CACHE.write().unwrap().insert(phrase.to_string(), bare.clone());
    bare
}

/// `_phrase_span_bare(forms, wanted, seen)`: the phrase said with articles skipped between.
pub fn _phrase_span_bare(forms: &[Forms], wanted: &[Forms]) -> Vec<usize> {
    let n = wanted.len();
    if forms.is_empty() {
        return vec![];
    }
    let last = forms.len() - 1;
    for start in 0..forms.len() {
        if !meets(&forms[start], &wanted[0]) {
            continue;
        }
        let mut span = vec![start];
        let mut here = start + 1;
        let mut complete = true;
        for want in wanted.iter().take(n).skip(1) {
            while here <= last && !meets(&forms[here], want) {
                if !meets(&forms[here], &_ARTICLE_FORMS) {
                    break;
                }
                here += 1;
            }
            if here > last || !meets(&forms[here], want) {
                complete = false;
                break;
            }
            span.push(here);
            here += 1;
        }
        if complete {
            return span;
        }
    }
    vec![]
}

/// `_shape(phrase)`: the phrase's words and what saying it is worth.
/// `_shape(phrase)`: the phrase's words and what saying it is worth.
pub fn _shape(phrase: &str) -> (Arc<[String]>, i64) {
    if let Some(info) = static_phrase(phrase) {
        return info.shape.clone();
    }
    cached_shape(phrase)
}

fn cached_shape(phrase: &str) -> (Arc<[String]>, i64) {
    static CACHE: LazyLock<RwLock<HashMap<String, (Arc<[String]>, i64)>>> = LazyLock::new(|| RwLock::new(HashMap::default()));
    if let Some(got) = CACHE.read().unwrap().get(phrase) {
        return got.clone();
    }
    let words: Arc<[String]> = text::split(&text::lower(phrase)).into();
    let weight = if words.len() > 1 { *PHRASE_WEIGHT } else { *WORD_WEIGHT };
    let got = (words, weight);
    CACHE.write().unwrap().insert(phrase.to_string(), got.clone());
    got
}

/// `_line_forms(forms, seen)`: every form in the line.
pub fn _line_forms(forms: &FormLine, seen: Option<&SeenRef>) -> Arc<HashSet<String>> {
    if let Some(s) = seen {
        if let Some((id, got)) = &s.borrow().line_forms {
            if *id == forms.id {
                return got.clone();
            }
        }
    }
    let mut every = HashSet::default();
    for word_forms in forms.iter() {
        every.extend(word_forms.iter().cloned());
    }
    let every = Arc::new(every);
    if let Some(s) = seen {
        s.borrow_mut().line_forms = Some((forms.id, every.clone()));
    }
    every
}

/// `_where(forms, seen)`: each form's positions, and its first.
pub fn _where(forms: &FormLine, seen: Option<&SeenRef>) -> Rc<Where> {
    if let Some(s) = seen {
        if let Some((id, got)) = &s.borrow().where_ {
            if *id == forms.id {
                return got.clone();
            }
        }
    }
    let mut where_: HashMap<String, Vec<usize>> = HashMap::default();
    for (index, word_forms) in forms.iter().enumerate() {
        for form in word_forms.iter() {
            where_.entry(form.clone()).or_default().push(index);
        }
    }
    let firsts: HashMap<String, usize> = where_.iter().map(|(f, at)| (f.clone(), at[0])).collect();
    let got = Rc::new((where_, firsts));
    if let Some(s) = seen {
        s.borrow_mut().where_ = Some((forms.id, got.clone()));
    }
    got
}

/// The memo's span table for this line (`seen[_PHRASE_SPANS]`), made fresh for a new line.
fn spans_get(seen: &SeenRef, forms: &FormLine, between: bool, phrase: &str) -> Option<Vec<usize>> {
    let mut s = seen.borrow_mut();
    match &s.spans {
        Some((id, plain, gaps)) if *id == forms.id => (if between { gaps } else { plain }).get(phrase).cloned(),
        _ => {
            s.spans = Some((forms.id, HashMap::default(), HashMap::default()));
            None
        }
    }
}

fn spans_put(seen: &SeenRef, forms: &FormLine, between: bool, phrase: &str, span: Vec<usize>) {
    let mut s = seen.borrow_mut();
    if let Some((id, plain, gaps)) = &mut s.spans {
        if *id == forms.id {
            (if between { gaps } else { plain }).insert(phrase.to_string(), span);
        }
    }
}

/// `_phrase_span(forms, phrase, seen=None)`: the positions the phrase is said at, or ().
pub fn _phrase_span(forms: &FormLine, phrase: &str, seen: Option<&SeenRef>) -> Vec<usize> {
    if let Some(s) = seen {
        if let Some(got) = spans_get(s, forms, false, phrase) {
            return got;
        }
        let end = _phrase_span_work(forms, phrase, Some(s));
        spans_put(s, forms, false, phrase, end.clone());
        return end;
    }
    _phrase_span_work(forms, phrase, None)
}

/// `_ask_frame_forms()`: every form of the ask frame's words.
pub fn _ask_frame_forms() -> &'static HashSet<String> {
    static GOT: LazyLock<HashSet<String>> = LazyLock::new(|| {
        let mut got = HashSet::default();
        for word in ASK_FRAME.iter() {
            got.extend(_forms(word).iter().cloned());
        }
        got
    });
    &GOT
}

/// `_phrase_span_between(forms, phrase, seen=None)`: the phrase said with a breath of filler
/// between its words.
pub fn _phrase_span_between(forms: &FormLine, phrase: &str, seen: Option<&SeenRef>) -> Vec<usize> {
    let wanted = _wanted(phrase);
    if wanted.is_empty() || wanted.len() == 1 {
        return vec![];
    }
    if meets(&wanted[0], _ask_frame_forms()) {
        return vec![];
    }
    if let Some(s) = seen {
        if let Some(got) = spans_get(s, forms, true, phrase) {
            return got;
        }
    }
    fn walk(forms: &[Forms], wanted: &[Forms], mut at: usize, next_word: usize, span: Vec<usize>) -> Vec<usize> {
        if next_word == wanted.len() {
            return span;
        }
        let mut gaps = 0;
        while at < forms.len() && gaps <= *_A_BREATH {
            if meets(&forms[at], &wanted[next_word]) {
                let mut longer = span.clone();
                longer.push(at);
                let got = walk(forms, wanted, at + 1, next_word + 1, longer);
                if !got.is_empty() {
                    return got;
                }
            }
            if !meets(&forms[at], &_SAID_BETWEEN) {
                return vec![];
            }
            gaps += 1;
            at += 1;
        }
        vec![]
    }
    let mut answer = vec![];
    for start in 0..forms.len() {
        if meets(&forms[start], &wanted[0]) {
            answer = walk(forms, &wanted, start + 1, 1, vec![start]);
            if !answer.is_empty() {
                break;
            }
        }
    }
    if let Some(s) = seen {
        spans_put(s, forms, true, phrase, answer.clone());
    }
    answer
}

/// `_ONE_SPELLING_EITHER_WAY`: each spelling's other spellings. Python's values are sets; they
/// are walked here in sorted order (Python's order is its string hashing).
static _ONE_SPELLING_EITHER_WAY: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    let mut m: HashMap<String, std::collections::BTreeSet<String>> = HashMap::default();
    for pair in data::get(&format!("{M}_ONE_SPELLING")).items() {
        let it = pair.strs();
        m.entry(it[0].clone()).or_default().insert(it[1].clone());
        m.entry(it[1].clone()).or_default().insert(it[0].clone());
    }
    m.into_iter().map(|(k, v)| (k, v.into_iter().collect())).collect()
});

/// `_the_same_words(phrase)`: the phrase in its other spellings ("won't" / "will not").
/// `_the_same_words(phrase)`: the phrase in its other spellings ("won't" / "will not").
pub fn _the_same_words(phrase: &str) -> Arc<Vec<String>> {
    if let Some(info) = static_phrase(phrase) {
        return info.same.clone();
    }
    cached_same_words(phrase)
}

fn cached_same_words(phrase: &str) -> Arc<Vec<String>> {
    static CACHE: LazyLock<RwLock<HashMap<String, Arc<Vec<String>>>>> = LazyLock::new(|| RwLock::new(HashMap::default()));
    if let Some(got) = CACHE.read().unwrap().get(phrase) {
        return got.clone();
    }
    let got = Arc::new(_work_out_the_same_words(phrase));
    CACHE.write().unwrap().insert(phrase.to_string(), got.clone());
    got
}

/// `_work_out_the_same_words(phrase)`.
pub fn _work_out_the_same_words(phrase: &str) -> Vec<String> {
    let words = text::split(&text::lower(phrase));
    if words.is_empty() {
        return vec![];
    }
    let mut out = Vec::new();
    for (i, word) in words.iter().enumerate() {
        for other in _ONE_SPELLING_EITHER_WAY.get(word).map(|v| v.as_slice()).unwrap_or(&[]) {
            let mut v: Vec<String> = words[..i].to_vec();
            if other.contains(' ') {
                v.extend(text::split(other));
            } else {
                v.push(other.clone());
            }
            v.extend(words[i + 1..].iter().cloned());
            out.push(v.join(" "));
        }
    }
    if words.len() > 1 {
        for i in 0..words.len() - 1 {
            let joined = format!("{} {}", words[i], words[i + 1]);
            for other in _ONE_SPELLING_EITHER_WAY.get(&joined).map(|v| v.as_slice()).unwrap_or(&[]) {
                if !other.contains(' ') {
                    let mut v: Vec<String> = words[..i].to_vec();
                    v.push(other.clone());
                    v.extend(words[i + 2..].iter().cloned());
                    out.push(v.join(" "));
                }
            }
        }
    }
    out
}

/// `_phrase_span_work(forms, phrase, seen)`: the phrase in its own spelling, or another.
pub fn _phrase_span_work(forms: &FormLine, phrase: &str, seen: Option<&SeenRef>) -> Vec<usize> {
    let end = _phrase_span_spelling(forms, phrase, seen);
    if !end.is_empty() {
        return end;
    }
    for other in _the_same_words(phrase).iter() {
        let end = _phrase_span_spelling(forms, other, seen);
        if !end.is_empty() {
            return end;
        }
    }
    vec![]
}

/// `_phrase_span_spelling(forms, phrase, seen)`: the first run of positions saying the phrase.
pub fn _phrase_span_spelling(forms: &FormLine, phrase: &str, seen: Option<&SeenRef>) -> Vec<usize> {
    let wanted = _wanted(phrase);
    if wanted.is_empty() {
        return vec![];
    }
    let n = wanted.len();
    let Some(s) = seen else {
        if n == 1 {
            for (i, word_forms) in forms.iter().enumerate() {
                if meets(word_forms, &wanted[0]) {
                    return vec![i];
                }
            }
            return vec![];
        }
        if forms.len() + 1 > n {
            for i in 0..forms.len() + 1 - n {
                if (0..n).all(|j| meets(&forms[i + j], &wanted[j])) {
                    return (i..i + n).collect();
                }
            }
        }
        return vec![];
    };
    let line = _line_forms(forms, Some(s));
    for word_forms in wanted.iter() {
        if !meets(word_forms, &line) {
            return vec![];
        }
    }
    let w = _where(forms, Some(s));
    let (where_, firsts) = (&w.0, &w.1);
    if n == 1 {
        let at = wanted[0].iter().filter_map(|form| firsts.get(form)).min();
        return at.map(|a| vec![*a]).unwrap_or_default();
    }
    let mut hits: Vec<HashSet<usize>> = Vec::new();
    for word_forms in wanted.iter() {
        let mut found = HashSet::default();
        for form in word_forms.iter() {
            if let Some(at) = where_.get(form) {
                found.extend(at.iter().copied());
            }
        }
        if found.is_empty() {
            return vec![];
        }
        hits.push(found);
    }
    let mut starts: Vec<usize> = hits[0].iter().copied().filter(|i| i + n <= forms.len()).collect();
    starts.sort();
    for start in starts {
        if (1..n).all(|j| hits[j].contains(&(start + j))) {
            return (start..start + n).collect();
        }
    }
    vec![]
}

/// `_phrase_found(forms, phrase)`.
pub fn _phrase_found(forms: &FormLine, phrase: &str) -> bool {
    !_phrase_span(forms, phrase, None).is_empty()
}

/// `_part_forms(part)`: every form any alternative of the part is said in.
/// `_part_forms(part)`: every form any alternative of the part is said in.
pub fn _part_forms(part: Part) -> Arc<HashSet<String>> {
    if let Some(got) = STATIC_PARTS.get(&part.key()) {
        return got.clone();
    }
    cached_part_forms(part)
}

fn cached_part_forms(part: Part) -> Arc<HashSet<String>> {
    let mut every = HashSet::default();
    for alternative in part.alternatives() {
        if text::strip(alternative).is_empty() {
            continue;
        }
        for word_forms in _wanted(alternative).iter() {
            every.extend(word_forms.iter().cloned());
        }
    }
    Arc::new(every)
}

/// `_part_matches(part, forms, seen=None)`: every alternative of the part the line says, as
/// (weight, words, span), the heaviest first.
pub fn _part_matches(part: Part, forms: &FormLine, seen: Option<&SeenRef>) -> Rc<Vec<Hit>> {
    let key = part.key();
    if let Some(s) = seen {
        if let Some(got) = s.borrow().parts.get(&key) {
            return got.clone();
        }
    }
    let every = _part_forms(part);
    if !meets(&every, &_line_forms(forms, seen)) {
        let none = Rc::new(Vec::new());
        if let Some(s) = seen {
            s.borrow_mut().parts.insert(key, none.clone());
        }
        return none;
    }
    let mut found: Vec<Hit> = Vec::new();
    for alternative in part.alternatives() {
        if text::strip(alternative).is_empty() {
            continue;
        }
        let mut span = _phrase_span(forms, alternative, seen);
        let mut bare_hit = false;
        if span.is_empty() {
            if let Some(bare) = _wanted_bare(alternative) {
                span = _phrase_span_bare(forms, &bare);
                bare_hit = !span.is_empty();
            }
        }
        if !span.is_empty() {
            let (words, mut weight) = _shape(alternative);
            if bare_hit && weight > *BARE_WEIGHT {
                weight = *BARE_WEIGHT;
            }
            found.push(Hit { weight, words, span });
        }
    }
    let order = |h: &Hit| (h.weight, !_only_the_frame(&h.words));
    found.sort_by(|a, b| order(b).cmp(&order(a)));
    let found = Rc::new(found);
    if let Some(s) = seen {
        s.borrow_mut().parts.insert(key, found.clone());
    }
    found
}

/// `_part_match(part, forms, covered=(), seen=None)`: the best way the part is said that is not
/// all inside what is already covered.
pub fn _part_match(part: Part, forms: &FormLine, covered: &[usize], seen: Option<&SeenRef>) -> (i64, Arc<[String]>, Vec<usize>) {
    let found = _part_matches(part, forms, seen);
    if found.is_empty() {
        return (0, Arc::from(Vec::<String>::new()), vec![]);
    }
    for hit in found.iter() {
        if covered.is_empty() || !hit.span.iter().all(|i| covered.contains(i)) {
            return (hit.weight, hit.words.clone(), hit.span.clone());
        }
    }
    let hit = &found[0];
    (hit.weight, hit.words.clone(), hit.span.clone())
}

/// `_part_score(part, forms, seen=None)`.
pub fn _part_score(part: Part, forms: &FormLine, seen: Option<&SeenRef>) -> i64 {
    _part_match(part, forms, &[], seen).0
}

/// `_only_the_frame(words)`: every word is the ask's frame ("how", "what", ...).
pub fn _only_the_frame(words: &[String]) -> bool {
    !words.is_empty() && words.iter().all(|w| ASK_FRAME.contains(w))
}

/// What the matcher works out about one phrase (`_wanted`, `_wanted_bare`, `_the_same_words`,
/// `_shape`), worked out once for every phrase the knowledge base holds.
pub struct PhraseInfo {
    pub wanted: Arc<Vec<Forms>>,
    pub bare: Option<Arc<Vec<Forms>>>,
    pub same: Arc<Vec<String>>,
    pub shape: (Arc<[String]>, i64),
}

/// Every phrase and part the knowledge base and the matcher's own tables hold, by where it lives.
/// They live for the whole run, so an address in the table is that phrase and no other (Python's
/// caches are keyed by the phrase, or by `id(part)` for a part).
fn static_phrases() -> (Vec<&'static str>, Vec<Part<'static>>) {
    let mut phrases: Vec<&'static str> = Vec::new();
    let mut parts: Vec<Part<'static>> = Vec::new();
    for item in all_entries() {
        for part in item.key.iter().chain(item.ask.iter()) {
            parts.push(part.into());
            phrases.extend(alternatives(part).iter().map(String::as_str));
        }
        for word in &item.avoid {
            parts.push(Part::Str(word));
            phrases.push(word);
        }
    }
    for table in [&*WANTING, &*WANT_KNOWING, &*PLAN_WORDS] {
        parts.push(Part::Tuple(table));
        for word in table.iter() {
            parts.push(Part::Str(word));
            phrases.push(word);
        }
    }
    (phrases, parts)
}

static STATIC_PHRASES: LazyLock<HashMap<(usize, usize), PhraseInfo>> = LazyLock::new(|| {
    let (phrases, _) = static_phrases();
    let mut m: HashMap<(usize, usize), PhraseInfo> = HashMap::default();
    for phrase in phrases {
        m.entry((phrase.as_ptr() as usize, phrase.len())).or_insert_with(|| PhraseInfo {
            wanted: cached_wanted(phrase),
            bare: cached_wanted_bare(phrase),
            same: cached_same_words(phrase),
            shape: cached_shape(phrase),
        });
    }
    m
});

static STATIC_PARTS: LazyLock<HashMap<PartKey, Arc<HashSet<String>>>> = LazyLock::new(|| {
    let (_, parts) = static_phrases();
    let mut m: HashMap<PartKey, Arc<HashSet<String>>> = HashMap::default();
    for part in parts {
        m.entry(part.key()).or_insert_with(|| cached_part_forms(part));
    }
    m
});

/// The worked-out phrase, when the phrase is one the knowledge base holds.
fn static_phrase(phrase: &str) -> Option<&'static PhraseInfo> {
    STATIC_PHRASES.get(&(phrase.as_ptr() as usize, phrase.len()))
}
