//! Port of `bot/modules/no_advice.py`: a line that says it does not want advice is heard, not
//! answered with the thing it declined. The reading and the wording are `compose::no_advice`'s;
//! this module gives the claim a band of its own (1050, above the arbiter's CHATTER line) and
//! declares it with `TAKES_AS_IT_COMES`.

use crate::runtime::guarded;
use crate::state::Memory;

/// Python: bot/modules/no_advice.py:37 `respond(text, memory)`: the listening line, or None on
/// every other line.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // A reading that raises answers the line as it did before.
    let pool = guarded(|| pool_name(text))?;
    let pool = pool?;
    if pool.is_empty() {
        return None;
    }
    if !TAKES_AS_IT_COMES(text, memory) {
        return None;
    }
    guarded(|| crate::compose::no_advice::reply_for(&pool, text, memory))
}

/// `no_advice.pool_name(text)`: the name of the pool the line is answered from, or None.
/// `compose::no_advice::pool_name` now returns `Option<String>` (COM5 gave it Python's type);
/// the `Reading` adapter carries it unchanged.
fn pool_name(text: &str) -> Option<String> {
    use crate::modules::arbiter::Reading;
    crate::compose::no_advice::pool_name(text).text()
}

/// Python: bot/modules/no_advice.py:53 `TAKES_AS_IT_COMES(text, memory=None)`: the lines this
/// module read whole and no entry has any business answering.
#[allow(non_snake_case)]
pub fn TAKES_AS_IT_COMES(text: &str, _memory: &Memory) -> bool {
    // A hook that raises claims nothing.
    guarded(|| {
        if !crate::compose::no_advice::says_no_advice(text) {
            return false;
        }
        // A death in the line is a death however the rest of it is put: the loss module's turn.
        !crate::compose::no_advice::says_a_death(text)
    })
    .unwrap_or(false)
}
