"""Every regex in the export, run with finditer over a corpus: the Python side of the regex check.

    python harness/regex_oracle.py BUNDLE.json CORPUS.txt OUT.jsonl

Writes one line per distinct (pattern, flags): {"i": n, "p": pattern, "f": flags, "m": [[row, start, end], ...]}
with start/end in code points, for every non-overlapping match in every corpus line.
"""
import json, re, sys, multiprocessing as mp

def walk(v, out):
    if isinstance(v, dict):
        if "re" in v and "flags" in v and len(v) == 2:
            out.add((v["re"], v["flags"]))
            return
        for x in v.values(): walk(x, out)
    elif isinstance(v, list):
        for x in v: walk(x, out)

def run(args):
    i, p, f = args
    try:
        rx = re.compile(p, f)
    except re.error as e:
        return {"i": i, "p": p, "f": f, "err": str(e)}
    m = []
    for row, line in enumerate(CORPUS):
        for mt in rx.finditer(line):
            m.append([row, mt.start(), mt.end()])
    return {"i": i, "p": p, "f": f, "m": m}

if __name__ == "__main__":
    bundle, corpus, out = sys.argv[1:4]
    pats = set(); walk(json.load(open(bundle)), pats)
    pats = sorted(pats)
    CORPUS = [json.loads(l) for l in open(corpus)]
    with mp.get_context("fork").Pool(11) as pool:
        res = pool.map(run, [(i, p, f) for i, (p, f) in enumerate(pats)], chunksize=50)
    with open(out, "w") as fh:
        for r in res: fh.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(len(pats), "patterns,", sum("err" in r for r in res), "errors,", sum(len(r.get("m", [])) for r in res), "matches", file=sys.stderr)
