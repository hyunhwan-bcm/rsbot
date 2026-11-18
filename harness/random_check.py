"""Reference values for rsbot's CPython-compatible Random: prints JSON lines of seed -> draws."""
import json, random
for seed in list(range(0, 300)) + [7919 * 3 + 5000, 123456789, 2**40 + 17, 98765432101234]:
    r = random.Random(seed)
    out = {"seed": seed, "random": r.random()}
    out["choice"] = [random.Random(seed).choice(list(range(n))) for n in (1, 2, 3, 5, 7, 10, 17, 100, 1000)]
    r = random.Random(seed); out["bits1"] = [r.getrandbits(1) for _ in range(5)]
    r = random.Random(seed); out["sample3_10"] = r.sample(list(range(10)), 3)
    r = random.Random(seed); out["sample3_40"] = r.sample(list(range(40)), 3)
    r = random.Random(seed); out["sample8_100"] = r.sample(list(range(100)), 8)
    r = random.Random(seed); xs = list(range(9)); r.shuffle(xs); out["shuffle9"] = xs
    print(json.dumps(out))
