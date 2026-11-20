// Copy a WebAssembly module without its custom sections (the debug names: 5 MB of rsbot.wasm).
// Every other section is copied byte for byte, so the code that runs is the code that was built
// and checked (`cargo`'s own `strip = true` rebuilds and changes the code section).
//
//   node strip.mjs IN.wasm OUT.wasm
import fs from "node:fs";

const [inp, out] = process.argv.slice(2);
const b = fs.readFileSync(inp);
if (b.readUInt32LE(0) !== 0x6d736100) throw new Error(inp + " is not a WebAssembly module");
const parts = [b.subarray(0, 8)];
let p = 8;
while (p < b.length) {
  const start = p, id = b[p++];
  let size = 0, shift = 0, x;
  do {
    x = b[p++];
    size += (x & 0x7f) * 2 ** shift;
    shift += 7;
  } while (x & 0x80);
  p += size;
  if (id !== 0) parts.push(b.subarray(start, p));
}
fs.writeFileSync(out, Buffer.concat(parts));
