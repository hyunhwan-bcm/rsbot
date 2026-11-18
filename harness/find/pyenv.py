"""Import upstream PyBot the way the harness does (clock pinned), for corpus dumps.

    import sys; sys.path.insert(0, SCRATCHPAD); from pyenv import bot_import, plainify
"""
import os, sys, json
H = "/Volumes/SSD/Workspace/rsbot/harness"
sys.path.insert(0, H)
import run_python
from export_data import encode
from plain import plain
run_python.pin_clock("2026-11-17")
sys.path.insert(0, os.path.abspath(run_python.PYBOT))

def plainify(v):
    """Python value -> plain JSON (tuples/sets as lists, dicts as objects)."""
    return plain(encode(v))

def texts():
    """Every benchmark text: gold, heldout, then the extra understand() call texts."""
    out, seen = [], set()
    base = "/Volumes/SSD/Workspace/rsbot/data/export"
    for f in ("oracle_gold.plain.jsonl", "oracle_heldout.plain.jsonl", "understand_calls.jsonl"):
        for line in open(os.path.join(base, f)):
            t = json.loads(line)["text"]
            if t not in seen:
                seen.add(t); out.append(t)
    return out
