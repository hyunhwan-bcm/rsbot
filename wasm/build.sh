#!/usr/bin/env bash
# Build rsbot for the browser. wasm/dist/ becomes a static site that runs the bot entirely in
# the page (no server): index.html, worker.js, wasi.js, files.json, rsbot.wasm.gz and
# bundle.json.gz.
#
#   WASI_SDK=/path/to/wasi-sdk wasm/build.sh
#
# Needs:
#   - rustup with a nightly toolchain, its rust-src and the wasm32-wasip1 target
#       rustup toolchain install nightly --profile minimal --component rust-src --target wasm32-wasip1
#   - wasi-sdk (https://github.com/WebAssembly/wasi-sdk/releases), to compile PCRE2's C source
#   - PyBot's data bundle, data/export/bundle.json next to the repository (or RSBOT_DATA=FILE)
#   - Node (strip.mjs; e2e.mjs checks the build against PyBot's recorded replies)
# Optional: RSBOT_WASM_TOOLCHAIN (default nightly).
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
: "${WASI_SDK:?set WASI_SDK to a wasi-sdk directory (https://github.com/WebAssembly/wasi-sdk/releases)}"
toolchain=${RSBOT_WASM_TOOLCHAIN:-nightly}
bundle=${RSBOT_DATA:-$here/../../data/export/bundle.json}
[ -f "$bundle" ] || { echo "no data bundle at $bundle (set RSBOT_DATA)" >&2; exit 1; }

# the toolchain's own cargo and rustc first on PATH (a Homebrew rustc may shadow rustup's)
PATH="$(dirname "$(rustup which --toolchain "$toolchain" cargo)"):$PATH"
export PATH
# PCRE2 is C: compile it with wasi-sdk's clang and libc headers, without its JIT (no WebAssembly
# backend; pcre2-nojit.h)
export CC_wasm32_wasip1="$WASI_SDK/bin/clang" AR_wasm32_wasip1="$WASI_SDK/bin/llvm-ar"
export CFLAGS_wasm32_wasip1="--sysroot=$WASI_SDK/share/wasi-sysroot -include $here/pcre2-nojit.h"

cd "$here"
cargo build --release
wasm=target/wasm32-wasip1/release/rsbot_wasm.wasm

rm -rf dist
mkdir -p dist
cp web/index.html web/worker.js web/wasi.js dist/
# both served gzipped (about 17 MB together) and unpacked in the page: GitHub Pages may not
# compress .wasm, and .json.gz is never compressed again
node strip.mjs "$wasm" dist/rsbot.wasm   # drop the debug names; the code is untouched
gzip -9 -n dist/rsbot.wasm
gzip -9 -n -c "$bundle" > dist/bundle.json.gz
# sizes for the progress bar, and a version that changes with either file (cache busting)
version=$(cat dist/rsbot.wasm.gz dist/bundle.json.gz | shasum -a 256 | cut -c1-12)
size() { wc -c < "$1" | tr -d ' '; }
printf '{"version": "%s", "files": {"rsbot.wasm.gz": %s, "bundle.json.gz": %s}}\n' \
  "$version" "$(size dist/rsbot.wasm.gz)" "$(size dist/bundle.json.gz)" > dist/files.json
touch dist/.nojekyll
ls -l dist
