"""Record every knowledge-entry guard and function-reply call PyBot makes (hot path).

    python harness/entry_calls.py OUT_DIR IN.jsonl...

Writes OUT_DIR/readings.jsonl ({"h": hash, "understood": plain JSON}) and OUT_DIR/calls.jsonl:
{"kind": "guard"|"reply", "fn": "module:qualname:line", "entry": entry id, "h": reading hash,
 "memory": plain JSON of memory without "understood"/"turns", "result": bool | str | null,
 "raised": exception name or null}, one line per distinct call.
"""
import hashlib, json, os, sys, random, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python
from export_data import encode
from plain import plain

H = "safety,memory_store,dreams,chat,answers,where_it_answers,first_line_open,the_shape_of_the_answer,greeting".split(",")

def fn_id(f):
    return f"{f.__module__}:{f.__qualname__}:{f.__code__.co_firstlineno}"

def main():
    out_dir, *ins = sys.argv[1:]
    os.makedirs(out_dir, exist_ok=True)
    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime
    from bot.knowledge import lookup
    runtime._HANDLERS[:] = [h for h in runtime._HANDLERS if h[0] in H]
    runtime._OBSERVERS[:] = [h for h in runtime._OBSERVERS if h[0] in ("understand", "intent")]
    import gc
    entries = [o for o in gc.get_objects() if isinstance(o, lookup.Entry)]
    readings, calls = {}, {}

    def rhash(reading):
        try:
            enc = json.dumps(plain(encode(reading)), sort_keys=True, ensure_ascii=False)
        except Exception:
            return None
        h = hashlib.sha1(enc.encode()).hexdigest()[:16]
        readings.setdefault(h, enc)
        return h

    def mem_view(memory):
        if not isinstance(memory, dict):
            return None
        try:
            return plain(encode({k: v for k, v in memory.items() if k not in ("understood", "turns")}))
        except Exception:
            return None

    def wrap_guard(e, g):
        fid = fn_id(g)
        def w(reading):
            raised = None
            try:
                res = g(reading)
            except Exception as ex:
                raised = type(ex).__name__; res = None
            key = ("guard", fid, e.id, rhash(reading))
            calls.setdefault(key, {"kind": "guard", "fn": fid, "entry": e.id, "h": key[3],
                                   "result": None if raised else bool(res), "raised": raised})
            if raised:
                raise Exception(raised)
            return res
        return w

    def wrap_reply(e, r):
        fid = fn_id(r)
        def w(memory, reading):
            before = mem_view(memory)
            raised = None
            try:
                res = r(memory, reading)
            except Exception as ex:
                raised = type(ex).__name__; res = None
            key = ("reply", fid, e.id, rhash(reading), json.dumps(before, sort_keys=True))
            calls.setdefault(key, {"kind": "reply", "fn": fid, "entry": e.id, "h": key[3], "memory": before,
                                   "result": res if isinstance(res, str) else None, "raised": raised})
            if raised:
                raise Exception(raised)
            return res
        return w

    for e in entries:
        if callable(e.guard):
            e.guard = wrap_guard(e, e.guard)
        if callable(e.reply):
            e.reply = wrap_reply(e, e.reply)
    rows = [json.loads(l) for f in ins for l in open(f)]
    import multiprocessing as mp
    def work(chunk, q):
        readings.clear(); calls.clear()
        for r in chunk:
            random.seed(0)
            try: runtime.turn(r["instruction"], {})
            except Exception: pass
        q.put((dict(readings), list(calls.values())))
    ctx = mp.get_context("fork"); q = ctx.Queue()
    ps = [ctx.Process(target=work, args=(rows[i::11], q)) for i in range(11)]
    for p in ps: p.start()
    allr, allc = {}, {}
    for _ in ps:
        rd, cl = q.get()
        allr.update(rd)
        for c in cl:
            allc[(c["kind"], c["fn"], c["entry"], c["h"], json.dumps(c.get("memory"), sort_keys=True))] = c
    for p in ps: p.join()
    with open(os.path.join(out_dir, "readings.jsonl"), "w") as f:
        for h, enc in allr.items():
            f.write(json.dumps({"h": h, "understood": json.loads(enc)}, ensure_ascii=False) + "\n")
    with open(os.path.join(out_dir, "calls.jsonl"), "w") as f:
        for c in allc.values():
            f.write(json.dumps(c, ensure_ascii=False) + "\n")
    import collections
    k = collections.Counter(c["kind"] for c in allc.values())
    fns = collections.Counter((c["kind"], c["fn"]) for c in allc.values())
    print(f"{len(allr)} distinct readings; calls {dict(k)}; distinct functions: guards {sum(1 for x in fns if x[0]=='guard')}, replies {sum(1 for x in fns if x[0]=='reply')}", file=sys.stderr)

if __name__ == "__main__":
    main()
