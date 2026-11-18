"""Which functions each entry point (observer, module respond, arbiter) reaches on a prompt set.

    python harness/reach.py IN.jsonl OUT.json [--pybot DIR] [--workers N]

Writes {"funcs": {"file:qualname:line": [entry, ...]}}: for every bot function that ran during a
turn, the entry points whose call tree it was in.
"""
import argparse, json, os, sys, random
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python

H = "safety,memory_store,dreams,chat,answers,where_it_answers,first_line_open,the_shape_of_the_answer,greeting".split(",")
O = ["understand", "intent"]

def main():
    ap = argparse.ArgumentParser(); ap.add_argument("inp"); ap.add_argument("out")
    ap.add_argument("--pybot", default=run_python.PYBOT); ap.add_argument("--workers", type=int, default=11)
    a = ap.parse_args()
    run_python.pin_clock("2026-11-17")
    root = os.path.abspath(a.pybot); sys.path.insert(0, root)
    from bot import runtime
    runtime._HANDLERS[:] = [h for h in runtime._HANDLERS if h[0] in H]
    runtime._OBSERVERS[:] = [h for h in runtime._OBSERVERS if h[0] in O]
    cur = ["-"]
    seen = {}
    botdir = os.path.join(root, "bot") + os.sep
    def wrap(label, f):
        def g(*args):
            prev = cur[0]; cur[0] = label
            try: return f(*args)
            finally: cur[0] = prev
        return g
    runtime._OBSERVERS[:] = [(n, p, wrap("observe:" + n, f)) for n, p, f in runtime._OBSERVERS]
    runtime._HANDLERS[:] = [(n, p, wrap("respond:" + n, f)) for n, p, f in runtime._HANDLERS]
    runtime._ARBITER = wrap("arbiter", runtime._ARBITER)
    mon = sys.monitoring
    def on_start(code, off):
        fn = code.co_filename
        if fn.startswith(botdir):
            key = f"{fn[len(root)+1:]}:{code.co_qualname}:{code.co_firstlineno}"
            seen.setdefault(key, set()).add(cur[0])
    mon.use_tool_id(4, "reach"); mon.register_callback(4, mon.events.PY_START, on_start)
    rows = [json.loads(l) for l in open(a.inp)]
    import multiprocessing as mp
    def work(chunk, q):
        seen.clear(); mon.set_events(4, mon.events.PY_START)
        for r in chunk:
            random.seed(0)
            try: runtime.turn(r["instruction"], {})
            except Exception: pass
        q.put({k: sorted(v) for k, v in seen.items()})
    ctx = mp.get_context("fork"); q = ctx.Queue()
    ps = [ctx.Process(target=work, args=(rows[i::a.workers], q)) for i in range(a.workers)]
    for p in ps: p.start()
    allf = {}
    for _ in ps:
        for k, v in q.get().items(): allf.setdefault(k, set()).update(v)
    for p in ps: p.join()
    json.dump({"funcs": {k: sorted(v) for k, v in allf.items()}}, open(a.out, "w"), indent=0)
    print(len(allf), "functions", file=sys.stderr)

if __name__ == "__main__":
    main()
