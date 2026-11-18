//! Port of `bot/knowledge/whenbuy.py`: the calendar of when things are cheapest. The rows are
//! data (`WHEN_TO_BUY`, built into `ENTRIES` by `_whenbuy`); this is the reading that decides
//! whether a line really asks the calendar question.

use crate::data;
use crate::data::FnRef;
use crate::knowledge::d2_shared::asks_where_a_shop_answers;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.whenbuy.{name}"))
}

static TIME_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("TIME_ASK"));
static TIME_ASK_SHAPE: LazyLock<Vec<&'static Re>> = LazyLock::new(|| {
    data::get("bot.knowledge.whenbuy.TIME_ASK_SHAPE").items().iter().map(|v| v.as_re().unwrap()).collect()
});
static WHEN_ASKING: LazyLock<Vec<String>> = LazyLock::new(|| strs("WHEN_ASKING"));
static PERIOD_NOUNS: LazyLock<Vec<String>> = LazyLock::new(|| strs("PERIOD_NOUNS"));
static PERIOD_PREPOSITION: LazyLock<Vec<String>> = LazyLock::new(|| strs("PERIOD_PREPOSITION"));
static COMPARING: LazyLock<Vec<String>> = LazyLock::new(|| strs("COMPARING"));
static NOT_THE_CALENDAR: LazyLock<Vec<String>> = LazyLock::new(|| strs("NOT_THE_CALENDAR"));
static _HAS_FAILED: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HAS_FAILED"));
static _BUY_THE_NEXT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BUY_THE_NEXT"));
static SALES_CALENDAR: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.whenbuy.SALES_CALENDAR"));

fn any_in(line: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|p| line.contains(p.as_str()))
}

/// True when a period named after a preposition is being compared: "cheaper in the summer months".
pub fn _a_period_beside_a_comparison(line: &str) -> bool {
    let words: Vec<&str> = crate::py::text::split_ref(line);
    if !words.iter().any(|w| COMPARING.iter().any(|c| c == w))
        && !["half price", "price drop", "prices drop"].iter().any(|p| line.contains(p))
    {
        return false;
    }
    for (at, word) in words.iter().enumerate() {
        if !PERIOD_NOUNS.iter().any(|p| p == word) {
            continue;
        }
        if words[at.saturating_sub(3)..at].iter().any(|b| PERIOD_PREPOSITION.iter().any(|p| p == b)) {
            return true;
        }
    }
    false
}

/// The line as words, from the reading, so a row reads the same text the matcher did.
pub fn _line(understood: &Understood) -> String {
    let mut text = if understood.clean.is_empty() { understood.norm_text.clone() } else { understood.clean.clone() };
    if text.is_empty() && !understood.words.is_empty() {
        text = understood.words.join(" ");
    }
    format!(" {} ", text.to_lowercase())
}

/// `re.search(r"(?<![a-z])" + re.escape(form) + r"(?![a-z])", line)`.
fn bounded(line: &str, form: &str) -> bool {
    if form.is_empty() {
        // an empty form matches wherever a non-letter is not on either side
        let b = line.as_bytes();
        return (0..=b.len()).any(|i| {
            (i == 0 || !b[i - 1].is_ascii_lowercase()) && (i == b.len() || !b[i].is_ascii_lowercase())
        });
    }
    let mut from = 0;
    while let Some(i) = line[from..].find(form) {
        let s = from + i;
        let e = s + form.len();
        let before = line[..s].chars().next_back();
        let after = line[e..].chars().next();
        if !before.is_some_and(|c| c.is_ascii_lowercase()) && !after.is_some_and(|c| c.is_ascii_lowercase()) {
            return true;
        }
        from = s + line[s..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

/// True when the line has this word itself, not just a word the matcher's families cover.
pub fn _says_it(line: &str, name: &str) -> bool {
    let mut forms = vec![name.to_string(), format!("{name}s"), format!("{name}es")];
    if name.ends_with('s') {
        forms.push(name[..name.len() - 1].to_string());
    }
    if forms.iter().any(|f| bounded(line, f)) {
        return true;
    }
    if name.contains(' ') || name.chars().count() < 6 {
        return false;
    }
    for word in crate::py::text::split_ref(line) {
        let word = crate::py::text::strip_chars(word, ".,!?;:'\"()");
        if word.chars().count() >= 5 && _one_edit_apart(word, name) {
            return true;
        }
    }
    false
}

/// True when `typed` is `name` with one letter added, dropped or swapped.
pub fn _one_edit_apart(typed: &str, name: &str) -> bool {
    let t: Vec<char> = typed.chars().collect();
    let n: Vec<char> = name.chars().collect();
    if t.len().abs_diff(n.len()) > 1 {
        return false;
    }
    if t.len() == n.len() {
        return t.iter().zip(n.iter()).filter(|(a, b)| a != b).count() <= 1;
    }
    let (short, long) = if t.len() < n.len() { (&t, &n) } else { (&n, &t) };
    (0..long.len()).any(|i| long[..i].iter().chain(long[i + 1..].iter()).eq(short.iter()))
}

/// True when the line asks when, and is not a how-long/repair/care question wearing a when.
pub fn _asks_the_calendar(understood: &Understood, own: &[String]) -> bool {
    let mut line = _line(understood);
    if !own.is_empty() && !own.iter().any(|name| _says_it(&line, name)) {
        return false;
    }
    for name in own {
        line = line.replace(&format!(" {name} "), " ");
        line = line.replace(&format!(" {name}s "), " ");
    }
    let ask: Vec<&String> = TIME_ASK.iter().filter(|p| line.contains(p.as_str())).collect();
    let compared_period = _a_period_beside_a_comparison(&line);
    if !(TIME_ASK_SHAPE.iter().any(|shape| shape.is_match(&line)) || compared_period) {
        return false;
    }
    // "when" on its own is the one time word that does not have to be asking anything
    if !ask.is_empty() && !ask.iter().any(|p| p.as_str() != "when") && !any_in(&line, &WHEN_ASKING) {
        return false;
    }
    if any_in(&line, &_HAS_FAILED) && !any_in(&line, &_BUY_THE_NEXT) {
        return false;
    }
    !any_in(&line, &NOT_THE_CALENDAR)
}

/// The care words, less the ones the row's own item is written with (build-time, kept for callers).
pub fn _avoid_for(own: &[String]) -> Vec<String> {
    NOT_THE_CALENDAR
        .iter()
        .filter(|word| {
            !own.iter().any(|name| {
                name.split_whitespace().any(|one| one.starts_with(word.as_str()) || word.starts_with(one))
            })
        })
        .cloned()
        .collect()
}

/// `_whenbuy(reply, own).<locals>.answer`: a calendar row's answer, and None for a line that is
/// not asking the calendar question.
fn _whenbuy_answer(f: &FnRef, _memory: &mut Memory, understood: &Understood) -> Option<String> {
    let own = f.cell("own").map(|v| v.strs()).unwrap_or_default();
    if !_asks_the_calendar(understood, &own) {
        return None;
    }
    f.cell("reply").and_then(|v| v.as_str()).map(str::to_string)
}

/// The whole-year calendar, for a line that asks when things are on sale and names no item.
pub fn _general_sales(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !_asks_the_calendar(understood, &[]) {
        return None;
    }
    Some(SALES_CALENDAR.clone())
}

/// A tree is bought in December; the decorations are bought in the January after.
pub fn _christmas(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !_asks_the_calendar(understood, &[]) {
        return None;
    }
    let line = _line(understood);
    let doing = [
        " take ", " taken ", " taking ", " put ", " putting ", " pack ", " packing ", " throw ", " go up ",
        " goes up ", " going up ", " gone up ", " come down ", " comes down ", " coming down ",
    ]
    .iter()
    .any(|one| line.contains(one));
    let gone = [" down ", " up ", " away ", " out "].iter().any(|one| line.contains(one));
    if doing && gone {
        return None;
    }
    let tree = ["tree", "trees", "real tree", "fir"].iter().any(|one| line.contains(one));
    let lights = ["light", "lights", "fairy", "decorations", "baubles", "tinsel", "ornaments"]
        .iter()
        .any(|one| line.contains(one));
    if tree && !lights {
        return Some(
            "Buy a real tree in mid-December, a week or two before Christmas: a tree cut in \
November drops its needles by the big day, so ask the grower when it was cut and \
keep it in water. Artificial trees are cheapest in the January sales, so buy the \
one you want now and store it."
                .to_string(),
        );
    }
    if lights && !tree {
        return Some(
            "Christmas lights and decorations are cheapest in the first week of January, at 70 \
to 90 per cent off, so buy for next year on the 2nd or 3rd rather than in \
November, when the same set is full price."
                .to_string(),
        );
    }
    Some(
        "Buy a real tree in mid-December, a week or two before Christmas, from a grower who \
will tell you when it was cut; lights and decorations are cheapest in the first week \
of January, at 70 to 90 per cent off, for next year."
            .to_string(),
    )
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.whenbuy:<lambda>:434", |_: &FnRef, u: &Understood| !asks_where_a_shop_answers(u)),
    ("bot.knowledge.whenbuy:<lambda>:447", |_: &FnRef, u: &Understood| !asks_where_a_shop_answers(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.whenbuy:_whenbuy.<locals>.answer:296", _whenbuy_answer),
    ("bot.knowledge.whenbuy:_general_sales:375", |_: &FnRef, m: &mut Memory, u: &Understood| _general_sales(m, u)),
    ("bot.knowledge.whenbuy:_christmas:382", |_: &FnRef, m: &mut Memory, u: &Understood| _christmas(m, u)),
];
