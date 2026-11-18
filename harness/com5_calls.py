"""Record the calls PyBot makes to the COM5 files (55 small readers of bot/compose) while it runs
whole conversations, for src/bin/com5_check.rs to replay.

    python3 harness/com5_calls.py OUT.jsonl.gz ORACLE [--limit N] [--skip N] [--workers 2] [--cap 300]

ORACLE is a turn oracle name (conversations, single, synthetic, extra): its conversations are
replayed from their user lines. Every top-level function of the COM5 files is wrapped (module
attribute, so calls inside the module are seen too). One JSON line per distinct call:
  {"fn": "bot.memory.narrow:narrow", "args": [...], "kwargs": {...}, "result": ..., "type": "str",
   "raised": null | "ExcName", "rng": null | [625 ints before the call, when the call used the
   global RNG], "mem_after": null | plain memory after (when the call changed it)}
A memory argument is recorded as {"__memory__": plain memory}; a reading as {"__reading__": ...};
a compiled pattern as {"__re__": pattern, "flags": n}. Calls whose arguments cannot be put in JSON
(match objects, iterators) are skipped. At most --cap distinct calls are kept per function.
"""
import argparse, gzip, inspect, json, os, random, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "harness"))
import run_python  # noqa: E402
from export_data import encode  # noqa: E402
from plain import plain  # noqa: E402

MODULES = tuple("bot." + line.split(" ->")[0][4:-3].replace("/", ".")
                for line in open(os.path.join(ROOT, "slice", "ws", "COM5.txt")) if line.strip())


def oracle_conversations(name):
    """The conversations of a turn oracle, as {"id", "turns": [user lines]}, in file order."""
    path = os.path.join(ROOT, "data", "export", "turn_oracle", name + ".jsonl.gz")
    convs, order = {}, []
    with gzip.open(path, "rt") as f:
        while True:
            try:
                line = f.readline()
            except EOFError:
                break
            if not line:
                break
            try:
                rec = json.loads(line)
            except Exception:  # noqa: BLE001 - a file still being written ends mid-line
                break
            if rec["conv"] not in convs:
                convs[rec["conv"]] = []
                order.append(rec["conv"])
            convs[rec["conv"]].append((int(rec["turn"]), rec["text"]))
    return [{"id": c, "turns": [t for _, t in sorted(convs[c])]} for c in order]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out"); ap.add_argument("conversations")
    ap.add_argument("--limit", type=int); ap.add_argument("--skip", type=int, default=0)
    ap.add_argument("--workers", type=int, default=2); ap.add_argument("--cap", type=int, default=300)
    a = ap.parse_args()
    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime
    import importlib
    seen, kept, counts = set(), [], {}
    busy = [False]

    def conv_arg(v):
        if isinstance(v, dict) and (type(v).__name__ == "Understood" or ("text" in v and "clean" in v and "words" in v)):
            return {"__reading__": plain(encode(v))}
        if isinstance(v, dict) and ("turns" in v or type(v).__name__ == "Memory" or not v or "understood" in v):
            return {"__memory__": plain(encode(v))}
        if isinstance(v, re.Pattern):
            return {"__re__": v.pattern, "flags": int(v.flags)}
        if isinstance(v, (str, int, float, bool)) or v is None:
            return v
        if isinstance(v, (list, tuple, set, frozenset)):
            items = [conv_arg(x) for x in v]
            if isinstance(v, (set, frozenset)):
                try:
                    items = sorted(items)
                except TypeError:
                    pass
            return items
        if isinstance(v, dict):
            return {"__dict__": plain(encode(v))}
        raise TypeError(type(v).__name__)

    def wrap(modname, name, f):
        fid = f"{modname}:{name}"
        params = list(inspect.signature(f).parameters)

        def w(*args, **kwargs):
            if busy[0] or counts.get(fid, 0) >= a.cap:
                return f(*args, **kwargs)
            try:
                cargs = [conv_arg(x) for x in args]
                ckw = {k: conv_arg(v) for k, v in kwargs.items()}
                key = fid + json.dumps([cargs, ckw], sort_keys=True, ensure_ascii=False)
            except Exception:  # noqa: BLE001
                return f(*args, **kwargs)
            if key in seen:
                return f(*args, **kwargs)
            st = random.getstate()
            raised = None
            try:
                res = f(*args, **kwargs)
            except Exception as e:  # noqa: BLE001
                res, raised = None, e
            busy[0] = True
            try:
                rng = list(st[1]) if random.getstate() != st else None
                mem_after = None
                for i, x in enumerate(args):
                    if isinstance(cargs[i], dict) and "__memory__" in cargs[i]:
                        after = plain(encode(x))
                        if after != cargs[i]["__memory__"]:
                            mem_after = after
                try:
                    cres = plain(encode(res))
                except Exception:  # noqa: BLE001
                    cres = repr(res)
                seen.add(key)
                counts[fid] = counts.get(fid, 0) + 1
                kept.append({"fn": fid, "params": params, "args": cargs, "kwargs": ckw, "result": cres,
                             "type": type(res).__name__, "raised": type(raised).__name__ if raised else None,
                             "rng": rng, "mem_after": mem_after})
            finally:
                busy[0] = False
            if raised is not None:
                raise raised
            return res
        w.__wrapped_com5__ = True
        return w

    for modname in MODULES:
        mod = importlib.import_module(modname)
        for name, f in list(vars(mod).items()):
            if inspect.isfunction(f) and f.__module__ == modname:
                setattr(mod, name, wrap(modname, name, f))

    convs = oracle_conversations(a.conversations)[a.skip:]
    convs = convs[: a.limit]

    def work(chunk, q):
        seen.clear(); kept.clear(); counts.clear()
        for c in chunk:
            random.seed(0)
            memory = {}
            for text in c["turns"]:
                try:
                    runtime.turn(text, memory)
                except Exception:  # noqa: BLE001
                    pass
        q.put(list(kept))

    import multiprocessing as mp
    ctx = mp.get_context("fork"); q = ctx.Queue()
    ps = [ctx.Process(target=work, args=(convs[i::a.workers], q)) for i in range(a.workers)]
    for p in ps: p.start()
    allc, per = {}, {}
    for _ in ps:
        for c in q.get():
            k = c["fn"] + json.dumps([c["args"], c["kwargs"]], sort_keys=True, ensure_ascii=False)
            if k not in allc and per.get(c["fn"], 0) < a.cap:
                allc[k] = c
                per[c["fn"]] = per.get(c["fn"], 0) + 1
    for p in ps: p.join()
    with gzip.open(a.out, "wt") as f:
        for c in allc.values():
            f.write(json.dumps(c, ensure_ascii=False) + "\n")
    print(f"{len(allc)} calls over {len(per)} functions", file=sys.stderr)
    for fn, n in sorted(per.items()):
        print(f"  {n:5d} {fn}", file=sys.stderr)


if __name__ == "__main__":
    main()
