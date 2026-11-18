"""Record every call PyBot makes to the COM3 files' functions while it runs whole conversations.

    python harness/com2_calls.py OUT.jsonl.gz --conversations data/smoltalkbench/conversations.jsonl
           [--limit N] [--skip K] [--workers 2]

Every top-level function of the 13 COM3 files of bot/compose is wrapped
wherever it is bound (its own module and every module that imported the name), so the calls the
modules make between themselves are recorded too. One JSON line per distinct call (gzip):

  {"fn": "bot.compose.verdict:reply_for", "args": [...], "kwargs": {...}, "result": ..., "type": ...,
   "raised": exception name or null, "mem_after": hash or absent}
  {"memory": hash, "plain": {...}}      a memory argument, written once; an argument that was the
                                        memory is {"$memory": hash}, an Understood dict {"$understood": {...}}

The clock is pinned to 2026-11-17 and random.seed(0) runs before every conversation, as in the
turn oracle. `src/bin/com3_check.rs` replays the lines against the Rust port.
"""
import argparse, gzip, hashlib, inspect, json, os, random, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import run_python  # noqa: E402
from export_data import encode  # noqa: E402
from plain import plain  # noqa: E402

MODULES = tuple("bot.compose." + m for m in (
    "check_with_you", "checking", "fallback_pools", "finding_out", "goodnews", "loss", "onsubject",
    "quantity", "reading", "remark", "ruledout", "trouble", "what_i_do_know"))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--conversations", required=True)
    ap.add_argument("--limit", type=int, default=300)
    ap.add_argument("--skip", type=int, default=0)
    ap.add_argument("--workers", type=int, default=2)
    a = ap.parse_args()
    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime  # noqa: F401 - import everything, as at run time
    import importlib

    convs = [json.loads(line) for line in open(a.conversations)][a.skip:a.skip + a.limit]
    state = {"seen": set(), "mems": set(), "out": [], "depth": 0}

    def mem_ref(memory):
        try:
            enc = json.dumps(plain(encode(dict(memory))), sort_keys=True, ensure_ascii=False)
        except Exception:  # noqa: BLE001
            return None
        h = hashlib.sha1(enc.encode()).hexdigest()[:16]
        if h not in state["mems"]:
            state["mems"].add(h)
            state["out"].append(json.dumps({"memory": h, "plain": json.loads(enc)}, ensure_ascii=False))
        return h

    def arg(v):
        if isinstance(v, dict) and ("turns" in v or not v or "understood" in v or "compose" in v):
            h = mem_ref(v)
            return {"$memory": h} if h else {"$opaque": "memory"}
        if isinstance(v, dict) and ("clean" in v and "words" in v):
            return {"$understood": plain(encode(v))}
        if callable(v):
            return {"$callable": getattr(v, "__qualname__", "?")}
        try:
            return plain(encode(v))
        except Exception:  # noqa: BLE001
            return {"$opaque": type(v).__name__}

    def wrap(name, f):
        params = list(inspect.signature(f).parameters)

        def w(*args, **kwargs):
            before = [arg(x) for x in args]
            kw = {k: arg(v) for k, v in kwargs.items()}
            mem_args = [x for x in list(args) + list(kwargs.values()) if isinstance(x, dict) and "turns" in x]
            raised = None
            try:
                res = f(*args, **kwargs)
            except Exception as e:  # noqa: BLE001
                raised, res = type(e).__name__, None
            key = json.dumps([name, before, kw], sort_keys=True, ensure_ascii=False)
            if key not in state["seen"]:
                state["seen"].add(key)
                rec = {"fn": name, "params": params, "args": before, "kwargs": kw, "raised": raised,
                       "type": type(res).__name__}
                try:
                    rec["result"] = plain(encode(res))
                except Exception:  # noqa: BLE001
                    rec["result"] = {"$opaque": type(res).__name__}
                if mem_args:
                    rec["mem_after"] = mem_ref(mem_args[0])
                state["out"].append(json.dumps(rec, ensure_ascii=False))
            if raised:
                raise Exception(raised)
            return res
        w.__wrapped_com3__ = f
        return w

    originals = {}
    for modname in MODULES:
        mod = importlib.import_module(modname)
        for name, f in list(vars(mod).items()):
            if inspect.isfunction(f) and f.__module__ == modname:
                originals[id(f)] = (f"{modname}:{name}", f, wrap(f"{modname}:{name}", f))
    for mod in list(sys.modules.values()):
        if not getattr(mod, "__name__", "").startswith("bot"):
            continue
        for name, f in list(vars(mod).items()):
            if inspect.isfunction(f) and id(f) in originals:
                setattr(mod, name, originals[id(f)][2])

    def work(chunk, q):
        state["seen"].clear(); state["mems"].clear(); state["out"].clear()
        for c in chunk:
            random.seed(0)
            memory = {}
            for text in c["turns"]:
                try:
                    runtime.turn(text, memory)
                except Exception:  # noqa: BLE001
                    pass
        q.put(list(state["out"]))

    import multiprocessing as mp
    ctx = mp.get_context("fork")
    q = ctx.Queue()
    ps = [ctx.Process(target=work, args=(convs[i::a.workers], q)) for i in range(a.workers)]
    for p in ps:
        p.start()
    seen_mem, seen_call, n = set(), set(), 0
    with gzip.open(a.out, "wt") as fh:
        for _ in ps:
            for line in q.get():
                d = json.loads(line)
                key = ("m", d["memory"]) if "memory" in d else ("c", json.dumps([d["fn"], d["args"], d["kwargs"]], sort_keys=True))
                if key[0] == "m" and key in seen_mem or key[0] == "c" and key in seen_call:
                    continue
                (seen_mem if key[0] == "m" else seen_call).add(key)
                fh.write(line + "\n")
                n += 1
    for p in ps:
        p.join()
    print(f"{len(convs)} conversations: {len(seen_call)} distinct calls, {len(seen_mem)} memories", file=sys.stderr)


if __name__ == "__main__":
    main()
