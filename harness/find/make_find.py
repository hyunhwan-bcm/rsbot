"""PyBot's lookup.find for each line's single-turn reading: the reading, find(u), find(u, use_context=True)."""
import sys, json, time
sys.path.insert(0, __import__("os").path.dirname(__import__("os").path.abspath(__file__)))
from pyenv import plainify, texts
from bot.understanding import understand
from bot.knowledge import lookup
src = sys.argv[1]
out = open(sys.argv[2], "w")
if src == "bench":
    lines = texts()
else:
    lines = [json.loads(l)["text"] for l in open(src)]
    if len(sys.argv) > 3:
        lines = lines[: int(sys.argv[3])]
t0 = time.time()
for i, text in enumerate(lines):
    u = understand(text, {})
    def fid(**kw):
        try:
            e = lookup.find(u, **kw)
            return e.id if e is not None else None
        except Exception as ex:
            return "!" + type(ex).__name__
    rec = {"text": text, "understood": plainify(u), "find": fid(), "ctx": fid(use_context=True)}
    out.write(json.dumps(rec) + "\n")
    if i % 200 == 0:
        print(i, round(time.time() - t0, 1), file=sys.stderr, flush=True)
out.close()
