"""lookup.find on the readings the turn oracle recorded (the understand observer's memory diff):
plain and use_context, on a sample of turns (every turn whose reading leans on an earlier line,
then others), for the port's find check."""
import sys, json, gzip, time, random as R
sys.path.insert(0, __import__("os").path.dirname(__import__("os").path.abspath(__file__)))
import pyenv
from bot.knowledge import lookup
src, dst, limit = sys.argv[1], sys.argv[2], int(sys.argv[3])
recs = []
for line in gzip.open(src, "rt"):
    rec = json.loads(line)
    u = None
    for o in rec["observe"]:
        if o["module"] == "understand":
            u = o["diff"]["set"].get("understood")
    if isinstance(u, dict):
        recs.append((rec["conv"], rec["turn"], rec["text"], u))
lean = [r for r in recs if r[3].get("leans_on")]
rest = [r for r in recs if not r[3].get("leans_on")]
R.seed(1)
R.shuffle(rest)
pick = (lean + rest)[:limit]
print(len(recs), "readings,", len(lean), "lean on a line", file=sys.stderr, flush=True)
out = open(dst, "w")
t0 = time.time()
for i, (conv, turn, text, u) in enumerate(pick):
    def fid(**kw):
        try:
            e = lookup.find(json.loads(json.dumps(u)), **kw)
            return e.id if e is not None else None
        except Exception as ex:
            return "!" + type(ex).__name__
    out.write(json.dumps({"text": text, "conv": conv, "turn": turn, "understood": u, "find": fid(), "ctx": fid(use_context=True)}) + "\n")
    out.flush()
    if i % 200 == 0:
        print(i, round(time.time() - t0, 1), file=sys.stderr, flush=True)
