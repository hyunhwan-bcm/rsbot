"""Call any PyBot function on JSON arguments and print the result as JSON (for porting checks).

    python3 harness/pycall.py bot.memory.facts extract '["i have a dog called rex"]' ['{"memory": {...}}']
    python3 harness/pycall.py bot.understanding understand '["hi there"]' --plain

Arguments are a JSON list (positional) and optionally a JSON object (keyword). A JSON object
whose keys look like a reading (text, clean, words) is passed as a dict, as PyBot does. The
clock is pinned to 2026-11-17 and random.seed(0) runs first, as in every oracle run.
"""
import json, os, random, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python
from export_data import encode
from plain import plain

run_python.pin_clock("2026-11-17")
sys.path.insert(0, os.path.abspath(run_python.PYBOT))
import importlib
from bot import runtime  # noqa: F401 - import everything, as at run time

mod, fn = sys.argv[1], sys.argv[2]
args = json.loads(sys.argv[3]) if len(sys.argv) > 3 and not sys.argv[3].startswith("--") else []
kwargs = json.loads(sys.argv[4]) if len(sys.argv) > 4 and not sys.argv[4].startswith("--") else {}
random.seed(0)
f = getattr(importlib.import_module(mod), fn)
try:
    res = f(*args, **kwargs)
    out = {"result": plain(encode(res)), "type": type(res).__name__}
except Exception as e:  # noqa: BLE001
    out = {"raised": type(e).__name__, "message": str(e)}
print(json.dumps(out, ensure_ascii=False))
