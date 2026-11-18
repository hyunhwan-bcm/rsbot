"""bot.knowledge.units on many lines, for the port's check (`kno_check units FILE`).

    python3 harness/kno/make_units.py OUT.jsonl [N_FUZZ]

One JSON line per case: {"c": clean line, "t": [earlier user lines], "r": {function: result}}.
`convert` is called with a memory of those turns; every other function with the line alone.
Only results that are not None / False are kept; an exception is "!TypeName".
Lines: the clean line of every recorded reading (benchmark and conversation turns, with the
conversation's earlier lines as turns), then lines made from the module's own tables.
"""
import sys, os, json, gzip, random
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "find"))
import pyenv  # noqa: F401  (pins the clock, puts PyBot on the path)
from bot.knowledge import units as U

ONE = """_object_weight _oven _temperature _temperature_rule _butter _pack _pack_count _loose
_liquid_weight _a_dish_for_a_table _per_person _portions_in_a_weight _birds_for_a_table
_a_whole_thing_feeds _share_of_a_measure _a_span_of_time _handful _own_height _cup_of_something
_piece_weight _frac people_in_groups _people_asked answers_more_than_a_pair _count_in_a_line
_metres_in_feet _the_thing_measured_by_itself _in_a_vessel _a_fraction_of_a_time_unit
_a_corrected_figure _a_table_without_people _asks_something_else _asks_for_weight""".split()


def call(f, *a):
    try:
        return f(*a)
    except Exception as e:  # noqa: BLE001
        return "!" + type(e).__name__


def case(clean, turns):
    r = {}
    memory = {"turns": [{"user": t, "bot": "", "module": None} for t in turns]} if turns else None
    v = call(U.convert, clean, memory)
    if v not in (None, False):
        r["convert"] = v
    for name in ONE:
        v = call(getattr(U, name), clean)
        if v not in (None, False):
            r[name] = v
    return {"c": clean, "t": turns, "r": r}


def recorded():
    base = "/Volumes/SSD/Workspace/rsbot/data/export"
    seen = set()
    for f in ("oracle_gold.plain.jsonl", "oracle_heldout.plain.jsonl", "understand_calls.jsonl"):
        for line in open(os.path.join(base, f)):
            row = json.loads(line)
            u = row.get("understood") or row.get("result") or {}
            c = u.get("clean") if isinstance(u, dict) else None
            if isinstance(c, str) and c and c not in seen:
                seen.add(c)
                yield c, []
    conv, said = None, []
    for line in gzip.open(os.path.join(base, "turn_oracle", "conversations.jsonl.gz"), "rt"):
        rec = json.loads(line)
        if rec["conv"] != conv:
            conv, said = rec["conv"], []
        u = None
        for o in rec["observe"]:
            if o["module"] == "understand":
                u = o["diff"]["set"].get("understood")
        c = (u or {}).get("clean") if isinstance(u, dict) else None
        if isinstance(c, str) and c:
            yield c, list(said[-6:])
        said.append(rec["text"])


def fuzz(n):
    R = random.Random(7)
    units = [w for fam in U.UNITS.values() for _, words, _ in fam for w in words]
    foods = [x[0] for x in U.GRAMS_PER_CUP] + [x[0] for x in U.GRAMS_PER_TSP]
    pieces = [x[0] for x in U.PIECE_GRAMS]
    pieces += [U._plural_of(p) for p in pieces]
    objects = [x[0] for x in U.OBJECT_GRAMS]
    packs = [x[0] for x in U.PACK_GRAMS]
    portions = [nm for row in U.PER_PERSON + U.PER_PERSON_WHEN_COUNTED for nm in row[0]]
    dishes = [nm for row in U.DISHES_FOR_A_TABLE for nm in row[0]]
    wholes = [nm for row in U.WHOLE_THING_SERVES for nm in row[0]]
    loose = [x[0] for x in U.LOOSE]
    nums = ["1", "2", "3", "4", "5", "6", "8", "10", "12", "15", "20", "25", "50", "75", "100", "150", "180", "200",
            "220", "250", "350", "400", "500", "750", "1000", "1.5", "2.5", "0.5", "0.25", "1,500", "a", "an", "one",
            "half a", "a quarter of a", "three quarters of a", "1/2", "3/4", "1 1/2", "2 1/4", "two", "three", "six",
            "ten", "twelve", "a dozen", "one and a half", "two and a half"]
    people = ["2", "4", "6", "8", "10", "12", "20", "30", "two", "four", "six", "eight", "twelve", "a dozen"]
    fracs = ["half", "a half", "a quarter", "three quarters", "two thirds", "a third", "one third", "1/2", "3/4", "2/3",
             "a fifth", "a sixth", "1/8", "0.75", ".25", "five sixths", "a tenth", "two fifths", "three-quarters"]
    tunits = ["hour", "hours", "an hour", "minute", "a minute", "day", "a day", "week", "a week", "hr", "mins"]
    packw = ["tin", "can", "jar", "packet", "bag", "block", "carton", "tub", "box", "bar", "loaf", "punnet", "bottle",
             "boxes", "tins", "loaves"]
    asks = ["how much", "how many", "what is", "whats", "what size", "how big", "how heavy is", "how do i convert"]
    T = [
        lambda: f"how many {R.choice(units)} in {R.choice(nums)} {R.choice(units)}",
        lambda: f"how many {R.choice(units)} in {R.choice(nums)} {R.choice(units)} of {R.choice(foods)}",
        lambda: f"{R.choice(nums)} {R.choice(units)} {R.choice(['to', 'in', 'into', 'as', '='])} {R.choice(units)}",
        lambda: f"{R.choice(nums)}{R.choice(units)} {R.choice(['to', 'in'])} {R.choice(units)}",
        lambda: f"{R.choice(nums)} {R.choice(units)} of {R.choice(foods)} in {R.choice(units)}",
        lambda: f"how many {R.choice(units)} is {R.choice(nums)} {R.choice(units)} of {R.choice(foods)}",
        lambda: f"{R.choice(['how do i convert', 'convert', 'how to change', ''])} {R.choice(units)} to {R.choice(units)}".strip(),
        lambda: f"{R.choice(asks)} a {R.choice(units)} of {R.choice(foods)} weigh",
        lambda: f"how many grams in a {R.choice(['cup', 'mug', 'tablespoon', 'teaspoon', 'tsp', 'tbsp', 'dessertspoon', 'handful'])} of {R.choice(foods)}",
        lambda: f"how much {R.choice(foods)} {R.choice(['in', 'per'])} a {R.choice(['cup', 'teaspoon', 'tablespoon', 'spoonful', 'cups'])}",
        lambda: f"how much {R.choice(['sugar', 'salt', 'fat', 'protein', 'caffeine'])} is in a {R.choice(['teaspoon', 'cup', 'tablespoon'])}{R.choice(['', ' of sugar', ' of tea', ' of salt'])}",
        lambda: f"{R.choice(['how much does', 'how heavy is', 'what is the weight of', 'whats the weight of'])} {R.choice(['a', 'an', 'my', 'the', 'two', '3'])} {R.choice(objects + [U._many(o) for o in objects[:20]])} {R.choice(['weigh', '', 'weigh about'])}".strip(),
        lambda: f"how many grams in {R.choice(nums)} {R.choice(pieces)}",
        lambda: f"how much does {R.choice(nums)} {R.choice(pieces)} weigh",
        lambda: f"{R.choice(nums)} {R.choice(pieces)} in grams",
        lambda: f"how many {R.choice(['grams', 'ml', 'ounces', 'g'])} in a {R.choice(packw)} of {R.choice(packs)}",
        lambda: f"how big is a {R.choice(packw)} of {R.choice(packs)}",
        lambda: f"how many {R.choice(pieces + ['eggs', 'teabags', 'slices', 'squares', 'stock cubes', 'grams'])} in a {R.choice(packw)}{R.choice(['', ' of ' + R.choice(packs + pieces)])}",
        lambda: f"how much is a {R.choice(loose)}",
        lambda: f"how many ml in a {R.choice(loose)}",
        lambda: f"how much {R.choice(['water', 'milk', 'juice', 'wine', 'cola', 'squash'])} in a {R.choice(['', 'small ', 'big ', 'large ', 'tall '])}{R.choice(['glass', 'cup', 'mug', 'pint'])}",
        lambda: f"how much does {R.choice(nums)} {R.choice(['pint', 'pints', 'litre', 'litres', 'ml', 'gallon', 'mug', 'cup', 'fl oz'])} of {R.choice(['milk', 'cream', 'honey', 'oil', 'water'])} weigh",
        lambda: f"how {R.choice(['much', 'many'])} {R.choice(portions)} {R.choice(['per person', 'for ' + R.choice(people) + ' people', 'for ' + R.choice(people), 'each', 'do i need for a family of ' + R.choice(people), 'between us', 'for ' + R.choice(people) + ' adults and ' + R.choice(people) + ' kids'])}",
        lambda: f"{R.choice(['how big a', 'what size'])} {R.choice(portions)} for {R.choice(people)}",
        lambda: f"i need {R.choice(portions)} for {R.choice(people)} people",
        lambda: f"im making {R.choice(dishes + portions)} for {R.choice(people)} {R.choice(['people', 'of us', 'guests', ''])}".strip(),
        lambda: f"how much {R.choice(dishes)} {R.choice(['', 'powder ', 'sauce '])}for {R.choice(people)} people".replace("  ", " "),
        lambda: f"how many portions is a portion of {R.choice(portions)}",
        lambda: f"{R.choice(['how many does', 'i have', 'i bought', 'is'])} {R.choice(nums)} {R.choice(['kg', 'g', 'kilo', 'grams', 'kilos'])} of {R.choice(portions)} {R.choice(['feed', 'enough for 4', 'how many portions', 'serve'])}",
        lambda: f"{R.choice(['i have', 'how many will'])} {R.choice(['6', '12', '2', '300', '0', '9'])} {R.choice(['', 'large ', 'big '])}{R.choice(portions)} {R.choice(['feed', 'how many portions', 'serve'])}",
        lambda: f"{R.choice(['is', 'does', 'will'])} {R.choice(['one', 'a', '1', 'the', 'a whole', 'one 2kg', 'a 1.6 kg', 'whole'])} {R.choice(wholes)} {R.choice(['enough for', 'feed', 'serve', 'go round', 'do for'])} {R.choice(people + [''])}".strip(),
        lambda: f"{R.choice(['how many', 'do i need two', 'are three', 'will 2', 'are 12'])} {R.choice(['chickens', 'ducks', 'whole chickens'])} {R.choice(['for', 'enough for', 'to feed'])} {R.choice(people)} {R.choice(['people', '', 'of us'])}".strip(),
        lambda: f"{R.choice(['what is', 'whats', 'how much is', 'how many tablespoons is'])} {R.choice(fracs)} {R.choice(['of a', 'of', 'a', ''])} {R.choice(['cup', 'cups', 'tablespoon', 'tbsp', 'teaspoon', 'tsp'])}".replace("  ", " "),
        lambda: f"what is {R.choice(fracs)} of {R.choice(fracs)} {R.choice(['a cup', 'cup', 'of a tablespoon'])}",
        lambda: f"{R.choice(['what is', 'how long is', 'how many minutes is', 'how much is', 'whats'])} {R.choice(fracs)} {R.choice(['of', ''])} {R.choice(tunits)}{R.choice(['', ' and a half', ' in minutes', ' in seconds'])}".replace("  ", " "),
        lambda: f"{R.choice(['what is', 'how many minutes in', 'how long is'])} {R.choice(['an', 'a', 'one', 'two', '3', '1.5', 'ten'])} {R.choice(['hour', 'hours', 'day', 'days', 'week'])} {R.choice(['and a half', 'and 15 minutes', 'and twenty minutes', '30 mins', 'and forty five minutes', 'half'])}",
        lambda: f"{R.choice(['what is', 'how much is'])} {R.choice(fracs)} of {R.choice(['an hour and a half', 'two hours and 30 minutes', 'three and a half hours', 'a day and a half'])}",
        lambda: f"what fraction of {R.choice(['an hour', 'a day', 'a week', 'a minute'])} is {R.choice(['20', '15', '45', '6', '90', 'a', '2.5'])} {R.choice(['minutes', 'hours', 'seconds', 'days', 'mins'])}",
        lambda: f"{R.choice(['20', '15', '8', '36'])} {R.choice(['minutes', 'hours', 'seconds'])} as a fraction of {R.choice(['an hour', 'a day', 'a minute'])}",
        lambda: f"{R.choice(['what is', 'convert', ''])} {R.choice(['180', '200', '350', '160', '-5', '37.5', '98.6', '425', '90', '300'])}{R.choice(['', ' ', ' degrees ', '°'])}{R.choice(['c', 'f', 'celsius', 'fahrenheit', 'fan', ''])} {R.choice(['to f', 'in celsius', 'in a fan oven', 'gas mark', 'in fahrenheit', 'to c', 'for a gas oven', 'fan'])}".strip(),
        lambda: f"{R.choice(['what is', 'whats'])} gas mark {R.choice('0123456789')} {R.choice(['in celsius', 'in a fan oven', ''])}".strip(),
        lambda: f"{R.choice(['how do i convert', 'convert', ''])} {R.choice(['c', 'celsius', 'f', 'fahrenheit', 'degrees c'])} to {R.choice(['f', 'fahrenheit', 'c', 'celsius'])}".strip(),
        lambda: f"{R.choice(['how much is', 'how many grams is', 'what is', 'how many cups is'])} {R.choice(['a', 'one', 'half a', '2', '1.5', 'an'])} stick{R.choice(['', 's'])} of butter{R.choice(['', ' in grams', ' in cups', ' in a pound', ' in a block'])}",
        lambda: f"{R.choice(['how big is', 'what is', 'how large is', 'is', 'whats'])} {R.choice(['an', 'a', 'half an', '0.15 of an', '2', 'three quarters of an', '10', 'a quarter of a'])} {R.choice(['acre', 'acres', 'hectare', 'square mile', 'sq km', 'square metre', 'sq ft', 'square yard'])}{R.choice(['', ' big', ' a big garden', ' a lot'])}",
        lambda: f"{R.choice(['im', 'i am', 'my height is'])} {R.choice(['1.75 m', '175cm', '5 foot 9', '5ft 11', chr(53) + chr(39) + '6', '1.8 metres', '6 foot 2', '2.5m', '160 cm'])}{R.choice(['', ' tall', ' what size bike do i need', ' in feet', ' how many feet is that', ' from the door', ' and my feet hurt'])}",
        lambda: f"{R.choice(['1.8m', '170cm', '5ft 6', '1.65 metres', '6 foot 1'])} in {R.choice(['feet', 'cm', 'feet and inches', 'metres'])}",
        lambda: f"how {R.choice(['many', 'much'])} is a handful of {R.choice(foods + ['spinach', 'it', 'crisps'])}{R.choice(['', ' in grams', ' weigh'])}",
        lambda: f"{R.choice(['how many', 'how much'])} {R.choice(units)} {R.choice(['are there in', 'in', 'is', 'to'])} {R.choice(nums)} {R.choice(units)} should i {R.choice(['run', 'drink', 'eat'])}",
    ]
    F = [
        (lambda: [f"how much {R.choice(portions)} for {R.choice(people)} people"],
         lambda: R.choice([f"and for {R.choice(people)}", f"what about {R.choice(people)}", f"for {R.choice(people)} people then", f"make it {R.choice(['1kg', '500g', 'a kilo', 'half a kilo', '2 kg', '750 grams'])}", f"what about {R.choice(['1kg', '300g'])} instead", "so how much now", "and how much then"])),
        (lambda: [f"how much {R.choice(portions)} do i need", f"oh sorry i meant {R.choice(people)} people not 6"],
         lambda: R.choice(["so how much now", "how much now", "and now", "ok so how many then"])),
        (lambda: [f"how many grams in a cup of {R.choice(foods)}"],
         lambda: R.choice(["and in a tablespoon", "what about a teaspoon", f"what about a cup of {R.choice(foods)}", "and a mug", "in ounces", "and how long does it keep", "and half of that"])),
        (lambda: [f"what is {R.choice(fracs)} of a cup"],
         lambda: R.choice(["and half of that", "what about a third of that", "and a quarter of it", "so half of the same", "half of that please"])),
        (lambda: [R.choice([f"we are {R.choice(people)} on saturday", f"there will be {R.choice(people)} of us", f"im cooking a lasagne for {R.choice(people)}", f"actually make it for {R.choice(people)}", f"i need it for {R.choice(people)}", "its 5 in the afternoon", f"so its {R.choice(people)} now"]), R.choice(["how long should it cook for", "my oven is fan assisted", f"so its {R.choice(people)} now", "ok"])],
         lambda: R.choice([f"how much {R.choice(portions)}", f"and how much {R.choice(portions)} should i buy", f"what size {R.choice(portions)} do i need", f"how much {R.choice(portions)} per person", f"how many portions of {R.choice(portions)}"])),
    ]
    for _ in range(n):
        if R.random() < 0.8:
            yield R.choice(T)(), []
        else:
            before, now = R.choice(F)
            yield now(), before()


def main():
    out = open(sys.argv[1], "w")
    n = int(sys.argv[2]) if len(sys.argv) > 2 else 30000
    seen, k = set(), 0
    import itertools
    for clean, turns in itertools.chain(recorded(), fuzz(n)):
        key = (clean, tuple(turns))
        if key in seen:
            continue
        seen.add(key)
        out.write(json.dumps(case(clean, turns), ensure_ascii=False) + "\n")
        k += 1
    print(k, "cases", file=sys.stderr)


if __name__ == "__main__":
    main()
