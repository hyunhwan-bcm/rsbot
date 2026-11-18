"""Knowledge entries' guards and replies on the readings and memory of real conversation turns.

    python3 harness/kno/make_entry_turns.py OUT.jsonl DOMAINS [--cap N] [--per-turn K] [ORACLE.jsonl.gz ...]

DOMAINS is a comma list of knowledge domains (file names), or "all". For every turn of the turn
oracles (default: conversations, synthetic, extra), the memory the respond stage sees is rebuilt
(mem_in plus every observer's diff: the reading, the earlier turns and everything else), and each
entry of those domains whose key's first part names a word of the line has its guard and its
reply called on it - the guard with the reading, the reply with a copy of that memory and the
reading - with random.seed(0) before each call. Entries with a non-function guard or reply skip
that half. At most --cap calls are kept per entry and kind, and --per-turn entries per turn.

Writes one line per turn used: {"h": key, "memory": plain memory, "calls": [{"e": entry id,
"k": "guard" | "reply", "r": result | "!ExceptionName"}]}; `kno_check entries` replays them.
"""
import sys, os, json, gzip, copy, random, argparse, collections
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "find"))
import pyenv  # noqa: F401
from pyenv import plainify
from bot import knowledge

ap = argparse.ArgumentParser()
ap.add_argument("out"); ap.add_argument("domains"); ap.add_argument("oracles", nargs="*")
ap.add_argument("--cap", type=int, default=400); ap.add_argument("--per-turn", type=int, default=40)
a = ap.parse_args()
base = "/Volumes/SSD/Workspace/rsbot/data/export/turn_oracle/"
oracles = a.oracles or [base + f for f in ("conversations.jsonl.gz", "synthetic.jsonl.gz", "extra.jsonl.gz")]
wanted = None if a.domains == "all" else set(a.domains.split(","))
rows = [e for e in knowledge.entries() if wanted is None or e.domain in wanted]
print(len(rows), "entries", file=sys.stderr)


def first_words(e):
    part = e.key[0] if e.key else ()
    words = part if isinstance(part, (tuple, list)) else (part,)
    return [str(w).lower() for w in words if w]


keyed = [(e, first_words(e)) for e in rows]
kept = collections.Counter()
out = open(a.out, "w")
n = 0
for path in oracles:
    for line in gzip.open(path, "rt"):
        rec = json.loads(line)
        memory = dict(rec["mem_in"])
        for o in rec["observe"]:
            memory.update(o["diff"]["set"])
            for k in o["diff"]["del"]:
                memory.pop(k, None)
        reading = memory.get("understood")
        if not isinstance(reading, dict):
            continue
        said = " %s %s " % (str(reading.get("clean") or "").lower(), " ".join(str(w) for w in reading.get("words") or []))
        calls = []
        for e, words in keyed:
            if len(calls) >= 2 * a.per_turn:
                break
            if not any(w in said for w in words):
                continue
            for kind in ("guard", "reply"):
                f = getattr(e, kind)
                if not callable(f) or kept[(e.id, kind)] >= a.cap:
                    continue
                kept[(e.id, kind)] += 1
                random.seed(0)
                try:
                    if kind == "guard":
                        r = bool(f(copy.deepcopy(reading)))
                    else:
                        r = f(copy.deepcopy(memory), copy.deepcopy(reading))
                        r = r if isinstance(r, str) else None
                except Exception as ex:  # noqa: BLE001
                    r = "!" + type(ex).__name__
                calls.append({"e": e.id, "k": kind, "r": r})
        if calls:
            out.write(json.dumps({"h": "%s/%s" % (rec["conv"], rec["turn"]), "memory": memory, "calls": calls}, ensure_ascii=False) + "\n")
            n += 1
print(n, "turns,", sum(kept.values()), "calls", file=sys.stderr)
