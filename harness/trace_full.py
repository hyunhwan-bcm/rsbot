"""Trace the FULL PyBot (all modules, all observers) over single prompts and whole conversations.

    python harness/trace_full.py OUT_DIR [--single IN.jsonl] [--conversations CONV.jsonl]
                                 [--coverage] [--types N] [--workers W] [--limit K]

--coverage  line coverage of bot/ during turns (sys.monitoring), -> OUT_DIR/coverage.json
--types N   for every function, the Python types of its arguments and return value on up to N
            calls per worker (sys.setprofile), -> OUT_DIR/types.json
            {"file:qualname:line": {"params": [names], "args": [[type, count]...] per param,
             "ret": [[type, count]...], "calls": n}}

A conversation is answered turn by turn with one memory, random.seed(0) at its start, exactly as
`python -m pybot` would in a pipe. A single prompt is a one-turn conversation.
"""
import argparse, collections, json, os, random, re, sys, types as T

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python


def desc(v, depth=0):
    if v is None: return "None"
    if isinstance(v, bool): return "bool"
    if isinstance(v, int): return "int"
    if isinstance(v, float): return "float"
    if isinstance(v, str): return "str"
    if isinstance(v, re.Match): return "Match"
    if isinstance(v, re.Pattern): return "Pattern"
    if isinstance(v, (T.FunctionType, T.BuiltinFunctionType, T.MethodType)): return "Callable"
    if isinstance(v, T.ModuleType): return "Module"
    if depth >= 2:
        return type(v).__name__
    if isinstance(v, dict):
        if "text" in v and "clean" in v and "words" in v: return "Understood"
        if "understood" in v or "turns" in v: return "Memory"
        if not v: return "dict"
        ks = sorted({desc(k, 9) for k in list(v)[:6]}); vs = sorted({desc(x, depth + 1) for x in list(v.values())[:6]})
        return f"dict[{'|'.join(ks)}:{'|'.join(vs)}]"
    for kind, name in ((list, "list"), (tuple, "tuple"), (set, "set"), (frozenset, "set")):
        if isinstance(v, kind):
            if not v: return f"{name}"
            items = list(v)[:6]
            return f"{name}[{'|'.join(sorted({desc(x, depth + 1) for x in items}))}]"
    if hasattr(v, "__iter__") and type(v).__name__ in ("generator", "map", "filter", "zip"):
        return "iter"
    return type(v).__name__


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--single"); ap.add_argument("--conversations")
    ap.add_argument("--coverage", action="store_true"); ap.add_argument("--types", type=int, default=0)
    ap.add_argument("--workers", type=int, default=11); ap.add_argument("--limit", type=int)
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    run_python.pin_clock("2026-11-17")
    root = os.path.abspath(run_python.PYBOT); sys.path.insert(0, root)
    botdir = os.path.join(root, "bot") + os.sep
    from bot import runtime  # noqa: E402
    convs = []
    if a.single:
        convs += [{"id": r["id"], "turns": [r["instruction"]]} for r in map(json.loads, open(a.single))]
    if a.conversations:
        convs += [{"id": "conv-" + r["id"], "turns": r["turns"]} for r in map(json.loads, open(a.conversations))]
    convs = convs[: a.limit]

    hits = {}
    tstats = {}

    def on_line(code, line):
        fn = code.co_filename
        if fn.startswith(botdir):
            hits.setdefault(fn[len(root) + 1:], set()).add(line)
        return sys.monitoring.DISABLE

    def profiler(frame, event, arg):
        code = frame.f_code
        fn = code.co_filename
        if not fn.startswith(botdir):
            return
        key = (fn, code.co_qualname, code.co_firstlineno)
        st = tstats.get(key)
        if event == "call":
            if st is None:
                n = code.co_argcount + code.co_kwonlyargcount
                st = tstats[key] = {"params": list(code.co_varnames[:n]), "args": [collections.Counter() for _ in range(n)],
                                    "ret": collections.Counter(), "calls": 0, "seen": 0}
            st["calls"] += 1
            if st["seen"] < a.types:
                st["seen"] += 1
                loc = frame.f_locals
                for i, p in enumerate(st["params"]):
                    st["args"][i][desc(loc.get(p))] += 1
        elif event == "return" and st is not None and st["seen"] <= a.types and sum(st["ret"].values()) < a.types:
            st["ret"][desc(arg)] += 1

    def run(chunk, q):
        hits.clear(); tstats.clear()
        if a.coverage:
            sys.monitoring.restart_events()
            sys.monitoring.set_events(3, sys.monitoring.events.LINE)
        if a.types:
            sys.setprofile(profiler)
        for c in chunk:
            random.seed(0)
            memory = {}
            for t in c["turns"]:
                try:
                    runtime.turn(t, memory)
                except Exception:
                    pass
        sys.setprofile(None)
        q.put(({k: sorted(v) for k, v in hits.items()},
               {f"{k[0][len(root) + 1:]}:{k[1]}:{k[2]}": {"params": v["params"], "args": [dict(c) for c in v["args"]],
                                                         "ret": dict(v["ret"]), "calls": v["calls"]} for k, v in tstats.items()}))

    if a.coverage:
        sys.monitoring.use_tool_id(3, "cov")
        sys.monitoring.register_callback(3, sys.monitoring.events.LINE, on_line)
    import multiprocessing as mp
    ctx = mp.get_context("fork"); q = ctx.Queue()
    ps = [ctx.Process(target=run, args=(convs[i::a.workers], q)) for i in range(a.workers)]
    for p in ps: p.start()
    cov, typ = {}, {}
    for _ in ps:
        h, t = q.get()
        for k, v in h.items(): cov.setdefault(k, set()).update(v)
        for k, v in t.items():
            cur = typ.setdefault(k, {"params": v["params"], "args": [collections.Counter() for _ in v["params"]], "ret": collections.Counter(), "calls": 0})
            for i, c in enumerate(v["args"]): cur["args"][i].update(c)
            cur["ret"].update(v["ret"]); cur["calls"] += v["calls"]
    for p in ps: p.join()
    if a.coverage:
        json.dump({k: sorted(v) for k, v in cov.items()}, open(os.path.join(a.out, "coverage.json"), "w"))
        print("coverage: files", len(cov), "lines", sum(map(len, cov.values())), file=sys.stderr)
    if a.types:
        json.dump({k: {"params": v["params"], "args": [c.most_common() for c in v["args"]], "ret": v["ret"].most_common(), "calls": v["calls"]}
                   for k, v in typ.items()}, open(os.path.join(a.out, "types.json"), "w"))
        print("types: functions", len(typ), file=sys.stderr)


if __name__ == "__main__":
    main()
