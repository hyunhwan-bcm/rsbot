"""MOD3: reference values of the small modules' pure readings.

    python3 harness/mod3_small_ref.py OUT.jsonl
Read by the ignored test `modules::arbiter::tests::small_module_readings_against_the_reference`
(SMALL_REF).
"""
import sys, os, json, gzip, re
HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(ROOT, 'harness'))
import run_python
run_python.pin_clock("2026-11-17")
sys.path.insert(0, os.path.abspath(run_python.PYBOT))
from bot import runtime  # noqa
from bot.modules import their_taste as T, a_limit_over_the_answer as L, a_return_brings_back_the_advice as R
lines, replies = set(), set()
for f in ('their_taste.py', 'a_limit_over_the_answer.py', 'a_return_brings_back_the_advice.py', 'done_ask.py', 'no_advice.py'):
    for l in open(os.path.join(run_python.PYBOT, 'bot/modules', f)):
        for m in re.finditer(r'"([a-zA-Z][^"]{3,140})"', l):
            lines.add(m.group(1))
with gzip.open(os.path.join(ROOT, 'data/export/turn_oracle/conversations.jsonl.gz'), 'rt') as fh:
    for i, line in enumerate(fh):
        if i >= 3000:
            break
        rec = json.loads(line)
        lines.add(rec['text'])
        replies.add(rec['reply'])
lines |= {"i love thai food", "i really like jazz", "im not a fan of football", "i cant stand mushrooms",
          "god i love thai food tbh", "i am a massive fan of soup", "i am not a fan of my job", "i love my new kitchen",
          "i hate my life", "i like to swim", "i love it here", "i love you", "i'm into climbing", "ive always loved the sea",
          "we rent the flat", "we have no car", "its a rescue", "we only have a balcony", "no garden", "i dont drive",
          "honestly we just rent tbh", "she is very nervous", "we dont own it"}
out = open(sys.argv[1], 'w')

def put(mod, name, args, res):
    out.write(json.dumps({"m": mod, "f": name, "args": args, "out": res}) + "\n")

for line in sorted(lines):
    low = " ".join(line.lower().split())
    plain = T._said_plainly(low)
    put("their_taste", "_said_plainly", [low], plain)
    taste = T._a_taste(plain)
    put("their_taste", "_a_taste", [plain], list(taste) if taste else None)
    put("their_taste", "_the_thing_said", [low], T._the_thing_said(low))
    put("a_limit_over_the_answer", "_clean", [line], L._clean(line))
    limit = L._a_limit(line)
    put("a_limit_over_the_answer", "_a_limit", [line], limit[0] if limit else None)
    put("a_limit_over_the_answer", "_the_move", [line], L._the_move(line))
    put("a_return_brings_back_the_advice", "_the_advice", [line], R._the_advice(line))
    put("a_return_brings_back_the_advice", "_has_substance", [line], R._has_substance(line))
for reply in sorted(replies):
    put("a_limit_over_the_answer", "_the_move", [reply], L._the_move(reply))
    advice = R._the_advice(reply)
    put("a_return_brings_back_the_advice", "_the_advice", [reply], advice)
    put("a_return_brings_back_the_advice", "_has_substance", [advice], R._has_substance(advice))
