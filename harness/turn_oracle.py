"""Per-turn, per-stage oracle of the FULL PyBot over whole conversations.

    python harness/turn_oracle.py OUT.jsonl.gz --conversations CONV.jsonl [--single IN.jsonl] [--workers W] [--limit K]

One JSON line per turn (gzip):
  {"conv": id, "turn": k, "text": ..., "mem_in": plain memory at turn start,
   "rng_in": global random state at turn start,
   "observe": [{"module", "diff", "rng_after"}],          # each observer, memory diff vs before it
   "rng_before_respond": state after the observers,
   "respond": [{"module", "priority", "reply" | null, "raised" | null, "diff", "rng_after"}],
               # every module's respond on its own copy of the post-observer memory, in runtime order;
               # diff vs the post-observer memory; the global RNG is NOT reset between modules
               # (PyBot does not reset it either), so rng_after of one module is the next one's input
   "candidates": [module names in order], "chosen": module | null, "reply": final reply,
   "after": [{"module", "diff"}],                          # after hooks, diff vs before each
   "mem_out": plain memory at turn end}
Memory is plain JSON (tuples/sets as lists, str-keyed dicts as objects; other dicts as [[k, v]...]).
A diff is {"set": {key: value}, "del": [keys]} over top-level memory keys.
RNG state is CPython's getstate()[1] as a list of 625 ints, or the string "same" when unchanged.
"""
import argparse, copy, gzip, json, os, random, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python
from export_data import encode
from plain import plain


def snap(memory):
    try:
        return {k: plain(encode(v)) for k, v in memory.items()}
    except Exception as e:  # noqa: BLE001
        return {"__unencodable__": repr(e)}


def diff(before, after):
    d = {"set": {}, "del": []}
    for k, v in after.items():
        if k not in before or before[k] != v:
            d["set"][k] = v
    d["del"] = [k for k in before if k not in after]
    return d


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out"); ap.add_argument("--conversations"); ap.add_argument("--single")
    ap.add_argument("--workers", type=int, default=10); ap.add_argument("--limit", type=int)
    a = ap.parse_args()
    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime
    convs = []
    if a.conversations:
        convs += [{"id": "conv-" + r["id"], "turns": r["turns"]} for r in map(json.loads, open(a.conversations))]
    if a.single:
        convs += [{"id": r["id"], "turns": [r["instruction"]]} for r in map(json.loads, open(a.single))]
    convs = convs[: a.limit]

    def rng(prev):
        st = list(random.getstate()[1])
        return ("same", prev) if st == prev else (st, st)

    def conversation(c):
        out = []
        random.seed(0)
        memory = {}
        last = None
        for k, text in enumerate(c["turns"]):
            rec = {"conv": c["id"], "turn": k, "text": text, "mem_in": snap(memory)}
            rec["rng_in"], last = rng(None)
            obs = []
            before = rec["mem_in"]
            for name, _, observe in runtime._OBSERVERS:
                try:
                    observe(text, memory)
                except Exception:  # noqa: BLE001
                    pass
                now = snap(memory)
                r, last = rng(last)
                obs.append({"module": name, "diff": diff(before, now), "rng_after": r})
                before = now
            rec["observe"] = obs
            rec["rng_before_respond"], last = rng(None)
            post = before
            responds, candidates, copies = [], [], []
            for name, prio, respond in runtime._HANDLERS:
                mine = runtime._copy(memory)
                raised = None
                try:
                    ans = respond(text, mine)
                except Exception as e:  # noqa: BLE001
                    ans, raised = None, type(e).__name__
                ok = isinstance(ans, str) and ans.strip()
                r, last = rng(last)
                responds.append({"module": name, "priority": prio, "reply": ans if isinstance(ans, str) else None,
                                 "raised": raised, "diff": diff(post, snap(mine)), "rng_after": r})
                if ok:
                    candidates.append({"module": name, "priority": prio, "reply": ans})
                    copies.append(mine)
            rec["respond"] = responds
            rec["candidates"] = [x["module"] for x in candidates]
            if candidates:
                i = runtime._choose(text, memory, candidates)
                reply, module = candidates[i]["reply"], candidates[i]["module"]
                if copies[i] is not memory:
                    memory.clear(); memory.update(copies[i])
            else:
                reply, module = runtime.FALLBACK, None
            rec["chosen"], rec["reply"] = module, reply
            afters = []
            before = snap(memory)
            for name, _, after in runtime._AFTER:
                try:
                    after(text, reply, memory)
                except Exception:  # noqa: BLE001
                    pass
                now = snap(memory)
                afters.append({"module": name, "diff": diff(before, now)})
                before = now
            rec["after"] = afters
            turns = memory.get("turns")
            if not isinstance(turns, list):
                turns = memory["turns"] = []
            turns.append({"user": text, "bot": reply, "module": module})
            rec["mem_out"] = snap(memory)
            out.append(json.dumps(rec, ensure_ascii=False))
        return out

    import multiprocessing as mp
    global _CONV; _CONV = conversation
    with mp.get_context("fork").Pool(a.workers) as pool, gzip.open(a.out, "wt") as fh:
        n = 0
        for lines in pool.imap(_call, convs, chunksize=2):
            for line in lines:
                fh.write(line + "\n"); n += 1
    print(f"{len(convs)} conversations, {n} turns", file=sys.stderr)


def _call(c):
    return _CONV(c)


if __name__ == "__main__":
    main()
