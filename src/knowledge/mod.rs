//! Port of PyBot's `bot/knowledge` package. Each Python file `bot/knowledge/<name>.py` is `<name>.rs` here.

pub mod registry;

// D2 workstream
pub mod d2_shared;
pub mod differences;
pub mod icing;
pub mod cleaning;
pub mod consumer;
pub mod d2dup_g8;
pub mod lifeadmin;
pub mod plans;
