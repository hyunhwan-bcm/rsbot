"""Call the MEM4 readers that take a memory on every turn of the per-turn oracle.

    python3 harness/mem4_turns.py OUT.jsonl.gz [--limit N] [--skip K] [--oracle NAME]

Each oracle turn gives the line and the memory the modules saw (the turn's memory with the
observers' changes applied), so every reader is asked about every turn, not only the turns
where PyBot's own flow reached it. random.seed(0) before each call, as mem4_check assumes.
Same output format as mem4_calls.py.
"""
import argparse, copy, gzip, json, os, random, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "harness"))
import run_python  # noqa: E402
from export_data import encode  # noqa: E402
from plain import plain  # noqa: E402

ORACLE = os.path.join(ROOT, "data", "export", "turn_oracle", "conversations.jsonl.gz")

# (module, function, shape): "tm" = (text, memory), "m" = (memory,), "tbm" = (text, last bot line, memory)
FUNCTIONS = [
    ("bot.memory.repeat", "asks_for_the_same_work", "tm"), ("bot.memory.repeat", "answer_for", "tm"),
    ("bot.memory.repeat", "carried_on", "tm"), ("bot.memory.repeat", "shorter_for", "tm"),
    ("bot.memory.repeat", "agreement_with_the_answer", "tm"), ("bot.memory.repeat", "the_figure_asked", "tm"),
    ("bot.memory.repeat", "the_part_asked", "tm"), ("bot.memory.repeat", "what_it_knows", "tm"),
    ("bot.memory.repeat", "the_figure_they_gave", "tm"), ("bot.memory.repeat", "asked_back", "tm"),
    ("bot.memory.repeat", "the_same_question_again", "tm"), ("bot.memory.repeat", "asks_what_that_costs", "tm"),
    ("bot.memory.repeat", "the_move", "tbm"), ("bot.memory.repeat", "_recent_answers", "m"),
    ("bot.memory.repeat", "_handed_back_last", "m"), ("bot.memory.repeat", "_a_what_to_do_answer", "m"),
    ("bot.memory.ack", "reply", "tm"), ("bot.memory.ack", "a_person_named_alone", "tm"),
    ("bot.memory.ack", "a_person_pointed_at", "tm"), ("bot.memory.ack", "a_second_person", "tm"),
    ("bot.memory.ack", "value_put_right", "tm"), ("bot.memory.ack", "a_limit_reply", "tm"),
    ("bot.memory.ack", "a_limit_for_the_plan", "tm"), ("bot.memory.ack", "pet_given_away", "tm"),
    ("bot.memory.ack", "carries_the_story_on", "tm"), ("bot.memory.ack", "the_old_one_aged", "tm"),
    ("bot.memory.ack", "_the_length", "tm"), ("bot.memory.ack", "_a_remark_on_what_was_said", "tm"),
    ("bot.memory.ack", "_another_of_what_we_said", "tm"), ("bot.memory.ack", "_a_thing_they_have", "tm"),
    ("bot.memory.ack", "_a_job_to_do", "tm"), ("bot.memory.ack", "_plain", "tm"),
    ("bot.memory.ack", "nobody_around", "tm"), ("bot.memory.ack", "a_person_getting_better", "tm"),
    ("bot.memory.ack", "_after_a_feeling", "tm"), ("bot.memory.ack", "_a_date_on_the_thing_in_hand", "tm"),
    ("bot.memory.ack", "_echo", "tm"), ("bot.memory.ack", "only_a_dislike", "tm"), ("bot.memory.ack", "_learned", "tm"),
    ("bot.memory.ack", "_adds_a_detail", "tm"), ("bot.memory.ack", "_a_condition_they_wonder_about", "tm"),
    ("bot.memory.ack", "_no_subject", "tm"), ("bot.memory.ack", "_bad_news_just_before", "tm"),
    ("bot.memory.ack", "_heavy", "m"), ("bot.memory.ack", "_statement", "m"), ("bot.memory.ack", "_the_unit_before", "m"),
    ("bot.memory.ack", "_the_last_reply_gave_a_plan", "m"), ("bot.memory.ack", "_asked_last", "m"),
    ("bot.memory.callback", "reply_for", "tm"), ("bot.memory.callback", "claims", "tm"),
    ("bot.memory.callback", "_a_medical_episode", "m"),
]


def post_observer(rec):
    memory = copy.deepcopy(rec["mem_in"])
    for step in rec.get("observe") or []:
        diff = step.get("diff") or {}
        for k, v in (diff.get("set") or {}).items():
            memory[k] = v
        for k in diff.get("del") or []:
            memory.pop(k, None)
    return memory


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out"); ap.add_argument("--limit", type=int, default=2000); ap.add_argument("--skip", type=int, default=0)
    ap.add_argument("--oracle", default="conversations")
    a = ap.parse_args()
    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime  # noqa: F401
    import importlib, inspect
    fns = [(m, n, s, getattr(importlib.import_module(m), n)) for m, n, s in FUNCTIONS]
    out = []
    with gzip.open(ORACLE.replace("conversations", a.oracle), "rt") as f:
        for at, line in enumerate(f):
            if at < a.skip:
                continue
            if at >= a.skip + a.limit:
                break
            try:
                rec = json.loads(line)
            except Exception:  # noqa: BLE001
                break
            text = rec["text"]
            base = post_observer(rec)
            if "__unencodable__" in base:
                continue
            last_bot = (base.get("turns") or [{}])[-1].get("bot", "") if base.get("turns") else ""
            for mod, name, shape, fn in fns:
                memory = copy.deepcopy(base)
                args = {"tm": [text, memory], "m": [memory], "tbm": [text, last_bot, memory]}[shape]
                plain_args = [({"__memory__": copy.deepcopy(base)} if x is memory else x) for x in args]
                random.seed(0)
                try:
                    res = fn(*args)
                    raised = None
                except Exception as e:  # noqa: BLE001
                    res, raised = None, type(e).__name__
                try:
                    cres = plain(encode(res))
                except Exception:  # noqa: BLE001
                    cres = repr(res)
                after = plain(encode(memory))
                out.append({"fn": f"{mod}:{name}", "params": list(inspect.signature(fn).parameters),
                            "args": plain_args, "kwargs": {}, "result": cres, "type": type(res).__name__,
                            "raised": raised, "rng": None, "mem_after": after if after != base else None})
    with gzip.open(a.out, "wt") as f:
        for c in out:
            f.write(json.dumps(c, ensure_ascii=False) + "\n")
    print(f"{len(out)} calls", file=sys.stderr)


if __name__ == "__main__":
    main()
