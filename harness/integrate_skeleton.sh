#!/bin/sh
# Generate the typed skeleton and install it in the main crate (rsbot/src), keeping hand-written files.
#   harness/integrate_skeleton.sh [TYPES.json]
set -e
cd "$(dirname "$0")/.."
TYPES=${1:-runs/trace_types/types.json}
OUT=$(mktemp -d)
python3 harness/gen_skeleton.py "$OUT" --types "$TYPES" --contracts slice/contracts.json
for d in understanding knowledge compose memory modules; do
  rm -rf rsbot/src/$d.keep && mkdir -p rsbot/src/$d.keep
  # hand-written files that live inside generated packages
  for f in entry.rs registry.rs; do [ -f rsbot/src/$d/$f ] && cp rsbot/src/$d/$f rsbot/src/$d.keep/ || true; done
  rm -rf rsbot/src/$d && cp -r "$OUT/$d" rsbot/src/$d
  cp rsbot/src/$d.keep/* rsbot/src/$d/ 2>/dev/null || true
  rm -rf rsbot/src/$d.keep
done
rm -rf "$OUT"
cd rsbot && cargo build --profile fast 2>&1 | grep -E "^(error|warning: unused)" -A6 | head -40
