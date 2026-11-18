//! Port of `bot/compose/a_count_of_packets.py`: an amount of medicine said as a count of
//! packets somebody has taken ("i swallowed two packets of pills").

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.a_count_of_packets.", stringify!($name))));)*
    };
}
regexes!(TAKEN, NO_VERB, AN_ANIMAL_TOOK, PACKET_WORDS_BEFORE, PACKET_WORDS_ON, STOCK_PLACE);

/// Python: bot/compose/a_count_of_packets.py:90 `is_a_dose(text)`
///
/// True when the line says an amount of medicine as a count of packets somebody has taken.
pub fn is_a_dose(text: &str) -> bool {
    let line = text::squash(&text::lower(text));
    if line.is_empty() {
        return false;
    }
    if AN_ANIMAL_TOOK.is_match(&line) {
        return false;
    }
    if PACKET_WORDS_BEFORE.is_match(&line) || PACKET_WORDS_ON.is_match(&line) || STOCK_PLACE.is_match(&line) {
        return false;
    }
    TAKEN.is_match(&line) || NO_VERB.is_match(&line)
}
