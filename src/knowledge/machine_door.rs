//! Port of `bot/knowledge/machine_door.py`: the door left shut, said after a smelly machine, is
//! the cause of the smell.

use crate::data::{self, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

static A_DOOR_FAULT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"cant|cannot|can not|won'?t|will not|does not|doesn'?t|hard to|stuck|swings?|swinging|on its own|itself|hanging|off|lock|keyhole|slam|bang|squeak|squeal|rub|paint|draught|draft|dragging|dropped|measure|door is not|door isnt|door isn'?t|door is broken|door is stiff|door is loose|door is stuck|door is jamming|door was broken|door was stiff|door was loose",
        pyre::I,
    )
});

/// `str(d.get("text") or "")` of a dict; anything else that is not None raises AttributeError.
fn text_of(v: &Value) -> String {
    match v {
        Value::Dict(_) => match v.get("text") {
            Some(t) if t.truthy() => t.as_str().map(str::to_string).unwrap_or_else(|| panic!("str() of a non-string text")),
            _ => String::new(),
        },
        other => panic!("AttributeError: {other:?} has no attribute 'get'"),
    }
}

/// The door left shut, said after a smelly machine, is the cause of the smell; None for any other
/// door line, or when the chat was not about a smelly machine.
pub fn _a_damp_machine_and_its_door(memory: &Memory, understood: &Understood) -> Option<String> {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let said_now = format!(" {} ", text::lower(line));
    if A_DOOR_FAULT.is_match(&said_now) {
        return None;
    }
    let topic = memory.get("topic").filter(|t| t.truthy()).map(text_of).unwrap_or_default();
    let mut said = vec![topic];
    if let Some(topics) = memory.get("topics").filter(|t| t.truthy()) {
        let items = topics.items();
        said.extend(items[items.len().saturating_sub(4)..].iter().map(text_of));
    }
    let chat = text::lower(&said.join(" "));
    let words = |name: &str| data::get(&format!("bot.knowledge.machine_door.{name}")).strs();
    if !words("_MACHINE_IN_THE_CHAT").iter().any(|w| chat.contains(w.as_str())) {
        return None;
    }
    if !words("_A_SMELLY_MACHINE").iter().any(|w| chat.contains(w.as_str())) {
        return None;
    }
    Some("Then that is very likely the cause: the inside of the machine stays wet between washes, and the smell grows in water that cannot dry. Leave the door and the detergent drawer open between washes, wipe the rubber seal round the door, and run one hot 60C or 90C wash with a machine cleaner or a cup of bicarbonate of soda to clear what has already built up.".into())
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.machine_door:_a_damp_machine_and_its_door:24", |_, m, u| _a_damp_machine_and_its_door(m, u))];
