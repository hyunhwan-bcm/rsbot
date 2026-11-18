//! Private ports of the helpers `bot/knowledge/maths.py` calls from modules other workstreams
//! own (the memory store's saving reader and topics, and the units table). Each is marked so the
//! integrator can swap in the owner's port; the data comes from the bundle by Python name.

use crate::data;
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;


// ---------------------------------------------------------------------------------------------
// bot/memory/topics.py


// was DUP(bot.memory.topics::in_a_safety_thread): now the real port
pub use crate::memory::topics::in_a_safety_thread;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/units.py

// ---------------------------------------------------------------------------------------------
// bot/memory/saving.py



/// `memory.saving.the_pot_and_the_rest(text)` (the real port), as (pot, rest).
pub fn the_pot_and_the_rest(line: &str) -> (i64, i64) {
    let both = crate::memory::saving::the_pot_and_the_rest(line);
    let items = both.items();
    (items[0].as_int().unwrap_or(0), items[1].as_int().unwrap_or(0))
}

// was DUP(bot.memory.saving::the_pot_said_alone): now the real port
pub use crate::memory::saving::the_pot_said_alone;

// was DUP(bot.memory.saving::the_pot_they_are_on): now the real port
pub use crate::memory::saving::the_pot_they_are_on;

// was DUP(bot.memory.saving::the_money_already_held): now the real port
pub use crate::memory::saving::the_money_already_held;





// was DUP(bot.memory.saving::a_rate_put_right): now the real port
pub use crate::memory::saving::a_rate_put_right;
