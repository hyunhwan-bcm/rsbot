"""Record the calls PyBot makes to the MEM2 modules (facts, again, corrected, rename) and to the
functions of other modules they lean on, for a function-level check of the Rust port.

    python3 harness/mem2_record.py OUT.jsonl.gz --lines ORACLE.jsonl.gz [...] [--max N]
        [--cap 200] [--turns]

Without --turns every user line of the oracle files is handed to `facts.extract` (both as a
statement and as a question) and to `about_themselves`, which reaches nearly every reader in
facts.py. With --turns the whole conversations are replayed through the runtime, so the
functions other modules call (again.replied, corrected._apply_correction, facts.subject...) are
recorded with the arguments they really get.

Each distinct call is one line: {"fn": "module:qualname", "args": [...], "kwargs": {...},
"result": ..., "raised": null | "ExcName", "after": [...] (args after the call, when a call
changed them)}. A Match argument is {"__match__": {"re", "flags", "string", "pos", "endpos",
"spans"}}; a compiled pattern is {"__re__": [pattern, flags]}. Positions are Python's (code
points): the checker converts them.
"""
import argparse, copy, gzip, inspect, json, os, random, re, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "harness"))
import run_python  # noqa: E402

MINE = ("bot.memory.facts", "bot.memory.again", "bot.memory.corrected", "bot.memory.rename")
# Functions of other modules that the MEM2 files call and that are not ported yet: recorded with
# no cap, so a checker can answer them from the recording.
DEPS = {
    "bot.memory.taste": ("read_other", "said_about", "read_all", "read", "only_a_taste"),
    "bot.memory.filler": ("without_filler", "without_the_end_shrug", "without_the_shrug"),
    "bot.memory.named": ("the_name_in", "carries_a_name"),
    "bot.compose.badnews": ("reads_like_a_symptom",),
    "bot.memory.quotes": ("sentences", "closed"),
    "bot.memory.returned": ("says_it_happened_again", "the_next_step"),
    "bot.memory.repeat": ("asks_for_the_move",),
    "bot.memory.relations": ("_animal_family", "_tidy_name"),
    "bot.memory": ("plan_sentence",),
}


def lines_of(f):
    """The lines of a gzip file, stopping quietly where a file still being written ends."""
    try:
        for line in f:
            if line.endswith("\n"):
                yield line
    except EOFError:
        return


def enc(v, depth=0):
    if depth > 30:
        return {"__opaque__": "deep"}
    if v is None or isinstance(v, (bool, int, float, str)):
        return v
    if isinstance(v, re.Match):
        return {"__match__": {"re": v.re.pattern, "flags": int(v.re.flags), "string": v.string,
                              "pos": v.pos, "endpos": v.endpos,
                              "spans": [list(v.span(i)) for i in range(v.re.groups + 1)]}}
    if isinstance(v, re.Pattern):
        return {"__re__": [v.pattern, int(v.flags)]}
    if isinstance(v, (list, tuple)):
        return [enc(x, depth + 1) for x in v]
    if isinstance(v, (set, frozenset)):
        items = [enc(x, depth + 1) for x in v]
        try:
            return sorted(items)
        except TypeError:
            return items
    if isinstance(v, dict):
        if all(isinstance(k, str) for k in v):
            return {k: enc(x, depth + 1) for k, x in v.items()}
        return {"__pairs__": [[enc(k, depth + 1), enc(x, depth + 1)] for k, x in v.items()]}
    if inspect.isgenerator(v):
        return {"__opaque__": "generator"}
    return {"__opaque__": type(v).__name__}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--lines", nargs="*", default=[])
    ap.add_argument("--max", type=int, default=0, help="at most this many conversations per file")
    ap.add_argument("--cap", type=int, default=200)
    ap.add_argument("--turns", action="store_true")
    ap.add_argument("--script", help="a JSON list of conversations: each one is played through the "
                    "runtime, and the correction readers are called on the memory before its last line")
    ap.add_argument("--fixes", action="store_true",
                    help="on each oracle turn that puts a value right, call the correction readers that "
                         "write memory on a copy of the turn's memory")
    ap.add_argument("--oracle", action="store_true",
                    help="call the memory-reading functions on each oracle turn's own memory")
    a = ap.parse_args()

    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    import importlib
    from bot import runtime  # noqa: F401
    seen, counts = set(), {}
    out = gzip.open(a.out, "wt")
    depth = [0]

    def wrap(name, fn, cap):
        def wrapper(*args, **kwargs):
            try:
                key = name + repr(args) + repr(sorted(kwargs.items()))
            except Exception:  # noqa: BLE001
                key = None
            fresh = key is not None and key not in seen and counts.get(name, 0) < cap
            before = None
            if fresh:
                try:
                    before = copy.deepcopy((args, kwargs))
                except Exception:  # noqa: BLE001
                    fresh = False
            depth[0] += 1
            try:
                result = fn(*args, **kwargs)
            except Exception as e:  # noqa: BLE001
                if fresh:
                    seen.add(key)
                    counts[name] = counts.get(name, 0) + 1
                    out.write(json.dumps({"fn": name, "args": enc(before[0]), "kwargs": enc(before[1]),
                                          "result": None, "raised": type(e).__name__},
                                         ensure_ascii=False) + "\n")
                raise
            finally:
                depth[0] -= 1
            if fresh:
                seen.add(key)
                counts[name] = counts.get(name, 0) + 1
                row = {"fn": name, "args": enc(before[0]), "kwargs": enc(before[1]),
                       "result": enc(result), "raised": None}
                after = enc(args)
                if after != row["args"]:
                    row["after"] = after
                out.write(json.dumps(row, ensure_ascii=False) + "\n")
            return result
        wrapper.__wrapped__ = fn
        return wrapper

    for modname in MINE:
        mod = importlib.import_module(modname)
        for attr, fn in list(vars(mod).items()):
            if inspect.isfunction(fn) and fn.__module__ == modname:
                cap = 10 ** 9 if attr in ("extract", "about_themselves") else a.cap
                setattr(mod, attr, wrap(f"{modname}:{attr}", fn, cap))
    for modname, names in DEPS.items():
        mod = importlib.import_module(modname)
        for attr in names:
            fn = getattr(mod, attr)
            w = wrap(f"{modname}:{attr}", fn, 10 ** 9)
            setattr(mod, attr, w)
            # Modules that imported the function by name hold their own reference.
            for user in MINE:
                umod = sys.modules[user]
                if getattr(umod, attr, None) is fn:
                    setattr(umod, attr, w)

    # Modules that imported a function by name hold the function itself: point them at the
    # wrapper too, so their calls are recorded.
    wrapped = {}
    for modname in MINE + tuple(DEPS):
        for attr, value in list(vars(sys.modules[modname]).items()):
            inner = getattr(value, "__wrapped__", None)
            if inner is not None:
                wrapped[id(inner)] = value
    for modname, mod in list(sys.modules.items()):
        if not modname.startswith("bot") or mod is None:
            continue
        for attr, value in list(vars(mod).items()):
            if callable(value) and id(value) in wrapped:
                setattr(mod, attr, wrapped[id(value)])
    facts = sys.modules["bot.memory.facts"]
    again = sys.modules["bot.memory.again"]
    corrected = sys.modules["bot.memory.corrected"]
    rename = sys.modules["bot.memory.rename"]

    def quietly(fn, *args):
        try:
            fn(*args)
        except Exception:  # noqa: BLE001
            pass

    def oracle_calls(row, text):
        """The turn's memory as the responders see it (mem_in and the observers' writes), handed
        to the functions that read memory, with each reply the responders gave."""
        memory = copy.deepcopy(row.get("mem_in") or {})
        for ob in row.get("observe") or []:
            diff = ob.get("diff") or {}
            for key, value in (diff.get("set") or {}).items():
                memory[key] = copy.deepcopy(value)
            for key in diff.get("del") or []:
                memory.pop(key, None)
        replies = [r["reply"] for r in row.get("respond") or []
                   if isinstance(r.get("reply"), str) and r["reply"].strip()]
        replies += [t.get("bot") for t in (memory.get("turns") or [])[-2:] if isinstance(t, dict)
                    and isinstance(t.get("bot"), str) and t.get("bot").strip()]
        for reply in replies[:6]:
            quietly(again.instead, reply, text, memory)
            quietly(again.already_had, reply, memory)
            quietly(again.a_thing_of_the_answer, reply, text, memory)
            quietly(again.the_cause_said_again, reply, memory)
            quietly(again.the_same_thing_underneath, text, reply, 2, memory)
            quietly(again._short, reply, memory)
            quietly(again._framed_as_a_repeat, reply, again._flat(reply), memory, 1, text)
            quietly(again.carries_a_lead_in, reply)
            quietly(again.the_name_the_answer_says, reply, text, reply)
        user = memory.get("user") if isinstance(memory.get("user"), dict) else {}
        pets = user.get("pets") or []
        turns = memory.get("turns") or []
        before = turns[-1].get("user", "") if turns and isinstance(turns[-1], dict) else ""
        quietly(facts.count_of_the_people, text, memory)
        quietly(facts.names_offered, memory)
        quietly(facts.a_name_picked, text, memory)
        quietly(facts.the_name_they_take_back, text, user.get("name") or "")
        quietly(facts.a_correction_without_a_subject, text, before)
        quietly(facts.denials, text, pets)
        quietly(facts.losses, text, pets, before)
        quietly(facts.given_away_one, text, pets)
        quietly(facts.their_back, text)
        quietly(facts.one_more, text)
        quietly(facts.the_limit_they_share, text)
        quietly(facts.kid_ages, text)
        quietly(facts.ages_said_one_by_one, text, 2)
        quietly(facts.pet_name_only, text, True)
        quietly(facts.pet_appositive, text)
        quietly(facts.bare_name, text)
        quietly(facts.a_moved_place, text)
        quietly(facts.the_date_put_right, text)
        quietly(facts.a_trip_date_put_right, text)
        quietly(facts.without_the_correction_opener, text)
        quietly(facts.is_a_correction_opener, text)
        quietly(facts._reported_clause, text)
        quietly(rename.moves, text)
        quietly(rename.the_name_a_line_moved_on, text.lower(), pets, False)
        quietly(rename.the_name_a_line_moved_on, text.lower(), pets, True)
        found = {}
        try:
            found = facts.extract(text)
        except Exception:  # noqa: BLE001
            pass
        fix = found.get("correction") if isinstance(found, dict) else None
        if isinstance(fix, dict):
            to = fix.get("to")
            quietly(corrected._a_name_for_a_person, to)
            quietly(corrected._a_new_name_was_said, to, text)
            quietly(corrected._without_the_naming_verb, to)
            quietly(corrected._date_said, str(to or "").lower())
            quietly(corrected._kind_of_value, str(to or ""), before)
            quietly(corrected._swap_in, before, to, fix.get("from"))
            quietly(corrected._an_amount_on_the_measure, before, to)
            quietly(corrected._count_said, str(to or ""))
            quietly(corrected._names_a_date, to)
            quietly(corrected._measured_the_same, to, before)
        quietly(corrected._asks, text)

    def fix_calls(row, text):
        """The readers that put a value right in memory, each on its own copy of the turn's memory."""
        memory = copy.deepcopy(row.get("mem_in") or {})
        for ob in row.get("observe") or []:
            diff = ob.get("diff") or {}
            for key, value in (diff.get("set") or {}).items():
                memory[key] = copy.deepcopy(value)
            for key in diff.get("del") or []:
                memory.pop(key, None)
        try:
            found = facts.extract(text)
        except Exception:  # noqa: BLE001
            return
        fix = found.get("correction") if isinstance(found, dict) else None
        if not isinstance(fix, dict) or not fix.get("to"):
            return
        to, wrong = fix.get("to"), fix.get("from")
        for fn, args in ((corrected._correct_count, (to, text)),
                         (corrected._correct_a_persons_name, (to, text, wrong or "")),
                         (corrected._a_value_already_the_plans, (to, text)),
                         (corrected._a_correction_is_not_a_plan, (text,)),
                         (corrected._correct_their_birthday, (to, wrong, text)),
                         (corrected._a_birthday_for_somebody, ()),
                         (corrected._stated_last, ()),
                         (corrected._stated_plan, ()),
                         (corrected._fact_naming, (wrong,)),
                         (corrected._a_birthday_on_the_last_line, ()),
                         (corrected._number_belongs_to_that_age, ()),
                         (corrected._the_plan_on_the_table, ())):
            quietly(fn, copy.deepcopy(memory), *args)
        quietly(corrected._make_corrects, to, copy.deepcopy(memory))
        quietly(corrected._the_person_the_plan_is_about, copy.deepcopy(memory),
                {"value": to, "sentence": text}, None)

    done = 0
    if a.script:
        from bot import runtime as rt
        for conv in json.load(open(a.script)):
            memory = {}
            random.seed(0)
            for line in conv[:-1]:
                quietly(rt.turn, line, memory)
            last = conv[-1]
            row = {"mem_in": memory, "observe": [], "respond": []}
            fix_calls(row, last)
            found = {}
            try:
                found = facts.extract(last)
            except Exception:  # noqa: BLE001
                pass
            fix = found.get("correction") if isinstance(found, dict) else None
            if isinstance(fix, dict):
                quietly(corrected._apply_correction, copy.deepcopy(memory), dict(fix), last)
                for fn in ("_correct_plan", "_correct_date", "_correct_clock", "_correct_number",
                           "_correct_count", "_correct_fact", "_correct_a_persons_name",
                           "_correct_when", "_correct_the_count_on_the_table", "_correct_other",
                           "_correct_birthday"):
                    reader = getattr(corrected, fn)
                    if fn in ("_correct_plan", "_correct_fact"):
                        quietly(reader, copy.deepcopy(memory), fix.get("to"), fix.get("from"), last)
                    elif fn in ("_correct_date", "_correct_clock", "_correct_a_persons_name"):
                        quietly(reader, copy.deepcopy(memory), fix.get("to"), last, fix.get("from") or "")
                    else:
                        quietly(reader, copy.deepcopy(memory), fix.get("to"), last)
            quietly(rt.turn, last, memory)
            done += len(conv)
    for path in a.lines:
        convs = 0
        last = None
        with gzip.open(path, "rt") as f:
            for line in lines_of(f):
                row = json.loads(line)
                if row.get("conv") != last:
                    last = row.get("conv")
                    convs += 1
                    if a.max and convs > a.max:
                        break
                text = row.get("text") or ""
                if a.turns:
                    continue
                if a.fixes:
                    fix_calls(row, text)
                    done += 1
                    continue
                if a.oracle:
                    oracle_calls(row, text)
                    done += 1
                    continue
                for question in (False, True):
                    try:
                        facts.extract(text, question)
                    except Exception:  # noqa: BLE001
                        pass
                try:
                    facts.about_themselves(text)
                except Exception:  # noqa: BLE001
                    pass
                done += 1
        if a.turns:
            # Whole conversations, through the runtime, as the turn oracle ran them.
            from bot import runtime as rt
            with gzip.open(path, "rt") as f:
                convs, last, memory = 0, None, None
                for line in f:
                    row = json.loads(line)
                    if row.get("conv") != last:
                        last = row.get("conv")
                        convs += 1
                        if a.max and convs > a.max:
                            break
                        memory = {}
                        random.seed(0)
                    try:
                        rt.turn(row.get("text") or "", memory)
                    except Exception:  # noqa: BLE001
                        pass
                    done += 1
    out.close()
    print(f"{done} lines; {len(counts)} functions; {sum(counts.values())} calls", file=sys.stderr)


if __name__ == "__main__":
    main()
