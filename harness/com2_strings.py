"""Call the COM2 files' readers of a single line on a corpus of real lines, for `com2_check`.

    python harness/com2_strings.py OUT.jsonl.gz [--limit N]

The answers corpus is every knowledge entry whose reply is a fixed string plus every reply the
turn oracles recorded (the bot's own lines, whichever module wrote them); the person's corpus is
every line typed in the benchmark and the conversations. The readers that take one string (and
the ones that take a pool and a string) are called on each, and every call is written in the
same shape `harness/com2_calls.py` writes, so `src/bin/com2_check.rs` replays both.
"""
import argparse, glob, gzip, json, os, random, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import run_python  # noqa: E402
from export_data import encode  # noqa: E402
from plain import plain  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
if not os.path.isdir(os.path.join(ROOT, "data")):
    ROOT = os.path.abspath(os.path.join(HERE, ".."))

# Readers of an answer (the bot's line), by name in bot.compose.verdict.
ANSWER_READERS = [
    "_steps", "_step", "_when", "_referral", "_stated_referral", "_unconditional_care", "_danger",
    "_a_how_to", "_says_what_it_is", "_the_list_before_the_call", "_the_call_and_its_reason",
    "_listed", "_action_count", "_care_pool", "_the_question_asked", "_ends_on_a_question",
    "_an_instruction_sentence", "_the_referral_it_closed_on", "_the_range_it_gave", "_said_no",
    "_said_enough", "_opening", "_a_judgement_fence", "_the_answer_already_warned",
    "opens_on_a_job", "_a_job_somewhere", "rules_on_the_doubt", "a_check_on_the_reply",
    "takes_a_fact_back", "_kept_step", "_a_call_for_now", "_the_call_sentence", "_care_for_now",
    "_the_referral_sentence", "_a_fix_with_no_warning", "_the_advice_itself", "_a_job_to_do",
    "_is_an_explanation", "_an_offer_is_waiting", "_is_a_question", "_a_described_condition",
    "_first_sentence_end", "_stems", "_subject_words", "_is_a_comparison", "_flat", "_words",
    "_own_pattern", "_a_throw_away", "_no_hanging_end", "_after_the_if", "_cut_condition",
    "_an_if_that_is_a_condition", "_stated_before_the_judgement", "_cut_at_a_join", "_first_clause",
]
# Readers of the person's line, by name in bot.compose.verdict.
LINE_READERS_VERDICT = ["_a_feeling_of_their_own", "_a_figure_owed", "_the_plan_can_hurt", "_the_thing",
                        "_the_sign_words", "_an_amount_asked"]
# Readers of the person's line, by name in bot.compose.fallback_chain (given the cleaned line).
LINE_READERS_CHAIN = [
    "_body_state", "_a_part_in_a_state", "reads_as_no_pleasure", "reads_as_a_question_about_the_bot",
    "_a_harm_said", "_loves_people", "_a_betrayal", "_a_thing_they_regret", "_a_day_said_badly",
    "_a_state_of_theirs", "_a_name_told", "_the_crowd_is_ahead", "_a_report_from_elsewhere",
    "_an_order_it_cannot_do", "_act_asked_of_me", "_the_morning_has_a_clock", "_a_near_miss",
    "_reads_as_a_bad_stretch", "_a_problem_stated", "_a_thing_got_ahead", "_a_claim_about_a_thing",
    "_words", "_a_day_of_someone_else", "_a_thing_happened", "_habit_is_a_problem",
    "_says_there_is_nothing_to_wear", "_a_wear_ask_about_a_day", "_the_wear_pool_for_the_line",
]
ONSCREEN_READERS = ["talks_about_the_conversation", "_the_parts"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--limit", type=int, default=0)
    a = ap.parse_args()
    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime  # noqa: F401
    import importlib, inspect
    from bot import knowledge
    verdict = importlib.import_module("bot.compose.verdict")
    chain = importlib.import_module("bot.compose.fallback_chain")
    onscreen = importlib.import_module("bot.compose.onscreen")

    answers, lines = [], []
    for e in knowledge.entries():
        if isinstance(getattr(e, "reply", None), str):
            answers.append(e.reply)
    for path in sorted(glob.glob(os.path.join(ROOT, "data", "export", "turn_oracle", "*.jsonl.gz"))):
        try:
            with gzip.open(path, "rt") as fh:
                for raw in fh:
                    d = json.loads(raw)
                    if isinstance(d.get("reply"), str):
                        answers.append(d["reply"])
                    if isinstance(d.get("text"), str):
                        lines.append(d["text"])
        except Exception:  # noqa: BLE001 - a file still being written ends early
            pass
    for path in ("all.jsonl",):
        for raw in open(os.path.join(ROOT, "data", "smoltalkbench", path)):
            d = json.loads(raw)
            if isinstance(d.get("instruction"), str):
                lines.append(d["instruction"])
    answers = list(dict.fromkeys(answers))
    lines = list(dict.fromkeys(lines))
    if a.limit:
        random.Random(0).shuffle(answers)
        random.Random(1).shuffle(lines)
        answers, lines = answers[:a.limit], lines[:a.limit]

    out = gzip.open(a.out, "wt")
    n = 0
    seen = set()

    def call(modname, mod, name, args):
        nonlocal n
        key = json.dumps([name, args], ensure_ascii=False)
        if key in seen:
            return
        seen.add(key)
        f = getattr(mod, name)
        params = list(inspect.signature(f).parameters)
        try:
            res, raised = f(*args), None
        except Exception as e:  # noqa: BLE001
            res, raised = None, type(e).__name__
        rec = {"fn": f"{modname}:{name}", "params": params, "args": [plain(encode(x)) for x in args],
               "kwargs": {}, "raised": raised, "type": type(res).__name__}
        try:
            rec["result"] = plain(encode(res))
        except Exception:  # noqa: BLE001
            rec["result"] = {"$opaque": type(res).__name__}
        out.write(json.dumps(rec, ensure_ascii=False) + "\n")
        n += 1

    for said in answers:
        for name in ANSWER_READERS:
            call("bot.compose.verdict", verdict, name, [said])
        call("bot.compose.verdict", verdict, "_first_positive", [said, list(verdict.REASSURE)])
        call("bot.compose.verdict", verdict, "_first", [said, list(verdict.CARE)])
        call("bot.compose.verdict", verdict, "_round_robin", [list(verdict.YES_LINES), said])
        # Readers that take a position or a step the answer gave, fed the ones PyBot reads off it.
        care_at = verdict._first(said.lower(), verdict.CARE)
        call("bot.compose.verdict", verdict, "_condition_after_care", [said, care_at, None])
        care = verdict._referral(said)
        if care is not None:
            call("bot.compose.verdict", verdict, "_a_plain_go", [care, said])
            call("bot.compose.verdict", verdict, "_told_to_act", [said, care])
            call("bot.compose.verdict", verdict, "_behind_a_description", [said.lower(), care])
        step = verdict._step(said)
        if step:
            call("bot.compose.verdict", verdict, "_step_sentence", [said, step])
            whole = verdict._step_sentence(said, step)
            if whole:
                call("bot.compose.verdict", verdict, "_up_to_a_break", [whole, step, verdict.HAND_OVER_WORDS])
            call("bot.compose.verdict", verdict, "_the_other_thing_in_the_answer", [said, step, {}])
        call("bot.compose.verdict", verdict, "_the_other_thing_in_the_answer", [said, None, {}])
        call("bot.compose.verdict", verdict, "_described_clause", [said])
        for name in ONSCREEN_READERS:
            call("bot.compose.onscreen", onscreen, name, [said])
    for line in lines:
        clean = " ".join(line.lower().split())
        for name in LINE_READERS_VERDICT:
            call("bot.compose.verdict", verdict, name, [line])
        for name in LINE_READERS_CHAIN:
            if name in ("_says_there_is_nothing_to_wear", "_a_wear_ask_about_a_day", "_the_wear_pool_for_the_line"):
                call("bot.compose.fallback_chain", chain, name, [clean, clean, line])
            else:
                call("bot.compose.fallback_chain", chain, name, [clean])
        call("bot.compose.verdict", verdict, "_flat", [line])
        call("bot.compose.onscreen", onscreen, "talks_about_the_conversation", [line])
        for word in clean.split():
            call("bot.compose.onscreen", onscreen, "_stem", [word])
    out.close()
    print(f"{len(answers)} answers, {len(lines)} lines, {n} calls", file=sys.stderr)


if __name__ == "__main__":
    main()
