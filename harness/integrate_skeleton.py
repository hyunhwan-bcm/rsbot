"""Install the generated skeleton into the main crate without touching ported code.

    python3 harness/integrate_skeleton.py [--types runs/trace_types/types.json]

- A stub file is written only where no Rust file exists yet.
- A file that exists (ported, partly ported or hand-written) gets the stubs of the Python
  functions it does not define yet, appended at the end; its own code is left alone, and its
  own signatures are the contract for those functions.
- A package `mod.rs` gets the `pub mod` lines it is missing.
- `src/modules/mod.rs` is regenerated: the registry of all 109 modules is generated code.
"""
import argparse, os, re, subprocess, sys, tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import workspace  # noqa: E402

PKGS = ("understanding", "knowledge", "compose", "memory", "modules")


def stub_blocks(text):
    """(name, block) for every stub function in a generated file."""
    out = []
    for m in re.finditer(r"(/// Python:[^\n]*\n(?:///[^\n]*\n)*pub fn (r#)?(\w+)[^\n]*\{\n    crate::unported!\([^\n]*\)\n\}\n)", text):
        out.append((m.group(3), m.group(1)))
    return out


def names_defined(text, here, src_root, depth=0, own_dir=None):
    """Function names a Rust file defines or re-exports, following `pub use ...::*` globs.

    `here` is the file's directory (what `super::` resolves against for a non-mod file);
    `own_dir` is where its child modules live (`x/` for `x.rs`, the directory for `mod.rs`)."""
    own_dir = own_dir or here
    names = set(re.findall(r"\bfn (?:r#)?(\w+)\s*[<(]", text))
    names |= set(re.findall(r"pub use [^;{]*?\b(\w+);", text))
    for grp in re.findall(r"pub use [^;]*\{([^}]*)\};", text):
        names |= {n.strip().split(" as ")[-1].strip() for n in grp.split(",") if n.strip()}
    if depth < 3:
        for path in re.findall(r"pub use ((?:\w+)(?:::\w+)*)::\*;", text):
            parts = path.split("::")
            if parts[0] == "crate":
                base = os.path.join(src_root, *parts[1:])
            elif parts[0] == "super":
                base = os.path.join(here, *parts[1:])
            elif parts[0] == "self":
                base = os.path.join(own_dir, *parts[1:])
            else:  # a child module: `mod find; pub use find::*;`
                base = os.path.join(own_dir, *parts)
            for target in (base + ".rs", os.path.join(base, "mod.rs")):
                if os.path.exists(target):
                    is_mod = target.endswith("mod.rs")
                    d = os.path.dirname(target)
                    names |= names_defined(open(target).read(), os.path.dirname(d) if is_mod else d, src_root,
                                           depth + 1, own_dir=d if is_mod else base)
                    break
    return names


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--types", default=os.path.join(workspace.root(), "runs", "trace_types", "types.json"))
    ap.add_argument("--crate", default=os.path.join(workspace.root(), "rsbot"))
    a = ap.parse_args()
    root = workspace.root()
    crate = a.crate
    out = tempfile.mkdtemp()
    subprocess.run([sys.executable, os.path.join(os.path.dirname(__file__), "gen_skeleton.py"), out, "--types", a.types,
                    "--contracts", os.path.join(root, "slice", "contracts.json")], check=True)
    created = appended = 0
    for pkg in PKGS:
        for dirpath, _, files in os.walk(os.path.join(out, pkg)):
            for f in files:
                src = os.path.join(dirpath, f)
                rel = os.path.relpath(src, out)
                dst = os.path.join(crate, "src", rel)
                if not os.path.exists(dst) and f != "mod.rs" and os.path.exists(os.path.join(dst[:-3], "mod.rs")):
                    dst = os.path.join(dst[:-3], "mod.rs")  # ported as a directory module
                gen = open(src).read()
                if rel == os.path.join("modules", "mod.rs"):
                    open(dst, "w").write(gen)
                    continue
                if not os.path.exists(dst):
                    os.makedirs(os.path.dirname(dst), exist_ok=True)
                    open(dst, "w").write(gen)
                    created += 1
                    continue
                cur = open(dst).read()
                add = []
                if f == "mod.rs":
                    for m in re.findall(r"^pub mod [\w#]+;$", gen, flags=re.M):
                        if m not in cur:
                            add.append(m)
                is_mod = dst.endswith("mod.rs")
                d = os.path.dirname(dst)
                defined = names_defined(cur, os.path.dirname(d) if is_mod else d, os.path.join(crate, "src"),
                                        own_dir=d if is_mod else dst[:-3])
                missing = [blk for name, blk in stub_blocks(gen) if name not in defined]
                if add or missing:
                    extra = "\n".join(add)
                    if missing:
                        body = "\n".join("    " + l if l else l for blk in missing for l in blk.split("\n"))
                        extra += ("\n\n// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not\n"
                                  "// define yet, with signatures from the generated contract. Port them in place (move a\n"
                                  "// function up into the file when you port it).\n"
                                  "pub use self::skeleton_stubs::*;\n"
                                  "#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]\n"
                                  "mod skeleton_stubs {\n"
                                  "    use super::*;\n"
                                  "    use crate::data::{self, Value};\n    use crate::knowledge::entry::Entry;\n"
                                  "    use crate::runtime::Candidate;\n    use crate::state::{Memory, Turn};\n"
                                  "    use crate::understood::Understood;\n\n" + body + "\n}\n")
                        appended += len(missing)
                    if add and f == "mod.rs":
                        # module declarations go near the top
                        lines = cur.split("\n")
                        at = next((i for i, l in enumerate(lines) if l.startswith("pub mod ")), len([l for l in lines if l.startswith("//")]))
                        lines[at:at] = add
                        cur = "\n".join(lines)
                        extra = extra.replace("\n".join(add), "")
                    open(dst, "w").write(cur.rstrip("\n") + "\n" + extra + "\n")
    print(f"created {created} stub files, appended {appended} stubs to existing files", file=sys.stderr)


if __name__ == "__main__":
    main()
