# Porting PyBot to Rust: conventions for every workstream

rsbot is an **exact, complete** Rust port of PyBot, written by AI agents, (github.com/huggingface/pybot at
commit 24cbc0439e07, source in `../upstream/pybot/bot/`): every module, every function, every
branch, and multi-turn conversations with memory. The target is the same reply and the same
memory as PyBot on every turn of every conversation. The SmolTalkBench numbers are only a sanity
check. Write readable, idiomatic Rust that a reviewer can compare with the Python side by side.

## The skeleton and its contract

`src/<package>/<file>.rs` exists for every `bot/<package>/<file>.py` (`__init__.py` is
`mod.rs`), with one stub per Python top-level function:

```rust
/// Python: bot/memory/facts.py:123 `extract(text, memory=None)`
pub fn extract(text: &str, memory: Option<&Memory>) -> Value { crate::unported!("bot.memory.facts:extract") }
```

- In a file that was already partly ported, the missing stubs sit at the bottom inside
  `mod skeleton_stubs { ... }` (re-exported with `pub use self::skeleton_stubs::*`). When you
  port one, move it up into the file and delete the stub; delete the empty module at the end.
- Some files hold a placeholder from before the skeleton (e.g. `modules::arbiter::choose`
  returning 0): a placeholder is not a port, replace it.
- **Port every stub in your files** by reading the original Python (with its comments) and
  writing the full logic: every branch, not only the ones the test corpus reaches. Done means
  `grep -c 'unported!' <your files>` is 0.
- **Signatures are a contract.** Other workstreams call your functions with these signatures
  while you port them, so do not change the signature of a function another file calls (see
  `../slice/cross_api.json`: "module:function" -> calling modules). A function only your own file
  calls may get a better signature. If a contract signature truly cannot express the Python
  behaviour, keep it and add a second function, and say so in your report.
- Types were chosen from what each parameter held when PyBot ran: `&str`, `&Understood`,
  `&Memory` (or `&mut Memory` when the function writes memory), `&[String]`, `&'static Entry`,
  `i64`, `bool`; anything mixed is `Value` / `&Value` (`crate::data::Value`, a Python-like value).
  Python default arguments are in the doc comment; Rust callers pass them explicitly.
- Calling a function another workstream has not ported yet is fine: it panics with
  `UNPORTED <name>`, which the runtime treats like a Python exception (the module is skipped),
  and the checkers count separately.
- Runtime hooks have fixed signatures: `observe(text, &mut Memory)`, `respond(text, &mut Memory)
  -> Option<String>`, `after(text, reply, &mut Memory)`, `choose(text, &Memory, &[Candidate]) ->
  usize`. `modules::registry()` lists all 109 modules in PyBot's order.

## Data

PyBot's module-level data is **not retyped**: tables, word lists, reply pools and compiled
regexes come from the exported bundle by Python name, held in `LazyLock` statics:

```rust
static ENDINGS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.ENDINGS"));
static BACK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup.BACK_OF_THE_HOUSE"));
```

`data::get(name)` gives a `Value`: `.str()`, `.strs()`, `.str_set()`, `.items()`, `.pairs()`,
`.get(key)`, `.attr(name)`, `.as_re()`, `.as_fn()`, `.as_int()`, `.truthy()`. A regex written
inline in a function becomes a `LazyLock<Re>` built with `Re::new(pattern, flags)` (copy the
pattern text exactly). If a module-level value is computed at import time by a function, it is
already in the bundle with its computed value: read it, do not port the builder (the builder
still gets ported if anything calls it at run time).

Knowledge entries: `knowledge::entry::{Entry, entries(), by_id()}` (the canonical list PyBot's
`bot.knowledge.entries()` returns, domains set, guard folds applied). Entry guards / replies /
shape functions run through `knowledge::registry` by Python id "module:qualname:line"; closure
values via `FnRef::cell(name)`.

## Memory

`state::Memory` is PyBot's memory dict: typed `understood: Option<Understood>`, `intent_wants`,
`asked_now`, `turns: Vec<Turn>`, and every other key in `extra` as a `Value`:
`memory.get("facts")`, `memory.set("topic", v)`, `memory.setdefault(key, Value::dict())`,
`memory.remove(key)`, `memory.has(key)`. `Value` containers are copy-on-write: `set_key`,
`del_key`, `get_mut`, `push`, `items_mut`, `setdefault`, `contains_str`, `len`. Python's runtime
deep-copies memory for every module; in Rust that is `memory.clone()` (cheap).

## Python behaviour you must keep

- **Regex** (`py::re::Re`): CPython's patterns unchanged, via PCRE2; identical to CPython on all
  10,024 PyBot patterns. `search`, `search_at`, `match_`, `fullmatch`, `finditer`, `findall`,
  `findall_str`, `sub` (Python templates), `sub_fn`, `subn`, `split`, `is_match`; `Match::get(i)`,
  `str(i)` (= `group(i) or ""`), `name(..)`, `start()`, `end()`, `span(i)`.
- **Offsets** are bytes in Rust, code points in Python: convert with `py::text::byte_at`,
  `char_index`, `slice` wherever Python does arithmetic on indices.
- **str methods** (`py::text`): `split`, `squash`, `strip*`, `capitalize`, `title`,
  `upper_first`, `partition`, `find`/`rfind` (char indices), `is_digit`, `float_repr`,
  `round_half_even`, `thousands`...
- **random** (`py::random`): CPython's generator bit for bit; `Random::new(seed)`, `ord_sum`,
  module-level `seed`/`choice`/`randrange`/`getstate`/`setstate`. The global generator is
  consumed in module order across a turn exactly as in Python: keep every call.
- **Exceptions**: where Python raises and a caller catches it, return the caller's fallback;
  where nothing catches it, a panic is right (the runtime skips the module, as Python's does).
- **Truthiness, dict order** (insertion order: use `IndexMap` or `Vec<(K, V)>`), **set order**
  (Python's string-set order depends on a per-process hash seed: if output depends on it, note it).
- **Clock**: `py::clock::today()` / `now_utc()` (chrono) for `date.today()` / `datetime.now(timezone.utc)`; never read the time anywhere else. Pinned by `RSBOT_TODAY` (the checkers pin 2026-11-17, as every oracle was recorded).

## Checking

```sh
cargo build --profile fast        # iteration build (target/fast/); --release for benchmarks
# every stage of every turn, fed PyBot's own inputs (memory, RNG state) at that stage:
./target/fast/turn_check ../data/export/turn_oracle/conversations.jsonl.gz --module <name> --show 5
./target/fast/turn_check ../data/export/turn_oracle/conversations.jsonl.gz --e2e   # whole conversations
./target/fast/oracle_check understood gold                                 # first-turn reading
./target/fast/entry_check gold --module bot.knowledge.cooking              # entry guards/replies
# any Python function on any input, to compare with your port:
python3 ../harness/pycall.py bot.memory.facts extract '["i have a dog called rex"]'
```

Turn oracles in `../data/export/turn_oracle/` (recorded with the clock pinned to 2026-11-17 and
`random.seed(0)` per conversation), written in this order as they finish: `conversations`
(2,379 SmolTalkBench-topic conversations, 9,348 turns), `single` (the 2,198 benchmark prompts),
`synthetic` (3,000 conversations aimed at knowledge entries), `extra` (4,500 conversations from
other chat datasets, 10,494 turns). A file is complete when gzip can read it to the end.

The corpus does not reach every branch; a branch it does not reach must still be ported by
careful reading. Add `#[cfg(test)]` tests with expected values from `pycall.py` for logic the
corpus does not exercise.

## Working in a workstream

Work only in your git worktree and only on your files (plus `mod.rs` lines and registry tables
when you add modules). Commit often with `git -c user.name=rsbot -c user.email=rsbot@localhost
commit`; keep the crate compiling at every commit; run `git merge main` now and then to pick up
other workstreams' ports. No `unsafe`, no new dependencies without a strong reason.
