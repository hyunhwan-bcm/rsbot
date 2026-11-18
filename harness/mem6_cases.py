"""CPython's values for MEM6 functions on hand-made inputs, for the branches the corpus rarely
reaches. Written in harness/mem6_record.py's row format, so `mem6_check` reads them:

    python3 harness/mem6_cases.py harness/mem6_cases.jsonl.gz
"""
import gzip, inspect, json, os, random, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "harness"))
import run_python  # noqa: E402
from export_data import encode  # noqa: E402
from plain import plain  # noqa: E402

run_python.pin_clock("2026-11-17")
sys.path.insert(0, os.path.abspath(run_python.PYBOT))
import importlib  # noqa: E402
from bot import runtime  # noqa: E402,F401


def turns(*pairs, module="answers"):
    return {"turns": [{"user": u, "bot": b, "module": module} for u, b in pairs]}


PLUNGER = ("Start with a plunger over the plughole, ten firm pushes. If that fails, undo the trap "
           "underneath (bucket first) and clean it out, then put it back and run the tap.")
ANTS = "Wipe the trail away with vinegar and water, then put bait down close to the nest."
BOILER = ("Top the pressure up to 1.5 bar with the filling loop. Then bleed the radiators. If it "
          "keeps dropping, get a heating engineer out.")
BREAD = "Mix 500g strong white flour, 7g yeast, 10g salt and 325ml warm water into a soft dough."
ROOF = ("A small patch repair is £150 to £400, a new flat roof is £1,500 to £3,000, and a whole "
        "re-roof runs £5,000 to £9,000.")
NEW_BOILER = "A new combi boiler fitted costs £2,000 to £4,000. Get three quotes."
CASES = [
    ("bot.memory.limit", "_steps", [PLUNGER]), ("bot.memory.limit", "_steps", [ANTS]),
    ("bot.memory.limit", "_kept", [PLUNGER, "plunger"]), ("bot.memory.limit", "_kept", [BOILER, "pressure"]),
    ("bot.memory.limit", "_say", ["Skip the plunger: ", ["undo the trap underneath (bucket first)", "and clean it out"]]),
    ("bot.memory.limit", "_say", ["", ["run the tap"]]),
    ("bot.memory.limit", "_with_a_pet", [ANTS, "a cat"]),
    ("bot.memory.limit", "_names", ["make a paste of bicarbonate of soda", "bicarbonate soda"]),
    ("bot.memory.limit", "_names", ["use a damp cloth", "cloth"]),
    ("bot.memory.limit", "is_a_thing_on_a_list", ["7g dried yeast", "yeast"]),
    ("bot.memory.limit", "is_a_thing_on_a_list", ["and put a bowl of ice in front of a fan", "fan"]),
    ("bot.memory.limit", "_among_measured_things", [BREAD, "yeast"]),
    ("bot.memory.limit", "_the_amount_of_it", ["use the same weight of soft butter", "butter"]),
    ("bot.memory.limit", "_needs_it", ["pressure", BOILER]),
    ("bot.memory.limit", "_holds_on", [BOILER, "pressure"]),
    ("bot.memory.limit", "_first_with", [PLUNGER, "trap"]),
    ("bot.memory.limit", "_up_to_words", [["one two three four five six seven eight nine ten", "a b c d e f g h i j k l m n o p q r s t u v", "last bit"]]),
    ("bot.memory.limit", "_without_a_repeated_time", [["cook them gently for 2 minutes", "10 minutes, give or take"]]),
    ("bot.memory.limit", "_bare_phrase", ["Start with a plunger", "plunger"]),
    ("bot.memory.limit", "_tidy", ["and then rinse it"]),
    ("bot.memory.limit", "reads_as_a_limit", ["i dont have a plunger"]),
    ("bot.memory.limit", "reads_as_a_limit", ["i ran out of oven cleaner"]),
    ("bot.memory.limit", "reads_as_a_limit", ["ive got a cat"]),
    ("bot.memory.limit", "reads_as_a_limit", ["i have a dog and a toddler"]),
    ("bot.memory.limit", "reads_as_a_limit", ["no ant powder because of the cat", turns(("how do i get rid of ants", ANTS))]),
    ("bot.memory.limit", "reads_as_a_limit", ["no oven cleaner but i have vinegar"]),
    ("bot.memory.limit", "reads_as_a_limit", ["my shower has no water pressure"]),
    ("bot.memory.limit", "reads_as_a_limit", ["no yeast in the house"]),
    ("bot.memory.limit", "reads_as_a_limit", ["no way"]),
    ("bot.memory.limit", "only_a_limit", ["no walking up hills"]),
    ("bot.memory.limit", "only_a_limit", ["no long walks please"]),
    ("bot.memory.limit", "_names_no_thing", ["whats the first thing to change"]),
    ("bot.memory.already", "the_trouble_is_over", ["my boiler is fixed"]),
    ("bot.memory.already", "the_trouble_is_over", ["the heating still isnt working"]),
    ("bot.memory.already", "the_trouble_is_over", ["we got the boiler fixed yesterday"]),
    ("bot.memory.already", "the_trouble_is_over", ["the sink is sorted, wasnt it"]),
    ("bot.memory.already", "the_trouble_is_over", ["dinner is done"]),
    ("bot.memory.already", "the_trouble_is_over", ["all my mates are sorted"]),
    ("bot.memory.already", "the_thing_that_is_mended", ["the washing machine is working now"]),
    ("bot.memory.already", "the_thing_that_is_mended", ["shes 62 and still works"]),
    ("bot.memory.already", "the_step_they_took", ["i already called the bank"]),
    ("bot.memory.already", "the_step_they_took", ["i did ring them though, twice"]),
    ("bot.memory.already", "the_step_they_took", ["i called them already"]),
    ("bot.memory.already", "the_step_they_took", ["i already told her sorry"]),
    ("bot.memory.already", "the_step_they_took", ["i already read that"]),
    ("bot.memory.already", "the_step_they_took", ["ive fixed it"]),
    ("bot.memory.already", "the_step_they_took", ["done that, thanks"]),
    ("bot.memory.already", "the_step_they_took", ["i already have a dog"]),
    ("bot.memory.already", "_the_steps_after", ["turned", "Turn the water off at the isolating valve under the sink, take the handle off, and take the old part to a hardware shop.", "i already turned the water off"]),
    ("bot.memory.already", "_says_the_step", ["bled", "Bleed the radiators first."]),
    ("bot.memory.already", "_says_the_step", ["apologise", "Say sorry for the shouting."]),
    ("bot.memory.already", "_of_a_thing", ["changed", "i changed the washer already"]),
    ("bot.memory.already", "_of_a_thing", ["changed", "ive changed him already"]),
    ("bot.memory.already", "_the_instruction_of", ["ask for an affordable payment plan: most councils will spread it"]),
    ("bot.memory.already", "_the_opening_of", ["Leave something that smells of home. Kneel down."]),
    ("bot.memory.already", "reply_for", ["i already called the bank", turns(("someone used my card online", "Ring your bank on the number on the back of the card."))]),
    ("bot.memory.already", "reply_for", ["i already bled the radiators", turns(("my boiler is losing pressure", "Bleed the radiators, then top the pressure up to 1.5 bar with the filling loop."))]),
    ("bot.memory.already", "reply_for", ["my boiler is fixed", turns(("my boiler is broken", "Check the pressure gauge."))]),
    ("bot.memory.already", "reply_for", ["i already answered that", {"turns": []}]),
    ("bot.memory.spent", "readings", ["i spent 300 on food this month and 40 on the train"]),
    ("bot.memory.spent", "readings", ["it cost me 80 for the train"]),
    ("bot.memory.spent", "readings", ["my rent is 700"]), ("bot.memory.spent", "readings", ["my son is 7, 9"]),
    ("bot.memory.spent", "readings", ["we pay 50 a month for broadband"]),
    ("bot.memory.spent", "_held_in", ["ive got 200 till payday"]), ("bot.memory.spent", "_held_in", ["my budget is 2k"]),
    ("bot.memory.spent", "_held_in", ["there are 3 eggs left"]), ("bot.memory.spent", "_held_in", ["theres 40 left"]),
    ("bot.memory.spent", "_the_price", ["a 200 coat"]), ("bot.memory.spent", "_the_price", ["10 tickets"]),
    ("bot.memory.spent", "_a_thing_to_buy", ["can i afford a takeaway tonight"]),
    ("bot.memory.spent", "what_is_left", ["how much have i got left", turns(("ive got 200 till payday", "Noted."), ("i spent 40 on food", "Noted."))]),
    ("bot.memory.spent", "what_is_left", ["can i afford a 50 coat", turns(("ive got 200 till payday", "Noted."))]),
    ("bot.memory.spent", "what_is_left", ["what if i spend another 300", turns(("ive got 200 till payday", "Noted."))]),
    ("bot.memory.spent", "what_is_left", ["how much is left", {"turns": []}]),
    ("bot.memory.spent", "answer", ["how much have i spent on food", turns(("i spent 300 on food", "Ok."), ("40 on a takeaway", "Ok."))]),
    ("bot.memory.spent", "answer", ["how much do i spend on water", turns(("we pay 30 a month for water", "Ok."))]),
    ("bot.memory.outgoings", "reply_for", ["im on 1800 a month and rent is 700"]),
    ("bot.memory.outgoings", "reply_for", ["i earn 30000 a year and rent is 700"]),
    ("bot.memory.outgoings", "reply_for", ["i make 1200 and my partner makes 800 and the rent is 900"]),
    ("bot.memory.outgoings", "reply_for", ["and rent is 900", turns(("i earn about 1800 a month", "Ok."))]),
    ("bot.memory.outgoings", "reply_for", ["rent is 750 and i put 200 aside", turns(("i earn 2000 a month", "Ok."))]),
    ("bot.memory.outgoings", "reply_for", ["i earn about 1800 a month", turns(("how much should i save each month", "Ten to twenty percent."))]),
    ("bot.memory.outgoings", "_sums", ["i earn £1,800.50 a month and rent is 700 pounds and 2k savings"]),
    ("bot.memory.outgoings", "_pounds", [1234.5]), ("bot.memory.outgoings", "_pounds", [1100.0]),
    ("bot.memory.price", "_prices_in", ["the cheapest quote i got was £2,400 or 3k"]),
    ("bot.memory.price", "_money", [2400.5]), ("bot.memory.price", "_money", [250.0]),
    ("bot.memory.price", "judged_with_the_card", ["is 6000 too much for a new boiler", NEW_BOILER, {"turns": []}]),
    ("bot.memory.price", "judged_with_the_card", ["is 1000 too much for a roof repair", ROOF, {"turns": []}]),
    ("bot.memory.price", "judged", ["is that a lot", turns(("my roof quote is 400 for a patch", ROOF))]),
    ("bot.memory.price", "judged", ["does that sound right", turns(("they quoted 3000", NEW_BOILER))]),
    ("bot.memory.price", "_the_thing_priced", ["is £60 a lot for a haircut"]),
    ("bot.memory.counts", "_counted", ["three brothers and two flat tyres"]),
    ("bot.memory.counts", "_counted", ["two cats called milo and tom"]),
    ("bot.memory.counts", "_beds", ["we live in a 3 bed house"]),
    ("bot.memory.counts", "_the_group", ["there are 4 of us"]), ("bot.memory.counts", "_the_party", ["a roast for 6 on sunday"]),
    ("bot.memory.counts", "answer", ["how many brothers do i have", {"user": {"how_many": {"brother": 3}}, "turns": []}]),
    ("bot.memory.counts", "answer", ["how many brothers and sisters do i have", {"user": {"how_many": {"brother": 3}}, "turns": []}]),
    ("bot.memory.counts", "answer", ["how many people again", {"user": {"how_many": {"person": 8}}, "turns": [{"user": "im doing a roast for 8", "bot": "Ok.", "module": "chat"}]}]),
    ("bot.memory.counts", "answer", ["how many of us are there", {"user": {"how_many": {"us": 2}}, "turns": []}]),
    ("bot.memory.decided", "read", ["actually a dog"]), ("bot.memory.decided", "read", ["lets go with spain"]),
    ("bot.memory.decided", "read", ["no, the blue one"]), ("bot.memory.decided", "read", ["im going for a run"]),
    ("bot.memory.decided", "read", ["ive decided on the blue one"]),
    ("bot.memory.decided", "_two_things", ["im torn between italy and spain"]), ("bot.memory.decided", "_two_things", ["should i get a cat or a dog"]),
    ("bot.memory.decided", "answer", ["what did i decide", {"decided": {"what": "a dog", "line": "actually a dog"}, "turns": [{"user": "should i get a cat or a dog", "bot": "Either.", "module": "chat"}, {"user": "actually a dog", "bot": "Ok.", "module": "chat"}]}]),
    ("bot.memory.decided", "answer", ["did we settle on spain", {"decided": {"what": "spain", "line": "lets go with spain"}, "turns": []}]),
    ("bot.memory.decided", "answer", ["what did i decide", {"turns": [{"user": "i cant decide between italy or spain", "bot": "Both good.", "module": "chat"}]}]),
    ("bot.memory.told", "_quote", ["im learning spanish", {"turns": []}]),
    ("bot.memory.told", "_quote", ["i said the party is on tuesday", {"turns": []}]),
    ("bot.memory.told", "_no_note", ["your sister", {"turns": []}]), ("bot.memory.told", "_no_note", ["i was quitting my job", {"turns": []}]),
    ("bot.memory.told", "_many", [{"animal": "cat", "name": "milo"}]), ("bot.memory.told", "_many", [{"animal": "dog", "count": 2}]),
    ("bot.memory.told", "_many", [{"animal": "fish", "count": 3}]),
    ("bot.memory.told", "_trim", ["word " * 30]), ("bot.memory.told", "_stem", ["savings"]),
    ("bot.memory.told", "_the_kind_of", ["holidays"]), ("bot.memory.told", "_yours", ["my back"]),
    ("bot.memory.told", "check", ["is my mum coeliac", turns(("my mum is coeliac", "Noted."))]),
    ("bot.memory.told", "check", ["is my son 7", turns(("my son is 7", "Ok."), ("no, my son is 8", "Ok."))]),
    ("bot.memory.told", "check", ["is my mum coeliac", turns(("my mum is coeliac", "Noted."), ("sorry she isnt", "Ok."))]),
    ("bot.memory.told", "check", ["is my sister called emma", turns(("my sister is called emma", "Lovely."))]),
    ("bot.memory.told", "is_a_boundary", ["i told you already, i dont want to talk about it"]),
    ("bot.memory.swap", "thing", ["what about a dessert"]), ("bot.memory.swap", "thing", ["no i mean brown rice"]),
    ("bot.memory.swap", "thing", ["no wait, a laptop"]), ("bot.memory.swap", "thing", ["what about my hip"]),
    ("bot.memory.swap", "thing", ["no thanks at all"]), ("bot.memory.swap", "thing", ["and in the freezer too"]),
    ("bot.memory.swap", "thing", ["what about a labrador probably"]),
    ("bot.memory.swap", "placed", ["how do i get rid of the smell in my fridge", "fridge", "in the freezer"]),
    ("bot.memory.swap", "placed", ["how do i clean the shower", "shower", "under the sink"]),
    ("bot.memory.swap", "placed", ["what about a carrot cake", "carrot", "coffee cake"]),
    ("bot.memory.swap", "_replace", ["how long do i boil an egg", "egg", "duck egg"]),
    ("bot.memory.swap", "_spans", ["how do i clean a stainless steel pan", "steel"]),
    ("bot.memory.swap", "_the_place_after_the_same_preposition", ["whats a good day trip from manchester", "from birmingham"]),
    ("bot.memory.choice", "options_in", ["*Groundhog Day* for comedy, or *Arrival* if you want something slower."]),
    ("bot.memory.choice", "options_in", ["Would you rather go Saturday or Sunday?"]),
    ("bot.memory.choice", "options_in", ["Scrambled eggs: quick and easy. Aglio e olio: pasta with garlic."]),
    ("bot.memory.choice", "options_in", ["nice soap, woolly socks, a good book"]),
    ("bot.memory.choice", "_the_thing_in", ["actually no, a scooter"]), ("bot.memory.choice", "_the_thing_in", ["lets go with the grey"]),
    ("bot.memory.choice", "_the_thing_in", ["i went for a run"]), ("bot.memory.choice", "_the_thing_in", ["no a kitten instead"]),
    ("bot.memory.choice", "_from_user_line", [{"turns": [{"user": "should i get a cat or a dog", "bot": "Hmm.", "module": "chat"}]}]),
    ("bot.memory.choice", "reply_for", ["you pick", turns(("what should i cook", "Pasta with garlic and oil; eggs on toast; or cold rice fried with veg."))]),
    ("bot.memory.choice", "reply_for", ["up to you", turns(("what should i cook", "My pick: eggs on toast."))]),
    ("bot.memory.repair", "_the_thing", ["no i meant the front room"]), ("bot.memory.repair", "_the_thing", ["no my car keys"]),
    ("bot.memory.repair", "_the_thing", ["no the boiler is fine"]), ("bot.memory.repair", "_the_thing", ["sorry i meant to say bristol"]),
    ("bot.memory.repair", "_the_thing", ["actually i have chicken"]),
    ("bot.memory.repair", "_swapped", ["i need to pick a paint colour for the hall", "hall", "the front room"]),
    ("bot.memory.repair", "_swapped", ["my grans 84", "gran", "my nan"]),
    ("bot.memory.repair", "_the_pointer_refines", ["the lower one", "my back hurts"]),
    ("bot.memory.repair", "_the_relation", ["grans"]),
    ("bot.memory.repair", "_the_same_words", ["a 2kg bird takes roughly 1 hour 40 minutes", "a 3kg bird takes roughly 2 hours 20 minutes"]),
    ("bot.memory.repair", "_the_figures_that_moved", ["Buy a 2kg joint. Roast for 80 minutes.", "Buy a 3kg joint. Roast for 110 minutes."]),
    ("bot.memory.carry", "_the_quality", ["hows about something cold"]), ("bot.memory.carry", "_the_quality", ["something lighter"]),
    ("bot.memory.carry", "_a_verdict_asked", ["is that fair"]), ("bot.memory.carry", "_a_verdict_asked", ["is that a good idea for a dog"]),
    ("bot.memory.carry", "_the_thing_they_named", ["we are thinking of getting a dog"]),
    ("bot.memory.carry", "_speaks_of", ["Tacos with lime and yogurt.", ["dairy"]]),
    ("bot.memory.carry", "_a_free_from", [["dairy"]]),
    ("bot.memory.history", "asked", ["what have we talked about"]), ("bot.memory.history", "asked", ["what was the first thing i said"]),
    ("bot.memory.history", "asked", ["what did you say first"]), ("bot.memory.history", "asked", ["what did you say about the boiler"]),
    ("bot.memory.history", "is_their_news", ["i got the job", 3]), ("bot.memory.history", "is_their_news", ["i want to go to rome", 3]),
    ("bot.memory.history", "topic_lines", [{"topics": [{"text": "my dog Biscuit"}, {"text": "hi"}], "turns": [{"user": "he keeps scratching", "bot": "Hm.", "module": "answers"}]}]),
    ("bot.memory.cause", "_a_thing", ["maybe its work"]), ("bot.memory.cause", "_a_thing", ["the baby"]),
    ("bot.memory.cause", "_a_thing", ["dunno, the baby"]), ("bot.memory.cause", "_a_thing", ["lol ok"]),
    ("bot.memory.cause", "_names_nothing", ["it all"]), ("bot.memory.cause", "_the_persons_own_reason", ["my sleep"]),
    ("bot.memory.cause", "_says_the_reason", ["That is a lot to carry with work.", "work"]),
    ("bot.memory.cause", "_without_what_was_just_said", ["One. Two. Three.", "Two."]),
    ("bot.memory.cause", "expand", ["the baby", turns(("im knackered", "Long one?"), ("why", "Which part?"))]),
    ("bot.memory", "you", ["i told you about my mum"]), ("bot.memory", "you", ["me, mum and dad went to spain"]),
    ("bot.memory", "plan_sentence", ["dentist at 3 on tuesday", None]),
]


def main():
    out = gzip.open(sys.argv[1], "wt")
    for module, name, args in CASES:
        fn = getattr(importlib.import_module(module), name)
        bound = inspect.signature(fn).bind(*args)
        bound.apply_defaults()
        before = {k: plain(encode(v)) for k, v in bound.arguments.items()}
        random.seed(0)
        state = random.getstate()
        row = {"fn": module + ":" + name, "args": before, "raised": None}
        try:
            row["result"] = plain(encode(fn(*args)))
        except Exception as e:  # noqa: BLE001
            row["result"], row["raised"] = None, type(e).__name__
        if random.getstate() != state:
            row["rng"] = True
        after = {k: plain(encode(v)) for k, v in bound.arguments.items()}
        changed = {k: v for k, v in after.items() if v != before[k]}
        if changed:
            row["after"] = changed
        out.write(json.dumps(row, ensure_ascii=False) + "\n")
    out.close()
    print(len(CASES), "cases", file=sys.stderr)


if __name__ == "__main__":
    main()
