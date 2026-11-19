"""Jitter plot of the time each reply took, PyBot (Python) vs rsbot (Rust), from the benchmark runs.

    uv run --with matplotlib python harness/plot_speed.py runs/bench docs/speed_jitter.png

Reads py_prompts.jsonl, rs_prompts.jsonl, py_conv500.jsonl, rs_conv500.jsonl (written by
run_python.py and `rsbot --jsonl`; "ms" is one reply's time, or a list of them for a
conversation). One panel per input set, a log-scale time axis shared by both panels, one dot per
reply with horizontal jitter, and the median marked and labelled.
"""
import json
import random
import statistics
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

SURFACE = "#fcfcfb"
TEXT, MUTED, GRID = "#1f1f1e", "#6b6a63", "#e6e5e0"
PYTHON, RUST = "#2a78d6", "#eb6834"  # categorical slots 1 and 2 (validated: CVD dE 24.7)


def times(path):
    out = []
    for line in open(path):
        ms = json.loads(line)["ms"]
        out.extend(ms if isinstance(ms, list) else [ms])
    return out


def main():
    src, dst = sys.argv[1], sys.argv[2]
    note = sys.argv[3] if len(sys.argv) > 3 else "Time per reply, one process each, same inputs and identical replies"
    import os
    sets = [(k, t) for k, t in [("prompts", "2,198 SmolTalkBench prompts\n(one-turn chats)"),
                                ("conv500", "500 multi-turn conversations\n(1,960 turns, memory carried)")]
            if os.path.exists(f"{src}/py_{k}.jsonl") and os.path.exists(f"{src}/rs_{k}.jsonl")]
    rng = random.Random(0)
    plt.rcParams.update({"font.family": "sans-serif", "font.size": 10, "text.color": TEXT,
                         "axes.labelcolor": MUTED, "xtick.color": MUTED, "ytick.color": MUTED})
    fig, axes = plt.subplots(1, len(sets), figsize=(4.8 * len(sets) + 0.6, 5), sharey=True, facecolor=SURFACE,
                             squeeze=False)
    axes = axes[0]
    for ax, (key, title) in zip(axes, sets):
        ax.set_facecolor(SURFACE)
        for x, (_name, prefix, color) in enumerate([("Python (PyBot)", "py", PYTHON), ("Rust (rsbot)", "rs", RUST)]):
            ys = [max(t, 0.05) for t in times(f"{src}/{prefix}_{key}.jsonl")]
            xs = [x + rng.uniform(-0.28, 0.28) for _ in ys]
            ax.scatter(xs, ys, s=6, color=color, alpha=0.35, linewidths=0, rasterized=True)
            med = statistics.median(ys)
            ax.plot([x - 0.34, x + 0.34], [med, med], color=TEXT, linewidth=2, solid_capstyle="round")
            ax.annotate(f"median {med:,.0f} ms" if med >= 100 else f"median {med:.1f} ms", (x + 0.36, med), xytext=(4, 0), textcoords="offset points",
                        va="center", fontsize=9, color=TEXT)
        ax.set_xticks([0, 1], ["Python (PyBot)", "Rust (rsbot)"])
        ax.set_xlim(-0.6, 1.95)
        ax.set_yscale("log")
        ax.set_title(title, fontsize=10, color=TEXT, loc="left")
        ax.grid(axis="y", color=GRID, linewidth=0.8)
        ax.set_axisbelow(True)
        for side in ("top", "right", "left"):
            ax.spines[side].set_visible(False)
        ax.spines["bottom"].set_color(GRID)
        ax.tick_params(length=0)
    axes[0].set_ylabel("time per reply (ms, log scale)")
    fig.suptitle(note, x=0.01, ha="left", fontsize=11, color=TEXT)
    fig.tight_layout()
    fig.savefig(dst, dpi=160, facecolor=SURFACE)


if __name__ == "__main__":
    main()
