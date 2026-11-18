"""Record the calls PyBot makes to the MEM7 modules (35 files of bot/memory/, see MINE), for a
function-level check of the Rust port (src/bin/mem7_check.rs).

    python3 harness/mem7_record.py OUT.jsonl.gz ORACLE.jsonl.gz [...] [--max N] [--cap 300]
        [--only taste,healing] [--lines]

Whole conversations of the oracle files are replayed through the runtime (random.seed(0) per
conversation, the clock pinned to 2026-11-17, as the oracles were recorded), and every distinct
call of a function of the chosen modules is one line:

    {"fn": "module:qualname", "args": [...], "kwargs": {...}, "result": ..., "raised": null |
     "ExcName", "after": [...] (the args after the call, when the call changed them),
     "rng": [...] (the global generator's state before the call, when the call drew from it)}

With --lines every user line is also handed to the line readers in LINE_PROBES, which reaches
readers no module happens to call on that line. A Match argument is {"__match__": {...}}, a
compiled pattern {"__re__": [pattern, flags]}, a function {"__fn__": "module:qualname:line"}.
Positions are Python's (code points): the checker converts them.
"""
import argparse, copy, gzip, inspect, json, os, random, re, sys, types

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "harness"))
import run_python  # noqa: E402

MINE = ("aboutthem amounts another asleep back_again carried_on change company condition earlier "
        "favourite guest healing household lineback matters meant measure obstacle outfit people "
        "places plan plans portions rates reject resize returns stands surname taking taste times "
        "years_in").split()

# Functions of other workstreams that the MEM7 files call and that are not ported yet. A call made
# while a MEM7 function is running is recorded as a "dep" row with a lookup key, so the test-only
# mock crate (harness/mem7_mockpatch.py) can answer it from the recording. The key leaves out a
# memory argument (pick's), which the Rust side cannot rebuild byte for byte.
DEPS = {
    "bot.memory.kinds": ("the_line_that_gives",),
    "bot.memory.choice": ("options_in",),
    "bot.memory.quotes": ("sentences",),
    "bot.memory.detail": ("_a_step", "_first_clause"),
    "bot.compose": ("pick",),
}


def dep_key(name, args, kwargs):
    """The lookup key of a dep call, in the compact JSON serde_json writes."""
    if name == "bot.compose:pick":
        options = [enc(option) for option in args[0]]
        seed = args[2] if len(args) > 2 else kwargs.get("seed_text", "")
        key = [options, seed]
    elif name == "bot.memory.quotes:sentences":
        key = [args[0], args[1].pattern, int(args[1].flags)]
    elif name == "bot.memory.detail:_a_step":
        flag = args[1] if len(args) > 1 else kwargs.get("questions_are_steps", False)
        key = [args[0], bool(flag)]
    else:
        key = enc(list(args))
    return json.dumps([name, key], ensure_ascii=False, separators=(",", ":"))


# Readers of one line, called on every user line with --lines: {module: [function names]}.
LINE_PROBES = {
    "taste": ["read", "read_all", "read_other", "only_a_taste", "asks_about_a_taste"],
}


def lines_of(f):
    """The lines of a gzip file, stopping quietly where a file still being written ends."""
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
    if isinstance(v, (types.FunctionType, types.BuiltinFunctionType, types.MethodType)):
        v = getattr(v, "__wrapped__", v)
        code = getattr(v, "__code__", None)
        return {"__fn__": "%s:%s:%s" % (getattr(v, "__module__", "?"), getattr(v, "__qualname__", "?"),
                                        code.co_firstlineno if code else 0)}
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
    ap.add_argument("oracles", nargs="*")
    ap.add_argument("--max", type=int, default=0, help="at most this many conversations per file")
    ap.add_argument("--skip", type=int, default=0, help="skip this many conversations per file")
    ap.add_argument("--cap", type=int, default=300)
    ap.add_argument("--only", default="", help="comma list of the modules to record")
    ap.add_argument("--lines", action="store_true")
    ap.add_argument("--no-turns", action="store_true")
    ap.add_argument("--text", help="a plain text file: each line is handed to the line readers")
    ap.add_argument("--script", help="a JSON file {name: [conversation, ...]}: each conversation (a "
                    "list of lines) is played through the runtime")
    ap.add_argument("--group", default="", help="comma list of the script groups to play")
    a = ap.parse_args()

    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    import importlib
    from bot import runtime as rt
    chosen = [m for m in MINE if not a.only or m in a.only.split(",")]
    seen, counts = set(), {}
    out = gzip.open(a.out, "wt")

    inside = [0]

    def wrap_dep(name, fn):
        def wrapper(*args, **kwargs):
            result = fn(*args, **kwargs)
            if inside[0] > 0:
                try:
                    key = dep_key(name, args, kwargs)
                except Exception:  # noqa: BLE001
                    key = None
                if key is not None and key not in seen_deps:
                    seen_deps.add(key)
                    out.write(json.dumps({"fn": name, "dep": True, "key": key, "result": enc(result)},
                                         ensure_ascii=False) + "\n")
            return result
        wrapper.__wrapped__ = fn
        return wrapper

    seen_deps = set()

    def wrap(name, fn, cap):
        def wrapper(*args, **kwargs):
            key = None
            if counts.get(name, 0) < cap:
                try:
                    key = hash(name + repr(args) + repr(sorted(kwargs.items())))
                except Exception:  # noqa: BLE001
                    key = None
            fresh = key is not None and key not in seen
            before = state = None
            if fresh:
                try:
                    before = enc(copy.deepcopy((args, kwargs)))
                except Exception:  # noqa: BLE001
                    try:
                        before = enc((args, kwargs))
                    except Exception:  # noqa: BLE001
                        fresh = False
                state = random.getstate()
            row = None
            inside[0] += 1
            try:
                result = fn(*args, **kwargs)
                if fresh:
                    row = {"fn": name, "args": before[0], "kwargs": before[1], "result": enc(result),
                           "raised": None}
            except Exception as e:  # noqa: BLE001
                if fresh:
                    row = {"fn": name, "args": before[0], "kwargs": before[1], "result": None,
                           "raised": type(e).__name__}
                raise
            finally:
                inside[0] -= 1
                if row is not None:
                    seen.add(key)
                    counts[name] = counts.get(name, 0) + 1
                    after = enc(args)
                    if after != row["args"]:
                        row["after"] = after
                    if random.getstate() != state:
                        row["rng"] = list(state[1])
                    out.write(json.dumps(row, ensure_ascii=False) + "\n")
            return result
        wrapper.__wrapped__ = fn
        return wrapper

    wrapped = {}
    for short in chosen:
        modname = "bot.memory." + short
        mod = importlib.import_module(modname)
        for attr, fn in list(vars(mod).items()):
            if inspect.isfunction(fn) and fn.__module__ == modname:
                w = wrap(f"{modname}:{attr}", fn, a.cap)
                setattr(mod, attr, w)
                wrapped[id(fn)] = w
    for modname, names in DEPS.items():
        mod = importlib.import_module(modname)
        for attr in names:
            fn = getattr(mod, attr)
            w = wrap_dep(f"{modname}:{attr}", fn)
            setattr(mod, attr, w)
            wrapped[id(fn)] = w
    # Modules that imported a function by name hold the function itself: point them at the
    # wrapper too, so their calls are recorded.
    for modname, mod in list(sys.modules.items()):
        if not modname.startswith("bot") or mod is None:
            continue
        for attr, value in list(vars(mod).items()):
            if isinstance(value, types.FunctionType) and id(value) in wrapped:
                setattr(mod, attr, wrapped[id(value)])

    probes = []
    if a.lines:
        for short, names in LINE_PROBES.items():
            if short in chosen:
                mod = sys.modules["bot.memory." + short]
                probes += [getattr(mod, n) for n in names]
    done = 0
    if a.text:
        taste = sys.modules["bot.memory.taste"]
        for text in open(a.text):
            text = text.rstrip("\n")
            for probe in probes:
                try:
                    probe(text)
                except Exception:  # noqa: BLE001
                    pass
            if "taste" in chosen:
                for found in re.finditer(r"\w+", text):
                    taste.said_about(text, found.start())
            done += 1
    if a.script:
        groups = json.load(open(a.script))
        for group, convs in groups.items():
            if a.group and group not in a.group.split(","):
                continue
            for conv in convs:
                memory = {}
                random.seed(0)
                for text in conv:
                    for probe in probes:
                        try:
                            probe(text)
                        except Exception:  # noqa: BLE001
                            pass
                    try:
                        rt.turn(text, memory)
                    except Exception:  # noqa: BLE001
                        pass
                    done += 1
    for path in a.oracles:
        with gzip.open(path, "rt") as f:
            convs, last, memory = 0, None, None
            for line in lines_of(f):
                row = json.loads(line)
                if row.get("conv") != last:
                    last = row.get("conv")
                    convs += 1
                    if a.max and convs > a.max + a.skip:
                        break
                    memory = {}
                    random.seed(0)
                if convs <= a.skip:
                    continue
                text = row.get("text") or ""
                for probe in probes:
                    try:
                        probe(text)
                    except Exception:  # noqa: BLE001
                        pass
                if not a.no_turns:
                    try:
                        rt.turn(text, memory)
                    except Exception:  # noqa: BLE001
                        pass
                done += 1
    out.close()
    print(f"{done} lines; {len(counts)} functions; {sum(counts.values())} calls", file=sys.stderr)


if __name__ == "__main__":
    main()
