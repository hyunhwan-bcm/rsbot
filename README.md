# rsbot: PyBot in Rust

An exact port of [PyBot](https://github.com/huggingface/pybot) to Rust, written by AI agents
(Claude Code) and verified against PyBot itself on every recorded turn. PyBot is
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
| synthetic | 9,000 / 9,000 (100%) |
| extra | not run (stopped to free the machine for the benchmark; turn by turn it is 116 / 116 identical) |

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

**Preliminary, from earlier runs.** The controlled benchmark described below is still running;
its numbers and plot will replace these.

![Time per reply, Python vs Rust, preliminary](docs/speed_jitter_preview.png)

| | Python (PyBot) | Rust (rsbot) |
|---|---|---|
| median time per reply, 2,198 prompts | 1,129 ms | 144 ms (7.8x faster) |
| 90th percentile | 4,364 ms | 382 ms (11.4x faster) |
| startup and one reply (one prompt, release build) | 3.06 s | 0.87 s |
| peak memory (one prompt) | 180 MB | 904 MB (5x more) |

Why these numbers are only preliminary:
- **Python:** the replies came from PyBot run with 10 parallel workers on a busy machine, which
  slows each reply.
- **Rust:** the replies came from the development build, run alone, while other checks were
  running.
- **Same work on both sides:** the replies themselves are identical on all 2,198 prompts.
- **Memory:** Rust uses more because it loads all of PyBot's exported data (47 MB of JSON) into
  memory at start, where Python imports what it needs. This can be fixed without changing any
  reply.

How they are measured:
- **Same work on both sides.** PyBot and rsbot's release build (`cargo build --release`, thin
  LTO) get the same inputs, the same pinned date (2026-11-17) and the same random seed. Their
  replies are compared before any timing is reported, and they must be identical.
- **Two input sets:**
  - all 2,198 SmolTalkBench prompts, each a fresh one-turn chat;
  - the first 500 multi-turn conversations, 1,960 turns, with memory carried from turn to
    turn.
- **One process each, no parallel workers.** The two runs go one after the other on an idle
  machine (12-core Apple Silicon Mac, 32 GB), after every other job has finished.
- **What is recorded:**
  - wall-clock time, CPU time and peak resident memory (`/usr/bin/time -l`, which includes
    startup and data loading);
  - each reply's own time, from the runner's clock around one turn.
- **Script:** `harness/run_bench.sh` runs the benchmark, and `harness/plot_speed.py` draws the
  jitter plot of time per reply.

## Web chat

`rsbot_web` serves a chat page for talking to the bot in a browser:

```sh
cargo build --release --bin rsbot_web
./target/release/rsbot_web --port 8790      # then open http://127.0.0.1:8790/
```

- **Conversations:** each browser tab is its own conversation. The server keeps the tab's
  memory and its own copy of PyBot's random generator, so a conversation answers exactly as it
  would running alone in `pybot.py`.
- **Under each reply:** the module that answered and the time it took.
- **Starting over:** "New chat" starts a fresh conversation.
- **Endpoints:**
  - `POST /api/chat` takes `{"session", "text"}` and returns
    `{"reply", "module", "ms", "turn"}`;
  - `POST /api/reset` forgets a conversation;
  - `GET /api/health` reports the server's state.
- **Conversation logs:** every turn is saved, as one JSON line, to `logs/rsbot-YYYY-MM-DD.jsonl`
  (one file per UTC day), recording the time, the tab's session id, the turn number, the user's
  line, the reply, the module that answered and the time it took. A "New chat" is logged as a
  `reset` event. Use `--log-dir DIR` to write them elsewhere, or `--no-log` to turn logging off.
  The logs hold whatever people type, so treat them as private; `logs/` is in `.gitignore`.
- **Built from:** the standard library only (no web framework), one request at a time. It is
  meant for local use; to reach it from other machines, bind it with `--host 0.0.0.0` behind
  your own proxy.

`harness/web_test.mjs` drives the page in real Chrome (via `puppeteer-core`). It checks:
- replies, and memory across turns;
- that "New chat" forgets;
- that two tabs keep separate memories;
- Shift+Enter and empty lines;
- dark mode;
- phone width without sideways scrolling;
- no page errors.

All 14 checks pass. A five-turn conversation through the page gave the same replies as PyBot
itself.

## How it was done

The port was written by Claude agents (Claude Code), coordinated from one session, between
2026-11-17 and 2026-11-18. That is about two days of wall-clock time, much of it spent waiting
for usage-limit resets.

1. **Measure before porting.**
   - Run PyBot over the benchmark and real conversations.
   - Record line coverage and the type of every argument and return value
     (`harness/trace_full.py`).
   - Export all module-level data (`harness/export_data.py`).
2. **Build the shared foundation first.** The Python behaviours everything rests on:
   - regex via PCRE2, checked against CPython on all 10,024 patterns;
   - CPython's Mersenne Twister;
   - Python `str` semantics;
   - a Python-like `Value` for exported data;
   - the conversation `Memory` and the runtime loop.
3. **Generate a contract skeleton.** One typed Rust stub for each of the ~10,800 Python
   functions. Workstreams can then call each other's functions before they are ported; a stub
   panics `UNPORTED`, which the runtime treats like a Python exception.
4. **Port in parallel workstreams.** There were 23 port workstreams, each in its own git
   worktree and branch:
   - the understanding engine (U);
   - the knowledge base (D1, D2, KNO);
   - memory (MEM1-8);
   - compose (COM1-5);
   - modules (MOD1-4).

   Each ported its files function by function and checked them against recorded PyBot calls
   before reporting. The coordinator merged the branches, fixed signature clashes between them
   and kept `main` building.
5. **Fix the remaining differences.** With nothing left unported, the turn-by-turn check left
   277 differing turns across 7 modules. Two fix workstreams (FIX1, FIX2) traced each one to its
   cause:
   - results dropped by unit-typed contract stubs;
   - Python's shallow `dict(memory)` copies;
   - entry replies writing to the live memory;
   - one private entry copy;
   - one checker artefact.

   Every turn on every set is now identical.
6. **Check end to end**, then **benchmark** against PyBot and **judge** on SmolTalkBench.

The user's prompts for the whole build, in order up to the merge of PR #1, are in
[docs/session-prompts.md](docs/session-prompts.md) (an API key removed; screened for slurs).

### What it cost

Estimated from the token counts in the session transcripts, priced at Anthropic API list prices:

| Who | Model | Output tokens | Cache-write tokens | Cache-read tokens | Cost at list price |
|---|---|---|---|---|---|
| the 25 workstream agents | Claude Opus 5.5 | 10.5M | 39.4M | 2,952M | $1,115 |
| the 25 workstream agents | Claude Fable 5.2 | 1.0M | 8.4M | 190M | $211 |
| coordinating session | Claude Opus 5.5 | 0.5M | 4.8M | 123M | $72 |
| coordinating session | Claude Fable 5.2 | 0.03M | 0.2M | 7M | $6 |
| SmolTalkBench judging | Claude Haiku 5.5 (batch) | 0.23M | - | - | $0.18 |
| **total** | | **12.2M** | **52.8M** | **3,272M** | **about $1,400** |

- **Totals:** about 3.3 billion tokens in all, almost all of them cache reads. Each agent turn
  re-reads its whole conversation from the prompt cache, so reads dominate the count but cost
  little per token.
- **How it is priced:**
  - Opus 5.5 at $4 / $20 per million input / output tokens, $0.20 for cache reads;
  - Fable 5.2 at $8 / $40, $0.20 for cache reads;
  - cache writes at the 1-hour rate (2x input).
- **Range:** at the 5-minute cache-write rate (1.25x input) the total is about $1,220, so the
  range is roughly **$1,200 to $1,400**.
- **Not a bill:** the work actually ran on a Claude subscription with a usage limit, so this is
  the equivalent API price, not what was paid.

## How the port is built and checked

- **Data is exported, logic is translated.** PyBot's module-level data (~233k lines: word
  lists, reply pools, 10k regexes, 8,081 knowledge entries) is exported from Python into a
  bundle (`harness/export_data.py`) and read by name; the ~152k lines of logic in 10,800
  functions are translated to Rust function by function, by the agents, from the Python source.
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

## How to run it

You need Rust (stable, 1.85 or later; edition 2024) and Python 3.12. You also need PyBot's
exported data: rsbot reads PyBot's word lists, reply pools, regexes and knowledge entries from
one JSON bundle instead of retyping them.

**1. Set up the workspace once.** rsbot looks for the bundle at `../data/export/bundle.json`,
next to this repository. Set `RSBOT_DATA=/path/to/bundle.json` to keep it elsewhere.

```sh
mkdir work && cd work
git clone https://github.com/hyunhwan-bcm/rsbot rsbot && ln -s rsbot/harness harness
git clone https://github.com/huggingface/pybot upstream/pybot
git -C upstream/pybot checkout 24cbc0439e07fd015c0747ab3be6e782a42b5c5b
python3 harness/export_data.py data/export/bundle.json   # PyBot's data, about 47 MB
cd rsbot && cargo build --release                         # about 2 minutes
```

**2. Chat in the terminal.** The options are the same as `pybot.py`'s.

```sh
./target/release/rsbot                      # interactive chat (/reset starts over, /exit quits)
./target/release/rsbot -p "how do I boil an egg?"   # one reply, then exit
./target/release/rsbot -v -p "hello"        # also show which module answered and how long it took
echo "hi there" | ./target/release/rsbot    # one reply per line read from stdin
./target/release/rsbot -s 42                # another random seed (the default is 0, as in PyBot)
```

**3. Run a file of prompts or conversations.** Each JSONL row gets a fresh conversation. A row
with `"instruction"` is one turn; a row with `"turns": [...]` runs as one conversation, with
memory carried from turn to turn. The output has one JSON line per row, holding the reply or
replies, the module and the time per reply in milliseconds.

```sh
./target/release/rsbot --jsonl ../data/smoltalkbench/all.jsonl replies.jsonl
```

**4. Pin the date for exact comparisons.** Some replies depend on today's date (countdowns,
"next Tuesday"). Set `RSBOT_TODAY=2026-11-17`, the date every oracle was recorded on, to get
exactly PyBot's recorded replies; leave it unset to use the real date.

**5. Run the checks against PyBot.** Build them with `cargo build --profile fast`, a quicker
build for development.

```sh
export RSBOT_TODAY=2026-11-17
./target/fast/oracle_check understood gold        # how the bot reads each benchmark prompt
./target/fast/entry_check gold                    # every knowledge-entry guard and reply
./target/fast/turn_check ../data/export/turn_oracle/conversations.jsonl.gz          # turn by turn
./target/fast/turn_check ../data/export/turn_oracle/conversations.jsonl.gz --e2e    # whole conversations
```

The turn oracles are recorded from PyBot with `harness/turn_oracle.py`; its docstring has the
command.

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
