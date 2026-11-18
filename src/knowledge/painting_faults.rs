//! Port of `bot/knowledge/painting_faults.py`: paint going wrong (a coat over gloss, a door the
//! paint sealed shut, the trim, a stain coming through).

use crate::data::FnRef;
use crate::knowledge::d2_shared::a_food_is_being_kept;
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::understood::Understood;

globals!("bot.knowledge.painting_faults",
    set { _A_FINISH, _A_SURFACE, _A_ACCIDENT, _A_WAIT, _A_STRIPPER, _PAINT_WORD, _ON_THE_WAY, _THE_TRIM,
          _A_MARK, _A_LEAKING, _A_OVER });

/// The words of the line, lower-cased, with the punctuation round them taken off.
pub fn _words(understood: &Understood) -> Vec<String> {
    understood.words.iter().map(|w| crate::py::text::strip_chars(&w.to_lowercase(), ",.;:!?").to_string()).collect()
}

/// True when the line is about paint going on a surface, not a food about to be kept.
pub fn _paint_is_the_subject(understood: &Understood) -> bool {
    !a_food_is_being_kept(understood)
}

/// A finish named on its own: the coat going over what is already there.
pub fn _a_finish_with_no_surface(understood: &Understood) -> bool {
    if !_paint_is_the_subject(understood) {
        return false;
    }
    let words = _words(understood);
    if !words.iter().any(|w| _A_FINISH.contains(w)) {
        return false;
    }
    if words.iter().any(|w| _A_SURFACE.contains(w)) {
        return false;
    }
    if words.iter().any(|w| _A_ACCIDENT.contains(w) || _A_WAIT.contains(w) || _A_STRIPPER.contains(w)) {
        return false;
    }
    true
}

/// The line itself names the paint that sealed the door.
pub fn _the_paint_has_sealed_it(understood: &Understood) -> bool {
    if !_paint_is_the_subject(understood) {
        return false;
    }
    _words(understood).iter().any(|w| _PAINT_WORD.contains(w))
}

/// True when the line paints the trim itself, rather than protecting it from a wall's paint.
pub fn _the_trim_is_what_is_painted(understood: &Understood) -> bool {
    if !_paint_is_the_subject(understood) {
        return false;
    }
    let words = _words(understood);
    for (at, word) in words.iter().enumerate() {
        if !_PAINT_WORD.contains(word) {
            continue;
        }
        for said in words.iter().skip(at + 1).take(4) {
            if _ON_THE_WAY.contains(said) {
                break;
            }
            if _THE_TRIM.contains(said) {
                return true;
            }
        }
    }
    false
}

/// A mark already in the surface, with a coat going over it: the blocking question.
pub fn _a_stain_under_the_coat(understood: &Understood) -> bool {
    if !_paint_is_the_subject(understood) {
        return false;
    }
    let words = _words(understood);
    if !words.iter().any(|w| _A_MARK.contains(w)) {
        return false;
    }
    if words.iter().any(|w| _A_ACCIDENT.contains(w) || _A_LEAKING.contains(w)) {
        return false;
    }
    words.iter().any(|w| _A_OVER.contains(w) || _A_FINISH.contains(w) || w == "paint" || w == "painting")
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.painting_faults:_a_finish_with_no_surface:120", |_: &FnRef, u: &Understood| _a_finish_with_no_surface(u)),
    ("bot.knowledge.painting_faults:_the_paint_has_sealed_it:141", |_: &FnRef, u: &Understood| _the_paint_has_sealed_it(u)),
    ("bot.knowledge.painting_faults:_the_trim_is_what_is_painted:153", |_: &FnRef, u: &Understood| {
        _the_trim_is_what_is_painted(u)
    }),
    ("bot.knowledge.painting_faults:_a_stain_under_the_coat:175", |_: &FnRef, u: &Understood| _a_stain_under_the_coat(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[];
