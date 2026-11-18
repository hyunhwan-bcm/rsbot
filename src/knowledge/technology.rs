//! Port of PyBot's `bot/knowledge/technology.py`: phones, laptops, wifi, passwords and the
//! everyday fixes, with each answer led by the machine the line names, and the guards that keep a
//! tech card off a line about glasses, a kitchen bin, a paper form or a person's habit.
use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g6 as dup;
use crate::knowledge::d1_shared::{self as sh, ShapeFn};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.knowledge.technology.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
/// A string global of this module.
fn s(name: &str) -> String {
    g(name).str().to_string()
}

macro_rules! strs {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($name)).strs());)*
    };
}
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> = LazyLock::new(|| rx(stringify!($name)));)*
    };
}

strs!(_WIPE_ASKS, WALLET_NAMED, DEBT_SHAPE, _TAP_PROBLEM, _MAC_WORDS, _DESKTOP_WORDS, VACUUM_NAMES, FILTER_MACHINES);
regexes!(
    _SPECTACLES_IN_THE_LINE,
    _PRICE_ASKED,
    _A_DAMAGE_IN_THE_LINE,
    _A_PHONE_WORD,
    _GLASS_IN_SOMETHING_ELSE,
    _A_SCRATCH_IN_THE_LINE,
    _A_BREAK_IN_THE_LINE,
    PLATFORM_MAC,
    PLATFORM_WINDOWS,
    COMPUTER_WORDS,
    FAN_WORDS,
    _SIM_FAULT,
    _SIM_HOW_TO,
    _A_FULL_BOX,
    _JUNK_SAID,
    _WIFI_WORD,
    _SCREEN_NAMED_RE,
    _A_PAPER_APPLICATION,
    _ON_A_DEVICE,
    CABLE_PREVENT_ASK,
    CABLE_DAMAGE_NOW,
    _A_PHONE_IN_THE_LINE,
    FIRST_PHONE_TIME,
    FIRST_PHONE_TIGHT,
    FIRST_PHONE_SPEND,
    RINGING_ME,
    PERSON_WHO_RINGS,
    MARKETING_CALL,
    _A_DURATION_ASK,
    _SOMETHING_TO_CANCEL,
    _THE_PHONE_CASE,
    _PREVENTION,
    _STICKY,
    ALARM_ITSELF,
    WAKING_IS_HAPPENING,
    ASKING_ABOUT_WAKING,
    _FORMATTED_BY_MISTAKE,
    _A_STICK_OR_CARD,
    _A_HARD_DISK,
    _A_COUNTED_BAR,
    NOWHERE_TO_KEEP_STILL,
    IN_A_FOLDER,
    A_CARD,
    A_DRIVE,
);

static _A_SIGNAL_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| g("_A_SIGNAL_WORDS").str_set());

/// `(name, text)` pairs of a `{str: str}` dict global, in dict order.
fn str_pairs(name: &str) -> Vec<(String, String)> {
    g(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
}

static SERVICE_ACCOUNTS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("SERVICE_ACCOUNTS"));
static SOCIAL_DELETE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("SOCIAL_DELETE"));
static FREE_PLATFORM_NAMES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("FREE_PLATFORM_NAMES").items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string())).collect()
});
static _SYMBOLS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    g("_SYMBOLS").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
});
/// `_TECH_WORDS`: (keys, reply, avoid) per word.
static _TECH_WORDS: LazyLock<Vec<(Vec<String>, String, Vec<String>)>> = LazyLock::new(|| {
    g("_TECH_WORDS")
        .items()
        .iter()
        .map(|r| (r.items()[0].strs(), r.items()[1].str().to_string(), r.items()[2].strs()))
        .collect()
});

/// The words of this line as typed, lower-cased and squashed, for a guard on the whole reading.
pub fn _the_line(understood: &Understood) -> String {
    // `raw` is not a key of the reading: None, so it falls through like an empty field
    let mut said = sh::clean_or_text(understood).to_string();
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    text::squash(&said.to_lowercase())
}

/// A laptop on the charger is the charging row's question, not the sleep choice.
pub fn _not_a_charging_ask(understood: &Understood) -> bool {
    static LAPTOP: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:laptop|macbook|notebook)\b", 0));
    let said = _the_line(understood);
    if !said.contains("charg") {
        return true;
    }
    !LAPTOP.is_match(&said)
}

/// Whether this line is about the glass in a phone (not spectacles, a price, a scratch, or
/// another glass thing in the house).
pub fn _the_glass_in_a_phone(understood: &Understood) -> bool {
    let said = _the_line(understood);
    if _SPECTACLES_IN_THE_LINE.is_match(&said) {
        return false;
    }
    if _PRICE_ASKED.is_match(&said) && !_A_DAMAGE_IN_THE_LINE.is_match(&said) {
        return false;
    }
    if _A_SCRATCH_IN_THE_LINE.is_match(&said) && !_A_BREAK_IN_THE_LINE.is_match(&said) {
        return false;
    }
    if _GLASS_IN_SOMETHING_ELSE.is_match(&said) && !_A_PHONE_WORD.is_match(&said) {
        return false;
    }
    true
}

/// "mac", "windows" or "" - the machine the line named, if it named one.
pub fn _platform_named(reading: &Understood) -> &'static str {
    let said = text::squash(&sh::text_or_clean(reading).to_lowercase());
    if PLATFORM_MAC.is_match(&said) {
        return "mac";
    }
    if PLATFORM_WINDOWS.is_match(&said) {
        return "windows";
    }
    ""
}

/// True when a fan the line names is a computer's, the only fan this row can answer for.
pub fn _a_fan_is_the_computers(reading: &Understood) -> bool {
    let said = text::squash(&sh::text_or_clean(reading).to_lowercase());
    if !FAN_WORDS.is_match(&said) {
        return true;
    }
    COMPUTER_WORDS.is_match(&said)
}

/// The steps for the machine the line named first, the other one after them.
pub fn _mac_first(mac_steps: &str, windows_steps: &str, reading: &Understood) -> String {
    if _platform_named(reading) == "mac" {
        return format!("{mac_steps} {windows_steps}");
    }
    format!("{windows_steps} {mac_steps}")
}

/// The wifi answer only for a line that names the wifi.
pub fn _wifi_drops(_memory: &Memory, reading: &Understood) -> Option<String> {
    let said = reading.clean.to_lowercase();
    let wifi = [
        "wifi", "wi-fi", "wi fi", "wireless", "wlan", "network", "connection", "connect", "internet", "online", "data",
        "signal",
    ];
    if !wifi.iter().any(|w| said.contains(w)) {
        return None;
    }
    Some(
        "Forget the network on the phone and rejoin it with the password typed fresh, since a saved network that \
         half-connects is the commonest cause; then restart the phone and the router once. If the wifi drops when \
         the screen is off, turn off the battery saving that sleeps wifi (on Android, Settings > Battery > Battery \
         saver; on an iPhone, Low Power Mode), and set the wifi to stay on during sleep. A phone that drops only in \
         one room is a coverage problem, not a phone problem."
            .to_string(),
    )
}

/// The battery-health check for the machine the line names, that machine's way first.
pub fn _battery_health(_memory: &Memory, reading: &Understood) -> String {
    _mac_first(&s("BATTERY_HEALTH_MAC"), &s("BATTERY_HEALTH_WINDOWS"), reading) + &s("BATTERY_HEALTH_TAIL")
}

/// Emptying the bin, on the machine the line named.
pub fn _empty_trash(_memory: &Memory, reading: &Understood) -> String {
    let mac = "On a Mac, open the Bin from the Dock and press Empty in the top right, or press Command-Shift-Delete \
               to do it from the keyboard; holding Option while you press Empty skips the question. Nothing is freed \
               until the Bin is emptied, so a full disk is often just this.";
    let windows = "On Windows, right-click the Recycle Bin on the desktop and choose Empty Recycle Bin, or open it and \
                   press Empty Recycle Bin at the top; the files are only really gone once you do, so a full disk is \
                   often just this.";
    _mac_first(mac, windows, reading)
}

/// Closing one stuck app, on the machine the line named.
pub fn _force_quit(_memory: &Memory, reading: &Understood) -> String {
    let mac = "On a Mac press Command-Option-Esc, pick the app in the list and press Force Quit: that closes the one \
               app and leaves everything else open, and if the whole Mac is stuck, hold the power button for ten \
               seconds instead.";
    let windows = "On Windows press Ctrl-Shift-Esc for Task Manager, pick the app and choose End task: that closes the \
                   one app and leaves everything else open, and if the whole machine is stuck, hold the power button \
                   for ten seconds instead.";
    _mac_first(mac, windows, reading)
}

/// Setting the browser that opens links, on the machine the line named.
pub fn _default_browser(_memory: &Memory, reading: &Understood) -> String {
    let mac = "On a Mac, open System Settings > Desktop & Dock and scroll to Default web browser near the bottom (on \
               older macOS it is Safari > Preferences > General > Default web browser). Pick the one you want from \
               the menu and every link opens there from then on.";
    let windows = "On Windows, open Settings > Apps > Default apps, type the browser's name in the search box and \
                   choose Set default; that sends links, .html files and mail links to it.";
    _mac_first(mac, windows, reading)
}

/// Uninstalling software, for the machine the line names.
pub fn _uninstall_program(_memory: &Memory, reading: &Understood) -> String {
    _mac_first(&s("UNINSTALL_PROGRAM_MAC"), &s("UNINSTALL_PROGRAM_WINDOWS"), reading) + &s("UNINSTALL_PROGRAM_TAIL")
}

/// Deleting an app on the machine the line names: a Mac and a PC are not phones.
pub fn _uninstall_app(_memory: &Memory, reading: &Understood) -> String {
    match _platform_named(reading) {
        "mac" => s("UNINSTALL_APP_MAC") + &s("UNINSTALL_APP_TAIL"),
        "windows" => s("UNINSTALL_APP_WINDOWS") + &s("UNINSTALL_APP_TAIL"),
        _ => s("UNINSTALL_APP_PHONE") + &s("UNINSTALL_APP_TAIL"),
    }
}

/// Which service is being closed decides the page; the rest is one shape of advice.
pub fn _service_delete_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::clean_or_text(understood).to_lowercase();
    if let Some((_, line)) = SERVICE_ACCOUNTS.iter().find(|(name, _)| said.contains(name.as_str())) {
        return line.clone();
    }
    "Close it from the service's own website in a browser rather than from the app: the button is usually the last \
     item in your account or profile settings, called Delete or Close account. Spend or download anything you have \
     paid for first - credit and files are lost - and shut down any subscription on the same account, or it keeps \
     billing."
        .to_string()
}

/// Which account is being deleted decides which settings path to give.
pub fn _social_delete_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::clean_or_text(understood).to_lowercase();
    if let Some((_, line)) = SOCIAL_DELETE.iter().find(|(name, _)| said.contains(name.as_str())) {
        return line.clone();
    }
    "Do it in the app, not by deleting the icon: every app hides it under Settings > Account, usually as Deactivate \
     (reversible) or Delete (permanent). Deactivating keeps your posts and your name; deleting does not, and most \
     apps give you 30 days to log back in and change your mind."
        .to_string()
}

/// The SIM card: a fault (the default) or how to put one in.
pub fn _sim_reply(_memory: &Memory, understood: &Understood) -> String {
    let line = sh::text_or_clean(understood);
    if _SIM_FAULT.is_match(line) {
        return s("_SIM_FAULT_REPLY");
    }
    if _SIM_HOW_TO.is_match(line) {
        return s("_SIM_HOW_TO_REPLY");
    }
    s("_SIM_FAULT_REPLY")
}

/// The spam row answers a box full of junk; a box that is merely full belongs to storage.
pub fn _junk_or_a_full_box(understood: &Understood) -> bool {
    let line = sh::text_or_clean(understood);
    if _A_FULL_BOX.is_match(line) {
        return _JUNK_SAID.is_match(line);
    }
    true
}

/// The smart meter answer the line asked for: a top-up, or a reading and a blank display.
pub fn _meter_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::clean_or_text(understood).to_lowercase();
    let top_up = ["top up", "topping up", "topup", "credit", "prepayment", "prepayment meter", "key meter", "emergency credit"];
    if top_up.iter().any(|w| said.contains(w)) {
        return s("METER_TOP_UP");
    }
    s("METER_READING")
}

/// Where the router goes, with the verdict first when the line asks whether to move it.
pub fn _router_place(_memory: &Memory, understood: &Understood) -> String {
    let said = format!(" {} ", understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" "));
    if !said.contains(" where ") && (said.contains(" move ") || said.contains(" moving ")) {
        return format!(
            "Yes, if it is in a cupboard, behind the TV, on the floor or at the far end of the flat, and it is worth \
             the five minutes. {}",
            s("_ROUTER_WHERE")
        );
    }
    s("_ROUTER_WHERE")
}

/// A wifi part has a wifi word in the line, or in the line it follows.
pub fn _wifi_is_what_it_is_about(understood: &Understood) -> bool {
    if _WIFI_WORD.is_match(&understood.words.join(" ")) {
        return true;
    }
    let lean = understood.leans_on.as_ref().map(|l| l.words.join(" ")).unwrap_or_default();
    _WIFI_WORD.is_match(&lean)
}

/// Finding the wifi password, changing it, and getting back in are three different jobs.
pub fn _wifi_password_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::clean_or_text(understood).to_lowercase();
    let change = ["change", "changed", "new password", "different", "someone knows", "hacked", "forgot how"];
    if change.iter().any(|w| said.contains(w)) {
        return s("WIFI_CHANGE");
    }
    s("WIFI_STICKER")
}

/// The reset how-to, or what the reset costs when that is what is asked.
pub fn _router_factory_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::clean_or_text(understood).to_lowercase();
    if sh::any_in(&said, &_WIPE_ASKS) {
        return s("ROUTER_WIPED");
    }
    s("ROUTER_FACTORY")
}

/// The reset hole in full for a how-to, and the short of it for a statement.
pub fn _admin_locked_reply(_memory: &Memory, understood: &Understood) -> String {
    if understood.kind == "question" {
        return s("WIFI_RESET");
    }
    s("WIFI_LOCKED_SHORT")
}

/// `" ".join(str(understood.get(key) or "") for key in ("text", "norm_text", "clean", "raw"))`:
/// `raw` is never a key of the reading, so it adds an empty last part.
fn four_keys(understood: &Understood) -> String {
    format!("{} {} {} ", understood.text, understood.norm_text, understood.clean)
}

/// True when the line names a screen, a page or a device the size card can be about.
pub fn _a_screen_is_named(understood: &Understood) -> bool {
    _SCREEN_NAMED_RE.is_match(&four_keys(understood))
}

/// False when the application the line names is one somebody fills in.
pub fn _an_app_not_a_paper(understood: &Understood) -> bool {
    let said = four_keys(understood);
    if !_A_PAPER_APPLICATION.is_match(&said) {
        return true;
    }
    _ON_A_DEVICE.is_match(&said)
}

/// True unless the line asks how to stop a cable fraying, which the care answer owns.
pub fn _cable_fault_not_the_care_question(understood: &Understood) -> bool {
    let said = four_keys(understood);
    if !CABLE_PREVENT_ASK.is_match(&said) {
        return true;
    }
    CABLE_DAMAGE_NOW.is_match(&said)
}

/// The four keys lower-cased and padded with a space each side.
fn four_keys_padded(understood: &Understood) -> String {
    format!(" {} ", four_keys(understood).to_lowercase())
}

/// True when the line is about the phone's own block list, not about a person's habit.
pub fn _the_block_is_the_phones(understood: &Understood) -> bool {
    _A_PHONE_IN_THE_LINE.is_match(&four_keys_padded(understood))
}

/// False when the line is about money owed, so the tap answer stands down for the debt's row.
pub fn _a_wallet_is_named(understood: &Understood) -> bool {
    let said = four_keys_padded(understood);
    if WALLET_NAMED.iter().any(|w| said.contains(&format!(" {w}"))) {
        return true;
    }
    !DEBT_SHAPE.iter().any(|w| said.contains(&format!(" {w}")))
}

/// Paying by phone: the tap that does not work, or adding the card to the wallet.
pub fn _card_on_a_phone(_memory: &Memory, understood: &Understood) -> String {
    let said = format!("{} {} {}", understood.text, understood.norm_text, understood.clean).to_lowercase();
    if sh::any_in(&said, &_TAP_PROBLEM) {
        return s("_TAP_REPLY");
    }
    s("_ADD_TO_WALLET")
}

/// Dark mode on the machine the line names: a Mac, a Windows machine, or (by default) a phone.
pub fn _dark_mode_reply(_memory: &Memory, understood: &Understood) -> String {
    let line = sh::clean_or_text(understood).to_lowercase();
    let flat = line.replace('?', " ").replace(',', " ");
    let words: HashSet<&str> = text::split_ref(&flat).into_iter().collect();
    if _MAC_WORDS.iter().any(|w| words.contains(w.as_str())) || line.contains("macbook") {
        return "On a Mac it is System Settings > Appearance > Dark, and System Settings > Displays has Night Shift \
                for the warmer evening light. Safari follows the system, and most websites switch too, so nothing \
                else needs setting."
            .to_string();
    }
    if _DESKTOP_WORDS.iter().any(|w| words.contains(w.as_str())) || line.contains("laptop") {
        return "On Windows 11 it is Settings > Personalisation > Colours and pick Dark under 'Choose your mode'; \
                Windows 10 has the same page under Settings > Personalisation > Colours. Switch on Night light there \
                as well if the screen is the bright thing in the room at night."
            .to_string();
    }
    s("_DARK_MODE_PHONE")
}

/// The money for a first phone, and the age if that is what was asked ("" for screen time).
pub fn _first_phone(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::clean_or_text(understood).to_lowercase();
    if FIRST_PHONE_TIME.is_match(&said) {
        return String::new();
    }
    if FIRST_PHONE_TIGHT.is_match(&said) {
        return s("FIRST_PHONE_BUDGET");
    }
    if FIRST_PHONE_SPEND.is_match(&said) {
        return s("FIRST_PHONE_BUYING");
    }
    s("FIRST_PHONE_AGE")
}

/// The line of the answer for the symbol the person actually asked about.
pub fn _symbol_on_the_keyboard(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = sh::clean_or_text(understood).to_lowercase();
    let (_, line) = _SYMBOLS.iter().find(|(names, _)| sh::any_in(&said, names))?;
    Some(format!(
        "On a UK keyboard {line}. If a symbol comes out as a different one, the keyboard is set to the US layout: \
         Settings > Time & language > Language on Windows, System Settings > Keyboard > Input Sources on a Mac."
    ))
}

/// A laptop to a TV is a cable; a phone is a cast. Which one was named decides the answer.
pub fn _to_the_tv(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    let phone = ["phone", "iphone", "android", "tablet", "ipad"].iter().any(|w| clean.contains(w));
    if phone {
        return "Cast it rather than cable it: put the phone and the TV on the same wifi, open the app (YouTube, \
                iPlayer, Netflix) and tap the Cast or AirPlay icon at the top of the video, then pick the TV. For the \
                whole screen, iPhone use Screen Mirroring in Control Centre with an Apple TV or an AirPlay set, \
                Android use Smart View or Cast in the quick settings. If the TV does not appear, it is the wifi band: \
                both have to be on the same network, and a guest network or the 5GHz-only setting hides it. A \
                Lightning or USB-C to HDMI adapter is the wired fallback and never drops."
            .to_string();
    }
    "A cable is the reliable way: an HDMI lead from the laptop to the TV, then press Source or Input on the TV \
     remote and pick the socket you used; a laptop with no HDMI port needs a USB-C to HDMI adapter, which does the \
     same job. Once the picture is up, pick the TV as the sound output - the speaker icon in the Windows tray, or \
     System Settings > Sound on a Mac - because the laptop keeps playing sound otherwise. No cable to hand and the \
     app has a Cast icon: that works too, with both on the same wifi."
        .to_string()
}

/// The nuisance-calls card, or None when the ringing in the line is a person they know.
pub fn _cold_calls(_memory: &Memory, reading: &Understood) -> Option<String> {
    let line = format!(" {} ", text::squash(&sh::clean_or_text(reading).to_lowercase()));
    if RINGING_ME.is_match(&line) && PERSON_WHO_RINGS.is_match(&line) && !MARKETING_CALL.is_match(&line) {
        return None; // somebody they know is doing the ringing: not a cold caller
    }
    Some(s("COLD_CALLS"))
}

/// False when the line asks how long a renewal takes and names nothing to cancel.
pub fn _a_renewal_that_takes_time(reading: &Understood) -> bool {
    let said = format!(" {} ", sh::clean_or_text(reading).to_lowercase());
    !(_A_DURATION_ASK.is_match(&said) && !_SOMETHING_TO_CANCEL.is_match(&said))
}

/// A phone screen: one already cracked, a cracked case, or one the person wants kept whole.
pub fn _the_phone_screen(_memory: &Memory, reading: &Understood) -> String {
    let said = format!(" {} ", reading.clean.to_lowercase());
    if _THE_PHONE_CASE.is_match(&said) && _A_BREAK_IN_THE_LINE.is_match(&said) {
        return "A cracked case is the cheap part going first, which is what it is for: take it off and look at the \
                corners and the screen under it, because a case that took a fall is telling you the phone landed on \
                its edge. Replace it - £8 to £25 buys one with a raised lip round the screen and a rigid back, and a \
                soft silicone one cushions a drop better than a hard shell. If the phone itself is bent, or the \
                screen has a mark that was not there before, get it looked at before the crack in the case hides \
                something worse."
            .to_string();
    }
    if !_PREVENTION.is_match(&said) {
        return "Put a screen protector and a case on it now, so the crack does not spread or cut your finger, and back \
                the phone up before you book a repair. A local repair shop is usually £60 to £120, well under a new \
                phone; go to the maker if it is under warranty or water resistant."
            .to_string();
    }
    "A tempered glass screen protector and a case with a raised lip round the screen are what keep it whole, and the \
     two together cost £8 to £15: the protector takes the scratch or the fall and is swapped for a few pounds, which \
     is the point of it. Keep the phone out of a pocket with keys or grit in it, and put it face down when you set \
     it down."
        .to_string()
}

/// A laptop or a desktop that got drink, rain or a spill in it: the power stays off first.
pub fn _laptop_water(_memory: &Memory, reading: &Understood) -> String {
    let said = text::squash(&sh::text_or_clean(reading).to_lowercase());
    if _STICKY.is_match(&said) {
        return "Turn it off and unplug it now, and leave it off: coffee or juice dries sticky, and current through a \
                wet board is what kills the machine. Wipe what you can reach, keep it upside down with the lid open \
                somewhere warm for two days, then get a repair shop to clean the board, which is \u{a3}40 to \u{a3}80 \
                rather than a new laptop."
            .to_string();
    }
    "Turn it off and unplug the charger straight away, and do not switch it back on to see if it works: power \
     through a wet board is what does the damage. Tip it the way the water went in, then leave it open and upside \
     down somewhere warm and airy for two days - rice does nothing except leave grit in the ports. Back the files up \
     the moment it runs again, and if it stays dead a repair shop's board check is \u{a3}40 to \u{a3}80."
        .to_string()
}

/// The shape: a charge level of 20% or under in the line, so the phone needs saving now.
pub fn _running_out_of_charge(line: &str) -> bool {
    static FIGURE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d{1,3})\s*(?:%|percent)", 0));
    FIGURE.findall_str(line).iter().any(|f| f.parse::<i64>().is_ok_and(|n| n <= 20))
}

/// The vacuum's filter, unless the filter in the talk is another machine's ("" then).
pub fn _vacuum_filter_reply(memory: &Memory, understood: &Understood) -> String {
    let said = sh::clean_or_text(understood).to_lowercase();
    if sh::any_in(&said, &VACUUM_NAMES) {
        return s("VACUUM_FILTER");
    }
    let mut heard = dup::memory_topics_context(memory, 3).join(" ").to_lowercase();
    heard.push(' ');
    heard.push_str(&dup::py_repr(memory.extra.get("topic").unwrap_or(&Value::None)));
    if sh::any_in(&heard, &FILTER_MACHINES) {
        return String::new();
    }
    s("VACUUM_FILTER")
}

/// True when the line is about the alarm or asks to be woken, false when it reports waking.
pub fn _a_request_to_be_woken(understood: &Understood) -> bool {
    let line = sh::clean_or_text(understood).to_lowercase();
    if ALARM_ITSELF.is_match(&line) {
        return true;
    }
    !(WAKING_IS_HAPPENING.is_match(&line) || ASKING_ABOUT_WAKING.is_match(&line))
}

/// Name the app, say it is free, and give the steps for a payment that is not the app's.
pub fn _free_app_reply(_memory: &Memory, understood: &Understood) -> String {
    let said = format!("{} {} {} ", understood.clean, understood.norm_text, understood.text).to_lowercase();
    let name = FREE_PLATFORM_NAMES
        .iter()
        .find(|(word, _)| said.contains(word.as_str()))
        .map(|(_, pretty)| pretty.as_str())
        .unwrap_or("That app");
    format!(
        "{name} is free: there is no subscription to cancel, so a payment going out is an in-app purchase or a \
         subscription bought through the app store. Cancel that in Settings > your name > Subscriptions on an \
         iPhone, or Play Store > Payments and subscriptions on Android. To leave the account itself, deactivate it in \
         the app's settings - deactivation hides it until you sign in again, and deletion is permanent."
    )
}

/// A stick or card wants exFAT; a disk being sold wants the files over-written ("" for a format
/// done by mistake).
pub fn _format_a_drive(_memory: &Memory, understood: &Understood) -> String {
    let clean = sh::clean_or_text(understood).to_lowercase();
    if _FORMATTED_BY_MISTAKE.is_match(&clean) {
        return String::new();
    }
    if !_A_STICK_OR_CARD.is_match(&clean) && _A_HARD_DISK.is_match(&clean) {
        return "Back everything up first, because a format wipes the disk: then on Windows right-click the drive in \
                File Explorer and choose Format (NTFS unless it is going to a Mac), or on a Mac open Disk Utility, \
                choose the disk and click Erase. For a disk you are selling, a quick format only takes out the index \
                and the files can still be read, so overwrite the whole disk instead - `cipher /w` in Windows, or a \
                free tool such as DBAN."
            .to_string();
    }
    "Pick exFAT for a stick or card that moves between Windows and a Mac, and FAT32 for one that goes in a TV, a car \
     stereo or a camera; NTFS only if it stays on Windows. On Windows right-click the drive in File Explorer and \
     choose Format, set the file system and tick Quick format; on a Mac open Disk Utility, choose the drive, click \
     Erase and pick the format. It deletes everything on it, so copy anything you want to keep first - and a stick \
     that arrives showing as unformatted is sorted out the same way."
        .to_string()
}

/// The bin in the sentence: the computer's only when the sentence names a machine.
pub fn _the_bin_is_the_one_on_the_computer(understood: &Understood) -> bool {
    if !understood.words.iter().any(|w| w == "bin" || w == "bins") {
        return true; // the row is reached by a name only the machine has ("trash")
    }
    dup::the_bin_is_on_a_machine(understood)
}

/// False for a bare "bar" that is not a phone signal.
pub fn _a_signal_reading(understood: &Understood) -> bool {
    let text = sh::clean_or_text(understood).to_lowercase();
    if understood.words.iter().any(|w| _A_SIGNAL_WORDS.contains(&w.to_lowercase())) {
        return true;
    }
    _A_COUNTED_BAR.is_match(&text)
}

/// The bin and the version history, or - on a stick or a card - stop writing to the drive.
pub fn _recover_deleted(_memory: &Memory, understood: &Understood) -> String {
    let clean = sh::clean_or_text(understood).to_lowercase();
    if NOWHERE_TO_KEEP_STILL.is_match(&clean) && !IN_A_FOLDER.is_match(&clean) {
        let thing = if A_CARD.is_match(&clean) {
            "card"
        } else if A_DRIVE.is_match(&clean) {
            "drive"
        } else {
            "stick"
        };
        return s("RECOVER_ON_A_DRIVE").replace("{thing}", thing);
    }
    s("RECOVER_IN_A_BIN")
}

/// What a tech word means (modem, cookie, ...), unless the line is about a fault with it.
pub fn _tech_word(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = understood.clean.as_str();
    let words = &understood.words;
    let said = |w: &String| text.contains(w.as_str()) || words.contains(w);
    for (keys, reply, avoid) in _TECH_WORDS.iter() {
        if !keys.iter().any(said) {
            continue;
        }
        if avoid.iter().any(said) {
            continue;
        }
        return Some(reply.clone());
    }
    None
}

/// A watch line is this card's only when the line is about a mark on the glass.
pub fn _the_watch_is_marked(understood: &Understood) -> bool {
    const MARKS: [&str; 19] = [
        "scratch", "scratches", "scratched", "scuff", "scuffs", "scuffed", "polish", "polished", "polishing", "buff",
        "hairline", "mark", "marks", "dull", "hazy", "haze", "worn", "crystal", "chipped",
    ];
    understood.words.iter().any(|w| MARKS.contains(&w.to_lowercase().as_str()))
}

// --- registry wrappers ------------------------------------------------------------------------

macro_rules! guards {
    ($($w:ident => $f:ident),* $(,)?) => {
        $(fn $w(_f: &FnRef, u: &Understood) -> bool { $f(u) })*
    };
}
guards!(
    g_a_fan_is_the_computers => _a_fan_is_the_computers,
    g_a_renewal_that_takes_time => _a_renewal_that_takes_time,
    g_a_request_to_be_woken => _a_request_to_be_woken,
    g_a_screen_is_named => _a_screen_is_named,
    g_a_signal_reading => _a_signal_reading,
    g_a_wallet_is_named => _a_wallet_is_named,
    g_an_app_not_a_paper => _an_app_not_a_paper,
    g_cable_fault_not_the_care_question => _cable_fault_not_the_care_question,
    g_junk_or_a_full_box => _junk_or_a_full_box,
    g_not_a_charging_ask => _not_a_charging_ask,
    g_the_bin_is_the_one_on_the_computer => _the_bin_is_the_one_on_the_computer,
    g_the_block_is_the_phones => _the_block_is_the_phones,
    g_the_glass_in_a_phone => _the_glass_in_a_phone,
    g_the_watch_is_marked => _the_watch_is_marked,
    g_wifi_is_what_it_is_about => _wifi_is_what_it_is_about,
);

macro_rules! replies {
    ($($w:ident => $f:ident : $kind:tt),* $(,)?) => {
        $(fn $w(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { replies!(@call $kind, $f, m, u) })*
    };
    (@call str, $f:ident, $m:ident, $u:ident) => { Some($f($m, $u)) };
    (@call opt, $f:ident, $m:ident, $u:ident) => { $f($m, $u) };
}
replies!(
    r_admin_locked_reply => _admin_locked_reply: str,
    r_battery_health => _battery_health: str,
    r_card_on_a_phone => _card_on_a_phone: str,
    r_cold_calls => _cold_calls: opt,
    r_dark_mode_reply => _dark_mode_reply: str,
    r_default_browser => _default_browser: str,
    r_empty_trash => _empty_trash: str,
    r_first_phone => _first_phone: str,
    r_force_quit => _force_quit: str,
    r_format_a_drive => _format_a_drive: str,
    r_free_app_reply => _free_app_reply: str,
    r_laptop_water => _laptop_water: str,
    r_meter_reply => _meter_reply: str,
    r_recover_deleted => _recover_deleted: str,
    r_router_factory_reply => _router_factory_reply: str,
    r_router_place => _router_place: str,
    r_service_delete_reply => _service_delete_reply: str,
    r_sim_reply => _sim_reply: str,
    r_social_delete_reply => _social_delete_reply: str,
    r_symbol_on_the_keyboard => _symbol_on_the_keyboard: opt,
    r_tech_word => _tech_word: opt,
    r_the_phone_screen => _the_phone_screen: str,
    r_to_the_tv => _to_the_tv: str,
    r_uninstall_app => _uninstall_app: str,
    r_uninstall_program => _uninstall_program: str,
    r_vacuum_filter_reply => _vacuum_filter_reply: str,
    r_wifi_drops => _wifi_drops: opt,
    r_wifi_password_reply => _wifi_password_reply: str,
);

fn shape_running_out_of_charge(_f: &FnRef, line: &str) -> bool {
    _running_out_of_charge(line)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.technology:_a_fan_is_the_computers:137", g_a_fan_is_the_computers),
    ("bot.knowledge.technology:_a_renewal_that_takes_time:963", g_a_renewal_that_takes_time),
    ("bot.knowledge.technology:_a_request_to_be_woken:1139", g_a_request_to_be_woken),
    ("bot.knowledge.technology:_a_screen_is_named:664", g_a_screen_is_named),
    ("bot.knowledge.technology:_a_signal_reading:1246", g_a_signal_reading),
    ("bot.knowledge.technology:_a_wallet_is_named:742", g_a_wallet_is_named),
    ("bot.knowledge.technology:_an_app_not_a_paper:687", g_an_app_not_a_paper),
    ("bot.knowledge.technology:_cable_fault_not_the_care_question:712", g_cable_fault_not_the_care_question),
    ("bot.knowledge.technology:_junk_or_a_full_box:427", g_junk_or_a_full_box),
    ("bot.knowledge.technology:_not_a_charging_ask:69", g_not_a_charging_ask),
    ("bot.knowledge.technology:_the_bin_is_the_one_on_the_computer:1220", g_the_bin_is_the_one_on_the_computer),
    ("bot.knowledge.technology:_the_block_is_the_phones:734", g_the_block_is_the_phones),
    ("bot.knowledge.technology:_the_glass_in_a_phone:89", g_the_glass_in_a_phone),
    ("bot.knowledge.technology:_the_watch_is_marked:8265", g_the_watch_is_marked),
    ("bot.knowledge.technology:_wifi_is_what_it_is_about:511", g_wifi_is_what_it_is_about),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.technology:_admin_locked_reply:563", r_admin_locked_reply),
    ("bot.knowledge.technology:_battery_health:187", r_battery_health),
    ("bot.knowledge.technology:_card_on_a_phone:753", r_card_on_a_phone),
    ("bot.knowledge.technology:_cold_calls:900", r_cold_calls),
    ("bot.knowledge.technology:_dark_mode_reply:761", r_dark_mode_reply),
    ("bot.knowledge.technology:_default_browser:215", r_default_browser),
    ("bot.knowledge.technology:_empty_trash:192", r_empty_trash),
    ("bot.knowledge.technology:_first_phone:826", r_first_phone),
    ("bot.knowledge.technology:_force_quit:204", r_force_quit),
    ("bot.knowledge.technology:_format_a_drive:1199", r_format_a_drive),
    ("bot.knowledge.technology:_free_app_reply:1163", r_free_app_reply),
    ("bot.knowledge.technology:_laptop_water:1002", r_laptop_water),
    ("bot.knowledge.technology:_meter_reply:435", r_meter_reply),
    ("bot.knowledge.technology:_recover_deleted:1286", r_recover_deleted),
    ("bot.knowledge.technology:_router_factory_reply:555", r_router_factory_reply),
    ("bot.knowledge.technology:_router_place:486", r_router_place),
    ("bot.knowledge.technology:_service_delete_reply:324", r_service_delete_reply),
    ("bot.knowledge.technology:_sim_reply:408", r_sim_reply),
    ("bot.knowledge.technology:_social_delete_reply:336", r_social_delete_reply),
    ("bot.knowledge.technology:_symbol_on_the_keyboard:838", r_symbol_on_the_keyboard),
    ("bot.knowledge.technology:_tech_word:8192", r_tech_word),
    ("bot.knowledge.technology:_the_phone_screen:971", r_the_phone_screen),
    ("bot.knowledge.technology:_to_the_tv:852", r_to_the_tv),
    ("bot.knowledge.technology:_uninstall_app:253", r_uninstall_app),
    ("bot.knowledge.technology:_uninstall_program:234", r_uninstall_program),
    ("bot.knowledge.technology:_vacuum_filter_reply:1086", r_vacuum_filter_reply),
    ("bot.knowledge.technology:_wifi_drops:165", r_wifi_drops),
    ("bot.knowledge.technology:_wifi_password_reply:521", r_wifi_password_reply),
];

pub const SHAPES: &[(&str, ShapeFn)] =
    &[("bot.knowledge.technology:_running_out_of_charge:1023", shape_running_out_of_charge)];
