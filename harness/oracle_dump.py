"""Per-stage oracle: what Python PyBot computes at each step of a single-turn reply.

    python harness/oracle_dump.py IN.jsonl OUT.jsonl [--workers N]

One line per prompt: {"id", "text", "understood" (after both observers), "intent_wants",
"memory_keys" (after observers), "candidates": {module: reply or null} for the 9 hot-path modules,
"memory_after": {module: sorted keys of that module's memory copy}, "chosen", "reply"}.
Values are JSON-encoded with harness/export_data.encode (tuples, sets and patterns tagged).
"""
import argparse, json, os, sys, random, copy
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python
from export_data import encode

H = "safety,memory_store,dreams,chat,answers,where_it_answers,first_line_open,the_shape_of_the_answer,greeting".split(",")

def main():
    ap = argparse.ArgumentParser(); ap.add_argument("inp"); ap.add_argument("out")
    ap.add_argument("--workers", type=int, default=11)
    a = ap.parse_args()
    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime
    runtime._HANDLERS[:] = [h for h in runtime._HANDLERS if h[0] in H]
    runtime._OBSERVERS[:] = [h for h in runtime._OBSERVERS if h[0] in ("understand", "intent")]
    rows = [json.loads(l) for l in open(a.inp)]

    def one(row):
        text = row["instruction"]
        random.seed(0)
        memory = {}
        for _, _, observe in runtime._OBSERVERS:
            try: observe(text, memory)
            except Exception: pass
        rec = {"id": row["id"], "text": text,
               "understood": encode(memory.get("understood")),
               "intent_wants": memory.get("intent_wants"),
               "memory_keys": sorted(memory)}
        cands, after, copies, order = {}, {}, [], []
        for name, prio, respond in runtime._HANDLERS:
            mine = runtime._copy(memory)
            try:
                ans = respond(text, mine)
            except Exception as e:
                ans = None; after[name + ":raised"] = type(e).__name__
            ok = isinstance(ans, str) and ans.strip()
            cands[name] = ans if ok else None
            after[name] = sorted(mine)
            if ok:
                order.append({"module": name, "priority": prio, "reply": ans}); copies.append(mine)
        chosen = runtime._choose(text, memory, order) if order else None
        rec.update({"candidates": cands, "memory_after": after,
                    "order": [c["module"] for c in order],
                    "chosen": order[chosen]["module"] if order else None,
                    "reply": " ".join(order[chosen]["reply"].split()) if order else "Hello!"})
        return rec

    import multiprocessing as mp
    global _ONE; _ONE = one
    with mp.get_context("fork").Pool(a.workers) as pool:
        res = pool.map(_call, rows, chunksize=4)
    with open(a.out, "w") as f:
        for r in res: f.write(json.dumps(r, ensure_ascii=False) + "\n")

def _call(r): return _ONE(r)

if __name__ == "__main__":
    main()
