// The static page in real Chrome: the bot loads in the page, answers, remembers, forgets on
// "New chat", works in dark mode and at phone width, without page errors. With RSBOT_BIN (a
// native rsbot), the same conversation is also run natively and every reply must be identical.
//
//   (cd wasm/dist && python3 -m http.server 8791) &
//   npm i puppeteer-core            # in the directory you run this from
//   RSBOT_URL=http://127.0.0.1:8791/ RSBOT_BIN=target/release/rsbot node wasm/web_test.mjs
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";

const puppeteer = createRequire(process.cwd() + "/")("puppeteer-core");
const URL = process.env.RSBOT_URL || "http://127.0.0.1:8791/";
const browser = await puppeteer.launch({
  executablePath: process.env.CHROME || "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  headless: true,
  args: ["--no-first-run", "--no-default-browser-check"],
});
const results = [];
const check = (name, ok, detail = "") => results.push(`${ok ? "PASS" : "FAIL"} ${name}${detail ? " - " + detail : ""}`);

async function ready(page) {
  await page.waitForFunction(() => !document.getElementById("text").disabled, { timeout: 180000 });
}
async function say(page, text) {
  const before = await page.$$eval(".msg.bot", (els) => els.length);
  await page.type("#text", text);
  await page.keyboard.press("Enter");
  await page.waitForFunction((n) => document.querySelectorAll(".msg.bot").length > n, { timeout: 60000 }, before);
  return page.$$eval(".msg.bot", (els) => els[els.length - 1].textContent);
}

const page = await browser.newPage();
await page.setViewport({ width: 1100, height: 760 });
await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: "light" }]);
const errors = [];
const watch = (p) => {
  p.on("pageerror", (e) => errors.push(e.message));
  p.on("console", (m) => m.type() === "error" && errors.push(m.text()));
};
watch(page);
const t0 = Date.now();
await page.goto(URL);
check("page loads", (await page.title()) === "rsbot chat", await page.title());
check("input waits for the bot", await page.$eval("#text", (t) => t.disabled));
await ready(page);
check("bot loads in the page", true, `${((Date.now() - t0) / 1000).toFixed(1)} s`);

const lines = [
  "my name is sam and i have a dog called rex",
  "how do I boil an egg?",
  "what's my dog's name?",
  "Can you give me a recipe for pancakes?",
  "what day is it today?",
  "thanks, bye!",
];
const replies = [];
for (const line of lines) replies.push(await say(page, line));
check("first reply greets by name", replies[0].startsWith("Hi Sam"), replies[0].slice(0, 60));
check("knowledge answer", replies[1].includes("boiling water"), replies[1].slice(0, 60));
check("memory across turns", replies[2] === "You've got a dog called Rex.", replies[2]);
const meta = await page.$$eval(".meta", (els) => els.map((e) => e.textContent));
check("module and time shown", meta.length === lines.length && /ms$/.test(meta[2]), meta[2]);
if (process.env.RSBOT_BIN) {
  // the native bot, seeded as the page is, on today's date like the page
  const native = execFileSync(process.env.RSBOT_BIN, ["-s", "0"], { input: lines.join("\n") + "\n", encoding: "utf8" })
    .trimEnd()
    .split("\n");
  const same = native.length === replies.length && native.every((r, i) => r === replies[i]);
  check("replies identical to native rsbot", same, same ? `${replies.length} turns` : JSON.stringify({ native, replies }).slice(0, 400));
}
await page.screenshot({ path: "wasm_desktop_light.png" });

await page.click("#reset");
check("new chat clears the log", (await page.$$eval(".msg", (els) => els.length)) === 0);
const r = await say(page, "what's my dog's name?");
check("new chat has no memory", r !== "You've got a dog called Rex.", r.slice(0, 70));

await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: "dark" }]);
const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
check("dark mode background", bg === "rgb(26, 26, 25)", bg);
await page.screenshot({ path: "wasm_desktop_dark.png" });

const phone = await browser.newPage();
watch(phone);
await phone.setViewport({ width: 390, height: 800, deviceScaleFactor: 2 });
await phone.goto(URL);
await phone.waitForSelector("#loading");
const loadingOverflow = await phone.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
await ready(phone);
await say(phone, "hello, can you help me plan meals for the week?");
const overflow = await phone.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
check("no sideways scroll on a phone", !overflow && !loadingOverflow);
await phone.screenshot({ path: "wasm_phone.png" });

check("no page errors", errors.length === 0, errors.join("; "));
await browser.close();
console.log(results.join("\n"));
process.exitCode = results.every((r) => r.startsWith("PASS")) ? 0 : 1;
