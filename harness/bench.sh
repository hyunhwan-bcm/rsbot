#!/bin/sh
# Speed and memory of one bot on one prompt set, single process, sequential (no parallel workers).
#   harness/bench.sh python-full|python-hot|rust IN.jsonl OUT_PREFIX
# Writes OUT_PREFIX.jsonl (replies with per-turn ms) and OUT_PREFIX.time (/usr/bin/time -l output).
set -e
kind=$1; inp=$2; out=$3
H=safety,memory_store,dreams,chat,answers,where_it_answers,first_line_open,the_shape_of_the_answer,greeting
case $kind in
  python-full) /usr/bin/time -l python3 harness/run_python.py "$inp" "$out.jsonl" --workers 1 2> "$out.time" ;;
  python-hot)  /usr/bin/time -l python3 harness/run_python.py "$inp" "$out.jsonl" --workers 1 --handlers $H --observers understand,intent 2> "$out.time" ;;
  rust)        /usr/bin/time -l rsbot/target/release/rsbot --jsonl "$inp" "$out.jsonl" 2> "$out.time" ;;
esac
grep -E "loaded|done|maximum resident|real" "$out.time" | sed 's/^ *//'
