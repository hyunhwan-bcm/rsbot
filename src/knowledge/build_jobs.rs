//! Port of `bot/knowledge/build_jobs.py`: how long a flat-pack or garden build takes, and the
//! tools it needs.

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::{clean_or_text, globals, memory_str};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

globals!("bot.knowledge.build_jobs", strs { _A_TIME_ASK, _A_TOOLS_ASK, _NOT_A_BUILD }, value { _BUILD_JOBS });

/// One build job: the names it goes by, the time, the tools and the caveat.
struct Job {
    names: Vec<String>,
    time: String,
    tools: String,
    caveat: String,
}

static JOBS: LazyLock<Vec<Job>> = LazyLock::new(|| {
    _BUILD_JOBS
        .items()
        .iter()
        .map(|row| {
            let r = row.items();
            Job { names: r[0].strs(), time: r[1].str().into(), tools: r[2].str().into(), caveat: r[3].str().into() }
        })
        .collect()
});

/// The time or the tools for the job the line names, whichever it asks for (the other one when
/// the last answer already gave it).
pub fn _a_build_job(memory: &mut Memory, reading: &Understood) -> Option<String> {
    let line = clean_or_text(reading).to_lowercase();
    let job = JOBS.iter().find(|job| job.names.iter().any(|n| reading.words.contains(n) || line.contains(n.as_str())))?;
    let wants_time = _A_TIME_ASK.iter().any(|a| line.contains(a.as_str()));
    let wants_tools = _A_TOOLS_ASK.iter().any(|a| line.contains(a.as_str()));
    let (mut chosen, spare) = if wants_tools && !wants_time { (&job.tools, &job.time) } else { (&job.time, &job.tools) };
    let last = memory_str(memory, "last_answer").to_lowercase();
    if !last.is_empty() && last.contains(&_opens_with(chosen).to_lowercase()) {
        if last.contains(&_opens_with(spare).to_lowercase()) {
            return None;
        }
        chosen = spare;
    }
    Some(format!("{} {}", chosen, job.caveat))
}

/// The first clause of a sentence, to recognise it in an earlier answer.
pub fn _opens_with(sentence: &str) -> String {
    let first = sentence.split(',').next().unwrap_or("");
    crate::py::text::strip(first.split(" - ").next().unwrap_or("")).to_string()
}

/// A line asking how long, or with what, a named job is put up.
pub fn _a_job_being_put_up(understood: &Understood) -> bool {
    let mut text = understood.words.join(" ").to_lowercase();
    if text.is_empty() {
        text = clean_or_text(understood).to_lowercase();
    }
    if _NOT_A_BUILD.iter().any(|s| text.contains(s.as_str())) {
        return false;
    }
    if !_A_TIME_ASK.iter().chain(_A_TOOLS_ASK.iter()).any(|a| text.contains(a.as_str())) {
        return false;
    }
    JOBS.iter().any(|job| job.names.iter().any(|n| text.contains(n.as_str())))
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.build_jobs:_a_job_being_put_up:162", |_: &FnRef, u: &Understood| _a_job_being_put_up(u))];

pub const REPLIES: &[(&str, ReplyFn)] =
    &[("bot.knowledge.build_jobs:_a_build_job:131", |_: &FnRef, m: &mut Memory, u: &Understood| _a_build_job(m, u))];
