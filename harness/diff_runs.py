"""Compare two run JSONLs by id: how many replies are byte-identical, and show some that are not."""
import json, sys
a = {json.loads(l)["id"]: json.loads(l) for l in open(sys.argv[1])}
b = {json.loads(l)["id"]: json.loads(l) for l in open(sys.argv[2])}
ids = [i for i in a if i in b]
diff = [i for i in ids if a[i]["reply"] != b[i]["reply"]]
print(f"{len(ids)-len(diff)}/{len(ids)} identical ({100*(len(ids)-len(diff))/max(1,len(ids)):.1f}%)")
for i in diff[:int(sys.argv[3]) if len(sys.argv) > 3 else 5]:
    print("--", i, a[i].get("module"), "->", b[i].get("module"))
    print("  A:", a[i]["reply"][:200]); print("  B:", b[i]["reply"][:200])
