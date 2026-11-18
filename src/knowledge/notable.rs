//! Port of `bot/knowledge/notable.py`: the people and numbers in the news (who holds an office,
//! who plays a part, how big a city is). The rows are data (`ENTRIES`, kept in a second file in
//! Python); `_row` is the helper that keeps one.

use crate::knowledge::entry::Entry;

/// `_row(*args, **kwargs)`: keep one entry - `entry()` builds it, the domain needs the list.
pub fn _row(entries: &mut Vec<Entry>, row: Entry) {
    entries.push(row);
}
