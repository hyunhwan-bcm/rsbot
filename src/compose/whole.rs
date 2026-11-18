//! Port of `bot/compose/whole.py`: a reply cut to length without ending on a condition whose
//! doing was cut away ("If it keeps happening." with the step in the clause the cut took).

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use std::sync::LazyLock;

static _LAST_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.whole._LAST_CLAUSE"));
static _CONTINUES: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.whole._CONTINUES"));
static _MOST_AFTER: LazyLock<i64> = LazyLock::new(|| data::get("bot.compose.whole._MOST_AFTER").as_int().unwrap());
/// `re.search(r"[,;]\s", ...)`, written inline in `a_whole_cut`.
static A_BREAK: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,;]\s", 0));

const ENDS: &str = " ,;:-";

/// `chars.rfind(sub, 0, end)` with Python's character indices (the match lies inside `[0, end)`).
fn rfind_before(chars: &[char], sub: &[char], end: i64) -> i64 {
    let n = chars.len() as i64;
    let end = if end < 0 { (n + end).max(0) } else { end.min(n) } as usize;
    if sub.len() > end {
        return -1;
    }
    (0..=end - sub.len()).rev().find(|&i| chars[i..i + sub.len()] == *sub).map(|i| i as i64).unwrap_or(-1)
}

/// Python: bot/compose/whole.py:33 `a_whole_cut(text, most)`
///
/// `text` cut to about `most` characters, never ending on a hanging condition. The last comma or
/// semicolon inside `most` is the cut; when that lands between a condition and its doing, the cut
/// moves to the end of the doing, or back to before the condition when the doing is too long.
pub fn a_whole_cut(text: &str, most: i64) -> String {
    let whole = text::strip(text);
    let chars: Vec<char> = whole.chars().collect();
    let n = chars.len() as i64;
    if n <= most {
        return whole.to_string();
    }
    let mut cut = rfind_before(&chars, &[',', ' '], most).max(rfind_before(&chars, &[';', ' '], most));
    if cut < 80 {
        cut = rfind_before(&chars, &[' '], most);
    }
    let cut = cut.max(80);
    let head = text::slice(whole, 0, Some(cut as isize));
    let tail = text::slice(whole, cut as isize, None);
    let kept = text::strip_chars(&head, ENDS);
    let dropped = text::strip_chars(&tail, ENDS);
    let Some(hanging) = _LAST_CLAUSE.search(kept) else { return kept.to_string() };
    if dropped.is_empty() || _CONTINUES.match_(dropped).is_some() {
        return kept.to_string();
    }
    let rest = text::slice(whole, (cut + 2) as isize, None);
    let end = match A_BREAK.search(&rest) {
        Some(after) => cut + 2 + text::char_index(&rest, after.start()) as i64,
        None => n,
    };
    let doing = text::slice(whole, cut as isize, Some(end as isize));
    if end - cut <= *_MOST_AFTER && !doing.contains('\'') && !doing.contains('"') {
        return text::strip_chars(&text::slice(whole, 0, Some(end as isize)), ENDS).to_string();
    }
    let start = hanging.span(1).map(|(s, _)| s).unwrap_or(0);
    text::strip_chars(&kept[..start], ENDS).to_string()
}
