"""lookup.find on the reading each turn of a whole conversation gets (full PyBot turns), plain and
with use_context=True: the multi-turn readings (leans_on, kept, ...) the matcher sees."""
import sys, json, copy, random, time
sys.path.insert(0, "/private/tmp/claude-501/-Volumes-SSD-Workspace-rsbot/af214818-d870-4c0f-97fe-8a12a12da545/scratchpad")
from pyenv import plainify
from bot import runtime
from bot.knowledge import lookup
src, dst, start, count = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
convs = [json.loads(l) for l in open(src)][start:start + count]
out = open(dst, "w")
t0 = time.time()
for ci, c in enumerate(convs):
    random.seed(0)
    memory = {}
    for k, text in enumerate(c["turns"]):
        for _, _, observe in runtime._OBSERVERS:
            try:
                observe(text, memory)
            except Exception:
                continue
        u = copy.deepcopy(memory.get("understood"))
        if isinstance(u, dict):
            def fid(**kw):
                try:
                    e = lookup.find(copy.deepcopy(u), **kw)
                    return e.id if e is not None else None
                except Exception as ex:
                    return "!" + type(ex).__name__
            state = random.getstate()
            rec = {"conv": c.get("id"), "turn": k, "text": text, "understood": plainify(u), "find": fid(), "ctx": fid(use_context=True)}
            random.setstate(state)
            out.write(json.dumps(rec) + "\n"); out.flush()
        candidates, copies = [], []
        for name, priority, respond in runtime._HANDLERS:
            mine = runtime._copy(memory)
            try:
                answer = respond(text, mine)
            except Exception:
                continue
            if isinstance(answer, str) and answer.strip():
                candidates.append({"module": name, "priority": priority, "reply": answer}); copies.append(mine)
        if candidates:
            i = runtime._choose(text, memory, candidates)
            reply, module = candidates[i]["reply"], candidates[i]["module"]
            if copies[i] is not memory:
                memory.clear(); memory.update(copies[i])
        else:
            reply, module = runtime.FALLBACK, None
        for _, _, after in runtime._AFTER:
            try:
                after(text, reply, memory)
            except Exception:
                continue
        turns = memory.get("turns")
        if not isinstance(turns, list):
            turns = memory["turns"] = []
        turns.append({"user": text, "bot": reply, "module": module})
    print(ci, round(time.time() - t0, 1), file=sys.stderr, flush=True)
