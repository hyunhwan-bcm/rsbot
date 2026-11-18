"""MOD3: rule-level reference for safety: the raw respond (no again.replied) with pick -> options[0].

    python3 harness/mod3_safety_ref.py INPUT OUT.jsonl [LIMIT]
INPUT is a turn oracle (.jsonl.gz) or a JSON list of lines (.json, empty memory each). Read by the
ignored test `modules::safety::tests::safety_against_the_reference` (SAFETY_REF, SAFETY_INPUT).
`--lines OUT.json` instead writes the example lines quoted in PyBot's safety sources.
"""
import sys, os, json, gzip, copy, re
HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(ROOT, 'harness'))
import run_python
run_python.pin_clock("2026-11-17")
sys.path.insert(0, os.path.abspath(run_python.PYBOT))
if sys.argv[1] == '--lines':
    lines = set()
    for f in ['safety.py', 'safety_cards.py', 'safety_tables.py', 'safety_phrases.py', 'arbiter.py']:
        for l in open(os.path.join(run_python.PYBOT, 'bot/modules', f)):
            for m in re.finditer(r'"([a-zA-Z][^"]{4,140})"', l.strip()):
                t = m.group(1)
                if '\\' not in t and '{' not in t and '(?' not in t:
                    lines.add(t)
    json.dump(sorted(lines), open(sys.argv[2], 'w'))
    sys.exit(0)
from bot import runtime  # noqa
from bot.modules import safety
raw = [c.cell_contents for c in safety.respond.__closure__ if callable(c.cell_contents)][0]
safety.pick = lambda options, memory, seed_text="": options[0]
path, out_path = sys.argv[1], sys.argv[2]
limit = int(sys.argv[3]) if len(sys.argv) > 3 else 10**9
out = open(out_path, 'w')

def one(idx, text, m):
    try:
        reply, err = raw(text, m), None
    except Exception as e:  # noqa
        reply, err = None, type(e).__name__
    out.write(json.dumps({'idx': idx, 'reply': reply, 'err': err, 'case': m.get('safety_case'),
                          'safe': m.get('safety_safe'), 'person': m.get('safety_person')}) + '\n')

if path.endswith('.json'):
    for idx, text in enumerate(json.load(open(path))):
        one(idx, text, {})
    sys.exit(0)
with gzip.open(path, 'rt') as f:
    for idx, line in enumerate(f):
        if idx >= limit:
            break
        rec = json.loads(line)
        mem = dict(rec['mem_in'])
        for o in rec['observe']:
            mem.update(o['diff']['set'])
            for k in o['diff']['del']:
                mem.pop(k, None)
        one(idx, rec['text'], copy.deepcopy(mem))
