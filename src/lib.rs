//! rsbot: a hand-written Rust port of PyBot's hot path.
//!
//! PyBot (github.com/huggingface/pybot) is a rule-based chatbot a swarm of agents wrote in
//! 640k lines of Python. This crate ports the code that actually produces its replies on
//! SmolTalkBench: two observers and nine modules, sliced to the branches the benchmark reaches
//! (see README and `harness/slicer.py`). PyBot's data (word lists, reply pools, regexes, the
//! knowledge entries) is not retyped: it is exported from Python into a bundle (`data`).
//!
//! Layout mirrors the Python package: `bot/<package>/<file>.py` is `crate::<package>::<file>`.
pub mod compose;
pub mod data;
pub mod knowledge;
pub mod memory;
pub mod modules;
pub mod oracle;
pub mod py;
pub mod runtime;
pub mod state;
pub mod understanding;
pub mod understood;
