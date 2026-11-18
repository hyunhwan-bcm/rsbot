//! Python `re` semantics on top of a Rust regex engine.
//!
//! PyBot's 10k patterns are written for CPython's `re`. [`Re`] takes the pattern text and the
//! Python flag bits exactly as `re.compile(pattern, flags)` got them, and offers the calls the
//! bot makes: `search`, `match_`, `fullmatch`, `finditer`, `findall`, `sub`, `split`.
//!
//! Two engines are available. PCRE2 (with JIT) is the default: it is a backtracking engine like
//! CPython's, and reads Python syntax almost unchanged (`$` before a final newline, literal `{`,
//! fixed-width lookbehind, escaped punctuation). `fancy-regex` is pure Rust; its syntax differs
//! more, so the pattern is translated first. `harness`'s regex check compares both against
//! CPython on every pattern and a corpus of real lines.
//!
//! Offsets are byte offsets into the `&str`, as in Rust. Python's are code points; the two agree
//! on ASCII text, and [`crate::py::text`] has helpers for the places that count characters.

use std::sync::OnceLock;

pub const I: u32 = 2; // re.IGNORECASE
pub const L: u32 = 4; // re.LOCALE
pub const M: u32 = 8; // re.MULTILINE
pub const S: u32 = 16; // re.DOTALL
pub const U: u32 = 32; // re.UNICODE (the default for str patterns)
pub const X: u32 = 64; // re.VERBOSE
pub const A: u32 = 256; // re.ASCII

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engine {
    Pcre2,
    Fancy,
}

static ENGINE: OnceLock<Engine> = OnceLock::new();

/// The engine every [`Re`] compiles with; set once, before the first pattern is used.
pub fn set_engine(engine: Engine) {
    let _ = ENGINE.set(engine);
}

fn engine() -> Engine {
    *ENGINE.get_or_init(|| match std::env::var("RSBOT_REGEX").as_deref() {
        Ok("fancy") => Engine::Fancy,
        _ => Engine::Pcre2,
    })
}

enum Compiled {
    Pcre2(pcre2::bytes::Regex),
    Fancy(fancy_regex::Regex),
}

impl Compiled {
    fn build(pattern: &str, flags: u32, eng: Engine) -> Result<Compiled, String> {
        match eng {
            Engine::Pcre2 => {
                let p = translate_pcre2(pattern);
                let mut b = pcre2::bytes::RegexBuilder::new();
                b.utf(true)
                    .ucp(flags & A == 0)
                    .caseless(flags & I != 0)
                    .multi_line(flags & M != 0)
                    .dotall(flags & S != 0)
                    .extended(flags & X != 0)
                    .jit_if_available(true);
                b.build(&p).map(Compiled::Pcre2).map_err(|e| format!("{e} in {p:?}"))
            }
            Engine::Fancy => {
                let p = translate_fancy(pattern, flags);
                fancy_regex::RegexBuilder::new(&p)
                    .backtrack_limit(10_000_000)
                    .build()
                    .map(Compiled::Fancy)
                    .map_err(|e| format!("{e} in {p:?}"))
            }
        }
    }

    /// Spans of every group (0 = whole match) of the first match at or after `pos`.
    fn captures_at(&self, text: &str, pos: usize) -> Option<Vec<Option<(usize, usize)>>> {
        match self {
            Compiled::Pcre2(r) => {
                let mut locs = r.capture_locations();
                match r.captures_read_at(&mut locs, text.as_bytes(), pos) {
                    Ok(Some(_)) => Some((0..locs.len()).map(|i| locs.get(i)).collect()),
                    Ok(None) => None,
                    Err(_) => None, // a match limit hit: Python would have gone on, we say no match
                }
            }
            Compiled::Fancy(r) => match r.captures_from_pos(text, pos) {
                Ok(Some(c)) => Some((0..c.len()).map(|i| c.get(i).map(|m| (m.start(), m.end()))).collect()),
                _ => None,
            },
        }
    }

    fn group_names(&self) -> Vec<Option<String>> {
        match self {
            Compiled::Pcre2(r) => r.capture_names().to_vec(),
            Compiled::Fancy(r) => r.capture_names().map(|n| n.map(str::to_string)).collect(),
        }
    }
}

/// A compiled Python regex. Compilation is lazy: the pattern is built the first time it is used.
pub struct Re {
    pattern: String,
    flags: u32,
    plain: OnceLock<Result<Compiled, String>>,
    anchored: OnceLock<Result<Compiled, String>>,
    full: OnceLock<Result<Compiled, String>>,
    advance: OnceLock<Result<Compiled, String>>,
    names: OnceLock<Vec<Option<String>>>,
}

impl std::fmt::Debug for Re {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Re({:?}, {})", self.pattern, self.flags)
    }
}

impl Clone for Re {
    fn clone(&self) -> Self {
        Re::new(&self.pattern, self.flags)
    }
}

/// One match: the text it was found in and the span of every group (byte offsets).
#[derive(Clone, Debug)]
pub struct Match<'t> {
    pub text: &'t str,
    spans: Vec<Option<(usize, usize)>>,
    names: &'t [Option<String>],
}

impl<'t> Match<'t> {
    pub fn start(&self) -> usize {
        self.spans[0].unwrap().0
    }
    pub fn end(&self) -> usize {
        self.spans[0].unwrap().1
    }
    pub fn as_str(&self) -> &'t str {
        let (s, e) = self.spans[0].unwrap();
        &self.text[s..e]
    }
    /// `m.group(i)`: None when the group did not take part in the match.
    pub fn get(&self, i: usize) -> Option<&'t str> {
        self.spans.get(i).copied().flatten().map(|(s, e)| &self.text[s..e])
    }
    /// `m.group(i) or ""`.
    pub fn str(&self, i: usize) -> &'t str {
        self.get(i).unwrap_or("")
    }
    pub fn span(&self, i: usize) -> Option<(usize, usize)> {
        self.spans.get(i).copied().flatten()
    }
    /// `m.group("name")`.
    pub fn name(&self, name: &str) -> Option<&'t str> {
        let i = self.names.iter().position(|n| n.as_deref() == Some(name))?;
        self.get(i)
    }
    pub fn name_str(&self, name: &str) -> &'t str {
        self.name(name).unwrap_or("")
    }
    pub fn name_span(&self, name: &str) -> Option<(usize, usize)> {
        let i = self.names.iter().position(|n| n.as_deref() == Some(name))?;
        self.span(i)
    }
    /// Number of groups, not counting group 0 (Python's `len(m.groups())`).
    pub fn group_count(&self) -> usize {
        self.spans.len() - 1
    }
    /// `m.groups()`.
    pub fn groups(&self) -> Vec<Option<&'t str>> {
        (1..self.spans.len()).map(|i| self.get(i)).collect()
    }
    /// `m.start(i)` / `m.end(i)` (Python gives -1 for a group that did not match).
    pub fn start_of(&self, i: usize) -> Option<usize> {
        self.span(i).map(|s| s.0)
    }
    pub fn end_of(&self, i: usize) -> Option<usize> {
        self.span(i).map(|s| s.1)
    }
    /// `m.lastindex`: the highest-numbered group that closed... approximated as the last
    /// group that took part.
    pub fn lastindex(&self) -> Option<usize> {
        (1..self.spans.len()).rev().find(|&i| self.spans[i].is_some())
    }
}

/// What `findall` gives per match: the whole match (no groups), the one group, or all groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Found {
    One(String),
    Many(Vec<String>),
}

impl Found {
    pub fn one(&self) -> &str {
        match self {
            Found::One(s) => s,
            Found::Many(v) => v.first().map(String::as_str).unwrap_or(""),
        }
    }
}

impl Re {
    pub fn new(pattern: &str, flags: u32) -> Re {
        Re {
            pattern: pattern.to_string(),
            flags,
            plain: OnceLock::new(),
            anchored: OnceLock::new(),
            full: OnceLock::new(),
            advance: OnceLock::new(),
            names: OnceLock::new(),
        }
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }

    /// Compile now and report an error instead of panicking later.
    pub fn check(&self) -> Result<(), String> {
        match self.plain() {
            Ok(_) => Ok(()),
            Err(e) => Err(e.clone()),
        }
    }

    fn plain(&self) -> &Result<Compiled, String> {
        self.plain.get_or_init(|| Compiled::build(&self.pattern, self.flags, engine()))
    }

    fn compiled(cell: &OnceLock<Result<Compiled, String>>, build: impl FnOnce() -> Result<Compiled, String>) -> &Compiled {
        match cell.get_or_init(build) {
            Ok(c) => c,
            Err(e) => panic!("regex failed to compile: {e}"),
        }
    }

    fn names(&self) -> &[Option<String>] {
        self.names.get_or_init(|| Self::compiled(&self.plain, || unreachable!()).group_names())
    }

    fn wrap(&self, front: &str, back: &str) -> Result<Compiled, String> {
        // Leading global inline flags such as "(?i)" have to stay in front of the wrapper.
        let (lead, rest) = split_leading_flags(&self.pattern);
        let sep = if self.flags & X != 0 { "\n" } else { "" };
        let p = format!("{lead}{front}(?:{rest}{sep}){back}");
        Compiled::build(&p, self.flags, engine())
    }

    fn make<'t>(&'t self, text: &'t str, spans: Vec<Option<(usize, usize)>>) -> Match<'t> {
        Match { text, spans, names: self.names() }
    }

    /// `rx.search(text)`.
    pub fn search<'t>(&'t self, text: &'t str) -> Option<Match<'t>> {
        self.search_at(text, 0)
    }

    /// `rx.search(text, pos)`: the text before `pos` still counts for lookbehind and `\b`.
    pub fn search_at<'t>(&'t self, text: &'t str, pos: usize) -> Option<Match<'t>> {
        let c = Self::compiled(&self.plain, || Compiled::build(&self.pattern, self.flags, engine()));
        let spans = c.captures_at(text, pos)?;
        Some(self.make(text, spans))
    }

    /// `bool(rx.search(text))`.
    pub fn is_match(&self, text: &str) -> bool {
        self.search(text).is_some()
    }

    /// `rx.match(text)`: a match that starts at the beginning.
    pub fn match_<'t>(&'t self, text: &'t str) -> Option<Match<'t>> {
        let _ = self.plain();
        let c = Self::compiled(&self.anchored, || self.wrap("\\A", ""));
        let spans = c.captures_at(text, 0)?;
        Some(self.make(text, spans))
    }

    /// `rx.fullmatch(text)`.
    pub fn fullmatch<'t>(&'t self, text: &'t str) -> Option<Match<'t>> {
        let _ = self.plain();
        let c = Self::compiled(&self.full, || self.wrap("\\A", "\\z"));
        let spans = c.captures_at(text, 0)?;
        Some(self.make(text, spans))
    }

    /// `rx.finditer(text)`, with CPython's rule for empty matches: after an empty match the
    /// next one may not be empty at the same place (it may be a non-empty one right there).
    pub fn finditer<'t>(&'t self, text: &'t str) -> Vec<Match<'t>> {
        let c = Self::compiled(&self.plain, || Compiled::build(&self.pattern, self.flags, engine()));
        let mut out = Vec::new();
        let mut pos = 0;
        let mut must_advance = false;
        while pos <= text.len() {
            let Some(mut spans) = c.captures_at(text, pos) else { break };
            let (s, e) = spans[0].unwrap();
            if must_advance && s == pos && e == pos {
                // CPython's must_advance: a non-empty match right here still counts. PCRE2 reads
                // that as "anchored at the start offset, and not ending there".
                let here = if engine() == Engine::Pcre2 {
                    let adv = Self::compiled(&self.advance, || self.wrap("\\G", "(?!\\G)"));
                    adv.captures_at(text, pos)
                } else {
                    None
                };
                match here {
                    Some(sp) => spans = sp,
                    None => {
                        let Some(n) = next_char(text, pos) else { break };
                        match c.captures_at(text, n) {
                            Some(sp) => spans = sp,
                            None => break,
                        }
                    }
                }
            }
            let (s, e) = spans[0].unwrap();
            out.push(self.make(text, spans));
            must_advance = s == e;
            pos = e;
        }
        out
    }

    /// `rx.findall(text)`.
    pub fn findall(&self, text: &str) -> Vec<Found> {
        self.finditer(text)
            .into_iter()
            .map(|m| match m.group_count() {
                0 => Found::One(m.as_str().to_string()),
                1 => Found::One(m.str(1).to_string()),
                n => Found::Many((1..=n).map(|i| m.str(i).to_string()).collect()),
            })
            .collect()
    }

    /// `rx.findall(text)` for a pattern with at most one group: the strings.
    pub fn findall_str(&self, text: &str) -> Vec<String> {
        self.findall(text).into_iter().map(|f| f.one().to_string()).collect()
    }

    /// `rx.sub(repl, text)` with a Python replacement template (`\1`, `\g<1>`, `\g<name>`).
    pub fn sub(&self, repl: &str, text: &str) -> String {
        self.subn(repl, text, 0)
    }

    /// `rx.sub(repl, text, count=n)` (0 = all).
    pub fn subn(&self, repl: &str, text: &str, count: usize) -> String {
        let template = Template::parse(repl);
        self.sub_with(text, count, |m| template.expand(m))
    }

    /// `rx.sub(function, text)`.
    pub fn sub_fn(&self, text: &str, f: impl FnMut(&Match) -> String) -> String {
        self.sub_with(text, 0, f)
    }

    pub fn sub_with(&self, text: &str, count: usize, mut f: impl FnMut(&Match) -> String) -> String {
        let mut out = String::with_capacity(text.len());
        let mut last = 0;
        for (n, m) in self.finditer(text).into_iter().enumerate() {
            if count != 0 && n >= count {
                break;
            }
            out.push_str(&text[last..m.start()]);
            out.push_str(&f(&m));
            last = m.end();
        }
        out.push_str(&text[last..]);
        out
    }

    /// `rx.split(text)`: groups in the pattern are kept in the output, as in Python
    /// (a group that did not take part gives None, kept here as "").
    pub fn split(&self, text: &str) -> Vec<String> {
        self.split_opt(text).into_iter().map(|s| s.unwrap_or_default()).collect()
    }

    pub fn split_opt(&self, text: &str) -> Vec<Option<String>> {
        let mut out = Vec::new();
        let mut last = 0;
        for m in self.finditer(text) {
            out.push(Some(text[last..m.start()].to_string()));
            for i in 1..=m.group_count() {
                out.push(m.get(i).map(str::to_string));
            }
            last = m.end();
        }
        out.push(Some(text[last..].to_string()));
        out
    }
}

fn next_char(text: &str, pos: usize) -> Option<usize> {
    text[pos..].chars().next().map(|c| pos + c.len_utf8())
}

/// Split "(?i)(?x)rest" into ("(?i)(?x)", "rest").
fn split_leading_flags(p: &str) -> (&str, &str) {
    let mut i = 0;
    let b = p.as_bytes();
    while b.len() > i + 3 && b[i] == b'(' && b[i + 1] == b'?' {
        let mut j = i + 2;
        while j < b.len() && b[j].is_ascii_alphabetic() {
            j += 1;
        }
        if j < b.len() && b[j] == b')' && j > i + 2 {
            i = j + 1;
        } else {
            break;
        }
    }
    (&p[..i], &p[i..])
}

/// A Python `re.sub` replacement template.
enum Piece {
    Lit(String),
    Group(usize),
    Named(String),
}

struct Template(Vec<Piece>);

impl Template {
    fn parse(repl: &str) -> Template {
        let mut pieces = Vec::new();
        let mut lit = String::new();
        let chars: Vec<char> = repl.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '\\' && i + 1 < chars.len() {
                let n = chars[i + 1];
                if n.is_ascii_digit() {
                    let mut j = i + 1;
                    let mut num = 0usize;
                    while j < chars.len() && j < i + 3 && chars[j].is_ascii_digit() {
                        num = num * 10 + chars[j].to_digit(10).unwrap() as usize;
                        j += 1;
                    }
                    if !lit.is_empty() {
                        pieces.push(Piece::Lit(std::mem::take(&mut lit)));
                    }
                    pieces.push(Piece::Group(num));
                    i = j;
                    continue;
                }
                if n == 'g' && i + 2 < chars.len() && chars[i + 2] == '<' {
                    if let Some(close) = chars[i + 3..].iter().position(|&c| c == '>') {
                        let name: String = chars[i + 3..i + 3 + close].iter().collect();
                        if !lit.is_empty() {
                            pieces.push(Piece::Lit(std::mem::take(&mut lit)));
                        }
                        match name.parse::<usize>() {
                            Ok(num) => pieces.push(Piece::Group(num)),
                            Err(_) => pieces.push(Piece::Named(name)),
                        }
                        i += 4 + close;
                        continue;
                    }
                }
                let esc = match n {
                    'n' => Some('\n'),
                    't' => Some('\t'),
                    'r' => Some('\r'),
                    'f' => Some('\x0c'),
                    'v' => Some('\x0b'),
                    'a' => Some('\x07'),
                    '\\' => Some('\\'),
                    _ => None,
                };
                match esc {
                    Some(e) => lit.push(e),
                    None => {
                        lit.push('\\');
                        lit.push(n);
                    }
                }
                i += 2;
                continue;
            }
            lit.push(c);
            i += 1;
        }
        if !lit.is_empty() {
            pieces.push(Piece::Lit(lit));
        }
        Template(pieces)
    }

    fn expand(&self, m: &Match) -> String {
        let mut out = String::new();
        for p in &self.0 {
            match p {
                Piece::Lit(s) => out.push_str(s),
                Piece::Group(i) => out.push_str(m.str(*i)),
                Piece::Named(n) => out.push_str(m.name_str(n)),
            }
        }
        out
    }
}

/// `re.escape(text)`: CPython (3.7+) escapes only the characters special in a pattern
/// (`()[]{}?*+-|^$\\.&~#`, space and the whitespace controls), so the pattern text is the same.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        if "()[]{}?*+-|^$\\.&~# \t\n\r\x0b\x0c".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Python syntax to PCRE2: only the few places where the two read a pattern differently.
fn translate_pcre2(p: &str) -> String {
    let mut out = String::with_capacity(p.len() + 8);
    let chars: Vec<char> = p.chars().collect();
    let mut i = 0;
    let mut in_class = false;
    let mut class_start = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            let n = chars[i + 1];
            match n {
                'Z' if !in_class => out.push_str("\\z"),
                'u' if i + 5 < chars.len() + 0 && chars.len() >= i + 6 => {
                    let hex: String = chars[i + 2..i + 6].iter().collect();
                    out.push_str(&format!("\\x{{{hex}}}"));
                    i += 6;
                    continue;
                }
                'U' if chars.len() >= i + 10 => {
                    let hex: String = chars[i + 2..i + 10].iter().collect();
                    out.push_str(&format!("\\x{{{}}}", hex.trim_start_matches('0')));
                    i += 10;
                    continue;
                }
                _ => {
                    out.push(c);
                    out.push(n);
                }
            }
            i += 2;
            continue;
        }
        if in_class {
            if c == ']' && i > class_start {
                in_class = false;
            } else if c == '[' && i + 1 < chars.len() && chars[i + 1] == ':' {
                // Python reads "[:" inside a class as two literal characters
                out.push_str("\\[");
                i += 1;
                continue;
            }
            out.push(c);
            i += 1;
            continue;
        }
        if c == '[' {
            in_class = true;
            out.push(c);
            i += 1;
            if i < chars.len() && chars[i] == '^' {
                out.push('^');
                i += 1;
            }
            class_start = i; // a ']' right here is a literal
            if i < chars.len() && chars[i] == ']' {
                out.push_str("\\]");
                i += 1;
            }
            continue;
        }
        if c == '(' && i + 2 < chars.len() && chars[i + 1] == '?' && chars[i + 2] == '#' {
            // (?#comment)
            while i < chars.len() && chars[i] != ')' {
                i += 1;
            }
            i += 1;
            continue;
        }
        if c == '{' && i + 1 < chars.len() && chars[i + 1] == ',' {
            // Python's {,n} is {0,n}
            out.push_str("{0");
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Python syntax to fancy-regex syntax, with the flags as inline flags.
fn translate_fancy(p: &str, flags: u32) -> String {
    let mut out = String::with_capacity(p.len() + 16);
    let mut f = String::new();
    if flags & I != 0 {
        f.push('i');
    }
    if flags & M != 0 {
        f.push('m');
    }
    if flags & S != 0 {
        f.push('s');
    }
    if flags & X != 0 {
        f.push('x');
    }
    if !f.is_empty() {
        out.push_str(&format!("(?{f})"));
    }
    let verbose = flags & X != 0;
    let chars: Vec<char> = p.chars().collect();
    let mut i = 0;
    let mut in_class = false;
    let mut class_start = 0;
    let hex = |c: char| format!("\\x{{{:X}}}", c as u32);
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            let n = chars[i + 1];
            if n.is_ascii_alphanumeric() {
                match n {
                    'Z' => out.push_str("\\z"),
                    'u' if chars.len() >= i + 6 => {
                        let h: String = chars[i + 2..i + 6].iter().collect();
                        out.push_str(&format!("\\x{{{h}}}"));
                        i += 6;
                        continue;
                    }
                    'U' if chars.len() >= i + 10 => {
                        let h: String = chars[i + 2..i + 10].iter().collect();
                        out.push_str(&format!("\\x{{{}}}", h.trim_start_matches('0')));
                        i += 10;
                        continue;
                    }
                    _ => {
                        out.push('\\');
                        out.push(n);
                    }
                }
            } else {
                // any escaped punctuation or space: as a hex escape both parsers accept
                out.push_str(&hex(n));
            }
            i += 2;
            continue;
        }
        if in_class {
            if c == ']' && i > class_start {
                in_class = false;
                out.push(']');
            } else if c == '[' || c == '&' || c == '~' || (c == '-' && i + 1 < chars.len() && chars[i + 1] == '-') {
                out.push_str(&hex(c));
            } else if verbose && (c.is_whitespace() || c == '#') {
                out.push_str(&hex(c));
            } else {
                out.push(c);
            }
            i += 1;
            continue;
        }
        match c {
            '[' => {
                in_class = true;
                out.push('[');
                i += 1;
                if i < chars.len() && chars[i] == '^' {
                    out.push('^');
                    i += 1;
                }
                class_start = i;
                if i < chars.len() && chars[i] == ']' {
                    out.push_str("\\]");
                    i += 1;
                }
                continue;
            }
            '(' if i + 2 < chars.len() && chars[i + 1] == '?' => {
                if chars[i + 2] == '#' {
                    while i < chars.len() && chars[i] != ')' {
                        i += 1;
                    }
                    i += 1;
                    continue;
                }
                if chars[i + 2] == 'P' && i + 3 < chars.len() && chars[i + 3] == '=' {
                    // (?P=name) -> \k<name>
                    let close = chars[i..].iter().position(|&c| c == ')').unwrap() + i;
                    let name: String = chars[i + 4..close].iter().collect();
                    out.push_str(&format!("\\k<{name}>"));
                    i = close + 1;
                    continue;
                }
                out.push(c);
            }
            '{' => {
                // a quantifier only when it reads as one; otherwise a literal brace
                let rest: String = chars[i..].iter().take(24).collect();
                if let Some(q) = quantifier_len(&rest) {
                    let body: String = chars[i..i + q].iter().collect();
                    out.push_str(&body.replace("{,", "{0,"));
                    i += q;
                    continue;
                }
                out.push_str("\\{");
            }
            '}' => out.push_str("\\}"),
            _ => out.push(c),
        }
        i += 1;
    }
    out
}

/// Length of a `{n}`, `{n,}`, `{n,m}` or `{,m}` quantifier at the start of `s`.
fn quantifier_len(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut i = 1;
    let mut digits = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
        digits += 1;
    }
    if i < b.len() && b[i] == b'}' && digits > 0 {
        return Some(i + 1);
    }
    if i < b.len() && b[i] == b',' {
        i += 1;
        let mut d2 = 0;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
            d2 += 1;
        }
        if i < b.len() && b[i] == b'}' && (digits > 0 || d2 > 0) {
            return Some(i + 1);
        }
    }
    None
}
