"""Export PyBot's module-level data (tables, word lists, regexes, knowledge entries) as JSON.

    python harness/export_data.py OUT.json [--pybot DIR]

The Rust port hand-writes the logic and reads the data from this bundle, so nothing in the
knowledge base is retyped. Every module global that is plain data is exported under
"<module>.<NAME>" (e.g. "bot.knowledge.lookup.RING_AS_A_CALL"). Encoding:

    str, int, float, bool, None   as JSON
    list                          {"l": [...]}
    tuple                         {"t": [...]}
    set / frozenset               {"s": [...]}            (sorted when the items sort)
    dict                          {"d": [[key, value], ...]}  (insertion order kept)
    re.Pattern                    {"re": pattern, "flags": int}
    function / lambda             {"fn": "module:qualname:line"}
    Entry / other object          {"obj": ClassName, "attrs": {...}}
    module                        {"mod": name}

Values that are not reachable as plain data are recorded as {"opaque": type name}.
"""
import argparse
import json
import os
import re
import sys
import types

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python  # noqa: E402


def encode(v, depth=0, seen=None):
    if seen is None:
        seen = set()
    if v is None or isinstance(v, (bool, int, str)):
        return v
    if isinstance(v, float):
        if v != v or v in (float("inf"), float("-inf")):
            return {"float": repr(v)}
        return v
    if depth > 40:
        return {"opaque": "too-deep"}
    if isinstance(v, re.Pattern):
        return {"re": v.pattern, "flags": int(v.flags)}
    if isinstance(v, (types.FunctionType, types.BuiltinFunctionType, types.MethodType)):
        code = getattr(v, "__code__", None)
        out = {"fn": f"{getattr(v, '__module__', '?')}:{getattr(v, '__qualname__', '?')}:{code.co_firstlineno if code else 0}"}
        # A closure's captured values and a function's defaults are part of what it does: one
        # factory (e.g. `_both(a, b)`) makes many guards that differ only in these.
        if depth < 30 and getattr(v, "__closure__", None):
            cells = {}
            for name, cell in zip(code.co_freevars, v.__closure__):
                try:
                    cells[name] = encode(cell.cell_contents, depth + 1, seen | {id(v)})
                except ValueError:
                    cells[name] = {"opaque": "empty-cell"}
            out["cells"] = cells
        if depth < 30 and getattr(v, "__defaults__", None):
            out["defaults"] = [encode(x, depth + 1, seen | {id(v)}) for x in v.__defaults__]
        return out
    if isinstance(v, types.ModuleType):
        return {"mod": v.__name__}
    if id(v) in seen:
        return {"opaque": "cycle"}
    seen = seen | {id(v)}
    if isinstance(v, list):
        return {"l": [encode(x, depth + 1, seen) for x in v]}
    if isinstance(v, tuple):
        return {"t": [encode(x, depth + 1, seen) for x in v]}
    if isinstance(v, (set, frozenset)):
        items = list(v)
        try:
            items.sort()
        except TypeError:
            items.sort(key=repr)
        return {"s": [encode(x, depth + 1, seen) for x in items]}
    if isinstance(v, dict):
        return {"d": [[encode(k, depth + 1, seen), encode(x, depth + 1, seen)] for k, x in v.items()]}
    slots = getattr(type(v), "__slots__", None)
    attrs = {}
    if slots:
        for name in slots:
            if hasattr(v, name):
                attrs[name] = encode(getattr(v, name), depth + 1, seen)
    elif hasattr(v, "__dict__"):
        for name, x in vars(v).items():
            attrs[name] = encode(x, depth + 1, seen)
    else:
        return {"opaque": type(v).__name__}
    return {"obj": type(v).__name__, "attrs": attrs}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--pybot", default=run_python.PYBOT)
    ap.add_argument("--today", default="2026-11-17")
    a = ap.parse_args()
    run_python.pin_clock(a.today)
    sys.path.insert(0, os.path.abspath(a.pybot))
    from bot import runtime  # noqa: F401 - imports every module
    out = {}
    for name, mod in sorted(sys.modules.items()):
        if not (name == "bot" or name.startswith("bot.")) or mod is None:
            continue
        for key, value in vars(mod).items():
            if key.startswith("__") or isinstance(value, types.ModuleType):
                continue
            if isinstance(value, (types.FunctionType, type)):
                continue  # code, not data
            out[f"{name}.{key}"] = encode(value)
    # The canonical knowledge base, as the matcher sees it: `bot.knowledge.entries()` runs once,
    # sets each entry's domain and folds a domain's STANDS_ASIDE / GUARD_FOLDS tests into its
    # guards. Module tables above are exported before that call, as they are at import.
    import bot.knowledge as K
    out["bot.knowledge.__entries__"] = encode(list(K.entries()))
    json.dump(out, open(a.out, "w"), ensure_ascii=False, separators=(",", ":"))
    print(f"exported {len(out)} globals, {os.path.getsize(a.out) / 1e6:.1f} MB", file=sys.stderr)


if __name__ == "__main__":
    main()
