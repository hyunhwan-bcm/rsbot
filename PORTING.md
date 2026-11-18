# Porting PyBot to Rust: conventions for every workstream

rsbot is a hand-written Rust port of PyBot's hot path (github.com/huggingface/pybot at commit
24cbc0439e07, in `../upstream/pybot`). The goal is the **same reply as PyBot** for every
SmolTalkBench prompt, written as readable, idiomatic Rust (not transpiled code).

## What to port: the sliced spec

`../slice/spec/bot/...` is PyBot with everything removed that did not run on the 1,000 gold
prompts: branches never taken, functions never called, comments and docstrings. It reproduces
PyBot's gold replies exactly, so **porting the spec faithfully is enough**. Each workstream has its
own subset (`../slice/ws/<WS>/` with an `index.json`, or a module list). Read the original file in
`../upstream/pybot/bot/` when a comment would help you understand intent; port the spec's logic.

Port **only what the spec contains**. Do not tune behaviour against held-out prompts: the
held-out set measures how well a port built from gold generalises.

## Layout and naming

- `bot/<package>/<file>.py` is `src/<package>/<file>.rs` (`__init__.py` is `mod.rs`). Add
  `pub mod <file>;` to the package's `mod.rs`.
- Keep Python function names (a leading underscore stays: `fn _forms(...)`), so a reader can go
  from Rust to Python by name. Module-level constants keep their names too.
- PyBot's data is **not retyped**. Module-level tables, word lists, reply pools and compiled
  regexes come from the bundle by Python name, held in `LazyLock` statics:

  ```rust
  static ENDINGS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.ENDINGS"));
  static BACK_OF_THE_HOUSE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.lookup.BACK_OF_THE_HOUSE"));
  ```

  `data::get(name)` returns a `data::Value` (None/Bool/Int/Float/Str/List/Tuple/Set/Dict/Re/Fn/Obj)
  with helpers: `.str()`, `.strs()`, `.str_set()`, `.items()`, `.pairs()`, `.get(key)`,
  `.attr(name)` (object attributes, e.g. an `Entry`), `.as_re()`, `.as_fn()`. Sets are exported
  in sorted order. `data::names_with_prefix("bot.knowledge.cooking.")` lists a module's globals.
- A regex written inline in a function (`re.search(r"...", text)`) becomes a `LazyLock<Re>`
  static built with `Re::new(pattern, flags)`. Copy the pattern text exactly.

## Python behaviour you must keep

- **Regex**: `crate::py::re::Re` runs CPython's patterns unchanged (PCRE2, verified identical to
  CPython on all 10,024 PyBot patterns). It has `search`, `search_at`, `match_`, `fullmatch`,
  `finditer`, `findall`, `findall_str`, `sub` (Python templates `\1`, `\g<name>`), `sub_fn`,
  `subn`, `split`, `is_match`; `Match` has `get(i)`, `str(i)` (= `group(i) or ""`), `name(..)`,
  `start()`, `end()`, `span(i)`. Flags are Python's bits (`re::I`, `re::X`, ...).
- **Offsets**: `Match` positions are byte offsets. Python's are code points. They agree on ASCII;
  where Python does arithmetic on indices (`text[m.end():m.end() + 24]`), convert with
  `py::text::byte_at` / `char_index` / `slice`.
- **str methods**: `crate::py::text` (`split` = `str.split()`, `squash` = `" ".join(s.split())`,
  `strip`, `capitalize`, `title`, `upper_first`, `partition`, `find`/`rfind` (char indices),
  `is_digit`, `float_repr` (Python's float printing), `round_half_even` (Python's `round`)...).
- **random**: `crate::py::random` is CPython's Mersenne Twister bit for bit. `random.Random(seed)`
  is `Random::new(seed)`; `ord_sum(text)` is `sum(ord(c) for c in text)`. Module-level
  `random.choice` uses `py::random::choice` (seeded per conversation like Python).
- **Exceptions**: where Python would raise (IndexError, KeyError, a None attribute...) and a
  caller catches it, return the caller's fallback. Where nothing in the module catches it, a Rust
  panic is acceptable: the runtime catches a panicking module and skips it, exactly as Python's
  runtime skips a module that raised. Guards/replies run through the registry are also caught.
- **Truthiness**: Python `x or y`, `if x:` on strings/lists/None. Be careful with `0`, `""`, `[]`.
- **Dict order**: Python dicts keep insertion order; use `IndexMap` or `Vec<(K, V)>` where order
  is observable. **Set order**: Python's set iteration order for strings depends on a per-process
  hash seed. If a mismatch comes from iterating a set, note it and move on.
- **Dates**: the clock is pinned to 2026-11-17 in every oracle run.

## Shared types

- `understood::Understood`: the reading (`memory["understood"]`), typed. `Option<String>` for
  values that can be None; `Vec<String>` for lists; `serde(default)` fills fields a partial
  reading lacks. `.wants` is written by the intent observer.
- `state::Memory`: `understood`, `intent_wants`, `asked_now`, `turns` (empty in single-turn
  replies), `extra` (anything else, by Python key, as `data::Value`).
- `knowledge::registry`: entry guards (`guard(reading) -> bool`) and function replies
  (`reply(memory, reading) -> str`) by Python id `"module:qualname:line"`. A ported module
  exposes `pub const GUARDS: &[(&str, GuardFn)]` and `pub const REPLIES: &[(&str, ReplyFn)]`
  and adds them to `guard_tables()` / `reply_tables()`. Closure values: `f.cell("name")`;
  default arguments: `f.defaults`. One Rust function can serve every entry made by one factory.
- `runtime`: `turn`, `answer`, `guarded` (catch a panic). `modules::hooks` wires each module.

## Checking your work

All checks compare against PyBot's own results, recorded on the benchmark prompts:

```sh
cargo build --release
./target/release/oracle_check understood gold --show 5     # the reading, field by field
./target/release/oracle_check module answers gold --show 5 # one module's candidate reply
./target/release/oracle_check all gold                     # every stage
./target/release/entry_check gold --module bot.knowledge.cooking --show 3   # guards/replies
RSBOT_ORACLE_UNDERSTAND=1 ...  # understanding::understand(text) answers from PyBot's recorded results
```

`oracle_check` feeds each stage PyBot's own inputs, so a stage can be checked before the stages
in front of it are ported. Commit to your branch often (`git commit -am ...`); the integrator
merges branches into `main`.

## Style

Idiomatic Rust: plain functions, `&str` in, `String`/`Option` out, iterators, early returns.
Keep the Python structure where it carries the logic, so a reviewer can compare the two side by
side. No `unsafe`. Add a one-line doc comment saying what a function decides, in the bot's terms.
No new dependencies without a strong reason.
