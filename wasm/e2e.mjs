// The WebAssembly build's end-to-end check, as `turn_check --e2e` does natively: every
// conversation of a turn oracle runs from scratch in rsbot.wasm (fresh memory, generator seeded
// with 0, date pinned to 2026-11-17 as the oracle was recorded), and each reply must equal
// PyBot's.
//
//   node wasm/e2e.mjs ORACLE.jsonl.gz [--wasm FILE] [--data bundle.json[.gz]] [--shard I/N] [--show K]
//
// --shard I/N runs every N-th conversation starting at I (0-based), to split a set over processes.
import fs from "node:fs";
import path from "node:path";
import readline from "node:readline";
import zlib from "node:zlib";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const { makeWasi } = createRequire(import.meta.url)(path.join(here, "web/wasi.js"));

const args = process.argv.slice(2);
const opt = (name, dflt) => {
  const i = args.indexOf(name);
  return i >= 0 ? args.splice(i, 2)[1] : dflt;
};
const wasmPath = opt("--wasm", path.join(here, "target/wasm32-wasip1/release/rsbot_wasm.wasm"));
const dataPath = opt("--data", path.join(here, "../../data/export/bundle.json"));
const [shard, shards] = opt("--shard", "0/1").split("/").map(Number);
const show = Number(opt("--show", "3"));
const oracle = args[0];
if (!oracle) {
  console.error("usage: node wasm/e2e.mjs ORACLE.jsonl.gz [--wasm FILE] [--data FILE] [--shard I/N] [--show K]");
  process.exit(2);
}

const module = await WebAssembly.compile(fs.readFileSync(wasmPath));
const wasi = makeWasi({ env: { RSBOT_TODAY: process.env.RSBOT_TODAY || "2026-11-17" } });
const ex = (await WebAssembly.instantiate(module, wasi.imports(module))).exports;
wasi.setMemory(ex.memory);
ex._initialize?.();
const enc = new TextEncoder(), dec = new TextDecoder();
function call(fn, bytes) {
  const ptr = ex.rsbot_alloc(bytes.length) >>> 0;
  new Uint8Array(ex.memory.buffer, ptr, bytes.length).set(bytes);
  const len = fn(ptr, bytes.length) >>> 0;
  return JSON.parse(dec.decode(new Uint8Array(ex.memory.buffer, ex.rsbot_out_ptr() >>> 0, len)));
}
const loaded = call(ex.rsbot_load, fs.readFileSync(dataPath));
if (!loaded.ok) throw new Error("data bundle: " + loaded.error);

const t0 = performance.now();
let n = 0, ok = 0, convs = 0, conv = null, mine = false;
const shown = [];
const lines = readline.createInterface({ input: fs.createReadStream(oracle).pipe(zlib.createGunzip()), crlfDelay: Infinity });
for await (const line of lines) {
  const rec = JSON.parse(line);
  if (rec.conv !== conv) {
    conv = rec.conv;
    mine = convs++ % shards === shard;
    if (mine) ex.rsbot_reset(0n);
  }
  if (!mine) continue;
  const r = call(ex.rsbot_turn, enc.encode(rec.text ?? ""));
  n++;
  if (r.raw === (rec.reply ?? "")) ok++;
  else if (shown.length < show)
    shown.push(`${rec.conv}#${rec.turn} wasm[${r.module ?? "none"}] py[${rec.chosen ?? "none"}] text ${JSON.stringify(rec.text)}\n      wasm: ${r.raw}\n      py:   ${rec.reply}`);
}
const secs = (performance.now() - t0) / 1000;
console.log(`${path.basename(oracle)} shard ${shard}/${shards}: ${ok}/${n} replies identical (${((100 * ok) / Math.max(n, 1)).toFixed(1)}%) in ${secs.toFixed(0)}s, ` +
  `peak memory ${(ex.memory.buffer.byteLength / 2 ** 20).toFixed(0)} MB`);
for (const s of shown) console.log("  -- " + s);
process.exitCode = ok === n ? 0 : 1;
