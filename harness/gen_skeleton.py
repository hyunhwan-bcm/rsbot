"""Generate the Rust skeleton: one stub per Python top-level function, with a fixed signature.

    python harness/gen_skeleton.py OUT_SRC_DIR [--types TYPES.json] [--skip-files a.py,b.py] [--contracts OUT.json]

Every `bot/<pkg>/<file>.py` gets `src/<pkg>/<file>.rs` with, for each top-level function,

    /// Python: bot/memory/facts.py:123 `extract(text, memory=None)`
    pub fn extract(text: &str, memory: Option<&Memory>) -> Value { crate::unported!("bot.memory.facts:extract") }

The signature is the contract other workstreams call against, so it is decided here, once:
- parameter types from what the parameter held when PyBot ran (TYPES.json from
  harness/trace_full.py --types), else from its name (memory, text, understood, words, ...);
- `memory`: `&mut Memory` only when the function (or something it passes memory to) writes to
  memory - found by a fixed point over the call graph - and `&Memory` otherwise, so a function
  can be handed `memory` and `memory.understood` at once;
- return type from what the function returned when PyBot ran, else from its return statements;
- anything mixed or unknown is `Value` / `&Value` (a Python-like dynamic value).
Python default arguments are listed in the doc comment; Rust callers pass them explicitly.
"""
import argparse, ast, collections, json, os, re

import sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import workspace  # noqa: E402

ROOT = os.path.join(workspace.root(), "upstream", "pybot")
RUST_KW = set("as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while async await dyn abstract become box do final macro override priv typeof unsized virtual yield try gen".split())
MEMORY_NAMES = {"memory", "mine", "mem"}
TEXT_NAMES = set("""text clean line said word reply low phrase sentence clause answer name question opening last
    before after rest place subject plan ask asked thing who kind unit lead draft body tail head note fact
    line_text typed raw lowered stem own one other noun verb part piece label title topic key value_text
    reply_text the_text was what when where why how chunk fragment""".split())
READING_NAMES = {"understood", "reading", "read", "understanding", "u"}
WORDS_NAMES = {"words", "tokens", "norm", "forms", "names", "lines", "keywords", "parts", "items_said", "said_words", "clauses_words"}
MUTATORS = {"append", "extend", "add", "update", "pop", "insert", "remove", "clear", "discard", "setdefault", "popitem", "sort", "reverse", "__setitem__"}


def ident(name):
    return f"r#{name}" if name in RUST_KW else name


def file_mod(rel):
    return rel[:-3].replace("/", ".").removesuffix(".__init__")


class Analysis:
    def __init__(self):
        self.files = {}      # module -> (rel, tree)
        self.funcs = {}      # (module, name) -> FunctionDef
        for r, _, fs in os.walk(os.path.join(ROOT, "bot")):
            for f in sorted(fs):
                if f.endswith(".py"):
                    p = os.path.join(r, f); rel = os.path.relpath(p, ROOT)
                    tree = ast.parse(open(p).read())
                    mod = file_mod(rel)
                    self.files[mod] = (rel, tree)
                    for s in tree.body:
                        if isinstance(s, ast.FunctionDef):
                            self.funcs[(mod, s.name)] = s   # a redefinition: the last one wins

    def resolve_imports(self, mod):
        rel, tree = self.files[mod]
        alias_mod, alias_fn = {}, {}
        for n in ast.walk(tree):
            if isinstance(n, ast.ImportFrom):
                if n.level:
                    base = mod.split(".")
                    if not rel.endswith("__init__.py"):
                        base = base[:-1]
                    base = base[: len(base) - (n.level - 1)]
                    target = ".".join(base + ([n.module] if n.module else []))
                else:
                    target = n.module
                for a in n.names:
                    local = a.asname or a.name
                    if f"{target}.{a.name}" in self.files:
                        alias_mod[local] = f"{target}.{a.name}"
                    elif (target, a.name) in self.funcs:
                        alias_fn[local] = (target, a.name)
            elif isinstance(n, ast.Import):
                for a in n.names:
                    if a.name in self.files:
                        alias_mod[a.asname or a.name] = a.name
        return alias_mod, alias_fn

    def calls(self, mod, fn, alias_mod, alias_fn):
        """[(callee key, [arg exprs])] for calls inside fn that resolve to bot functions."""
        out = []
        for n in ast.walk(fn):
            if not isinstance(n, ast.Call):
                continue
            f = n.func
            key = None
            if isinstance(f, ast.Name):
                if (mod, f.id) in self.funcs:
                    key = (mod, f.id)
                elif f.id in alias_fn:
                    key = alias_fn[f.id]
            elif isinstance(f, ast.Attribute) and isinstance(f.value, ast.Name) and f.value.id in alias_mod:
                key = (alias_mod[f.value.id], f.attr) if (alias_mod[f.value.id], f.attr) in self.funcs else None
            if key:
                out.append((key, n))
        return out


def mutates_param_directly(fn, pname):
    """Does fn write into the object its parameter `pname` refers to (or into a value read out of it)?"""
    aliases = {pname}
    # names assigned from pname[...] / pname.get(...) / pname.setdefault(...) are views into it
    for n in ast.walk(fn):
        if isinstance(n, ast.Assign) and len(n.targets) == 1 and isinstance(n.targets[0], ast.Name):
            v = n.value
            src = v.func.value if isinstance(v, ast.Call) and isinstance(v.func, ast.Attribute) else (v.value if isinstance(v, ast.Subscript) else None)
            if isinstance(src, ast.Name) and src.id in aliases:
                aliases.add(n.targets[0].id)
    for n in ast.walk(fn):
        tgt = []
        if isinstance(n, (ast.Assign, ast.AugAssign, ast.AnnAssign)):
            tgt = n.targets if isinstance(n, ast.Assign) else [n.target]
        elif isinstance(n, ast.Delete):
            tgt = n.targets
        for t in tgt:
            if isinstance(t, ast.Subscript) and isinstance(t.value, ast.Name) and t.value.id in aliases:
                return True
            if isinstance(t, ast.Subscript) and isinstance(t.value, ast.Subscript):
                base = t.value
                while isinstance(base, ast.Subscript):
                    base = base.value
                if isinstance(base, ast.Name) and base.id in aliases:
                    return True
        if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute) and n.func.attr in MUTATORS:
            base = n.func.value
            while isinstance(base, (ast.Subscript, ast.Call)) and not isinstance(base, ast.Name):
                base = base.value if isinstance(base, ast.Subscript) else (base.func.value if isinstance(base.func, ast.Attribute) else base.func)
            if isinstance(base, ast.Name) and base.id in aliases:
                if n.func.attr == "get":
                    continue
                return True
    return False


def type_from_trace(entries, name, role):
    """Rust type from traced Python type names (role: 'param' or 'ret')."""
    descs = {t for t, _ in entries}
    opt = "None" in descs
    ds = descs - {"None"}
    if not ds:
        return None if role == "param" else ("()" if opt else None)
    def base(ds):
        if ds == {"str"}:
            return "&str" if role == "param" else "String"
        if ds == {"bool"}:
            return "bool"
        if ds <= {"int", "bool"}:
            return "i64"
        if ds == {"float"} or ds == {"int", "float"}:
            return "f64" if ds == {"float"} else ("&Value" if role == "param" else "Value")
        if ds == {"Understood"}:
            return "&Understood" if role == "param" else "Understood"
        if ds == {"Entry"}:
            return "&'static Entry"
        if all(re.fullmatch(r"(list|tuple|set)(\[str\])?", d) for d in ds):
            return "&[String]" if role == "param" else "Vec<String>"
        return "&Value" if role == "param" else "Value"
    b = base(ds)
    if opt:
        if b == "()":
            return "()"
        if b in ("&Value", "Value"):
            return b
        return f"Option<{b}>"
    return b


def type_from_name(name):
    if name in MEMORY_NAMES:
        return "MEMORY"
    if name in READING_NAMES:
        return "&Understood"
    if name in TEXT_NAMES or name.endswith(("_text", "_line", "_word", "_reply", "_said")):
        return "&str"
    if name in WORDS_NAMES or name.endswith(("_words", "_tokens", "_names")):
        return "&[String]"
    if name in ("item", "entry", "row", "card"):
        return "&'static Entry"
    if name in ("candidates",):
        return "&[Candidate]"
    if name in ("turns",):
        return "&[Turn]"
    if name in ("match", "m", "found_match"):
        return "&Value"
    if name in ("count", "n", "index", "at", "start", "end", "keep", "least", "limit", "size", "depth", "k"):
        return "i64"
    return "&Value"


STR_METHODS = {"lower", "upper", "strip", "lstrip", "rstrip", "replace", "format", "join", "capitalize",
               "title", "casefold", "translate", "center", "ljust", "rjust", "zfill", "removeprefix", "removesuffix"}
BOOL_FUNCS = {"bool", "any", "all", "isinstance", "callable", "hasattr"}
BOOL_METHODS = {"startswith", "endswith", "isdigit", "isalpha", "isalnum", "isupper", "islower", "isspace", "istitle", "isnumeric"}


def expr_type(e, rets, names):
    """Python type name of an expression: str, bool, int, None, list[str], or '?'."""
    if e is None:
        return "None"
    if isinstance(e, ast.Constant):
        v = e.value
        return "None" if v is None else "bool" if isinstance(v, bool) else "int" if isinstance(v, int) else "float" if isinstance(v, float) else "str" if isinstance(v, str) else "?"
    if isinstance(e, ast.JoinedStr):
        return "str"
    if isinstance(e, (ast.Compare,)) or (isinstance(e, ast.UnaryOp) and isinstance(e.op, ast.Not)):
        return "bool"
    if isinstance(e, ast.BoolOp):
        ts = {expr_type(v, rets, names) for v in e.values}
        return ts.pop() if len(ts) == 1 else "?"
    if isinstance(e, ast.IfExp):
        a, b = expr_type(e.body, rets, names), expr_type(e.orelse, rets, names)
        if a == b:
            return a
        if {a, b} == {"str", "None"}:
            return "str|None"
        return "?"
    if isinstance(e, ast.BinOp):
        a, b = expr_type(e.left, rets, names), expr_type(e.right, rets, names)
        if isinstance(e.op, ast.Add) and "str" in (a, b):
            return "str"
        if isinstance(e.op, ast.Mod) and a == "str":
            return "str"
        if a == b == "int" and isinstance(e.op, (ast.Add, ast.Sub, ast.Mult, ast.FloorDiv, ast.Mod)):
            return "int"
        return "?"
    if isinstance(e, ast.Call):
        f = e.func
        if isinstance(f, ast.Name):
            if f.id == "str":
                return "str"
            if f.id in BOOL_FUNCS:
                return "bool"
            if f.id in ("len", "int", "round"):
                return "int"
            if f.id in names:
                return rets.get(names[f.id], "?")
        if isinstance(f, ast.Attribute):
            if f.attr in STR_METHODS:
                return "str"
            if f.attr in BOOL_METHODS:
                return "bool"
            if f.attr in ("sub",) and isinstance(f.value, ast.Name) and f.value.id == "re":
                return "str"
            if isinstance(f.value, ast.Name) and (f.value.id, f.attr) in names:
                return rets.get(names[(f.value.id, f.attr)], "?")
        return "?"
    if isinstance(e, (ast.List, ast.ListComp, ast.Tuple)):
        elts = e.elts if isinstance(e, (ast.List, ast.Tuple)) else [e.elt]
        ts = {expr_type(x, rets, names) for x in elts}
        return "list[str]" if ts == {"str"} else ("list" if not ts else "?")
    return "?"


def returns_of(fn):
    if any(isinstance(n, (ast.Yield, ast.YieldFrom)) for n in ast.walk(fn)):
        return None
    out = []
    stack = list(fn.body)
    while stack:
        n = stack.pop()
        if isinstance(n, (ast.FunctionDef, ast.Lambda, ast.ClassDef)):
            continue
        if isinstance(n, ast.Return):
            out.append(n.value)
        stack.extend(ast.iter_child_nodes(n))
    # falling off the end returns None
    last = fn.body[-1] if fn.body else None
    if not isinstance(last, (ast.Return, ast.Raise)):
        out.append(None)
    return out


def combine(ts):
    ts = set(ts)
    if not ts or ts == {"None"}:
        return "None"
    if "?" in ts:
        return "?"
    if "str|None" in ts:
        ts = (ts - {"str|None"}) | {"str", "None"}
    if ts == {"bool"}:
        return "bool"
    if ts == {"str"}:
        return "str"
    if ts == {"int"}:
        return "int"
    if ts <= {"str", "None"}:
        return "str|None"
    if ts <= {"bool", "None"}:
        return "bool|None"
    if ts == {"list[str]"}:
        return "list[str]"
    return "?"


# Hand-written modules that live beside the generated ones.
EXTRA_MODS = {"bot/knowledge/__init__.py": ["entry", "registry"]}

HOOKS = {
    "observe": ("text: &str, memory: &mut Memory", ""),
    "respond": ("text: &str, memory: &mut Memory", " -> Option<String>"),
    "after": ("text: &str, reply: &str, memory: &mut Memory", ""),
    "choose": ("text: &str, memory: &Memory, candidates: &[Candidate]", " -> usize"),
}

RUST_RET = {"None": "()", "bool": "bool", "str": "String", "int": "i64", "str|None": "Option<String>",
            "bool|None": "Option<bool>", "list[str]": "Vec<String>", "?": "Value"}


def infer_returns(A, imports):
    """Fixed point of return types over the call graph, from return statements alone."""
    rets = {}
    names_of = {}
    for key, fn in A.funcs.items():
        am, af = imports[key[0]]
        names = {}
        for (m, n) in A.funcs:
            if m == key[0]:
                names[n] = (m, n)
        for local, target in af.items():
            names[local] = target
        for local, m in am.items():
            for (mm, n) in A.funcs:
                if mm == m:
                    names[(local, n)] = (mm, n)
        names_of[key] = names
    for _ in range(8):
        changed = False
        for key, fn in A.funcs.items():
            rs = returns_of(fn)
            t = "?" if rs is None else combine(expr_type(r, rets, names_of[key]) for r in rs)
            if rets.get(key) != t:
                rets[key] = t
                changed = True
        if not changed:
            break
    return {k: RUST_RET[v] for k, v in rets.items()}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out"); ap.add_argument("--types"); ap.add_argument("--skip-files", default="")
    ap.add_argument("--contracts")
    a = ap.parse_args()
    A = Analysis()
    traced = {}
    if a.types and os.path.exists(a.types):
        for k, v in json.load(open(a.types)).items():
            file, qual, line = k.rsplit(":", 2)
            if "." in qual or "<" in qual:
                continue
            traced[(file_mod(file), qual)] = v
    # memory / reading mutation, to a fixed point over the call graph
    imports = {m: A.resolve_imports(m) for m in A.files}
    calls = {}
    mut = {}  # (key, param) -> bool
    for key, fn in A.funcs.items():
        am, af = imports[key[0]]
        calls[key] = A.calls(key[0], fn, am, af)
        for p in fn.args.args:
            mut[(key, p.arg)] = mutates_param_directly(fn, p.arg)
    changed = True
    while changed:
        changed = False
        for key, fn in A.funcs.items():
            params = [p.arg for p in fn.args.args]
            for callee, call in calls[key]:
                cfn = A.funcs[callee]
                cparams = [p.arg for p in cfn.args.args]
                pairs = list(zip(cparams, call.args)) + [(kw.arg, kw.value) for kw in call.keywords if kw.arg]
                for cp, arg in pairs:
                    if isinstance(arg, ast.Name) and arg.id in params and mut.get((callee, cp)) and not mut[(key, arg.id)]:
                        mut[(key, arg.id)] = True
                        changed = True
    ast_rets = infer_returns(A, imports)
    skip = set(filter(None, a.skip_files.split(",")))
    contracts = {}
    written = 0
    for mod, (rel, tree) in sorted(A.files.items()):
        if rel in skip or rel.startswith("bot/runtime") or rel == "bot/__main__.py":
            continue
        lines = [f"//! Port of `{rel}`.", "//!", "//! GENERATED SKELETON: every function is a stub until its workstream ports it. The",
                 "//! signatures are the contract other files call against; keep them.",
                 "#![allow(unused_variables, unused_imports, non_snake_case, clippy::all)]", "",
                 "use crate::data::{self, Value};", "use crate::knowledge::entry::Entry;", "use crate::py::re::Re;",
                 "use crate::runtime::Candidate;", "use crate::state::{Memory, Turn};", "use crate::understood::Understood;",
                 "use std::sync::LazyLock;", ""]
        seen = set()
        fns = [s for s in tree.body if isinstance(s, ast.FunctionDef)]
        last = {s.name: s for s in fns}
        for s in fns:
            if last[s.name] is not s or s.name in seen:
                continue
            seen.add(s.name)
            key = (mod, s.name)
            tr = traced.get(key)
            hook = HOOKS.get(s.name) if mod.startswith("bot.modules.") else None
            if hook:
                pysig = ", ".join(p.arg for p in s.args.args)
                lines.append(f"/// Python: {rel}:{s.lineno} `{s.name}({pysig})` - a runtime hook; its signature is the runtime's.")
                lines.append(f"pub fn {s.name}({hook[0]}){hook[1]} {{")
                lines.append(f"    crate::unported!(\"{mod}:{s.name}\")")
                lines.append("}")
                lines.append("")
                contracts[f"{mod}:{s.name}"] = {"file": rel, "line": s.lineno, "rust": f"crate::{mod[4:].replace('.', '::')}::{s.name}",
                                                "params": [], "ret": hook[1], "defaults": {}, "traced": bool(tr), "hook": True}
                continue
            params = []
            for i, p in enumerate(s.args.args):
                t = None
                if tr and i < len(tr["args"]) and tr["args"][i]:
                    descs = {d for d, _ in tr["args"][i]}
                    if descs & {"Memory"} and descs <= {"Memory", "None", "dict"}:
                        t = "MEMORY_OPT" if "None" in descs else "MEMORY"
                    else:
                        t = type_from_trace(tr["args"][i], p.arg, "param")
                if t is None:
                    t = type_from_name(p.arg)
                if t in ("MEMORY", "MEMORY_OPT"):
                    m = "&mut Memory" if mut[(key, p.arg)] else "&Memory"
                    t = f"Option<{m}>" if t == "MEMORY_OPT" else m
                if t == "&Understood" and mut[(key, p.arg)]:
                    t = "&mut Understood"
                params.append((p.arg, t))
            if s.args.vararg:
                params.append((s.args.vararg.arg, "&[Value]"))
            for p in s.args.kwonlyargs:
                params.append((p.arg, type_from_name(p.arg)))
            ret = None
            if tr and tr["ret"]:
                ret = type_from_trace(tr["ret"], s.name, "ret")
            if ret is None:
                ret = ast_rets.get(key, "Value")
            # defaults, for the doc comment
            defs = {}
            nd = len(s.args.defaults)
            for p, d in zip(s.args.args[len(s.args.args) - nd:], s.args.defaults):
                defs[p.arg] = ast.unparse(d)
            pysig = ", ".join(p.arg + (f"={defs[p.arg]}" if p.arg in defs else "") for p in s.args.args)
            rsig = ", ".join(f"{ident(n)}: {t}" for n, t in params)
            rret = "" if ret == "()" else f" -> {ret}"
            lines.append(f"/// Python: {rel}:{s.lineno} `{s.name}({pysig})`{'' if tr else ' (never called on the trace corpus)'}")
            lines.append(f"pub fn {ident(s.name)}({rsig}){rret} {{")
            lines.append(f"    crate::unported!(\"{mod}:{s.name}\")")
            lines.append("}")
            lines.append("")
            contracts[f"{mod}:{s.name}"] = {"file": rel, "line": s.lineno, "rust": f"crate::{mod[4:].replace('.', '::')}::{ident(s.name)}" if mod != "bot" else ident(s.name),
                                            "params": params, "ret": ret, "defaults": defs, "traced": bool(tr)}
        if rel.endswith("__init__.py"):
            pkg_dir = os.path.dirname(os.path.join(ROOT, rel))
            subs = sorted(f[:-3] for f in os.listdir(pkg_dir) if f.endswith(".py") and f != "__init__.py")
            extras = EXTRA_MODS.get(rel, [])
            lines[5:5] = [*[f"pub mod {ident(n)};" for n in sorted(set(subs) | set(extras))], ""]
        dest = os.path.join(a.out, rel[len("bot/"):].replace("__init__.py", "mod.rs").removesuffix(".py") + ("" if rel.endswith("__init__.py") else ".rs"))
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        open(dest, "w").write("\n".join(lines) + "\n")
        written += 1
    if a.contracts:
        json.dump(contracts, open(a.contracts, "w"), indent=0)
    meta = json.load(open(os.path.join(workspace.root(), "slice", "modules_meta.json")))
    reg = os.path.join(a.out, "modules", "mod.rs")
    body = open(reg).read() if os.path.exists(reg) else ""
    mods = sorted({m["name"] for m in meta} | {"arbiter"})
    rows = []
    for m in meta:
        hook = lambda h: f"Some({m['name']}::{h})" if m[h] else "None"
        rows.append(f"        ModuleDef {{ name: \"{m['name']}\", priority: {float(m['priority'])!r}, observe: {hook('observe')}, respond: {hook('respond')}, after: {hook('after')} }},")
    reg_text = "\n".join([
        "//! Port of `bot/modules`: every module, in the order `bot/runtime.py` runs them",
        "//! (highest PRIORITY first, then by file name). Generated from PyBot's own module list.",
        "#![allow(clippy::all)]", "",
        "use crate::runtime::ModuleDef;", "",
        *[f"pub mod {ident(n)};" for n in mods], "",
        "/// Every module, highest PRIORITY first, then by name.",
        "pub fn registry() -> &'static [ModuleDef] {",
        "    static REGISTRY: &[ModuleDef] = &[",
        *rows,
        "    ];",
        "    REGISTRY",
        "}", "",
        "/// The arbiter's `choose`.",
        "pub fn choose(text: &str, memory: &crate::state::Memory, candidates: &[crate::runtime::Candidate]) -> usize {",
        "    arbiter::choose(text, memory, candidates)",
        "}", ""])
    open(reg, "w").write(reg_text + "\n" + body.split("\n", 4)[-1] if False else reg_text)
    n_mut = sum(1 for (k, p), v in mut.items() if v and p in MEMORY_NAMES)
    n_mem = sum(1 for (k, p) in mut if p in MEMORY_NAMES)
    print(f"{written} files, {len(contracts)} functions; memory params that write: {n_mut}/{n_mem}")


if __name__ == "__main__":
    main()
