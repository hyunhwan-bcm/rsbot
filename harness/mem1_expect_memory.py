import json, os, sys, copy, inspect
sys.path.insert(0, "/Volumes/SSD/Workspace/rsbot/harness")
import run_python
run_python.pin_clock("2026-11-17")
sys.path.insert(0, os.path.abspath(run_python.PYBOT))
from bot import runtime  # noqa
from export_data import encode
from plain import plain
from bot.memory import store_replies as sr

def T(u, b="", m=None): return {"user": u, "bot": b, "module": m}
roast = [T("how long do i roast a chicken", "20 minutes per 500g plus 20 minutes at 190C, so a 2kg bird takes roughly 1 hour 40 minutes.", "answers")]
rice = [T("how much rice for 4 people", "About 75 g of dry rice each, so 300 g for four.", "answers")]
cases = [
 ("_the_people_that_size_feeds", ["is that enough for 6", {"turns": roast}]),
 ("_the_people_that_size_feeds", ["its 2kg, enough for 3", {"turns": roast}]),
 ("_the_people_that_size_feeds", ["will it feed a dozen", {"turns": roast + [T("its 2kg", "ok")]}]),
 ("_the_amount_they_made", ["i only made 200g", {"turns": rice}]),
 ("_the_amount_they_made", ["i only made 400g", {"turns": rice}]),
 ("_the_figure_for_the_size_they_named", ["its about 2kg", {"turns": roast}]),
 ("_the_figure_for_the_size_they_named", ["its about 1.5kg", {"turns": [T("how long do i roast a chicken", "20 minutes per 500g, so a 2kg bird takes roughly 1 hour 40 minutes, and a 1.5kg one takes 1 hour 20 minutes.", "answers")]}]),
 ("_vary", [["a", "b", "c"], {"turns": [T("x", "a"), T("y", "b")]}]),
 ("_vary", [["a", "b"], {"turns": [T("x", "a"), T("y", "b")]}]),
 ("_recent_answers", [{"turns": [T("1", "one"), T("2", ""), T("3", "three"), T("4", "four"), T("5", " five ")]}, 3]),
 ("_the_answer_already_gives", ["money", {"turns": [T("ive got a dentist appointment", "An NHS check-up is £27.40 in England.", "answers")]}]),
 ("_the_answer_already_gives", ["money", {"turns": [T("x", "How much is it?", "answers")]}]),
 ("_the_plan_was_just_said_back", [{"turns": [T("sorry i meant tuesday", "Got it - you begin your new job on Tuesday.")]}, "i begin my new job on tuesday"]),
 ("_a_check_came_before", [{"turns": [T("did you get that", "Yes"), T("did you get that", "")]}, "did you get that"]),
 ("_the_line_before_the_check", [{"turns": [T("my boiler is broken", "Oh no"), T("did you get that", "")]}, "did you get that"]),
 ("_a_check_choice", [["Yes — I've got it.", "I heard you, and I'm still here.", "Yes, that came through."], {"turns": [T("a", "Yes — I've got it.")]}, 7]),
 ("_held_thread", [{"turns": [T("x", "Please call an ambulance now")]}]),
 ("_gentle_subject", [{"turns": [T("my mum died", "I'm so sorry", "grief")]}]),
 ("_last_answer", [{"turns": [T("q", "an answer", "answers")]}]),
 ("_last_answer", [{"turns": [T("q", "an answer", "chat")]}]),
 ("_how_far_along", [{"user": {"weeks": 12}}, "how far along am i"]),
 ("_how_far_along", [{"user": {"pregnant": True}}, "how many weeks am i"]),
 ("_how_far_along", [{"others": [{"who": "wife", "weeks": 8}]}, "how far along is my wife"]),
 ("_a_length_i_have", [{"others": [{"who": "wife", "weeks": 8}]}, "how far along is she"]),
 ("_kids_reply", [{"user": {"kids": 0}}, "how many kids do i have"]),
 ("_kids_reply", [{"user": {"kids": 3}}, "how many kids do i have"]),
 ("_kids_reply", [{"user": {"kids": 2}, "turns": [T("a", "b")]}, "how many kids do i have"]),
 ("_kids_reply", [{"user": {"kids": 1}}, "how many children do we have"]),
 ("_kids_reply", [{"others": [{"who": "son", "age": 4}]}, "how many kids do i have"]),
 ("_kids_reply", [{"user": {"people": ["daughter"]}}, "how many kids do i have"]),
 ("_kids_reply", [{}, "how many kids do i have"]),
 ("_weight_reply", [{"user": {"weight": "90 kilos"}}]),
 ("_weight_reply", [{"user": {"weight": "90 kilos"}, "turns": [T("x", "You weigh 90 kilos.")]}]),
 ("_height_reply", [{"user": {}}]),
 ("_the_fact_on_record", [{"user": {"diet": "vegan", "allergies": []}}]),
 ("_the_person_last_given_a_fact", [{"others": [{"who": "wife", "allergies": ["nuts"]}, {"who": "son", "age": 3}]}]),
 ("_food_facts", [{"user": {"allergies": "nuts", "condition": ["coeliac", "asthma"]}}]),
 ("_what_a_dinner_has_to_leave_out", [{"user": {"allergies": ["nuts", "eggs", "milk"], "condition": "coeliac"}}]),
 ("_age_compare", [{"user": {"age": 34}, "others": [{"who": "wife", "age": 32}]}, "who is older, me or my wife"]),
 ("_age_compare", [{"user": {"age": 34}, "others": [{"who": "wife", "age": 32}]}, "how much older am i than my wife"]),
 ("_age_compare", [{"others": [{"who": "son", "age": 7}, {"who": "daughter", "age": 4}]}, "who is older"]),
 ("_age_compare", [{"others": [{"who": "son", "age": 7}, {"who": "daughter", "age": 7}]}, "which one is older"]),
 ("_age_compare", [{"user": {"age": 34}, "others": [{"who": "wife", "age": 32}]}, "is my car older than my wife"]),
 ("_age_compare", [{"others": [{"who": "son", "age": 7}, {"who": "brother", "age": 9}]}, "how much older is he"]),
 ("_can_close", ["i dont want to talk about it", {"turns": [T("my boiler broke", "That's a pain")]}]),
 ("_can_close", ["i dont want to talk about it", {"turns": []}]),
 ("_nobody_else_kept", [{"others": []}]),
 ("_a_bare_limit_ask", ["is she vegan", {}]),
 ("_a_bare_limit_ask", ["is she vegan, shes coming for dinner", {}]),
 ("_name_fixed_this_turn", ["no shes called dana", {"name_fixed": {"said": "no shes called dana", "name": "dana", "from": "dave"}}]),
 ("_name_fixed_this_turn", ["no shes called dana", {"name_fixed": {"said": "no shes called dana", "name": "dana"}}]),
 ("_correction_said", [{"key": "budget", "value": "500 pounds"}, {}]),
 ("_correction_said", [{"key": None, "value": "porto"}, {}]),
 ("_correction_said", [{"key": "allergies", "value": "nuts", "dropped": True}, {}]),
 ("_correction_said", [{"key": "plan", "value": "june", "wrong": "may"}, {}]),
 ("_correction_said", [{"note": "I'll keep your mum off those."}, {}]),
 ("_correction_said", [{"key": "pet", "value": "max", "animal": "cat"}, {}]),
 ("_remember_asked", [{"turns": [T("a", "b")]}, "name"]),
 ("_slot_words", ["pets", {"user": {"pets": [{"animal": "dog"}, {"animal": "cat"}]}}]),
 ("_slot_words", ["pets", {"user": {"pets": [{"animal": "dog"}, {"animal": "dog"}]}}]),
 ("_a_rule_on_eating", ["so do i eat meat", {"user": {"diet": "vegetarian"}}]),
 ("_a_rule_on_eating", ["do i drink beer", {"user": {"diet": "no alcohol"}}]),
 ("_a_rule_on_eating", ["do i eat bread", {"user": {"condition": ["coeliac"]}}]),
 ("_a_rule_on_eating", ["do i eat peanuts", {"user": {"allergies": ["peanuts"]}}]),
 ("_a_rule_on_eating", ["do i eat meat", {"user": {"no_meat": True}}]),
 ("_the_line_before_is_their_own_animal", [{"turns": [T("we have two dogs, rufus and nell"), T("what do they eat")]}, 3]),
 ("_the_line_before_is_their_own_animal", [{"turns": [T("my neighbour has two cats")]}, 3]),
 ("_still_holding", [{"user": {"pets": [{"animal": "Cat "}]}}, ["cat"]]),
 ("_the_limits_on", [{"allergies": ["nuts"], "diet": "", "condition": []}]),
 ("_the_size_on_the_screen", [[T("is it 3 lbs"), T("how long", "so a 2.5 kg bird takes")]]),
 ("_old_values", [{"facts_log": [{"text": "im going to lisbon in may", "key": "plan", "value": "Lisbon in may"}]}, "im going to lisbon in may"]),
 ("_a_whole_answer_to_hand_back", [{"turns": [T("how do i boil an egg", "Bring a pan of water to the boil and lower the egg in for six minutes for a soft yolk.", "answers")]}, "in full"]),
 ("_said_lines" if False else "_hearing_again", [[T("how do i boil an egg", "Bring a pan of water to the boil and lower the egg in.", "answers"), T("thanks", "Glad that helped.", "chat")], "say that again"]),
]
out = []
for name, args in cases:
    f = getattr(sr, name)
    sig = inspect.signature(f)
    params = list(sig.parameters)
    mem_at = params.index("memory") if "memory" in params else None
    args = copy.deepcopy(args)
    bound = sig.bind(*args); bound.apply_defaults()
    values = list(bound.arguments.values())
    memory = values[mem_at] if mem_at is not None else None
    before = plain(encode(memory)) if isinstance(memory, dict) else None
    plain_args = [None if i == mem_at else plain(encode(x)) for i, x in enumerate(values)]
    try:
        r = f(*values)
        if isinstance(r, (set, frozenset)): r = sorted(r)
        res, raised = plain(encode(r)), None
    except Exception as e:
        res, raised = None, type(e).__name__
    after = {k: plain(encode(v)) for k, v in memory.items() if k not in ("understood", "turns")} if isinstance(memory, dict) else None
    out.append({"fn": name, "args": plain_args, "kwargs": {}, "memory": [mem_at, before] if mem_at is not None else None,
                "result": res, "raised": raised, "after": after})
for o in out: print(json.dumps(o, ensure_ascii=False))
