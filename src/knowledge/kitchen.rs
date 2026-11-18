//! Port of `bot/knowledge/kitchen.py`: three kitchen faults, each with its own cause - a cloudy
//! jam, a pickle with mould, a film or a fizz, and pulses that will not soften.

use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;

/// Three kitchen faults, each with its own cause; None for a line about none of them.
pub fn _kitchen_fault(_memory: &Memory, understood: &Understood) -> Option<String> {
    let words: Vec<String> = understood.words.iter().map(|w| text::lower(w)).collect();
    let said = words.join(" ");
    let has = |wanted: &[&str]| words.iter().any(|word| wanted.iter().any(|want| word == want || word.starts_with(want)));
    let says = |phrases: &[&str]| phrases.iter().any(|p| said.contains(p));
    if has(&["jam", "marmalade", "chutney"]) {
        if (has(&["cloudy"]) || says(&["is cloudy", "looks cloudy", "went cloudy"])) {
            return Some("A cloudy jam or marmalade is scum and trapped air, not a fault: it sets and eats exactly the same. Skim the froth off with a slotted spoon as it boils, add a small knob of butter at the end to settle the bubbles, and let it stand five minutes off the heat before potting so the peel does not float. A marmalade made with the whole fruit is cloudy by nature, and only a clear jelly asks for strained juice.".to_string());
        }
    }
    if (has(&["pickle", "pickles", "chutney"]) || says(&["white stuff", "white bits", "white film"])) {
        if (has(&["fizzy", "bubbling", "bubbles"]) || says(&["is fizzy", "went fizzy", "bubbling"])) {
            return Some("Bubbles and a little fizz in a brine pickle are the fermentation working, and the pickle is fine: lactic acid bacteria give off carbon dioxide as they turn the sugar to acid, and the jar settles down after a week or two. Keep the vegetables under the brine and taste a piece - a sharp, clean sourness is right. If the brine has gone thick and slimy, or it smells of rot rather than vinegar, throw that jar away.".to_string());
        }
        if (has(&["white stuff", "white bits", "white specks", "bloom"]) || says(&["white stuff", "white bits", "white specks", "white film", "white bloom", "cloudy brine"])) {
            return Some("A flat white film or speckle on top of a pickle is usually kahm yeast, which is harmless: skim it off, keep the vegetables under the brine and the pickle is fine. Fuzzy or hairy mould - blue, green, black - is a different thing: throw that jar away, because the mould's threads reach down through the brine and spooning it off the top does not make it safe. A weight that keeps everything under the liquid stops both.".to_string());
        }
        if (has(&["mouldy", "mould", "moldy", "mold", "furry", "fuzz", "green", "blue", "black", "hairy"]) || says(&["mould on", "mouldy", "gone mouldy", "went mouldy"])) {
            return Some("Throw that jar away: mould on a pickle is not something you can rescue, because the threads reach down through the brine and lifting the patch off the top leaves them behind. The one exception is a flat white film with no fuzz - that is usually kahm yeast, and skimming it off is enough, provided the pickle still smells sour and tastes clean. Keeping the vegetables under the brine with a weight is what stops mould taking hold.".to_string());
        }
    }
    if (has(&["lentils", "lentil", "chickpeas", "chickpea", "beans", "haricot", "butter beans", "split peas"]) || says(&["split peas", "haricot beans", "butter beans", "cannellini"])) {
        if (has(&["still hard", "went hard", "stay hard", "stayed hard", "wont soften", "won't soften", "didnt soften", "didn't soften", "not softening", "rock hard", "too hard"]) || says(&["still hard", "wont soften", "won't soften", "stay hard"])) {
            return Some("Old pulses are the usual reason: lentils and beans lose the ability to soften as they age, so cook them in plenty of unsalted water, add the salt at the end, and give chickpeas and dried beans an overnight soak and 60 to 90 minutes at a bare simmer.".to_string());
        }
    }
    return None;

}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.kitchen:_kitchen_fault:35", |_, m, u| _kitchen_fault(m, u))];
