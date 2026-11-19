#!/bin/sh
# Speed and memory, Python PyBot vs Rust rsbot (release), one process each, run one after another.
cd /Volumes/SSD/Workspace/rsbot
while pgrep -f "target/fast/turn_check" >/dev/null; do sleep 20; done
export RSBOT_TODAY=2026-11-17
B=runs/bench; mkdir -p $B
uptime > $B/load_at_start.txt
for set in prompts:data/smoltalkbench/all.jsonl conv500:scratch/conv500.jsonl; do
  name=${set%%:*}; inp=${set#*:}
  /usr/bin/time -l python3 rsbot/harness/run_python.py "$inp" $B/py_$name.jsonl --workers 1 2> $B/py_$name.time
  /usr/bin/time -l rsbot/target/release/rsbot --jsonl "$inp" $B/rs_$name.jsonl 2> $B/rs_$name.time
done
uptime > $B/load_at_end.txt
echo bench done
