//! rsbot in the browser: the bot compiled to WebAssembly (wasm32-wasip1), driven by a page.
//!
//! The page (`web/worker.js`) runs this module in a Web Worker, downloads PyBot's data bundle,
//! hands it over with [`rsbot_load`] and then sends one line at a time to [`rsbot_turn`]. One
//! module instance holds one conversation, so PyBot's global random generator belongs to it
//! alone, as in `pybot.py`; [`rsbot_reset`] starts a new one the way `rsbot_web` does (fresh
//! memory, generator seeded).
//!
//! The interface is plain C ABI (no wasm-bindgen): strings go in as UTF-8 bytes the page
//! writes into memory from [`rsbot_alloc`], and come out as JSON at [`rsbot_out_ptr`] /
//! [`rsbot_out_len`].

use rsbot::py::random;
use rsbot::state::Memory;
use std::cell::RefCell;
use std::io::Read;
use std::panic::{AssertUnwindSafe, catch_unwind};

struct Chat {
    memory: Memory,
    turns: usize,
}

thread_local! {
    static CHAT: RefCell<Chat> = RefCell::new(Chat { memory: Memory::new(), turns: 0 });
    static OUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn set_out(v: serde_json::Value) -> usize {
    OUT.with(|o| {
        let mut o = o.borrow_mut();
        *o = v.to_string().into_bytes();
        o.len()
    })
}

/// Take ownership of `len` bytes at `ptr` (from [`rsbot_alloc`]).
unsafe fn take(ptr: *mut u8, len: usize) -> Vec<u8> {
    if len == 0 {
        return Vec::new();
    }
    unsafe { Vec::from_raw_parts(ptr, len, len) }
}

/// Reserve `len` bytes for the page to write into; pass them back to a call that takes them.
#[unsafe(no_mangle)]
pub extern "C" fn rsbot_alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// Where the last result's JSON is.
#[unsafe(no_mangle)]
pub extern "C" fn rsbot_out_ptr() -> *const u8 {
    OUT.with(|o| o.borrow().as_ptr())
}

/// How long the last result's JSON is.
#[unsafe(no_mangle)]
pub extern "C" fn rsbot_out_len() -> usize {
    OUT.with(|o| o.borrow().len())
}

/// The browser's time zone, in minutes east of UTC, for `datetime.date.today()`.
#[unsafe(no_mangle)]
pub extern "C" fn rsbot_set_utc_offset(minutes: i32) {
    rsbot::py::clock::set_local_offset_minutes(minutes);
}

/// Load PyBot's data bundle: `len` bytes at `ptr` (from [`rsbot_alloc`], taken over here), JSON
/// or gzipped JSON. Returns the length of the result JSON: `{"ok": true, "globals": n}` or
/// `{"ok": false, "error": text}`.
///
/// # Safety
/// `ptr` must come from [`rsbot_alloc`]`(len)` and hold `len` written bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rsbot_load(ptr: *mut u8, len: usize) -> usize {
    rsbot::runtime::quiet_panics();
    let bytes = unsafe { take(ptr, len) };
    let out = catch_unwind(AssertUnwindSafe(|| {
        let json = if bytes.starts_with(&[0x1f, 0x8b]) {
            let mut json = Vec::new();
            flate2::read::GzDecoder::new(&bytes[..]).read_to_end(&mut json).expect("the bundle is gzip");
            drop(bytes);
            json
        } else {
            bytes
        };
        rsbot::data::supply(json);
        rsbot::data::load()
    }));
    set_out(match out {
        Ok(n) => serde_json::json!({"ok": true, "globals": n}),
        Err(_) => serde_json::json!({"ok": false, "error": rsbot::runtime::take_last_panic().unwrap_or_default()}),
    })
}

/// Start a new conversation: empty memory and the random generator seeded with `seed`, as
/// `rsbot_web` starts each conversation (and `pybot.py --seed`).
#[unsafe(no_mangle)]
pub extern "C" fn rsbot_reset(seed: i64) {
    random::seed(seed as i128);
    CHAT.with(|c| *c.borrow_mut() = Chat { memory: Memory::new(), turns: 0 });
}

/// One turn: the user's line is `len` UTF-8 bytes at `ptr` (from [`rsbot_alloc`], taken over
/// here). Returns the length of the result JSON:
/// `{"reply", "raw", "module", "ms", "turn"}`, where `reply` is what `rsbot_web` shows
/// (`pybot.py`'s one-line answer) and `raw` is the runtime's reply before whitespace is
/// collapsed (what PyBot's turn oracle records).
///
/// # Safety
/// `ptr` must come from [`rsbot_alloc`]`(len)` and hold `len` written bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rsbot_turn(ptr: *mut u8, len: usize) -> usize {
    let bytes = unsafe { take(ptr, len) };
    let text = String::from_utf8_lossy(&bytes).into_owned();
    CHAT.with(|c| {
        let mut chat = c.borrow_mut();
        let t = std::time::Instant::now();
        let mut memory = std::mem::take(&mut chat.memory);
        let out = catch_unwind(AssertUnwindSafe(|| {
            let r = rsbot::runtime::turn(&text, &mut memory);
            (r, memory)
        }));
        let ms = (t.elapsed().as_secs_f64() * 10000.0).round() / 10.0;
        chat.turns += 1;
        let (raw, module) = match out {
            Ok((r, memory)) => {
                chat.memory = memory;
                (r.reply, r.module)
            }
            // the runtime already treats a module's failure as Python does; this is the last net,
            // as in rsbot_web (the conversation's memory is lost with it)
            Err(_) => (rsbot::runtime::FALLBACK.to_string(), None),
        };
        // `runtime::answer`: whitespace collapsed, "Hello!" if nothing is left
        let squashed = rsbot::py::text::squash(&raw);
        let reply = if squashed.is_empty() { rsbot::runtime::FALLBACK.to_string() } else { squashed };
        set_out(serde_json::json!({"reply": reply, "raw": raw, "module": module, "ms": ms, "turn": chat.turns}))
    })
}
