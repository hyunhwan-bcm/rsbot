//! Port of `bot/knowledge/entertainment.py`: films, books, telly, podcasts, music, games and the
//! made-up things (a poem, a story, a rap) a line can ask for.
//!
//! The entries, word lists, reply pools and regexes are data from the bundle; this file holds the
//! guards and replies the entries call, and the helpers they share. Build-time only (their results
//! are in the bundle): `_mood_key_words` (MOOD_WORDS), `_mood_signals` (MOOD_SIGNALS),
//! `_picks_by_title` (PICK_TITLES), and the factories' outer bodies (`_mood_pick`, `_film_list`,
//! `_with_the_detail_asked_for`, `_a_second_one_read_first`), whose inner functions are here.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d2_shared;
use crate::knowledge::d2dup_entertainment as dup;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

// ---------------------------------------------------------------------------------------------
// Reading the module's data

const P: &str = "bot.knowledge.entertainment.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}
fn strs_of(name: &str) -> Vec<String> {
    g(name).strs()
}
fn set_of(name: &str) -> HashSet<String> {
    g(name).str_set()
}
fn map_of(name: &str) -> IndexMap<String, String> {
    g(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
}
fn list_map_of(name: &str) -> IndexMap<String, Vec<String>> {
    g(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
}
fn int_map_of(name: &str) -> IndexMap<String, i64> {
    g(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
}
fn re_of(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("{P}{name} is not a regex"))
}
fn pairs_of(name: &str) -> Vec<(String, String)> {
    g(name).items().iter().map(|row| (row.items()[0].str().to_string(), row.items()[1].str().to_string())).collect()
}

macro_rules! strs { ($($n:ident),* $(,)?) => { $(static $n: LazyLock<Vec<String>> = LazyLock::new(|| strs_of(stringify!($n)));)* }; }
macro_rules! sets { ($($n:ident),* $(,)?) => { $(static $n: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(stringify!($n)));)* }; }
macro_rules! maps { ($($n:ident),* $(,)?) => { $(static $n: LazyLock<IndexMap<String, String>> = LazyLock::new(|| map_of(stringify!($n)));)* }; }
macro_rules! list_maps { ($($n:ident),* $(,)?) => { $(static $n: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| list_map_of(stringify!($n)));)* }; }
macro_rules! res { ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Re> = LazyLock::new(|| re_of(stringify!($n)));)* }; }
macro_rules! text_of { ($($n:ident),* $(,)?) => { $(static $n: LazyLock<String> = LazyLock::new(|| g(stringify!($n)).str().to_string());)* }; }

/// A regex built from text at run time (`re.search(r"\b" + re.escape(x) + r"\b", ...)`),
/// compiled once per distinct pattern.
fn rx(pattern: String) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<String, &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(r) = cache.get(&pattern) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(&pattern, 0)));
    cache.insert(pattern, r);
    r
}

/// An inline pattern written in a function, compiled once.
macro_rules! lazy_re {
    ($name:ident, $pat:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, 0));
    };
    ($name:ident, $pat:expr, $flags:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, $flags));
    };
}

/// `(?<![a-z0-9])` + re.escape(phrase) + `(?![a-z0-9])`.
fn bounded_alnum(phrase: &str) -> &'static Re {
    rx(format!("(?<![a-z0-9]){}(?![a-z0-9])", pyre::escape(phrase)))
}

/// Python `x or y` on strings.
fn or<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.is_empty() { b } else { a }
}

/// Python's truth of an optional reply (None and "" are both false).
fn truthy(s: &Option<String>) -> bool {
    s.as_deref().is_some_and(|s| !s.is_empty())
}

/// `str(value)` for the plain values memory holds.
fn py_str(v: &Value) -> String {
    match v {
        Value::None => "None".into(),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Str(s) => s.to_string(),
        _ => String::new(),
    }
}

/// `str(d.get(key) or "")` for a dict value in memory.
fn dict_text(d: Option<&Value>, key: &str) -> String {
    match d.and_then(|d| d.get(key)) {
        Some(v) if v.truthy() => py_str(v),
        _ => String::new(),
    }
}

/// `memory.get(key)` when it is a dict, else None.
fn mem_dict<'a>(memory: &'a Memory, key: &str) -> Option<&'a Value> {
    memory.extra.get(key).filter(|v| matches!(v, Value::Dict(_)))
}

/// `" ".join(words)` lower-cased, the way `str(word).lower()` reads each word.
fn lower_words(words: &[String]) -> Vec<String> {
    words.iter().map(|w| w.to_lowercase()).collect()
}

/// Python `str.format` for the named fields the verse and story templates use.
fn format_named(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                out.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                out.push('}');
            }
            '{' => {
                let mut name = String::new();
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                    name.push(c);
                }
                match fields.iter().find(|(k, _)| *k == name) {
                    Some((_, v)) => out.push_str(v),
                    None => panic!("KeyError: {name}"),
                }
            }
            c => out.push(c),
        }
    }
    out
}

/// `template % value` for a template with one `%s`.
fn percent_s(template: &str, value: &str) -> String {
    template.replacen("%s", value, 1)
}

/// The sum of a text's code points, the seed several pickers use.
fn ord_sum(text: &str) -> i64 {
    text.chars().map(|c| c as i64).sum()
}

/// The last `n` turns (`turns[-n:]`).
fn last_turns(memory: &Memory, n: usize) -> &[crate::state::Turn] {
    let t = &memory.turns;
    &t[t.len().saturating_sub(n)..]
}

// ---------------------------------------------------------------------------------------------
// Chess, cities, who made what

res!(_CHESS_WORDS, _CASTLING_VERB);
strs!(_AMBIGUOUS_CITY_WORDS);

/// Whether the line is about chess (the pieces, the board, castling).
pub fn _a_chess_line(understood: &Understood) -> bool {
    let said = format!("{} {} {}", understood.clean, understood.norm_text, understood.text).to_lowercase();
    if _CHESS_WORDS.is_match(&said) {
        return true;
    }
    _CASTLING_VERB.is_match(&said)
}

/// Each city travel knows, with its bounded pattern (the ambiguous words left out).
static CITY_PATTERNS: LazyLock<Vec<&'static Re>> = LazyLock::new(|| {
    data::get("bot.knowledge.travel.CITIES")
        .pairs()
        .iter()
        .map(|(k, _)| k.str())
        .filter(|city| !_AMBIGUOUS_CITY_WORDS.iter().any(|w| w == city))
        .map(|city| rx(format!("(?<![a-z]){}(?![a-z])", pyre::escape(city))))
        .collect()
});

/// True when the line names no city travel has a card for.
pub fn _no_city_is_named(understood: &Understood) -> bool {
    let clean = format!(" {} ", understood.clean.to_lowercase().replace('-', " "));
    !CITY_PATTERNS.iter().any(|r| r.is_match(&clean))
}

sets!(_SMALL);

/// A work's name in title case, small words left small.
pub fn _title(work: &str) -> String {
    let words = text::split_ref(work);
    if let Some(first) = words.first() {
        if text::is_digit(first) {
            return work.to_string();
        }
    }
    words
        .iter()
        .enumerate()
        .map(|(i, w)| if i > 0 && _SMALL.contains(*w) { w.to_string() } else { text::upper_first(w) })
        .collect::<Vec<_>>()
        .join(" ")
}

/// WORKS: name -> (maker, detail, kind), each with its `\b` pattern.
static WORKS: LazyLock<Vec<(String, [String; 3], &'static Re)>> = LazyLock::new(|| {
    g("WORKS")
        .pairs()
        .iter()
        .map(|(k, v)| {
            let work = k.str().to_string();
            let it = v.strs();
            let re = rx(format!("\\b{}\\b", pyre::escape(&work)));
            (work, [it[0].clone(), it[1].clone(), it[2].clone()], re)
        })
        .collect()
});

/// Who made a named work: "Nineteen Eighty-Four is a novel by George Orwell, ...".
pub fn who_made(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = or(&understood.clean, &understood.text).to_lowercase().replace(['\'', '\u{2019}'], "");
    let mut best: Option<&(String, [String; 3], &'static Re)> = None;
    for row in WORKS.iter() {
        if row.2.is_match(&clean) && best.is_none_or(|b| row.0.chars().count() > b.0.chars().count()) {
            best = Some(row);
        }
    }
    let (work, [maker, detail, kind], _) = best?;
    let name = _title(work);
    let first: String = kind.chars().take(1).collect();
    let article = if "aeiou".contains(first.as_str()) { "an" } else { "a" };
    Some(format!("{name} is {article} {kind} by {maker}, {detail}."))
}

static BOOK_AUTHORS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs_of("BOOK_AUTHORS"));
maps!(TITLE_SPELLING, AUTHOR_ALSO, TITLE_AS);
strs!(TITLE_SMALL);

/// The surname an author is found by.
pub fn _author_key(author: &str) -> String {
    text::split_ref(author).last().map(|w| w.to_lowercase()).unwrap_or_else(|| panic!("IndexError"))
}

/// Every title the table gives this author.
pub fn _titles_of(author: &str) -> Vec<String> {
    BOOK_AUTHORS.iter().filter(|(_, w)| w == author).map(|(t, _)| t.clone()).collect()
}

/// The longest book title the line names, with its author.
pub fn _title_in(clean: &str) -> Option<(String, String)> {
    let mut best: Option<&(String, String)> = None;
    for row in BOOK_AUTHORS.iter() {
        if bounded_alnum(&row.0).is_match(clean) && best.is_none_or(|b| row.0.chars().count() > b.0.chars().count()) {
            best = Some(row);
        }
    }
    best.cloned()
}

/// The author the line names (by surname or a longer form), skipping words of the title.
pub fn _author_in(clean: &str, title: Option<&str>) -> Option<String> {
    let mut seen: Vec<&str> = Vec::new();
    for (_, writer) in BOOK_AUTHORS.iter() {
        if seen.contains(&writer.as_str()) {
            continue;
        }
        seen.push(writer);
    }
    for author in seen {
        let key = _author_key(author);
        let mut words = vec![key.clone()];
        if let Some(also) = AUTHOR_ALSO.get(&key) {
            if !also.is_empty() {
                words.push(also.clone());
            }
        }
        for word in &words {
            if let Some(t) = title {
                if !t.is_empty() && t.contains(word.as_str()) {
                    continue;
                }
            }
            if bounded_alnum(word).is_match(clean) {
                return Some(author.to_string());
            }
        }
    }
    None
}

/// A title as it is printed.
pub fn _display(text_: &str) -> String {
    if let Some(shown) = TITLE_AS.get(text_) {
        return shown.clone();
    }
    let words = text::split_ref(text_);
    let mut out = vec![text::upper_first(words[0])];
    for word in &words[1..] {
        out.push(if TITLE_SMALL.iter().any(|s| s == word) { word.to_string() } else { text::upper_first(word) });
    }
    out.join(" ")
}

/// A name as said, without its full stops.
pub fn _said(text_: &str) -> String {
    text_.replace('.', "")
}

/// "a, b and c".
pub fn _list_words(items: &[String]) -> String {
    if items.len() == 1 {
        return items[0].clone();
    }
    format!("{} and {}", items[..items.len() - 1].join(", "), items.last().expect("IndexError"))
}

/// Who wrote a book, or which books an author wrote.
pub fn _who_wrote(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut clean = understood.clean.replace(" & ", " and ");
    for (spelling, standard) in TITLE_SPELLING.iter() {
        clean = clean.replace(spelling.as_str(), standard);
    }
    let found = _title_in(&clean);
    let named = _author_in(&clean, found.as_ref().map(|f| f.0.as_str()));
    if let Some((title, author)) = found {
        let others: Vec<String> = _titles_of(&author)
            .into_iter()
            .filter(|t| *t != title && !t.contains(title.as_str()) && !t.starts_with(title.as_str()) && !title.starts_with(t.as_str()))
            .collect();
        let rest = if others.is_empty() {
            String::new()
        } else {
            let shown: Vec<String> = others.iter().take(3).map(|t| _display(t)).collect();
            format!(" Other books by {}: {}.", _said(&author), _list_words(&shown))
        };
        if let Some(named) = &named {
            if *named != author {
                return Some(format!("Not quite - {} is by {}, not {}.{rest}", _display(&title), _said(&author), _said(named)));
            }
            return Some(format!("Yes - {} is by {}.{rest}", _display(&title), _said(&author)));
        }
        return Some(format!("{} is by {}.{rest}", _display(&title), _said(&author)));
    }
    let named = named?;
    let titles = _titles_of(&named);
    if titles.is_empty() {
        return None;
    }
    let shown: Vec<String> = titles.iter().take(5).map(|t| _display(t)).collect();
    let said = _list_words(&shown);
    let more = if titles.len() <= 5 { "" } else { ", among others" };
    Some(format!("{} wrote {said}{more}.", _said(&named)))
}

// ---------------------------------------------------------------------------------------------
// Books like a named one; children's parties; music for an occasion; family films

static BOOK_MATCHES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs_of("BOOK_MATCHES"));
res!(LIKE_A_NAMED_BOOK);

/// The longest book title from the match table that the text names, or "".
pub fn _book_title_named(text_: &str) -> String {
    let mut best = String::new();
    for (title, _) in BOOK_MATCHES.iter() {
        if title.chars().count() > best.chars().count() && bounded_alnum(title).is_match(text_) {
            best = title.clone();
        }
    }
    best
}

lazy_re!(
    BOOK_LIKE_A_CHILD,
    "\\b\\d+ ?year old|\\byear olds?\\b|\\bkids?\\b|\\bchildren?\\b|\\btoddlers?\\b|\\bbab(?:y|ies)\\b|\\b(?:my|our|his|her|their) \
     (?:son|daughter|child|boy|girl|grandson|granddaughter|niece|nephew)\\b"
);
lazy_re!(LIKE_CUT, "\\b(?:but|that|which|without|and|because)\\b");

/// A book like the one the line names; "" when it is a child's reading or no book is named.
pub fn _book_like(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = or(&understood.clean, &understood.norm_text).to_lowercase();
    let named = _book_title_named(&text_);
    let scan = if named.is_empty() { text_.clone() } else { text_.replace(named.as_str(), " ") };
    if BOOK_LIKE_A_CHILD.is_match(&scan) {
        return Some(String::new());
    }
    for (title, reply) in BOOK_MATCHES.iter() {
        if *title == named {
            return Some(reply.clone());
        }
    }
    let mut said = String::new();
    if let Some(found) = LIKE_A_NAMED_BOOK.search(&text_) {
        let got = or(found.str(1), found.str(2));
        let got = text::strip_chars(got, " .?!");
        let first = LIKE_CUT.split(got).into_iter().next().unwrap_or_default();
        said = text::strip_chars(&first, " .?!").to_string();
    }
    if !said.is_empty() {
        return Some(format!(
            "I don't know *{}* well enough to match it, and a guess would waste your evening. Tell me \
             what you liked about it - the people, the pace, or the writing - and I'll pick something \
             close.",
            text::title(&said)
        ));
    }
    Some(String::new())
}

strs!(CHILD_WORDS, _AGE_SHAPES);
text_of!(_PARTY_TINY, _PARTY_MIDDLE, _PARTY_OLDER, _PARTY_ANY);
static AGE_SHAPES: LazyLock<Vec<&'static Re>> = LazyLock::new(|| _AGE_SHAPES.iter().map(|s| rx(s.clone())).collect());

/// A child's age (1 to 16) the line gives.
pub fn _age_in(line: &str) -> Option<i64> {
    for shape in AGE_SHAPES.iter() {
        if let Some(found) = shape.search(line) {
            let years: i64 = found.str(1).parse().expect("int()");
            if (1..=16).contains(&years) {
                return Some(years);
            }
        }
    }
    None
}

/// Party games for a child's age.
pub fn _party_games_for_kids(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = or(&understood.clean, &understood.text).to_lowercase();
    if !CHILD_WORDS.iter().any(|w| line.contains(w.as_str())) {
        return Some(String::new());
    }
    if line.contains("adult") || line.contains("grown up") || line.contains("grown-up") {
        return Some(String::new());
    }
    let years = match _age_in(&line) {
        Some(y) => y,
        None => {
            if ["toddler", "toddlers", "baby", "babies", "nursery"].iter().any(|w| line.contains(w)) {
                2
            } else {
                let said = _PARTY_ANY.as_str();
                return Some(if line.contains("indoor") { _indoors(said, &line) } else { said.to_string() });
            }
        }
    };
    let said = if years <= 4 {
        _PARTY_TINY.as_str()
    } else if years <= 7 {
        _PARTY_MIDDLE.as_str()
    } else {
        _PARTY_OLDER.as_str()
    };
    Some(_indoors(said, &line))
}

/// The garden games moved indoors when the line says it is inside.
pub fn _indoors(said: &str, line: &str) -> String {
    if line.contains("indoor") || line.contains("indoors") || line.contains("inside") || line.contains("flat") {
        return said.replace("round the garden", "round the house");
    }
    said.to_string()
}

fn any_in(line: &str, words: &[&str]) -> bool {
    words.iter().any(|w| line.contains(w))
}

const WEDDING_THREE: &str = "Three moments, three songs: walk in - *Canon in D*, or the *Trumpet Voluntary* if it is \
    grand; sign the register - *A Thousand Years* or *Make You Feel My Love*; walk out - \
    *Signed, Sealed, Delivered* or *This Will Be*. For the evening, the songs everyone \
    already knows (ABBA, Queen, *Mr Brightside*) fill a floor better than anything tasteful. \
    Ask the couple which three songs they would dance to in an empty kitchen.";

/// What to listen to for an occasion or a journey.
pub fn _listen_when(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = or(&understood.clean, &understood.text).to_lowercase();
    if line.contains("podcast") || line.contains("podcasts") {
        let said = format!(" {} ", text::squash(&line));
        if said.contains(" about ") || said.contains(" on ") {
            return None;
        }
    }
    let reply = |s: &str| Some(s.to_string());
    if any_in(&line, &["drive", "driving", "car", "road trip", "motorway"]) {
        return reply(
            "For a long drive: *Rumours* by Fleetwood Mac and *Graceland* by Paul Simon are both easy \
             to sing along to, and *The Rest Is History* if you would rather have talking than music. \
             Make one playlist before you go, because changing it at 70mph is the annoying part.",
        );
    }
    if any_in(
        &line,
        &[
            "kids party", "kids' party", "childrens party", "children's party", "childs party", "child's party",
            "toddler party", "party for kids", "party for children", "kids birthday", "childrens birthday",
        ],
    ) {
        return reply(
            "For a children's party the music wants to be louder and shorter than you think: a 60 to \
             90 minute playlist of songs they already know - *The Greatest Showman*, *Frozen*, Taylor \
             Swift, whatever the school run plays - from a phone on a speaker away from the food \
             table, started the moment they arrive. Cue musical statues about halfway through: it \
             costs nothing and it resets a room that is getting wild.",
        );
    }
    if any_in(&line, &["aisle", "walking in", "processional", "walk in to", "coming in to"]) && !line.contains("first dance") {
        return reply(
            "For the aisle pick the entrance, not a favourite song: *Canon in D* (Pachelbel) is the \
             default for good reason, *Here Comes the Bride* (Bridal Chorus) is the one everybody \
             knows, *A Thousand Years* (Christina Perri) or *Sweet Disposition* (Temper Trap) for \
             something modern, or a string version of a song you love if the room is small. Two \
             minutes is plenty, and play it again as you leave.",
        );
    }
    if any_in(&line, &["funeral", "memorial", "cremation", "crematorium", "burial", "church service", "send off", "wake"]) {
        return reply(
            "Three songs is the usual shape: one going in, one for the reflection or the photos, one \
             to walk out to. *Somewhere Over the Rainbow* (the Israel Kamakawiwo'ole version) or *Ave \
             Maria* suit a church, *Fix You* and *Everybody Hurts* suit a room that wants to cry, *My \
             Way* (Sinatra) or *Thank You for the Days* (the Kinks) walk people out smiling. Give the \
             funeral director titles, artists and exact versions a week ahead.",
        );
    }
    if line.contains("first dance") {
        return reply(
            "The song you both already sing badly in the car beats any list, but if you want \
             somewhere to start: *At Last* by Etta James, *You Are the Best Thing* by Ray LaMontagne, \
             *Can't Help Falling in Love* by Elvis, *Tenerife Sea* by Ed Sheeran, *The Book of Love* \
             by Peter Gabriel for something quieter, or *You're My Best Friend* by Queen for a room \
             that will sing along. Play each one all the way through and pick the one you can sway to \
             for three minutes without counting the beats, then give the DJ that exact version - a \
             remix has ruined more first dances than bad dancing.",
        );
    }
    if any_in(&line, &["signing of the register", "sign the register", "register signing", "signing the register"]) {
        return reply(
            "For the register, something the room knows and can hum while they wait: *A Thousand \
             Years* (Christina Perri), *Make You Feel My Love* (Adele), *Jesu, Joy of Man's Desiring* \
             if it is a church, or *Better Together* (Jack Johnson). Two songs is right - one while \
             the first people sign, one for the second - because signing takes longer than anyone \
             plans for.",
        );
    }
    if (line.contains("wedding") || line.contains("marrying")) && any_in(&line, &["music", "song", "songs", "playlist", "band", "dj"]) {
        return reply(WEDDING_THREE);
    }
    if any_in(
        &line,
        &[
            "wedding song", "wedding songs", "songs for a wedding", "wedding music", "music for a wedding",
            "song for a wedding", "ceremony music", "song for my wedding",
        ],
    ) {
        return reply(WEDDING_THREE);
    }
    if any_in(&line, &["wedding", "first dance", "reception", "bridal"]) {
        return reply(
            "Split the day, because a wedding is two different rooms: for the ceremony and the meal, \
             something quiet and familiar that nobody has to listen to - a solo piano or string \
             version of songs the couple likes - and for the evening, the songs everyone already \
             knows so the dance floor fills. Ask the couple for five songs they both love and build \
             the evening around those, keep a do-not-play list, and have the first dance track loaded \
             and cued rather than searched for on the night.",
        );
    }
    if any_in(&line, &["dinner party", "party", "barbecue", "bbq", "guests", "drinks", "buffet"]) {
        return reply(
            "For a room with people talking in it, the music is wallpaper, not the show: something \
             familiar and mid-tempo that nobody has to listen to - a Motown or soul compilation, \
             *Rumours*, or any 'dinner party' playlist in the 90 to 110 beats-per-minute range. Start \
             it thirty minutes before people arrive, keep it quiet enough to talk over while you eat, \
             and turn it up once the plates are cleared.",
        );
    }
    if !line.contains("podcast")
        && any_in(
            &line,
            &[
                "rainy day", "rainy afternoon", "rainy sunday", "rainy weekend", "rainy morning", "wet afternoon",
                "wet day", "wet weekend", "snowy day", "stuck indoors",
            ],
        )
    {
        return reply(
            "For a rainy day: *Rumours* by Fleetwood Mac, *Blue* by Joni Mitchell if you want \
             something quieter, or *Graceland* by Paul Simon for a kitchen that needs waking up. Rain \
             is the one day a whole album beats a playlist, so put one on and leave it alone.",
        );
    }
    if any_in(&line, &["cook", "cooking", "kitchen", "dinner"]) {
        return reply(
            "While you cook: *Kind of Blue* by Miles Davis, *The Köln Concert* by Keith Jarrett, or \
             any Motown compilation. All of them have no words to follow, so you can still hear the \
             timer.",
        );
    }
    if any_in(&line, &["run", "running", "jog", "gym", "workout"]) {
        return reply(
            "For a run: something around 160 to 170 beats per minute keeps the pace steady, so *Mr \
             Brightside*, *Titanium* and *Don't Stop Me Now* all work, or a Daft Punk album run \
             straight through if you would rather not think about it.",
        );
    }
    if any_in(&line, &["walk", "walking", "way to work", "walk to work", "commute", "commuting", "journey to work"]) {
        return reply(
            "For a commute: *The Rest Is History* or *Desert Island Discs* if you want talking, \
             *Rumours* if you want music. Podcasts beat music for a commute because the time passes \
             without you counting it.",
        );
    }
    reply(
        "For a long listen: *Rumours* by Fleetwood Mac, *Graceland* by Paul Simon, or *The Rest \
         Is History* if you would rather hear talking. Tell me what the journey is and I'll make \
         it specific.",
    )
}

strs!(CHILD);
lazy_re!(FAMILY_TEEN, "\\bteen\\w*|\\badolescen\\w*|\\b1[3-9]\\s*(?:year|yr)|\\b(?:aged|age)\\s*1[3-9]\\b");
lazy_re!(
    FAMILY_AGES,
    "\\b(\\d{1,2})\\s*(?:year|yr)s?[ -]old\\b|\\bage[d]?\\s*(\\d{1,2})\\b|\\b(?:is|are)\\s+(\\d{1,2})\\b"
);

/// A film the whole family can watch, by the youngest age named.
pub fn _family_film(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.as_str();
    let said = format!("{} {}", understood.clean, understood.text);
    let family = ["family", "teenager", "teenagers", "teen", "teens"];
    if !CHILD.iter().any(|w| said.contains(w.as_str())) && !family.iter().any(|w| said.contains(w)) {
        return _films_within(
            Some(
                "For a laugh: *Groundhog Day* is the funniest safe pick, *The Grand Budapest Hotel* if \
                 you want something stylish with the same warmth, and *Paddington 2* if you want to laugh \
                 from the first minute. All three are fine with anyone in the room.",
            ),
            Some(&said),
        );
    }
    if FAMILY_TEEN.is_match(clean) {
        return Some(
            "With teenagers in the room: *Spider-Man: Into the Spider-Verse* (funny, fast and \
             genuinely good), *Knives Out* if they will sit through a 12A whodunnit, and *Back to the \
             Future* for the one everybody ends up watching. *The Mitchells vs the Machines* is the \
             safe pick for a mixed-age sofa, and *Paddington 2* still works on a teenager who has not \
             seen it."
                .into(),
        );
    }
    let mut ages: Vec<i64> = FAMILY_AGES
        .finditer(clean)
        .iter()
        .map(|m| or(or(m.str(1), m.str(2)), m.str(3)).parse::<i64>().expect("int()"))
        .filter(|n| (1..=16).contains(n))
        .collect();
    ages.sort();
    ages.dedup();
    if let Some(&years) = ages.first() {
        if years <= 4 {
            return Some(
                "For a four-year-old: *The Gruffalo* and *Room on the Broom* (both about 25 minutes) and \
                 *The Snail and the Whale* are the safest, then *Zootropolis* or *Paddington 2* for a \
                 full film. Save *Matilda* and *The Mitchells vs the Machines* for a year or two."
                    .into(),
            );
        }
        if years <= 7 {
            return Some(
                "For a five- or six-year-old: *Paddington 2* (funny, warm, no real peril), *The Gruffalo* \
                 and *Room on the Broom* (both about 25 minutes), and *Zootropolis* if you want a full \
                 film. Save *Matilda* for a year or two."
                    .into(),
            );
        }
    }
    Some(
        "*Paddington 2*: funny for the adults too, and nothing in it will scare an 8- or \
         11-year-old. *The Mitchells vs the Machines* or *Matilda* are good backups."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// Film moods: what the chat turned down, and what it holds

text_of!(GENTLE_LIST, EASY_FILM_LIST);
maps!(FILM_MOODS);

/// The gentle film list, or the funny one when the gentle list was the last answer.
pub fn _the_gentle_list(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = dict_text(mem_dict(memory, "last_answer"), "reply");
    if !said.is_empty() && _plain_words(&said) == _plain_words(&GENTLE_LIST) {
        let kind = _pick_kind(&_own_texts(understood).join(" "));
        if !kind.is_empty() && kind != "film" {
            return Some(GENTLE_LIST.clone());
        }
        let other = FILM_MOODS.get("funny").cloned().unwrap_or_default();
        if !other.is_empty() && _plain_words(&other) != _plain_words(&GENTLE_LIST) {
            return Some(other);
        }
        return None;
    }
    Some(GENTLE_LIST.clone())
}

/// The texts of the chat's topics (`memory["topics"]`), newest first; None when it is not a list.
fn topic_texts_newest_first(memory: &Memory) -> Option<Vec<String>> {
    let topics = memory.extra.get("topics")?;
    let Value::List(items) = topics else { return None };
    Some(
        items
            .iter()
            .rev()
            .map(|t| if matches!(t, Value::Dict(_)) { dict_text(Some(t), "text") } else { String::new() })
            .collect(),
    )
}

/// Whether the chat's last topic that names one of `names` turned it down.
pub fn _the_chat_turned_down(names: &HashSet<String>, memory: &Memory, after: Option<&Re>) -> bool {
    let Some(texts) = topic_texts_newest_first(memory) else { return false };
    for text_ in texts {
        if text_.is_empty() {
            continue;
        }
        if !_plain_words(&text_).split(' ').any(|w| names.contains(w)) {
            continue;
        }
        return _a_word_is_turned_down(&text_, names, after);
    }
    false
}

/// Whether the chat's last topic with a mood in it turned that mood down.
pub fn _the_mood_the_chat_turned_down(memory: &Memory) -> bool {
    let Some(texts) = topic_texts_newest_first(memory) else { return false };
    for text_ in texts {
        if text_.is_empty() {
            continue;
        }
        if let Some(got) = _mood_of(&text_) {
            return got.1;
        }
    }
    false
}

/// The mood the chat's last topic with a mood in it asked for (not one it turned down).
pub fn _the_mood_the_chat_holds(memory: &Memory) -> Option<String> {
    let texts = topic_texts_newest_first(memory)?;
    for text_ in texts {
        if text_.is_empty() {
            continue;
        }
        if let Some(got) = _mood_of(&text_) {
            if !got.1 {
                return Some(got.0);
            }
        }
    }
    None
}

static FILM_MATCHES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| pairs_of("FILM_MATCHES"));
res!(_ASKS_FOR_A_FILM);
lazy_re!(
    FILM_LIKE_NAMED,
    "\\b(?:film|movie|show|series|programme)s?\\s+(?:like|similar to)\\s+([a-z0-9'&: -]{2,40})"
);
lazy_re!(FILM_LIKE_CUT, "\\b(?:but|that|which|without|and|or|please)\\b|\\b(?:film|movie|show|series|programme)s?\\b");
lazy_re!(FILMS_WORD, "\\b(?:films?|movies?)\\b");

/// A film like the one the line names, or the easy list when it names none.
pub fn _film_like(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let texts: Vec<String> =
        [&understood.clean, &understood.norm_text].iter().filter(|t| !t.is_empty()).map(|t| t.to_lowercase()).collect();
    if !_the_liked_thing_is_asked_about(understood) {
        if let Some(held) = _the_mood_the_chat_holds(memory) {
            if let Some(picks) = FILM_MOODS.get(&held) {
                if !picks.is_empty() {
                    return _films_within(Some(picks), Some(&understood.clean));
                }
            }
        }
        return Some(EASY_FILM_LIST.clone());
    }
    for t in &texts {
        let pick = _table_pick(t);
        if !pick.is_empty() {
            return Some(pick);
        }
    }
    for t in &texts {
        if let Some(named) = FILM_LIKE_NAMED.search(t) {
            let first = FILM_LIKE_CUT.split(named.str(1)).into_iter().next().unwrap_or_default();
            let title = text::strip_chars(&first, " .?!");
            if !title.is_empty() {
                return Some(format!(
                    "I don't know *{}* well enough to match it. Tell me what you liked about it: the story, \
                     the jokes, or how tense it was, and I'll pick something close.",
                    text::title(title)
                ));
            }
        }
    }
    let own = or(&understood.text, &understood.clean).to_lowercase();
    if !(_ASKS_FOR_A_FILM.is_match(&own) && FILMS_WORD.is_match(&own)) {
        return Some(String::new());
    }
    Some(
        "Tell me what you liked about the last film you enjoyed: the story, the jokes, or how \
         tense it was. Name one and I'll suggest something close."
            .into(),
    )
}

res!(NOT_A_TURN_DOWN);
strs!(TURNED_DOWN_WANT);
sets!(TURNS_BACK, NEGATORS, SCARE_TURNED_DOWN, SCARE_WORDS, FILM_ASK_WORDS);

/// Whether the text turns down one of `names` ("nothing scary", "no horror please").
pub fn _a_word_is_turned_down(text_: &str, names: &HashSet<String>, after: Option<&Re>) -> bool {
    if names.is_empty() || text_.is_empty() {
        return false;
    }
    let mut plain = NOT_A_TURN_DOWN.sub(" ", &_plain_words(text_));
    for phrase in TURNED_DOWN_WANT.iter() {
        plain = plain.replace(phrase.as_str(), " not ");
    }
    let words = text::split_ref(&plain);
    for (at, word) in words.iter().enumerate() {
        if !names.contains(*word) && !names.contains(word.trim_end_matches('s')) {
            continue;
        }
        let mut before = &words[at.saturating_sub(8)..at];
        for cut in (0..before.len()).rev() {
            if TURNS_BACK.contains(before[cut]) {
                before = &before[cut + 1..];
                break;
            }
        }
        if before.iter().any(|one| NEGATORS.contains(*one) || SCARE_TURNED_DOWN.contains(*one)) {
            return true;
        }
        if let Some(after) = after {
            let end = (at + 9).min(words.len());
            let start = (at + 1).min(end);
            if after.is_match(&words[start..end].join(" ")) {
                return true;
            }
        }
    }
    false
}

/// The line as the person typed it, or its readings when the typed text is missing.
pub fn _own_texts(understood: &Understood) -> Vec<String> {
    if !text::strip(&understood.text).is_empty() {
        return vec![understood.text.clone()];
    }
    let mut texts: Vec<String> = Vec::new();
    for value in [&understood.norm_text, &understood.clean] {
        if !value.is_empty() && !texts.contains(value) {
            texts.push(value.clone());
        }
    }
    texts
}

res!(SCARE_REFUSED_AFTER);

/// Whether the line turns a scare down ("nothing scary").
pub fn _the_scare_is_refused(understood: &Understood) -> bool {
    _own_texts(understood).iter().any(|t| _a_word_is_turned_down(t, &SCARE_WORDS, Some(&SCARE_REFUSED_AFTER)))
}

/// Whether the line asks for a film (or turns a scare down, which is the same ask).
pub fn _a_film_ask(understood: &Understood) -> bool {
    for t in _own_texts(understood) {
        let plain = format!(" {} ", _plain_words(&t));
        if FILM_ASK_WORDS.iter().any(|w| plain.contains(&format!(" {w} "))) {
            return true;
        }
    }
    _the_scare_is_refused(understood)
}

/// The scare rows answer only a line that does not turn the scare down.
pub fn _the_scare_is_wanted(understood: &Understood) -> bool {
    !_the_scare_is_refused(understood)
}

static VIEWS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| map_of("VIEWS"));

/// The longest work the verdict table knows that the text names, or "".
pub fn _work_named(text_: &str) -> String {
    let mut best = String::new();
    for title in VIEWS.keys() {
        if title.chars().count() > best.chars().count() && bounded_alnum(title).is_match(text_) {
            best = title.clone();
        }
    }
    best
}

res!(_FRESH_LIST);
lazy_re!(LIKED_WORD, "\\b(?:like|likes|liked|loves?|loved|enjoyed|similar)\\b");

/// Whether the line asks about a thing it liked (rather than for a fresh list).
pub fn _the_liked_thing_is_asked_about(understood: &Understood) -> bool {
    let own = or(&understood.text, &understood.clean).to_lowercase();
    if !_FRESH_LIST.is_match(&own) {
        return true;
    }
    if LIKED_WORD.is_match(&own) {
        return true;
    }
    !_book_title_named(&own).is_empty() || !_work_named(&own).is_empty()
}

res!(AGE_ASK, ASKING_FIRST, SEEN_ALREADY, _THE_OFFICE_IS);
strs!(A_PLACE_NOT_A_WORK);

/// Whether the line asks about a person (an actor's age, someone seen already).
pub fn _about_a_person(text_: &str) -> bool {
    if AGE_ASK.is_match(text_) {
        return true;
    }
    !ASKING_FIRST.is_match(text_) && SEEN_ALREADY.is_match(text_)
}

/// Whether "the office" and the like are a place in this line, not the series.
pub fn _a_place_not_a_work(text_: &str) -> bool {
    _THE_OFFICE_IS.is_match(text_) || A_PLACE_NOT_A_WORK.iter().any(|p| text_.contains(p.as_str()))
}

// ---------------------------------------------------------------------------------------------
// Moods

static MOOD_ORDER: LazyLock<Vec<String>> = LazyLock::new(|| strs_of("MOOD_ORDER"));
list_maps!(MOODS);
maps!(CONTRACTIONS, MOOD_BY_WORD);
sets!(SOFT, WANTING, MOOD_SIGNAL_SET);
lazy_re!(DASH_OR_QUOTE, "[-']");
lazy_re!(ALNUM_RUN, "[a-z0-9]+");
lazy_re!(NOT_CONTRACTION, "n'\\s*t\\b");

/// Each mood's phrases as word lists, in MOOD_ORDER.
static MOOD_PHRASES: LazyLock<Vec<(String, Vec<Vec<String>>)>> = LazyLock::new(|| {
    MOOD_ORDER
        .iter()
        .map(|mood| {
            let phrases = MOODS[mood]
                .iter()
                .map(|p| ALNUM_RUN.findall_str(&DASH_OR_QUOTE.sub(" ", p)))
                .collect();
            (mood.clone(), phrases)
        })
        .collect()
});

/// Whether the phrase at `start` is turned down by the words before it.
pub fn _negated(tokens: &[&str], start: usize, phrase: &[&str]) -> bool {
    if start == 0 || NEGATORS.contains(phrase[0]) {
        return false;
    }
    if NEGATORS.contains(tokens[start - 1]) {
        return true;
    }
    if start > 1 && NEGATORS.contains(tokens[start - 2]) && SOFT.contains(tokens[start - 1]) {
        return true;
    }
    for back in 1..=4 {
        if start > back && WANTING.contains(tokens[start - back]) && NEGATORS.contains(tokens[start - back - 1]) {
            return true;
        }
    }
    false
}

/// The line typed, as plain words with spaces round it.
pub fn _typed_line(understood: &Understood) -> String {
    format!(" {} ", _plain_words(or(&understood.text, &understood.clean)))
}

strs!(_WET_WORDS, _WET_PHRASES, _EVENING_WORDS);

/// Whether the line names wet weather.
pub fn _names_the_weather(understood: &Understood) -> bool {
    let line = _typed_line(understood);
    _WET_WORDS.iter().any(|w| line.contains(&format!(" {w} "))) || _WET_PHRASES.iter().any(|p| line.contains(p.as_str()))
}

/// Whether the line names the evening.
pub fn _names_the_evening(understood: &Understood) -> bool {
    let line = _typed_line(understood);
    _EVENING_WORDS.iter().any(|w| line.contains(&format!(" {w} ")))
}

/// The line lower-cased into plain words, contractions spelt out ("dont" -> "do not").
pub fn _plain_words(line: &str) -> String {
    let plain = line.to_lowercase().replace('\u{2019}', "'").replace('-', " ");
    let plain = NOT_CONTRACTION.sub("nt", &plain);
    let plain = plain.replace('\'', "");
    let mut words: Vec<String> = Vec::new();
    for word in ALNUM_RUN.findall_str(&plain) {
        match CONTRACTIONS.get(&word) {
            Some(full) => words.extend(text::split(full)),
            None => words.extend(text::split(&word)),
        }
    }
    words.join(" ")
}

/// The mood a line asks for, and whether it was only turned down (then the mood is "light").
pub fn _mood_of(line: &str) -> Option<(String, bool)> {
    let plain = _plain_words(line);
    let tokens = text::split_ref(&plain);
    if tokens.is_empty() {
        return None;
    }
    let mut refused: Option<String> = None;
    for (mood, phrases) in MOOD_PHRASES.iter() {
        for words in phrases {
            if words.is_empty() || words.len() > tokens.len() {
                continue;
            }
            let words: Vec<&str> = words.iter().map(String::as_str).collect();
            for start in 0..=(tokens.len() - words.len()) {
                if tokens[start..start + words.len()] != words[..] {
                    continue;
                }
                if _negated(&tokens, start, &words) {
                    if refused.is_none() {
                        refused = Some(mood.clone());
                    }
                    continue;
                }
                return Some((mood.clone(), false));
            }
        }
    }
    for (at, token) in tokens.iter().enumerate() {
        if !MOOD_SIGNAL_SET.contains(*token) {
            continue;
        }
        let mood = MOOD_BY_WORD.get(*token);
        if _negated(&tokens, at, &[token]) {
            if refused.is_none() {
                refused = Some(mood.cloned().unwrap_or_else(|| token.to_string()));
            }
            continue;
        }
        if let Some(mood) = mood {
            return Some((mood.clone(), false));
        }
    }
    if refused.as_deref().is_some_and(|r| !r.is_empty()) {
        return Some(("light".into(), true));
    }
    None
}

/// The general picks stand down for a line that names a mood.
pub fn _no_named_mood(understood: &Understood) -> bool {
    _mood_of(&understood.words.join(" ")).is_none()
}

static FILM_MINUTES: LazyLock<IndexMap<String, i64>> = LazyLock::new(|| int_map_of("FILM_MINUTES"));
lazy_re!(STARRED_TITLE, "\\*([^*]{2,60})\\*");

/// A film list trimmed to the running time the line asks for.
pub fn _films_within(text_: Option<&str>, line: Option<&str>) -> Option<String> {
    let (Some(text_), Some(line)) = (text_, line) else { return text_.map(str::to_string) };
    if text_.is_empty() {
        return Some(text_.to_string());
    }
    let (limit, strict) = _limit_wanted(line);
    if limit == 0 {
        return Some(text_.to_string());
    }
    let mut named: Vec<String> = Vec::new();
    for title in STARRED_TITLE.findall_str(text_) {
        if !named.contains(&title) {
            named.push(title);
        }
    }
    if named.is_empty() || named.iter().any(|t| !FILM_MINUTES.contains_key(t)) {
        return Some(text_.to_string());
    }
    let fits = |t: &String| if strict { FILM_MINUTES[t] < limit } else { FILM_MINUTES[t] <= limit };
    let fit: Vec<&String> = named.iter().filter(|t| fits(t)).collect();
    if fit.len() == named.len() {
        return Some(text_.to_string());
    }
    let over = named.len() - fit.len();
    let lead = text::strip(text_.split(':').next().unwrap_or("")).to_string();
    let has_lead = text_.contains(':') && !lead.is_empty() && lead.chars().count() <= 40 && !lead.contains('*');
    let mut short = &named[0];
    for t in &named {
        if FILM_MINUTES[t] < FILM_MINUTES[short] {
            short = t;
        }
    }
    let (under_at, under_upto) = if strict { ("under", "under") } else { ("at", "up to") };
    if fit.is_empty() {
        if has_lead {
            return Some(format!(
                "{lead}, nothing there comes in {under_at} {limit} minutes - the shortest is *{short}* at {}. Say \
                 another mood and I will look again.",
                FILM_MINUTES[short]
            ));
        }
        return Some(format!(
            "Nothing in that list comes in {under_at} {limit} minutes - the shortest is *{short}* at {}. Say \
             another mood and I will look again.",
            FILM_MINUTES[short]
        ));
    }
    let said_fit: Vec<String> = fit.iter().map(|t| format!("*{t}* ({} minutes)", FILM_MINUTES[*t])).collect();
    let body = if said_fit.len() == 1 {
        said_fit[0].clone()
    } else {
        format!("{} or {}", said_fit[..said_fit.len() - 1].join(", "), said_fit.last().unwrap())
    };
    if has_lead {
        return Some(format!(
            "{lead}, {under_upto} {limit} minutes: {body}. The other {} run{} longer, so say if you would rather \
             have one of those.",
            if over == 1 { "one" } else { "two" },
            if over == 1 { "s" } else { "" }
        ));
    }
    Some(format!(
        "Films that fit {under_upto} {limit} minutes: {body}. The rest of that list run{} longer, so say if you \
         would rather have one of those.",
        if over == 1 { "s" } else { "" }
    ))
}

/// `_mood_pick(picks).<locals>.pick`: the row for the mood the line asks for.
pub fn _mood_pick_pick(picks: &Value, _memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = format!("{} {}", understood.clean, understood.norm_text);
    let got = _mood_of(&line)?;
    let mood = if got.1 { "light".to_string() } else { got.0 };
    if mood.is_empty() {
        return None;
    }
    _films_within(picks.get(&mood).and_then(Value::as_str), Some(&line))
}

static MEDIUM_WORDS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    g("MEDIUM_WORDS").items().iter().map(|row| (row.items()[0].str().to_string(), row.items()[1].strs())).collect()
});

/// The medium (film, book, tv, podcast, music) the text names first.
pub fn _medium_named(text_: &str) -> Option<String> {
    let said = format!(" {} ", _plain_words(text_));
    let mut best: Option<&String> = None;
    let mut where_ = said.len() + 1;
    for (medium, words) in MEDIUM_WORDS.iter() {
        for word in words {
            if let Some(at) = said.find(&format!(" {word} ")) {
                if at < where_ {
                    best = Some(medium);
                    where_ = at;
                }
            }
        }
    }
    best.cloned()
}

/// The medium the line, the last turn or the last answer is about.
pub fn _the_medium(understood: &Understood, memory: &Memory) -> Option<String> {
    let mut order = vec![format!("{} {}", understood.clean, understood.norm_text)];
    // TYPE(understood.leans_on): Python reads leans_on["text"] when it is a dict; the typed
    // reading keeps it as a string, so there is no leaned-on text to add here.
    if let Some(last) = memory.turns.last() {
        order.push(last.user.clone());
        order.push(last.bot.clone());
    }
    for t in &order {
        if let Some(m) = _medium_named(t) {
            return Some(m);
        }
    }
    for t in &order {
        if let Some(m) = _title_medium(t) {
            return Some(m);
        }
    }
    let id = dict_text(mem_dict(memory, "last_answer"), "id");
    let named = id.split('-').next().unwrap_or("");
    if !named.is_empty() {
        for (medium, words) in MEDIUM_WORDS.iter() {
            if words.iter().any(|w| text::split_ref(w).first().is_some_and(|f| *f == named)) {
                return Some(medium.clone());
            }
        }
    }
    None
}

maps!(PICK_TITLES);

/// The medium of the pick title the text names first.
pub fn _title_medium(text_: &str) -> Option<String> {
    let said = format!(" {} ", _plain_words(text_));
    let mut best: Option<&String> = None;
    let mut where_ = said.len() + 1;
    for (title, medium) in PICK_TITLES.iter() {
        if let Some(at) = said.find(&format!(" {title} ")) {
            if at < where_ {
                best = Some(medium);
                where_ = at;
            }
        }
    }
    best.cloned()
}

/// Whether the chat is about picking something to watch, read or hear.
pub fn _chat_is_about_a_pick(understood: &Understood, memory: &Memory) -> bool {
    let domain = dict_text(mem_dict(memory, "last_answer"), "domain");
    // TYPE(understood.leans_on): the leaned-on topic is only read when it is a dict.
    let topic = understood.topic.clone().unwrap_or_default();
    if domain == "entertainment" || topic == "entertainment" {
        return true;
    }
    domain.is_empty() && (topic.is_empty() || topic == "feelings")
}

/// A pick in the other mood, for a line that turned its mood down.
pub fn _a_pick_of_the_other_mood(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = format!("{} {}", understood.clean, understood.norm_text);
    let got = _mood_of(&line)?;
    if !got.1 {
        return None;
    }
    _a_pick_in_any_medium(memory, understood)
}

strs!(A_LENGTH_MOOD);
static MOOD_TABLES: LazyLock<IndexMap<String, IndexMap<String, String>>> = LazyLock::new(|| {
    g("MOOD_TABLES")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.pairs().iter().map(|(a, b)| (a.str().to_string(), b.str().to_string())).collect()))
        .collect()
});
maps!(ANY_MOODS);

/// A pick for the mood the line asks for, in the medium the chat is about.
pub fn _a_pick_in_any_medium(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = format!("{} {}", understood.clean, understood.norm_text);
    let got = _mood_of(&line)?;
    if !_chat_is_about_a_pick(understood, memory) {
        return None;
    }
    if A_LENGTH_MOOD.contains(&got.0) && _pick_kind(&line).is_empty() && _pick_carry(memory, &line).is_empty() {
        return None;
    }
    let own = _mood_of(&_own_texts(understood).join(" "));
    let mood = match own {
        Some((_, true)) => "light".to_string(),
        Some((m, false)) => m,
        None => match _the_mood_the_chat_holds(memory) {
            Some(m) if !m.is_empty() => m,
            _ => {
                if !_the_mood_the_chat_turned_down(memory) {
                    return None;
                }
                "light".to_string()
            }
        },
    };
    static EMPTY: LazyLock<IndexMap<String, String>> = LazyLock::new(IndexMap::new);
    let picks: &IndexMap<String, String> =
        if _limit_wanted(&line).0 != 0 && _pick_kind(&line).is_empty() && _pick_carry(memory, &line).is_empty() {
            MOOD_TABLES.get("film").unwrap_or(&EMPTY)
        } else {
            _the_medium(understood, memory).and_then(|m| MOOD_TABLES.get(&m)).unwrap_or(&EMPTY)
        };
    let said = picks.get(&mood).filter(|s| !s.is_empty()).or_else(|| ANY_MOODS.get(&mood)).cloned();
    let last_reply = dict_text(mem_dict(memory, "last_answer"), "reply");
    let said = _films_within(said.as_deref(), Some(&line));
    if let Some(s) = said.as_deref() {
        if !s.is_empty() && _plain_words(s) == _plain_words(&last_reply) {
            if let Some(first) = STARRED_TITLE.search(s) {
                return Some(format!("Of those, I'd start with *{}*.", first.str(1)));
            }
            return None;
        }
    }
    said
}

strs!(VIEWING, ASKING_TO_WATCH_FOR);

/// False when the line asks what to watch out for (a content warning), not what to watch.
pub fn not_asking_what_to_watch_for(understood: &Understood) -> bool {
    let joined = understood.words.join(" ");
    let said = [&understood.text, &understood.clean, &understood.norm_text]
        .into_iter()
        .find(|s| !s.is_empty())
        .map(String::as_str)
        .unwrap_or(&joined)
        .to_lowercase();
    if said.is_empty() {
        return true;
    }
    if VIEWING.iter().any(|w| said.contains(w.as_str())) {
        return true;
    }
    !ASKING_TO_WATCH_FOR.iter().any(|a| said.contains(a.as_str()))
}

// ---------------------------------------------------------------------------------------------
// Starting a hobby, a genre, a card game, tuning; verdicts on named works

res!(_A_THING_IS_BEING_BAKED, _FISH_IS_DINNER, _PAINTED_THINGS, _PAINT_AS_A_THING, _PAINT_COLOURS);
sets!(_NOTHING_TO_BAKE, _PAINT_VERBS, _PAINTED_SURFACES, _PAINTED_FAULTS, _FISH_VERBS);
strs!(_PAINTED_PHRASES);
maps!(ACTIVITY_START, ACTIVITY_VERBS, ACTIVITY_OTHER_NAMES);
lazy_re!(LETTER_RUN, "[a-z']+");

/// Whether the text bakes a named thing ("baking a cake"), which is a recipe, not the hobby.
pub fn _a_thing_is_being_baked(text_: &str) -> bool {
    _A_THING_IS_BEING_BAKED.finditer(text_).iter().any(|hit| !_NOTHING_TO_BAKE.contains(hit.str(1)))
}

/// Whether a surface is being painted (a wall, a door), which is decorating, not the hobby.
pub fn _a_surface_is_being_painted(words: &HashSet<String>, padded: &str) -> bool {
    if !words.iter().any(|w| _PAINT_VERBS.contains(w)) {
        return false;
    }
    if words.iter().any(|w| _PAINTED_SURFACES.contains(w) || _PAINTED_FAULTS.contains(w)) {
        return true;
    }
    _PAINTED_PHRASES.iter().any(|p| padded.contains(p.as_str()))
}

/// How to start a hobby the line names.
pub fn _activity_start(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if dup::names_a_dish(understood).is_some() {
        return None;
    }
    let text_ = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    if _a_thing_is_being_baked(&text_) {
        return None;
    }
    let found = LETTER_RUN.findall_str(&text_);
    let padded = format!(" {} ", found.join(" "));
    let words: HashSet<String> = found.into_iter().collect();
    if _a_surface_is_being_painted(&words, &padded) {
        return None;
    }
    for (name, card) in ACTIVITY_START.iter() {
        if padded.contains(&format!(" {name} ")) {
            return Some(card.clone());
        }
    }
    for (name, activity) in ACTIVITY_VERBS.iter() {
        if words.contains(name) {
            if _FISH_VERBS.contains(name) && _FISH_IS_DINNER.is_match(&text_) {
                return None;
            }
            if _PAINT_VERBS.contains(name) && _paints_a_thing(&text_) {
                return None;
            }
            return Some(ACTIVITY_START.get(activity).expect("KeyError").clone());
        }
    }
    for (name, real) in ACTIVITY_OTHER_NAMES.iter() {
        if padded.contains(&format!(" {name} ")) {
            return Some(ACTIVITY_START.get(real).expect("KeyError").clone());
        }
    }
    None
}

/// Whether the painting is of a thing (a picture, a colour), not a pastime.
pub fn _paints_a_thing(text_: &str) -> bool {
    _PAINTED_THINGS.is_match(text_) || _PAINT_AS_A_THING.is_match(text_) || _PAINT_COLOURS.is_match(text_)
}

strs!(ORDINARY_GENRES, GENRE_WORDS_AFTER, GENRE_NAMES_BEFORE, GENRE_NOT_DESCRIBED_BY);
maps!(GENRE_START);

/// Whether a genre name is an ordinary word in this line ("house", "country", "soul").
pub fn _an_ordinary_word(name: &str, text_: &str) -> bool {
    if !ORDINARY_GENRES.iter().any(|g| g == name) {
        return false;
    }
    for hit in rx(format!("\\b{name}\\b")).finditer(text_) {
        let after = text::split_ref(&text_[hit.end()..]);
        let follows = after.first().map(|w| text::strip_chars(w, ",.!?;:")).unwrap_or("");
        let before = text::split_ref(&text_[..hit.start()]);
        if GENRE_WORDS_AFTER.iter().any(|w| w == follows) {
            continue;
        }
        if let Some(last) = before.last() {
            if GENRE_NAMES_BEFORE.iter().any(|w| w == text::strip_chars(last, ",.!?;:")) {
                return true;
            }
        }
        if ["and", "or", "vs", "versus", "is"].contains(&follows) || GENRE_NOT_DESCRIBED_BY.iter().any(|w| w == follows) {
            continue;
        }
        if !follows.is_empty() {
            return true;
        }
    }
    false
}

static MUSIC_PAIRS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    g("MUSIC_PAIRS").items().iter().map(|row| (row.items()[0].strs(), row.items()[1].str().to_string())).collect()
});

/// Where to start with a music genre, or how two genres differ.
pub fn _genre_start(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    if text_.contains("countryside") || text_.contains("village") || text_.contains("walk in the country") {
        return None;
    }
    let comparing = ["difference", "different", "differ", " vs ", "versus", "compared", "compare"].iter().any(|w| text_.contains(w));
    if comparing {
        for (pair, difference) in MUSIC_PAIRS.iter() {
            if pair.iter().all(|n| text_.contains(n.as_str())) {
                return Some(difference.clone());
            }
        }
    }
    for name in [
        "hip hop", "k-pop", "kpop", "r&b", "house music", "dance music", "electronic", "techno", "classical", "country",
        "metal", "motown", "soul", "blues", "folk", "reggae", "opera", "punk", "indie", "gospel", "disco", "jazz", "rap",
        "house",
    ] {
        if let Some(card) = GENRE_START.get(name) {
            if text_.contains(name) {
                let typed = [&understood.clean, &understood.norm_text]
                    .into_iter()
                    .find(|s| !s.is_empty())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_else(|| text_.clone());
                if _an_ordinary_word(name, &typed) {
                    return None;
                }
                return Some(card.clone());
            }
        }
    }
    None
}

maps!(CARD_GAMES, IN_TUNE);

/// The rules of a card game the line names.
pub fn _card_game(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    for (name, game) in [
        ("uno", "uno"), ("solitaire", "solitaire"), ("patience", "solitaire"), ("rummy", "rummy"), ("snap", "snap"),
        ("go fish", "go fish"), ("blackjack", "blackjack"), ("pontoon", "blackjack"), ("twenty one", "blackjack"),
    ] {
        if text_.contains(name) {
            return CARD_GAMES.get(game).cloned();
        }
    }
    None
}

/// How to tune the instrument the line names.
pub fn _in_tune(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    let card = |k: &str| Some(IN_TUNE.get(k).expect("KeyError").clone());
    if text_.contains("piano") || text_.contains("keyboard") {
        return card("piano");
    }
    for name in ["guitar", "bass", "ukulele", "violin", "fiddle", "cello", "banjo", "mandolin"] {
        if text_.contains(name) {
            if name == "bass" {
                return card("bass");
            }
            if ["violin", "fiddle", "cello"].contains(&name) {
                return card("violin");
            }
            return card(if name == "banjo" || name == "mandolin" { "guitar" } else { name });
        }
    }
    None
}

/// Why a guitar string buzzes.
pub fn _buzz(_memory: &mut Memory, _understood: &Understood) -> Option<String> {
    Some(
        "One buzzing string is usually that string: old, kinked, or sitting too low in its nut, so \
         change it first and see. Buzz that follows one fret up and down the neck is that fret or \
         the neck; buzz on every note is usually how hard you are picking, so try a lighter touch \
         and press just behind the fret rather than on top of it. If it still rattles in the same \
         place with a new string, the action is too low or the neck has moved - a shop sets that up \
         for about forty pounds, and it is worth it, because no amount of technique fixes a guitar \
         set up badly."
            .into(),
    )
}

/// The verdict on a work the table knows.
pub fn _worth_it(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = or(&understood.clean, &understood.norm_text).to_lowercase();
    if _about_a_person(&text_) || _a_place_not_a_work(&text_) {
        return None;
    }
    let title = _work_named(&text_);
    if title.is_empty() { None } else { Some(VIEWS[&title].clone()) }
}

strs!(NOT_A_VERDICT, JUDGE_ASKS);
res!(A_PERSON_TO_CONSULT);

/// An honest "I can't call it" for a verdict asked on a work the table does not know.
pub fn _worth_it_unnamed(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = or(&understood.clean, &understood.norm_text).to_lowercase();
    if _about_a_person(&text_) || !_work_named(&text_).is_empty() || _a_place_not_a_work(&text_) {
        return None;
    }
    if NOT_A_VERDICT.iter().any(|w| text_.contains(w.as_str())) {
        return None;
    }
    if A_PERSON_TO_CONSULT.is_match(&text_) {
        return None;
    }
    if !JUDGE_ASKS.iter().any(|p| text_.contains(p.as_str())) {
        return None;
    }
    if let Some(named) = _named_work(&text_) {
        return Some(format!(
            "I don't know *{named}* well enough to call it honestly. Tell me what you liked about the \
             last thing you enjoyed and I'll say whether it's that kind."
        ));
    }
    Some(
        "Not one I can call honestly, sorry. Name the last thing you enjoyed and I'll say whether \
         this one is that kind of thing."
            .into(),
    )
}

lazy_re!(NAMED_WORK_IS, "\\b(?:is|are|was)\\s+((?:the\\s+|this\\s+|that\\s+)?[a-z0-9'&: -]{2,40})");
lazy_re!(NAMED_WORK_VERB, "\\b(?:watch|watching|read|reading|see|play)\\s+((?:the\\s+|a\\s+)?[a-z0-9'&: -]{2,40})");

/// The title a verdict line names, as printed; None when it names none.
pub fn _named_work(text_: &str) -> Option<String> {
    const CUTS: &[&str] = &[
        "worth", "any good", "good", "boring", "like", "as good", "better", "any cop", "in the", "or", "on", "at",
        "tonight", "with", "because", "before", "after", "if", "but", "for me", "or not", "yet", "then", "my", "your",
        "his", "her", "our", "their",
    ];
    for pattern in [&*NAMED_WORK_IS, &*NAMED_WORK_VERB] {
        let Some(found) = pattern.search(text_) else { continue };
        let mut title = found.str(1).to_string();
        for cut in CUTS {
            title = rx(format!("\\b{}\\b", pyre::escape(cut))).split(&title).into_iter().next().unwrap_or_default();
        }
        let title = text::strip_chars(&title, " .?!,-\t");
        if title.is_empty() || ["it", "this", "that", "something", "anything", "one", "worth", "the", "a", "an"].contains(&title) {
            continue;
        }
        return Some(_display(title));
    }
    None
}

strs!(WORK_EVIDENCE, NEXT_EVIDENCE);
sets!(PLAIN_TITLES);
list_maps!(PLAIN_SENSE);

/// The "after this, watch" row for a film or series the text names, or "".
pub fn _table_pick(text_: &str) -> String {
    let about_a_work = WORK_EVIDENCE.iter().any(|w| text_.contains(w.as_str())) || NEXT_EVIDENCE.iter().any(|p| text_.contains(p.as_str()));
    for (title, reply) in FILM_MATCHES.iter() {
        if !rx(format!("\\b{}\\b", pyre::escape(title))).is_match(text_) {
            continue;
        }
        if PLAIN_TITLES.contains(title) {
            if PLAIN_SENSE.get(title).expect("KeyError").iter().any(|p| text_.contains(p.as_str())) {
                continue;
            }
            if !about_a_work {
                continue;
            }
        }
        return reply.clone();
    }
    String::new()
}

/// What to watch after a named film or series.
pub fn _after_named(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    Some(_table_pick(&text_))
}

// ---------------------------------------------------------------------------------------------
// Books on a subject; reservations; listings near me; a book to read

/// Whether the phrase stands as its own word(s) in the text.
pub fn _says(phrase: &str, text_: &str) -> bool {
    rx(format!("(?<![a-z]){}(?![a-z])", pyre::escape(phrase))).is_match(text_)
}

res!(_BOOK_ANCHOR, _BOOK_SUBJECT_BEFORE);
maps!(SUBJECT_BOOKS, SUBJECT_BOOK_NAMES);
strs!(_SUBJECT_BOOK_ORDER, _SUBJECT_UNCOVERED_ORDER);

/// The book row for the subject the line asks about, and whether a subject was named at all.
pub fn _book_subject(text_: &str) -> (Option<String>, bool) {
    let tail = match _BOOK_ANCHOR.search(text_) {
        Some(m) => m.str(1).to_string(),
        None => match _BOOK_SUBJECT_BEFORE.search(text_) {
            Some(m) => m.str(1).to_string(),
            None => return (None, false),
        },
    };
    for phrase in _SUBJECT_BOOK_ORDER.iter() {
        if _says(phrase, &tail) {
            let name = SUBJECT_BOOK_NAMES.get(phrase).expect("KeyError");
            return (Some(SUBJECT_BOOKS.get(name).expect("KeyError").clone()), true);
        }
    }
    for phrase in _SUBJECT_UNCOVERED_ORDER.iter() {
        if _says(phrase, &tail) {
            return (None, true);
        }
    }
    (None, false)
}

/// A thriller to read.
pub fn _book_thriller(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text_ = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    if text_.contains("thursday murder club") {
        return Some(
            "The next one is *The Man Who Died Twice* by Richard Osman, the same four friends again. \
             For a different author, *The Marlow Murder Club* by Robert Thorogood is close in tone, or \
             *The Silent Patient* if you want a darker twist."
                .into(),
        );
    }
    Some(
        "*The Silent Patient* by Alex Michaelides and *The Thursday Murder Club* by Richard Osman \
         are both quick and hard to put down; the second one is much funnier. Tana French's *In the \
         Woods* if you want something longer and darker."
            .into(),
    )
}

/// A book about the subject the line names.
pub fn _book_about(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !_a_genre_is_the_taste(understood) {
        return None;
    }
    let text_ = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    _book_subject(&text_).0
}

/// The book rows answer only a line that does not book a table or a ticket.
pub fn _not_a_reservation(understood: &Understood) -> bool {
    if _a_judged_work_named(understood) {
        return false;
    }
    !the_book_is_a_reservation(understood)
}

strs!(JUDGE_WORDS);

/// Whether the line judges a work the verdict table knows ("is the office worth it").
pub fn _a_judged_work_named(understood: &Understood) -> bool {
    let text_ = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    if _work_named(&text_).is_empty() {
        return false;
    }
    JUDGE_WORDS.iter().any(|w| _says(w, &text_))
}

/// Whether "book" in the line is booking (a table, tickets), not reading.
pub fn the_book_is_a_reservation(understood: &Understood) -> bool {
    [&understood.clean, &understood.norm_text].iter().any(|r| _a_reservation(&r.to_lowercase()))
}

strs!(_RESERVING_PHRASES);
res!(_BOOK_VERB_BEFORE, _BOOK_VERB_AFTER, _A_DETERMINER_JUST_BEFORE);

/// Whether this text books something.
pub fn _a_reservation(text_: &str) -> bool {
    if text::strip(text_).is_empty() {
        return false;
    }
    if _RESERVING_PHRASES.iter().any(|p| _says(p, text_)) {
        return true;
    }
    if _BOOK_VERB_BEFORE.is_match(text_) {
        return true;
    }
    _BOOK_VERB_AFTER.finditer(text_).iter().any(|m| !_A_DETERMINER_JUST_BEFORE.is_match(&text_[..m.start()]))
}

res!(_NEAR_ME, _ASKS_IF_OPEN, _SAYS_WHAT_IS_ON);

/// False when the line asks what is on near them (a listing the bot cannot see).
pub fn _a_listings_ask_near_me(understood: &Understood) -> bool {
    let text_ = text::squash(&understood.clean.to_lowercase());
    if !_NEAR_ME.is_match(&text_) {
        return true;
    }
    if _ASKS_IF_OPEN.is_match(&text_) {
        return false;
    }
    _SAYS_WHAT_IS_ON.is_match(&text_)
}

res!(_A_BOOK_I_AM_NOT_ENJOYING, _ASKS_FOR_A_PICK);

/// A book to read next, by who it is for.
pub fn _book_recommend(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let earlier = last_turns(memory, 3).iter().map(|t| t.user.clone()).collect::<Vec<_>>().join(" ");
    let said = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    let (pick, named) = _book_subject(&said);
    if named {
        return pick;
    }
    if _A_BOOK_I_AM_NOT_ENJOYING.is_match(&said) && !_ASKS_FOR_A_PICK.is_match(&said) {
        return Some(
            "Put it down - finishing a book you are not enjoying is not a virtue, and 50 pages is a \
             fair trial before you give up on it. If you want another: *The Thursday Murder Club* by \
             Richard Osman is funny with short chapters, *The Silent Patient* if you want a twist, and \
             *The Midnight Library* by Matt Haig if you want something to think about. What did the \
             last one you loved have that this one has not?"
                .into(),
        );
    }
    let mut context = format!("{} {}", understood.clean, earlier).to_lowercase();
    let named = _book_title_named(&context);
    let young = ["son", "daughter", "child", "kid", "year old", "grandson", "granddaughter", "niece", "nephew", "teenager"];
    if !named.is_empty() {
        context = context.replace(named.as_str(), " ");
        if !young.iter().any(|w| context.contains(w)) {
            return _book_like(memory, understood);
        }
    }
    if ["baby", "babies", "toddler", "one year old", "two year old", "18 months", "six months", "nine months", "newborn"]
        .iter()
        .any(|w| context.contains(w))
    {
        return Some(
            "For a baby, board books that survive being chewed and pulled: *The Very Hungry \
             Caterpillar* by Eric Carle, *Dear Zoo* by Rod Campbell, *Where's Spot?* by Eric Hill, or \
             any of the *That's Not My...* touchy-feely books. Read the same three or four over and \
             over - the repetition is the point at that age - and let them turn the pages, even if it \
             slows you down."
                .into(),
        );
    }
    if ["my class", "the class", "assembly", "story time", "storytime", "reception class", "nursery"].iter().any(|w| context.contains(w)) {
        return Some(
            "For reading aloud to a group: *The Gruffalo* by Julia Donaldson for the rhymes, *The \
             Tiger Who Came to Tea* by Judith Kerr for the pause at the doorbell, and *Oi Frog!* by \
             Kes Gray for the joke on every page. *The Gruffalo* is the safest one for a mixed group: \
             the rhythm carries it and there is a repeated line they can join in with."
                .into(),
        );
    }
    if young.iter().any(|w| context.contains(w)) {
        return Some(
            "For a younger reader: *The Bolds* by Julian Clary (funny on every page), *The Worst \
             Witch* (short chapters), *The Explorer* by Katherine Rundell for animals and adventure, \
             or *Dog Man* if comics are easier. Start with *The Bolds* or *Dog Man*: jokes first, and \
             no long descriptions to get through."
                .into(),
        );
    }
    Some(
        "*The Thursday Murder Club* by Richard Osman is the safest pick: funny, short chapters, \
         easy to pick up. For something darker, *The Silent Patient*; for a younger reader, \
         *Wonder* by R. J. Palacio or *The Explorer* by Katherine Rundell. Tell me what you last \
         enjoyed and I'll narrow it down."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// Made-up things: a haiku, a limerick, a poem, a story, a rap, a song

strs!(POSSESSIVE);

/// "my nan's garden" said back as the person said it, when the thing comes right after a
/// possessive; "" otherwise.
pub fn _the_possessive_phrase(thing: &str, said: &str) -> String {
    if said.is_empty() || thing.is_empty() {
        return String::new();
    }
    for word in POSSESSIVE.iter() {
        let step = word.chars().next().map(char::len_utf8).unwrap_or(1);
        let mut start = 0;
        while start <= said.len() {
            let Some(at) = said.get(start..).and_then(|s| s.find(word.as_str())).map(|i| i + start) else { break };
            let rest = &said[at + word.len()..];
            if let Some(spot) = rest.find(thing) {
                if text::split_ref(&rest[..spot]).len() <= 2 {
                    let end = at + word.len() + spot + thing.len();
                    return text::squash(&said[at..end]);
                }
            }
            start = at + step;
        }
    }
    String::new()
}

/// The article forms a template fills in: `{a_thing}`, `{the_thing}`, `{The_thing}`.
pub struct ArticleForms {
    pub a_thing: String,
    pub the_thing: String,
    pub the_thing_upper: String,
}

/// The thing with its article, or with the person's own possessive when they said one.
pub fn _article_for(thing: &str, article: &str, clean: &str) -> ArticleForms {
    let said = clean.to_lowercase();
    let theirs = _the_possessive_phrase(thing, &said);
    if !theirs.is_empty() {
        return ArticleForms { a_thing: theirs.clone(), the_thing: theirs.clone(), the_thing_upper: text::upper_first(&theirs) };
    }
    let the = format!("{article} {thing}");
    ArticleForms { a_thing: format!("a {thing}"), the_thing_upper: text::upper_first(&the), the_thing: the }
}

/// A template filled with the thing and its article forms.
fn fill_thing(template: &str, thing: &str, forms: &ArticleForms) -> String {
    format_named(
        template,
        &[("thing", thing), ("a_thing", &forms.a_thing), ("the_thing", &forms.the_thing), ("The_thing", &forms.the_thing_upper)],
    )
}

sets!(SUBJECT_END, CONNECTORS, SUBJECT_TAIL, ING_NOUNS, SUBJECT_STOP, EMPTY_SUBJECT, WHOM_IT_IS_FOR, S_FINAL_SINGULARS);

/// The words of a subject after "about": cut at its end, at most three, no trailing filler.
pub fn _subject_words(text_: &str) -> Vec<String> {
    let mut words = LETTER_RUN.findall_str(text_);
    if let Some(place) = words.iter().position(|w| SUBJECT_END.contains(w)) {
        words.truncate(place);
    }
    if let Some(place) = words.iter().enumerate().position(|(i, w)| i > 0 && CONNECTORS.contains(w)) {
        words.truncate(place);
    }
    words.truncate(3);
    while words.last().is_some_and(|w| SUBJECT_TAIL.contains(w) || CONNECTORS.contains(w)) {
        words.pop();
    }
    if words.len() > 1 {
        let last = words.last().unwrap();
        if last.ends_with("ing") && last.chars().count() > 4 && !ING_NOUNS.contains(last)
            && words[..words.len() - 1].iter().any(|w| !SUBJECT_STOP.contains(w) && !EMPTY_SUBJECT.contains(w))
        {
            words.pop();
        }
    }
    words
}

list_maps!(NO_SUBJECT_VERSE, VERSE_FORM_WORDS);
strs!(_OTHER_FORM_WORDS);

/// A verse with no subject, for a line that asks for one and names nothing.
pub fn _no_subject_verse(form: &str, noun: Option<&str>) -> Option<String> {
    let lines = NO_SUBJECT_VERSE.get(form).filter(|l| !l.is_empty())?;
    let mut lead = match form {
        "haiku" => "A haiku, then:",
        "limerick" => "A limerick, then:",
        _ => "A short poem, then:",
    };
    if noun == Some("rhyme") {
        lead = "A rhyme, then:";
    }
    Some(_lines(lines, Some(lead)))
}

/// Whether the line names a verse form other than this one.
pub fn _names_another_form(clean: &str, form: &str) -> bool {
    static NONE: Vec<String> = Vec::new();
    let mine: &Vec<String> = VERSE_FORM_WORDS.get(form).unwrap_or(&NONE);
    for word in LETTER_RUN.findall_str(clean) {
        if mine.contains(&word) {
            continue;
        }
        if _OTHER_FORM_WORDS.contains(&word) {
            return true;
        }
        for (name, others) in VERSE_FORM_WORDS.iter() {
            if name != form && others.contains(&word) {
                return true;
            }
        }
    }
    false
}

lazy_re!(
    HOW_TO_WRITE,
    "\\b(?:how do i|how to|how can i|how should i|tips for|advice on|writes|wrote|writing|written|reads|reading)\\b"
);
lazy_re!(ABOUT_FOR, "\\b(?:about|of|on|regarding|describing|named|called|for)\\s+(.+)$");
lazy_re!(ABOUT_SUBJECT, "\\b(?:about|of|on|regarding|describing|for|named|called)\\s+(.+)$");

/// The subject-less verse for a line that asks for one and names no subject.
pub fn _asks_for_a_verse(clean: &str, form: &str, noun: Option<&str>) -> Option<String> {
    if _names_another_form(clean, form) {
        return None;
    }
    if HOW_TO_WRITE.is_match(clean) {
        return None;
    }
    if let Some(found) = ABOUT_FOR.search(clean) {
        let words = _subject_words(found.str(1));
        if !words.is_empty() && !(words.len() == 1 && WHOM_IT_IS_FOR.contains(&words[0])) {
            return None;
        }
    }
    _no_subject_verse(form, noun)
}

/// The subject after "about" (up to three words), or "".
pub fn _subject(clean: &str) -> String {
    let Some(found) = ABOUT_SUBJECT.search(clean) else { return String::new() };
    let words = _subject_words(found.str(1));
    if !words.is_empty() && words.iter().all(|w| EMPTY_SUBJECT.contains(w)) {
        return String::new();
    }
    words.join(" ")
}

/// The one thing the subject names, singular ("my cats" -> "cat").
pub fn _thing(clean: &str) -> String {
    let Some(found) = ABOUT_SUBJECT.search(clean) else { return String::new() };
    let mut words = _subject_words(found.str(1));
    if words.is_empty() {
        return String::new();
    }
    let mut word = words.last().unwrap().clone();
    while CONNECTORS.contains(&word) && words.len() > 1 {
        words.pop();
        word = words.last().unwrap().clone();
    }
    if let Some(w) = word.strip_suffix("'s") {
        word = w.to_string();
    }
    while SUBJECT_STOP.contains(&word) && words.len() > 1 {
        words.pop();
        word = words.last().unwrap().clone();
        if let Some(w) = word.strip_suffix("'s") {
            word = w.to_string();
        }
    }
    if word.ends_with("ies") && word.chars().count() > 4 {
        return format!("{}y", &word[..word.len() - 3]);
    }
    if word.ends_with('s')
        && !(word.ends_with("ss") || word.ends_with("is") || word.ends_with("us"))
        && !S_FINAL_SINGULARS.contains(&word)
        && word.chars().count() > 3
    {
        return word[..word.len() - 1].to_string();
    }
    word
}

/// Whether the line says "about" and then names only empty words ("about something").
pub fn _named_nothing(clean: &str) -> bool {
    let Some(found) = ABOUT_SUBJECT.search(clean) else { return false };
    let words = _subject_words(found.str(1));
    !words.is_empty() && words.iter().all(|w| EMPTY_SUBJECT.contains(w))
}

/// The lines of a verse, under a lead line when there is one.
pub fn _lines<S: AsRef<str>>(lines: &[S], lead: Option<&str>) -> String {
    let body = lines.iter().map(|l| l.as_ref()).collect::<Vec<_>>().join("\n");
    match lead {
        Some(lead) if !lead.is_empty() => format!("{lead}\n{body}"),
        _ => body,
    }
}

/// A haiku about the line's subject (the build behind the `_haiku` writer).
pub fn _haiku(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.as_str();
    if _a_report_of_the_habit(clean) {
        return _a_report_answered(clean, Some(understood));
    }
    let subject = _subject(clean);
    if subject.is_empty() {
        return _asks_for_a_verse(clean, "haiku", None);
    }
    _say("haiku", &_thing(clean), &subject, None)
}

/// A limerick about the line's subject (the build behind the `_limerick` writer).
pub fn _limerick(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.as_str();
    if _a_report_of_the_habit(clean) {
        return _a_report_answered(clean, Some(understood));
    }
    let subject = _subject(clean);
    if subject.is_empty() {
        return _asks_for_a_verse(clean, "limerick", None);
    }
    _say("limerick", &_thing(clean), &subject, None)
}

/// A short poem (or rhyme) about the line's subject (the build behind the `_poem` writer).
pub fn _poem(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.as_str();
    if _a_report_of_the_habit(clean) {
        return _a_report_answered(clean, Some(understood));
    }
    let noun = if clean.contains("rhyme") { "rhyme" } else { "poem" };
    let subject = _subject(clean);
    if subject.is_empty() {
        return _asks_for_a_verse(clean, "poem", Some(noun));
    }
    _say("poem", &_thing(clean), &subject, Some(noun))
}

list_maps!(RHYMES_BY_ENDING);
maps!(NO_RHYME);
lazy_re!(RHYME_WITH, "rhyme\\w*\\s+(?:with|for|to)\\s+([a-z']+)");

/// Words that rhyme with the one the line names.
pub fn _rhyme(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.as_str();
    let found = RHYME_WITH.search(clean)?;
    let word = found.str(1);
    if let Some(none) = NO_RHYME.get(word) {
        return Some(none.clone());
    }
    let bare = if word.ends_with('e') && word.chars().count() > 3 { &word[..word.len() - 1] } else { word };
    let mut endings: Vec<&String> = RHYMES_BY_ENDING.keys().collect();
    endings.sort_by_key(|e| std::cmp::Reverse(e.chars().count()));
    for ending in endings {
        if word.ends_with(ending.as_str()) || bare.ends_with(ending.as_str()) {
            let picks: Vec<String> = RHYMES_BY_ENDING[ending].iter().filter(|one| *one != word).take(5).cloned().collect();
            if !picks.is_empty() {
                return Some(format!(
                    "Words that rhyme with *{word}*: {}. For a poem, the easiest thing is to put *{word}* at \
                     the end of a line and pick the rhyme that says what you mean.",
                    _list_words(&picks)
                ));
            }
        }
    }
    None
}

strs!(STORY_IDEAS);

/// Two story ideas to start from.
pub fn _story_idea(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.as_str();
    let mut seed = ord_sum(clean);
    if seed == 0 {
        seed = 1;
    }
    let n = STORY_IDEAS.len() as i64;
    let first = &STORY_IDEAS[(seed % n) as usize];
    let second = &STORY_IDEAS[((seed + 7) % n) as usize];
    Some(format!(
        "Two to start with:\n- {first}\n- {second}\nWhich one sounds like yours? I can help you find the \
         first scene, or the thing that goes wrong."
    ))
}

res!(_A_VERSE_LEAD);

/// The verse form the last turn or two wrote or asked for.
pub fn _the_form_last_time(memory: &Memory) -> Option<String> {
    for turn in last_turns(memory, 2).iter().rev() {
        let bot = turn.bot.as_str();
        if let Some(written) = _A_VERSE_LEAD.match_(bot) {
            return Some(written.str(1).to_lowercase());
        }
        let said = format!("{} {}", turn.user, bot).to_lowercase();
        for form in ["haiku", "limerick", "poem"] {
            if said.contains(form) {
                return Some(form.to_string());
            }
        }
    }
    None
}

/// The lead line of a verse about a subject.
pub fn _lead(form: &str, subject: &str) -> String {
    match form {
        "haiku" => format!("A haiku about {subject}, 5-7-5:"),
        "limerick" => format!("A limerick about {subject}:"),
        _ => format!("A short poem about {subject}:"),
    }
}

/// The offer to write one round a real detail, for an abstract subject.
pub fn _offer(form: &str, subject: &str) -> String {
    let name = match form {
        "haiku" | "limerick" | "rap" | "song" | "story" => form,
        _ => "poem",
    };
    format!(
        "A {name} about {subject} needs one real thing in it: give me a sound, a smell or a day from it \
         and I'll build the {name} round that."
    )
}

static FORMS: LazyLock<IndexMap<String, IndexMap<String, Vec<String>>>> = LazyLock::new(|| {
    g("FORMS")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.pairs().iter().map(|(a, b)| (a.str().to_string(), b.strs())).collect()))
        .collect()
});
list_maps!(HAIKU, POEMS);
strs!(PEOPLE, ABSTRACT, WARM_POEM, GENERIC_POEM, GENERIC_LIMERICK, GENERIC_POEM_TWO);

fn with_thing(lines: &[String], thing: &str) -> Vec<String> {
    lines.iter().map(|one| format_named(one, &[("thing", thing)])).collect()
}

/// The verse for a thing in a form: a known one, a warm one for a person, or the generic one.
pub fn _say(form: &str, thing: &str, subject: &str, noun: Option<&str>) -> Option<String> {
    let form = noun.filter(|n| !n.is_empty()).unwrap_or(form);
    let in_form = FORMS.get(form).is_some_and(|f| f.contains_key(thing));
    let known = in_form || POEMS.contains_key(thing) || PEOPLE.iter().any(|p| p == thing);
    let last_abstract = text::split_ref(subject).last().is_some_and(|w| ABSTRACT.iter().any(|a| a == w));
    if !known && (ABSTRACT.iter().any(|a| a == thing) || last_abstract) {
        return Some(_offer(form, subject));
    }
    if form == "haiku" {
        return HAIKU.get(thing).filter(|f| !f.is_empty()).map(|found| _lines(found, Some(&_lead(form, subject))));
    }
    let Some(table) = FORMS.get(form) else {
        let lead = format!("A rhyme about {subject}:");
        if let Some(thing_line) = POEMS.get(thing).filter(|l| !l.is_empty()) {
            return Some(_lines(thing_line, Some(&lead)));
        }
        if PEOPLE.iter().any(|p| p == thing) {
            return Some(_lines(&WARM_POEM, Some(&lead)));
        }
        return Some(_lines(&with_thing(&GENERIC_POEM, thing), Some(&lead)));
    };
    if let Some(lines) = table.get(thing) {
        return Some(_lines(lines, Some(&_lead(form, subject))));
    }
    if form != "limerick" && PEOPLE.iter().any(|p| p == thing) {
        return Some(_lines(&WARM_POEM, Some(&_lead(form, subject))));
    }
    if form == "limerick" {
        return Some(_lines(&with_thing(&GENERIC_LIMERICK, thing), Some(&_lead(form, subject))));
    }
    Some(_lines(&with_thing(&GENERIC_POEM, thing), Some(&_lead(form, subject))))
}

/// One of the choices, picked by the sum of the line's code points.
pub fn _picked<'a>(choices: &'a [String], clean: &str) -> &'a str {
    let place = if clean.is_empty() { 0 } else { (ord_sum(clean) % choices.len() as i64) as usize };
    &choices[place]
}

lazy_re!(LISTENER_AGE, "\\b(\\d+|one|two|three|four|five|six|seven|eight|nine|ten)\\b");
strs!(CHILD_STORIES, CHILD_STORY_OPENINGS);

/// "A short one for a 5 year old", by the age the line gives.
pub fn _the_listener_lead(clean: &str) -> String {
    match LISTENER_AGE.search(clean) {
        None => "A short one for a child".into(),
        Some(age) => format!("A short one for a {} year old", age.str(1)),
    }
}

/// A child's story for the listener the line names.
pub fn _a_story_for_the_listener(clean: &str) -> String {
    format!("{}: {}", _the_listener_lead(clean), _picked(&CHILD_STORIES, clean))
}

/// A child's story about the thing the line names.
pub fn _a_child_story_about(clean: &str, thing: &str) -> String {
    let opening = _picked(&CHILD_STORY_OPENINGS, &format!("{clean}{thing}"));
    format!("{}: {}", _the_listener_lead(clean), format_named(opening, &[("thing", thing)]))
}

res!(ASKS_FOR_A_STORY, ASKS_THE_BOT, A_STORY_ALREADY_HEARD, A_NEW_ONE_ASKED_FOR, A_STORY_ALREADY_TOLD);

/// Whether the line asks the bot for a story (not about one it heard).
pub fn _a_story_is_asked_for(clean: &str) -> bool {
    if !ASKS_FOR_A_STORY.is_match(clean) {
        return false;
    }
    if ASKS_THE_BOT.is_match(clean) {
        return true;
    }
    if A_STORY_ALREADY_HEARD.is_match(clean) {
        return false;
    }
    if A_NEW_ONE_ASKED_FOR.is_match(clean) {
        return true;
    }
    !A_STORY_ALREADY_TOLD.is_match(clean)
}

res!(_SECOND_STORY);

/// The typed line when it asks for a second story in words the clean reading lost.
pub fn _the_line_to_read_for_a_story(understood: &Understood) -> String {
    let typed = text::strip(&understood.text);
    let clean = understood.clean.as_str();
    if !typed.is_empty() && typed != clean && _SECOND_STORY.is_match(&typed.to_lowercase()) {
        return typed.to_string();
    }
    clean.to_string()
}

lazy_re!(STORY_WORD_I, "\\bstory\\b", pyre::I);

/// Whether the bot's last reply was a story.
pub fn _the_story_was_just_told(memory: &Memory) -> bool {
    for turn in memory.turns.iter().rev() {
        if !turn.bot.is_empty() {
            return STORY_WORD_I.is_match(&turn.bot);
        }
    }
    false
}

sets!(_PLACEHOLDER_SUBJECT);

/// Whether the subject is only a placeholder ("something", "anything").
pub fn _a_placeholder_word(subject: &str) -> bool {
    match text::split_ref(subject).first() {
        None => true,
        Some(w) => _PLACEHOLDER_SUBJECT.contains(&text::strip_chars(w, ".,!?;:'\"").to_lowercase()),
    }
}

res!(REPORTS_ON_A_FORM, ASKS_FOR_ONE);

/// Whether the line reports a story or song (heard, sung) rather than asking for one.
pub fn _a_report_of_the_habit(clean: &str) -> bool {
    REPORTS_ON_A_FORM.is_match(clean) && !ASKS_FOR_ONE.is_match(clean)
}

/// Whether the person's last line was such a report.
pub fn _the_last_line_reported_a_form(memory: &Memory) -> bool {
    let turns = &memory.turns;
    for (i, turn) in turns.iter().enumerate().rev() {
        if turn.user.is_empty() {
            continue;
        }
        if text::strip(&turn.bot).is_empty() && i + 1 == turns.len() {
            continue;
        }
        return _a_report_of_the_habit(&turn.user);
    }
    false
}

res!(A_QUESTION_BESIDE_THE_REPORT, A_READING_ROUTINE, A_MAKING_UP, A_REPEATED_REPORT);
list_maps!(REPORT_ONCE_BACK, REPORT_BACK);
strs!(REPORT_ONCE_GENERIC);
static _A_FORM_IN_THE_REPORT: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    g("_A_FORM_IN_THE_REPORT").items().iter().map(|row| (row.items()[0].str().to_string(), row.items()[1].as_re().unwrap())).collect()
});

/// The words back for a reported story or song.
pub fn _a_report_answered(clean: &str, _understood: Option<&Understood>) -> Option<String> {
    if !_a_report_of_the_habit(clean) {
        return None;
    }
    let line = clean;
    if A_QUESTION_BESIDE_THE_REPORT.is_match(line) {
        return None;
    }
    if A_READING_ROUTINE.is_match(line) && !A_MAKING_UP.is_match(line) {
        return None;
    }
    let form = _A_FORM_IN_THE_REPORT.iter().find(|(_, shape)| shape.is_match(line)).map(|(n, _)| n.as_str()).unwrap_or("story");
    if !A_REPEATED_REPORT.is_match(line) {
        if let Some(once) = REPORT_ONCE_BACK.get(form).filter(|o| !o.is_empty()) {
            return Some(_picked(once, line).to_string());
        }
        return Some(percent_s(_picked(&REPORT_ONCE_GENERIC, line), form));
    }
    Some(_picked(REPORT_BACK.get(form).expect("KeyError"), line).to_string())
}

res!(_A_STORY_WORD, _ASKS_ABOUT_A_STORY, _A_LISTENER_IS_NAMED);
strs!(HEARD_IT_ALREADY);
text_of!(GENERIC_STORY_THING);
maps!(STORY_LEADING, NO_SUBJECT_LEADING, NO_THING_IN_A_VERSE);
lazy_re!(STORY_ABOUT_NAMED, "\\b(?:about|of|on|regarding)\\s+(?P<subject>\\S.*)$");

/// A story made up on the spot (the build behind the `_story` writer).
pub fn _story(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = _the_line_to_read_for_a_story(understood);
    let clean = clean.as_str();
    if _a_report_of_the_habit(clean) {
        return _a_report_answered(clean, Some(understood));
    }
    if _the_last_line_reported_a_form(memory) && !ASKS_FOR_ONE.is_match(clean) && _SECOND_STORY.match_(text::strip(&understood.plain)).is_none() {
        return None;
    }
    let typed = or(&understood.plain, &understood.text);
    if !typed.is_empty() && typed != clean && !_A_STORY_WORD.is_match(typed) {
        if _SECOND_STORY.match_(text::strip(typed)).is_none() || !_the_story_was_just_told(memory) {
            return None;
        }
    }
    if !_A_STORY_WORD.is_match(clean) {
        if _SECOND_STORY.match_(text::strip(clean)).is_none() {
            return None;
        }
        if !_the_story_was_just_told(memory) {
            return None;
        }
    }
    if _ASKS_ABOUT_A_STORY.is_match(clean) {
        return None;
    }
    if A_STORY_ALREADY_HEARD.is_match(clean) && ASKS_FOR_A_STORY.is_match(clean) {
        return Some(_picked(&HEARD_IT_ALREADY, clean).to_string());
    }
    if !_a_story_is_asked_for(clean) {
        return None;
    }
    if _A_LISTENER_IS_NAMED.is_match(clean) {
        let named = STORY_ABOUT_NAMED.search(clean);
        let Some(named) = named.filter(|n| !_a_placeholder_word(n.name_str("subject"))) else {
            return Some(_a_story_for_the_listener(clean));
        };
        let child_thing = _thing(named.as_str());
        if !child_thing.is_empty() && !_a_placeholder_word(&child_thing) {
            return Some(_a_child_story_about(clean, &child_thing));
        }
        return Some(_a_story_for_the_listener(clean));
    }
    let mut thing = _thing(clean);
    if thing.is_empty() {
        thing = GENERIC_STORY_THING.clone();
    }
    if _a_placeholder_word(&thing) {
        thing = GENERIC_STORY_THING.clone();
    }
    let mut subject = Some(_subject(clean));
    if subject.as_deref().is_some_and(|s| !s.is_empty() && _a_placeholder_word(s)) {
        subject = None;
    }
    let subject = subject.filter(|s| !s.is_empty());
    if ABSTRACT.contains(&thing) {
        if let Some(subject) = &subject {
            return Some(_offer("story", subject));
        }
    }
    let lead = match &subject {
        Some(s) => format_named(STORY_LEADING.get("story").expect("KeyError"), &[("subject", s)]),
        None => "Here is one, made up on the spot:".to_string(),
    };
    let opening = _the_opening_not_said_yet(&thing, memory, clean);
    Some(format!("{lead}\n{opening}"))
}

strs!(STORY_OPENINGS);

/// A story opening the chat has not had yet.
pub fn _the_opening_not_said_yet(thing: &str, memory: &Memory, clean: &str) -> String {
    let n = STORY_OPENINGS.len();
    let start = if thing.is_empty() { 0 } else { (ord_sum(thing) % n as i64) as usize };
    let told = memory.turns.iter().map(|t| t.bot.as_str()).collect::<Vec<_>>().join(" ");
    let forms = _article_for(thing, "the", clean);
    for step in 0..n {
        let one = fill_thing(&STORY_OPENINGS[(start + step) % n], thing, &forms);
        let first = one.split('\n').next().unwrap_or("");
        if !told.contains(first) {
            return one;
        }
    }
    fill_thing(&STORY_OPENINGS[start], thing, &forms)
}

strs!(GENERIC_RAP);
text_of!(GENERIC_SONG);

/// A rap about the line's thing (the build behind the `_rap` writer).
pub fn _rap(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.as_str();
    if _a_report_of_the_habit(clean) {
        return _a_report_answered(clean, Some(understood));
    }
    if _named_nothing(clean) {
        return None;
    }
    let thing = Some(_thing(clean)).filter(|t| !t.is_empty()).unwrap_or_else(|| NO_THING_IN_A_VERSE.get("rap").expect("KeyError").clone());
    let subject = Some(_subject(clean)).filter(|s| !s.is_empty()).unwrap_or_else(|| thing.clone());
    if ABSTRACT.contains(&thing) {
        return Some(_offer("rap", &subject));
    }
    let forms = _article_for(&thing, "the", clean);
    let lines: Vec<String> = GENERIC_RAP.iter().map(|one| fill_thing(one, &thing, &forms)).collect();
    let lead = if !_subject(clean).is_empty() {
        format_named(STORY_LEADING.get("rap").expect("KeyError"), &[("subject", &subject)])
    } else {
        NO_SUBJECT_LEADING.get("rap").expect("KeyError").clone()
    };
    Some(_lines(&lines, Some(&lead)))
}

/// A song about the line's thing (the build behind the `_song` writer).
pub fn _song(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.as_str();
    if _a_report_of_the_habit(clean) {
        return _a_report_answered(clean, Some(understood));
    }
    if _named_nothing(clean) {
        return None;
    }
    let thing = Some(_thing(clean)).filter(|t| !t.is_empty()).unwrap_or_else(|| NO_THING_IN_A_VERSE.get("song").expect("KeyError").clone());
    let subject = Some(_subject(clean)).filter(|s| !s.is_empty()).unwrap_or_else(|| thing.clone());
    if ABSTRACT.contains(&thing) {
        return Some(_offer("song", &subject));
    }
    let lead = if !_subject(clean).is_empty() {
        format_named(STORY_LEADING.get("song").expect("KeyError"), &[("subject", &subject)])
    } else {
        NO_SUBJECT_LEADING.get("song").expect("KeyError").clone()
    };
    let forms = _article_for(&thing, "the", clean);
    Some(format!("{lead}\n{}", fill_thing(&GENERIC_SONG, &thing, &forms)))
}

strs!(CHARACTER_NAME_RULES, NICK_FIRST, NICK_LAST, QUIZ_NAMES, TEAM_NAMES);
lazy_re!(VILLAIN_OR_HERO, "\\b(?:villain|villainess|baddie|antagonist|hero|heroine)\\b", pyre::I);

/// How to name a character.
pub fn _character_name(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.as_str();
    if VILLAIN_OR_HERO.is_match(clean) {
        return Some(CHARACTER_NAME_RULES[1].clone());
    }
    Some(_picked(&CHARACTER_NAME_RULES, clean).to_string())
}

/// Three nicknames, picked from the line.
pub fn _nickname(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let seed = ord_sum(&understood.clean);
    let picks: Vec<String> = (0..3i64)
        .map(|step| {
            let first = &NICK_FIRST[((seed + step * 3) % NICK_FIRST.len() as i64) as usize];
            let last = &NICK_LAST[((seed / 7 + step * 5) % NICK_LAST.len() as i64) as usize];
            format!("{first} {last}")
        })
        .collect();
    Some(format!(
        "Start with *{}*; *{}* and *{}* if it sticks. A nickname works best when it comes from \
         something only you two know.",
        picks[0], picks[1], picks[2]
    ))
}

res!(_NOT_A_VERSE_ASK, _A_SECOND_ONE);

/// What `_the_second_one_asked_for` found: no ask for another one (Python's ""), an ask with no
/// subject (None), or an ask for another one about something.
#[derive(Clone, Debug, PartialEq)]
pub enum SecondOne {
    NotAsked,
    Bare,
    About(String),
}

/// Whether the line asks for another one ("another one about the sea"), and about what.
pub fn _the_second_one_asked_for(understood: &Understood) -> SecondOne {
    for field in [&understood.text, &understood.plain, &understood.clean] {
        let typed = text::squash(field);
        if typed.is_empty() || _NOT_A_VERSE_ASK.is_match(&typed) {
            continue;
        }
        if let Some(found) = _A_SECOND_ONE.match_(&typed) {
            let about = text::strip_chars(found.str(1), " .,!?");
            return if about.is_empty() { SecondOne::Bare } else { SecondOne::About(about.to_string()) };
        }
    }
    SecondOne::NotAsked
}

maps!(_ANOTHER_FORM);
strs!(BRIGHT, CALM);
lazy_re!(BRIGHTER, "happ|bright|cheer|funny|nicer|lighter|lift");
lazy_re!(CALMER, "calm|quiet|sooth|gentl|soft|slower|easier");

/// A reading that holds only a made-up line, as a writer is handed `{"clean": .., "text": ..}`.
fn asked_reading(asked: &str) -> Understood {
    Understood { clean: asked.to_string(), text: asked.to_string(), ..Default::default() }
}

/// "a haiku about the sea", or "a haiku" for a bare ask.
fn asked_line(form: &str, second: &SecondOne) -> String {
    match second {
        SecondOne::About(s) => format!("a {form} about {s}"),
        _ => format!("a {form}"),
    }
}

/// Another verse in the form the chat was just having.
pub fn _another_one(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let form = _the_form_last_time(memory).filter(|f| !f.is_empty())?;
    let second = _the_second_one_asked_for(understood);
    if second != SecondOne::NotAsked {
        if let Some(name) = _ANOTHER_FORM.get(&form) {
            let asked = asked_line(&form, &second);
            let said = call_form_global(name, memory, &asked_reading(&asked));
            if let Some(said) = said.filter(|s| !s.is_empty()) {
                return Some(said);
            }
        }
    }
    let clean = understood.clean.as_str();
    let thing = _thing(clean);
    if !thing.is_empty() {
        return _say(&form, &thing, &_subject(clean), None);
    }
    let brighter = BRIGHTER.is_match(clean);
    let calmer = brighter || CALMER.is_match(clean);
    if !calmer {
        return None;
    }
    let keys: &[String] = if brighter { &BRIGHT } else { &CALM };
    for key in keys {
        if let Some(lines) = FORMS.get(&form).and_then(|f| f.get(key)) {
            let lead = if brighter { "Something brighter, then." } else { "Something quieter, then." };
            return Some(_lines(lines, Some(lead)));
        }
    }
    None
}

/// A name for a quiz team or any team.
pub fn _quiz_name(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    static QUIZ: LazyLock<Re> = LazyLock::new(|| Re::new("\\b(?:quiz|trivia)\\b", 0));
    let clean = understood.clean.as_str();
    let mut seed = ord_sum(clean);
    if seed == 0 {
        seed = 1;
    }
    let names: &[String] = if QUIZ.is_match(clean) { &QUIZ_NAMES } else { &TEAM_NAMES };
    let n = names.len() as i64;
    let picks: Vec<&String> = [0, 3, 6].iter().map(|step| &names[((seed + step) % n) as usize]).collect();
    Some(format!("For the sheet: *{}*. Others that go down well: *{}* and *{}*.", picks[0], picks[1], picks[2]))
}

/// Whether the line says it already tried the very thing this row would offer.
pub fn _asked_for_what_they_already_tried(understood: &Understood, things: &[String]) -> bool {
    if understood.tried.is_empty() {
        return false;
    }
    if dup::asks_for_one_more(&understood.norm) {
        return false;
    }
    let tried: HashSet<String> = understood.tried.iter().map(|w| w.to_lowercase()).collect();
    things.iter().any(|thing| LETTER_RUN.findall_str(&thing.to_lowercase()).iter().any(|w| tried.contains(w)))
}

strs!(_OFFERED);
text_of!(_OFFER_TAIL);
list_maps!(_THE_FORM_NAMED);

/// A writer a made-up-thing entry calls: a build function, or a build wrapped by
/// `_with_the_detail_asked_for` (with its form and the words that name it).
#[derive(Clone)]
pub enum Writer {
    Build(BuildFn),
    Detail { build: BuildFn, form: String, names: Vec<String> },
}

/// The signature of the verse and story builders.
pub type BuildFn = fn(&mut Memory, &Understood) -> Option<String>;

fn run_writer(writer: &Writer, memory: &mut Memory, understood: &Understood) -> Option<String> {
    match writer {
        Writer::Build(f) => f(memory, understood),
        Writer::Detail { build, form, names } => _with_the_detail_asked_for_answer(*build, form, names, memory, understood),
    }
}

/// `_with_the_detail_asked_for(build, form).<locals>.answer`: the verse, plus an ask for a real
/// detail when the subject is one the bot can only write round.
pub fn _with_the_detail_asked_for_answer(
    build: BuildFn,
    form: &str,
    names: &[String],
    memory: &mut Memory,
    understood: &Understood,
) -> Option<String> {
    if _asked_for_what_they_already_tried(understood, names) {
        return None;
    }
    let reply = build(memory, understood);
    if !truthy(&reply) {
        return reply;
    }
    let mut reply = reply.unwrap();
    let clean = understood.clean.as_str();
    if _a_report_of_the_habit(clean) {
        return Some(reply);
    }
    let subject = _subject(clean);
    let thing = _thing(clean);
    let last = text::split_ref(&subject).last().map(|w| w.to_string());
    if _OFFERED.contains(&thing) || last.is_some_and(|w| _OFFERED.contains(&w)) {
        reply.push_str(&format_named(&_OFFER_TAIL, &[("form", form)]));
    }
    Some(reply)
}

/// `_a_second_one_read_first(writer, form).<locals>.write`: "another one (about X)" is read
/// before the writer, and goes to the form the chat was having when that is another one.
pub fn _a_second_one_read_first_write(writer: &Writer, form: &str, memory: &mut Memory, understood: &Understood) -> Option<String> {
    let second = _the_second_one_asked_for(understood);
    if second == SecondOne::NotAsked {
        return run_writer(writer, memory, understood);
    }
    if let Some(chosen) = _the_form_last_time(memory).filter(|c| !c.is_empty()) {
        if chosen != form {
            if let Some(name) = _ANOTHER_FORM.get(&chosen) {
                let asked = asked_line(&chosen, &second);
                return call_form_global(name, memory, &asked_reading(&asked));
            }
        }
    }
    let asked = asked_line(form, &second);
    let said = run_writer(writer, memory, &asked_reading(&asked));
    if truthy(&said) {
        return said;
    }
    if second == SecondOne::Bare {
        return None;
    }
    run_writer(writer, memory, understood)
}

/// A build function by its Python name.
fn build_by_name(name: &str) -> Option<BuildFn> {
    Some(match name {
        "_haiku" => _haiku,
        "_limerick" => _limerick,
        "_poem" => _poem,
        "_story" => _story,
        "_rap" => _rap,
        "_song" => _song,
        "_rhyme" => _rhyme,
        _ => return None,
    })
}

/// The module global a form's name stands for after import: the build wrapped for the detail
/// (all but `_rhyme`) and then for "another one", with the form word it was wrapped with.
fn form_global(name: &str) -> Option<(Writer, &'static str)> {
    let form: &'static str = match name {
        "_haiku" => "haiku",
        "_limerick" => "limerick",
        "_poem" => "poem",
        "_story" => "story",
        "_rap" => "rap",
        "_song" => "song",
        "_rhyme" => "rhyme",
        _ => return None,
    };
    let build = build_by_name(name)?;
    if name == "_rhyme" {
        return Some((Writer::Build(build), form));
    }
    let names = _THE_FORM_NAMED.get(form).cloned().unwrap_or_else(|| vec![form.to_string(), format!("{form}s")]);
    Some((Writer::Detail { build, form: form.to_string(), names }, form))
}

/// `globals()[name](memory, understood)` for a form's writer (e.g. "_haiku").
pub fn call_form_global(name: &str, memory: &mut Memory, understood: &Understood) -> Option<String> {
    let (writer, form) = form_global(name).unwrap_or_else(|| panic!("KeyError: {name}"));
    _a_second_one_read_first_write(&writer, form, memory, understood)
}

/// A writer from the captured function value of a `write` closure.
fn writer_from_fn(f: &FnRef) -> Writer {
    let qualname = f.name().rsplit(':').next().unwrap_or("");
    if qualname == "_with_the_detail_asked_for.<locals>.answer" {
        let build = f.cell("build").and_then(Value::as_fn).expect("build cell");
        let build_name = build.name().rsplit(':').next().unwrap_or("");
        return Writer::Detail {
            build: build_by_name(build_name).unwrap_or_else(|| panic!("no build {build_name}")),
            form: f.cell("form").map(|v| v.str().to_string()).unwrap_or_default(),
            names: f.cell("names").map(Value::strs).unwrap_or_default(),
        };
    }
    Writer::Build(build_by_name(qualname).unwrap_or_else(|| panic!("no writer {qualname}")))
}

// ---------------------------------------------------------------------------------------------
// What to watch or read: by company, a limit, a country, a kind of film, a running time

/// CINEMA: a country word -> (adjective, country, language).
static CINEMA: LazyLock<IndexMap<String, (String, String, String)>> = LazyLock::new(|| {
    g("CINEMA")
        .pairs()
        .iter()
        .map(|(k, v)| {
            let it = v.strs();
            (k.str().to_string(), (it[0].clone(), it[1].clone(), it[2].clone()))
        })
        .collect()
});
strs!(_COUNTRY_AFTER_A_LITTLE_WORD, _COUNTRY_BEFORE_A_THING);

/// False when a country word is an adjective of something else ("french toast").
pub fn _a_country_is_a_country(understood: &Understood) -> bool {
    let words = lower_words(&understood.words);
    for i in 0..words.len().saturating_sub(1) {
        if !CINEMA.contains_key(&words[i]) {
            continue;
        }
        let after = &words[i + 1];
        if _COUNTRY_AFTER_A_LITTLE_WORD.contains(after) || _COUNTRY_BEFORE_A_THING.contains(after) {
            return false;
        }
    }
    true
}

/// A kind of film's shelf: (label, said as, rows of (title, minutes, reason)).
type Shelf = (String, String, Vec<(String, i64, String)>);

static FILM_KINDS: LazyLock<IndexMap<String, Shelf>> = LazyLock::new(|| {
    g("FILM_KINDS")
        .pairs()
        .iter()
        .map(|(k, v)| {
            let it = v.items();
            let rows = it[2]
                .items()
                .iter()
                .map(|r| {
                    let r = r.items();
                    (r[0].str().to_string(), r[1].as_int().unwrap_or(0), r[2].str().to_string())
                })
                .collect();
            (k.str().to_string(), (it[0].str().to_string(), it[1].str().to_string(), rows))
        })
        .collect()
});

/// The words that name a kind of film ("horror", "horrors", "a horror film").
pub fn _kind_words(kind: &str) -> HashSet<String> {
    let (label, said_as) = FILM_KINDS.get(kind).map(|(l, s, _)| (l.as_str(), s.as_str())).unwrap_or(("", ""));
    [kind.to_string(), format!("{kind}s"), label.to_string(), format!("{label}s"), said_as.to_string()]
        .into_iter()
        .filter(|w| !w.is_empty())
        .collect()
}

maps!(SHOW_SUBJECTS, SHOW_SUBJECT_NAMES);
static SHOW_SUBJECT_NAME_SET: LazyLock<HashSet<String>> = LazyLock::new(|| SHOW_SUBJECT_NAMES.keys().cloned().collect());
static DOCUMENTARY_WORD_SET: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("DOCUMENTARY_WORDS"));
res!(KIND_REFUSED_AFTER);

/// The subject rows answer only a line that does not turn the subject down.
pub fn _the_subject_is_wanted(understood: &Understood) -> bool {
    !_own_texts(understood).iter().any(|t| _a_word_is_turned_down(t, &SHOW_SUBJECT_NAME_SET, Some(&KIND_REFUSED_AFTER)))
}

/// The documentary rows answer only a line that does not turn documentaries down.
pub fn _a_documentary_is_wanted(understood: &Understood) -> bool {
    !_own_texts(understood).iter().any(|t| _a_word_is_turned_down(t, &DOCUMENTARY_WORD_SET, Some(&KIND_REFUSED_AFTER)))
}

strs!(FILM_GENRE_WORDS);

/// Whether the line names a film genre.
pub fn _names_a_film_genre(line: &str) -> bool {
    let words = format!(" {} ", _plain_words(line));
    FILM_GENRE_WORDS.iter().any(|w| bounded_alnum(w).is_match(&words))
}

res!(_PLAYING_NOT_WATCHING, _SHOW_YES_OR_NO, _SHOW_SUBJECT_LEAD);
sets!(SHOW_MEDIUM_WORDS);
lazy_re!(WORD_WITH_QUOTE, "[a-z0-9']+");

/// False when the line is about playing, not watching.
pub fn _a_programme_is_wanted(understood: &Understood) -> bool {
    !_PLAYING_NOT_WATCHING.is_match(&or(&understood.text, &understood.clean).to_lowercase())
}

/// The subject a show is asked about, by its first naming word.
pub fn _show_subject(line: &str) -> String {
    for word in WORD_WITH_QUOTE.findall_str(&_plain_words(line)) {
        if let Some(subject) = SHOW_SUBJECT_NAMES.get(&word) {
            if !SHOW_MEDIUM_WORDS.contains(&word) {
                return subject.clone();
            }
        }
    }
    String::new()
}

/// A show about the subject the line names.
pub fn _show_about(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    // Python passes the reading itself to `_show_subject`, whose `str(...)` is the dict's repr;
    // the words of the repr that matter are the reading's own, read here from its fields.
    let subject = _show_subject(&reading_repr(reading));
    if subject.is_empty() {
        return None;
    }
    let pick = SHOW_SUBJECTS.get(&subject).filter(|p| !p.is_empty())?;
    let said = or(&reading.clean, &reading.text);
    if _SHOW_YES_OR_NO.is_match(said) {
        return Some(format!("Yes - {}", _SHOW_SUBJECT_LEAD.sub("", pick)));
    }
    Some(pick.clone())
}

/// What a reading's words are when Python prints the reading dict (`str(reading)`), field by
/// field in the reading's key order, so a word-scan over it finds the same first word.
fn reading_repr(reading: &Understood) -> String {
    let value = serde_json::to_value(reading).unwrap_or_default();
    py_repr_json(&value)
}

/// Python's repr of a JSON-shaped value (dict keys in order, True/False/None).
fn py_repr_json(v: &serde_json::Value) -> String {
    use serde_json::Value as J;
    match v {
        J::Null => "None".into(),
        J::Bool(b) => if *b { "True".into() } else { "False".into() },
        J::Number(n) => n.to_string(),
        J::String(s) => d2_shared::py_repr_str(s),
        J::Array(a) => format!("[{}]", a.iter().map(py_repr_json).collect::<Vec<_>>().join(", ")),
        J::Object(o) => format!(
            "{{{}}}",
            o.iter().map(|(k, v)| format!("{}: {}", d2_shared::py_repr_str(k), py_repr_json(v))).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// The medium a line names: "book", "series", "podcast", "film" or "".
pub fn _pick_kind(line: &str) -> String {
    for word in WORD_WITH_QUOTE.findall_str(&_plain_words(line)) {
        match word.as_str() {
            "book" | "books" | "novel" | "novels" | "read" | "reading" => return "book".into(),
            "series" | "boxset" | "show" | "shows" | "telly" | "episode" | "episodes" => return "series".into(),
            "podcast" | "podcasts" => return "podcast".into(),
            "film" | "films" | "movie" | "movies" | "watch" | "watching" => return "film".into(),
            _ => {}
        }
    }
    String::new()
}

res!(_REFUSED_KIND);

/// The medium the line turns down ("not a film"), or "".
pub fn _pick_refused(line: &str) -> String {
    match _REFUSED_KIND.search(&format!(" {} ", _plain_words(line))) {
        Some(found) => _pick_kind(found.name_str("kind")),
        None => String::new(),
    }
}

/// Who the pick is for: "parents", "partner", "friends" or "".
pub fn _pick_who(line: &str) -> String {
    let said = format!(" {} ", _plain_words(line));
    if [" my mum", " my dad", " my mother", " my father", "in laws", "in law", "parents"].iter().any(|t| said.contains(t)) {
        return "parents".into();
    }
    if ["partner", "wife", "husband", "girlfriend", "boyfriend", "my other half"].iter().any(|t| said.contains(t)) {
        return "partner".into();
    }
    if [" friends", " mate", "mates", "everyone", "the girls", "the lads"].iter().any(|t| said.contains(t)) {
        return "friends".into();
    }
    String::new()
}

strs!(SUBS_TELLS, FREE_TELLS);

/// The limit on a pick: "nosubs", "free" or "", from this line or the last two.
pub fn _pick_limit(line: &str, memory: Option<&Memory>) -> String {
    let said = format!(" {} {} ", line.to_lowercase().replace('\u{2019}', "'"), _plain_words(line));
    if SUBS_TELLS.iter().any(|t| said.contains(t.as_str())) {
        return "nosubs".into();
    }
    if FREE_TELLS.iter().any(|t| said.contains(t.as_str())) {
        return "free".into();
    }
    let Some(memory) = memory else { return String::new() };
    let mine = _plain_words(line);
    let mine = text::strip(&mine);
    for turn in last_turns(memory, 2) {
        let user = turn.user.as_str();
        if text::strip(&_plain_words(user)) == mine {
            continue;
        }
        let said = format!(" {} {} ", user.to_lowercase().replace('\u{2019}', "'"), _plain_words(user));
        if SUBS_TELLS.iter().any(|t| said.contains(t.as_str())) {
            return "nosubs".into();
        }
        if FREE_TELLS.iter().any(|t| said.contains(t.as_str())) {
            return "free".into();
        }
    }
    String::new()
}

strs!(SUBS_WANT, SUBS_WANT_IF_WATCHING);
res!(SUBS_TURNED_DOWN, WATCHES_SOMETHING);

/// Whether the line wants a subtitled film.
pub fn _wants_subtitles(line: &str) -> bool {
    let words = format!(" {} ", _plain_words(line));
    if SUBS_TELLS.iter().any(|t| words.contains(t.as_str())) || SUBS_TURNED_DOWN.is_match(&words) {
        return false;
    }
    if SUBS_WANT.iter().any(|t| words.contains(t.as_str())) {
        return true;
    }
    WATCHES_SOMETHING.is_match(&words) && SUBS_WANT_IF_WATCHING.iter().any(|t| words.contains(t.as_str()))
}

/// The country whose films the line asks for: (adjective, country, language).
pub fn _cinema_of(line: &str) -> Option<(String, String, String)> {
    let words = format!(" {} ", _plain_words(line));
    for (word, said) in CINEMA.iter() {
        let Some(at) = words.find(&format!(" {word} ")) else { continue };
        let before = text::split_ref(&words[..at]);
        let before = &before[before.len().saturating_sub(2)..];
        if before.iter().any(|one| NEGATORS.contains(*one)) {
            continue;
        }
        return Some(said.clone());
    }
    None
}

/// One row of the pick table: (title, kind, who it suits, free to air, needs no subtitles, why).
#[derive(Clone, Debug, PartialEq)]
pub struct PickRow {
    pub title: String,
    pub kind: String,
    pub who: Vec<String>,
    pub free: bool,
    pub nosubs: bool,
    pub reason: String,
}

static PICKS: LazyLock<Vec<PickRow>> = LazyLock::new(|| {
    g("PICKS")
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            PickRow {
                title: r[0].str().to_string(),
                kind: r[1].str().to_string(),
                who: r[2].strs(),
                free: r[3].truthy(),
                nosubs: r[4].truthy(),
                reason: r[5].str().to_string(),
            }
        })
        .collect()
});

/// NATIONAL_CINEMA: (title, country, why).
static NATIONAL_CINEMA: LazyLock<Vec<(String, String, String)>> = LazyLock::new(|| {
    g("NATIONAL_CINEMA")
        .items()
        .iter()
        .map(|r| {
            let r = r.strs();
            (r[0].clone(), r[1].clone(), r[2].clone())
        })
        .collect()
});

/// A country's films as pick rows.
pub fn _rows_from_cinema(country: &str) -> Vec<PickRow> {
    NATIONAL_CINEMA
        .iter()
        .filter(|(_, of, _)| of == country)
        .map(|(title, _, reason)| PickRow {
            title: title.clone(),
            kind: "film".into(),
            who: Vec::new(),
            free: false,
            nosubs: false,
            reason: reason.clone(),
        })
        .collect()
}

/// The first `count` picks of a kind, by limit and by who they suit.
pub fn _pick_rows(kind: &str, who: &str, limit: &str, count: usize, country: &str) -> Vec<PickRow> {
    if !country.is_empty() {
        return _rows_from_cinema(country).into_iter().take(count).collect();
    }
    let mut rows: Vec<PickRow> = PICKS.iter().filter(|r| r.kind == kind).cloned().collect();
    let limit = if kind == "film" || kind == "series" { limit } else { "" };
    if limit == "free" {
        let free: Vec<PickRow> = rows.iter().filter(|r| r.free).cloned().collect();
        if !free.is_empty() {
            rows = free;
        }
    }
    if limit == "nosubs" {
        let plain: Vec<PickRow> = rows.iter().filter(|r| r.nosubs).cloned().collect();
        if !plain.is_empty() {
            rows = plain;
        }
    }
    if !who.is_empty() {
        let suited: Vec<PickRow> = rows.iter().filter(|r| r.who.iter().any(|w| w == who)).cloned().collect();
        let rest: Vec<PickRow> = rows.iter().filter(|r| !suited.contains(r)).cloned().collect();
        rows = suited.into_iter().chain(rest).collect();
    }
    rows.truncate(count);
    rows
}

/// "*A* (why), *B* (why) or *C* (why)".
pub fn _pick_list(rows: &[PickRow]) -> String {
    let said: Vec<String> = rows.iter().map(|r| format!("*{}* ({})", r.title, r.reason)).collect();
    if said.len() < 2 {
        return said.first().cloned().unwrap_or_default();
    }
    format!("{} or {}", said[..said.len() - 1].join(", "), said.last().unwrap())
}

/// The sentence on subtitles for a country's films.
pub fn _subs_sentence(count: usize, language: &str) -> String {
    match count {
        1 => format!("It is in {language} with subtitles."),
        2 => format!("Both are in {language} with subtitles."),
        _ => format!("All three are in {language} with subtitles."),
    }
}

maps!(_PICK_PARENT_VERB);

/// The pick reply: a head for the kind and the company, the list, and one closing line.
pub fn _pick_reply(kind: &str, who: &str, limit: &str, rows: &[PickRow], country: &str, language: &str) -> String {
    let mut head = match kind {
        "series" => "For a series",
        "book" => "For a book",
        "podcast" => "For a podcast",
        _ => "For a film",
    }
    .to_string();
    match who {
        "parents" => head.push_str(" with your parents"),
        "partner" => head.push_str(" for the two of you"),
        "friends" => head.push_str(" with friends"),
        _ => {}
    }
    if !country.is_empty() {
        let first: String = country.chars().take(1).collect::<String>().to_lowercase();
        let article = if "aeiou".contains(first.as_str()) { "an" } else { "a" };
        head = format!("For {article} {country} film");
    }
    let mut said = format!("{head}: {}.", _pick_list(rows));
    if !country.is_empty() {
        said.push(' ');
        said.push_str(&_subs_sentence(rows.len(), language));
    }
    let screen = kind == "film" || kind == "series";
    if limit == "free" && screen {
        said.push_str(
            " All three are free-to-air rather than behind a subscription, and iPlayer and ITVX are \
             both free - rights move, so check iPlayer first.",
        );
    } else if limit == "nosubs" && screen {
        said.push_str(
            " Nothing there needs subtitles: all of them are in English and none of them is harder to \
             follow than a chat show.",
        );
    } else if who == "parents" {
        said.push_str(&format!(
            " All of them are safe to {}, and none needs any explaining.",
            _PICK_PARENT_VERB.get(kind).expect("KeyError")
        ));
    } else if kind == "book" {
        said.push_str(" Tell me the last one you enjoyed and I'll match the next.");
    } else {
        said.push_str(" Say the mood and I'll narrow it down.");
    }
    said
}

/// The medium one of the last two lines asked for, when this line names none.
pub fn _pick_carry(memory: &Memory, line: &str) -> String {
    let mine = _plain_words(line);
    let mine = text::strip(&mine);
    for turn in last_turns(memory, 2) {
        let user = turn.user.as_str();
        if user.is_empty() || text::strip(&_plain_words(user)) == mine {
            continue;
        }
        let kind = _pick_kind(user);
        if !kind.is_empty() {
            return kind;
        }
    }
    String::new()
}

res!(BOOK_AS_VERB);
strs!(PICK_OTHER_MEDIUM, PICK_SUBJECT, PICK_REACT, FILM_SAID, OTHER_SUBJECT);

/// Whether the plain pick rows stand down for this line (booking, another medium, a reaction,
/// a family).
pub fn _pick_stands_down(line: &str) -> bool {
    let words = format!(" {} ", _plain_words(line));
    if BOOK_AS_VERB.is_match(&words) {
        return true;
    }
    if PICK_OTHER_MEDIUM.iter().any(|t| words.contains(t.as_str())) {
        return true;
    }
    if PICK_SUBJECT.iter().any(|t| words.contains(&format!(" {t} "))) {
        return true;
    }
    if PICK_REACT.iter().any(|t| words.contains(&format!(" {t} "))) {
        return true;
    }
    ["family", "the kids", "kids", "children", "child"].iter().any(|t| words.contains(t))
}

/// Whether the line is about another subject altogether (and names no film).
pub fn _another_subject(line: &str) -> bool {
    let words = format!(" {} ", _plain_words(line));
    if FILM_SAID.iter().any(|w| words.contains(&format!(" {w} "))) {
        return false;
    }
    OTHER_SUBJECT.iter().any(|w| words.contains(&format!(" {w} ")))
}

/// The line's typed, clean and normalised text together.
pub fn _pick_line(understood: &Understood) -> String {
    [&understood.text, &understood.clean, &understood.norm_text]
        .iter()
        .filter(|p| !p.is_empty())
        .map(|p| p.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

strs!(FILM_KIND_WORDS);

/// The kind of film the line names (not after a negator), or "".
pub fn _film_kind(line: &str) -> String {
    let words = format!(" {} ", _plain_words(line));
    for kind in FILM_KIND_WORDS.iter() {
        let Some(found) = rx(format!("(?<![a-z0-9]){kind}(?:s|es)?(?![a-z0-9])")).search(&words) else { continue };
        let before = text::split_ref(&words[..found.start()]);
        let before = &before[before.len().saturating_sub(2)..];
        if before.iter().any(|one| NEGATORS.contains(*one)) {
            continue;
        }
        return kind.clone();
    }
    String::new()
}

res!(_UNDER, _LIMIT_CUE, _HOUR_MINUTES);
sets!(STRICT_LIMIT);
static _HOUR_WORDS: LazyLock<IndexMap<String, i64>> = LazyLock::new(|| int_map_of("_HOUR_WORDS"));
lazy_re!(TWO_OR_THREE_DIGITS, "(\\d{2,3})\\b");

/// The running time the line allows, in minutes (0 for none), and whether it is strict ("under").
pub fn _limit_wanted(line: &str) -> (i64, bool) {
    let words = format!(" {} ", _plain_words(line));
    let (strict, tail) = match _UNDER.search(&words) {
        Some(found) => (STRICT_LIMIT.contains(found.str(1)), text::strip(found.str(2)).to_string()),
        None => match _LIMIT_CUE.search(&words) {
            Some(found) => (false, text::strip(found.str(2)).to_string()),
            None => return (0, false),
        },
    };
    if let Some(number) = TWO_OR_THREE_DIGITS.match_(&tail) {
        return (number.str(1).parse().expect("int()"), strict);
    }
    let mut phrases: Vec<(&String, &i64)> = _HOUR_WORDS.iter().collect();
    phrases.sort_by_key(|(p, _)| std::cmp::Reverse(p.chars().count()));
    for (phrase, minutes) in phrases {
        if tail.starts_with(phrase.as_str()) {
            return (*minutes, strict);
        }
    }
    if let Some(hours) = _HOUR_MINUTES.match_(&tail) {
        return match hours.str(1).parse::<f64>() {
            Ok(h) => (text::round_half_even(h * 60.0), strict),
            Err(_) => (0, false),
        };
    }
    (0, false)
}

/// The running time the line allows, in minutes.
pub fn _minutes_wanted(line: &str) -> i64 {
    _limit_wanted(line).0
}

/// FILM_RUNTIME: (title, minutes, why), shortest first.
static FILM_RUNTIME: LazyLock<Vec<(String, i64, String)>> = LazyLock::new(|| {
    g("FILM_RUNTIME")
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            (r[0].str().to_string(), r[1].as_int().unwrap_or(0), r[2].str().to_string())
        })
        .collect()
});

/// Films that fit the running time the line gives.
pub fn _film_under(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _pick_line(understood);
    if _pick_stands_down(&line) {
        return None;
    }
    if _another_subject(&line) {
        return None;
    }
    let (limit, strict) = _limit_wanted(&line);
    if limit == 0 {
        return None;
    }
    if !_film_kind(&line).is_empty() {
        return _film_kind_reply(memory, understood);
    }
    if _mood_of(&line).is_some() {
        if !_pick_kind(&line).is_empty() || !_pick_carry(memory, &line).is_empty() {
            return None;
        }
        return _a_pick_in_any_medium(memory, understood);
    }
    if _names_a_film_genre(&line) {
        return None;
    }
    let carried = _kind_carry(memory, &line);
    if !carried.is_empty() {
        if let Some(within) = _shelf_within(&carried, limit, strict) {
            return Some(within);
        }
    }
    let rows: Vec<&(String, i64, String)> =
        FILM_RUNTIME.iter().filter(|r| if strict { r.1 < limit } else { r.1 <= limit }).collect();
    let under_at = if strict { "under" } else { "at" };
    if rows.is_empty() {
        return Some(format!(
            "Nothing I know comes in {under_at} {limit} minutes. The shortest I have is *A Grand Day Out* \
             at 23 minutes, and *The Snowman* at 27 - say the number again if I have read it wrong."
        ));
    }
    let picked: Vec<&&(String, i64, String)> = rows[rows.len().saturating_sub(3)..].iter().rev().collect();
    let said: Vec<String> = picked.iter().map(|r| format!("*{}* ({} minutes)", r.0, r.1)).collect();
    let body = if said.len() == 1 { said[0].clone() } else { format!("{} or {}", said[..said.len() - 1].join(", "), said.last().unwrap()) };
    Some(format!(
        "{body} - all {} {limit} minutes. Say what you feel like and I will pick one.",
        if strict { "under" } else { "up to" }
    ))
}

/// The films on a kind's shelf that fit the time; None when they all do.
pub fn _shelf_within(kind: &str, limit: i64, strict: bool) -> Option<String> {
    let (label, said_as, rows) = FILM_KINDS.get(kind).expect("KeyError");
    let fit: Vec<&(String, i64, String)> = rows.iter().filter(|r| if strict { r.1 < limit } else { r.1 <= limit }).collect();
    if fit.is_empty() {
        let mut shortest = &rows[0];
        for r in rows {
            if r.1 < shortest.1 {
                shortest = r;
            }
        }
        return Some(format!(
            "Nothing on the {label} shelf comes in {} {limit} minutes - the shortest is *{}* at {}. Say the \
             number again if I have read it wrong, or name another kind.",
            if strict { "under" } else { "at" },
            shortest.0,
            shortest.1
        ));
    }
    if fit.len() == rows.len() {
        return None;
    }
    let said: Vec<String> = fit.iter().map(|(title, minutes, reason)| format!("*{title}* ({minutes} minutes, {reason})")).collect();
    let body = if said.len() == 1 { said[0].clone() } else { format!("{} or {}", said[..said.len() - 1].join(", "), said.last().unwrap()) };
    let over = rows.len() - fit.len();
    Some(format!(
        "{body}. {} {} {limit} minutes, and the other {} on that shelf run{} longer, so say if you would \
         rather have one of those.",
        text::upper_first(said_as),
        if strict { "under" } else { "up to" },
        if over == 1 { "one" } else { "two" },
        if over == 1 { "s" } else { "" }
    ))
}

/// The kind of film one of the last two lines asked for, when this line names none.
pub fn _kind_carry(memory: &Memory, line: &str) -> String {
    let mine = _plain_words(line);
    let mine = text::strip(&mine);
    for turn in last_turns(memory, 2) {
        let user = turn.user.as_str();
        if user.is_empty() || text::strip(&_plain_words(user)) == mine {
            continue;
        }
        let kind = _film_kind(user);
        if !kind.is_empty() {
            return kind;
        }
    }
    String::new()
}

strs!(SHORTEST_WORDS);

/// Whether the line asks for the shortest one of a list.
pub fn _shortest_wanted(line: &str) -> bool {
    if !SHORTEST_WORDS.iter().any(|w| bounded_alnum(w).is_match(line)) {
        return false;
    }
    ["one", "ones", "film", "films", "those", "these", "them", "pick", "of", "which", "what", "three", "any", "any of"]
        .iter()
        .any(|w| rx(format!("(?<![a-z0-9]){w}(?![a-z0-9])")).is_match(line))
}

/// Films of the kind the line names (christmas, horror, ...), within any time it gives.
pub fn _film_kind_reply(memory: &mut Memory, understood: &Understood) -> Option<String> {
    static IS_WAS: LazyLock<Re> = LazyLock::new(|| Re::new("\\s*(?:is|was|are|were|isnt|wasnt)\\b", 0));
    let line = _pick_line(understood);
    if _pick_stands_down(&line) {
        return None;
    }
    if _another_subject(&line) {
        return None;
    }
    let words = format!(" {} ", _plain_words(&line));
    let kind = _film_kind(&line);
    if kind.is_empty() {
        return None;
    }
    let medium = _pick_kind(&line);
    if !medium.is_empty() && medium != "film" {
        return None;
    }
    if IS_WAS.match_(&_plain_words(&line)).is_some() && words.contains(&format!(" {kind} ")) {
        return None;
    }
    let kind_words = _kind_words(&kind);
    if _a_word_is_turned_down(&line, &kind_words, Some(&KIND_REFUSED_AFTER))
        || _the_chat_turned_down(&kind_words, memory, Some(&KIND_REFUSED_AFTER))
    {
        return None;
    }
    let (_label, said_as, rows) = FILM_KINDS.get(&kind).expect("KeyError");
    if _shortest_wanted(&words) {
        let mut shortest = &rows[0];
        for r in rows {
            if r.1 < shortest.1 {
                shortest = r;
            }
        }
        let longer: Vec<&(String, i64, String)> = rows.iter().filter(|r| r.0 != shortest.0).collect();
        let mut said = format!("The shortest on that shelf is *{}* at {} minutes, {}", shortest.0, shortest.1, shortest.2);
        if let Some(next) = longer.first() {
            said.push_str(&format!(". *{}* runs {} if you have the time.", next.0, next.1));
        }
        return Some(said);
    }
    let (limit, strict) = _limit_wanted(&line);
    if limit != 0 {
        if let Some(within) = _shelf_within(&kind, limit, strict) {
            return Some(within);
        }
    }
    let said: Vec<String> = rows.iter().map(|(title, minutes, reason)| format!("*{title}* ({minutes} minutes, {reason})")).collect();
    Some(format!(
        "For {said_as}: {}, or {}. Any of them is a good way in, and I can go shorter or pick one if you \
         tell me the mood.",
        said[..said.len() - 1].join(", "),
        said.last().expect("IndexError")
    ))
}

/// A plain pick of the medium the line (or the chat) asks for, by company, limit or country.
pub fn _pick_plain(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _pick_line(understood);
    if _pick_stands_down(&line) {
        return None;
    }
    if _mood_of(&line).is_some() {
        return None;
    }
    let mut kind = _pick_kind(&line);
    if kind.is_empty() {
        kind = _pick_carry(memory, &line);
    }
    let refused = _pick_refused(&line);
    if !refused.is_empty() && kind == refused {
        let carried = _pick_carry(memory, &line);
        if !carried.is_empty() && carried != refused {
            kind = carried;
        } else if refused == "film" || refused == "series" {
            kind = if refused == "film" { "series".into() } else { "film".into() };
        } else {
            return None;
        }
    }
    if kind.is_empty() || kind == "podcast" {
        return None;
    }
    if let Some((adjective, country, language)) = _cinema_of(&line) {
        if kind == "film" {
            let rows = _pick_rows("film", "", "", 3, &country);
            if rows.is_empty() {
                let mut have: Vec<&String> = Vec::new();
                for (_, of, _) in NATIONAL_CINEMA.iter() {
                    if !have.contains(&of) {
                        have.push(of);
                    }
                }
                let heard = format!("{}, {} or {}", have[0], have[1], have[2]);
                let rest = have.len() as i64 - 3;
                let more = if rest > 0 { format!(" and {rest} other countries") } else { String::new() };
                return Some(format!(
                    "I have not got a {adjective} list, and I would rather say so than hand you the wrong film. \
                     I do have {heard}{more} - tell me a {adjective} film you liked and I will match the next \
                     one."
                ));
            }
            return Some(_pick_reply("film", "", "", &rows, &adjective, &language));
        }
    }
    if kind == "film" && _wants_subtitles(&line) {
        let rows: Vec<PickRow> = NATIONAL_CINEMA
            .iter()
            .take(9)
            .step_by(4)
            .map(|(title, of, reason)| PickRow {
                title: title.clone(),
                kind: "film".into(),
                who: Vec::new(),
                free: false,
                nosubs: false,
                reason: format!("{of}: {reason}"),
            })
            .collect();
        return Some(format!(
            "Subtitled, then: {}. Tell me a country and I will give you three of its films.",
            _pick_list(&rows)
        ));
    }
    let who = _pick_who(&line);
    let limit = _pick_limit(&line, Some(memory));
    let rows = _pick_rows(&kind, &who, &limit, 3, "");
    if rows.is_empty() {
        return None;
    }
    Some(_pick_reply(&kind, &who, &limit, &rows, "", ""))
}

/// The pick again under the limit the line adds ("nothing with subtitles", "only free ones").
pub fn _pick_limit_reply(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _pick_line(understood);
    if _pick_stands_down(&line) {
        return None;
    }
    let limit = _pick_limit(&line, Some(memory));
    if limit.is_empty() {
        return None;
    }
    let kind = _pick_kind(&line);
    if kind == "book" {
        return Some(_pick_reply("book", "", "", &_pick_rows("book", "", "", 3, ""), "", ""));
    }
    if limit == "nosubs" {
        let rows = _pick_rows("film", "", "nosubs", 3, "");
        return Some(format!(
            "Nothing that needs subtitles, then: {}. All three are in English, so nobody has to read the \
             screen to follow them.",
            _pick_list(&rows[..rows.len().min(3)])
        ));
    }
    let kind = if kind == "film" || kind == "series" { kind.as_str() } else { "series" };
    let rows = _pick_rows(kind, "", "free", 3, "");
    Some(format!(
        "Then it is iPlayer and ITVX, and both are free. {}. All three are BBC, so they are on iPlayer, \
         and a film on ITV turns up most weeks too - check iPlayer rather than trusting a listing, \
         because rights move.",
        _pick_list(&rows[..rows.len().min(3)])
    ))
}

// ---------------------------------------------------------------------------------------------
// Books and comics for a reader's age

static _WORD_AGES: LazyLock<IndexMap<String, i64>> = LazyLock::new(|| int_map_of("_WORD_AGES"));
lazy_re!(READER_YEARS_OLD, "\\b(\\d{1,2})\\s*(?:-|\\s)?\\s*(?:year|years|yr)s?\\s*-?\\s*old\\b");
lazy_re!(READER_AGED, "\\b(?:aged|age|who is|who's|is)\\s+(\\d{1,2})\\b");
lazy_re!(TEEN_WORD, "\\bteen(?:ager)?s?\\b");

/// The reader's age the line gives (0 for none).
pub fn _the_reader_age(line: &str) -> i64 {
    if let Some(m) = READER_YEARS_OLD.search(line) {
        return m.str(1).parse().expect("int()");
    }
    if let Some(m) = READER_AGED.search(line) {
        return m.str(1).parse().expect("int()");
    }
    for (word, age) in _WORD_AGES.iter() {
        if line.contains(word.as_str()) {
            return *age;
        }
    }
    0
}

static _AGE_BANDS: LazyLock<Vec<(i64, i64, String)>> = LazyLock::new(|| {
    g("_AGE_BANDS")
        .items()
        .iter()
        .map(|r| {
            let r = r.items();
            (r[0].as_int().unwrap_or(0), r[1].as_int().unwrap_or(0), r[2].str().to_string())
        })
        .collect()
});

/// Books for a child reading alone, by age.
pub fn _book_for_the_age(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = or(&understood.clean, &understood.text).to_lowercase();
    let mut age = _the_reader_age(&line);
    if age == 0 && TEEN_WORD.is_match(&line) {
        age = 13;
    }
    for (low, high, lead) in _AGE_BANDS.iter() {
        if age != 0 && *low <= age && age <= *high {
            return Some(format!(
                "{lead} Take them to a bookshop and let them pick the first one themselves: the series they \
                 choose is the one they read."
            ));
        }
    }
    Some(
        "Short chapters and a picture every few pages are what a 5 to 7 year old reading alone \
         wants: *The Worst Witch*, *Mr Gum*, *Dog Man* or *Captain Underpants*. For 8 to 9, *The Boy \
         at the Back of the Class*, *Horrible Histories* or *Percy Jackson*. For 10 to 12, *Wonder*, \
         *Artemis Fowl* or *Skulduggery Pleasant*. For 13 and over, *The Hunger Games*, *Northern \
         Lights* or *The Book Thief*. Take them to a bookshop and let them pick the first one \
         themselves; the series they choose is the one they read."
            .into(),
    )
}

res!(_A_SHORT_TIME);
text_of!(_AN_HOUR, _A_DAY);

/// An hour's plan or a day's, by what the line asks for.
pub fn _an_hour_or_a_day(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = or(&understood.clean, &understood.text).to_lowercase();
    Some(if _A_SHORT_TIME.is_match(&line) { _AN_HOUR.clone() } else { _A_DAY.clone() })
}

sets!(_CHILD_OF_THEIRS, _LIKING, _ASKS_FOR_A_PICK_WORDS, _TEEN_WORDS, _KIND_ASKS, _BEGINNER_WORDS);

/// Whether a parent says their child already likes comics (news, not an ask for a pick).
pub fn _a_child_who_reads_already(understood: &Understood) -> bool {
    let words = lower_words(&understood.words);
    if !words.iter().any(|w| _CHILD_OF_THEIRS.contains(w)) {
        return false;
    }
    if !words.iter().any(|w| w == "my" || w == "our") {
        return false;
    }
    if !words.iter().any(|w| _LIKING.contains(w)) {
        return false;
    }
    !words.iter().any(|w| _ASKS_FOR_A_PICK_WORDS.contains(w))
}

res!(_SUITABILITY_ASK, _COMIC_MEANING_ASK);
text_of!(_COMIC_SUITABILITY, _COMIC_MEANING, _COMIC_BEGINNER, _COMIC_FOR_A_TEEN, _COMIC_FOR_A_CHILD, _COMIC_FOR_A_GROWN_UP);
maps!(_KIND_WORDS);

/// Whether the line asks if a comic suits an age.
pub fn _asks_if_a_comic_suits_an_age(understood: &Understood) -> bool {
    _SUITABILITY_ASK.is_match(&or(&understood.clean, &understood.text).to_lowercase())
}

/// The card for the kind of comic the line asks about (manga, marvel, ...), or "".
pub fn _the_kind_named(understood: &Understood) -> String {
    let said = or(&understood.clean, &understood.text).to_lowercase();
    if _COMIC_MEANING_ASK.is_match(&said) {
        return _COMIC_MEANING.clone();
    }
    let words = lower_words(&understood.words);
    if !words.iter().any(|w| _KIND_ASKS.contains(w)) {
        return String::new();
    }
    for (word, card) in _KIND_WORDS.iter() {
        if words.contains(word) {
            return card.clone();
        }
    }
    if words.iter().any(|w| _BEGINNER_WORDS.contains(w)) || said.contains("where do i start") || said.contains("where to start") {
        return _COMIC_BEGINNER.clone();
    }
    String::new()
}

/// The reader a line names: "teen", "child" or "".
pub fn _reader_named(words: &[String], numbers: &[crate::understood::Number]) -> String {
    if words.iter().any(|w| _TEEN_WORDS.contains(w)) {
        return "teen".into();
    }
    for figure in numbers {
        if figure.unit.as_deref() != Some("years") {
            continue;
        }
        let age = match &figure.value {
            Some(crate::understood::Num::Int(i)) => *i,
            Some(crate::understood::Num::Float(f)) if f.is_finite() => f.trunc() as i64,
            _ => continue,
        };
        if (13..=19).contains(&age) {
            return "teen".into();
        }
        if age <= 12 {
            return "child".into();
        }
    }
    if words.iter().any(|w| _CHILD_OF_THEIRS.contains(w)) {
        return "child".into();
    }
    String::new()
}

lazy_re!(NOT_A_WORD_CHAR, "[^a-z']+");

/// A comic to read, by the reader the line or the chat names.
pub fn _comic_pick(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if _asks_if_a_comic_suits_an_age(understood) {
        return Some(_COMIC_SUITABILITY.clone());
    }
    let words = lower_words(&understood.words);
    if _reader_named(&words, &understood.numbers).is_empty() {
        let named = _the_kind_named(understood);
        if !named.is_empty() {
            return Some(named);
        }
    }
    let mut reader = _reader_named(&words, &understood.numbers);
    if reader.is_empty() {
        for turn in memory.turns.iter().rev() {
            let said = turn.user.to_lowercase();
            let earlier: Vec<String> = NOT_A_WORD_CHAR.split(&said).into_iter().filter(|w| !w.is_empty()).collect();
            reader = _reader_named(&earlier, &[]);
            if !reader.is_empty() {
                break;
            }
        }
    }
    Some(match reader.as_str() {
        "teen" => _COMIC_FOR_A_TEEN.clone(),
        "child" => _COMIC_FOR_A_CHILD.clone(),
        _ => _COMIC_FOR_A_GROWN_UP.clone(),
    })
}

res!(_LIKE_A_GENRE);

/// False when the line asks for a book like a genre ("something like a thriller"), which the
/// genre rows answer.
pub fn _a_genre_is_the_taste(understood: &Understood) -> bool {
    !_LIKE_A_GENRE.is_match(&or(&understood.clean, &understood.text).to_lowercase())
}

// ---------------------------------------------------------------------------------------------
// A longer one, another one: read by the answers module after a verse or a story

res!(_LONGER_ASK, _VERSE_IN_THE_LAST_TURN, _FIXED_COUNT_FORM, _POEM_FORM);
strs!(_LONGER_NOT_THIS, VERSE_LONGER, STORY_LONGER);

/// The last verse or story again with more on the end, for "make it longer".
pub fn the_verse_longer(text_: &str, memory: &Memory) -> Option<String> {
    let line = text::squash(&text_.to_lowercase());
    if !_LONGER_ASK.is_match(&line) {
        return None;
    }
    if _LONGER_NOT_THIS.iter().any(|one| line.contains(one.as_str())) {
        return None;
    }
    let mut said = String::new();
    let mut thing = String::new();
    for turn in memory.turns.iter().rev() {
        said = turn.bot.clone();
        if !said.is_empty() {
            thing = _thing(&turn.user);
            break;
        }
    }
    if said.is_empty() || !said.contains('\n') {
        return None;
    }
    if said.starts_with("Once more") {
        said = said.split_once('\n').map(|(_, rest)| rest.to_string()).unwrap_or(said);
    }
    let (mut lead, mut rest) = match said.split_once('\n') {
        Some((l, r)) => (l.to_string(), r.to_string()),
        None => (String::new(), said.clone()),
    };
    if !text::rstrip(&lead).ends_with(':') {
        lead = String::new();
        rest = said.clone();
    }
    let probe = if lead.is_empty() { rest.split('\n').next().unwrap_or("").to_string() } else { lead.clone() };
    if !_VERSE_IN_THE_LAST_TURN.is_match(&probe) {
        return None;
    }
    if _FIXED_COUNT_FORM.is_match(&lead) {
        return None;
    }
    if thing.is_empty() {
        thing = GENERIC_STORY_THING.clone();
    }
    let pool: &[String] = if _POEM_FORM.is_match(&lead) { &VERSE_LONGER } else { &STORY_LONGER };
    let more = format_named(_picked(pool, &thing), &[("thing", &thing)]);
    Some(if lead.is_empty() { format!("{rest}\n{more}") } else { format!("{lead}\n{rest}\n{more}") })
}

res!(_BARE_COUNT_AGAIN, _SECOND_VERSE_ABOUT, _ASK_POLITENESS, _NO_SUBJECT, _THE_SUBJECT_IN_A_LEAD, _THE_COUNT_AGAIN);
lazy_re!(
    NOT_A_SECOND_VERSE,
    "\\b(?:tell me more|more information|what else|explain|meaning|why is|how do i|how to|write|writing|wrote|read|reading|book|books)\\b"
);

/// Whether the line asks for one more verse ("another one", "one more about the sea").
pub fn _a_second_verse_is_asked_for(text_: &str) -> bool {
    let line = text::squash(&text_.to_lowercase());
    if line.is_empty() {
        return false;
    }
    if _BARE_COUNT_AGAIN.match_(&line).is_some() {
        return true;
    }
    if NOT_A_SECOND_VERSE.is_match(&line) {
        return false;
    }
    _SECOND_VERSE_ABOUT.is_match(&line)
}

/// The subject after "another one about", without the politeness round it.
pub fn _the_subject_from_the_ask(after_the_ask: &str) -> String {
    let words: Vec<&str> = text::split_ref(after_the_ask).into_iter().filter(|one| !_ASK_POLITENESS.is_match(one)).collect();
    let joined = words.join(" ");
    let subject = text::strip_chars(&joined, " ,.!?;:");
    if subject.is_empty() || _NO_SUBJECT.match_(subject).is_some() {
        return String::new();
    }
    subject.to_string()
}

lazy_re!(LIMERICK_WORD_I, "\\blimerick\\b", pyre::I);
lazy_re!(HAIKU_WORD_I, "\\bhaiku\\b", pyre::I);

/// The ask for a subject when the last verse had none.
pub fn _the_second_one_short(lead: &str) -> String {
    let form = if LIMERICK_WORD_I.is_match(lead) {
        "limerick"
    } else if HAIKU_WORD_I.is_match(lead) {
        "haiku"
    } else {
        "poem"
    };
    format!(
        "That last one was about nothing in particular - give me one word (the sea, my nan, the bus) \
         and the next {form} will be about that."
    )
}

lazy_re!(CHILD_STORY_LEAD, "(A short one for [^:]{1,40}):\\s", pyre::I);

/// "A short one for a 5 year old:" when the last reply was a child's story, else "".
pub fn _a_child_story_lead(said: &str) -> String {
    match CHILD_STORY_LEAD.match_(said) {
        Some(found) => format!("{}:", found.str(1)),
        None => String::new(),
    }
}

maps!(_FORM_OF_A_WORD);

/// The form a repeat ask names ("another poem"): its last form word, or "".
pub fn _the_form_a_repeat_ask_names(line: &str) -> String {
    let mut named = String::new();
    for word in LETTER_RUN.findall_str(line) {
        if let Some(form) = _FORM_OF_A_WORD.get(&word).filter(|f| !f.is_empty()) {
            named = form.clone();
        }
    }
    named
}

lazy_re!(STORY_OR_TALE_I, "\\bstor(?:y|ies)\\b|\\btale\\b", pyre::I);
lazy_re!(HERE_IS_ONE_I, "here is one\\b", pyre::I);
lazy_re!(RAP_WORD_I, "\\brap\\b", pyre::I);
lazy_re!(SONG_WORD_I, "\\bsong\\b", pyre::I);
lazy_re!(RHYME_WORD_I, "\\brhyme\\b", pyre::I);

/// The form of the last verse, read from its lead line.
pub fn _the_form_of_the_lead(lead: &str) -> String {
    if HAIKU_WORD_I.is_match(lead) {
        return "haiku".into();
    }
    if LIMERICK_WORD_I.is_match(lead) {
        return "limerick".into();
    }
    if STORY_OR_TALE_I.is_match(lead) || HERE_IS_ONE_I.match_(lead).is_some() {
        return "story".into();
    }
    if RAP_WORD_I.is_match(lead) {
        return "rap".into();
    }
    if SONG_WORD_I.is_match(lead) {
        return "song".into();
    }
    if RHYME_WORD_I.is_match(lead) {
        return "rhyme".into();
    }
    if _POEM_FORM.is_match(lead) {
        return "poem".into();
    }
    String::new()
}

/// One more verse or story in the form the chat just had, for "another one (about X)".
pub fn the_next_verse(text_: &str, memory: &Memory) -> Option<String> {
    if !_a_second_verse_is_asked_for(text_) {
        return None;
    }
    let line = text::squash(&text_.to_lowercase());
    let said = memory.turns.iter().rev().map(|t| t.bot.as_str()).find(|b| !b.is_empty()).unwrap_or("").to_string();
    if said.is_empty() {
        return None;
    }
    let told = memory.turns.iter().map(|t| t.bot.as_str()).collect::<Vec<_>>().join(" ");
    let child = _a_child_story_lead(&said);
    if !child.is_empty() {
        for story in CHILD_STORIES.iter() {
            let head: String = story.chars().take(70).collect();
            if !told.contains(&head) {
                return Some(format!("{child} {story}"));
            }
        }
        return None;
    }
    let (lead, _) = said.split_once('\n')?;
    if !text::rstrip(lead).ends_with(':') {
        return None;
    }
    let mut form = _the_form_a_repeat_ask_names(&line);
    if form.is_empty() {
        form = _the_form_of_the_lead(lead);
    }
    if form.is_empty() {
        return None;
    }
    let found = _SECOND_VERSE_ABOUT.search(&line);
    let mut subject = _the_subject_from_the_ask(found.as_ref().map(|f| &line[f.end()..]).unwrap_or(""));
    let mut thing = if subject.is_empty() { String::new() } else { _thing(&line) };
    if subject.is_empty() {
        for one in memory.turns.iter().rev().map(|t| t.bot.as_str()).filter(|b| !b.is_empty()) {
            let Some(named) = _THE_SUBJECT_IN_A_LEAD.search(one.split('\n').next().unwrap_or("")) else { continue };
            subject = text::strip(_the_subject_from_the_ask(named.str(1)).split(',').next().unwrap_or("")).to_string();
            if !subject.is_empty() {
                break;
            }
        }
    }
    if thing.is_empty() && !subject.is_empty() {
        thing = _thing(&format!("about {subject}"));
    }
    if form == "haiku" || form == "limerick" {
        let about_it = if !subject.is_empty() && !thing.is_empty() { _say(&form, &thing, &subject, None) } else { None };
        if let Some(about_it) = about_it {
            if about_it.contains('\n') && !told.contains(&about_it) {
                return Some(about_it);
            }
        }
        if let Some(lines) = NO_SUBJECT_VERSE.get(&form).filter(|l| !l.is_empty()) {
            if !told.contains(lines[0].as_str()) {
                return Some(format!("Another {form}:\n{}", lines.join("\n")));
            }
        }
        if !thing.is_empty() {
            if let Some(verse) = POEMS.get(&thing).filter(|v| !v.is_empty()) {
                let lines = with_thing(verse, &thing);
                if !told.contains(lines[0].as_str()) {
                    return Some(format!(
                        "That is every {form} I have about {subject} - a short poem about it instead:\n{}",
                        lines.join("\n")
                    ));
                }
            }
        }
        return Some(_the_second_one_short(lead));
    }
    if form == "story" {
        if subject.is_empty() || thing.is_empty() {
            return Some(_the_second_one_short(lead));
        }
        return Some(format!(
            "{}\n{}",
            format_named(STORY_LEADING.get("story").expect("KeyError"), &[("subject", &subject)]),
            _the_opening_not_said_yet(&thing, memory, "")
        ));
    }
    if !(_THE_COUNT_AGAIN.is_match(&line) || _BARE_COUNT_AGAIN.match_(&line).is_some()) {
        return None;
    }
    if subject.is_empty() || thing.is_empty() {
        return Some(_the_second_one_short(lead));
    }
    let noun = &form;
    let poems = POEMS.get(&thing).cloned().unwrap_or_default();
    for verse in [&poems, &*GENERIC_POEM, &*GENERIC_POEM_TWO] {
        if verse.is_empty() {
            continue;
        }
        let lines = with_thing(verse, &thing);
        if !told.contains(lines[0].as_str()) {
            return Some(format!("Another {noun} about {subject}:\n{}", lines.join("\n")));
        }
    }
    Some(format!("Another {noun} about {subject}:\n{}", with_thing(&GENERIC_POEM_TWO, &thing).join("\n")))
}

/// `_film_list(text).<locals>.said`: a film row trimmed to the running time the line gives.
pub fn _film_list_said(text_: &str, _memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = format!("{} {}", understood.clean, understood.norm_text);
    _films_within(Some(text_), Some(&line))
}

// ---------------------------------------------------------------------------------------------
// Music words, instruments, practice

strs!(_POSSESSIVE, _LOOKING_FOR_THE_THING, _A_MUSIC_WORD, _A_THING_OF_THEIRS);
sets!(_NEEDS_A_MUSIC_WORD);
maps!(MUSIC_TERMS, _INSTRUMENT_COUNTS);
list_maps!(_MUSIC_ELSEWHERE, _MUSIC_WORD_BESIDE);

/// MUSIC_TERMS' words, longest first (a stable sort, as Python's).
static MUSIC_TERMS_LONGEST_FIRST: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut words: Vec<String> = MUSIC_TERMS.keys().cloned().collect();
    words.sort_by_key(|w| std::cmp::Reverse(w.chars().count()));
    words
});

/// Whether the word at `at` is somebody's own thing ("my keys"), not the music word.
pub fn _somebodys_own_thing(said: &[String], at: usize) -> bool {
    if at == 0 {
        return false;
    }
    let before = said[at - 1].as_str();
    let two = if at >= 2 { said[at - 2..at].join(" ") } else { String::new() };
    if !_POSSESSIVE.iter().any(|p| p == before) && !(at >= 2 && _POSSESSIVE.contains(&two)) {
        return false;
    }
    if before == "your" || two.ends_with("your") {
        return _LOOKING_FOR_THE_THING.iter().any(|one| said.contains(one));
    }
    true
}

/// False when the music word in the line belongs to another subject ("the key to the door").
pub fn _music_elsewhere(understood: &Understood) -> bool {
    let said = lower_words(&understood.words);
    let flat = said.join(" ");
    for word in MUSIC_TERMS_LONGEST_FIRST.iter() {
        let plural = format!("{word}s");
        let forms: Vec<usize> = said.iter().enumerate().filter(|(_, one)| *one == word || **one == plural).map(|(at, _)| at).collect();
        if !forms.is_empty() || (word.contains(' ') && flat.contains(word.as_str())) {
            let elsewhere: Vec<&String> = _A_MUSIC_WORD.iter().filter(|one| *one != word && **one != plural).collect();
            if _A_THING_OF_THEIRS.contains(word)
                && forms.iter().any(|at| _somebodys_own_thing(&said, *at))
                && !elsewhere.iter().any(|one| flat.contains(one.as_str()))
            {
                return false;
            }
            if _NEEDS_A_MUSIC_WORD.contains(word) && !_A_MUSIC_WORD.iter().any(|one| flat.contains(one.as_str())) {
                return false;
            }
            return !_MUSIC_ELSEWHERE.get(word).is_some_and(|others| others.iter().any(|one| flat.contains(one.as_str())));
        }
    }
    true
}

/// What a music word means.
pub fn _music_term(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = lower_words(&understood.words);
    let flat = said.join(" ");
    for word in MUSIC_TERMS_LONGEST_FIRST.iter() {
        if said.contains(word) || (word.contains(' ') && flat.contains(word.as_str())) {
            if let Some(beside) = _MUSIC_WORD_BESIDE.get(word).filter(|b| !b.is_empty()) {
                if !beside.iter().any(|one| flat.contains(one.as_str())) {
                    continue;
                }
            }
            return Some(MUSIC_TERMS[word].clone());
        }
    }
    Some(
        "A chord is three or more notes together, the key is the home note a song keeps returning \
         to, and an octave is the same note higher or lower. The time signature says how many beats \
         are in a bar. Ask about one and I'll give you its answer."
            .into(),
    )
}

/// Whether "bridge" in the line is the part of a song.
pub fn _a_bridge_in_music(understood: &Understood) -> bool {
    let low = format!(" {} ", understood.clean.to_lowercase());
    let stripped = text::strip(&low);
    [
        "song", "songs", "chorus", "verse", "music", "musical", "track", "tune", "album", "band", "bridge of the song",
        "in the song",
    ]
    .iter()
    .any(|word| low.contains(&format!(" {word} ")) || stripped.starts_with(&format!("{word} ")))
}

/// How many strings (or keys) an instrument has.
pub fn _instrument_count(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let flat = lower_words(&understood.words).join(" ");
    let mut names: Vec<&String> = _INSTRUMENT_COUNTS.keys().collect();
    names.sort_by_key(|n| std::cmp::Reverse(n.chars().count()));
    for name in names {
        if flat.contains(name.as_str()) {
            return Some(_INSTRUMENT_COUNTS[name].clone());
        }
    }
    Some(
        "A guitar has six strings, a violin four, a bass four and a piano 88 keys. Ask me about the \
         one you have and I'll tell you the count and the tuning."
            .into(),
    )
}

strs!(_READ_MUSIC_WORDS);
text_of!(_PRACTICE_READ_MUSIC, _PRACTICE_CORE);

/// How to practise an instrument (and to read music, when the line asks).
pub fn _music_practice(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = lower_words(&understood.words).join(" ");
    if _READ_MUSIC_WORDS.iter().any(|w| said.contains(w.as_str())) {
        return Some(format!("{} {}", *_PRACTICE_READ_MUSIC, *_PRACTICE_CORE));
    }
    Some(_PRACTICE_CORE.clone())
}

// ---------------------------------------------------------------------------------------------
// Games

/// False when the game is for a crowd (a family, a party), which the party rows answer.
pub fn _not_a_crowd(understood: &Understood) -> bool {
    if understood.topic.as_deref() == Some("family-and-parenting") {
        return false;
    }
    // TYPE(understood.leans_on): Python reads the leaned-on line's words, subject and text for
    // crowd words when leans_on is a dict; the typed reading keeps it as a string, so nothing here.
    true
}

strs!(_CHILD_WORDS);
lazy_re!(CHAT_AGES, "\\b(\\d{1,2})\\s*(?:years? old|year old|yo)\\b");

/// Whether a child under 13 came up in the last eight lines.
pub fn _a_child_in_the_chat(memory: &Memory) -> bool {
    for turn in last_turns(memory, 8) {
        let words = turn.user.to_lowercase();
        if words.is_empty() || !_CHILD_WORDS.iter().any(|w| words.contains(w.as_str())) {
            continue;
        }
        let ages: Vec<i64> = CHAT_AGES.findall_str(&words).iter().map(|a| a.parse().expect("int()")).collect();
        if ages.iter().min().is_some_and(|m| *m >= 13) {
            continue;
        }
        return true;
    }
    false
}

static GAME_PICKS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    g("GAME_PICKS").items().iter().map(|row| (row.items()[0].strs(), row.items()[1].str().to_string())).collect()
});

/// A game to play, by platform or company.
pub fn _the_game_pick(memory: &mut Memory, reading: &Understood) -> Option<String> {
    if dup::keeping_an_animal(reading) {
        return Some(String::new());
    }
    let line = format!("{} {}", reading.clean, reading.norm_text).to_lowercase();
    if _a_child_in_the_chat(memory) {
        return Some(
            "*Dobble*, *UNO* and *Jenga* work from about five up, and *Catan Junior* if they are ready \
             for a proper board game. A treasure hunt round the house costs nothing and takes the \
             longest."
                .into(),
        );
    }
    for (words, said) in GAME_PICKS.iter() {
        if words.iter().any(|w| line.contains(w.as_str())) {
            return Some(said.clone());
        }
    }
    Some(
        "*Hades* is the one I'd hand you first: fast, short runs, and it never wastes your time. \
         *Stardew Valley* if you want calm, and *The Last of Us* if you want a story to sit through."
            .into(),
    )
}

strs!(_PLATFORMS, _MACHINE_SHAPES, GAME_JUDGED, NAMED_GAME_TITLES);
res!(_BUYING_THE_MACHINE);

/// Whether the line is about buying a console or a PC, not a game to play on it.
pub fn _a_hardware_decision(understood: &Understood) -> bool {
    let clean = format!(" {} ", understood.clean.to_lowercase().replace('-', " "));
    if !_PLATFORMS.iter().any(|w| clean.contains(w.as_str())) {
        return false;
    }
    if _BUYING_THE_MACHINE.is_match(&clean) {
        return true;
    }
    _MACHINE_SHAPES.iter().any(|s| clean.contains(s.as_str()))
}

/// Whether the line asks for a game to play (not for a crowd, a machine or a verdict).
pub fn _a_game_pick_line(understood: &Understood) -> bool {
    if !_not_a_crowd(understood) {
        return false;
    }
    if _a_hardware_decision(understood) {
        return false;
    }
    let clean = format!(" {} ", understood.clean.to_lowercase().replace('-', " "));
    if [" get better at ", " get good at ", " how do i improve ", " how can i improve "].iter().any(|s| clean.contains(s)) {
        return false;
    }
    let judged = GAME_JUDGED.iter().any(|s| clean.contains(s.as_str()));
    if judged {
        for title in NAMED_GAME_TITLES.iter() {
            if rx(format!("(?<![a-z]){}(?![a-z])", pyre::escape(title))).is_match(&clean) {
                return false;
            }
        }
    }
    true
}

/// A game to play alone.
pub fn _the_solo_game(memory: &mut Memory, _reading: &Understood) -> Option<String> {
    if _a_child_in_the_chat(memory) {
        return Some(
            "*Dobble* is fine with one player as a speed game, and *UNO* works solo if you play the \
             house down. *Catan Junior* really needs two, though."
                .into(),
        );
    }
    Some(
        "*Stardew Valley* is the one: farm, fish, befriend the village, nothing timed against you, \
         and it runs on almost anything. *Hades* if you want ten-minute runs with a fight in them, \
         or *Unpacking* for an evening of something quiet."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// The entries' lambdas

/// Entry guard (line 7915): a book pick, not a booking, and no mood named.
fn _book_pick_guard(understood: &Understood) -> bool {
    !the_book_is_a_reservation(understood) && _no_named_mood(understood)
}

/// Entry guard (line 8171): a comic pick, not a parent telling the bot about their child.
fn _comic_pick_guard(understood: &Understood) -> bool {
    !_a_child_who_reads_already(understood)
}

/// Entry guard (line 8520): party games, unless the line owns a deed to repair.
fn _party_games_guard(reading: &Understood) -> bool {
    !d2_shared::owns_the_deed_reading(reading)
}

// ---------------------------------------------------------------------------------------------
// Registry tables

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.entertainment:<lambda>:7915", |_, u| _book_pick_guard(u)),
    ("bot.knowledge.entertainment:<lambda>:8171", |_, u| _comic_pick_guard(u)),
    ("bot.knowledge.entertainment:<lambda>:8520", |_, u| _party_games_guard(u)),
    ("bot.knowledge.entertainment:_a_bridge_in_music:9870", |_, u| _a_bridge_in_music(u)),
    ("bot.knowledge.entertainment:_a_chess_line:22", |_, u| _a_chess_line(u)),
    ("bot.knowledge.entertainment:_a_country_is_a_country:5513", |_, u| _a_country_is_a_country(u)),
    ("bot.knowledge.entertainment:_a_documentary_is_wanted:5662", |_, u| _a_documentary_is_wanted(u)),
    ("bot.knowledge.entertainment:_a_film_ask:1790", |_, u| _a_film_ask(u)),
    ("bot.knowledge.entertainment:_a_game_pick_line:10512", |_, u| _a_game_pick_line(u)),
    ("bot.knowledge.entertainment:_a_genre_is_the_taste:7040", |_, u| _a_genre_is_the_taste(u)),
    ("bot.knowledge.entertainment:_a_listings_ask_near_me:2944", |_, u| _a_listings_ask_near_me(u)),
    ("bot.knowledge.entertainment:_a_programme_is_wanted:5976", |_, u| _a_programme_is_wanted(u)),
    ("bot.knowledge.entertainment:_music_elsewhere:9804", |_, u| _music_elsewhere(u)),
    ("bot.knowledge.entertainment:_names_the_evening:3367", |_, u| _names_the_evening(u)),
    ("bot.knowledge.entertainment:_names_the_weather:3360", |_, u| _names_the_weather(u)),
    ("bot.knowledge.entertainment:_no_city_is_named:43", |_, u| _no_city_is_named(u)),
    ("bot.knowledge.entertainment:_no_named_mood:3444", |_, u| _no_named_mood(u)),
    ("bot.knowledge.entertainment:_not_a_reservation:2863", |_, u| _not_a_reservation(u)),
    ("bot.knowledge.entertainment:_the_scare_is_wanted:1799", |_, u| _the_scare_is_wanted(u)),
    ("bot.knowledge.entertainment:_the_subject_is_wanted:5651", |_, u| _the_subject_is_wanted(u)),
    ("bot.knowledge.entertainment:not_asking_what_to_watch_for:3747", |_, u| not_asking_what_to_watch_for(u)),
];

/// `_a_second_one_read_first(writer, form).<locals>.write` from its captured values.
fn second_one_write(f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    let writer = writer_from_fn(f.cell("writer").and_then(Value::as_fn).expect("writer cell"));
    let form = f.cell("form").map(Value::str).unwrap_or("");
    _a_second_one_read_first_write(&writer, form, m, u)
}

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.entertainment:_a_pick_in_any_medium:3670", |_, m, u| _a_pick_in_any_medium(m, u)),
    ("bot.knowledge.entertainment:_a_pick_of_the_other_mood:3654", |_, m, u| _a_pick_of_the_other_mood(m, u)),
    ("bot.knowledge.entertainment:_a_second_one_read_first.<locals>.write:5367", second_one_write),
    ("bot.knowledge.entertainment:_activity_start:2229", |_, m, u| _activity_start(m, u)),
    ("bot.knowledge.entertainment:_after_named:2575", |_, m, u| _after_named(m, u)),
    ("bot.knowledge.entertainment:_an_hour_or_a_day:6824", |_, m, u| _an_hour_or_a_day(m, u)),
    ("bot.knowledge.entertainment:_another_one:5227", |_, m, u| _another_one(m, u)),
    ("bot.knowledge.entertainment:_book_about:2817", |_, m, u| _book_about(m, u)),
    ("bot.knowledge.entertainment:_book_for_the_age:6780", |_, m, u| _book_for_the_age(m, u)),
    ("bot.knowledge.entertainment:_book_like:1078", |_, m, u| _book_like(m, u)),
    ("bot.knowledge.entertainment:_book_recommend:2966", |_, m, u| _book_recommend(m, u)),
    ("bot.knowledge.entertainment:_book_thriller:2805", |_, m, u| _book_thriller(m, u)),
    ("bot.knowledge.entertainment:_buzz:2469", |_, m, u| _buzz(m, u)),
    ("bot.knowledge.entertainment:_card_game:2440", |_, m, u| _card_game(m, u)),
    ("bot.knowledge.entertainment:_character_name:5166", |_, m, u| _character_name(m, u)),
    ("bot.knowledge.entertainment:_comic_pick:6993", |_, m, u| _comic_pick(m, u)),
    ("bot.knowledge.entertainment:_family_film:1324", |_, m, u| _family_film(m, u)),
    ("bot.knowledge.entertainment:_film_kind_reply:6581", |_, m, u| _film_kind_reply(m, u)),
    ("bot.knowledge.entertainment:_film_like:1615", |_, m, u| _film_like(m, u)),
    ("bot.knowledge.entertainment:_film_list.<locals>.said:9533", |f, m, u| {
        _film_list_said(f.cell("text").map(Value::str).unwrap_or(""), m, u)
    }),
    ("bot.knowledge.entertainment:_film_under:6447", |_, m, u| _film_under(m, u)),
    ("bot.knowledge.entertainment:_genre_start:2414", |_, m, u| _genre_start(m, u)),
    ("bot.knowledge.entertainment:_in_tune:2454", |_, m, u| _in_tune(m, u)),
    ("bot.knowledge.entertainment:_instrument_count:9917", |_, m, u| _instrument_count(m, u)),
    ("bot.knowledge.entertainment:_listen_when:1194", |_, m, u| _listen_when(m, u)),
    ("bot.knowledge.entertainment:_mood_pick.<locals>.pick:3527", |f, m, u| {
        _mood_pick_pick(f.cell("picks").unwrap_or(&Value::None), m, u)
    }),
    ("bot.knowledge.entertainment:_music_practice:9965", |_, m, u| _music_practice(m, u)),
    ("bot.knowledge.entertainment:_music_term:9830", |_, m, u| _music_term(m, u)),
    ("bot.knowledge.entertainment:_nickname:5177", |_, m, u| _nickname(m, u)),
    ("bot.knowledge.entertainment:_party_games_for_kids:1162", |_, m, u| _party_games_for_kids(m, u)),
    ("bot.knowledge.entertainment:_pick_limit_reply:6702", |_, m, u| _pick_limit_reply(m, u)),
    ("bot.knowledge.entertainment:_pick_plain:6637", |_, m, u| _pick_plain(m, u)),
    ("bot.knowledge.entertainment:_quiz_name:5265", |_, m, u| _quiz_name(m, u)),
    ("bot.knowledge.entertainment:_show_about:6001", |_, m, u| _show_about(m, u)),
    ("bot.knowledge.entertainment:_story_idea:4594", |_, m, u| _story_idea(m, u)),
    ("bot.knowledge.entertainment:_the_game_pick:10443", |_, m, u| _the_game_pick(m, u)),
    ("bot.knowledge.entertainment:_the_gentle_list:1518", |_, m, u| _the_gentle_list(m, u)),
    ("bot.knowledge.entertainment:_the_solo_game:10546", |_, m, u| _the_solo_game(m, u)),
    ("bot.knowledge.entertainment:_who_wrote:782", |_, m, u| _who_wrote(m, u)),
    ("bot.knowledge.entertainment:_worth_it:2481", |_, m, u| _worth_it(m, u)),
    ("bot.knowledge.entertainment:_worth_it_unnamed:2496", |_, m, u| _worth_it_unnamed(m, u)),
    ("bot.knowledge.entertainment:who_made:538", |_, m, u| who_made(m, u)),
];


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/knowledge/entertainment.py:3259 `_mood_key_words(moods, drop=())` (never called on the trace corpus)
    pub fn _mood_key_words(moods: &Value, drop: &Value) -> Value {
        crate::unported!("bot.knowledge.entertainment:_mood_key_words")
    }

    /// Python: bot/knowledge/entertainment.py:3291 `_mood_signals(words)` (never called on the trace corpus)
    pub fn _mood_signals(words: &[String]) -> Value {
        crate::unported!("bot.knowledge.entertainment:_mood_signals")
    }

    /// Python: bot/knowledge/entertainment.py:3525 `_mood_pick(picks)` (never called on the trace corpus)
    pub fn _mood_pick(picks: &Value) -> Value {
        crate::unported!("bot.knowledge.entertainment:_mood_pick")
    }

    /// Python: bot/knowledge/entertainment.py:3586 `_picks_by_title()` (never called on the trace corpus)
    pub fn _picks_by_title() -> Value {
        crate::unported!("bot.knowledge.entertainment:_picks_by_title")
    }

    /// Python: bot/knowledge/entertainment.py:5336 `_with_the_detail_asked_for(build, form)` (never called on the trace corpus)
    pub fn _with_the_detail_asked_for(build: &Value, form: &Value) -> Value {
        crate::unported!("bot.knowledge.entertainment:_with_the_detail_asked_for")
    }

    /// Python: bot/knowledge/entertainment.py:5358 `_a_second_one_read_first(writer, form)` (never called on the trace corpus)
    pub fn _a_second_one_read_first(writer: &Value, form: &Value) -> Value {
        crate::unported!("bot.knowledge.entertainment:_a_second_one_read_first")
    }

    /// Python: bot/knowledge/entertainment.py:9526 `_film_list(text)` (never called on the trace corpus)
    pub fn _film_list(text: &str) -> Value {
        crate::unported!("bot.knowledge.entertainment:_film_list")
    }

}

