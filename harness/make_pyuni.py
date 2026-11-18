import sys
flags = {
 "alpha": str.isalpha, "digit": str.isdigit, "decimal": str.isdecimal, "numeric": str.isnumeric,
 "lower": str.islower, "upper": str.isupper, "title": str.istitle, "space": str.isspace,
}
out=[]
for name, f in flags.items():
    ranges=[]; start=None
    for cp in range(0x110001):
        ok = cp < 0x110000 and not (0xD800 <= cp <= 0xDFFF) and f(chr(cp))
        if name == "title":  # per-char titlecase property: Lt
            import unicodedata
            ok = cp < 0x110000 and not (0xD800 <= cp <= 0xDFFF) and unicodedata.category(chr(cp)) == "Lt"
        if name in ("lower","upper"):
            # per-char Py_UNICODE_ISLOWER/ISUPPER: a lone char's islower() is exactly that
            pass
        if ok and start is None: start=cp
        if not ok and start is not None: ranges.append((start,cp-1)); start=None
    out.append(name+" "+" ".join("%x-%x"%r for r in ranges))
open(sys.argv[1],"w").write("\n".join(out)+"\n")
