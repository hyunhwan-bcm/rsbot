"""Record calls to COM4's functions (bot/compose/__init__.py and 47 more compose files) while PyBot
runs whole conversations, for the function-level check `src/bin/com4_check.rs`.

    python3 harness/com4_calls.py OUT_DIR [--oracle FILE.jsonl.gz] [--limit N] [--workers 2]

The conversations are read back from a turn oracle (its "text" per turn, in order) and run
through `runtime.turn` exactly as the oracle ran them (clock pinned, random.seed(0) per
conversation). Every module-level function of the three files is wrapped wherever it is bound
(its own module and every module that imported it by name), so internal calls are seen too.

Writes OUT_DIR/calls.jsonl: {"fn": "bot.compose.fallback:_has", "args": [...], "result": ...,
"raised": null | "TypeError"} with every default filled in; a dict argument (a memory, a
reading) is stored once in OUT_DIR/objs.jsonl and referenced as {"$ref": hash}. Each function
keeps up to 40 distinct calls, then up to 160 when the result is one it has seen fewer than 6
times, so rare outcomes are kept.
"""
import argparse, collections, gzip, hashlib, inspect, json, os, random, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python
from export_data import encode
from plain import plain

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
FIRST, MOST, PER_RESULT = 60, 240, 8


class Unencodable(Exception):
    pass


def enc(v):
    if isinstance(v, dict):
        return {k if isinstance(k, str) else repr(k): plain(encode(x)) for k, x in v.items()}
    out = plain(encode(v))
    if isinstance(out, dict) and "opaque" in out:
        raise Unencodable(type(v).__name__)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--oracle", default=os.path.join(os.path.dirname(__file__), "..", "..", "data",
                                                     "export", "turn_oracle", "conversations.jsonl.gz"))
    ap.add_argument("--limit", type=int)
    ap.add_argument("--workers", type=int, default=2)
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)

    convs = collections.OrderedDict()
    try:
        with gzip.open(a.oracle, "rt") as f:
            for line in f:
                try:
                    r = json.loads(line)
                except Exception:  # noqa: BLE001 - a file still being written ends mid-line
                    break
                if a.limit and r["conv"] not in convs and len(convs) >= a.limit:
                    break
                convs.setdefault(r["conv"], []).append((r["turn"], r["text"]))
    except EOFError:
        pass  # a file still being written: the conversations read so far
    # the last conversation may be cut short by a file still being written
    if convs:
        convs.popitem()
    convs = [[t for _, t in sorted(v)] for v in convs.values()][: a.limit]

    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime  # noqa: F401 - imports everything
    import importlib

    records = collections.defaultdict(dict)      # fn -> {key: record}
    result_counts = collections.defaultdict(collections.Counter)
    objs = {}

    def ref(v):
        e = json.dumps(enc(v), sort_keys=True, ensure_ascii=False)
        h = hashlib.sha1(e.encode()).hexdigest()[:16]
        objs.setdefault(h, e)
        return {"$ref": h}

    def wrap(name, f):
        sig = inspect.signature(f)

        def w(*args, **kwargs):
            recs = records[name]
            if len(recs) >= MOST:
                return f(*args, **kwargs)
            try:
                bound = sig.bind(*args, **kwargs)
                bound.apply_defaults()
                enc_args = [ref(v) if isinstance(v, dict) else enc(v) for v in bound.arguments.values()]
                key = json.dumps(enc_args, sort_keys=True, ensure_ascii=False)
            except Exception:  # noqa: BLE001 - an argument the check cannot carry
                return f(*args, **kwargs)
            if key in recs:
                return f(*args, **kwargs)
            raised = None
            try:
                res = f(*args, **kwargs)
            except Exception as e:
                raised, res = type(e).__name__, None
                exc = e
            try:
                enc_res = enc(res)
            except Exception:  # noqa: BLE001
                enc_res = {"unencodable": type(res).__name__, "truthy": bool(res)}
            rkey = json.dumps(enc_res, sort_keys=True, ensure_ascii=False)
            if len(recs) < FIRST or result_counts[name][rkey] < PER_RESULT:
                recs[key] = {"fn": name, "args": enc_args, "result": enc_res, "raised": raised}
                result_counts[name][rkey] += 1
            if raised:
                raise exc
            return res
        w.__wrapped_com4__ = f
        return w

    originals = {}
    for mod in MODULES:
        m = importlib.import_module(mod)
        for attr, v in list(vars(m).items()):
            if inspect.isfunction(v) and v.__module__ == mod:
                originals[id(v)] = (v, wrap(f"{mod}:{attr}", v))
    for m in list(sys.modules.values()):
        d = getattr(m, "__dict__", None)
        if not isinstance(d, dict):
            continue
        for attr, v in list(d.items()):
            if inspect.isfunction(v) and id(v) in originals and originals[id(v)][0] is v:
                d[attr] = originals[id(v)][1]

    def run(chunk, q):
        for turns in chunk:
            random.seed(0)
            memory = {}
            for text in turns:
                try:
                    runtime.turn(text, memory)
                except Exception:  # noqa: BLE001
                    pass
        q.put(({k: list(v.values()) for k, v in records.items()}, objs))

    import multiprocessing as mp
    ctx = mp.get_context("fork")
    q = ctx.Queue()
    ps = [ctx.Process(target=run, args=(convs[i::a.workers], q)) for i in range(a.workers)]
    for p in ps:
        p.start()
    merged, allobjs = collections.defaultdict(dict), {}
    for _ in ps:
        recs, ob = q.get()
        allobjs.update(ob)
        for fn, rs in recs.items():
            for r in rs:
                merged[fn].setdefault(json.dumps(r["args"], sort_keys=True), r)
    for p in ps:
        p.join()
    used = set()
    with open(os.path.join(a.out, "calls.jsonl"), "w") as f:
        for fn in sorted(merged):
            for r in merged[fn].values():
                for x in r["args"]:
                    if isinstance(x, dict) and "$ref" in x:
                        used.add(x["$ref"])
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
    with open(os.path.join(a.out, "objs.jsonl"), "w") as f:
        for h in used:
            f.write(json.dumps({"h": h, "v": json.loads(allobjs[h])}, ensure_ascii=False) + "\n")
    total = sum(len(v) for v in merged.values())
    print(f"{len(convs)} conversations; {len(merged)} functions called; {total} calls kept; "
          f"{len(used)} objects", file=sys.stderr)


if __name__ == "__main__":
    main()
