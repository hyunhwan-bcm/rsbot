"""Probe bot/compose/why.py directly with the bot's own answers, for the replay check
`src/bin/com1_check.rs` (whose recording from whole conversations reaches only why.py's early
returns, since a bare "why" after a long answer is rare in the oracles).

    python3 harness/why_probe.py OUT_DIR [--oracle FILE.jsonl.gz ...] [--most N]

Every distinct turn ({user, bot, module}) in the oracles' memories is handed to why.py's
functions: the string readers on the answer and on its sentences, parts and colon halves; the
turn readers on the turn; and the replies (`respond`, `respond_to_a_saying_ask`,
`takes_as_it_comes`, `said_again`) on a memory of the turns up to it, for a set of follow-ups.
Writes OUT_DIR/calls.jsonl in com1_calls.py's format (arguments inline) and an empty
OUT_DIR/objs.jsonl.
"""
import argparse, gzip, json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_python
from export_data import encode
from plain import plain

ORACLES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "data", "export",
                       "turn_oracle")
FOLLOW_UPS = ("why?", "why", "why not?", "but why?", "why is that?", "how come?",
              "why is it doing that?", "what's causing it?", "why does that happen?",
              "say that again", "say that again?", "come again?", "why do you say that?",
              "why would you say that", "what makes you think that?", "why's that")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--oracle", nargs="*", default=[os.path.join(ORACLES, n) for n in (
        "conversations.jsonl.gz", "synthetic.jsonl.gz", "single.jsonl.gz")])
    ap.add_argument("--most", type=int, default=4000, help="distinct answers to probe")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)

    seen, memories = set(), []
    for path in a.oracle:
        try:
            with gzip.open(path, "rt") as f:
                for line in f:
                    if len(memories) >= a.most:
                        break
                    try:
                        r = json.loads(line)
                    except Exception:  # noqa: BLE001 - a file still being written
                        break
                    turns = (r.get("mem_in") or {}).get("turns") or []
                    if not turns:
                        continue
                    last = turns[-1]
                    key = (last.get("user"), last.get("bot"), last.get("module"))
                    if not isinstance(last.get("bot"), str) or key in seen:
                        continue
                    seen.add(key)
                    memories.append([{k: t.get(k) for k in ("user", "bot", "module")}
                                     for t in turns[-4:]])
        except EOFError:
            pass

    run_python.pin_clock("2026-11-17")
    sys.path.insert(0, os.path.abspath(run_python.PYBOT))
    from bot import runtime  # noqa: F401 - imports everything
    from bot.compose import why
    import random

    out = open(os.path.join(a.out, "calls.jsonl"), "w")
    kept = 0

    def call(name, *args):
        nonlocal kept
        raised, res = None, None
        try:
            res = getattr(why, name)(*args)
        except Exception as e:  # noqa: BLE001
            raised = type(e).__name__
        rec = {"fn": f"bot.compose.why:{name}", "args": [plain(encode(x)) for x in args],
               "result": plain(encode(res)), "raised": raised}
        out.write(json.dumps(rec, ensure_ascii=False) + "\n")
        kept += 1
        return res

    def halves(s):
        for m in why.CLAUSE_END.finditer(s):
            yield s[:m.start()], s[m.end():]

    for turns in memories:
        last = turns[-1]
        said = last["bot"]
        for strict in (False, True):
            call("_a_reason_in", said, strict)
            call("_reason_in", last, strict)
        reason = call("_reason_in", last, False)
        call("_named_as_a_sentence", last, reason or "")
        for name in ("_a_shorter_way", "_not_shortened", "_first_sentence", "_answered_nothing",
                     "_named_by_definition", "_the_class_it_belongs_to", "_without_examples",
                     "_the_route_is_in_it", "is_a_reason_line", "_end_a_sentence"):
            call(name, said)
        squashed = " ".join(said.split())
        call("_past_a_lead_in", squashed)
        for name in ("_gave_a_method", "_a_cause_named_outright", "_an_answer_to_act_on",
                     "_reads_a_fault"):
            call(name, last)
        first = why._first_sentence(squashed)
        for name in ("_a_clause_of", "_the_doing_of"):
            call(name, first or squashed)
        short = why._a_shorter_way(said)
        for lead in why.REPEAT_LEADS[:2]:
            call("is_the_answer_shorter", lead + (short or said), said)
        sentences = re.split(r"(?<=[.!?])\s+", said)[:why.FIRST_SENTENCES]
        for number, sentence in enumerate(sentences):
            call("_trigger_in", sentence)
            call("_is_the_answer_itself", said, sentence)
            parts = [sentence]
            for mark in why.PARTS:
                parts = [piece for part in parts for piece in part.split(mark)]
            for part in parts:
                for ok in (False, True):
                    clause = call("_cause_clause", part, number, ok)
                    if clause:
                        call("_carries_the_fix", sentence, clause)
                        call("_inside_a_quotation", said, clause)
            for before, after in halves(sentence.strip()):
                call("_the_half_that_does_it", before, after)
                for half in (before, after):
                    for name in ("_carries_a_verb", "_leans_back", "_opens_a_doing",
                                 "_needs_its_subject", "_says_how_it_works",
                                 "_announces_the_list", "_the_reason", "_opens_on_a_figure",
                                 "_names_no_particular", "_points_at_what_follows",
                                 "_brackets_closed"):
                        call(name, half)
        for cut in range(1, len(turns) + 1):
            memory = {"turns": turns[:cut]}
            call("_last_turn", memory)
            call("_said_the_reason_cannot_be_seen", memory)
            call("_about_someone_elses_decision", memory)
            call("_the_suggestion_was_theirs", memory)
        memory = {"turns": turns}
        for clean in FOLLOW_UPS:
            random.seed(0)
            call("respond", clean, memory, {}, False)
            call("respond", clean, memory, {}, True)
            call("respond_to_a_saying_ask", clean, memory, {}, False)
            call("takes_as_it_comes", clean, memory)
            call("said_again", clean, memory)
            call("_is_bare_why", clean)
            call("asks_about_the_last_answer", clean)
        if reason:
            for frame in why.WHY_NAMED[:3] + why.WHY_NAMED_BECAUSE[:2]:
                call("_in_frame", frame, reason)
    out.close()
    open(os.path.join(a.out, "objs.jsonl"), "w").close()
    print(f"{len(memories)} answers probed; {kept} calls", file=sys.stderr)


if __name__ == "__main__":
    main()
