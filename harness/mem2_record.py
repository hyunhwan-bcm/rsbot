"""Record the calls PyBot makes to the MEM2 modules (facts, again, corrected, rename) and to the
functions of other modules they lean on, for a function-level check of the Rust port.

    python3 harness/mem2_record.py OUT.jsonl.gz --lines ORACLE.jsonl.gz [...] [--max N]
        [--cap 200] [--turns]

Without --turns every user line of the oracle files is handed to `facts.extract` (both as a
statement and as a question) and to `about_themselves`, which reaches nearly every reader in
facts.py. With --turns the whole conversations are replayed through the runtime, so the
functions other modules call (again.replied, corrected._apply_correction, facts.subject...) are
recorded with the arguments they really get.

Each distinct call is one line: {"fn": "module:qualname", "args": [...], "kwargs": {...},
"result": ..., "raised": null | "ExcName", "after": [...] (args after the call, when a call
changed them)}. A Match argument is {"__match__": {"re", "flags", "string", "pos", "endpos",
"spans"}}; a compiled pattern is {"__re__": [pattern, flags]}. Positions are Python's (code
points): the checker converts them.
"""
import argparse, copy, gzip, inspect, json, os, random, re, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "harness"))
import run_python  # noqa: E402

MINE = ("bot.memory.facts", "bot.memory.again", "bot.memory.corrected", "bot.memory.rename")
# Functions of other modules that the MEM2 files call and that are not ported yet: recorded with
# no cap, so a checker can answer them from the recording.
DEPS = {
    "bot.memory.taste": ("read_other", "said_about", "read_all", "read", "only_a_taste"),
    "bot.memory.filler": ("without_filler", "without_the_end_shrug", "without_the_shrug"),
    "bot.memory.named": ("the_name_in",),
    "bot.compose.badnews": ("reads_like_a_symptom",),
}


def enc(v, depth=0):
    if depth > 30:
        return {"__opaque__": "deep"}
    if v is None or isinstance(v, (bool, int, float, str)):
        return v
    if isinstance(v, re.Match):
        return {"__match__": {"re": v.re.pattern, "flags": int(v.re.flags), "string": v.string,
                              "pos": v.pos, "endpos": v.endpos,
                              "spans": [list(v.span(i)) for i in range(v.re.groups + 1)]}}
    if isinstance(v, re.Pattern):
        return {"__re__": [v.pattern, int(v.flags)]}
    if isinstance(v, (list, tuple)):
        return [enc(x, depth + 1) for x in v]
    if isinstance(v, (set, frozenset)):
        items = [enc(x, depth + 1) for x in v]
        try:
            return sorted(items)
        except TypeError:
            return items
    if isinstance(v, dict):
        if all(isinstance(k, str) for k in v):
            return {k: enc(x, depth + 1) for k, x in v.items()}
        return {"__pairs__": [[enc(k, depth + 1), enc(x, depth + 1)] for k, x in v.items()]}
    if inspect.isgenerator(v):
        return {"__opaque__": "generator"}
    return {"__opaque__": type(v).__name__}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--lines", nargs="+", required=True)
    ap.add_argument("--max", type=int, default=0, help="at most this many conversations per file")
    ap.add_argument("--cap", type=int, default=200)
    ap.add_argument("--turns", action="store_true")
    a = ap.parse_args()

    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    import importlib
    from bot import runtime  # noqa: F401
    seen, counts = set(), {}
    out = gzip.open(a.out, "wt")
    depth = [0]

    def wrap(name, fn, cap):
        def wrapper(*args, **kwargs):
            try:
                key = name + repr(args) + repr(sorted(kwargs.items()))
            except Exception:  # noqa: BLE001
                key = None
            fresh = key is not None and key not in seen and counts.get(name, 0) < cap
            before = None
            if fresh:
                try:
                    before = copy.deepcopy((args, kwargs))
                except Exception:  # noqa: BLE001
                    fresh = False
            depth[0] += 1
            try:
                result = fn(*args, **kwargs)
            except Exception as e:  # noqa: BLE001
                if fresh:
                    seen.add(key)
                    counts[name] = counts.get(name, 0) + 1
                    out.write(json.dumps({"fn": name, "args": enc(before[0]), "kwargs": enc(before[1]),
                                          "result": None, "raised": type(e).__name__},
                                         ensure_ascii=False) + "\n")
                raise
            finally:
                depth[0] -= 1
            if fresh:
                seen.add(key)
                counts[name] = counts.get(name, 0) + 1
                row = {"fn": name, "args": enc(before[0]), "kwargs": enc(before[1]),
                       "result": enc(result), "raised": None}
                after = enc(args)
                if after != row["args"]:
                    row["after"] = after
                out.write(json.dumps(row, ensure_ascii=False) + "\n")
            return result
        wrapper.__wrapped__ = fn
        return wrapper

    for modname in MINE:
        mod = importlib.import_module(modname)
        for attr, fn in list(vars(mod).items()):
            if inspect.isfunction(fn) and fn.__module__ == modname:
                cap = 10 ** 9 if attr in ("extract", "about_themselves") else a.cap
                setattr(mod, attr, wrap(f"{modname}:{attr}", fn, cap))
    for modname, names in DEPS.items():
        mod = importlib.import_module(modname)
        for attr in names:
            fn = getattr(mod, attr)
            w = wrap(f"{modname}:{attr}", fn, 10 ** 9)
            setattr(mod, attr, w)
            # Modules that imported the function by name hold their own reference.
            for user in MINE:
                umod = sys.modules[user]
                if getattr(umod, attr, None) is fn:
                    setattr(umod, attr, w)

    facts = sys.modules["bot.memory.facts"]
    done = 0
    for path in a.lines:
        convs = 0
        last = None
        with gzip.open(path, "rt") as f:
            for line in f:
                row = json.loads(line)
                if row.get("conv") != last:
                    last = row.get("conv")
                    convs += 1
                    if a.max and convs > a.max:
                        break
                text = row.get("text") or ""
                if a.turns:
                    continue
                for question in (False, True):
                    try:
                        facts.extract(text, question)
                    except Exception:  # noqa: BLE001
                        pass
                try:
                    facts.about_themselves(text)
                except Exception:  # noqa: BLE001
                    pass
                done += 1
        if a.turns:
            # Whole conversations, through the runtime, as the turn oracle ran them.
            from bot import runtime as rt
            with gzip.open(path, "rt") as f:
                convs, last, memory = 0, None, None
                for line in f:
                    row = json.loads(line)
                    if row.get("conv") != last:
                        last = row.get("conv")
                        convs += 1
                        if a.max and convs > a.max:
                            break
                        memory = {}
                        random.seed(0)
                    try:
                        rt.turn(row.get("text") or "", memory)
                    except Exception:  # noqa: BLE001
                        pass
                    done += 1
    out.close()
    print(f"{done} lines; {len(counts)} functions; {sum(counts.values())} calls", file=sys.stderr)


if __name__ == "__main__":
    main()
