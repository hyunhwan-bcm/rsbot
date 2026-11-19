# rsbot: PyBot in Rust

A hand-written, exact port of [PyBot](https://github.com/huggingface/pybot) to Rust. PyBot is
a rule-based chatbot (no weights, no training: if-else branches, regexes and lookup tables) that
a swarm of agents wrote in about 640k lines of Python; its authors report a 26% win rate on
SmolTalkBench. The goal here is **the same reply and the same conversation memory as PyBot on
every turn**, as readable Rust, plus a speed and memory comparison.

**Status: complete.** Every PyBot function is ported (no stubs left), and on every recorded
turn the Rust bot gives the same reply and the same memory as PyBot.

## Results

### Correctness: identical to PyBot

Every check compares rsbot with PyBot itself (commit 24cbc0439e07), with the clock pinned to
2026-11-17 and the same random seed. The reference ("oracle") was recorded by running PyBot over
four sets of input:

| Set | What it is | Size |
|---|---|---|
| single | the SmolTalkBench prompts (1,000 gold + 1,198 held-out) | 2,198 one-turn chats |
| conversations | multi-turn chats on SmolTalkBench topics (smoltalk everyday-conversations) | 2,379 chats, 9,348 turns |
| synthetic | conversations written to reach the knowledge base's 8,081 entries | 3,000 chats, 9,000 turns |
| extra | chats from other public sets (smol-magpie-ultra, systemchats-30k, openhermes-100k) | 4,500 chats, 10,494 turns |

**Turn by turn** (`turn_check`): at every turn, each of the 116 module stages (16 observers,
97 responders, 2 after-hooks, the arbiter) is handed PyBot's exact memory and RNG state for that
point, and its reply and memory changes must equal PyBot's.

| Set | Stage checks identical | Module stages fully identical |
|---|---|---|
| single | 254,518 / 254,518 | 116 / 116 |
| conversations | 1,082,696 / 1,082,696 | 116 / 116 |
| synthetic | 1,042,045 / 1,042,045 | 116 / 116 |
| extra | 1,214,371 / 1,214,371 | 116 / 116 |

**End to end** (`turn_check --e2e`): each conversation runs from scratch in rsbot alone, carrying
its own memory from turn to turn; only the user's lines come from the recording.

| Set | Replies identical |
|---|---|
| single | 2,198 / 2,198 (100%) |
| conversations | 9,348 / 9,348 (100%) |
| synthetic | *(running)* |
| extra | *(running)* |

Lower-level checks, all with 0 differences: the regex layer (10,024 / 10,024 PyBot patterns,
`regex_check`), CPython's `random` (304 / 304 seeds), the understanding engine's reading of every
benchmark prompt (2,198 / 2,198, `oracle_check`), every knowledge-entry guard and reply
(916,598 recorded calls, `entry_check`), and per-workstream replays of several million recorded
PyBot function calls (`src/bin/*_check.rs`).

### SmolTalkBench win rate: the claim reproduces

PyBot's authors report a 26% win rate on SmolTalkBench. rsbot's replies are identical to
PyBot's on all 2,198 prompts, so judging them judges both. Judged with Claude Haiku 5.5
(`harness/judge_claude.py`: AlpacaEval 2.0's annotator prompt and per-prompt order, WildBench's
length penalty, each prompt judged in both orders to cancel position bias):

| Split | Prompts | Win rate vs reference (both orders) | AlpacaEval order only |
|---|---|---|---|
| gold | 1,000 | 27.4% | 27.1% |
| held-out | 1,198 | 24.9% | 25.1% |
| **all** | **2,198** | **26.0%** | **26.0%** |

The verdict depended on the order of the two replies for 120 of the 2,198 prompts. The run cost
$0.18 (Message Batches API, effort low). The published figure comes from a different judge
(DeepSeek-V4.1-Flash read through its token probabilities); Claude returns a verdict, not
probabilities, so the numbers are comparable in method but not identical in construction. The
"on par with SmolLM-360M-Instruct" part of the claim was not re-measured.

### Speed and memory

*Full run in progress* (`scratch/run_bench.sh`: PyBot and rsbot's release build, one process
each, run one after the other on all 2,198 prompts and on 500 multi-turn conversations).

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

## Reproducing the benchmarks

```sh
# replies: PyBot and rsbot, one process each (rows with "turns" run as one conversation)
python3 harness/run_python.py data/smoltalkbench/all.jsonl runs/py.jsonl --workers 1
RSBOT_TODAY=2026-11-17 rsbot/target/release/rsbot --jsonl data/smoltalkbench/all.jsonl runs/rs.jsonl
# win rate (dry run prints the cost; --submit sends the batch; needs ANTHROPIC_API_KEY)
uv run --with anthropic python harness/judge_claude.py --bench data/smoltalkbench/all.jsonl \
    --replies runs/rs.jsonl --out runs/judge.jsonl --model claude-haiku-5-5 --effort low --submit
# correctness against the recorded oracles
RSBOT_TODAY=2026-11-17 rsbot/target/fast/turn_check data/export/turn_oracle/conversations.jsonl.gz --e2e
```

## License

Apache-2.0, as PyBot. See LICENSE and NOTICE.
