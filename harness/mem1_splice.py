"""Splice a chunk of ported Rust functions into src/memory/store_replies.rs (MEM1 workstream).

    python3 harness/mem1_splice.py <chunk.rs>

Every `fn NAME` in the chunk whose stub (`unported!("bot.memory.store_replies:NAME")`) is in the
file has its stub deleted; the whole chunk goes in at the place of the first stub it replaces,
so the file keeps PyBot's order. A chunk that replaces no stub is appended at the end.
"""
import re, sys

PATH = "src/memory/store_replies.rs"
chunk = open(sys.argv[1]).read().rstrip() + "\n"
src = open(PATH).read()
names = re.findall(r"^(?:pub )?fn (\w+)", chunk, re.M)
first = None
removed = []
for name in names:
    stub = re.compile(r"(?:///[^\n]*\n)*pub fn " + re.escape(name) + r"\b[^\n]*\{\n\s*crate::unported!\(\"bot\.memory\.store_replies:"
                      + re.escape(name) + r"\"\)\n\}\n\n?")
    m = stub.search(src)
    if not m:
        continue
    pos = m.start()
    src = src[:pos] + "\x00" * 0 + src[m.end():]
    if first is None or pos < first:
        first = pos
    else:
        pass
    removed.append(name)
# recompute: insert at `first` (positions before it are unchanged since we only cut after or at it)
if first is None:
    src = src.rstrip() + "\n\n" + chunk
else:
    src = src[:first] + chunk + "\n" + src[first:]
open(PATH, "w").write(src)
print(f"replaced {len(removed)} stubs: {' '.join(removed)}")
missing = [n for n in names if n not in removed]
if missing:
    print("new (no stub):", " ".join(missing))
