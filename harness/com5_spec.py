"""COM5 test cases: lines that trigger each reader, run through PyBot by com5_cases.py."""
CASES = []


def case(m, f, args, rs, k, mem=None):
    CASES.append({"m": m, "f": f, "a": args, "rs": rs, "k": k, "mem": mem})


def lines(m, f, texts, k="bool", rs=None):
    for t in texts:
        case(m, f, [t], (rs or f + "({0})").replace("{0}", rs_lit(t)), k)


def rs_lit(s):
    import json
    return json.dumps(s, ensure_ascii=False)


# ---- whole
LONG = ("If the lid is red or the label says it is hazardous, take it to the tip rather than the "
        "kerbside bin, and if it keeps happening, ask the council for a bigger one, because they do that")
for most in (40, 90, 100, 120, 150, 300):
    case("whole", "a_whole_cut", [LONG, most], "a_whole_cut(%s, %d)" % (rs_lit(LONG), most), "str")
W2 = ("Ring the ward first thing in the morning and ask for the nurse in charge of the bay, then say "
      "who you are, unless they already know you, in which case just ask how the night went for him.")
for most in (95, 110, 130):
    case("whole", "a_whole_cut", [W2, most], "a_whole_cut(%s, %d)" % (rs_lit(W2), most), "str")
W3 = ("Keep the receipt somewhere safe and photograph it as well; if the shop asks for it later, "
      "you will have it, and “I lost it” is the one answer they never accept at the till.")
for most in (85, 100):
    case("whole", "a_whole_cut", [W3, most], "a_whole_cut(%s, %d)" % (rs_lit(W3), most), "str")
case("whole", "a_whole_cut", ["  short one  ", 50], 'a_whole_cut("  short one  ", 50)', "str")

# ---- news_ask
lines("news_ask", "reads_as_an_ask_for_the_news", [
    "anything in the news", "the headlines", "news about the strike", "i have news", "good news",
    "any news from 2001", "the news is getting me down", "whats the news today", "news",
    "did you hear the news", "any news about my son", "so any news on the election please"])

# ---- for_the_day
lines("for_the_day", "for_the_ask", ["is it too hot to run", "is it too cold for a walk",
      "will it be too windy later", "is it warm enough out", "too hot to bake a cake",
      "is it cold out there today?"], "opt_pool", "for_the_ask({0}, \"\")")
case("for_the_day", "for_the_ask", ["", "is it too windy to go cycling"],
     'for_the_ask("", "is it too windy to go cycling")', "opt_pool")

# ---- judgement
for t in ["is that a good idea", "is that stupid", "sensible?", "daft", "hello there"]:
    case("judgement", "asks_for_a_judgement", [t], "asks_for_a_judgement(%s, {mem}).unwrap()" % rs_lit(t), "bool")
for t in ["is that a good idea", "daft?", "is it stupid"]:
    case("judgement", "asks_for_a_judgement", [t, "$mem"], "asks_for_a_judgement(%s, {mem}).unwrap()" % rs_lit(t), "bool",
         mem=[["im thinking of quitting my job to travel", "Big step. What's pulling you?", "chat"]])

# ---- breakup
ILL = "I would be guessing, and this is not a thing to guess about. Ask whoever is treating them, or ring 111."
import importlib
fb = importlib.import_module("bot.compose.fallback")
for pool_line in list(fb.UNKNOWN_HOW_IT_GOES_FOR_SOMEONE)[:2] + ["Not a pool line."]:
    for turns in ([["my mum is having a hard time since her and dad broke up", "That sounds hard.", "chat"]],
                  [["my mum broke her hip and is having a hard time", "Oh no.", "chat"]],
                  [["my mum and dad broke up", "Oh.", "chat"]]):
        case("breakup", "answer_for", ["is she going to be ok", pool_line, "$mem"],
             "answer_for(\"is she going to be ok\", %s, {mem})" % rs_lit(pool_line), "opt_str", mem=turns)
case("breakup", "_thread_said", ["$mem", "Is She OK"], '_thread_said({mem}, "Is She OK")', "str",
     mem=[["One", "a", ""], ["Two", "b", ""], ["Three", "c", ""], ["Four", "D", ""]])

# ---- a_count_of_packets
lines("a_count_of_packets", "is_a_dose", [
    "i swallowed two packets of pills", "two packets of paracetamol and i feel sick",
    "i bought two packets of paracetamol", "my dog swallowed two packets of pills",
    "i took two packets of pills from the cupboard", "my bag has two packets of pills in it",
    "we are away this week and i took two packets of pills", "two packets of pills for the trip", ""])

# ---- a_thing_wants_doing
for t in ["the garden is overgrown", "my car is really dirty", "my son is messy", "the meeting was long",
          "im sick of it being dirty", "my hair is long", "the kitchen looks a bit grim",
          "our shed is old", "the spare room is piled up"]:
    case("a_thing_wants_doing", "reads", [t], "reads(%s, {mem})" % rs_lit(t), "opt_str")
    case("a_thing_wants_doing", "pool", [t], "pool(%s, {mem})" % rs_lit(t), "opt_pool")

# ---- a_first_time
for t, turns in [("its my first time doing a speech", None), ("ive never had a curry before", None),
                 ("first time", [["i have my driving test next week", "Good luck.", "chat"]]),
                 ("first time ever", [["going for a walk", "Nice.", "chat"]]),
                 ("ive never lost anyone before", None), ("is it my first time?", None),
                 ("never done one before", [["i have to give a talk at work", "Ok.", "chat"]])]:
    case("a_first_time", "reply_for", [t, "$mem"], "reply_for(%s, {mem})" % rs_lit(t), "opt_str", mem=turns)
    case("a_first_time", "_a_task_they_faced", [t, "$mem"], "_a_task_they_faced(%s, {mem})" % rs_lit(t), "bool", mem=turns)

# ---- facts
lines("facts", "asks_for_a_when", ["when is my dentist appointment", "what time is my train",
      "when should i plant tomatoes", "when is the best time to plant", "so when do we fly"])
for r in ["On Monday at 10am.", "In July.", "At 7.", "Over £150,000 a year.", "The 3rd of May, 2025."]:
    case("facts", "holds_a_date", [r], "holds_a_date(%s)" % rs_lit(r), "bool")
    case("facts", "the_fact_asked_for", ["when is my flight", r], "the_fact_asked_for(\"when is my flight\", %s)" % rs_lit(r), "bool")
for a, b in [("Friday at 3pm.", "Your train is friday."), ("In July.", "July."), ("Monday.", "Tuesday."), ("No date.", "No date.")]:
    case("facts", "the_same_date", [a, b], "the_same_date(%s, &Value::from(%s))" % (rs_lit(a), rs_lit(b)), "bool")

# ---- stepwords
for w in ["learn", "run", "dye", "lie", "be", "quit", "visit", "begin", "swim", "go", "make", "see", "fix", "singing", "ok"]:
    case("stepwords", "_ing", [w], "_ing(%s)" % rs_lit(w), "str")
lines("stepwords", "a_step", ["Learn.", " RUN ", "gym", ""])
for p in ["dye my hair pink", "the gym", "bed early tonight.", "  be 40 next week ", "", "quit smoking"]:
    case("stepwords", "said_after_a_to", [p], "said_after_a_to(%s, &Value::None)" % rs_lit(p), "str")

# ---- doubt
lines("doubt", "reads_as_a_doubt", ["is that bad?", "so should i be worried", "is that bad for the baby",
      "am i ok then", "Is it really serious!", "is that a lot"])
CARE = "Get it checked by a GP within two weeks. Most of these are nothing."
for t, turns in [("is that bad", [["the pharmacist said to see a gp about my mole", CARE, "answers"]]),
                 ("is that bad", [["should i get my mole checked", CARE, "answers"]]),
                 ("is that a lot", [["the nurse told me to get it checked", CARE, "answers"]]),
                 ("should i be worried", [["the nurse told me to get it checked", CARE, "safety"]]),
                 ("is that bad", [["the gp wants me to have it looked at", "Ring 111 if it gets worse.", "answers"]])]:
    case("doubt", "reply_for", [t, "$mem"], "reply_for(%s, {mem})" % rs_lit(t), "opt_str", mem=turns)

# ---- recall
for s, turns in [("You start your new job on monday.", None), ("You're away next week.", None),
                 ("Your dads funeral is on friday.", None), ("You start your new job on monday. And more.", None),
                 ("You start your new job on monday.", [["x", "You start your new job on monday. How are you feeling about it?", "memory"]]),
                 ("Your mum's operation is on the 3rd of December, at the big hospital in town, early.", None)]:
    case("recall", "with_a_tail", [s, "$mem"], "with_a_tail(%s, {mem})" % rs_lit(s), "str", mem=turns)

# ---- pleased
for t in ["what a lovely day", "oh what a beautiful morning!", "what a relief", "such a result", "great news",
          "what a great shame", "what a lovely cake", "what a lovely day?", "what a lovely day it is today honestly"]:
    case("pleased", "answer_for", [t], "answer_for(%s, {mem})" % rs_lit(t), "opt_str")

# ---- nottheirbest
lines("nottheirbest", "reads_as_a_low_day", ["not my best day", "ive had better weeks", "could be better tbh",
      "my wifi could be better", "is today not the best day for a bbq", "things could be better", "never had a better day"])
lines("nottheirbest", "_may_replace", ["Nice one. How did the rest of it go?", "Noted: you had a day.",
      "I'm listening — say more if it helps.", "Ring the GP.", ""])
for t, r in [("not my best day", "Nice one. How did the rest of it go?"), ("not my best day", "Ring the GP."),
             ("all good", "Nice one.")]:
    case("nottheirbest", "answer_for", [t, r], "answer_for(%s, %s, {mem})" % (rs_lit(t), rs_lit(r)), "opt_str")

# ---- moving
lines("moving", "reads_as_a_move", ["were moving house next month", "we're moving into our first flat",
      "im moving out because the landlord is selling up", "should i move out", "our move is next week",
      "i moved house last september", "moving on from it"])
for t in ["were moving house next month", "i moved house last september"]:
    case("moving", "reads_as_a_move_cheered", [t], "reads_as_a_move_cheered(%s)" % rs_lit(t), "bool")
    case("moving", "reply_for", [t], "reply_for(%s, {mem})" % rs_lit(t), "opt_str")

# ---- talktime
for t in ["can we talk tomorrow", "can we chat later tonight please", "keep me company",
          "stay with me tonight", "can we talk tomorrow about my funeral plans", "i want to talk to you at 6pm",
          "can we talk about my mum"]:
    case("talktime", "answer_for", [t], "answer_for(%s, {mem})" % rs_lit(t), "opt_str")

# ---- morning_after
lines("morning_after", "reads_as_a_morning_after", ["i got wasted at the party", "i was so drunk last night",
      "im hungover today", "i was drunk and i said something stupid", "my husband was drunk again last night",
      "im going on a bender on friday", "i got smashed last night and ended up in hospital",
      "i smashed my phone", "heavy night, should i stop drinking", "im hanging the curtains"])

# ---- unseen
lines("unseen", "reads", ["nobody cares", "i feel invisible", "nobody cares about my plants",
      "my friends never invite me", "nobody would notice if i left", "im not invisible", "i feel ignored by my doctor",
      "i felt ignored by the satnav", "nobody listens to me", "my mum says nobody cares about her",
      "my ex doesnt want me back", "she doesnt want me to come", "nobody asks me how i am",
      "i felt ignored by my friends"])

# ---- ward
for t, turns in [("how long will she be in for", [["my mum went into hospital last night", "Oh no.", "chat"]]),
                 ("what ward is my dad on", None), ("how long will i be in hospital", None),
                 ("when can he come home from hospital", None), ("how long until she is back on her feet",
                  [["my nan is home from hospital at last", "Good.", "chat"]]),
                 ("is my dog still at the vets", None), ("when is my mums operation", None),
                 ("where does he work", [["she is on the ward", "Ok.", "chat"]]),
                 ("can you visit the hospital", None), ("what time are visiting hours for him", None)]:
    case("ward", "answer_for", [t, "$mem"], "answer_for(%s, {mem})" % rs_lit(t), "opt_str", mem=turns)
for s in ["she is in", "is he ok", "they are in", "i am in"]:
    case("ward", "_whom", [s], "_whom(%s)" % rs_lit(s), "str")

# ---- outlook
lines("outlook", "points_at_the_thread", ["will it get better", "does it always feel like this",
      "how long till i feel better", "what happens now", "my knee hurts"])
for t in ["will it get better", "does it stop the itching", "will it stop soon", "will it stop bleeding",
          "is it normal", "will things ever change"]:
    case("outlook", "asks_the_outlook", [t, t], "asks_the_outlook(%s, %s)" % (rs_lit(t), rs_lit(t)), "bool")

# ---- toknow
for t in ["anything i should know about the job offer", "is there anything i should know about the wedding",
          "what should i know about it", "anything i should know about cats before getting one", "hello"]:
    case("toknow", "_the_thing_asked_about", [t], "_the_thing_asked_about(%s)" % rs_lit(t), "str")

# ---- circumstance
for t in ["im a carer", "we rent", "I'm a student, tbh", "im a single mum lol", "i rent and the boiler is broken",
          "we own our house", "im on universal credit", "am i a carer?", "I’m self-employed."]:
    case("circumstance", "reads_as_situation", [t], "reads_as_situation(%s)" % rs_lit(t), "opt_str")
    case("circumstance", "pool", [t], "pool(%s)" % rs_lit(t), "opt_pool")
case("circumstance", "_line", ["  We're renters now anyway lol. "], '_line("  We\'re renters now anyway lol. ")', "str")

# ---- invitation
for t in ["fancy a pint?", "can you come to the party", "are you there", "do you want to know a secret",
          "would you like to come for a walk", "can you come round", "i fancy a curry", "can you come up with a name",
          "shall we watch a film"]:
    case("invitation", "_the_invitation", [t], "_the_invitation(%s)" % rs_lit(t), "opt_pool")

# ---- handoff
for t in ["one more thing", "quick question", "one more question about the boiler", "another thing about it",
          "one more thing about whether to go", "last question", "and one more thing, i need to ask about the car"]:
    case("handoff", "reads_as_handoff_about_a_thing", [t], "reads_as_handoff_about_a_thing(%s)" % rs_lit(t), "opt_str")
    case("handoff", "takes_as_it_comes", [t, "$mem"], "takes_as_it_comes(%s, {mem})" % rs_lit(t), "bool")

# ---- hands
for t in ["can you set a timer for 10 minutes", "put it in my calendar", "can you send this email for me",
          "send the email tomorrow", "play some music", "can you read my emails", "can you google it for me",
          "how do i set a timer", "can you write a text to my mum", "could you print this for me",
          "note that down", "turn the telly up"]:
    case("hands", "reads", [t], "reads(%s)" % rs_lit(t), "opt_str")
    case("hands", "writes_the_words", [t], "writes_the_words(%s)" % rs_lit(t), "bool")
case("hands", "_bare", ["Can't YOU, set it?!"], '_bare("Can\'t YOU, set it?!")', "str")

# ---- fallback_data
for t in ["how much is in my account", "when does my parcel arrive", "do you get bored answering the same things",
          "are you ever tired", "what time is it"]:
    case("fallback_data", "the_place_the_answer_is_kept", [t], "the_place_the_answer_is_kept(%s)" % rs_lit(t), "opt_pool")
    case("fallback_data", "the_bots_own_answer", [t], "the_bots_own_answer(%s)" % rs_lit(t), "opt_pool")

# ---- aboutness
for t in ["im fine really, anyway whats a good way to sell her furniture", "anyway", "by the way my car is broken down again",
          "back to the garden"]:
    case("aboutness", "after_a_change_of_subject", [t], "after_a_change_of_subject(%s)" % rs_lit(t), "str")
for t in ["what is the best way to sell second hand furniture online", "hello there how are you", "My garden's roses keep dying"]:
    case("aboutness", "subject_words", [t], "subject_words(%s)" % rs_lit(t), "set")
for r, t in [("Selling furniture online works best with photos.", "what is the best way to sell second hand furniture online"),
             ("what is the best way to sell furniture", "what is the best way to sell furniture"),
             ("I'm not sure about that one.", "how do i prune roses"), ("Nice weather today.", "how do i prune roses")]:
    case("aboutness", "restates_the_line", [r, t], "restates_the_line(%s, %s)" % (rs_lit(r), rs_lit(t)), "bool")
    case("aboutness", "is_a_hedge", [r], "is_a_hedge(%s)" % rs_lit(r), "bool")

# ---- childwords
for t in ["how do i tell my son", "how do i tell my 4 year old about his dad", "how do i tell my daughter shes going to be a big sister",
          "how do i tell my son his grandad died", "what do i tell the kids"]:
    for turns in ([["my dad died yesterday", "I'm so sorry.", "loss"]], [["my dad killed himself", "I'm so sorry.", "safety"]], None):
        case("childwords", "asks_the_words_for_a_child", [t, "$mem"], "asks_the_words_for_a_child(%s, {mem})" % rs_lit(t), "bool", mem=turns)
for t in ["", " his dad", " shes pregnant", " that grandad died", " please now"]:
    case("childwords", "_the_tail_is_about_the_death", [t], "_the_tail_is_about_the_death(%s)" % rs_lit(t), "bool")

# ---- kept_nearby
for t, turns in [("can i save anything", [["im on benefits", "Ok.", "chat"]]), ("and the kids, do they get one each", [["one bag is 23 kilos", "Ok.", "chat"]]),
                 ("is a backpack ok as hand luggage", None), ("how do i do it", [["im on benefits", "Ok.", "chat"]]),
                 ("can i renew it", [["my passport expires soon", "Ok.", "chat"]]), ("will it be alright", [["i dropped my phone in the sink and the warranty", "Ok.", "chat"]])]:
    case("kept_nearby", "answer_for", [t, "$mem"], "answer_for(%s, {mem})" % rs_lit(t), "opt_pool", mem=turns)
for w in ["my orders are late", "the bus is late", "passports and visas"]:
    case("kept_nearby", "_without_plurals", [w], "_without_plurals(%s)" % rs_lit(w), "str")

# ---- lead_in
for l in ["and is b&q open on sunday", "so what time is it", "and ok", "well, how do i boil an egg", "is it open"]:
    case("lead_in", "_the_rest_after_the_lead_in", [l], "_the_rest_after_the_lead_in(%s)" % rs_lit(l), "str")
for s, l in [("Is b&q open on sunday — noted.", "and is b&q open on sunday"), ("The opening hours change on bank holidays.", "and is b&q open on sunday")]:
    case("lead_in", "_an_echo_of_the_ask", [s, l], "_an_echo_of_the_ask(%s, %s)" % (rs_lit(s), rs_lit(l)), "bool")

# ---- no_advice
for t in ["I wish they would just listen to me and understand my perspective.", "i dont want advice", "i just need to vent",
          "no advice please, what do you think", "i dont want advice but what should i do?", "my mum died, i dont need advice",
          "what advice would you give", "i just want to be heard, is that ok", "can i just talk"]:
    case("no_advice", "pool_name", [t], "pool_name(%s)" % rs_lit(t), "opt_str")
    case("no_advice", "says_a_death", [t], "says_a_death(%s)" % rs_lit(t), "bool")

# ---- no_page
for t in ["how do i handle my mum", "how do i get through christmas", "how do i cope with it all", "how do i deal with my ex when he drops the kids",
          "how do i handle the pain", "how do i manage half term", "how do i fix the boiler", "how can we survive the summer holidays on my own"]:
    case("no_page", "reads", [t], "reads(%s)" % rs_lit(t), "opt_str")
    case("no_page", "reply_for", [t], "reply_for(%s, {mem})" % rs_lit(t), "opt_str")

# ---- onmybehalf
for t in ["book me a table", "google our flight times for me", "send this to my boss", "can you search for cheap flights please",
          "save it", "how do i book a table", "remind me to call mum", "order my prescription"]:
    case("onmybehalf", "_what_was_asked_for", [t], "_what_was_asked_for(%s)" % rs_lit(t), "str")
    case("onmybehalf", "_about_an_errand", [t], "_about_an_errand(%s)" % rs_lit(t), "opt_pool")

# ---- own_ask
for t in ["how do i stop feeling guilty", "whats wrong with me", "how do i stop the panic alarm", "why do i feel so low",
          "how can i forgive myself", "i feel like a failure", "how do i stop feeling anxious about work", "how do i get over it"]:
    case("own_ask", "answer", [t], "answer(%s, {mem})" % rs_lit(t), "opt_str")

# ---- place_facts
for t in ["do they allow dogs", "is there parking", "is the market on on sunday", "is the museum free on sunday", "can i bring my dog to the pub",
          "can i take the kids to school", "is there wifi in my flat", "is the match on saturday", "does the cafe have vegan food", "is the pub busy on friday"]:
    case("place_facts", "the_line", [t], "the_line(%s)" % rs_lit(t), "opt_pool")

# ---- postcode
for t in ["who is my mp", "when is my bin day", "is my area at risk of flooding", "how can i save on my energy supplier", "whats my council tax band", "hello"]:
    case("postcode", "a_postcode_lookup", [t], "a_postcode_lookup(%s)" % rs_lit(t), "opt_pool")

# ---- prices
for t, b in [("how much does a plumber cost", ""), ("is that a fair price", "they quoted 400 for the boiler service"), ("is 1200 a month enough to live on", ""),
             ("how much flour per person", ""), ("are they worth it", "they cost 300 quid"), ("how much is left on my gift card", ""),
             ("how much is it", "i want to bake bread"), ("is that expensive", "")]:
    case("prices", "the_price_reply", [t, ["cost"], b, ["OPT"]], "the_price_reply(%s, &[\"cost\".to_string()], %s, &[\"OPT\".to_string()])" % (rs_lit(t), rs_lit(b)), "opt_pool")

# ---- the_rule_for_it
for t in ["how much does a tattoo cost", "is the chicken still ok to eat", "how long does it take to roast a chicken", "how much paint do i need",
          "how much rice for four people", "is that taxi expensive", "how long have i been waiting", "is my baby too heavy"]:
    case("the_rule_for_it", "the_rule_for_it", [t], "the_rule_for_it(%s, \"\")" % rs_lit(t), "opt_pool")

# ---- waited_for
for t in ["how long will it be", "should i wait for the sales", "when will it arrive", "hello"]:
    case("waited_for", "asks_for_a_date", [t], "asks_for_a_date(%s, \"\")" % rs_lit(t), "bool")
for t, th in [("should i wait for the sales", ""), ("when will it be done", "the boiler is broken"), ("when will it arrive", "i ordered a sofa"), ("x", "nothing")]:
    case("waited_for", "_the_kind", [t, th], "_the_kind(%s, %s)" % (rs_lit(t), rs_lit(th)), "opt_str")

# ---- receiving_end
for r in ["my boss", "HR", "the council", "her landlord", "teacher"]:
    case("receiving_end", "_the_role_said_back", [r], "_the_role_said_back(%s)" % rs_lit(r), "str")

# ---- repair
for t in ["i let my mum down", "i shouted at my son", "i forgot my wifes birthday", "i hurt my back", "i shouted at the dog",
          "i never lied to her", "i didnt call my mum back", "i feel sick that i lied to her", "i regret shouting at him"]:
    case("repair", "deed_of", [t], "deed_of(&Value::from(%s))" % rs_lit(t), "opt_str")
    case("repair", "amends_to", [t], "amends_to(%s)" % rs_lit(t), "bool")
for t in ["how do i make it up to my mum", "how can i make amends", "make it up to the dog"]:
    case("repair", "amends_to", [t], "amends_to(%s)" % rs_lit(t), "bool")
for r, t in [("Say sorry soon.", "i feel sick that i lied to her"), ("Say sorry soon.", "i lied and i feel awful"), ("Send a card.", "i feel awful about forgetting her birthday, i forgot my wifes birthday")]:
    case("repair", "with_the_feeling_named", [r, t], "with_the_feeling_named(%s, %s)" % (rs_lit(r), rs_lit(t)), "str")
for turns in ([["i made my mum cry", "Oh.", "chat"], ["i dont know what to say", "Say sorry.", "chat"]], [["the garden is lovely", "Nice.", "chat"]]):
    case("repair", "a_repair_is_open", ["$mem"], "a_repair_is_open({mem})", "bool", mem=turns)

# ---- rule
for t, turns in [("is it legal to park on the pavement", None), ("is it illegal to record a call", None), ("will they fine me", [["i put my bin out early, am i allowed to", "Ok.", "chat"]]),
                 ("how long is the tennis court open", None), ("what happens if i ignore the letter", [["my ex is taking me to court", "Ok.", "chat"]]), ("is it legal to give my kids paracetamol", None)]:
    case("rule", "pool_for", [t, "$mem"], "pool_for(%s, {mem})" % rs_lit(t), "opt_pool", mem=turns)

# ---- safetyask
for t in ["is it safe", "is the heater safe to leave on", "is the plug socket cracked dangerous"]:
    case("safetyask", "_a_safety_word", [t], "_a_safety_word(%s)" % rs_lit(t), "bool")
for r in ["A quick search online will get you there.", "The boiler needs a Gas Safe engineer."]:
    case("safetyask", "_a_shrug", [r], "_a_shrug(%s)" % rs_lit(r), "bool")

# ---- setting_off
for t in ["what time should i leave for the airport", "what time should i leave my husband", "what time should i set off if my flight is at 9am",
          "when does the bus leave", "whats the latest i can leave", "what time should he leave", "when should i set off"]:
    case("setting_off", "reads_it", [t], "reads_it(%s)" % rs_lit(t), "bool")
    case("setting_off", "answer_for", [t, "$mem"], "answer_for(%s, {mem})" % rs_lit(t), "opt_str")
case("setting_off", "answer_for", ["when should i set off", "$mem"], 'answer_for("when should i set off", {mem})', "opt_str",
     mem=[["i have an interview at 2.30pm", "Good luck.", "chat"]])
lines("setting_off", "is_a_pointer", ["A search should sort you out.", "Leave at 8."])

# ---- talkto
for t in ["can we talk about my mum", "i need to think through my options", "can we talk tomorrow about my mum", "can u talk to me about it",
          "i wanna wanna talk about my job and stuff", "can you help me think about how to tell her", "can we talk"]:
    case("talkto", "_subject", [t], "_subject(%s)" % rs_lit(t), "str")
    case("talkto", "the_line_as_read", [t], "the_line_as_read(%s)" % rs_lit(t), "str")

# ---- the_trouble_named
for t in ["the boiler", "my car", "the wifi", "the rent", "the cat"]:
    case("the_trouble_named", "_family", [t], "_family(%s)" % rs_lit(t), "opt_str")

# ---- the_turn_after
case("the_turn_after", "_words_of", [["opening hours", "is it open", "the bus"]], '_words_of(&["opening hours".to_string(), "is it open".to_string(), "the bus".to_string()])', "set")

# ---- their_life
for t, turns in [("do i like it", None), ("what does my son do", None), ("how long will she be in", [["my mum is in hospital", "Oh no.", "chat"]]),
                 ("how long will she be in", None), ("where is my dog", None), ("what is his job like", None),
                 ("what car does she have", [["my sister bought a car", "Nice.", "chat"], ["she loves it", "Good.", "chat"]])]:
    case("their_life", "reads", [t, "$mem"], "reads(%s, {mem})" % rs_lit(t), "opt_str", mem=turns)
    case("their_life", "reply_for", [t, "$mem"], "reply_for(%s, {mem})" % rs_lit(t), "opt_str", mem=turns)

# ---- theirperson
for t in ["My best friend is having a really hard time!", "my mum is ill", "my son has adhd"]:
    case("theirperson", "_said", [t], "_said(%s)" % rs_lit(t), "str")
for r, t in [("I'm listening — say more if it helps.", "my mum is struggling"), ("Your mum is struggling, noted.", "my mum is struggling"), ("Lovely.", "my mum is struggling")]:
    case("theirperson", "may_replace", [r, t], "may_replace(%s, %s)" % (rs_lit(r), rs_lit(t)), "bool")

# ---- worn
for c in ["this week has been a nightmare", "work is killing me", "my back is killing me"]:
    case("worn", "_names_a_job", [c], "_names_a_job(%s)" % rs_lit(c), "bool")
    case("worn", "_is_a_role", [c], "_is_a_role(%s)" % rs_lit(c), "bool")

# ---- a_document_lives_with_an_office
for t, r in [("we lost the log book", ""), ("we lost the log book", "Ring the DVLA."), ("i lost my dog", ""), ("where do i get a copy of the deeds", "")]:
    case("a_document_lives_with_an_office", "a_lost_record", [t, r], "a_lost_record(%s, %s)" % (rs_lit(t), rs_lit(r)), "bool")
    case("a_document_lives_with_an_office", "_the_kind", [t], "_the_kind(%s)" % rs_lit(t), "str")
lines("a_document_lives_with_an_office", "a_map_app_line", ["No map on this side, sorry.", "Ring them."])
