//! Port of `bot/knowledge/pasta.py`: how long a pasta shape takes, the families of shape, and
//! which shape a dish wants.

use crate::data::{self, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

const P: &str = "bot.knowledge.pasta.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}

/// `PASTA_MINUTES`: (the words for a shape, its minutes).
static PASTA_MINUTES: LazyLock<Vec<(Vec<String>, String)>> =
    LazyLock::new(|| g("PASTA_MINUTES").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect());
/// `PASTA_KINDS`: (family, its shapes, what it is for).
static PASTA_KINDS: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| g("PASTA_KINDS").items().iter().map(Value::strs).collect());
/// `PASTA_FOR_DISH`: (the words for a dish, the shapes, why).
static PASTA_FOR_DISH: LazyLock<Vec<(Vec<String>, String, String)>> = LazyLock::new(|| {
    g("PASTA_FOR_DISH").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string(), r.items()[2].str().to_string())).collect()
});

/// The families of shape, what each is for, and the times those shapes already have.
pub fn _the_kinds(_memory: Option<&Memory>, _understood: Option<&Understood>) -> String {
    let members: Vec<String> = PASTA_KINDS.iter().map(|k| format!("{} ({}) for {}", k[0], k[1], k[2])).collect();
    format!(
        "Pasta comes down to six families, and the family decides the job: {} and {}. The time goes by the shape as well: 8 to 9 minutes for thin strands, 10 to 11 for spaghetti and penne, 11 to 13 for rigatoni and tagliatelle, and 3 to 4 for the filled ones. Fresh pasta is 2 to 3 minutes, not the packet time.",
        members[..members.len() - 1].join("; "),
        members[members.len() - 1]
    )
}

/// The minutes for the shape the line names, or the range that covers dried pasta.
pub fn _minutes(words: &[String]) -> String {
    let joined = words.join(" ");
    if joined.contains("noodle") || joined.contains("noodles") {
        return g("NOODLES").str().to_string();
    }
    for (keys, minutes) in PASTA_MINUTES.iter() {
        if keys.iter().any(|k| joined.contains(k.as_str())) {
            return format!("About {minutes} for that one.");
        }
    }
    "Dried pasta is 8 to 12 minutes, depending on the shape: 8 to 9 for thin spaghetti.".into()
}

/// The time by shape, then the two things that decide whether it is right.
pub fn _pasta_time(_memory: &Memory, understood: &Understood) -> String {
    let words: Vec<String> = understood.words.iter().map(|w| text::lower(w)).collect();
    let first = if words.iter().any(|w| w == "fresh") {
        "Fresh pasta is 2 to 3 minutes, not the packet time: it is done the moment it floats and loses its chalky middle, so taste a strand at 2 minutes.".to_string()
    } else {
        _minutes(&words)
    };
    format!(
        "{first} Use a big pan with about 10g of salt per litre, keep a mug of the water for the sauce, and taste a piece a minute before the packet says: shapes differ, and so do pans."
    )
}

/// The shapes for the dish the line names, then why those and not the ones next to them.
pub fn _for_the_dish(_memory: Option<&Memory>, understood: &Understood) -> String {
    let mut said = understood.words.iter().map(|w| text::lower(w)).collect::<Vec<_>>().join(" ");
    if said.is_empty() {
        said = text::lower(&understood.clean);
    }
    for (keys, shapes, why) in PASTA_FOR_DISH.iter() {
        if keys.iter().any(|k| said.contains(k.as_str())) {
            return format!("{shapes}, because {why}.");
        }
    }
    "Match the shape to the sauce rather than to the packet: a thick meat sauce wants a wide strand or a tube, a thin one wants a long strand, and a bake wants penne or rigatoni.".into()
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.pasta:_the_kinds:86", |_, m, u| Some(_the_kinds(Some(m), Some(u)))),
    ("bot.knowledge.pasta:_pasta_time:109", |_, m, u| Some(_pasta_time(m, u))),
    ("bot.knowledge.pasta:_for_the_dish:173", |_, m, u| Some(_for_the_dish(Some(m), u))),
];
