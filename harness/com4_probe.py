"""Probe every function of the 48 COM4 compose files directly with the oracles' turns, for the
replay check `src/bin/com4_check.rs` (a copy of com1_probe.py). A whole-conversation recording
(`com4_calls.py`) only reaches a function when the chain above it does; this calls each one on
every turn, so its deep branches are reached too.

    python3 harness/com1_probe.py OUT_DIR [--oracle FILE.jsonl.gz ...] [--turns N] [--workers 2]

Arguments are filled by parameter name from the turn: `clean` (the text squashed and lowered),
`text`/`line` (the text), `words` (clean.split()), `memory` (the turn's memory going in, a fresh
copy per call), `turns` (its turns), `understood` (the reading of this turn, which the oracle
holds as the next turn's `mem_in.understood`), `said`/`reply`/`last`/`answer` (the bot's last
line), `repaired` (clean), `asking` (understood["asking"]), and clean again for a part of the
line (`sentence`, `question`, `first`, `rest`, `tail`, `after`, `phrase`, `prefix`); a parameter with a default keeps it, and a
function with any other parameter is left out. random.seed(0) before every call; clock pinned.

Writes OUT_DIR/calls.jsonl and objs.jsonl in com4_calls.py's format. Each function keeps up to
40 distinct calls, then up to 200 when the result is one it has kept fewer than 8 times.
"""
import argparse, collections, copy, gzip, hashlib, inspect, json, os, random, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python
from export_data import encode
from plain import plain

ORACLES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "data", "export",
                       "turn_oracle")
MODULES = (
    "bot.compose",
    "bot.compose.a_detail_dropped",
    "bot.compose.a_number_the_bot_knows",
    "bot.compose.a_place_nobody_can_see",
    "bot.compose.a_thing_being_weighed",
    "bot.compose.a_trouble",
    "bot.compose.a_wait_about_a_body",
    "bot.compose.badnews",
    "bot.compose.colour_on_the_body",
    "bot.compose.draft",
    "bot.compose.find_a_place",
    "bot.compose.findout",
    "bot.compose.gap",
    "bot.compose.habit",
    "bot.compose.harm",
    "bot.compose.how_it_went",
    "bot.compose.is_it_worth_it",
    "bot.compose.lead",
    "bot.compose.leaving",
    "bot.compose.limit",
    "bot.compose.missing_half",
    "bot.compose.nearby_one",
    "bot.compose.no_eyes",
    "bot.compose.not_knowing",
    "bot.compose.occasionplan",
    "bot.compose.onething",
    "bot.compose.only_they_know",
    "bot.compose.pace",
    "bot.compose.plan_ask",
    "bot.compose.plans",
    "bot.compose.reaction",
    "bot.compose.rotten_luck",
    "bot.compose.routes",
    "bot.compose.said_about_me",
    "bot.compose.saidby",
    "bot.compose.still",
    "bot.compose.stretches",
    "bot.compose.taste",
    "bot.compose.the_course_of_it",
    "bot.compose.the_shape_without_the_fact",
    "bot.compose.their_day",
    "bot.compose.their_detail",
    "bot.compose.thread_detail",
    "bot.compose.tone_again",
    "bot.compose.urgent",
    "bot.compose.what_decides_it",
    "bot.compose.who",
    "bot.compose.who_to_see",
)
FIRST, MOST, PER_RESULT = 40, 200, 8


def enc(v):
    if isinstance(v, dict):
        return {k if isinstance(k, str) else repr(k): plain(encode(x)) for k, x in v.items()}
    return plain(encode(v))


def read_turns(paths, most):
    """(text, memory, understood) for distinct texts, round-robin over the oracles."""
    def one(path):
        prev = None
        try:
            with gzip.open(path, "rt") as f:
                for line in f:
                    try:
                        r = json.loads(line)
                    except Exception:  # noqa: BLE001 - a file still being written
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
        "single.jsonl.gz", "conversations.jsonl.gz", "synthetic.jsonl.gz")])
    ap.add_argument("--turns", type=int, default=2000)
    ap.add_argument("--workers", type=int, default=2)
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
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

    def ref(v, objs):
        e = json.dumps(enc(v), sort_keys=True, ensure_ascii=False)
        h = hashlib.sha1(e.encode()).hexdigest()[:16]
        objs.setdefault(h, e)
        return {"$ref": h}

    from bot.compose import fallback as _fb, a_thing_being_weighed as _w, a_detail_dropped as _d
    placeholders = [""] + list(_fb.NEUTRAL) + list(_w.RECEIPTS) + [
        "Got it. Is there anything you'd like a hand with?", "Congratulations! That's great news.",
        "Nice one.", "That's beyond what I know well, sorry. Searching for it will turn up better sources.",
        "I'm not sure about that one - a quick search online will get you a solid answer."]

    def run(chunk, q):
        objs, records = {}, {}
        prepared = []
        for index, (text, mem, und) in enumerate(turns):
            clean = " ".join(text.lower().split())
            last = (mem.get("turns") or [{}])[-1] if isinstance(mem.get("turns"), list) else {}
            said = str((last or {}).get("bot") or "")
            prepared.append({
                "clean": (clean, clean), "text": (text, text), "line": (text, text),
                "words": (clean.split(), clean.split()), "repaired": (clean, clean),
                "said": (said, said), "last": (said, said),
                "reply": ((said, said) if index % 3 == 0 else
                          (placeholders[index % len(placeholders)],) * 2),
                "answer": (said, said),
                **{k: (clean, clean) for k in ("sentence", "question", "first", "rest", "tail",
                                               "after", "phrase", "prefix")},
                "asking": (und.get("asking"), enc(und.get("asking"))),
                "memory": (mem, ref(mem, objs)), "understood": (und, ref(und, objs)),
                "turns": (mem.get("turns") or [], enc(mem.get("turns") or [])),
            })
        for name, f in chunk:
            sig = inspect.signature(f)
            params = list(sig.parameters.values())
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
                        v, e = values[p.name]
                        args.append(copy.deepcopy(v) if isinstance(v, (dict, list)) else v)
                        enc_args.append(e)
                    else:
                        args.append(p.default)
                        enc_args.append(ref(p.default, objs) if isinstance(p.default, dict)
                                        else enc(p.default))
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
                    enc_res = enc(res)
                except Exception:  # noqa: BLE001
                    enc_res = {"unencodable": type(res).__name__, "truthy": bool(res)}
                rkey = json.dumps(enc_res, sort_keys=True, ensure_ascii=False)
                if len(recs) < FIRST or counts[rkey] < PER_RESULT:
                    recs[key] = {"fn": name, "args": enc_args, "result": enc_res, "raised": raised}
                    counts[rkey] += 1
            records[name] = list(recs.values())
        q.put((records, objs))

    import multiprocessing as mp
    ctx = mp.get_context("fork")
    q = ctx.Queue()
    ps = [ctx.Process(target=run, args=(funcs[i::a.workers], q)) for i in range(a.workers)]
    for p in ps:
        p.start()
    merged, allobjs = {}, {}
    for _ in ps:
        recs, ob = q.get()
        allobjs.update(ob)
        merged.update(recs)
    for p in ps:
        p.join()
    used = set()
    with open(os.path.join(a.out, "calls.jsonl"), "w") as f:
        for fn in sorted(merged):
            for r in merged[fn]:
                for x in r["args"]:
                    if isinstance(x, dict) and "$ref" in x:
                        used.add(x["$ref"])
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
    with open(os.path.join(a.out, "objs.jsonl"), "w") as f:
        for h in used:
            f.write(json.dumps({"h": h, "v": json.loads(allobjs[h])}, ensure_ascii=False) + "\n")
    print(f"{len(turns)} turns; {len(merged)} functions probed; "
          f"{sum(len(v) for v in merged.values())} calls kept; {len(used)} objects", file=sys.stderr)


if __name__ == "__main__":
    main()
