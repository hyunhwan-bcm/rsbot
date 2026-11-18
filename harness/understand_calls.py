"""Record every understand(text, memory) call PyBot makes while answering prompts (hot path).

    python harness/understand_calls.py IN.jsonl... OUT.jsonl

Writes one line per distinct text: {"text", "understood" (plain JSON), "memory_dependent": bool}
where memory_dependent says whether the same text gave different readings under different memory.
"""
import json, os, sys, random
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python
from export_data import encode
from plain import plain

H = "safety,memory_store,dreams,chat,answers,where_it_answers,first_line_open,the_shape_of_the_answer,greeting".split(",")

def main():
    *ins, out = sys.argv[1:]
    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime
    import bot.understanding as U
    runtime._HANDLERS[:] = [h for h in runtime._HANDLERS if h[0] in H]
    runtime._OBSERVERS[:] = [h for h in runtime._OBSERVERS if h[0] in ("understand", "intent")]
    original = U.understand
    calls = {}
    def wrapped(text, memory=None, *a, **k):
        res = original(text, memory, *a, **k)
        try:
            enc = json.dumps(plain(encode(res)), sort_keys=True)
        except Exception:
            enc = None
        calls.setdefault(text, set()).add(enc)
        return res
    n = 0
    for name, mod in list(sys.modules.items()):
        if name.startswith("bot") and getattr(mod, "understand", None) is original:
            mod.understand = wrapped; n += 1
    print("patched", n, "modules", file=sys.stderr)
    rows = [json.loads(l) for f in ins for l in open(f)]
    import multiprocessing as mp
    def work(chunk, q):
        calls.clear()
        for r in chunk:
            random.seed(0)
            try: runtime.turn(r["instruction"], {})
            except Exception: pass
        q.put({k: sorted(v, key=str) for k, v in calls.items()})
    ctx = mp.get_context("fork"); q = ctx.Queue()
    ps = [ctx.Process(target=work, args=(rows[i::11], q)) for i in range(11)]
    for p in ps: p.start()
    allc = {}
    for _ in ps:
        for k, v in q.get().items(): allc.setdefault(k, set()).update(x for x in v if x)
    for p in ps: p.join()
    dep = 0
    with open(out, "w") as f:
        for text, vs in allc.items():
            vs = sorted(vs)
            dep += len(vs) > 1
            f.write(json.dumps({"text": text, "understood": json.loads(vs[0]), "memory_dependent": len(vs) > 1}, ensure_ascii=False) + "\n")
    print(len(allc), "distinct texts,", dep, "with memory-dependent readings", file=sys.stderr)

if __name__ == "__main__":
    main()
