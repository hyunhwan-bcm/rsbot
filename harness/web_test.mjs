// End-to-end UI test of rsbot_web in real Chrome: type messages, check replies, reset, themes, mobile width.
//   npm i puppeteer-core && RSBOT_URL=http://127.0.0.1:8790/ node harness/web_test.mjs
import puppeteer from "puppeteer-core";

const URL = process.env.RSBOT_URL || "http://127.0.0.1:8790/";
const browser = await puppeteer.launch({
  executablePath: process.env.CHROME || "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  headless: true,
  args: ["--no-first-run", "--no-default-browser-check"],
});
const results = [];
const check = (name, ok, detail = "") => results.push(`${ok ? "PASS" : "FAIL"} ${name}${detail ? " - " + detail : ""}`);

async function say(page, text) {
  const before = await page.$$eval(".msg.bot", (els) => els.length);
  await page.type("#text", text);
  await page.keyboard.press("Enter");
  await page.waitForFunction((n) => document.querySelectorAll(".msg.bot").length > n, { timeout: 20000 }, before);
  return page.$$eval(".msg.bot", (els) => els[els.length - 1].textContent);
}

// desktop, light
const page = await browser.newPage();
await page.setViewport({ width: 1100, height: 760 });
await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: "light" }]);
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
await page.goto(URL, { waitUntil: "networkidle0" });
check("page loads", (await page.title()) === "rsbot chat", await page.title());

const r1 = await say(page, "my name is sam and i have a dog called rex");
check("first reply greets by name", r1.startsWith("Hi Sam"), r1.slice(0, 60));
const r2 = await say(page, "how do I boil an egg?");
check("knowledge answer", r2.includes("boiling water"), r2.slice(0, 60));
const r3 = await say(page, "what's my dog's name?");
check("memory across turns", r3 === "You've got a dog called Rex.", r3);
const meta = await page.$$eval(".meta", (els) => els.map((e) => e.textContent));
check("module and time shown", meta.length === 3 && /ms$/.test(meta[2]), meta[2]);
check("user message shown", (await page.$$eval(".msg.user", (els) => els.length)) === 3);
// shift+enter makes a new line, not a send
await page.type("#text", "line one");
await page.keyboard.down("Shift"); await page.keyboard.press("Enter"); await page.keyboard.up("Shift");
await page.type("#text", "line two");
check("shift+enter keeps typing", (await page.$eval("#text", (t) => t.value)) === "line one\nline two");
await page.$eval("#text", (t) => (t.value = ""));
// empty message is not sent
await page.keyboard.press("Enter");
check("empty line not sent", (await page.$$eval(".msg.user", (els) => els.length)) === 3);
await page.screenshot({ path: "desktop_light.png" });

// new chat forgets
await page.click("#reset");
check("new chat clears the log", (await page.$$eval(".msg", (els) => els.length)) === 0);
const r4 = await say(page, "what's my dog's name?");
check("new chat has no memory", r4 !== "You've got a dog called Rex.", r4.slice(0, 70));

// two tabs are separate conversations
const other = await browser.newPage();
await other.goto(URL, { waitUntil: "networkidle0" });
const o1 = await say(other, "my name is alex");
const o2 = await say(other, "what is my name?");
check("second tab keeps its own memory", o2 === "You're Alex.", o2);

// dark mode
await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: "dark" }]);
const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
check("dark mode background", bg === "rgb(26, 26, 25)", bg);
await page.screenshot({ path: "desktop_dark.png" });

// phone width
const phone = await browser.newPage();
await phone.setViewport({ width: 390, height: 800, deviceScaleFactor: 2 });
await phone.goto(URL, { waitUntil: "networkidle0" });
await say(phone, "hello, can you help me plan meals for the week?");
const overflow = await phone.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
check("no sideways scroll on a phone", !overflow);
await phone.screenshot({ path: "phone.png" });

check("no page errors", errors.length === 0, errors.join("; "));
await browser.close();
console.log(results.join("\n"));
