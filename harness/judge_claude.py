"""SmolTalkBench-style pairwise judging with Claude (Message Batches API).

    uv run --with anthropic python harness/judge_claude.py \
        --bench data/smoltalkbench/gold.jsonl --replies runs/python_gold_seed0.jsonl \
        --out runs/judge/python_gold.jsonl [--orders both|fixed] [--limit N] [--submit]

What it reproduces from SmolTalkBench (huggingface.co/datasets/HuggingFaceSwarms/SmolTalkBench):
- AlpacaEval 2.0's weighted annotator prompt, verbatim (data/alpaca_eval/clf_prompt.txt),
  reference = Llama-3.1-70B's reply, candidate = the bot's reply.
- AlpacaEval's fixed per-prompt order: random.Random("is_switched_outputs" + instruction + "0")
  .choices([False, True]); unswitched, the reference is "m" and the bot is "M".
- WildBench's length penalty: a reply more than 500 characters longer than its reference
  scores at most a tie (0.5).

What it cannot reproduce: the published judge is DeepSeek-V4.1-Flash read through top-5
logprobs (a probability per prompt). The Claude API returns no logprobs, so each judgement is
a verdict (1 if the bot's reply is picked, else 0). With --orders both (default) every prompt is
judged in both orders and scored as the mean, which cancels position bias and gives 0, 0.5 or
1 per prompt. The "fixed" score (AlpacaEval's order only) is reported too.

Without --submit it is a dry run: it prints how many requests would be sent and an estimate of
the cost. Verdicts are cached in runs/judge/cache.jsonl by request content, so a reply judged
once (e.g. a Rust reply identical to Python's) is never sent again.
"""
import argparse
import hashlib
import json
import os
import random
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import workspace  # noqa: E402

ROOT = workspace.root()
PROMPT_FILE = os.path.join(ROOT, "data", "alpaca_eval", "clf_prompt.txt")
CACHE = os.path.join(ROOT, "runs", "judge", "cache.jsonl")
# Batch prices per million tokens (50% of standard) for the default model.
PRICE_IN, PRICE_OUT = 2.00, 10.00


def load_prompt():
    raw = open(PROMPT_FILE).read()
    system = raw.split("<|im_start|>system\n", 1)[1].split("\n<|im_end|>", 1)[0]
    user = raw.split("<|im_start|>user\n", 1)[1].split("\n<|im_end|>", 1)[0]
    return system, user


def is_switched(instruction):
    """AlpacaEval's RandomSwitchTwoColumnsProcessor, seed 0, seeded on the instruction."""
    return random.Random("is_switched_outputs" + instruction + "0").choices([False, True], k=1)[0]


def render(template, instruction, output_1, output_2):
    return (template.replace("{instruction}", instruction)
            .replace("{output_1}", output_1).replace("{output_2}", output_2))


def key_of(model, effort, system, user):
    return hashlib.sha256(json.dumps([model, effort, system, user]).encode()).hexdigest()[:32]


def load_cache():
    cache = {}
    if os.path.exists(CACHE):
        for line in open(CACHE):
            r = json.loads(line)
            cache[r["key"]] = r
    return cache


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bench", required=True)
    ap.add_argument("--replies", required=True, help="JSONL with id and reply (harness/run_python.py or rsbot --jsonl)")
    ap.add_argument("--out", required=True)
    ap.add_argument("--orders", choices=["both", "fixed"], default="both")
    ap.add_argument("--model", default="claude-opus-5-5")
    ap.add_argument("--effort", default="low")
    ap.add_argument("--max-tokens", type=int, default=4096)
    ap.add_argument("--limit", type=int)
    ap.add_argument("--submit", action="store_true", help="send the batch (otherwise a dry run)")
    a = ap.parse_args()

    system, template = load_prompt()
    bench = [json.loads(l) for l in open(a.bench)][: a.limit]
    replies = {json.loads(l)["id"]: json.loads(l)["reply"] for l in open(a.replies)}
    cache = load_cache()

    # One judgement per (prompt, order). "bot_is" says which identifier holds the bot's reply.
    jobs = []
    for row in bench:
        bot, ref = replies[row["id"]], row["output"]
        fixed = is_switched(row["instruction"])
        orders = [fixed] if a.orders == "fixed" else [fixed, not fixed]
        for switched in orders:
            o1, o2, bot_is = (bot, ref, "m") if switched else (ref, bot, "M")
            user = render(template, row["instruction"], o1, o2)
            jobs.append({"id": row["id"], "switched": switched, "fixed": switched == fixed, "bot_is": bot_is,
                         "key": key_of(a.model, a.effort, system, user), "user": user})
    todo = {j["key"]: j for j in jobs if j["key"] not in cache}
    chars = sum(len(system) + len(j["user"]) for j in todo.values())
    est_in = chars / 3.6
    est_out = 300 * len(todo)  # thinking at low effort plus a one-token answer; measured on a pilot
    print(f"{len(bench)} prompts, {len(jobs)} judgements, {len(jobs) - len(todo)} cached, {len(todo)} to send")
    print(f"estimate: ~{est_in / 1e6:.2f}M input + ~{est_out / 1e6:.2f}M output tokens "
          f"= ~${est_in / 1e6 * PRICE_IN + est_out / 1e6 * PRICE_OUT:.2f} at batch prices ({a.model}, effort {a.effort})")

    if todo and not a.submit:
        print("dry run: pass --submit to send")
        return
    if todo:
        import anthropic
        from anthropic.types.message_create_params import MessageCreateParamsNonStreaming
        from anthropic.types.messages.batch_create_params import Request

        client = anthropic.Anthropic()
        batch = client.messages.batches.create(requests=[
            Request(custom_id=k, params=MessageCreateParamsNonStreaming(
                model=a.model, max_tokens=a.max_tokens, output_config={"effort": a.effort},
                system=system, messages=[{"role": "user", "content": j["user"]}]))
            for k, j in todo.items()])
        print(f"batch {batch.id} submitted", flush=True)
        while True:
            batch = client.messages.batches.retrieve(batch.id)
            if batch.processing_status == "ended":
                break
            c = batch.request_counts
            print(f"  {batch.processing_status}: processing {c.processing}, succeeded {c.succeeded}, errored {c.errored}", flush=True)
            time.sleep(30)
        os.makedirs(os.path.dirname(CACHE), exist_ok=True)
        usage_in = usage_out = 0
        with open(CACHE, "a") as fh:
            for result in client.messages.batches.results(batch.id):
                rec = {"key": result.custom_id, "model": a.model, "effort": a.effort, "status": result.result.type}
                if result.result.type == "succeeded":
                    msg = result.result.message
                    text = next((b.text for b in msg.content if b.type == "text"), "")
                    rec.update({"text": text.strip(), "stop_reason": msg.stop_reason,
                                "in": msg.usage.input_tokens, "out": msg.usage.output_tokens})
                    usage_in += msg.usage.input_tokens
                    usage_out += msg.usage.output_tokens
                    cache[result.custom_id] = rec
                    fh.write(json.dumps(rec) + "\n")
        print(f"usage: {usage_in} input + {usage_out} output tokens = "
              f"${usage_in / 1e6 * PRICE_IN + usage_out / 1e6 * PRICE_OUT:.2f}")

    # Score.
    per = {}
    invalid = 0
    for j in jobs:
        rec = cache.get(j["key"])
        verdict = rec.get("text") if rec else None
        if verdict not in ("m", "M"):
            invalid += 1
            continue
        won = 1.0 if verdict == j["bot_is"] else 0.0
        p = per.setdefault(j["id"], {"id": j["id"], "fixed": None, "both": []})
        p["both"].append(won)
        if j["fixed"]:
            p["fixed"] = won
    bench_by_id = {r["id"]: r for r in bench}
    rows = []
    for pid, p in per.items():
        bot, ref = replies[pid], bench_by_id[pid]["output"]
        long = len(bot) - len(ref) > 500
        both = sum(p["both"]) / len(p["both"])
        rows.append({"id": pid, "fixed": p["fixed"], "both": both, "long": long,
                     "fixed_lc": None if p["fixed"] is None else (min(p["fixed"], 0.5) if long else p["fixed"]),
                     "both_lc": min(both, 0.5) if long else both, "n": len(p["both"])})
    os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)
    with open(a.out, "w") as fh:
        for r in rows:
            fh.write(json.dumps(r) + "\n")

    def mean(xs):
        xs = [x for x in xs if x is not None]
        return 100 * sum(xs) / len(xs) if xs else float("nan")

    consistent = [r for r in rows if r["n"] == 2]
    print(f"scored {len(rows)}/{len(bench)} prompts ({invalid} judgements without a clean m/M verdict)")
    print(f"win rate, both orders, length-penalised: {mean(r['both_lc'] for r in rows):.1f}%")
    print(f"win rate, AlpacaEval order only, length-penalised: {mean(r['fixed_lc'] for r in rows):.1f}%")
    if consistent:
        flips = sum(r["both"] == 0.5 for r in consistent)
        print(f"position-dependent verdicts: {flips}/{len(consistent)} prompts")


if __name__ == "__main__":
    main()
