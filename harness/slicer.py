"""Slice PyBot down to the code that actually ran on a set of prompts.

    python harness/slicer.py COVERAGE.json SRC_BOT_DIR OUT_DIR [--mode exec|spec]

COVERAGE.json is what harness/coverage_run.py writes: {"import": {file: [lines]}, "turn": {...}}.

exec mode writes a runnable copy of the bot package in which every statement that never ran
(import or turn) is removed, every `if` branch that was never taken is dropped, and every
function whose body never ran returns None. On the prompts the coverage came from it must give
exactly the replies the full bot gives; on other prompts the difference measures what slicing
cost.

spec mode writes the hand-port specification: per file, only the functions whose bodies ran
during turns (sliced the same way), without comments or docstrings, and the module-level names
those functions read. Module-level data is not in the spec: it is exported as data.
"""
import argparse
import ast
import json
import os
import sys


def lines_of(node):
    return range(node.lineno, (node.end_lineno or node.lineno) + 1)


class Slicer:
    def __init__(self, hits):
        self.hits = hits

    def hit(self, node, lo=None, hi=None):
        lo = node.lineno if lo is None else lo
        hi = (node.end_lineno or node.lineno) if hi is None else hi
        return any(line in self.hits for line in range(lo, hi + 1))

    def block(self, stmts):
        out = []
        for s in stmts:
            out.extend(self.stmt(s))
        return out

    def is_docstring(self, s):
        return isinstance(s, ast.Expr) and isinstance(s.value, ast.Constant) and isinstance(s.value.value, str)

    def stmt(self, s):
        if self.is_docstring(s):
            return []
        if isinstance(s, (ast.Global, ast.Nonlocal)):
            return [s]
        if isinstance(s, (ast.FunctionDef, ast.AsyncFunctionDef)):
            head_lo = min([d.lineno for d in s.decorator_list] + [s.lineno])
            body_first = s.body[0].lineno
            if not self.hit(s, head_lo, body_first - 1) and not self.hit(s, body_first, s.end_lineno):
                return []
            body_stmts = [b for b in s.body if not self.is_docstring(b)]
            generator = is_generator(s)
            if not body_stmts or not any(self.hit(b) for b in body_stmts):
                s.body = [ast.Return(value=ast.Constant(value=None))]
            else:
                s.body = self.block(body_stmts) or [ast.Pass()]
            if generator and not is_generator(s):
                # every yield was sliced away: keep it a generator, one that yields nothing
                s.body.append(ast.If(test=ast.Constant(value=False),
                                     body=[ast.Expr(value=ast.Yield(value=None))], orelse=[]))
            return [s]
        if isinstance(s, ast.ClassDef):
            return [s] if self.hit(s) else []
        if isinstance(s, ast.If):
            if not self.hit(s.test):
                return []
            body = self.block(s.body)
            orelse = self.block(s.orelse)
            if not body and not orelse:
                return []
            if not body:
                return orelse
            s.body, s.orelse = body, orelse
            return [s]
        if isinstance(s, (ast.For, ast.AsyncFor)):
            if not self.hit(s, s.lineno, s.iter.end_lineno):
                return []
            s.body = self.block(s.body) or [ast.Pass()]
            s.orelse = self.block(s.orelse)
            return [s]
        if isinstance(s, ast.While):
            # A while loop keeps its whole body: a statement that never ran on these prompts may
            # be the one that moves the loop on (a counter, the next item), and without it the
            # loop never ends on a line that reaches the second pass.
            return [s] if self.hit(s.test) else []
        if isinstance(s, ast.Try):
            body = self.block(s.body)
            handlers = []
            for h in s.handlers:
                hb = self.block(h.body)
                if hb:
                    h.body = hb
                    handlers.append(h)
            orelse = self.block(s.orelse)
            final = self.block(s.finalbody)
            if not body:
                return final
            if not handlers and not final:
                return body + orelse
            if not handlers:
                s.handlers = []
            s.body, s.handlers, s.orelse, s.finalbody = body, handlers, orelse, final
            if not s.handlers and not s.finalbody:
                return body + orelse
            return [s]
        if isinstance(s, (ast.With, ast.AsyncWith)):
            if not self.hit(s, s.lineno, s.items[-1].context_expr.end_lineno):
                return []
            s.body = self.block(s.body) or [ast.Pass()]
            return [s]
        # simple statement
        return [s] if self.hit(s) else []


def is_generator(fn):
    """True when fn's own body (not a nested def or lambda) yields."""
    stack = list(fn.body)
    while stack:
        n = stack.pop()
        if isinstance(n, (ast.Yield, ast.YieldFrom)):
            return True
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef)):
            continue
        stack.extend(ast.iter_child_nodes(n))
    return False


def module_level_names(tree):
    names = set()
    for s in tree.body:
        targets = []
        if isinstance(s, ast.Assign):
            targets = s.targets
        elif isinstance(s, (ast.AnnAssign, ast.AugAssign)):
            targets = [s.target]
        for t in targets:
            for n in ast.walk(t):
                if isinstance(n, ast.Name):
                    names.add(n.id)
    return names


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("coverage")
    ap.add_argument("src")
    ap.add_argument("out")
    ap.add_argument("--mode", choices=["exec", "spec"], default="exec")
    a = ap.parse_args()
    cov = json.load(open(a.coverage))
    turn = {k: set(v) for k, v in cov["turn"].items()}
    imp = {k: set(v) for k, v in cov["import"].items()}
    src_parent = os.path.dirname(os.path.abspath(a.src))
    total_in = total_out = 0
    report = {}
    for root, _, files in os.walk(a.src):
        for f in sorted(files):
            if not f.endswith(".py"):
                continue
            path = os.path.join(root, f)
            rel = os.path.relpath(path, src_parent)
            text = open(path).read()
            tree = ast.parse(text)
            total_in += text.count("\n")
            if a.mode == "exec" and rel == os.path.join("bot", "runtime.py"):
                out_text = text  # the fixed interface stays whole: it is what catches a failing module
            elif a.mode == "exec":
                hits = turn.get(rel, set()) | imp.get(rel, set())
                tree.body = Slicer(hits).block(tree.body) or [ast.Pass()]
                out_text = ast.unparse(tree) + "\n"
            else:
                hits = turn.get(rel, set())
                if not hits:
                    continue
                data_names = module_level_names(tree)
                kept = []
                for s in tree.body:
                    if isinstance(s, (ast.FunctionDef, ast.ClassDef)):
                        body_stmts = [b for b in s.body if not Slicer(hits).is_docstring(b)]
                        if body_stmts and any(Slicer(hits).hit(b) for b in body_stmts):
                            kept.extend(Slicer(hits).stmt(s))
                if not kept:
                    continue
                used = set()
                for s in kept:
                    for n in ast.walk(s):
                        if isinstance(n, ast.Name) and n.id in data_names:
                            used.add(n.id)
                imports = [s for s in tree.body if isinstance(s, (ast.Import, ast.ImportFrom))]
                mod = ast.Module(body=imports + kept, type_ignores=[])
                out_text = "# data used (exported, not ported): " + ", ".join(sorted(used)) + "\n" + ast.unparse(mod) + "\n"
                report[rel] = {"functions": [s.name for s in kept], "data": sorted(used),
                               "lines": out_text.count("\n")}
            dest = os.path.join(a.out, rel)
            os.makedirs(os.path.dirname(dest), exist_ok=True)
            open(dest, "w").write(out_text)
            total_out += out_text.count("\n")
    if a.mode == "spec":
        json.dump(report, open(os.path.join(a.out, "spec_index.json"), "w"), indent=1)
    print(f"{a.mode}: {total_in} source lines -> {total_out} sliced lines", file=sys.stderr)


if __name__ == "__main__":
    main()
