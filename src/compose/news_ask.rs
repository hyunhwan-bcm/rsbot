//! Port of `bot/compose/news_ask.py`: a line that names the news and nothing else ("anything in
//! the news", "the headlines") is the ask with the words left out.

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.news_ask.", stringify!($name))));)*
    };
}
regexes!(A_FRAME, A_NEWS_DOING, A_VERDICT, THEIR_OWN, A_REPORT_VERB, A_YEAR);

/// Python: bot/compose/news_ask.py:57 `reads_as_an_ask_for_the_news(clean)`
///
/// True when the line asks for the news and says nothing else about it.
pub fn reads_as_an_ask_for_the_news(clean: &str) -> bool {
    let said = text::strip_chars(text::strip(clean), " .!");
    if A_FRAME.match_(said).is_none() {
        return false;
    }
    if A_NEWS_DOING.is_match(said) || A_VERDICT.is_match(said) || THEIR_OWN.is_match(said) {
        return false;
    }
    if A_REPORT_VERB.is_match(said) || A_YEAR.is_match(said) {
        return false;
    }
    true
}
