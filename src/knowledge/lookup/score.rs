//! lookup.py L3036-L4750 and the body-part readers (L6706-L6800): whether an entry's key is
//! said in a line (`score`), how well (`_rank`), and the readers both lean on: the ask frame,
//! reports, denials, a keeping row, the vocabulary and the word index.

use super::*;
use std::rc::Rc;

/// The line's words for the report readers: `typed` when it has one word per position, else
/// each position's first form in sorted order.
fn line_words(forms: &[Forms], typed: Option<&[String]>) -> Vec<String> {
    if let Some(t) = typed {
        if t.len() == forms.len() {
            return t.iter().map(|w| text::lower(w)).collect();
        }
    }
    forms.iter().map(|group| group.iter().map(|f| text::lower(f)).min().unwrap_or_default()).collect()
}

/// `_own_words(item)`: the key's words long enough to be misspelt.
pub fn _own_words(item: &Entry) -> HashSet<String> {
    let mut words = HashSet::default();
    for part in &item.key {
        for key in alternatives(part) {
            for word in text::split(&text::lower(text::strip(key))) {
                if text::len(&word) >= *MIN_TYPO_LENGTH {
                    words.insert(word);
                }
            }
        }
    }
    words
}

/// `_misspelt_avoid(item, typed, own=None)`: the line misspells a word the entry avoids.
pub fn _misspelt_avoid(item: &Entry, typed: &[String], own: Option<&HashSet<String>>) -> bool {
    let owned;
    let own = match own {
        Some(o) => o,
        None => {
            owned = _own_words(item);
            &owned
        }
    };
    for word in &item.avoid {
        let key = text::lower(text::strip(word));
        if text::len(&key) < *MIN_TYPO_LENGTH || key.contains(' ') {
            continue;
        }
        let first = key.chars().next();
        for typed_word in typed {
            if own.contains(typed_word) {
                continue;
            }
            if text::len(typed_word) >= *MIN_TYPO_LENGTH && !_engine_reads(typed_word) && typed_word.chars().next() == first && _one_edit(typed_word, &key) {
                return true;
            }
        }
    }
    false
}

/// `_about_someone_else(phrase)`: an ask about me said about another ("my" -> "his").
pub fn _about_someone_else(phrase: &str) -> Vec<String> {
    let words = text::split(phrase);
    if !words.iter().any(|w| matches!(text::lower(w).as_str(), "i" | "me" | "my")) {
        return vec![];
    }
    let mut out = Vec::new();
    for person in ASK_PERSON_PRONOUNS.iter() {
        out.push(words.iter().map(|w| if matches!(text::lower(w).as_str(), "i" | "me") { person.as_str() } else { w.as_str() }).collect::<Vec<_>>().join(" "));
    }
    for person in ASK_PERSON_POSSESSIVES.iter() {
        out.push(words.iter().map(|w| if text::lower(w) == "my" { person.as_str() } else { w.as_str() }).collect::<Vec<_>>().join(" "));
    }
    out
}

/// `_said_about_someone(ask, forms, seen=None)`: the ask is said about another person.
pub fn _said_about_someone(ask: &[KeyPart], forms: &FormLine, seen: Option<&SeenRef>) -> bool {
    let line = _line_forms(forms, seen);
    if !ASK_PEOPLE.iter().any(|w| line.contains(w)) {
        return false;
    }
    for part in ask {
        let key = Part::from(part).key();
        if let Some(s) = seen {
            let got = s.borrow().about_someone.get(&key).copied();
            if let Some((id, said)) = got {
                if id == forms.id {
                    if said {
                        return true;
                    }
                    continue;
                }
            }
        }
        let mut said = false;
        for phrase in alternatives(part) {
            for variant in _about_someone_else(phrase) {
                if !_phrase_span(forms, &variant, seen).is_empty() {
                    said = true;
                    break;
                }
            }
            if said {
                break;
            }
        }
        if let Some(s) = seen {
            s.borrow_mut().about_someone.insert(key, (forms.id, said));
        }
        if said {
            return true;
        }
    }
    false
}

/// `_ask_about_someone_only(ask, forms, seen=None)`.
pub fn _ask_about_someone_only(ask: &[KeyPart], forms: &FormLine, seen: Option<&SeenRef>) -> bool {
    if ask.is_empty() {
        return false;
    }
    if ask.iter().any(|part| _part_score(part.into(), forms, seen) != 0) {
        return false;
    }
    _said_about_someone(ask, forms, seen)
}

/// `_the_subject_it_avoids(item, forms, carried, seen=None)`: the ask is about another person
/// and the subject carried from before is one the entry avoids.
pub fn _the_subject_it_avoids(item: &Entry, forms: &FormLine, carried: &FormLine, seen: Option<&SeenRef>) -> bool {
    if carried.is_empty() {
        return false;
    }
    if !_ask_about_someone_only(&item.ask, forms, seen) {
        return false;
    }
    let memo = new_seen();
    for word in &item.avoid {
        if !text::strip(word).is_empty() && _part_score(Part::Str(word), carried, Some(&memo)) != 0 {
            return true;
        }
    }
    false
}

/// `_the_subject_the_line_leans_on(understood)`: the forms of the words carried from before.
pub fn _the_subject_the_line_leans_on(understood: &Understood) -> Vec<Forms> {
    let Some(leans) = &understood.leans_on else { return vec![] };
    leans
        .words
        .iter()
        .filter(|w| !ASK_PEOPLE.contains(&text::lower(text::strip(w))))
        .map(|w| _forms(w))
        .take(*CONTEXT_WORDS)
        .collect()
}

/// `_the_ask_said(ask, forms, seen=None)`.
pub fn _the_ask_said(ask: &[KeyPart], forms: &FormLine, seen: Option<&SeenRef>) -> bool {
    if ask.is_empty() {
        return true;
    }
    if ask.iter().any(|part| _part_score(part.into(), forms, seen) != 0) {
        return true;
    }
    _said_about_someone(ask, forms, seen)
}

/// `_wanting(forms, seen=None)`: the line says what the speaker wants to do.
pub fn _wanting(forms: &FormLine, seen: Option<&SeenRef>) -> bool {
    if let Some(s) = seen {
        if let Some((id, answer)) = s.borrow().wanting {
            if id == forms.id {
                return answer;
            }
        }
    }
    let answer = if _part_score(Part::Tuple(&WANT_KNOWING), forms, seen) > 0 { false } else { _part_score(Part::Tuple(&WANTING), forms, seen) > 0 };
    if let Some(s) = seen {
        s.borrow_mut().wanting = Some((forms.id, answer));
    }
    answer
}

/// `_only_the_want(item, forms, seen=None)`.
pub fn _only_the_want(item: &Entry, forms: &FormLine, seen: Option<&SeenRef>) -> bool {
    if item.ask.is_empty() {
        false
    } else {
        !_the_ask_said(&item.ask, forms, seen)
    }
}

/// `_key_on_the_want(item, forms, seen=None)`: the key itself is the want ("want to").
pub fn _key_on_the_want(item: &Entry, forms: &FormLine, seen: Option<&SeenRef>) -> bool {
    for part in &item.key {
        for alternative in alternatives(part) {
            if text::split(&text::lower(alternative)).iter().any(|w| WANT_FILLER.contains(w)) && _part_score(Part::Str(alternative), forms, seen) != 0 {
                return true;
            }
        }
    }
    false
}

/// `_denier_inside(covered, forms, index)`.
pub fn _denier_inside(covered: &[usize], forms: &[Forms], index: usize) -> bool {
    let mut back = 2;
    while back <= 4 && index >= back {
        let at = index - back;
        if !covered.contains(&at) {
            return false;
        }
        if meets(&forms[at], &DENIERS) {
            return true;
        }
        back += 1;
    }
    false
}

/// `_denied(forms, covered)`: the key is said only to be denied ("it is not swollen").
pub fn _denied(forms: &[Forms], covered: &[usize]) -> bool {
    for &index in covered {
        if index >= 1 && covered.contains(&(index - 1)) && meets(&forms[index - 1], &DENIERS) {
            continue;
        }
        if _denier_inside(covered, forms, index) {
            continue;
        }
        let mut back = 1;
        while back <= 4 && index >= back {
            let before = &forms[index - back];
            if meets(before, &DENIERS) {
                let subject = index as i64 - back as i64 - 1;
                if subject < 0 || !meets(&forms[subject as usize], &DENY_SAID_OF) {
                    break;
                }
                let between = &forms[index - back + 1..index];
                if between.iter().any(|w| meets(w, &STATES_AFTER)) && between.iter().all(|w| meets(w, &STATES_AFTER) || meets(w, &DENY_BETWEEN)) {
                    return true;
                }
                break;
            }
            if !meets(before, &DENY_BETWEEN) && !meets(before, &STATES_AFTER) {
                break;
            }
            back += 1;
        }
    }
    false
}

/// `_says_removal(forms)`: the line says "off" or "out of".
pub fn _says_removal(forms: &[Forms]) -> bool {
    static TAKE_OFF: LazyLock<(Forms, Forms, Forms)> = LazyLock::new(|| (_forms("off"), _forms("out"), _forms("of")));
    let (off, out, of) = &*TAKE_OFF;
    for (index, word) in forms.iter().enumerate() {
        if word.is_empty() {
            continue;
        }
        if meets(word, off) {
            return true;
        }
        if !out.is_empty() && !of.is_empty() && index + 1 < forms.len() && meets(word, out) && meets(&forms[index + 1], of) {
            return true;
        }
    }
    false
}

/// `_frame_pronoun_forms()`.
pub fn _frame_pronoun_forms() -> &'static HashSet<String> {
    static GOT: LazyLock<HashSet<String>> = LazyLock::new(|| FRAME_PRONOUNS.iter().flat_map(|w| _forms(w).iter().cloned().collect::<Vec<_>>()).collect());
    &GOT
}

/// `_the_subjects_own_clause(here)`: the words after a frame start a clause of their own.
pub fn _the_subjects_own_clause(here: &[Forms]) -> bool {
    if here.is_empty() || here[0].is_empty() {
        return false;
    }
    let mine = _frame_pronoun_forms();
    if meets(&here[0], mine) {
        return true;
    }
    if !here[0].iter().any(|f| f.ends_with("ing")) {
        return false;
    }
    here[1..here.len().min(3)].iter().any(|w| !w.is_empty() && meets(w, mine))
}

/// Whether the frame phrase `words` is said at `index`.
fn frame_said_at(forms: &[Forms], index: usize, words: &[String]) -> bool {
    let span = &forms[index..(index + words.len()).min(forms.len())];
    span.len() == words.len() && span.iter().zip(words).all(|(word, wanted)| !word.is_empty() && meets(word, &_forms(wanted)))
}

/// `_a_clause_about_the_thing(forms)`: a frame followed by a clause of its own.
pub fn _a_clause_about_the_thing(forms: &[Forms]) -> bool {
    let (starts, _end_forms) = _frame_starts();
    for (index, here) in forms.iter().enumerate() {
        if here.is_empty() {
            continue;
        }
        for (first_forms, words) in starts {
            if !meets(here, first_forms) {
                continue;
            }
            if frame_said_at(forms, index, words) {
                let after = index + words.len();
                let tail = &forms[after.min(forms.len())..(after + 3).min(forms.len())];
                if _the_subjects_own_clause(tail) {
                    return true;
                }
            }
        }
    }
    false
}

/// `_frame_starts()`: each frame phrase with its first word's forms, and the forms that end a
/// frame.
pub fn _frame_starts() -> &'static (Vec<(Forms, Vec<String>)>, HashSet<String>) {
    static GOT: LazyLock<(Vec<(Forms, Vec<String>)>, HashSet<String>)> = LazyLock::new(|| {
        let starts = FRAME_WORDS
            .iter()
            .map(|phrase| {
                let words = text::split(phrase);
                (_forms(&words[0]), words)
            })
            .collect();
        let mut end_forms = HashSet::default();
        for word in FRAME_ENDS.iter() {
            end_forms.extend(_forms(word).iter().cloned());
        }
        (starts, end_forms)
    });
    &GOT
}

/// `_frame_masked(forms)`: the line with the words a frame wraps blanked out, and the frames'
/// first words. None when no frame is said (Python returns the same list).
pub fn _frame_masked(forms: &[Forms]) -> (Option<Vec<Forms>>, HashSet<String>) {
    let (starts, end_forms) = _frame_starts();
    let mut found = HashSet::default();
    let mut dead = HashSet::default();
    let mut index = 0;
    while index < forms.len() {
        let here = &forms[index];
        let mut hit: Option<&String> = None;
        if !here.is_empty() {
            for (first_forms, words) in starts {
                if !meets(here, first_forms) {
                    continue;
                }
                if frame_said_at(forms, index, words) {
                    hit = Some(&words[0]);
                    index += words.len() - 1;
                    break;
                }
            }
        }
        let Some(hit) = hit else {
            index += 1;
            continue;
        };
        found.insert(hit.clone());
        index += 1;
        while index < forms.len() {
            if !forms[index].is_empty() && meets(&forms[index], end_forms) {
                break;
            }
            dead.insert(index);
            index += 1;
        }
    }
    if found.is_empty() {
        return (None, found);
    }
    (Some(forms.iter().enumerate().map(|(at, w)| if dead.contains(&at) { no_forms() } else { w.clone() }).collect()), found)
}

/// `_key_names(frame_words, item)`: the key itself says a frame word.
pub fn _key_names(frame_words: &HashSet<String>, item: &Entry) -> bool {
    item.key.iter().any(|part| alternatives(part).iter().any(|a| text::split(&text::lower(a)).iter().any(|w| frame_words.contains(w))))
}

/// `_unframed(forms, seen)`: (the masked line, its memo, the frame words); the line is None
/// when nothing was masked (Python's `masked is forms`, whose memo is `seen` itself).
pub fn _unframed(forms: &FormLine, seen: Option<&SeenRef>) -> (Option<Rc<FormLine>>, Option<SeenRef>, Arc<HashSet<String>>) {
    if let Some(s) = seen {
        if let Some((id, masked, words)) = &s.borrow().frame_mask {
            if *id == forms.id {
                return match masked {
                    Some((line, memo)) => (Some(line.clone()), Some(memo.clone()), words.clone()),
                    None => (None, Some(s.clone()), words.clone()),
                };
            }
        }
    }
    let (masked, frame_words) = _frame_masked(forms);
    let frame_words = Arc::new(frame_words);
    let masked = masked.map(|m| (Rc::new(FormLine::new(m)), new_seen()));
    if let Some(s) = seen {
        s.borrow_mut().frame_mask = Some((forms.id, masked.clone(), frame_words.clone()));
    }
    match masked {
        Some((line, memo)) => (Some(line), Some(memo), frame_words),
        None => (None, seen.cloned(), frame_words),
    }
}

/// `_without_ask_phrase(forms, seen)`: the line without "best way" / "how best", with its own
/// memo; None when the line does not say one (Python returns `forms, seen`).
pub fn _without_ask_phrase(forms: &FormLine, seen: Option<&SeenRef>) -> (Option<Rc<FormLine>>, Option<SeenRef>) {
    let mut span = vec![];
    for phrase in ["best way", "best ways", "how best"] {
        span = _phrase_span(forms, phrase, seen);
        if !span.is_empty() {
            break;
        }
    }
    if span.is_empty() {
        return (None, seen.cloned());
    }
    if let Some(s) = seen {
        if let Some((id, masked, memo)) = &s.borrow().ask_phrase_mask {
            if *id == forms.id {
                return (Some(masked.clone()), Some(memo.clone()));
            }
        }
    }
    let mut masked: Vec<Forms> = forms.forms.clone();
    for at in span {
        masked[at] = no_forms();
    }
    let masked = Rc::new(FormLine::new(masked));
    match seen {
        Some(s) => {
            let memo = new_seen();
            s.borrow_mut().ask_phrase_mask = Some((forms.id, masked.clone(), memo.clone()));
            (Some(masked), Some(memo))
        }
        None => (Some(masked), None),
    }
}

/// `_said_again(forms, covered, typed=None)`: the key is followed only by "again" words.
pub fn _said_again(forms: &[Forms], covered: &[usize], typed: Option<&[String]>) -> bool {
    if covered.is_empty() {
        return false;
    }
    let words = line_words(forms, typed);
    let last = *covered.iter().max().unwrap();
    let after = words.get(last + 1..).unwrap_or(&[]);
    if after.is_empty() {
        return false;
    }
    after.iter().all(|w| HAPPENING_AGAIN.contains(w) || REPORT_FILLER.contains(w))
}

/// `_bare_report(forms, covered, typed=None)`: the line only reports the thing ("i have a
/// rash").
pub fn _bare_report(forms: &[Forms], covered: &[usize], typed: Option<&[String]>) -> bool {
    if covered.is_empty() {
        return false;
    }
    let words = line_words(forms, typed);
    let first = *covered.iter().min().unwrap();
    let mut lead = false;
    let mut person = false;
    for word in &words[..first] {
        if REPORT_LEADS.contains(word) {
            lead = true;
            continue;
        }
        if REPORT_PERSON.contains(word) {
            person = true;
            continue;
        }
        if REPORT_FILLER.contains(word) {
            continue;
        }
        return false;
    }
    if person && !lead {
        return false;
    }
    lead
}

/// `_asks_to_explain(forms, start, typed=None)`: "explain ..." / "what is ..." before the key.
pub fn _asks_to_explain(forms: &[Forms], start: usize, typed: Option<&[String]>) -> bool {
    if start == 0 || start > 12 {
        return false;
    }
    let words = line_words(forms, typed);
    let mut lead = false;
    for word in &words[..start] {
        if EXPLAIN_LEADS.contains(word) {
            lead = true;
            continue;
        }
        if EXPLAIN_FILLER.contains(word) {
            continue;
        }
        return false;
    }
    lead
}

/// `_only_as_a_kind(forms, span, covered, item, typed=None)`: the key word is said only as a
/// kind of something else ("a type of ...").
pub fn _only_as_a_kind(forms: &[Forms], span: &[usize], covered: &[usize], item: &Entry, typed: Option<&[String]>) -> bool {
    if span.len() != 1 {
        return false;
    }
    let start = span[0];
    if start < 1 {
        return false;
    }
    let words = line_words(forms, typed);
    if start - 1 >= words.len() {
        return false;
    }
    if !AS_A_KIND.contains(&words[start - 1]) {
        return false;
    }
    let mut asked = HashSet::default();
    for part in &item.ask {
        for alternative in alternatives(part) {
            for word in text::split(&text::lower(alternative)) {
                asked.extend(_forms(&word).iter().map(|f| text::lower(f)));
                asked.insert(word);
            }
        }
    }
    let mut said = HashSet::default();
    for &index in covered {
        if span.contains(&index) || index >= words.len() {
            continue;
        }
        said.insert(words[index].clone());
    }
    said.remove("");
    said.iter().all(|w| asked.contains(w) || EXPLAIN_FILLER.contains(w))
}

/// `_says_a_shape(shapes, typed, forms)`: one of the entry's figure readers fits the line.
pub fn _says_a_shape(shapes: &[crate::data::FnRef], typed: Option<&[String]>, forms: &[Forms]) -> bool {
    let words = line_words(forms, typed);
    let line = words.iter().filter(|w| !w.is_empty()).cloned().collect::<Vec<_>>().join(" ");
    if line.is_empty() {
        return false;
    }
    shapes.iter().any(|shape| crate::knowledge::registry::shape(shape, &line))
}

/// `score(item, forms, typed=None, seen=None, frame=True, framed=False)`: how much of the
/// entry's key the line says, or 0 when it does not answer the line.
pub fn score(item: &Entry, forms: &FormLine, typed: Option<&[String]>, seen: Option<&SeenRef>, frame: bool, framed: bool) -> i64 {
    if forms.is_empty() {
        return 0;
    }
    let typed_truthy = typed.is_some_and(|t| !t.is_empty());
    if typed_truthy && _under_their_own_hands(item, typed.unwrap()) {
        return 0;
    }
    let (masked, memo, frame_words) = if frame { _unframed(forms, seen) } else { (None, seen.cloned(), Arc::new(HashSet::default())) };
    let (forms, seen): (&FormLine, Option<SeenRef>) =
        if !frame_words.is_empty() && !_key_names(&frame_words, item) { (masked.as_deref().unwrap_or(forms), memo) } else { (forms, seen.cloned()) };
    let seen = seen.as_ref();
    let framed_one =
        framed && !item.key.is_empty() && ASK_FRAME_IDS.contains(&item.id) && _the_whole_subject((&item.key[0]).into(), forms, seen);
    if !item.report && !item.explain && !framed_one && item.shape.is_empty() && !_two_positions(forms, item) {
        return 0;
    }
    if !item.ask.is_empty() && !_the_ask_said(&item.ask, forms, seen) && (!_wanting(forms, seen) || _key_on_the_want(item, forms, seen)) {
        return 0;
    }
    let mut total = 0;
    let mut covered: Vec<usize> = Vec::new();
    let mut first: Vec<usize> = Vec::new();
    let removal = _says_removal(forms);
    let grouped = item.key.iter().any(|p| matches!(p, KeyPart::Any(_)));
    for (index, part) in item.key.iter().enumerate() {
        let part: Part = part.into();
        let (mut weight, mut words, mut span) = _part_match(part, forms, &covered, seen);
        if index == 0 && typed_truthy && span.len() == 1 && span[0] < typed.unwrap().len() && _asking_with(typed.unwrap(), span[0]) {
            let mut bare = forms.forms.clone();
            bare[span[0]] = no_forms();
            let (w, _, s) = _part_match(part, &FormLine::new(bare), &covered, None);
            weight = w;
            span = s;
        } else if index > 0 && removal && weight > 1 && _only_the_frame(&words) && first.len() < 2 {
            let all = _part_matches(part, forms, None);
            let spare: Vec<&Hit> = all.iter().filter(|h| !_only_the_frame(&h.words)).collect();
            if let Some(hit) = spare.first() {
                weight = hit.weight;
                words = hit.words.clone();
                span = hit.span.clone();
            } else {
                weight = if words.len() == 1 { 1 } else { 0 };
            }
        }
        let _ = &words;
        total += weight;
        covered.extend(span.iter().copied());
        if index == 0 {
            first = span;
        }
    }
    if grouped && first.is_empty() {
        return 0;
    }
    let bare = item.report && (_bare_report(forms, &covered, typed) || _said_again(forms, &covered, typed));
    let shaped = !item.shape.is_empty() && !first.is_empty() && _says_a_shape(&item.shape, typed, forms);
    let explained = item.explain && !first.is_empty() && _asks_to_explain(forms, *first.iter().min().unwrap(), typed);
    if !first.is_empty() && !explained && _only_as_a_kind(forms, &first, &covered, item, typed) {
        return 0;
    }
    let one_word = covered.iter().collect::<HashSet<_>>().len() < 2;
    if one_word && !bare && !explained && !framed_one && !shaped {
        return 0;
    }
    if bare || shaped || explained {
        total += *WORD_WEIGHT;
    }
    if _denied(forms, &covered) {
        return 0;
    }
    if !one_word && total < *MIN_SCORE {
        return 0;
    }
    let parts_said = if typed_truthy { Some(_body_parts(typed.unwrap())) } else { None };
    let mut ask_line: Option<(Option<Rc<FormLine>>, Option<SeenRef>)> = None;
    for word in &item.avoid {
        let key = text::lower(text::strip(word));
        if ASK_PHRASE_WORDS.contains(&key) {
            if ask_line.is_none() {
                ask_line = Some(_without_ask_phrase(forms, seen));
            }
            let (ask_forms, ask_seen) = ask_line.as_ref().unwrap();
            if _part_score(Part::Str(word), ask_forms.as_deref().unwrap_or(forms), ask_seen.as_ref()) != 0 {
                return 0;
            }
        } else if (_part_score(Part::Str(word), forms, seen) != 0 || !_phrase_span_between(forms, word, seen).is_empty())
            && !_a_part_the_line_does_not_have(word, parts_said.as_ref())
        {
            return 0;
        }
    }
    if typed_truthy && _misspelt_avoid(item, typed.unwrap(), None) {
        return 0;
    }
    if let Some(failing_at) = _keeps_failing_at(forms, seen) {
        if _keeping_row(item) && (first.is_empty() || *first.iter().min().unwrap() != failing_at + 1) {
            return 0;
        }
    }
    total
}

/// `_under_their_own_hands(item, typed)`: the line asks how to do it by hand or by eye, and the
/// entry is about a thing on their side.
pub fn _under_their_own_hands(item: &Entry, typed: &[String]) -> bool {
    let line = typed.iter().map(|w| text::lower(w)).collect::<Vec<_>>().join(" ");
    if !ASKS_FOR_THE_HANDS.is_match(&line) || !A_HAND_OR_EYE_DOES_IT.is_match(&line) {
        return false;
    }
    if ASKS_WHAT_IT_SAYS.is_match(&line) {
        return false;
    }
    if !OWN_THING_ON_THEIR_SIDE.is_match(&line) {
        return false;
    }
    _key_words(item).iter().any(|w| THEIR_OWN_HANDS_THINGS.contains(w))
}

/// `_keeping_row(item)`: the key says keeping and a life ("how long does it keep").
pub fn _keeping_row(item: &Entry) -> bool {
    static CACHE: LazyLock<RwLock<HashMap<String, bool>>> = LazyLock::new(|| RwLock::new(HashMap::default()));
    if let Some(known) = CACHE.read().unwrap().get(&item.id) {
        return *known;
    }
    let mut keeps = false;
    let mut lives = false;
    for part in &item.key {
        for alternative in alternatives(part) {
            let shaped = text::lower(text::strip(alternative));
            if _KEEPING_KEY.contains(&shaped) {
                keeps = true;
            } else if _LIFE_KEY.contains(&shaped) {
                lives = true;
            }
        }
    }
    let known = keeps && lives;
    if item.index != usize::MAX {
        CACHE.write().unwrap().insert(item.id.clone(), known);
    }
    known
}

/// `_gerund(forms, index)`: the -ing form at a position. Python walks the frozenset in its own
/// order; the forms are walked sorted here (a position rarely has two -ing forms).
pub fn _gerund(forms: &[Forms], index: usize) -> String {
    let mut all: Vec<&String> = forms[index].iter().collect();
    all.sort();
    for form in all {
        if _ING_NOUNS.contains(form) || text::len(form) < 5 || !form.ends_with("ing") {
            continue;
        }
        return form.clone();
    }
    String::new()
}

/// `_later_words(forms, index, reach=3)`.
pub fn _later_words(forms: &[Forms], index: usize, reach: usize) -> HashSet<String> {
    let mut seen = HashSet::default();
    for step in index..(index + reach).min(forms.len()) {
        seen.extend(forms[step].iter().cloned());
    }
    seen
}

/// `_keeps_failing_at(forms, seen=None)`: where "keeps <failing>" is said, if it is.
pub fn _keeps_failing_at(forms: &[Forms], seen: Option<&SeenRef>) -> Option<usize> {
    if let Some(s) = seen {
        if let Some(got) = s.borrow().keeps_failing_at {
            return got;
        }
    }
    let mut failing = None;
    for index in 0..forms.len().saturating_sub(1) {
        if !meets(&forms[index], &_KEEPING_VERB) {
            continue;
        }
        let gerund = _gerund(forms, index + 1);
        if gerund.is_empty() {
            continue;
        }
        if _RECURRING.contains(&gerund) {
            continue;
        }
        let later = _later_words(forms, index + 1, 3);
        if _RECURRING_ANYWAY.contains(&gerund) && meets(&later, &_RECURRING_WITH) {
            continue;
        }
        let result = later.contains("out") && (meets(&later, &_RESULT_VERB) || _RESULT_VERB.contains(&gerund));
        let spoils = meets(&later, &_SPOILING) || _SPOILING.contains(&gerund);
        if result || !spoils {
            failing = Some(index);
            break;
        }
    }
    if let Some(s) = seen {
        s.borrow_mut().keeps_failing_at = Some(failing);
    }
    failing
}

/// `_keeps_failing(forms, seen=None)`.
pub fn _keeps_failing(forms: &[Forms], seen: Option<&SeenRef>) -> bool {
    _keeps_failing_at(forms, seen).is_some()
}

/// `_the_whole_subject(part, forms, seen=None)`: the part covers every word that is not a
/// pointer.
pub fn _the_whole_subject(part: Part, forms: &FormLine, seen: Option<&SeenRef>) -> bool {
    let mut span = HashSet::default();
    for hit in _part_matches(part, forms, seen).iter() {
        span.extend(hit.span.iter().copied());
    }
    let wanted: Vec<usize> = forms.iter().enumerate().filter(|(_, w)| !meets(w, &_SUBJECT_POINTS)).map(|(i, _)| i).collect();
    !wanted.is_empty() && wanted.iter().all(|i| span.contains(i))
}

/// `_rarity(entries)`: how many entries use each key word.
pub fn _rarity(entries: &[&Entry]) -> HashMap<String, usize> {
    let mut counts = HashMap::default();
    for item in entries {
        for word in _key_words(item) {
            *counts.entry(word).or_default() += 1;
        }
    }
    counts
}

/// `_the_tail_asks_about(forms)`: where a late ask word starts the line's tail, or -1.
pub fn _the_tail_asks_about(forms: &[Forms]) -> i64 {
    static ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
        ["what", "whats", "which", "how", "where", "when", "who", "why", "should", "shall", "could", "would", "can"]
            .iter()
            .flat_map(|w| _forms(w).iter().cloned().collect::<Vec<_>>())
            .collect()
    });
    let mut index = forms.len() as i64 - 1;
    while index > 3 {
        let word = &forms[index as usize];
        if !word.is_empty() && meets(word, &ASK_WORDS) {
            return index;
        }
        index -= 1;
    }
    -1
}

/// `_rank(item, forms, rarity, seen=None)`: how well the entry fits, rare key words counting
/// extra; 1 when it says nothing but the ask's frame.
pub fn _rank(item: &Entry, forms: &FormLine, rarity: &HashMap<String, usize>, seen: Option<&SeenRef>) -> i64 {
    let tail_from = _the_tail_asks_about(forms);
    let in_tail = |words: &[String], span: &[usize]| tail_from >= 0 && *span.iter().min().unwrap() as i64 >= tail_from && _only_the_frame(words);
    let matched: Vec<(i64, Arc<[String]>, Vec<usize>)> = item.key.iter().map(|p| _part_match(p.into(), forms, &[], seen)).collect();
    let said: i64 = matched.iter().filter(|(w, words, span)| *w != 0 && !in_tail(words, span)).map(|(w, _, _)| *w).sum();
    let mut total = 0;
    let mut says_what = false;
    for part in &item.key {
        let (weight, words, span) = _part_match(part.into(), forms, &[], seen);
        if weight != 0 && said > 2 && in_tail(&words, &span) {
            continue;
        }
        total += weight + words.iter().filter(|w| rarity.get(*w).copied().unwrap_or(0) == 1).count() as i64;
        if words.iter().any(|w| !ASK_FRAME.contains(w) && !WHEN_WORDS.contains(w) && !A_LENGTH_WORD.contains(w) && !A_STAND_IN.contains(w)) {
            says_what = true;
        }
    }
    if !says_what {
        return 1;
    }
    total + item.bias
}

/// `_key_words(item)`: every word of the key.
pub fn _key_words(item: &Entry) -> HashSet<String> {
    let mut words = HashSet::default();
    for part in &item.key {
        for alternative in alternatives(part) {
            words.extend(text::split(&text::lower(alternative)));
        }
    }
    words
}

/// `_key_forms(item)`: every form of every key word.
pub fn _key_forms(item: &Entry) -> Arc<HashSet<String>> {
    static EVERY: LazyLock<Vec<Arc<HashSet<String>>>> = LazyLock::new(|| all_entries().iter().map(work_out_key_forms).collect());
    let canonical = item.index != usize::MAX && all_entries().get(item.index).is_some_and(|e| std::ptr::eq(e, item));
    if canonical {
        return EVERY[item.index].clone();
    }
    work_out_key_forms(item)
}

fn work_out_key_forms(item: &Entry) -> Arc<HashSet<String>> {
    let mut forms = HashSet::default();
    for word in _key_words(item) {
        forms.extend(_forms(&word).iter().cloned());
    }
    Arc::new(forms)
}

/// `_two_positions(forms, item)`: key words are said at two positions at least.
pub fn _two_positions(forms: &[Forms], item: &Entry) -> bool {
    let key = _key_forms(item);
    let mut found = false;
    for word_forms in forms {
        if meets(word_forms, &key) {
            if found {
                return true;
            }
            found = true;
        }
    }
    false
}

/// Whether `entries` is the whole knowledge base (the list Python's caches mostly hold).
fn is_every_entry(entries: &[&'static Entry]) -> bool {
    let all = all_entry_refs();
    std::ptr::eq(entries.as_ptr(), all.as_ptr()) && entries.len() == all.len()
}

/// `_word_index(entries)`: {form: the positions of the entries whose key says it}.
pub fn _word_index(entries: &[&'static Entry]) -> Arc<HashMap<String, Vec<usize>>> {
    static EVERY: LazyLock<Arc<HashMap<String, Vec<usize>>>> = LazyLock::new(|| Arc::new(work_out_word_index(all_entry_refs())));
    if is_every_entry(entries) {
        return EVERY.clone();
    }
    Arc::new(work_out_word_index(entries))
}

fn work_out_word_index(entries: &[&Entry]) -> HashMap<String, Vec<usize>> {
    let mut index: HashMap<String, Vec<usize>> = HashMap::default();
    for (position, item) in entries.iter().enumerate() {
        for word in _key_words(item) {
            for form in _forms(&word).iter() {
                let at = index.entry(form.clone()).or_default();
                if at.last() != Some(&position) {
                    at.push(position);
                }
            }
        }
    }
    index
}

/// `_reaching(entries, forms)`: the entries a form of the line reaches (all when none does), in
/// their order.
pub fn _reaching(entries: &[&'static Entry], forms: &[Forms]) -> Vec<&'static Entry> {
    let index = _word_index(entries);
    let mut hit: HashSet<usize> = HashSet::default();
    for word_forms in forms {
        for form in word_forms.iter() {
            if let Some(positions) = index.get(form) {
                hit.extend(positions.iter().copied());
            }
        }
    }
    if hit.is_empty() {
        return entries.to_vec();
    }
    let mut at: Vec<usize> = hit.into_iter().collect();
    at.sort();
    at.into_iter().map(|p| entries[p]).collect()
}

/// `_domain_rarity(entries)`: `(by_domain, every)`, how many entries use each key word.
pub fn _domain_rarity(entries: &[&'static Entry]) -> (HashMap<String, HashMap<String, usize>>, HashMap<String, usize>) {
    let mut by_domain: HashMap<String, HashMap<String, usize>> = HashMap::default();
    let mut owners: HashMap<String, HashSet<&str>> = HashMap::default();
    for item in entries {
        let counts = by_domain.entry(item.domain.clone()).or_default();
        for word in _key_words(item) {
            *counts.entry(word.clone()).or_default() += 1;
            owners.entry(word).or_default().insert(item.id.as_str());
        }
    }
    let mut every = HashMap::default();
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

/// What `_vocabulary(entries)` works out: each domain's key words with how many entries use
/// them, the same over every domain, and every form of those words.
#[derive(Clone, Debug, Default)]
pub struct Vocabulary {
    pub by_domain: HashMap<String, HashMap<String, usize>>,
    pub every: HashMap<String, usize>,
    pub known: HashSet<String>,
}

/// `_vocabulary(entries)`: `(by_domain, every, known)`, worked out once for the whole knowledge
/// base (Python caches by the list's identity).
pub fn _vocabulary(entries: &[&'static Entry]) -> Arc<Vocabulary> {
    static EVERY: LazyLock<Arc<Vocabulary>> = LazyLock::new(|| Arc::new(work_out_vocabulary(all_entry_refs())));
    if is_every_entry(entries) {
        return EVERY.clone();
    }
    Arc::new(work_out_vocabulary(entries))
}

fn work_out_vocabulary(entries: &[&'static Entry]) -> Vocabulary {
    let (by_domain, every) = _domain_rarity(entries);
    let slips: HashSet<String> = entries.iter().flat_map(|item| item.slips.iter().map(|s| text::lower(s))).collect();
    let every: HashMap<String, usize> = every.into_iter().filter(|(word, _)| !slips.contains(&text::lower(word))).collect();
    let mut known = HashSet::default();
    for word in every.keys() {
        known.extend(_forms(word).iter().cloned());
    }
    Vocabulary { by_domain, every, known }
}

/// `_inside(part, other)`: a body part is (inside) the other ("heel" in "foot").
pub fn _inside(part: &str, other: &str) -> bool {
    let mut part = part.to_string();
    let mut seen = 0;
    while !part.is_empty() && seen < 4 {
        if meets(&_forms(&part), &_forms(other)) {
            return true;
        }
        part = PART_INSIDE.get(&part).cloned().unwrap_or_default();
        seen += 1;
    }
    false
}

/// `_a_cut_of_meat(words, index)`: the body word is a cut ("leg of lamb").
pub fn _a_cut_of_meat(words: &[String], index: usize) -> bool {
    let i = index as i64;
    [i - 2, i - 1, i + 1, i + 2]
        .iter()
        .filter(|&&at| at >= 0 && (at as usize) < words.len())
        .any(|&at| _CUT_BESIDE.contains(&text::lower(&words[at as usize])))
}

/// `_body_parts(words)`: the particular body parts the words name.
pub fn _body_parts(words: &[String]) -> HashSet<String> {
    let mut parts = HashSet::default();
    for (index, word) in words.iter().enumerate() {
        let word = text::lower(word);
        if !BODY_WORDS.contains(&word) {
            continue;
        }
        if word == "back" && index > 0 && RETURNED_AFTER.contains(&text::lower(&words[index - 1])) {
            continue;
        }
        if GENERAL_BODY.contains(&word) {
            continue;
        }
        if _a_cut_of_meat(words, index) {
            continue;
        }
        parts.insert(word);
    }
    parts
}

/// `_key_body_parts(item)`.
pub fn _key_body_parts(item: &Entry) -> HashSet<String> {
    let mut parts = HashSet::default();
    for part in &item.key {
        for alternative in alternatives(part) {
            parts.extend(_body_parts(&text::split(&text::lower(alternative))));
        }
    }
    parts
}

/// `_a_part_the_line_does_not_have(word, parts_said)`: the avoided word is a body part the line
/// does not name.
pub fn _a_part_the_line_does_not_have(word: &str, parts_said: Option<&HashSet<String>>) -> bool {
    let key = text::lower(text::strip(word));
    if key.is_empty() || key.contains(' ') || !BODY_WORDS.contains(&key) {
        return false;
    }
    !parts_said.is_some_and(|p| p.contains(&key))
}

/// `_another_part(item, line_parts)`: the entry is about a body part the line does not name.
pub fn _another_part(item: &Entry, line_parts: &HashSet<String>) -> bool {
    if line_parts.is_empty() {
        return false;
    }
    let parts = _key_body_parts(item);
    if parts.is_empty() || parts.len() > *SPECIFIC_PARTS {
        return false;
    }
    !parts.iter().any(|part| line_parts.iter().any(|other| _inside(other, part)))
}
