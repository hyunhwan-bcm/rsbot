"""Record the calls PyBot makes to the MEM5 modules (bot/memory: detail, enough, pile, relations,
saving, tried, trips, when) while it replays the turn-oracle conversations, for a function-level
check of the Rust port (src/bin/mem5_check.rs).

    python3 harness/mem5_record.py OUT.jsonl.gz ORACLE.jsonl.gz [...] [--mods tried,when]
        [--cap 300] [--max N] [--skip N] [--script FILE.json]

Each distinct call is one line: {"fn": "module:qualname", "args": [...], "kwargs": {...},
"result": ..., "raised": null | "ExcName", "after": [...] (args after the call, when the call
changed them)}. A function argument is {"__fn__": "module:qualname:line"}, a compiled pattern
{"__re__": [pattern, flags]}, a Match {"__match__": {...}} with Python (code point) positions.
--script plays a JSON list of conversations (lists of lines) instead of oracle files.
"""
import argparse, copy, gzip, inspect, json, os, random, re, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "harness"))
import run_python  # noqa: E402

ALL = ("detail", "enough", "pile", "relations", "saving", "tried", "trips", "when")
# Functions of other workstreams the MEM5 files call and that were not ported when this was
# written: recorded with no cap, keyed by their arguments with any dict (a memory) left out, so
# the test-only mock crate (harness/mem5_mockpatch.py) can answer them from the recording.
DEPS = {
    "bot.compose": ("pick",),
    "bot.compose.trouble": ("reads_as_trouble",),
    "bot.memory.not_a_trip": ("not_a_destination", "the_visit"),
    "bot.memory": ("plan_line", "last_bot"),
    "bot.memory.topics": ("content",),
    "bot.memory.condition": ("conditions",),
    "bot.memory.history": ("is_their_news",),
    "bot.memory.spoken": ("_spoken_as_numbers",),
    "bot.memory.purse": ("against_the_bands",),
    "bot.memory.places": ("_a_place",),
}


def lines_of(f):
    try:
        for line in f:
            if line.endswith("\n"):
                yield line
    except EOFError:
        return


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
            return {"__set__": sorted(items)}
        except TypeError:
            return {"__set__": items}
    if isinstance(v, dict):
        if all(isinstance(k, str) for k in v):
            return {k: enc(x, depth + 1) for k, x in v.items()}
        return {"__pairs__": [[enc(k, depth + 1), enc(x, depth + 1)] for k, x in v.items()]}
    if inspect.isfunction(v):
        inner = getattr(v, "__wrapped__", v)
        return {"__fn__": f"{inner.__module__}:{inner.__qualname__}:{inner.__code__.co_firstlineno}"}
    if inspect.isgenerator(v):
        return {"__opaque__": "generator"}
    import datetime
    if isinstance(v, datetime.datetime):
        return {"__datetime__": v.isoformat()}
    if isinstance(v, datetime.date):
        return {"__date__": v.isoformat()}
    return {"__opaque__": type(v).__name__}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("oracles", nargs="*")
    ap.add_argument("--mods", default=",".join(ALL))
    ap.add_argument("--cap", type=int, default=300)
    ap.add_argument("--max", type=int, default=0, help="at most this many conversations per file")
    ap.add_argument("--skip", type=int, default=0, help="skip this many conversations per file")
    ap.add_argument("--script")
    a = ap.parse_args()

    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    import importlib
    from bot import runtime as rt
    mine = tuple("bot.memory." + m for m in a.mods.split(","))
    seen, counts = set(), {}
    out = gzip.open(a.out, "wt")

    def keyed(args):
        return [None if isinstance(a, dict) or (isinstance(a, (list, tuple)) and any(isinstance(x, dict) for x in a)) else a for a in args]

    def wrap(name, fn, cap, dep=False):
        def wrapper(*args, **kwargs):
            if dep:
                result = fn(*args, **kwargs)
                try:
                    key = json.dumps([name, enc(keyed(args))], ensure_ascii=False)
                except Exception:  # noqa: BLE001
                    return result
                if key not in seen:
                    seen.add(key)
                    out.write(json.dumps({"dep": name, "key": enc(keyed(args)), "result": enc(result)},
                                         ensure_ascii=False) + "\n")
                return result
            fresh = counts.get(name, 0) < cap
            key = before = None
            if fresh:
                try:
                    key = name + repr(args) + repr(sorted(kwargs.items()))
                    fresh = key not in seen
                    if fresh:
                        before = copy.deepcopy((args, kwargs))
                except Exception:  # noqa: BLE001
                    fresh = False
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
        wrapper.__name__ = getattr(fn, "__name__", "wrapper")
        wrapper.__qualname__ = getattr(fn, "__qualname__", "wrapper")
        wrapper.__module__ = fn.__module__
        return wrapper

    wrapped = {}
    for modname in mine:
        mod = importlib.import_module(modname)
        for attr, fn in list(vars(mod).items()):
            if inspect.isfunction(fn) and fn.__module__ == modname:
                w = wrap(f"{modname}:{attr}", fn, a.cap)
                wrapped[id(fn)] = w
                setattr(mod, attr, w)
    for modname, names in DEPS.items():
        mod = importlib.import_module(modname)
        for attr in names:
            fn = getattr(mod, attr)
            w = wrap(f"{modname}:{attr}", fn, 0, dep=True)
            wrapped[id(fn)] = w
            setattr(mod, attr, w)
    # Modules that imported a function by name hold the function itself: point them at the wrapper.
    for modname, mod in list(sys.modules.items()):
        if not modname.startswith("bot") or mod is None:
            continue
        for attr, value in list(vars(mod).items()):
            if inspect.isfunction(value) and id(value) in wrapped:
                setattr(mod, attr, wrapped[id(value)])

    done = 0
    if a.script:
        for conv in json.load(open(a.script)):
            memory = {}
            random.seed(0)
            for line in conv:
                try:
                    rt.turn(line, memory)
                except Exception:  # noqa: BLE001
                    pass
                done += 1
    for path in a.oracles:
        with gzip.open(path, "rt") as f:
            convs, last, memory = 0, None, None
            for line in lines_of(f):
                # only the conversation id and the line are needed; the rows are large
                row = json.loads(line)
                if row.get("conv") != last:
                    last = row.get("conv")
                    convs += 1
                    if a.max and convs > a.skip + a.max:
                        break
                    memory = {}
                    random.seed(0)
                if convs <= a.skip:
                    continue
                try:
                    rt.turn(row.get("text") or "", memory)
                except Exception:  # noqa: BLE001
                    pass
                done += 1
    out.close()
    print(f"{done} turns; {len(counts)} functions; {sum(counts.values())} calls", file=sys.stderr)
    for name in sorted(counts):
        print(f"{counts[name]:6d} {name}", file=sys.stderr)


if __name__ == "__main__":
    main()
