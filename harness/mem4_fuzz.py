"""Call the MEM4 functions that read only text on many real lines, for mem4_check to replay.

    python3 harness/mem4_fuzz.py OUT.jsonl.gz [--per 400] [--seed 1]

The corpus run (`mem4_calls.py`) only reaches the arguments PyBot happens to pass, and most of
them stop at the first guard. This feeds every text-only reader the person's real lines and the
bot's real answers from the per-turn oracle (and the answer paired with the line that followed
it, for the readers of a follow-up), so the branches past the guards are reached too. Same output
format as mem4_calls.py.
"""
import argparse, gzip, json, os, random, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "harness"))
import run_python  # noqa: E402
from export_data import encode  # noqa: E402
from plain import plain  # noqa: E402

ORACLE = os.path.join(ROOT, "data", "export", "turn_oracle", "conversations.jsonl.gz")

# (module, function, shape): shape names the arguments - "u" a user line, "b" a bot answer,
# "bu" (answer, the line after it), "ub" (line, answer), "uu" two user lines.
FUNCTIONS = {
    "bot.memory.narrow": [
        ("only_a_length", "u"), ("kind_of", "u"), ("asks_a_part", "u"), ("step_word", "u"),
        ("the_outcome_asked", "u"), ("asked_unit", "u"), ("flat", "u"), ("tidy", "b"),
        ("words_of", "u"), ("has_a_date", "b"), ("_the_remedy_in", "b"), ("_as_a_length", "b"),
        ("narrow", "bu"), ("_narrow", "bu"), ("the_part_asked", "bu"), ("part_in_the_unit", "bu"),
        ("the_doings_length", "bu"), ("_named_clause", "bu"), ("narrow", "bq"), ("_narrow", "bq"),
        ("the_part_asked", "bq"), ("part_in_the_unit", "bq"), ("the_doings_length", "bq"),
        ("_named_clause", "bq"), ("kind_of", "q"), ("asks_a_part", "q"), ("_property_ask", "u"),
        ("_option_ask", "u"), ("_named_follow_up", "u"), ("_measure", "b"), ("_root", "w"),
        ("_unit_stem", "w"), ("_says_a_time", "b"), ("_only_a_cut_off", "b"),
        ("_informative", "b"), ("_stands_alone", "b"), ("_only_a_qualifier", "b"),
        ("_the_quotation", "b"), ("_names_a_thing", "u"), ("_only_a_count", "b"),
        ("_counts_on_the_answer", "b"), ("_states_a_length", "b"),
    ],
    "bot.memory.repeat": [
        ("without_a_reset", "u"), ("names_nothing", "u"), ("asks_for_the_move", "u"),
        ("asks_to_put_it_right", "u"), ("a_step_of_the_same_answer", "u"), ("asks_for_more", "u"),
        ("asks_for_less", "u"), ("_short", "b"), ("_opening_move", "s"), ("_a_move", "b"),
        ("_a_move_in", "b"), ("_a_conditional_move", "b"), ("_the_first_step", "b"),
        ("_sentences", "b"), ("_short_enough", "b"), ("_ended", "b"), ("_the_quotation", "b"),
        ("_read_up_to_the_colon", "b"), ("_whether_to", "u"), ("_a_stem", "w"),
        ("the_plain_line", "u"), ("how_long_its_been", "u"), ("after_an_ack", "u"),
        ("_the_advice", "b"), ("_next_step", "b"), ("_later_part", "ub"), ("_opens_a_doing", "s"),
        ("_tried", "u"), ("_says_they_agree", "u"), ("_the_thing_they_named", "ub"),
        ("_the_act", "w"), ("_the_steps_done", "u"), ("_a_step_done", "ub"),
        ("_starting_out_names_a_thing", "u"), ("says_where_to_start", "u"), ("a_more_ask", "u"),
        ("_pieces", "s"), ("_a_list_item", "s"), ("_a_bare_ask", "u"), ("_the_same_words", "uu"),
        ("_shortened", "b"), ("_says_what_to_do", "s"), ("_explains", "s"), ("_a_question", "s"),
        ("_unbalanced", "s"), ("_closing_the_quote", "sb"), ("_keeps", "bu"),
        ("_names_no_thing_of_its_own", "u"), ("_a_bare_time", "s"), ("_kept_clause", "u"),
        ("_a_lead_of_its_own", "s"), ("_words", "u"), ("_names_a_thing_of_its_own", "u"),
        ("_a_help_ask", "u"), ("only_said_it_back", "ub"), ("_trouble", "u"),
        ("_a_thing_of_its_own", "ub"), ("_a_feeling", "u"), ("_a_new_work", "uu"),
        ("asks_which_part", "u"), ("_leads_for", "u"), ("_the_lead_for", "us"),
        ("_a_lead_and_a_step", "ls"), ("_the_same_word", "ww"), ("_a_list_of_ages", "u"),
        ("_the_ask_gave_that_figure", "wu"), ("_money_paid_for_a_thing", "uu"),
        ("_a_bare_count", "w"), ("carries_the_figure", "sb"), ("answers_nothing", "b"),
        ("_a_step_comes_after", "s"), ("_opens_an_instruction", "s"), ("_opens_on_a_verb", "s"),
        ("_the_doing", "s"),
    ],
    "bot.memory.ack": [
        ("_a_relation", "w"), ("reads_as_a_receipt", "b"), ("_one_frame", "b"),
        ("reads_as_a_trip", "u"), ("reads_as_cruelty", "u"), ("reads_as_an_ask", "u"),
        ("_shape_of", "u"), ("_clock_shape", "u"), ("_tells_them_what_to_do", "b"),
        ("_a_bare_state", "u"), ("has_news", "u"), ("_the_bare_value", "u"),
        ("_after_the_hello", "u"), ("_a_day_coming", "u"), ("_an_hour_on_the_day", "u"),
        ("_the_figure_is_an_hour", "u"), ("_a_wanted_animal", "u"), ("_an_animal_wanted", "u"),
        ("_an_animal_they_had_once", "u"), ("_about_them", "u"), ("_who_is_getting_better", "u"),
        ("_said_to_the_bot", "u"), ("_words", "u"), ("_names_nothing", "u"), ("_plural", "w"),
        ("_is_a_contraction", "w"), ("_only_the_correction_in_front", "uu"),
        ("_strip_lead_in", "u"), ("_after_a_reaction", "u"), ("_spoken", "u"), ("_pick_off", "u"),
        ("_contrast_off", "u"), ("_reason_off", "uu"), ("_a_value", "w"), ("_cut", "u"),
        ("_tidy", "u"), ("_only_the_figure", "u"), ("_the_rest_of_a_question", "uu"),
        ("_a_visit", "u"), ("_the_milestone", "u"), ("_a_price_still_to_come", "uu"),
        ("_ending_of", "f"), ("without_the_frame", "u"), ("_the_units_measure", "u"),
        ("_the_values_measures", "u"), ("_shape_of", "u"), ("_a_thing_the_line_named", "u"),
    ],
}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out"); ap.add_argument("--per", type=int, default=400)
    ap.add_argument("--seed", type=int, default=1)
    a = ap.parse_args()
    rng = random.Random(a.seed)
    users, bots, pairs = [], [], []
    with gzip.open(ORACLE, "rt") as f:
        last = {}
        for line in f:
            try:
                r = json.loads(line)
            except Exception:  # noqa: BLE001 - a file still being written ends mid-line
                break
            users.append(r["text"])
            if r.get("reply"):
                bots.append(r["reply"])
            if last.get(r["conv"]) is not None and r["turn"] > 0:
                pairs.append((last[r["conv"]], r["text"]))
            last[r["conv"]] = r.get("reply")
    # The lines the readers were written for: every quoted example in the comments and docstrings
    # of the four files (and their neighbours'), mixed in with the real lines half the time.
    import re as _re
    examples = set()
    near = {}  # (file, line) -> the examples quoted there, for the readers written beside them
    src = os.path.join(ROOT, "upstream", "pybot", "bot", "memory")
    for name in ("ack.py", "repeat.py", "narrow.py", "callback.py"):
        for at, line in enumerate(open(os.path.join(src, name), encoding="utf-8"), 1):
            for found in _re.findall(r'"([^"\n]{3,240})"', line):
                if "{" in found or "\\" in found or "(?" in found or "|" in found:
                    continue
                examples.add(found.strip())
                near.setdefault(name, []).append((at, found.strip()))
    examples = sorted(examples)
    ex_users = [e for e in examples if len(e.split()) <= 16]
    ex_bots = [e for e in examples if len(e.split()) >= 6]
    sentences = [s for b in bots for s in b.replace("? ", "?\n").replace(". ", ".\n").split("\n") if s.strip()]
    words = sorted({w.strip(".,!?;:\"'()").lower() for u in users for w in u.split()} - {""})
    frames = ["{Thing} — noted.", "Got it — {thing}.", "{Thing}. Not long now.", "{When} so: {x}."]
    leads = ["Start with this: {step}.", "That's the one: {said}", "The first thing to do: {step}."]
    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime  # noqa: F401
    import importlib, inspect

    # Follow-ups that ask for a part of an answer, in the shapes the readers name.
    asks = ["how long", "and how long?", "how long does it take", "how long will it last", "how long before i can play a song",
            "how long should i rest it", "how long in the oven", "how long for a hard one?", "how many weeks", "how many days",
            "how many minutes", "what temperature", "what temp", "how hot", "what gas mark", "how much", "how much does it cost",
            "how much will that cost", "is that expensive", "how much is that", "how far", "how many miles", "what time",
            "what time should she go to bed", "when", "when did they leave", "when is the deadline", "what year", "how long ago",
            "why", "why is that", "what could be causing it", "what can i take for it", "what can i take", "how often",
            "how often should i do it", "and how often", "how often should i feed it", "what pressure", "how much water",
            "before or after", "can i make it the day before", "do i need to parboil them first", "should i have a go",
            "is it worth cleaning the filter", "and if i use a loaf tin?", "and for a hard boiled one?", "how do i get them crisp",
            "any tips for the first thirty seconds?", "will it be alright", "is that bad", "what happens now", "what does that mean",
            "when does it expire", "how long is the passport valid", "how much per kilo", "how long per kilo",
            "what about chops", "and for a steak", "how long for black tea", "should i say something", "when exactly",
            "how long until it is done", "how long has it been", "how long should i ice it"]
    local = []

    def sample(kind):
        if kind in "ub" and local and rng.random() < 0.5:
            return rng.choice(local)
        if kind == "u": return rng.choice(ex_users if ex_users and rng.random() < 0.5 else users)
        if kind == "b": return rng.choice(ex_bots if ex_bots and rng.random() < 0.5 else bots)
        if kind == "s": return rng.choice(ex_bots if ex_bots and rng.random() < 0.5 else sentences)
        if kind == "w": return rng.choice(words)
        if kind == "f": return rng.choice(frames)
        if kind == "q": return rng.choice(asks)
        if kind == "l": return rng.choice(leads)
        raise KeyError(kind)

    out, seen = [], set()
    for modname, fns in FUNCTIONS.items():
        mod = importlib.import_module(modname)
        for name, shape in fns:
            f = getattr(mod, name)
            params = list(inspect.signature(f).parameters)
            at = f.__code__.co_firstlineno
            local = [e for line, e in near.get(modname.rsplit(".", 1)[1] + ".py", []) if at - 160 <= line <= at + 160]
            for _ in range(a.per):
                if shape == "bu":
                    args = list(rng.choice(pairs)) if pairs and rng.random() < 0.4 else [sample("b"), sample("u")]
                elif shape == "bq":
                    args = [rng.choice(ex_bots if rng.random() < 0.5 else bots), rng.choice(asks)]
                elif shape == "ub":
                    one = rng.choice(pairs) if pairs else (sample("b"), sample("u"))
                    args = [one[1], one[0]]
                else:
                    args = [sample(k) for k in shape]
                key = name + json.dumps(args, ensure_ascii=False)
                if key in seen:
                    continue
                seen.add(key)
                random.seed(0)
                try:
                    res = f(*args)
                    if inspect.isgenerator(res):
                        res = list(res)
                    raised = None
                except Exception as e:  # noqa: BLE001
                    res, raised = None, type(e).__name__
                if isinstance(res, (set, frozenset)):
                    res = sorted(res)
                try:
                    cres = plain(encode(res))
                except Exception:  # noqa: BLE001
                    cres = repr(res)
                out.append({"fn": f"{modname}:{name}", "params": params, "args": args, "kwargs": {},
                            "result": cres, "type": type(res).__name__, "raised": raised,
                            "rng": None, "mem_after": None})
    with gzip.open(a.out, "wt") as f:
        for c in out:
            f.write(json.dumps(c, ensure_ascii=False) + "\n")
    print(f"{len(out)} calls", file=sys.stderr)


if __name__ == "__main__":
    main()
