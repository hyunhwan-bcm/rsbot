"""Record calls to bot/memory/store_replies.py's functions while PyBot runs conversations (MEM1).

    python3 harness/mem1_calls.py OUT.jsonl.gz ORACLE.jsonl.gz [--limit N] [--workers W]
        [--per-fn K] [--per-mem-fn M]

The conversations are the ones a turn oracle recorded (its "text" per turn, grouped by "conv"),
run through the full PyBot with the clock pinned and random.seed(0) per conversation, as the
oracles were. Every function of the module is wrapped in its module dict (so the module's own
calls go through the wrapper) and in bot.modules.memory_store's (which imports them by name).

One JSON line per distinct call:
  {"fn": name, "args": [plain...], "kwargs": {...}, "memory": [index of the memory arg, plain
   memory before] | null, "result": plain | null, "raised": exception name | null,
   "after": plain memory after (top-level keys other than understood/turns) | null}
Calls without a memory argument are kept up to --per-fn per function, calls with one up to
--per-mem-fn, so the file stays small. `src/bin/mem1_check.rs` replays them against the port.
"""
import argparse, collections, gzip, hashlib, inspect, json, os, random, sys

HARNESS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "harness")
sys.path.insert(0, os.path.abspath(HARNESS))
import run_python  # noqa: E402
from export_data import encode  # noqa: E402
from plain import plain  # noqa: E402


def snap(v):
    try:
        return plain(encode(v))
    except Exception as e:  # noqa: BLE001
        return {"__unencodable__": repr(e)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("oracle")
    ap.add_argument("--limit", type=int, default=400)
    ap.add_argument("--workers", type=int, default=2)
    ap.add_argument("--per-fn", type=int, default=300)
    ap.add_argument("--per-mem-fn", type=int, default=30)
    a = ap.parse_args()
    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime  # noqa: F401
    from bot.memory import store_replies as sr
    from bot.modules import memory_store as ms

    convs = collections.OrderedDict()
    with gzip.open(a.oracle, "rt") as fh:
        for line in fh:
            r = json.loads(line)
            convs.setdefault(r["conv"], []).append(r["text"])
            if len(convs) > a.limit:
                convs.popitem()
                break
    convs = list(convs.items())[: a.limit]

    calls, counts = {}, collections.Counter()

    def wrap(name, f):
        sig = inspect.signature(f)
        params = list(sig.parameters)
        mem_at = params.index("memory") if "memory" in params else None

        def w(*args, **kwargs):
            cap = a.per_mem_fn if mem_at is not None else a.per_fn
            if counts[name] >= cap:
                return f(*args, **kwargs)
            memory = None
            if mem_at is not None:
                if mem_at < len(args):
                    memory = args[mem_at]
                elif "memory" in kwargs:
                    memory = kwargs["memory"]
            # Every parameter, in order, with Python's defaults filled in (the port takes them all).
            try:
                bound = sig.bind(*args, **kwargs)
                bound.apply_defaults()
                values = list(bound.arguments.values())
            except TypeError:
                values = list(args)
            plain_args = [None if (mem_at is not None and i == mem_at) else snap(x) for i, x in enumerate(values)]
            plain_kwargs = {}
            before = snap(memory) if isinstance(memory, dict) else None
            raised = None
            try:
                res = f(*args, **kwargs)
            except Exception as e:  # noqa: BLE001
                res, raised = None, type(e).__name__
                exc = e
            rec = {"fn": name, "args": plain_args, "kwargs": plain_kwargs,
                   "memory": [mem_at, before] if mem_at is not None else None,
                   "result": None if raised else snap(res), "raised": raised, "after": None}
            if isinstance(memory, dict):
                rec["after"] = {k: snap(v) for k, v in memory.items() if k not in ("understood", "turns")}
            key = hashlib.sha1(json.dumps([name, plain_args, plain_kwargs, before], sort_keys=True,
                                          ensure_ascii=False).encode()).hexdigest()
            if key not in calls:
                calls[key] = rec
                counts[name] += 1
            if raised:
                raise exc
            return res
        w.__wrapped__ = f
        return w

    originals = {name: f for name, f in vars(sr).items()
                 if inspect.isfunction(f) and f.__module__ == sr.__name__}
    wrapped = {name: wrap(name, f) for name, f in originals.items()}
    for name, w in wrapped.items():
        setattr(sr, name, w)
    for name, value in list(vars(ms).items()):
        for oname, f in originals.items():
            if value is f:
                setattr(ms, name, wrapped[oname])

    for _, turns in convs:
        random.seed(0)
        memory = {}
        for text in turns:
            try:
                runtime.turn(text, memory)
            except Exception:  # noqa: BLE001
                pass
    with gzip.open(a.out, "wt") as fh:
        for rec in calls.values():
            fh.write(json.dumps(rec, ensure_ascii=False) + "\n")
    print(f"{len(convs)} conversations; {len(calls)} calls over {len(counts)} functions", file=sys.stderr)


if __name__ == "__main__":
    main()
