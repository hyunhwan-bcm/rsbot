//! Port of `bot/compose/judgement.py`: is this line asking the bot to rule on the plan already
//! on the table? The reading `fallback._a_verdict_on_their_plan` makes, without the reply.

use crate::compose::fallback::{_matchable, _the_plan_on_the_table};
use crate::compose::fallback_chain::caught;
use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

static JUDGE_BARE_FACES: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::set("bot.compose.fallback_data.JUDGE_BARE_FACES"));
static JUDGE_ASK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.fallback.JUDGE_ASK"));
static JUDGE_ASK_BARE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.fallback.JUDGE_ASK_BARE"));

/// Python: bot/compose/judgement.py:22 `asks_for_a_judgement(text, memory=None)` (never called on the trace corpus)
///
/// True when the line asks the bot to rule on the plan on the table, else False. A bare adjective
/// only counts where a plan is already named. The contract's `Option` is always `Some`: Python
/// returns a bool on every path (a reading that fails claims nothing).
pub fn asks_for_a_judgement(text: &str, memory: &Memory) -> Option<bool> {
    let clean = text::squash(&text::lower(text));
    let read = caught(|| {
        let matchable = _matchable(&clean);
        let said = text::rstrip_chars(&matchable, " ?.!,");
        if JUDGE_ASK.match_(said).is_some() {
            return true;
        }
        let Some(bare) = JUDGE_ASK_BARE.match_(said) else { return false };
        // `bare.group("bare_noun") or bare.group("face") in JUDGE_BARE_FACES`
        let noun = bare.name("bare_noun").is_some_and(|g| !g.is_empty());
        let face = bare.name("face").is_some_and(|g| JUDGE_BARE_FACES.contains(g));
        if !(noun || face) {
            return false;
        }
        _the_plan_on_the_table(memory).truthy()
    });
    Some(read.unwrap_or(false))
}
