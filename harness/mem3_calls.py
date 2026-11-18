"""Record calls to the MEM3 modules (bot.memory.topics / named / thread / deadline) while PyBot
runs whole conversations, for the Rust port's function-level check (`src/bin/mem3_check.rs`).

    python3 harness/mem3_calls.py OUT.jsonl.gz ORACLE.jsonl.gz [--limit N] [--stride K] [--cap C]

The conversations are read back from a turn oracle file (conv, turn, text). Every function of the
four modules is wrapped at module level, so the calls the modules make to each other are seen too.
Each distinct call (by its encoded arguments) is kept, at most C per function:
  {"fn": "bot.memory.topics:note", "args": [...], "kwargs": {...}, "result": ..., "raised": name|null,
   "mem_after": memory after the call (only for the functions that write memory),
   "alias": index of memory["topics"] that is the very memory["topic"] object, or null}
A memory argument is plain JSON with "turns" and "understood"; a reading is its plain dict; an
Entry is {"entry": id}; a callable is {"fn": "module:qualname"}.
"""
import argparse, gzip, hashlib, json, os, random, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python
from export_data import encode
from plain import plain

MODULES = ("bot.memory.topics", "bot.memory.named", "bot.memory.thread", "bot.memory.deadline")
DEPS = [
    ("bot.memory.facts", "a_moved_place", []), ("bot.memory.facts", "extract", []),
    ("bot.memory.facts", "no_tone", ["bot.memory.thread"]),
    ("bot.memory.facts", "without_the_correction_opener", []),
    ("bot.memory.detail", "answers_the_line", []), ("bot.memory.detail", "the_detail", []),
    ("bot.memory.already", "the_step_they_took", []), ("bot.memory.repeat", "_tried", []),
    ("bot.memory.quotes", "sentences", []), ("bot.memory.quotes", "closed", []),
    ("bot.memory.again", "carries_a_lead_in", []),
    ("bot.memory.returned", "says_it_happened_again", []),
    ("bot.memory.narrow", "narrow", []), ("bot.memory.narrow", "part_in_the_unit", []),
    ("bot.memory.narrow", "kind_of", []),
    ("bot.memory.filler", "without_the_shrug", ["bot.memory.topics"]),
    ("bot.memory.when", "asks_what_today_is", []),
    ("bot.compose.fallback", "is_a_hedge", []), ("bot.compose.fallback", "reads_as_a_judgement", []),
    ("bot.memory", "_to_the_bot", ["bot.memory.thread"]),
    ("bot.memory", "_from_the_bot", ["bot.memory.thread"]),
    ("bot.compose.trouble", "reads_as_trouble", []),
]
WRITERS = {"note", "_note", "remember_answer", "_keep_person", "_resume", "_back_to",
           "the_step_back_takes_its_words_back", "_the_step_back_is_undone", "_the_words_this_turn",
           "_the_detail_steps_back", "_the_plan_a_correction_renames", "_the_answer_on_the_return"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out"); ap.add_argument("oracle")
    ap.add_argument("--limit", type=int, default=200); ap.add_argument("--stride", type=int, default=1)
    ap.add_argument("--cap", type=int, default=40)
    a = ap.parse_args()
    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime
    from bot.knowledge import lookup
    import importlib, inspect

    convs, order = {}, []
    with gzip.open(a.oracle, "rt") as f:
        for line in f:
            r = json.loads(line)
            if r["conv"] not in convs:
                order.append(r["conv"]); convs[r["conv"]] = []
            convs[r["conv"]].append(r["text"])
    order = order[::a.stride][:a.limit]

    def enc(v, memo=None):
        if isinstance(v, lookup.Entry):
            return {"entry": v.id}
        if callable(v) and not isinstance(v, type):
            return {"fn": f"{getattr(v, '__module__', '')}:{getattr(v, '__qualname__', '')}"}
        if isinstance(v, dict):
            return {str(k): enc(x) for k, x in v.items()}
        if isinstance(v, (list, tuple, set, frozenset)):
            items = [enc(x) for x in v]
            return sorted(items, key=json.dumps) if isinstance(v, (set, frozenset)) else items
        try:
            return plain(encode(v))
        except Exception:  # noqa: BLE001
            return repr(v)

    def alias_of(args, kwargs):
        for v in list(args) + list(kwargs.values()):
            if isinstance(v, dict) and "turns" in v or (isinstance(v, dict) and "topic" in v):
                topic, topics = v.get("topic"), v.get("topics")
                if isinstance(topics, list):
                    for i, t in enumerate(topics):
                        if t is topic and topic is not None:
                            return i
                return None
        return None

    counts, seen = {}, set()
    out = gzip.open(a.out, "wt")

    def wrap(modname, name, f):
        key_fn = f"{modname}:{name}"
        writes = name in WRITERS

        def w(*args, **kwargs):
            if counts.get(key_fn, 0) >= a.cap:
                return f(*args, **kwargs)
            try:
                enc_args = enc(list(args)); enc_kw = enc(kwargs)
                h = hashlib.sha1(json.dumps([key_fn, enc_args, enc_kw], sort_keys=True,
                                            ensure_ascii=False).encode()).hexdigest()
            except Exception:  # noqa: BLE001
                return f(*args, **kwargs)
            if h in seen:
                return f(*args, **kwargs)
            alias = alias_of(args, kwargs)
            state = random.getstate()
            raised = None
            try:
                res = f(*args, **kwargs)
            except Exception as ex:  # noqa: BLE001
                raised = type(ex).__name__; res = None
                exc = ex
            rec = {"fn": key_fn, "args": enc_args, "kwargs": enc_kw, "alias": alias,
                   "result": enc(res), "raised": raised,
                   "rng_used": random.getstate() != state}
            if writes:
                mem = next((v for v in args if isinstance(v, dict)), None)
                rec["mem_after"] = enc(mem) if mem is not None else None
            seen.add(h)
            counts[key_fn] = counts.get(key_fn, 0) + 1
            out.write(json.dumps(rec, ensure_ascii=False) + "\n")
            if raised:
                raise exc
            return res
        return w

    # The other files' functions the four modules call: their results are recorded too, keyed by
    # the simple arguments (str, number, bool, None, list of str; anything else is null), so the
    # checker can answer those calls while the files are still stubs in Rust.
    dep_seen = set()

    def simple(v):
        if v is None or isinstance(v, (str, int, float, bool)):
            return v
        if isinstance(v, (list, tuple)) and all(isinstance(x, str) for x in v):
            return list(v)
        return None

    def wrap_dep(key_fn, f):
        def w(*args, **kwargs):
            res = f(*args, **kwargs)
            key = json.dumps([key_fn, [simple(x) for x in args]], ensure_ascii=False)
            if key not in dep_seen:
                dep_seen.add(key)
                out.write(json.dumps({"dep": key_fn, "args": [simple(x) for x in args],
                                      "result": enc(res)}, ensure_ascii=False) + "\n")
            return res
        return w

    for modname, name, also in DEPS:
        mod = importlib.import_module(modname)
        wrapped = wrap_dep(f"{modname}:{name}", getattr(mod, name))
        setattr(mod, name, wrapped)
        for other in also:
            setattr(importlib.import_module(other), name, wrapped)

    for modname in MODULES:
        mod = importlib.import_module(modname)
        for name, f in list(vars(mod).items()):
            if inspect.isfunction(f) and f.__module__ == modname:
                setattr(mod, name, wrap(modname, name, f))

    for n, cid in enumerate(order):
        random.seed(0)
        memory = {}
        for text in convs[cid]:
            try:
                runtime.turn(text, memory)
            except Exception:  # noqa: BLE001
                break
        if n % 20 == 0:
            print(n, len(order), sum(counts.values()), file=sys.stderr, flush=True)
    out.close()
    print(json.dumps(counts, indent=0, sort_keys=True), file=sys.stderr)


if __name__ == "__main__":
    main()
