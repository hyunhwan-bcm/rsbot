"""Probe the functions of the 13 COM3 files of bot/compose directly with the oracles' turns, for
the replay check `src/bin/com3_check.rs`. A whole-conversation recording (`com3_calls.py`) only
reaches a function when the chain above it does; this calls each one on every turn, so its deep
branches are reached too.

    python3 harness/com3_probe.py OUT.jsonl.gz [--oracle FILE.jsonl.gz ...] [--turns N] [--workers 2]

Arguments are filled by parameter name from the turn: `clean`/`tight` (the text squashed and
lowered), `text`/`line` (the text), `words` (clean.split()), `memory` (the turn's memory going in,
a fresh copy per call), `understood` (the reading of this turn), `said`/`reply`/`answer` (the
bot's last line), `repaired` (the reading's `plain`); a parameter with a default keeps it, and a
function with any other parameter is left out. random.seed(0) before every call; clock pinned.

Writes com3_calls.py's format: {"memory": h, "plain": ...} once per memory, and one line per call
with "params", positional "args" ({"$memory": h} / {"$understood": ...}), "result", "type" and
"raised". Each function keeps up to FIRST distinct calls, then up to MOST when the result is one
it has kept fewer than PER_RESULT times.
"""
import argparse, collections, copy, gzip, hashlib, inspect, json, os, random, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import run_python  # noqa: E402
from export_data import encode  # noqa: E402
from plain import plain  # noqa: E402

ORACLES = os.path.join(HERE, "..", "..", "data", "export", "turn_oracle")
MODULES = tuple("bot.compose." + m for m in (
    "check_with_you", "checking", "fallback_pools", "finding_out", "goodnews", "loss", "onsubject",
    "quantity", "reading", "remark", "ruledout", "trouble", "what_i_do_know"))
FIRST, MOST, PER_RESULT = 150, 1500, 20


def read_turns(paths, most):
    """(text, memory, understood) for distinct texts, round-robin over the oracles."""
    def one(path):
        prev = None
        try:
            with gzip.open(path, "rt") as f:
                for line in f:
                    try:
                        r = json.loads(line)
                    except Exception:  # noqa: BLE001
                        return
                    if prev is not None and prev["conv"] == r["conv"]:
                        und = (r.get("mem_in") or {}).get("understood")
                        if isinstance(und, dict):
                            yield prev["text"], prev.get("mem_in") or {}, und
                    prev = r
        except EOFError:
            return
    gens = [one(p) for p in paths]
    seen, out = set(), []
    while gens and len(out) < most:
        for g in list(gens):
            try:
                text, mem, und = next(g)
            except StopIteration:
                gens.remove(g)
                continue
            if text in seen:
                continue
            seen.add(text)
            out.append((text, mem, und))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--oracle", nargs="*", default=[os.path.join(ORACLES, n) for n in (
        "single.jsonl.gz", "conversations.jsonl.gz", "synthetic.jsonl.gz", "extra.jsonl.gz")])
    ap.add_argument("--turns", type=int, default=6000)
    ap.add_argument("--workers", type=int, default=2)
    a = ap.parse_args()
    turns = read_turns(a.oracle, a.turns)

    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime  # noqa: F401 - imports everything
    import importlib

    funcs = []
    for mod in MODULES:
        m = importlib.import_module(mod)
        for attr, v in sorted(vars(m).items()):
            if inspect.isfunction(v) and v.__module__ == mod:
                funcs.append((f"{mod}:{attr}", v))

    def mem_ref(memory, mems):
        enc = json.dumps(plain(encode(dict(memory))), sort_keys=True, ensure_ascii=False)
        h = hashlib.sha1(enc.encode()).hexdigest()[:16]
        mems.setdefault(h, enc)
        return {"$memory": h}

    def run(chunk, q):
        mems, out = {}, []
        prepared = []
        for text, mem, und in turns:
            clean = " ".join(text.lower().split())
            last = (mem.get("turns") or [{}])[-1] if isinstance(mem.get("turns"), list) else {}
            said = str((last or {}).get("bot") or "")
            repaired = str(und.get("plain") or clean)
            prepared.append({
                "clean": clean, "tight": clean, "text": text, "line": text, "words": clean.split(),
                "said": said, "reply": said, "answer": said, "repaired": repaired,
                "memory": mem, "understood": und,
            })
        for name, f in chunk:
            params = list(inspect.signature(f).parameters.values())
            if any(p.kind not in (p.POSITIONAL_OR_KEYWORD, p.POSITIONAL_ONLY) for p in params):
                continue
            if any(p.name not in prepared[0] and p.default is p.empty for p in params):
                continue
            recs, counts = {}, collections.Counter()
            for values in prepared:
                if len(recs) >= MOST:
                    break
                args, enc_args = [], []
                for p in params:
                    if p.name in values:
                        v = values[p.name]
                        if p.name == "memory":
                            args.append(copy.deepcopy(v))
                            enc_args.append(mem_ref(v, mems))
                        elif p.name == "understood":
                            args.append(copy.deepcopy(v))
                            enc_args.append({"$understood": plain(encode(v))})
                        else:
                            args.append(list(v) if isinstance(v, list) else v)
                            enc_args.append(plain(encode(v)))
                    else:
                        args.append(p.default)
                        enc_args.append(plain(encode(p.default)) if not isinstance(p.default, dict)
                                        else mem_ref(p.default, mems))
                key = json.dumps(enc_args, sort_keys=True, ensure_ascii=False)
                if key in recs:
                    continue
                random.seed(0)
                raised, res = None, None
                try:
                    res = f(*args)
                except Exception as e:  # noqa: BLE001
                    raised = type(e).__name__
                try:
                    enc_res = plain(encode(res))
                except Exception:  # noqa: BLE001
                    enc_res = {"$opaque": type(res).__name__}
                rkey = json.dumps(enc_res, sort_keys=True, ensure_ascii=False)
                if len(recs) < FIRST or counts[rkey] < PER_RESULT:
                    recs[key] = {"fn": name, "params": [p.name for p in params], "args": enc_args,
                                 "kwargs": {}, "result": enc_res, "type": type(res).__name__,
                                 "raised": raised}
                    counts[rkey] += 1
            out.extend(recs.values())
        q.put((out, mems))

    import multiprocessing as mp
    ctx = mp.get_context("fork")
    q = ctx.Queue()
    ps = [ctx.Process(target=run, args=(funcs[i::a.workers], q)) for i in range(a.workers)]
    for p in ps:
        p.start()
    calls, mems = [], {}
    for _ in ps:
        c, m = q.get()
        calls.extend(c)
        mems.update(m)
    for p in ps:
        p.join()
    used = {x["$memory"] for r in calls for x in r["args"] if isinstance(x, dict) and "$memory" in x}
    with gzip.open(a.out, "wt") as fh:
        for h in used:
            fh.write(json.dumps({"memory": h, "plain": json.loads(mems[h])}, ensure_ascii=False) + "\n")
        for r in calls:
            fh.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(f"{len(turns)} turns; {len({r['fn'] for r in calls})} functions probed; {len(calls)} calls",
          file=sys.stderr)


if __name__ == "__main__":
    main()
