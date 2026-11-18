"""Record the calls PyBot makes to the MEM6 modules while it replays oracle conversations, for a
function-level check of the Rust port (`src/bin/mem6_check.rs`).

    python3 harness/mem6_record.py OUT.jsonl.gz ORACLE.jsonl.gz [...] [--skip K] [--max N]
        [--some 40] [--none 10] [--probes 4000] [--only module[,module]]

Each recorded call is one line: {"fn": "bot.memory.x:name", "args": {param: value, ...} (every
parameter by name, defaults filled in), "result": ..., "raised": null | "ExcName", "rng": true
when the call moved the global generator, "after": {param: value} for the arguments the call
changed}. Values are the oracle's plain JSON (`plain(encode(v))`). Per function at most `--some`
calls with a truthy result and `--none` with a falsy one are kept, and after `--probes` calls of
a function only one call in 10 is looked at, then one in 100.
"""
import argparse, gzip, inspect, json, os, random, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "harness"))
import run_python  # noqa: E402
from export_data import encode  # noqa: E402
from plain import plain  # noqa: E402

MINE = ("bot.memory", "bot.memory.already", "bot.memory.carry", "bot.memory.cause",
        "bot.memory.choice", "bot.memory.counts", "bot.memory.decided", "bot.memory.history",
        "bot.memory.limit", "bot.memory.outgoings", "bot.memory.price", "bot.memory.repair",
        "bot.memory.spent", "bot.memory.swap", "bot.memory.told")


def lines_of(f):
    try:
        for line in f:
            if line.endswith("\n"):
                yield line
    except EOFError:
        return


def snap(v):
    return plain(encode(v))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("oracles", nargs="+")
    ap.add_argument("--skip", type=int, default=0)
    ap.add_argument("--max", type=int, default=0)
    ap.add_argument("--some", type=int, default=40)
    ap.add_argument("--none", type=int, default=10)
    ap.add_argument("--probes", type=int, default=150)
    ap.add_argument("--only", default="")
    a = ap.parse_args()

    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    import importlib
    from bot import runtime as rt
    out = gzip.open(a.out, "wt")
    some, none, probes, seen = {}, {}, {}, set()
    mine = tuple(m for m in MINE if not a.only or m.rsplit(".", 1)[-1] in a.only.split(",") or m in a.only.split(","))

    def wrap(name, fn):
        sig = inspect.signature(fn)

        def wrapper(*args, **kwargs):
            if some.get(name, 0) >= a.some and none.get(name, 0) >= a.none:
                return fn(*args, **kwargs)
            # Every call at first, then one in 10, then one in 100: the calls kept are spread
            # over the corpus without a snapshot of the arguments for each of them.
            n = probes[name] = probes.get(name, 0) + 1
            if (n > a.probes and n % 10) or (n > 10 * a.probes and n % 100):
                return fn(*args, **kwargs)
            try:
                bound = sig.bind(*args, **kwargs)
                bound.apply_defaults()
                before = json.dumps({k: snap(v) for k, v in bound.arguments.items()}, ensure_ascii=False)
            except Exception:  # noqa: BLE001
                return fn(*args, **kwargs)
            state = random.getstate()
            raised, result = None, None
            try:
                result = fn(*args, **kwargs)
            except Exception as e:  # noqa: BLE001
                raised = type(e).__name__
                raise
            finally:
                kind = none if (raised or not result) else some
                cap = a.none if kind is none else a.some
                key = (name, before)
                if kind.get(name, 0) < cap and key not in seen:
                    seen.add(key)
                    kind[name] = kind.get(name, 0) + 1
                    row = {"fn": name, "args": json.loads(before), "result": snap(result), "raised": raised}
                    if random.getstate() != state:
                        row["rng"] = True
                    try:
                        after = {k: snap(v) for k, v in bound.arguments.items()}
                        changed = {k: v for k, v in after.items() if v != row["args"][k]}
                        if changed:
                            row["after"] = changed
                    except Exception:  # noqa: BLE001
                        pass
                    out.write(json.dumps(row, ensure_ascii=False) + "\n")
            return result
        wrapper.__wrapped__ = fn
        return wrapper

    wrapped = {}
    for modname in mine:
        mod = importlib.import_module(modname)
        for attr, fn in list(vars(mod).items()):
            if inspect.isfunction(fn) and fn.__module__ == modname:
                w = wrap(f"{modname}:{attr}", fn)
                wrapped[id(fn)] = w
                setattr(mod, attr, w)
    # Modules that imported a function by name hold the function itself.
    for modname, mod in list(sys.modules.items()):
        if not modname.startswith("bot") or mod is None:
            continue
        for attr, value in list(vars(mod).items()):
            if inspect.isfunction(value) and id(value) in wrapped:
                setattr(mod, attr, wrapped[id(value)])

    done = 0
    for path in a.oracles:
        with gzip.open(path, "rt") as f:
            convs, last, memory = 0, None, None
            for line in lines_of(f):
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
    names = set(some) | set(none)
    print(f"{done} turns; {len(names)} functions; {sum(some.values())} truthy and {sum(none.values())} falsy calls",
          file=sys.stderr)


if __name__ == "__main__":
    main()
