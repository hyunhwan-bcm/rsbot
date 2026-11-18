//! Port of PyBot's `bot/knowledge/kitchen_odds.py`: kitchen questions about the tools rather than
//! the food (the board, the knife, the air fryer basket), and the probe row's guard.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g3 as dup;
use crate::knowledge::dishes;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

static AIR_FRYER_BASKET: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.kitchen_odds.AIR_FRYER_BASKET")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.str().to_string()))
        .collect()
});

fn basket(key: &str) -> Option<String> {
    AIR_FRYER_BASKET.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

/// What to say about the thing the line wants to put in the air fryer basket.
pub fn _air_fryer_basket(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = format!(" {} ", understood.clean.to_lowercase());
    let has = |w: &str| text.contains(w);
    if ["parchment", "baking paper", "greaseproof"].iter().any(|w| has(w)) {
        return basket("parchment");
    }
    if ["cling film", "plastic", "freezer bag", "tub", "tupperware", "container"].iter().any(|w| has(w)) {
        return basket("plastic");
    }
    if has("foil") {
        return basket("foil");
    }
    if has("whole chicken") || (has("chicken") && has("whole")) {
        return basket("chicken");
    }
    if ["glass", "pyrex", "ceramic", "ramekin", "dish", "bowl", "plate", "mug"].iter().any(|w| has(w)) {
        return basket("glass");
    }
    if has("silicone") || has("sillicone") {
        return basket("silicone");
    }
    if ["metal", "tin", "mould", "mold", "skewer", "skewers", "loaf"].iter().any(|w| has(w)) {
        return basket("metal");
    }
    None
}

/// True when the line names a dish the dish table has an oven heat for.
pub fn a_dish_has_its_own_heat(understood: &Understood) -> bool {
    let text = crate::knowledge::d1_shared::clean_or_text(understood);
    if text.is_empty() {
        return false;
    }
    dishes::dish_in(text)
        .and_then(dishes::plan)
        .and_then(|p| p.get("temp"))
        .is_some_and(|t| !t.is_empty())
}

/// The probe row's guard: not a person's own temperature, and not a dish with its own heat.
pub fn lambda_464(understood: &Understood) -> bool {
    !dup::a_persons_temperature(understood) && !a_dish_has_its_own_heat(understood)
}

fn g_lambda_464(_f: &FnRef, u: &Understood) -> bool {
    lambda_464(u)
}

fn r_air_fryer_basket(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _air_fryer_basket(m, u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.kitchen_odds:<lambda>:464", g_lambda_464)];
pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.kitchen_odds:_air_fryer_basket:69", r_air_fryer_basket)];
