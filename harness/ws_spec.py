"""Port spec for one workstream: the sliced functions its entry points reach.

    python harness/ws_spec.py REACH.json SLICED_SPEC_DIR OUT_DIR --entries observe:understand,observe:intent
           [--exclude-entries observe:understand,...]

A top-level function (or class) is in the spec when any code object inside it was reached by one
of --entries, and (with --exclude-entries) none by those. Writes OUT_DIR/<file>.py with only those
functions (from the sliced spec) and OUT_DIR/index.json {file: [functions]}.
"""
import argparse, ast, json, os, collections

ap = argparse.ArgumentParser()
ap.add_argument("reach"); ap.add_argument("spec"); ap.add_argument("out")
ap.add_argument("--entries", required=True); ap.add_argument("--exclude-entries", default="")
ap.add_argument("--only-files", default="", help="comma list of path prefixes to keep")
a = ap.parse_args()
want = set(a.entries.split(",")); excl = set(filter(None, a.exclude_entries.split(",")))
prefixes = tuple(filter(None, a.only_files.split(",")))
reach = json.load(open(a.reach))["funcs"]
tops = collections.defaultdict(set)
for k, entries in reach.items():
    file, qual, line = k.rsplit(":", 2)
    es = set(entries)
    if not (es & want) or (es & excl):
        continue
    if prefixes and not file.startswith(prefixes):
        continue
    top = qual.split(".")[0]
    tops[file].add(top)
index = {}; total = 0
for file, names in sorted(tops.items()):
    src = os.path.join(a.spec, file)
    if not os.path.exists(src):
        continue
    tree = ast.parse(open(src).read())
    data_line = open(src).readline().strip()
    keep = [s for s in tree.body if isinstance(s, (ast.FunctionDef, ast.ClassDef)) and s.name in names]
    imports = [s for s in tree.body if isinstance(s, (ast.Import, ast.ImportFrom))]
    if not keep:
        continue
    text = data_line + "\n" + ast.unparse(ast.Module(body=imports + keep, type_ignores=[])) + "\n"
    dest = os.path.join(a.out, file); os.makedirs(os.path.dirname(dest), exist_ok=True)
    open(dest, "w").write(text)
    index[file] = [s.name for s in keep]; total += text.count("\n")
json.dump(index, open(os.path.join(a.out, "index.json"), "w"), indent=1)
print(f"{len(index)} files, {sum(map(len, index.values()))} functions, {total} lines")
