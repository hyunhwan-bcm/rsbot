"""MOD3: reference values of the arbiter's own readings on the lines and candidates of a turn oracle.

    python3 harness/mod3_arbiter_ref.py ORACLE.jsonl.gz OUT.jsonl LIMIT
Read by the ignored test `modules::arbiter::tests::arbiter_readings_against_the_reference`
(ARBITER_REF, ARBITER_INPUT). About 50 KB per turn: keep LIMIT in the hundreds.
"""
import sys, os, json, gzip, copy
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(HERE)), 'harness'))
import run_python
run_python.pin_clock("2026-11-17")
sys.path.insert(0, os.path.abspath(run_python.PYBOT))
from bot import runtime  # noqa
from bot.modules import arbiter as A

def norm(v):
    if isinstance(v, (set, frozenset)):
        return sorted(norm(x) for x in v)
    if isinstance(v, (tuple, list)):
        return [norm(x) for x in v]
    return v

path, out_path, limit = sys.argv[1], sys.argv[2], int(sys.argv[3])
out = open(out_path, 'w')
n = 0
with gzip.open(path, 'rt') as f:
    for idx, line in enumerate(f):
        rec = json.loads(line)
        mem = dict(rec['mem_in'])
        for o in rec['observe']:
            mem.update(o['diff']['set'])
            for k in o['diff']['del']:
                mem.pop(k, None)
        cands = [{"module": r["module"], "priority": r["priority"], "reply": r["reply"]}
                 for r in rec['respond'] if isinstance(r.get('reply'), str) and r['reply'].strip()]
        if len(cands) < 2:
            continue
        n += 1
        if n > limit:
            break
        text = rec['text']
        last = A._last_reply(mem)

        def call(name, *args):
            real = [copy.deepcopy(mem) if a == 'MEM' else (copy.deepcopy(cands) if a == 'CANDS' else a) for a in args]
            if name == '_the_candidates_that_hand_the_detail_back':
                real[0] = set(real[0])
            try:
                res, err = norm(getattr(A, name)(*real)), None
            except Exception as e:  # noqa
                res, err = None, type(e).__name__
            out.write(json.dumps({"rec": idx, "f": name, "args": args, "out": res, "err": err}) + "\n")

        replies = [c['reply'] for c in cands]
        words = sorted(set(A.WORD.findall(text.lower())))
        for name in ('_the_ask_after_the_facts', '_a_refusal', '_a_line_asking_for_plainer_words',
                     '_a_report_and_no_ask', '_the_words_the_report_names', '_names_a_figure',
                     '_the_store_reads_an_explain_again_ask', '_subject_words', '_the_subject_words',
                     '_the_ask_clause', '_content', '_the_numbers_in', '_label', '_read_topic'):
            call(name, text)
        call('_amends_a_detail', text, 'MEM')
        call('_the_lines_own_words', text, 'MEM')
        call('_said_before', 'MEM', text)
        for name in ('_carried_words', '_line_topics', '_named_topics', '_only_acknowledges', '_a_presence_check',
                     '_a_bare_yes_no', '_an_answer_behind_the_ask', '_the_figures_they_gave', '_the_answer_written_out'):
            call(name, 'MEM')
        call('_about', text, 'MEM')
        call('_the_word_the_line_added', text, last)
        report = list(A._the_words_the_report_names(text))
        for r in replies:
            for name in ('_content', '_announces_a_repeat', '_points_back', '_the_quoted_repeat', '_asks_it_back',
                         '_a_condolence', '_places_the_figure', '_explains_the_method',
                         '_says_what_they_already_said', '_the_numbers_in', '_subject_words',
                         '_a_clause_was_cut_in_half', '_asks'):
                call(name, r)
            for ann in (False, True):
                call('_echoes', r, last, ann)
            for name in ('_the_answer_again_with_a_tail', '_says_nothing_new', '_says_the_answer_again',
                         '_says_the_same_thing_again'):
                call(name, r, last)
            call('_only_says_the_line_back', A._content(r), text)
            call('_says_the_subject', r, A._content(text).lower())
            call('_about_the_line', r, words, '')
            call('_says_only_the_line', r, words)
            call('_is_advice_about', r, report)
            call('_how_much_it_names', r, report)
            call('_names_what_they_said', r, text, 'MEM')
            call('_names_a_word_of', r, sorted(A._the_lines_own_words(text, mem)))
            call('_names_the_answers_figure', r, 'MEM')
            call('_about_the_news', r, 'MEM')
            call('_offers_the_food', r, words[:6])
            call('_says_back_one_of', r, sorted(A._the_figures_they_gave(mem)))
            call('_made_of_the_answer', r, sorted(set(last.split())))
            answer = " ".join(A._the_answer_written_out(mem).split()) or last
            toks = [(m.group(0), m.end()) for m in A.TOKEN.finditer(answer.lower())]
            ws_ = A._content(r).split()
            call('_run_in_the_answer', ws_, toks, answer)
            call('_carries_on', ws_, ws_[1:4])
            call('_a_run_of', last.split(), ws_[2:9])
            for w in words[:4]:
                call('_names_the_thing', w, r, words)
                call('_names_the_word', r, w)
                for x in A.WORD.findall(r.lower())[:6]:
                    for name in ('_one_word', '_the_same_plain_word', '_the_same_word', '_shares_a_stem'):
                        call(name, w, x)
        for w in words[:6]:
            call('_the_forms_of', w)
        for c in range(len(cands)):
            call('_a_card_besides', 'CANDS', c)
            call('_a_fresh_answer_besides', 'CANDS', c, last)
            call('_a_card_names_the_line', 'CANDS', c, text, 'MEM')
            call('_every_real_answer_offers', words[0] if words else 'x', 'CANDS', c)
        real = [i for i, c in enumerate(cands) if A._priority(c) >= 0] or [0]
        call('_real_answers_besides', 'CANDS', last)
        call('_the_candidates_that_answer_the_ask', text, 'CANDS', real)
        call('_the_candidates_with_a_figure', text, 'MEM', 'CANDS', real)
        call('_the_advice_over_a_definition', 'CANDS', real, text, real[0])
        call('_the_answer_that_names_what_the_line_added', 'CANDS', real, text, last)
        call('_the_cause_asked_for', 'CANDS', real, text, real[0])
        call('_the_verdict_back', 'CANDS', last, 'MEM')
        call('_the_verdict_turn', 'CANDS', sorted(A._verdict_lines()))
        call('_a_lead_line', 'CANDS', ['The thing to do first: {x}', 'After that: {x}'], last)
        call('_a_shorter_said_again', 'CANDS', last)
        call('_a_whole_line_module_spoke', 'CANDS')
        call('_a_knowledge_entry_has_it', 'CANDS')
        call('_seed', 'CANDS')
        call('_the_candidates_that_hand_the_detail_back', sorted(A._amends_a_detail(text, mem)) or ['may'], 'CANDS', real)
        own, carried, ws2, clean = A._about(text, mem)
        for c in cands:
            call('_match', c['module'], sorted(own), sorted(carried), sorted(ws2), clean)
            call('_declares_topics', c['module'])
            call('_declares_instead', c['module'])
            call('_declared', c['module'])
