# rsbot: PyBot in Rust

A hand-written, exact port of [PyBot](https://github.com/huggingface/pybot) to Rust. PyBot is
a rule-based chatbot (no weights, no training: if-else branches, regexes and lookup tables) that
a swarm of agents wrote in about 640k lines of Python; its authors report a 26% win rate on
SmolTalkBench. The goal here is **the same reply and the same conversation memory as PyBot on
every turn**, as readable Rust, plus a speed and memory comparison.

**Status: work in progress.** The foundation and the verification tooling are done; the
port of the ~152k lines of PyBot logic is being done in parallel workstreams (branches `ws-*`).

## What is verified so far

| Piece | Check against CPython / PyBot | Result |
|---|---|---|
| Regex layer (`src/py/re.rs`, PCRE2) | every one of PyBot's 10,024 patterns, all match spans over 5,231 lines | 10,024 / 10,024 identical, ~6.9x faster than CPython's `re` |
| `random` (`src/py/random.rs`) | CPython's Mersenne Twister: `random`, `choice`, `getrandbits`, `sample`, `shuffle` | 304 / 304 seeds identical |
| Understanding: `normalize.py` (branch `ws-U`) | every text PyBot normalises on the benchmark + stress lines | identical on 5,639 + 7,055 lines |

## How the port is built and checked

- **Data is exported, logic is hand-written.** PyBot's module-level data (~233k lines: word
  lists, reply pools, 10k regexes, 8,081 knowledge entries) is exported from Python into a
  bundle (`harness/export_data.py`) and read by name; the ~152k lines of logic in 10,800
  functions are ported by hand.
- **A contract skeleton** (`harness/gen_skeleton.py`): one Rust stub per Python function, with
  a fixed signature inferred from the types PyBot's functions actually receive and return
  (`harness/trace_full.py --types`) and from a call-graph analysis of which functions write to
  memory, so workstreams can call each other's functions before they are ported.
- **Oracles recorded from PyBot itself**:
  - `harness/turn_oracle.py`: every stage of every turn of whole conversations (memory going
    in, each observer's change, each of the 97 modules' reply and memory side effects, the
    arbiter's pick, the after-hooks, the global RNG state at each stage). `turn_check` feeds
    each Rust stage PyBot's exact input and compares.
  - `harness/entry_calls.py`: every knowledge-entry guard / reply call; `entry_check`.
  - `harness/oracle_dump.py`: per-stage results on SmolTalkBench; `oracle_check`.
  - `harness/pycall.py`: run any PyBot function on any input.
- Corpus: SmolTalkBench (1,000 gold + 1,198 held-out prompts), 2,379 multi-turn
  `everyday-conversations` from smoltalk, 4,500 conversations from other smoltalk subsets, and
  3,000 synthetic conversations keyed on knowledge entries. On real conversations PyBot only
  runs 42% of its statements, so branches the corpus does not reach are ported by reading.

See [PORTING.md](PORTING.md) for the conventions every workstream follows.

## Layout

```
src/py/          Python behaviour: re (PCRE2), random (CPython MT19937), str methods
src/data.rs      the exported data bundle, as Python-like values
src/understood.rs, src/state.rs   the typed reading and the conversation memory
src/runtime.rs   one turn: observers, modules, arbiter, after-hooks (bot/runtime.py)
src/<package>/   bot/<package>/<file>.py -> src/<package>/<file>.rs
src/bin/         rsbot (CLI, same options as pybot.py), oracle_check, turn_check, entry_check, ...
harness/         Python side: exports, oracles, tracers, skeleton generator, benchmarks
```

## Setting up the workspace

The crate expects a workspace directory with this repository as `rsbot/` and PyBot beside it:

```sh
mkdir work && cd work
git clone <this repo> rsbot && ln -s rsbot/harness harness
git clone https://github.com/huggingface/pybot upstream/pybot
git -C upstream/pybot checkout 24cbc0439e07fd015c0747ab3be6e782a42b5c5b
python3 harness/export_data.py data/export/bundle.json      # PyBot's data for the port
cd rsbot && cargo build --release
```

The oracles are regenerated with the scripts in `harness/` (see each script's docstring).

## Benchmark (sanity check)

SmolTalkBench: `harness/run_python.py` runs PyBot, `rsbot --jsonl` runs the port, and
`harness/judge_claude.py` judges pairwise against the reference replies (AlpacaEval 2.0 prompt
and order, WildBench length penalty; Claude returns no logprobs, so it scores verdicts in both
orders rather than the published logprob probability). `harness/bench.sh` measures speed and
peak memory.

## License

Apache-2.0, as PyBot. See LICENSE and NOTICE.
