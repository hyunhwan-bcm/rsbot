"""Line coverage of PyBot's bot/ package while it answers a JSONL of prompts (sys.monitoring, 3.12+).

    python harness/coverage_run.py IN.jsonl OUT.json [--workers N]

Writes {file: [executed line numbers]} for files under bot/. Uses fork workers; each worker
records lines with sys.monitoring and sends its set back.
"""
import json, os, sys, argparse, random
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python

def main():
    ap = argparse.ArgumentParser(); ap.add_argument("inp"); ap.add_argument("out")
    ap.add_argument("--workers", type=int, default=10); ap.add_argument("--today", default="2026-11-17")
    ap.add_argument("--handlers"); ap.add_argument("--observers")
    a = ap.parse_args()
    run_python.pin_clock(a.today)
    root = os.path.abspath(run_python.PYBOT)
    sys.path.insert(0, root)
    mon = sys.monitoring
    TOOL = 3
    hits = {}
    botdir = os.path.join(root, "bot") + os.sep
    def on_line(code, line):
        fn = code.co_filename
        if fn.startswith(botdir):
            hits.setdefault(fn[len(root)+1:], set()).add(line)
        return mon.DISABLE
    mon.use_tool_id(TOOL, "cov")
    mon.register_callback(TOOL, mon.events.LINE, on_line)
    mon.set_events(TOOL, mon.events.LINE)
    from bot import runtime
    import_hits = {k: set(v) for k, v in hits.items()}
    if a.handlers is not None:
        keep = set(a.handlers.split(",")); runtime._HANDLERS[:] = [h for h in runtime._HANDLERS if h[0] in keep]
    if a.observers is not None:
        keep = set(filter(None, a.observers.split(","))); runtime._OBSERVERS[:] = [h for h in runtime._OBSERVERS if h[0] in keep]
    rows = [json.loads(l) for l in open(a.inp)]
    chunks = [rows[i::a.workers] for i in range(a.workers)]
    import multiprocessing as mp
    def work(chunk, q):
        hits.clear(); mon.restart_events()
        for r in chunk:
            random.seed(0)
            try: runtime.turn(r["instruction"], {})
            except Exception: pass
        q.put({k: sorted(v) for k, v in hits.items()})
    ctx = mp.get_context("fork"); q = ctx.Queue()
    ps = [ctx.Process(target=work, args=(c, q)) for c in chunks]
    for p in ps: p.start()
    turn_hits = {}
    for _ in ps:
        for k, v in q.get().items(): turn_hits.setdefault(k, set()).update(v)
    for p in ps: p.join()
    json.dump({"import": {k: sorted(v) for k, v in import_hits.items()},
               "turn": {k: sorted(v) for k, v in turn_hits.items()}}, open(a.out, "w"))
    print("files touched during turns:", len(turn_hits), "lines:", sum(map(len, turn_hits.values())), file=sys.stderr)

if __name__ == "__main__":
    main()
