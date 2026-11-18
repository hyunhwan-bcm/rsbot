//! Port of PyBot's `bot/knowledge/freshness.py`: has that food been out too long to eat, how to
//! cool a dish before the fridge, and food that has gone a bit past its best.
use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g6 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.knowledge.freshness.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
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

strs!(
    LEFT_OUT_KEEP,
    LEFT_OUT_SOFT_CHEESE,
    LEFT_OUT_DAIRY,
    LEFT_OUT_BOTTLE,
    LEFT_OUT_RICE,
    LEFT_OUT_MEAT,
    LEFT_OUT_EGG,
    EGG_COOKED,
    STILL_OK_ASK,
    LEFT_OUT_PHRASE,
    OUT_OF_THE_FRIDGE,
    LEFT_OUT_FOODS,
    COOKING_RESULT,
    BURNT_WORK,
    BURNT_VESSEL,
    SAYS_BIN,
    HOW_IT_IS_KEEPING,
    _GADGETS,
    PAST_IT_PRODUCE,
    PAST_IT_BIN,
);
regexes!(
    _FREEZER_WORD,
    _OUT_OF_THE_FREEZER,
    _LEAVING_OUT,
    _SPAN,
    _CHICKEN_OUT,
    _LEFTOVER_WORD,
    _LEFT_OUT_WORD,
    _OUT_OF_PLACE_WORD,
    _SAID_OUT,
    _TAKING_OUT,
    _SPENT_ON_THE_SIDE,
    _A_BIRD,
    WHAT_TO_DO_WITH_IT,
    _A_FAULT_IN_THE_FOOD,
    _STATE_WORD,
    _NOT_WANTING_TO_EAT,
    _HOW_TO_MAKE_IT_SO,
    _A_PERIOD_OF_TIME,
    _COLD_MACHINE,
    _IN_THE_DISH,
    _DRY_LOAF,
    _BREAD_SPOILT,
    _HOW_TO_KEEP_IT_FRESH,
);

static _A_COOLING_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| g("_A_COOLING_SUBJECT").str_set());
static PAST_IT_KINDS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    g("PAST_IT_KINDS").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
});
static PAST_IT_GENERAL: LazyLock<String> = LazyLock::new(|| g("PAST_IT_GENERAL").str().to_string());

/// The line's clean (or text) lower-cased: `str(u.get("clean") or u.get("text") or "").lower()`.
fn said_lower(understood: &Understood) -> String {
    sh::clean_or_text(understood).to_lowercase()
}

/// True unless the line is a soak that also does not say the food was left out.
pub fn soak_but_not_left_out(understood: &Understood) -> bool {
    static SOAK: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bsoak", 0));
    let said = said_lower(understood);
    if !SOAK.is_match(&said) {
        return true;
    }
    sh::any_in(&said, &OUT_OF_THE_FRIDGE)
}

/// True unless the line is a soak with nothing out, or the food is still in the freezer.
pub fn soak_but_not_left_out_or_in_the_freezer(understood: &Understood) -> bool {
    if !soak_but_not_left_out(understood) {
        return false;
    }
    let said = said_lower(understood);
    if _FREEZER_WORD.is_match(&said) && !_OUT_OF_THE_FREEZER.is_match(&said) {
        return false;
    }
    true
}

/// True when the line asks whether the thing in hand can still be used.
pub fn _asks_if_it_is_still_all_right(text: &str) -> bool {
    sh::any_in(text, &STILL_OK_ASK)
}

/// The line, with the leaving line before it in front when the line asks about that food; the
/// flag says whether the earlier line was borrowed.
pub fn _the_line_it_leans_on(memory: &Memory, reading: &Understood, text: &str) -> (String, bool) {
    if _leaving_out(text) {
        return (text.to_string(), false); // this line says the leaving itself
    }
    if !_asks_if_it_is_still_all_right(text) {
        return (text.to_string(), false);
    }
    let mut earlier = reading.leans_on.as_ref().map(|l| l.text.to_lowercase()).unwrap_or_default();
    if earlier.is_empty() {
        earlier = sh::last_user(memory).to_lowercase();
    }
    let earlier = format!(" {earlier} ");
    if earlier.trim().is_empty() {
        return (text.to_string(), false);
    }
    let before = _names_any(&earlier, &LEFT_OUT_FOODS);
    if !(!before.is_empty() && _leaving_out(&earlier)) {
        return (text.to_string(), false);
    }
    let named = _names_any(text, &LEFT_OUT_FOODS);
    if !named.is_empty() && !named.iter().any(|w| before.contains(w)) {
        return (text.to_string(), false); // a different food
    }
    (earlier + text, true)
}

/// True when the line says a food was left out on the side, and so is a food-safety line.
pub fn says_a_food_was_left_out(text: &str) -> bool {
    _leaving_out(&format!(" {} ", text.to_lowercase()))
}

/// Does the line say a food was left out, rather than that a dish came out a certain way?
pub fn _leaving_out(text: &str) -> bool {
    if sh::any_in(text, &LEFT_OUT_PHRASE) {
        return true;
    }
    _LEAVING_OUT.is_match(text)
}

/// The hours the line says the food was out, or None if it does not say.
pub fn _hours_out(text: &str) -> Option<f64> {
    static NIGHT: LazyLock<Re> =
        LazyLock::new(|| Re::new(r"\b(overnight|all night|the night|night|next morning|in the morning)\b", 0));
    if let Some(found) = _SPAN.search(text) {
        let word = found.str(1);
        let count = match g("_COUNT_WORDS").get(word).and_then(Value::as_f64) {
            Some(c) => c,
            None => match word.parse::<f64>() {
                Ok(c) => c,
                Err(_) => return None,
            },
        };
        return Some(if found.str(2).starts_with("min") { count / 60.0 } else { count });
    }
    if NIGHT.is_match(text) {
        return Some(8.0);
    }
    None
}

/// The words of `words` that the text names as whole words, in the list's order.
pub fn _names_any<S: AsRef<str>>(text: &str, words: &[S]) -> Vec<String> {
    words
        .iter()
        .map(AsRef::as_ref)
        .filter(|w| sh::re_search(&format!(r"\b{}\b", sh::escape(w)), text, 0))
        .map(str::to_string)
        .collect()
}

/// The verb forms that agree with a food: eggs are plural and a pizza is not.
pub struct Agree {
    pub be: &'static str,
    pub were: &'static str,
    pub them: &'static str,
    pub they: &'static str,
    pub look: &'static str,
    pub smell: &'static str,
}

/// Eggs are plural and a pizza is not, and the reply says which.
pub fn _agree(thing: &str) -> Agree {
    let many = thing.ends_with('s') && !thing.ends_with("ss");
    Agree {
        be: if many { "are" } else { "is" },
        were: if many { "were" } else { "was" },
        them: if many { "them" } else { "it" },
        they: if many { "they" } else { "it" },
        look: if many { "look" } else { "looks" },
        smell: if many { "smell" } else { "smells" },
    }
}

/// "How long can it sit out": the rule for the food, told as a limit rather than a bin notice.
pub fn _how_long(_text: &str, soft: &[String], kept: &[String], meat: &[String], rice: &[String], dairy: &[String]) -> String {
    if !soft.is_empty() {
        return "Two hours on the side is the line for soft cheese: past that, bin it, because brie, camembert, \
                mozzarella, feta and cream cheese can carry listeria, which grows in the fridge and changes \
                neither the taste nor the smell. Hard cheese is the exception and can stay out a lot longer."
            .to_string();
    }
    if !rice.is_empty() {
        return "Two hours is the line for cooked rice, and it is the strictest food there is: cool it fast, get \
                it into the fridge within the hour and eat it within a day, because the toxin it can grow \
                survives reheating."
            .to_string();
    }
    if !meat.is_empty() {
        return "Two hours is the line for cooked meat, fish, gravy, pasta and anything with egg or mayonnaise in \
                it - past that it is a bin job even if it looks and smells fine. Rice is the same. Bread, cake, \
                butter, hard cheese, jam and fruit are the exceptions and come to no harm at all."
            .to_string();
    }
    if !dairy.is_empty() {
        return "Two hours for milk, cream and yoghurt, and a whole night out is the bin job: they sour and can \
                grow bacteria you cannot see or smell. Go by the smell if it is borderline."
            .to_string();
    }
    if let Some(word) = kept.first() {
        let verb = _agree(word);
        return format!(
            "{} {} one of the ones with no time limit: there was never a food-poisoning risk at room \
             temperature, so a night out only makes {} stale, dry or soft. The two hours and the bin apply to \
             cooked meat, rice, milk and anything with mayonnaise.",
            text::capitalize(word),
            verb.be,
            verb.them
        );
    }
    "Two hours out of the fridge is the line for anything cooked, and after a whole night it is a bin job - \
     meat, fish, rice, milk and anything with egg or mayonnaise especially. Bread, cake, butter, hard cheese, \
     jam and fruit are the exceptions and keep for a long time on the side."
        .to_string()
}

/// What happens to a food that spent the night on the side, by the food; a follow-up asking
/// whether it can still be used gets the verdict said about time.
pub fn _left_out(memory: &Memory, reading: &Understood) -> Option<String> {
    let text = format!(" {} ", reading.clean.to_lowercase());
    let (text, borrowed) = _the_line_it_leans_on(memory, reading, &text);
    let reply = _left_out_card(&text);
    match reply {
        Some(r) if borrowed && !r.is_empty() => Some(_the_follow_up(&text, &r)),
        other => other,
    }
}

/// The left-out card for a line that names the leaving and the food (None: not this row's line).
pub fn _left_out_card(text: &str) -> Option<String> {
    if _CHICKEN_OUT.is_match(text) && !_LEFT_OUT_WORD.is_match(text) {
        return None;
    }
    if _TAKING_OUT.is_match(text) && _A_BIRD.is_match(text) && !_SPENT_ON_THE_SIDE.is_match(text) {
        return None;
    }
    if !_leaving_out(text) {
        return None;
    }
    if !_names_any(text, &COOKING_RESULT).is_empty() && _hours_out(text).is_none() && !_SAID_OUT.is_match(text) {
        return None;
    }
    if _LEFTOVER_WORD.is_match(text) && !_OUT_OF_PLACE_WORD.is_match(text) {
        return None;
    }
    let hours = _hours_out(text);
    let short = hours.is_some_and(|h| h <= 2.0);
    let soft = _names_any(text, &LEFT_OUT_SOFT_CHEESE);
    let kept = _names_any(text, &LEFT_OUT_KEEP);
    let mut meat = _names_any(text, &LEFT_OUT_MEAT);
    let rice = _names_any(text, &LEFT_OUT_RICE);
    let dairy = _names_any(text, &LEFT_OUT_DAIRY);
    let egg = _names_any(text, &LEFT_OUT_EGG);
    let cooked_egg = !egg.is_empty() && !_names_any(text, &EGG_COOKED).is_empty();

    // A pan or a tin with something burnt on it is a cleaning job, not a night on the side.
    if !_names_any(text, &BURNT_WORK).is_empty() && !_names_any(text, &BURNT_VESSEL).is_empty() {
        return None;
    }

    // "How long can cooked chicken sit out" is a question about the rule.
    let sitting = [" sit", "sits", "sitting", " leave", "leaves", "out", "keep", "keeps"].iter().any(|w| text.contains(w));
    if text.contains("how long") && sitting && !cooked_egg {
        return Some(_how_long(text, &soft, &kept, &meat, &rice, &dairy));
    }

    if cooked_egg && meat.is_empty() {
        meat = vec!["egg".to_string()];
    }

    // Whole eggs keep for weeks on a shelf.
    if !egg.is_empty() && !cooked_egg && !short {
        return Some(
            "Whole eggs are fine: they are sold and kept on a shelf at room temperature and keep for weeks, so \
             a night on the side changes nothing. It is a cracked egg, or a cooked one - scrambled, boiled, \
             fried, an omelette - that follows the two-hour rule and goes in the bin after a night out."
                .to_string(),
        );
    }

    if !soft.is_empty() {
        if short {
            return Some(
                "Inside the two hours, so it is fine even for this one: get it back in the fridge now and eat \
                 it within a couple of days. Soft cheese is the risky end of cheese, so it does not get the \
                 long stay a hard cheddar does."
                    .to_string(),
            );
        }
        return Some(
            "Bin it, I'm afraid. Soft cheese is the risky end of cheese: brie, camembert, mozzarella, feta, \
             cream cheese and blue cheese can carry listeria, which grows in the fridge and does not change the \
             taste or the smell. Anything more than two hours out is a bin job, and a hard cheese is the only \
             one you can leave."
                .to_string(),
        );
    }
    if !kept.is_empty() && !(!meat.is_empty() || !rice.is_empty() || !_names_any(text, &LEFT_OUT_DAIRY).is_empty()) {
        let word = &kept[0];
        let verb = _agree(word);
        return Some(format!(
            "{} {} one of the safe ones: there was never a food-poisoning risk at room temperature, so a night \
             on the side only makes {} stale, dry or soft. Put {} back and carry on - it is cooked meat, rice, \
             milk and anything with mayonnaise that you cannot leave out.",
            text::capitalize(word),
            verb.be,
            verb.them,
            verb.them
        ));
    }
    let bottle = _names_any(text, &LEFT_OUT_BOTTLE);
    if !bottle.is_empty() {
        if hours.is_some_and(|h| h >= 3.0) {
            return Some(
                "Bin it. A made-up bottle is two hours from the moment it is made, and an hour once the baby \
                 has started it, whatever the room is like: bacteria from the baby's mouth get into the milk \
                 and warm milk grows them fast. It is not one of the things you can judge by smell."
                    .to_string(),
            );
        }
        return Some(
            "Made-up formula or expressed milk is good for two hours at room temperature and a bottle the baby \
             has started for one hour, so this one is inside the window: use it now rather than later, and \
             past two hours it is a bin job. It is not dairy you can test by smell - two hours is the number, \
             not the sniff."
                .to_string(),
        );
    }
    if !_names_any(text, &LEFT_OUT_DAIRY).is_empty() {
        if short {
            return Some(
                "A couple of hours on the table is fine for milk, cream and yoghurt: get it back in the fridge \
                 now and it will be good to the date on the pack. A whole night out is the bin job, because \
                 they sour and can grow bacteria you cannot smell."
                    .to_string(),
            );
        }
        return Some(
            "Milk, cream and yoghurt are fine on the table for a couple of hours at a meal, and a whole night \
             out is a bin job: they sour and can grow bacteria you cannot see or smell. Keep it at the back of \
             the fridge, not in the door, and go by the smell if you are not sure."
                .to_string(),
        );
    }
    if !rice.is_empty() {
        if short {
            return Some(
                "Inside the two hours, so it is fine: get it into the fridge now in a shallow tub so it cools \
                 fast, and eat it within a day. Rice is the one to be careful with, because the toxin it can \
                 grow survives reheating."
                    .to_string(),
            );
        }
        return Some(
            "Bin it after 2 hours out, and this one is the strictest of the lot: cooked rice can grow a toxin \
             that reheating does not destroy, and it looks and smells fine while it does it. Next time cool it \
             fast, fridge it within the hour, and eat it within a day."
                .to_string(),
        );
    }
    if let Some(word) = meat.first() {
        let verb = _agree(word);
        if short {
            return Some(format!(
                "{} inside the two-hour limit {} fine: get {} into the fridge now, where cooked food keeps 3 \
                 days. It is only past two hours that this becomes a bin job.",
                text::capitalize(word),
                verb.be,
                verb.them
            ));
        }
        return Some(format!(
            "Bin it, I'm afraid. {} left out for more than 2 hours {} not safe to eat, even if {} {} and {} \
             fine. Meat, fish, gravy and anything with egg or mayonnaise in it all follow that rule, so next \
             time get it into the fridge within the hour.",
            text::capitalize(word),
            verb.be,
            verb.they,
            verb.look,
            verb.smell
        ));
    }
    if short {
        return Some(
            "Inside the two hours, so it is fine: get it covered into the fridge now and eat it within a day or \
             two. Two hours out is the limit, and it is a night out that makes it a bin job."
                .to_string(),
        );
    }
    Some(
        "Two hours out of the fridge is the line, and after a whole night it is a bin job for anything cooked, \
         especially meat, fish, rice, milk or a dish with mayonnaise or egg in it. Bread, cake, butter, hard \
         cheese, jam and fruit are the exceptions and come to no harm."
            .to_string(),
    )
}

/// The verdict for "can i still cook it" after a line that had food left out on the side, or
/// for "what can i do with it" after a bin verdict; None when the two lines do not make a pair.
pub fn the_still_ok_follow_up(text: &str, memory: &Memory) -> Option<String> {
    let here = format!(" {} ", text.to_lowercase());
    if _leaving_out(&here) || _asks_if_it_is_still_all_right(&here) {
        if let Some(verdict) = _the_left_out_follow_up(&here, memory).filter(|v| !v.is_empty()) {
            return Some(verdict);
        }
    }
    _the_food_it_just_said_to_bin(&here, memory)
}

/// The verdict for a line asking whether a food that sat out can still be cooked.
pub fn _the_left_out_follow_up(here: &str, memory: &Memory) -> Option<String> {
    let earlier = format!(" {} ", sh::last_user(memory).to_lowercase());
    let before = _names_any(&earlier, &LEFT_OUT_FOODS);
    if !(!before.is_empty() && _leaving_out(&earlier)) {
        return None;
    }
    let named = _names_any(here, &LEFT_OUT_FOODS);
    if !named.is_empty() && !named.iter().any(|w| before.contains(w)) {
        return None; // a different food
    }
    let both = format!("{earlier}{here}");
    let card = _left_out_card(&both).filter(|c| !c.is_empty())?;
    Some(_the_follow_up(&both, &card))
}

/// What to do with a food the answer before this line told the person to bin.
pub fn _the_food_it_just_said_to_bin(here: &str, memory: &Memory) -> Option<String> {
    if !WHAT_TO_DO_WITH_IT.is_match(here) {
        return None;
    }
    let said = format!(" {} ", dup::last_bot(memory).to_lowercase());
    let earlier = format!(" {} ", sh::last_user(memory).to_lowercase());
    let first = said.split(". ").next().unwrap_or("");
    if !SAYS_BIN.iter().any(|w| first.contains(w.as_str())) {
        return None;
    }
    if !(_asks_if_it_is_still_all_right(&earlier) || _leaving_out(&earlier) || sh::any_in(&earlier, &HOW_IT_IS_KEEPING)) {
        return None;
    }
    let food = _names_any(&earlier, &LEFT_OUT_FOODS);
    let thing = food.first()?;
    let named = _names_any(here, &LEFT_OUT_FOODS);
    if !named.is_empty() && !named.iter().any(|w| food.contains(w)) {
        return None; // it names a different food
    }
    let a = _agree(thing);
    Some(format!(
        "Nothing to cook, I'm afraid: {thing} that {look} or {smell} like that {be} a bin job, and no recipe \
         makes {them} safe - heat kills the bacteria growing on {them} but not the toxins they have already \
         left behind. If the only thing wrong with {them} is the date on the packet, use {them} tonight rather \
         than keep {them} for another day.",
        look = a.look,
        smell = a.smell,
        be = a.be,
        them = a.them
    ))
}

/// The verdict as the answer to "can I still cook it", instead of the card over again.
pub fn _the_follow_up(text: &str, reply: &str) -> String {
    let first = reply.split(". ").next().unwrap_or("").to_lowercase();
    if !first.contains("bin") || !_names_any(text, &LEFT_OUT_BOTTLE).is_empty() {
        return reply.to_string();
    }
    if _hours_out(text).is_some_and(|h| h <= 2.0) {
        return "Inside the two hours, so yes: use it now rather than later, and do not put it back in the \
                fridge to keep. Two hours out is the line - past that it is a bin job even when it looks and \
                smells fine."
            .to_string();
    }
    "Bin it, I'm afraid: past two hours out it is not safe, and cooking does not undo that - heat kills the \
     bacteria that grew while it sat out, but not the toxins some of them have already left behind, so it can \
     still make you ill. Two hours is the line for meat, fish, rice, milk and anything with egg or mayonnaise \
     in it."
        .to_string()
}

/// A food carried in a warm car: the same two hours, less on a hot day.
pub fn _in_the_car(_memory: &Memory, reading: &Understood) -> String {
    static HOT: LazyLock<Re> = LazyLock::new(|| {
        Re::new(r"\b(hot day|heatwave|in the sun|all afternoon|all day|sunny|summer|hot car|warm car)\b", 0)
    });
    static CAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(car|cars|boot|back seat|van|drive)\b", 0));
    let text = format!(" {} ", reading.clean.to_lowercase());
    let mut thing: Option<String> = None;
    for words in [&*LEFT_OUT_BOTTLE, &*LEFT_OUT_DAIRY, &*LEFT_OUT_MEAT, &*LEFT_OUT_KEEP] {
        if let Some(found) = _names_any(&text, words).into_iter().next() {
            thing = Some(found);
            break;
        }
    }
    let hot = HOT.is_match(&text);
    let thing = thing.unwrap_or_else(|| "food".to_string());
    if hot {
        return format!(
            "{} in a hot car is the worst place it can be: a warm car gets well past fridge temperature, so \
             count it as more than two hours out and bin anything cooked, dairy or made with mayonnaise that \
             spent the afternoon there. On a hot day a cool bag, or buying the {} last, saves the whole question.",
            text::capitalize(&thing),
            thing
        );
    }
    let where_ = if CAR.is_match(&text) {
        format!(
            "A car is a warm room with no fridge in it, so the two-hour rule applies to the {thing} as it does \
             anywhere else: fine for the drive home, then straight into the fridge."
        )
    } else {
        format!(
            "Out of the fridge with no cool bag is the same as the table, so the two-hour rule applies to the \
             {thing} as it does anywhere else: fine for the journey, then into the fridge as soon as you are in."
        )
    };
    format!(
        "{where_} Anything cooked, dairy or made with mayonnaise that spent the afternoon warm is a bin job - \
         warmth is what the bacteria that change neither taste nor smell need."
    )
}

/// A marinade is meat in the fridge, not meat left on the side.
pub fn _marinate(_memory: &Memory, reading: &Understood) -> String {
    let text = format!(" {} ", reading.clean.to_lowercase());
    let meat = _names_any(&text, &["chicken", "fish", "salmon", "prawns", "turkey"]);
    if let Some(thing) = meat.first() {
        return format!(
            "Overnight is fine for a marinade, in the fridge: {thing} takes 30 minutes to 2 hours at room \
             temperature and up to a day in the fridge, and the fridge is the safer way to do it. Whisk 3 parts \
             oil to 1 part acid - lemon, wine or vinegar - with salt, then garlic, honey, soy or herbs."
        );
    }
    "Overnight is what a marinade is for, and it belongs in the fridge: tougher cuts take 4 hours or a full \
     night, steak and lamb chops a few hours, and fish no more than an hour because the acid starts to cure \
     it. Whisk 3 parts oil to 1 part acid - lemon, wine or vinegar - with salt, then garlic, honey, soy or \
     herbs. More than a day in acid or sugar turns the meat mushy."
        .to_string()
}

/// True when a gadget is what the conversation is about, not the food in the line.
pub fn _a_gadget_in_the_talk(memory: &Memory, understood: &Understood) -> bool {
    if sh::any_in(&understood.clean.to_lowercase(), &_GADGETS) {
        return true;
    }
    let turns = &memory.turns;
    let recent: Vec<String> = turns[turns.len().saturating_sub(3)..].iter().map(|t| t.user.to_lowercase()).collect();
    recent.iter().any(|line| sh::any_in(line, &_GADGETS))
}

/// How to get something hot into the fridge, said for the thing in the person's hand.
pub fn _cool_before_fridge(memory: &Memory, understood: &Understood) -> Option<String> {
    static WHEN: LazyLock<Re> =
        LazyLock::new(|| Re::new(r"\bwhat time\b|\bwhen should i\b|\bwhen do i\b|\bwhat time do i\b", 0));
    static COLD_END: LazyLock<Re> = LazyLock::new(|| {
        Re::new(r"\b(?:fridge|freez\w*|refrigerat\w*|cool\w*|cold|ice|chill\w*|worktop|defrost\w*)\b", 0)
    });
    let words: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let said = understood.clean.to_lowercase();
    let has = |set: &[&str]| set.iter().any(|w| words.contains(*w));
    if !words.iter().any(|w| _A_COOLING_SUBJECT.contains(w)) {
        return None;
    }
    if _A_FAULT_IN_THE_FOOD.is_match(&said) {
        return None;
    }
    if WHEN.is_match(&said) && !COLD_END.is_match(&said) {
        return None;
    }
    if _a_gadget_in_the_talk(memory, understood) {
        return None;
    }
    let quick = has(&["quickly", "quick", "fast", "faster", "fastest", "quickest", "rapidly", "speed", "sooner", "hurry"]);
    if words.contains("rice") {
        return Some(
            "Cool cooked rice fast, not slowly: spread it in a shallow tray and leave it no more than an hour, \
             then cover it and put it in the fridge and eat it within a day, reheated until it is steaming hot. \
             Rice left at room temperature grows a toxin that reheating does not destroy, which is why it gets \
             an hour where a stew gets two."
                .to_string(),
        );
    }
    if has(&["milk", "bottle", "bottles", "formula", "bottle of milk", "breast milk"]) {
        return Some(
            "Stand the bottle in a jug of cold water for ten minutes, or hold it under the cold tap, then put it \
             straight back in the fridge. Milk that has been warmed keeps about two days, so use it before the \
             rest of the bottle and do not warm it twice - and never keep what a baby has already drunk from."
                .to_string(),
        );
    }
    if has(&[
        "cake", "cakes", "sponge", "brownie", "brownies", "muffin", "muffins", "cupcake", "cupcakes", "loaf", "loaves",
        "traybake", "bread", "buns",
    ]) {
        return Some(
            "Leave it in the tin for ten minutes, then turn it out onto a rack and let it go cold before anything \
             goes on it: a warm cake steams itself soggy and the icing slides off. Do not put it in the fridge to \
             speed it up - a chilled sponge goes heavy and stale faster than one kept in a tin."
                .to_string(),
        );
    }
    if quick {
        return Some(
            "Stand the pot in a sink of cold water, with a cold pack or a tray of ice in it, and stir every few \
             minutes: a big pot comes down to warm in about 20 minutes rather than two hours. Then cover it and \
             put it in the fridge - it does not need to be stone cold, just not steaming. Past two hours out of \
             the fridge, bin it."
                .to_string(),
        );
    }
    if has(&[
        "pot", "pots", "pan", "saucepan", "stew", "soup", "curry", "chilli", "chili", "casserole", "stock", "batch",
        "bolognese", "sauce",
    ]) {
        return Some(
            "Let it cool on the worktop for up to two hours, then cover it and put it in the fridge. A big pot \
             straight off the hob warms the whole fridge, so split it into shallow tubs first - it does not need \
             to be stone cold, just not steaming. Past two hours out, bin it."
                .to_string(),
        );
    }
    Some(
        "Spread it out in a shallow tub so it cools faster, leave it on the worktop for up to two hours, then \
         cover it and put it in the fridge - it does not need to be stone cold, just not steaming. Eat leftovers \
         within two days and reheat them until they are steaming hot all through. Past two hours out of the \
         fridge, bin it."
            .to_string(),
    )
}

/// True when the line says what state the food is in, not only what state it is.
pub fn _says_the_state(understood: &Understood) -> bool {
    let said = said_lower(understood);
    if _NOT_WANTING_TO_EAT.is_match(&said) {
        return false;
    }
    if _HOW_TO_MAKE_IT_SO.is_match(&said) {
        return false;
    }
    _STATE_WORD.is_match(&said)
}

/// The verdict on food that has gone a bit past its best, said about the food in hand.
pub fn _past_its_best(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = format!(" {} ", said_lower(understood));
    if _names_any(&text, &PAST_IT_PRODUCE).is_empty() {
        return None;
    }
    if sh::any_in(&text, &PAST_IT_BIN) {
        return Some(
            "Bin it. The feel and the smell are what the stomach reacts to, not the colour, and cooking does not \
             make it safe again: the bacteria that grew leave toxins behind that heat does not touch. Compost it \
             and wash the drawer it was in."
                .to_string(),
        );
    }
    for (names, said) in PAST_IT_KINDS.iter() {
        if sh::any_in(&text, names) {
            return Some(said.clone());
        }
    }
    Some(PAST_IT_GENERAL.clone())
}

/// False when the line is the fridge or freezer being slow rather than a dish to cool.
pub fn the_cold_machine_and_not_the_dish(understood: &Understood) -> bool {
    let mut said = understood.words.join(" ").to_lowercase();
    if said.is_empty() {
        said = said_lower(understood);
    }
    !(_COLD_MACHINE.is_match(&said) && !_IN_THE_DISH.is_match(&said))
}

/// True unless the line says how long the dish has been in the fridge already.
pub fn a_cooling_question_not_a_keeping_one(understood: &Understood) -> bool {
    let mut said = understood.words.join(" ").to_lowercase();
    if said.is_empty() {
        said = understood.text.to_lowercase();
    }
    !_A_PERIOD_OF_TIME.is_match(&said)
}

/// False for a loaf that is dry or hard and says nothing about being spoilt.
pub fn _not_just_a_hard_loaf(understood: &Understood) -> bool {
    let said = said_lower(understood);
    if !_DRY_LOAF.is_match(&said) {
        return true;
    }
    _BREAD_SPOILT.is_match(&said)
}

/// False when the line asks how to stop a loaf going stale, not what to do with a stale one.
pub fn _not_how_to_keep_it_fresh(understood: &Understood) -> bool {
    !_HOW_TO_KEEP_IT_FRESH.is_match(&said_lower(understood))
}

/// The cool-before-fridge guard: a cooling question, and a dish rather than the machine.
pub fn lambda_1050(understood: &Understood) -> bool {
    a_cooling_question_not_a_keeping_one(understood) && the_cold_machine_and_not_the_dish(understood)
}

/// The food-in-the-car guard: false when a pet, not a food, is the thing in the car.
pub fn lambda_1322(understood: &Understood) -> bool {
    let clean = understood.clean.to_lowercase();
    let words = text::split_ref(&clean);
    !["dog", "dogs", "puppy", "pup", "puppies", "cat", "cats", "kitten", "kittens", "pet", "pets"]
        .iter()
        .any(|w| words.contains(w))
}

// --- registry wrappers ------------------------------------------------------------------------

fn g_1050(_f: &FnRef, u: &Understood) -> bool {
    lambda_1050(u)
}
fn g_1322(_f: &FnRef, u: &Understood) -> bool {
    lambda_1322(u)
}
fn g_not_how_to_keep_it_fresh(_f: &FnRef, u: &Understood) -> bool {
    _not_how_to_keep_it_fresh(u)
}
fn g_not_just_a_hard_loaf(_f: &FnRef, u: &Understood) -> bool {
    _not_just_a_hard_loaf(u)
}
fn g_says_the_state(_f: &FnRef, u: &Understood) -> bool {
    _says_the_state(u)
}
fn g_soak_but_not_left_out(_f: &FnRef, u: &Understood) -> bool {
    soak_but_not_left_out(u)
}
fn g_soak_but_not_left_out_or_in_the_freezer(_f: &FnRef, u: &Understood) -> bool {
    soak_but_not_left_out_or_in_the_freezer(u)
}

fn r_cool_before_fridge(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _cool_before_fridge(m, u)
}
fn r_in_the_car(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_in_the_car(m, u))
}
fn r_left_out(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _left_out(m, u)
}
fn r_marinate(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_marinate(m, u))
}
fn r_past_its_best(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _past_its_best(m, u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.freshness:<lambda>:1050", g_1050),
    ("bot.knowledge.freshness:<lambda>:1322", g_1322),
    ("bot.knowledge.freshness:_not_how_to_keep_it_fresh:1023", g_not_how_to_keep_it_fresh),
    ("bot.knowledge.freshness:_not_just_a_hard_loaf:1015", g_not_just_a_hard_loaf),
    ("bot.knowledge.freshness:_says_the_state:914", g_says_the_state),
    ("bot.knowledge.freshness:soak_but_not_left_out:122", g_soak_but_not_left_out),
    ("bot.knowledge.freshness:soak_but_not_left_out_or_in_the_freezer:139", g_soak_but_not_left_out_or_in_the_freezer),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.freshness:_cool_before_fridge:737", r_cool_before_fridge),
    ("bot.knowledge.freshness:_in_the_car:640", r_in_the_car),
    ("bot.knowledge.freshness:_left_out:371", r_left_out),
    ("bot.knowledge.freshness:_marinate:681", r_marinate),
    ("bot.knowledge.freshness:_past_its_best:924", r_past_its_best),
];
