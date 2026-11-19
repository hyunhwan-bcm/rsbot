"""Run upstream PyBot on a JSONL of prompts, one fresh single-turn conversation per row.

    python harness/run_python.py IN.jsonl OUT.jsonl [--workers N] [--seed S] [--today YYYY-MM-DD]

Each row is answered exactly as pybot.py's one-shot mode does it (fresh memory, whitespace
collapsed, "Hello!" on failure), with random.seed(S) before every row and the clock pinned to
--today, so a run is reproducible and comparable with another implementation.
"""
import argparse, datetime as _dt, json, os, random, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import workspace  # noqa: E402

PYBOT = os.path.join(workspace.root(), "upstream", "pybot")


def pin_clock(day):
    """Replace date.today / datetime.now with a fixed day (noon UTC) before the bot imports."""
    y, m, d = map(int, day.split("-"))

    class FixedDate(_dt.date):
        @classmethod
        def today(cls):
            return cls(y, m, d)

    class FixedDateTime(_dt.datetime):
        @classmethod
        def now(cls, tz=None):
            base = _dt.datetime(y, m, d, 12, 0, 0, tzinfo=_dt.timezone.utc)
            return cls.fromtimestamp(base.timestamp(), tz) if tz else cls(y, m, d, 12, 0, 0)

        @classmethod
        def today(cls):
            return cls(y, m, d, 12, 0, 0)

    _dt.date = FixedDate
    _dt.datetime = FixedDateTime


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("inp"); ap.add_argument("out")
    ap.add_argument("--workers", type=int, default=os.cpu_count())
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--today", default="2026-11-17")
    ap.add_argument("--field", default="instruction")
    ap.add_argument("--handlers", help="comma list: only these modules may respond")
    ap.add_argument("--observers", help="comma list: only these modules observe ('' for none)")
    ap.add_argument("--limit", type=int)
    ap.add_argument("--pybot", default=PYBOT, help="directory holding the bot package")
    a = ap.parse_args()
    pin_clock(a.today)
    sys.path.insert(0, os.path.abspath(a.pybot))
    t0 = time.time()
    from bot import runtime
    print(f"loaded bot in {time.time()-t0:.1f}s", file=sys.stderr)
    if a.handlers is not None:
        keep = set(a.handlers.split(","))
        runtime._HANDLERS[:] = [h for h in runtime._HANDLERS if h[0] in keep]
    if a.observers is not None:
        keep = set(filter(None, a.observers.split(",")))
        runtime._OBSERVERS[:] = [h for h in runtime._OBSERVERS if h[0] in keep]
    rows = [json.loads(l) for l in open(a.inp)][:a.limit]

    def conversation(row):
        """A row with "turns": one conversation, memory carried from turn to turn."""
        random.seed(a.seed)
        memory, replies, ms = {}, [], []
        for text in row["turns"]:
            start = time.perf_counter()
            try:
                res = runtime.turn(text, memory)
                reply = " ".join(str(res["reply"]).split()) or "Hello!"
            except Exception:
                reply = "Hello!"
            replies.append(reply)
            ms.append(round((time.perf_counter() - start) * 1000, 1))
        return {"id": row["id"], "replies": replies, "ms": ms}

    def one(row):
        if "turns" in row:
            return conversation(row)
        random.seed(a.seed)
        start = time.perf_counter()
        try:
            res = runtime.turn(row[a.field], {})
            reply = " ".join(str(res["reply"]).split()) or "Hello!"
            module = res.get("module")
            cands = [c["module"] for c in res.get("candidates", [])]
        except Exception:
            reply, module, cands = "Hello!", None, []
        return {"id": row["id"], "reply": reply, "module": module, "candidates": cands,
                "ms": round((time.perf_counter() - start) * 1000, 1)}

    import multiprocessing as mp
    if a.workers > 1:
        global _ONE
        _ONE = one
        with mp.get_context("fork").Pool(a.workers) as pool:
            results = pool.map(_call, rows, chunksize=4)
    else:
        results = [one(r) for r in rows]
    with open(a.out, "w") as f:
        for r in results:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(f"done {len(results)} rows in {time.time()-t0:.1f}s", file=sys.stderr)


def _call(row):
    return _ONE(row)


if __name__ == "__main__":
    main()
