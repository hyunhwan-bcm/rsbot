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
