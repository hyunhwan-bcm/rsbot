"""Tagged export encoding (export_data.encode) -> plain JSON the Rust structs deserialize."""
import json, sys

def plain(v):
    if isinstance(v, list):
        return [plain(x) for x in v]
    if not isinstance(v, dict):
        return v
    k = next(iter(v)) if v else None
    if k in ("l", "t", "s"):
        return [plain(x) for x in v[k]]
    if k == "d":
        pairs = v[k]
        if all(isinstance(p[0], str) for p in pairs):
            return {p[0]: plain(p[1]) for p in pairs}
        return [[plain(p[0]), plain(p[1])] for p in pairs]
    return v

if __name__ == "__main__":
    src, dst = sys.argv[1:3]
    with open(dst, "w") as out:
        for line in open(src):
            r = json.loads(line)
            r["understood"] = plain(r["understood"])
            out.write(json.dumps(r, ensure_ascii=False) + "\n")
