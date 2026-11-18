//! Port of `bot/modules/keeping.py`: how long this chat can go, and whether the person is keeping the bot too long.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::understood::Understood;

macro_rules! pool {
    ($($name:ident),*) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(concat!("bot.modules.keeping.", stringify!($name))));
    )*};
}
pool!(COUNT_LINES, WHILE_LINES, KEEPING_LINES, WHILE_TAILS, KEEPING_TAILS);
static _A_COUNT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.keeping._A_COUNT"));
static THEIR_OWN_STOP: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.keeping.THEIR_OWN_STOP"));

/// `hashlib.sha1(data).digest()`.
fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for block in message.chunks(64) {
        let mut w = [0u32; 80];
        for (i, word) in block.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = h;
        for (i, word) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let next = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = next;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Python: bot/modules/keeping.py `_seed(text)`
///
/// A number that stands for this line, so the same words always get the same reply: the first
/// six bytes of the line's SHA-1, big-endian.
pub fn _seed(text: &str) -> u64 {
    sha1(text.as_bytes())[..6].iter().fold(0u64, |n, byte| (n << 8) | *byte as u64)
}

/// Python: bot/modules/keeping.py `_pick(lines, text)`
///
/// One line, chosen by the line itself, so the same words get the same reply.
pub fn _pick(lines: &[String], text: &str) -> String {
    lines[(_seed(text) % lines.len() as u64) as usize].clone()
}

/// Python: bot/modules/keeping.py `_a_bare_stop_about_a_thing(text, memory)`
///
/// True when a bare "should i stop" is about the thing the chat is on, not about the chat.
pub fn _a_bare_stop_about_a_thing(text: &str, memory: &Memory) -> bool {
    let said = fold(text);
    if THEIR_OWN_STOP.fullmatch(&said).is_none() {
        return false;
    }
    // The same reading the rest of the bot uses, on the same normalised line the test above reads:
    // "Should I stop?" with a capital and a question mark is the line the test writes, and reading
    // the raw one as well was two readings of one thing.
    if !ret_true(crate::understanding::kinds::a_stop_ask(&said)) {
        return false;
    }
    // Anything of the person's in hand is a thing the stop could be about, and the knowledge base
    // having a row for it was the wrong test: "i smoke about 20 a day", "im 40", "should i quit" is
    // the smoking, and no entry answers "i smoke about 20 a day", so the line kept "Not at all -
    // carry on, I'm following." The topic is the reading; the entry is one way to have a topic.
    !text::strip(&sget(mget(memory, "topic"), "text")).is_empty()
}

/// Python: bot/modules/keeping.py `_the_kind(understood)`: "while", "keeping" or None.
pub fn _the_kind(understood: Option<&Understood>) -> Option<String> {
    understood?.about_bot_kind.clone().filter(|kind| kind == "while" || kind == "keeping")
}

/// Python: bot/modules/keeping.py `_already_said(memory)`
///
/// What the bot said last, so the same sentence is not repeated to a person who asks twice.
pub fn _already_said(memory: &Memory) -> String {
    last_bot(memory)
}

/// Python: bot/modules/keeping.py `TAKES_AS_IT_COMES(text, memory=None)`
///
/// This line is answered as it comes: nobody else can answer a question about this chat.
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    _the_kind(memory.understood.as_ref()).is_some()
}

/// Python: bot/modules/keeping.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The reply for a question about how long this chat can go, or whether they are a nuisance.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let kind = _the_kind(memory.understood.as_ref())?;
    if kind == "keeping" && _a_bare_stop_about_a_thing(text, memory) {
        return None;
    }
    let said = _already_said(memory);
    let no_tail = vec![String::new()];
    let (lines, tails): (&[String], &[String]) = if kind == "while" && _A_COUNT.is_match(text) {
        (&COUNT_LINES, &no_tail)
    } else if kind == "while" {
        (&WHILE_LINES, &WHILE_TAILS)
    } else {
        (&KEEPING_LINES, &KEEPING_TAILS)
    };
    let order = _rotate(lines, text);
    // the first line not already in the last reply, else the first of the order
    let line = order.iter().find(|line| !said.contains(line.as_str())).unwrap_or(&order[0]).clone();
    let tail = _pick(tails, &format!("{text} tail"));
    let reply = if tail.is_empty() { line } else { format!("{line} {tail}") };
    Some(text::strip(&reply).to_string())
}

/// Python: bot/modules/keeping.py `_rotate(lines, text)`
///
/// The pool in an order that starts where this line's digest points.
pub fn _rotate(lines: &[String], text: &str) -> Vec<String> {
    let at = (_seed(text) % lines.len() as u64) as usize;
    lines[at..].iter().chain(lines[..at].iter()).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_is_pythons_sha1_seed() {
        // hashlib.sha1(b"abc").hexdigest()
        let hex: String = sha1(b"abc").iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, "a9993e364706816aba3e25717850c26c9cd0d89d");
        // int.from_bytes(hashlib.sha1("how long have you got".encode()).digest()[:6], "big")
        assert_eq!(_seed("how long have you got"), 163707420263721);
    }
}
