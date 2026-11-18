//! Port of PyBot's `bot/knowledge/pay.py`: gross and net pay, and whether a quoted figure
//! includes tax.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g7 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.pay.{name}"))
}
fn s(name: &str) -> String {
    data::string(&format!("bot.knowledge.pay.{name}"))
}
static _GROSS_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GROSS_WORDS"));
static _QUOTED_PAY_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_QUOTED_PAY_WORDS"));
static _NET_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NET_WORDS"));
static _PAY_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PAY_WORDS"));
static _GROSS_AND_NET_COMPARED: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GROSS_AND_NET_COMPARED"));
static _GROSS_AND_NET_INCLUDES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GROSS_AND_NET_INCLUDES"));
static _GROSS_AND_NET_THREE: LazyLock<String> = LazyLock::new(|| s("_GROSS_AND_NET_THREE"));
static _GROSS_AND_NET_GROSS: LazyLock<String> = LazyLock::new(|| s("_GROSS_AND_NET_GROSS"));
static _GROSS_AND_NET_NET: LazyLock<String> = LazyLock::new(|| s("_GROSS_AND_NET_NET"));
static _GROSS_AND_NET_INCLUDES_REPLY: LazyLock<String> = LazyLock::new(|| s("_GROSS_AND_NET_INCLUDES_REPLY"));

/// `" %s " % " ".join(line.lower().split())`.
fn spaced(understood: &Understood) -> String {
    format!(" {} ", text::squash(&sh::clean_or_text(understood).to_lowercase()))
}

/// Whether the line asks about gross and net pay, or whether a quoted salary includes tax (a
/// line with a figure in it is a sum for the pay calculator, unless it asks about inclusion).
pub fn _about_pay(understood: &Understood) -> bool {
    let text = spaced(understood);
    let says_gross = sh::any_word_in(&text, &_GROSS_WORDS);
    let says_net = sh::any_word_in(&text, &_NET_WORDS);
    if text.chars().any(dup::is_digit_char)
        && !sh::any_word_in(
            &text,
            &["include", "includes", "included", "include tax", "includes tax", "tax included", "before or after", "after or before"],
        )
    {
        return false;
    }
    if says_gross && says_net {
        return true;
    }
    if says_gross || says_net {
        return sh::any_word_in(&text, &_PAY_WORDS) || sh::any_word_in(&text, &["tax", "taxed"]);
    }
    if !sh::any_word_in(&text, &_QUOTED_PAY_WORDS) {
        return false;
    }
    if !sh::any_word_in(&text, &["tax", "taxed"]) {
        return false;
    }
    sh::any_word_in(&text, &["include", "includes", "included", "taxed", "before", "after", "quoted", "advertised"])
}

/// The gross/net card the line asks for: both explained, what "includes" means, net, or gross.
pub fn _gross_or_net(_memory: &Memory, understood: &Understood) -> String {
    let text = spaced(understood);
    let says_gross = sh::any_word_in(
        &text,
        &["gross", "gross pay", "gross salary", "gross income", "before tax", "pre tax", "pre-tax", "pretax"],
    );
    let says_net = sh::any_word_in(
        &text,
        &["net", "net pay", "net income", "after tax", "take home", "take-home", "takehome", "take home pay", "net of tax"],
    );
    if says_gross && says_net {
        return _GROSS_AND_NET_THREE.clone();
    }
    if [" before or after ", " after or before ", " gross or net ", " net or gross "].iter().any(|p| text.contains(p)) {
        return _GROSS_AND_NET_THREE.clone();
    }
    if sh::any_word_in(&text, &_GROSS_AND_NET_COMPARED) && says_gross {
        return _GROSS_AND_NET_THREE.clone();
    }
    if sh::any_word_in(&text, &_GROSS_AND_NET_INCLUDES) {
        return _GROSS_AND_NET_INCLUDES_REPLY.clone();
    }
    if says_net {
        return _GROSS_AND_NET_NET.clone();
    }
    if says_gross {
        return _GROSS_AND_NET_GROSS.clone();
    }
    _GROSS_AND_NET_THREE.clone()
}

fn g_about_pay(_f: &FnRef, u: &Understood) -> bool {
    _about_pay(u)
}
fn r_gross_or_net(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_gross_or_net(m, u))
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.pay:_about_pay:86", g_about_pay)];
pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.pay:_gross_or_net:124", r_gross_or_net)];
