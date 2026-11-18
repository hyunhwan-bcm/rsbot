"""Reference values for the MOD4 modules' pure readers, straight from PyBot.

    python3 harness/mod4_cases.py > src/modules/mod4_cases.json

Each case is {"fn": "<module>.<function>", "args": [...], "want": <PyBot's value>}; the Rust test
`modules::mod4_tests` calls the port of the same function on the same arguments. The lines are
the branches the turn oracles do not reach (a module that answers a handful of shapes is silent
on almost every corpus line).
"""
import importlib, json, os, random, sys

HERE = os.path.dirname(os.path.abspath(__file__))
for root in (HERE, os.path.join(HERE, "..", "..", "rsbot", "harness")):
    if os.path.exists(os.path.join(root, "run_python.py")):
        sys.path.insert(0, root)
        break
import run_python  # noqa: E402

run_python.pin_clock("2026-11-17")
sys.path.insert(0, os.path.abspath(run_python.PYBOT))
from bot import runtime  # noqa: E402,F401 - import everything, as at run time

M = "bot.modules."
CASES = {
    "particles.a_thing_without_its_tail": [["your dress taken up"], ["my car being washed"], ["taken up"], ["the hedge cut back."], [""]],
    "particles.a_job_word": [["taken"], ["Up!"], ["dress"], ["(washing)"], [""]],
    "memory_store._the_unit_asked": [["how many weeks"], ["how much a month"], ["how much"], ["how many was it"], ["how long"],
                                     ["and how long will the drive take"], ["so how many minutes and hours"], ["how much water"], ["what time"],
                                     ["how far in miles"], ["how much per week roughly"]],
    "memory_store._budget_sign": [["ive got £60 to spend", "60"], ["i have $1,200 or so", "1,200"], ["60 pounds", "60"], ["its € 45 all in", "45"], ["£60", ""]],
    "memory_store._names_this": [["Peanuts", "peanut"], ["nut", "nuts"], ["nuts", "nuts"], ["tree nuts", "nuts"], ["glass", "glas"]],
    "memory_store._plan_names": [["Lisbon in may", "lisbon"], ["my exams in june", "exam"], ["the gym tomorrow", "gyms"], ["Rome", "paris"]],
    "memory_store._without_a_lead_in": [["and also oh fred is 5"], ["no, sorry, he is 6"], ["well well well well well ok"], ["fred is 5"], ["   "]],
    "memory_store._the_ask_behind_the_recall": [["what did i say her birthday was"], ["what did i say my dog was called"], ["how old did i say she was"],
                                                ["How old did I say my son was?"], ["what did i say about the cake"], ["what did we tell you his name is?"],
                                                ["what did i say my budget was"], ["when did i say our  address is"], ["and what did i call the time"],
                                                ["what did i say his age was"]],
    "memory_store._a_fact_word": [["what did i say about my job"], ["remember what i said about the boiler"], ["about"], ["what about my Dog and cat"]],
    "memory_store._names_nothing_in_a_recall": [[" about it"], [" about the boiler"], [""], [" that's what i said"]],
    "memory_store._asks_about_someone_else": [["is he allergic to peanuts"], ["Does  my mum eat meat she is vegetarian"], ["is it expensive"], ["my son is 5"], ["...can they come"]],
    "memory_store._another_animal_is_named": [["no my OTHER cat is called max"], ["no the cat is called max"], ["we also have a cat"]],
    "memory_store._the_money_figures": [["Repair it if the phone is under four years old: £50 to £120 at a local shop, or 40 pounds if you do it."],
                                        ["If it happens again pay £20; otherwise nothing."], ["About 10 to 15 percent, or 20% when the service was good."],
                                        ["Nothing to pay."], ["It costs £60 to £120, and £15 whenever you call them out."]],
    "memory_store._the_figures_in": [["Eight to ten weeks is a realistic build. Run three times a week for 20 minutes at a time.", ["week"]],
                                     ["The 5 to 6 minute egg is runny; boil for about 7 minutes.", ["minute"]],
                                     ["Give it a week, then another 2 weeks 3 days if it is slow.", ["week", "day"]],
                                     ["For the first two days rest it; after 6 weeks it is healed.", ["day", "week"]],
                                     ["It is £50 to £120 at a shop.", ["money"]],
                                     ["Around 3 months per year, or up to 12 months.", ["month"]]],
    "memory_store._a_figure_of_those_units": [["£4,000 at £300 a month is 14 months", ["month"]], ["no length here", ["week"]], ["it is 20% off", ["money"]],
                                              ["ten to twelve Weeks", ["week"]], ["about a week", ["week"]]],
    "memory_store._a_person": [["my wife"], ["the sisters"], ["peanuts"], [None], ["Mum's"]],
    "memory_store._the_clause": [["one. two 5 weeks; three", 9, 16], ["no marks 5 weeks here", 9, 16], ["a: b. c 5 weeks", 8, 15]],
    "bulky._group": [["the old carpet tiles"], ["My Sofa and my fridge"], ["a shed"], ["nothing here"]],
    "bulky._reply": [["floor", "carpet"], ["furniture", "sofa"], ["electrical", "fridge"], ["garden", "shed"], ["", "thing"]],
    "bulky.respond": [["what do i do with my old carpet", {}], ["we are having the carpet replaced", {}], ["how do i get rid of an old fridge", {}],
                      ["how do i get rid of asbestos sheets", {}], ["we are getting the boiler replaced", {}], ["where can i take the sofa", {}],
                      ["how do i get rid of it", {"turns": [{"user": "my mattress is ruined", "bot": "ok"}]}], ["can i recycle a broken trampoline", {}],
                      ["how do i clean and get rid of the smell in a sofa", {}]],
    "a_figure_of_a_dish.respond": [["how many calories in my lasagne", {}], ["how much sugar is in a banana", {}], ["How many grams of protein per portion?", {}], ["", {}],
                                   ["how many carbs for the stew i made", {}]],
    "record_due._the_record": [["where is my asos parcel"], ["my order of the words"], ["has my library book come back"], ["my ordering"], ["our tracking and my delivery that came"]],
    "record_due.respond": [["where is my parcel", {}], ["my parcel still hasnt moved in three days", {}], ["my order never turned up again", {}],
                           ["when is my library book due", {}], ["my library book is late", {}], ["i have lost my library card", {}], ["is my library card expired", {}],
                           ["can you pick up my parcel from the shop", {}], ["how do i return my order", {}], ["i like my parcel", {}],
                           ["where is my parcel", {"turns": [{"user": "x", "bot": "The tracking link in the confirmation email or the shop's app has the last scan, and if it has not moved for a day or two it is the shop to chase - they open the case with the courier, not you. I can't see the tracking from here."}]}]],
    "live_source.respond": [["when is the next bank holiday", {}], ["what are the term dates for my sons school?", {}], ["how long is the wait at a&e", {}],
                            ["is the queue at the airport bad today?", {}], ["did i win the lottery", {}], ["have my premium bonds won this month?", {}],
                            ["has my raffle ticket come up", {}], ["when is the bin collection on a bank holiday", {}], ["bank holiday", {}],
                            ["ok so when do the school holidays start", {}], ["what should we do in the school holidays", {}], ["did my ticket win", {}]],
    "live_source._without_lead_in": [["ok so and when is it"], ["well"], ["oh ok"], ["when is it"]],
    "a_letter_about_money.respond": [["i got a letter about an overpayment of benefits", {}], ["i recieved a bill for 80 pounds from the council tax", {}],
                                     ["my tax bill still hasnt arrived", {}], ["i need to write a letter about my rent", {}], ["i got a letter from my nan", {}],
                                     ["a letter came about my rent arrears yesterday for 300 pounds", {}], ["we had a notice about the debt today", {}],
                                     ["im still waiting for the letter about my pension", {}], ["", {}], ["the bill came about the mortgage", {}]],
    "a_letter_about_money._ask_for_what_is_missing": [["Tell me the amount and the dates on it and I'll help you check it.", "bill for 80 pounds", "bill"],
                                                       ["What is it for, and how much? Tell me the amount and the dates on it and I'll help you check it.", "came today", "bill"],
                                                       ["What does it say the amount is? What is it for, and how much?", "80 pounds on monday", "letter"],
                                                       ["What is it for, and how much?", "nothing", "bill"]],
    "judge_ask.reads_it": [["do you judge people"], ["do you judge people by their past"], ["dont judge the cake by its look"], ["my mum would judge me"],
                           ["would you judge someone for that"], ["judging by the weather do you judge people"], ["do you judge a person by their past"], [""]],
    "judge_ask._says_nothing": [[""], ["Got it. Is there anything you'd like a hand with?"], ["I try not to."], ["Noted: fine"]],
    "what_they_said._details": [["what did the vet say about the dog's leg"], ["I told them"], [""]],
    "what_they_said._the_kind_of_answer": [["what did they quote me"], ["what did the vet say"], ["how much did she charge"]],
    "what_they_said._the_reported_clause": [["i told the garage about the noise and they said bring it in tuesday."], ["i told my boss i was quitting"],
                                            ["the vet asked for 60"], ["she said what time is it"], ["they said that it would be fine, probably, and then went on and on and on and on and on and on about it forever"],
                                            ["nothing reported here"], ["mum reckoned ..."]],
    "what_they_said._the_figure_the_line_gave": [["we got quoted 400 for the fence"], ["the car is 3 years old and they said bring it in tuesday"],
                                                 ["they said £400 or £450"], ["it was 3 grand for 2 doors"], ["no figure"], ["they wanted 20 and 30"]],
    "words._quoted": [["Ask once: 'I'm sorting my budget, when can you pay me back?' and leave it."], ["Send a 'saw this and thought of you' or a photo."],
                      ["Say \"Hi - the boiler has stopped working, please let me know.\" to them."], ["Just say 'no'."], ["Try “I missed hearing from you on my birthday” first."],
                      ["a note like 'thinking of you today, hope it goes well.' works"], ["nothing quoted"]],
    "words._names_its_own": [["what should i say to him"], ["how do i say sorry"], ["how do i say it without a row"], ["what do i text my boss please"],
                             ["how should i put this"], ["what should i say"], ["how do i bring it up"], ["no ask here"], ["how do i word it, then, out loud"]],
    "hardpast._the_hard_thing": [["my dad was an alcoholic", {}], ["the prison was built in 1800", {}], ["we are getting a divorce", {}],
                                 ["i went to prison years ago but i have a job now", {}], ["i was homeless for a while", {}], ["was your dad an alcoholic?", {}],
                                 ["my mum's partner was abusive back then", {}], ["i saw a documentary about prison when i was young", {}],
                                 ["my wife had a miscarriage last year", {}], ["i was in a bad crash years back", {}],
                                 ["my dad was an alcoholic", {"understood": {"kind": "question"}}]],
    "how_long_a_stay._the_opening": [["Ten minutes. Then rest."], ["no end here"], ["Really? Yes"]],
    "how_long_a_stay._a_subject_of_its_own": [["how long will it take"], ["how long does the drive take"], ["how many days is the visit 3"], ["how long should she stay"]],
    "how_long_a_stay._the_length_said": [[{"turns": [{"user": "my mum is coming to stay for four days", "bot": "ok"}]}],
                                         [{"turns": [{"user": "my sister is visiting us for the weekend", "bot": "ok"}]}],
                                         [{"turns": [{"user": "he is staying with us a fortnight", "bot": "ok"}]}],
                                         [{"turns": [{"user": "my mum is coming next week", "bot": "ok"}]}], [{"turns": []}]],
    "familyask.the_relation": [["do you have a mum and dad"], ["how are your parents"], ["how many brothers and sisters do you have"], ["whats your favourite colour"],
                               ["is your mum, or your nan, alive"]],
    "familyask.the_class": [["do you have a mum"], ["do you have any kids"], ["how is your family"], ["hello"]],
    "familyask.is_a_family_ask": [["do you have a mum and dad"], ["your mum"], ["how was your family christmas?"], ["is my mum like your mum"], [""],
                                  ["have you got any brothers"], ["would you rather have a brother or a sister"]],
    "thing_i_did_with._thing": [["what coat did i order"], ["which coat was it i ordered"], ["what size coat did i order"], ["which one was it i ordered"],
                                ["what did i say"], ["and which colour paint was it that i bought?"], ["what size did i order"]],
    "thing_i_did_with._did": [["what coat did i order"], ["which coat was it that i ordered"], ["hello"]],
    "thing_i_did_with._stem": [["coats"], ["ordering"], ["ordered"], ["boxes"], ["bus"], ["'dress'"], ["glasses"]],
    "a_fact_asked_again._the_date_in": [["flying on the 12th of june"], ["dentist on june 3rd"], ["going next friday"], ["the 4th"], ["rome in May"], ["no date"]],
    "a_fact_asked_again._the_noun_asked": [["what was my goal again"], ["what were my aims again"], ["what am i saving for again"], ["whats the Plan and target again"]],
    "a_fact_asked_again._a_name": [["Ford fiesta by june"], ["ford fiesta"], ["Get to 12"], ["May is the month"], [""]],
    "a_fact_asked_again.respond": [["when is it again", {"user": {"plans": ["flying on the 12th of june"]}}],
                                   ["what was my goal again", {"user": {"plan": "I want to Get to 12 stone."}}],
                                   ["what was my target again", {"user": {"plans": ["Ford Fiesta by june", "SAVE 300"]}}],
                                   ["what do i keep him away from again", {"others": [{"who": "son", "allergies": ["nuts"]}, {"who": "", "allergies": ["", "eggs"]}]}],
                                   ["should i eat less again", {"user": {"plan": "x"}}], ["what was my goal", {"user": {"plan": "x"}}],
                                   ["what day is it again", {"user": {"plans": ["a on monday", "b on tuesday"]}}],
                                   ["what was my aim again", {"user": {"plan": "to Run a 5k"}}]],
    "a_job_that_is_not_a_diy_job._the_job": [["how do i remove a chimney breast"], ["can i fit a gas hob myself"], ["how do i fell a tree"], ["how do i paint a door"],
                                             ["can i replace a broken skylight"]],
    "a_job_that_is_not_a_diy_job._has": [["have a go at it", ["a go at it", "diy"]], ["training", ["rain"]], ["need a2", ["need a"]]],
    "a_shock_at_a_bill._the_ending_and_the_page": [["i nearly died when i saw the bill"], ["i read the invoice and nearly died"], ["i paid the bill and nearly died on the motorway"],
                                                   ["the bill took my breath away"], ["i nearly died"], [""], ["im holding the gas bill"], ["i havent seen the bill yet but i will faint"]],
    "a_shock_at_a_bill._a_complaint_added_after_the_ending": [["i nearly fainted when i saw the bill and now i feel sick"], ["i nearly died when i saw the bill"],
                                                              ["i nearly died when i saw the bill and my knee hurts"], ["i nearly fell over at the bill, my chest is still tight"]],
    "a_shock_at_a_bill._the_ending_is_somebody_elses": [["my mum nearly died when she saw the bill"], ["i nearly died when i saw the bill"], ["the bill"]],
    "a_shock_at_a_bill._a_kind_word_in_front_of_the_page": [["i nearly died when i saw the vet bill"], ["i nearly died when i saw the big bill"], ["i saw my bank statement"],
                                                            ["when the awful dentist invoice arrived i nearly died"], ["no page"]],
    "a_shock_at_a_bill._a_bill_of_another_kind": [["i nearly died when i saw the council tax bill"], ["i nearly died when i saw the bill"], ["i fainted at the vet bill"]],
    "a_shock_at_a_bill._ask_for_what_is_missing": [[{}, "i nearly died when i saw the bill"], [{}, "i nearly died when i saw the bill for £400"],
                                                   [{}, "i havent opened the bill yet"]],
    "a_figure_beside_the_state.a_figure_of_heat": [["its 20 degrees and im freezing"], ["its -3 outside"], ["minus 3 and windy"], ["at 180 for an hour"], ["20c in here"]],
    "a_figure_beside_the_state.without_the_figure": [["its 20 degrees and im freezing"], ["im freezing, its -3 outside"], ["its -3 outside and im freezing"],
                                                     ["only 16 degrees here today so im cold"], ["its about minus 5 degrees"], ["im boiling and its only twenty degrees celsius"]],
    "keeping._seed": [["how long have you got"], [""], ["am i boring you — really?"]],
    "keeping._rotate": [[["a", "b", "c", "d"], "how long have you got"], [["a"], "x"], [["a", "b", "c"], "am i keeping you tail"]],
    "keeping._pick": [[["", "Where were we?", "What were you in the middle of?"], "how long can we talk tail"], [["a", "b"], "z"]],
    "keeping.respond": [["how long have you got", {"understood": {"about_bot_kind": "while"}}],
                        ["how many messages can i send you", {"understood": {"about_bot_kind": "while"}}],
                        ["am i boring you", {"understood": {"about_bot_kind": "keeping"}}],
                        ["am i boring you", {"understood": {"about_bot_kind": "keeping"}, "turns": [{"user": "x", "bot": "You're fine. I'm reading all of it, and I'll ask if I lose the thread."}]}],
                        ["am i boring you", {"understood": {"about_bot_kind": "other"}}], ["am i boring you", {}]],
    "the_wait_about_a_body.is_generic": [["That sounds rough. Do you want to talk?"], ["Same as before: I'm listening"], ["Noted: fine"], ["A real answer."], [""],
                                         ["I’m sorry — that’s frustrating"]],
    "the_wait_about_a_body._words": [["Got it — My sister's MS, our worry!"], [""]],
    "the_wait_about_a_body.said_back": [["Got it - your sister might have ms.", "my sister might have ms"], ["Got it - your sister might have ms. When do you hear?", "my sister might have ms"],
                                        ["Noted: your dog", "My dog!"], ["Your sister might have ms", "my sister might have ms"], ["got it, ", ""]],
    "a_question_about_somebody._flat": [["  How OLD is  he’s  "], [""]],
    "a_question_about_somebody.about_somebody_unnamed": [["how old is he", {}], ["how old is my son", {}], ["what do they do", {}], ["whats their name?", {}],
                                                         ["how old is he", {"turns": [{"user": "my son is here", "bot": "ok"}]}], ["where does she live", {}],
                                                         ["how do you know if they like you", {}], ["he is old", {}], ["is that woman his mum", {}]],
    "dreams._settled": [["i dreamt my dad died", ["a", "b"]], ["", ["a", "b", "c"]], ["café dream", ["a", "b", "c"]]],
    "trip_date_said_again._the_first_clause": [["actually we fly in september, my sister moved her holiday"], ["no, june"], ["one"], [""]],
    "a_day_somebody_else_keeps.respond": [["what time does my sons school finish", {}], ["when do my classes start", {}], ["what time does my daughters bus get home", {}],
                                          ["my son wont eat vegetables", {}], ["when does our swimming lesson end?", {"turns": [{"user": "a", "bot": "b"}]}], ["", {}],
                                          ["what time does my kids swimming finish", {}]],
    "a_day_somebody_else_keeps._choose": [[["a", "b", "c"], {"turns": [{"user": "x", "bot": " b "}, {"user": "y", "bot": "b"}]}, "When?"], [["a"], {"turns": [{"bot": "a"}]}, ""],
                                          [["a", "b", "c", "d"], {}, "what time does school finish"]],
    "safety_phrases._both_here": [[["a", "b"], ["c"]], [["x"]]],
    "the_kind_one_line_up._answered_by_the_fallback": [[{"turns": [{"user": "x", "bot": "y", "module": "greeting"}]}], [{"turns": [{"user": "x", "bot": "y", "module": "answers"}]}],
                                                       [{"turns": [{"user": "x", "bot": "y", "module": None}]}], [{}]],
    "first_line_open._has_a_turn_behind_it": [[{"turns": [{"user": "x", "bot": " "}]}], [{"turns": [{"user": "x", "bot": "hello"}]}], [{}]],
    "doubt._the_question": [["the bank was closed", "bank"], ["", "seal"], ["café seal", "seal"]],
    "confusing._said_back": [["my payslip"], ["our letter to me"], [" mine "], [""]],
    "name_taken_back._note": [["  Thats NOT my   name "]],
    "name_taken_back.respond": [["thats not my name", {"name_taken_back": {"name": "", "said": "thats not my name"}}], ["thats not my name", {"name_taken_back": {"name": "x", "said": "other"}}],
                                ["hello", {}]],
    "the_shape_of_the_answer._the_reply_is_only_a_placeholder": [[""], ["A real answer about boilers."]],
    "talkto._a_route_is_named": [[""], ["Citizens Advice covers this free, and a call beats searching."], ["A real answer."]],
    "their_pastime._a_thing_nobody_is_into": [["drugs"], ["another woman lately"], ["baking"], ["trouble at school"], ["hill-walking"]],
    "their_pastime._a_plural_thing": [["dinosaurs"], ["baking"], ["chess"], ["bowls"], ["cats"], ["the bus"], [""]],
    "their_pastime._it_word": [["trains"], ["tennis"]],
    "their_pastime._watched": [["football"], ["knitting"], ["amateur dramatics"], ["five a side"]],
    "their_pastime._does_pool": [["football"], ["knitting"]],
    "their_pastime._is_a_placeholder": [["  Noted - thanks"], ["Got it. Is there anything"], ["A real answer"]],
    "their_pastime._starts_a_sentence": [["is she staying"], [""]],
    "where_it_answers._content": [["Is there a lift at the station?"], ["can i keep the old sofa", ["keep", "old"]], [""]],
    "said_fact_back._nothing_to_give": [["I'm not sure about that one, sorry — I'd rather not guess."], ["Your sister is vegan."]],
}

def run():
    out = []
    for name, calls in CASES.items():
        module, fn = name.split(".")
        f = getattr(importlib.import_module(M + module), fn)
        for args in calls:
            random.seed(0)
            try:
                want = f(*json.loads(json.dumps(args)))
            except Exception as e:  # noqa: BLE001
                want = {"raised": type(e).__name__}
            if isinstance(want, (set, frozenset)):
                want = sorted(want)
            if hasattr(want, "id") and not isinstance(want, (str, dict, list, tuple)):
                want = {"entry": want.id}
            out.append({"fn": name, "args": args, "want": want})
    return out

if __name__ == "__main__":
    print(json.dumps(run(), ensure_ascii=False, indent=0, default=lambda o: sorted(o) if isinstance(o, (set, frozenset)) else str(o)))
