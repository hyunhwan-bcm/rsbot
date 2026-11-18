import json, os, sys, copy
sys.path.insert(0, 'harness')
import run_python
run_python.pin_clock("2026-11-17")
sys.path.insert(0, os.path.abspath(run_python.PYBOT))
from bot import runtime
import bot.memory as m
out = []
def s(fn, *a):
    out.append([fn, list(a), getattr(m, fn)(*a)])
for t in ["i told you about my mum", "my mums birthday is on friday", "im seeing my sisters tomorrow", "me and me sister went to me mums",
          "I was not there, my dad said \"im not coming\" and i left", "i sent my mum a thank you card", "thank you for helping me, i called You yesterday",
          "we wasnt sure, ive got mine myself", "shes 72 and Dont care", "me, mum and dad went to spain", "talk to me whenever you like",
          "Im off to my brothers tonight", "i dont get on with me sister", "our daughters school is far"]:
    s("you", t)
for p, l in [("Spain in july", "we're going to spain in july"), ("dentist at 3 on tuesday", ""), ("call the plumber on monday", ""),
             ("A cake on friday", "im making a cake on friday"), ("her appointment on 3 june", ""), ("the clocks go back on 27 october", ""),
             ("i start my new job on monday", ""), ("The gym tomorrow", ""), ("planning a picnic for my daughter", ""), ("Saving up for a bike", ""),
             ("Visit my mum next weekend lol", ""), ("to paint the fence", ""), ("also my mums birthday on saturday", "also my mums birthday is saturday"),
             ("sons parents evening on thursday", "my sons parents evening is on thursday"), ("Portugal on the 12th of june", ""),
             ("plan to visit my mum", ""), ("the kids and i are going to spain", ""), ("book club on friday", ""), ("party for 8 on saturday", ""),
             ("bake bread on sunday", ""), ("Leave for the airport at 4", "we need to leave for the airport at 4"), ("france in may", ""),
             ("im going to york in may 3", ""), ("bath on monday", ""), ("", "x"), ("viewing the flat at 3", ""), ("meet sam at 3pm on tuesday", "")]:
    s("plan_sentence", p, l)
for t in ["i want to save 480 pounds in 6 months, how much should i put away", "were moving house in march", "what do i need", "we are going. "]:
    s("_statement", t)
for v, t in [("Two kids", ""), ("Know about saving", "what do i need to know about saving"), ("Spain in july", "we go to spain in july"), ("x", "")]:
    s("reads_as_a_plan", v, t)
s("_said_another", "my other little girl is 2", "girl"); s("_said_another", "he turns 8 the second week of june", "son")
s("named_days", "we may go to york on tuesday, 3 may or in MARCH and march 14th")
# memory scenarios
def scen(name, steps):
    mem = {"turns": [{"user": "hi", "bot": "yo", "module": "chat"}]}
    rets = []
    for fn, a in steps:
        r = getattr(m, fn)(mem, *copy.deepcopy(a))
        rets.append(r if fn in ("forget", "plan_line", "said", "last_user", "last_bot", "last_module", "name") else None)
    out.append(["scen:" + name, steps, {"mem": mem, "rets": rets}])
scen("pets", [("remember", ["pets", [{"animal": "cat"}, {"animal": "cat", "name": "Tom"}], "t1"]),
              ("remember", ["pets", [{"animal": "cat", "name": "Max"}], "t2"]),
              ("remember", ["pets", [{"animal": "dog", "count": 2}], "t3"]),
              ("remember", ["pets", [{"animal": "dog", "name": "Rex", "another": True, "age": 5, "unit": "years"}], "t4"]),
              ("remember", ["pets", [{"animal": "puppy", "coming": True}], "t5"]),
              ("remember", ["pets", [{"animal": "labrador", "breed": "labrador"}], "t6"]),
              ("remember", ["pets", [{"animal": "fish", "not_theirs": True}], "t7"]),
              ("remember", ["pets", [{"animal": "dog", "count": 3}], "t8"]),
              ("forget", ["pets", [{"animal": "dog", "count": 3, "name": "Rex", "age": 5, "unit": "years"}], "died"]),
              ("forget", ["pets", ["cat"], None]),
              ("forget", ["pets", "*"]) ])
scen("facts", [("remember", ["name", "Sam", "im sam"]), ("remember", ["name", "Sam", "im sam"]), ("remember", ["allergies", ["nuts", "milk lol"], "a"]),
               ("remember", ["allergies", "nuts", "b"]), ("remember", ["plans", ["my sister is coming in may", "my brother is coming in june", "Two kids"], "my sister is coming in may and my brother in june"]),
               ("remember", ["plans", "my brother is coming in july tbh", "actually my brother is coming in july"]),
               ("remember", ["plan", "Spain in july haha", "we're going to spain in july, what should i pack"]),
               ("remember", ["tastes", [{"thing": "Cats", "how": "yes"}, {"thing": "cats ", "how": "no"}, {"thing": "fish"}], "c"]),
               ("remember", ["owns", {"car": "fiesta", "phone": "pixel", "": "x"}, "d"]), ("remember", ["owns", {"car": "fiesta"}, "e"]),
               ("remember", ["age", 34, "f"]), ("remember", ["age", 0, "g"]), ("remember", ["x", "", "h"]),
               ("plan_line", ["spain in  JULY haha"]), ("plan_line", ["nope"]), ("said", ["name"]), ("said", ["zz"]), ("name", []),
               ("forget", ["allergies", ["Nuts "]]), ("forget", ["age", 0]), ("forget", ["name", ["bob"]]), ("forget", ["name", "sam"]), ("forget", ["places", "x"]),
               ("last_user", [1]), ("last_user", [3]), ("last_bot", []), ("last_module", [])])
scen("others", [("remember_other", [{"who": "son", "age": 7}, "my son is 7"]), ("remember_other", [{"who": "son", "age": 8, "turns": "june"}, "he turns 8 in june"]),
                ("remember_other", [{"who": "son", "age": 5}, "my other son is 5"]), ("remember_other", [{"who": "kids", "ages": (4, 7)}, "k"]),
                ("remember_other", [{"who": "kids", "age": 9}, "k2"]), ("remember_other", [{"who": "kids", "ages": [5, 8]}, "k3"]),
                ("remember_other", [{"who": "daughter", "name": "Ada", "allergies": ["nuts"]}, "x"]), ("remember_other", [{"who": "daughter", "name": "Bea"}, "x"]),
                ("remember_other", [{"name": "ada", "allergies": ["nuts", "eggs", "eggs"], "diet": "veggie", "pronoun": "she"}, "y"]),
                ("remember_other", [{"name": "zed", "age": 3}, "z"]), ("remember_other", [{"who": "", "x": None}, "z"]),
                ("remember_other", [{"who": "son", "age": 9}, "no he is 9"]), ("remember_other", [{"who": "", "diet": "vegan"}, "hes vegan"])])
json.dump(out, open('harness/mem6_init_cases.json', 'w'), ensure_ascii=False, default=list)
print(len(out))
