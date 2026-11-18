"""Change the signature of still-unported stubs in src/memory/store_replies.rs (MEM1 workstream).

    python3 harness/mem1_sig.py 'NAME(args) -> Ret' ...

Each argument is the new `pub fn` signature text after `pub fn `; the stub body is kept.
"""
import re, sys

PATH = "src/memory/store_replies.rs"
src = open(PATH).read()
for sig in sys.argv[1:]:
    name = sig.split("(", 1)[0].strip()
    pat = re.compile(r"^pub fn " + re.escape(name) + r"\(.*\{$", re.M)
    m = pat.search(src)
    if not m:
        print("no stub line for", name)
        continue
    src = src[:m.start()] + "pub fn " + sig + " {" + src[m.end():]
open(PATH, "w").write(src)
