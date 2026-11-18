//! Port of `bot/knowledge/bake_faults.py`: baking faults that came back as a shrug - each with
//! its own cause and its own fix - and the cause of "everything i bake comes out ...".

use crate::data;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

/// `_BAKE_FAULT_FAMILIES`: (words, cause, fix, use).
static _BAKE_FAULT_FAMILIES: LazyLock<Vec<(Vec<String>, String, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.bake_faults._BAKE_FAULT_FAMILIES")
        .items()
        .iter()
        .map(|r| (r.items()[0].strs(), r.items()[1].str().to_string(), r.items()[2].str().to_string()))
        .collect()
});

/// One word matches another loosely: eggs/egg, potatoes/potato, minced/mince.
pub fn _same(word: &str, wanted: &str) -> bool {
    !word.is_empty() && !wanted.is_empty() && (word == wanted || word.starts_with(wanted))
}

/// Eight baking faults that came back as a shrug, each with its own cause and its own fix; ""
/// for a line that names none of them.
pub fn _bake_fault(_memory: &Memory, understood: &Understood) -> String {
    let words: Vec<String> = understood.words.iter().map(|w| text::lower(w)).collect();
    let said = words.join(" ");
    let has = |wanted: &[&str]| words.iter().any(|word| wanted.iter().any(|want| _same(word, want)));
    let says = |phrases: &[&str]| phrases.iter().any(|p| said.contains(p));
    if has(&["cheesecake", "cheesecakes"]) {
        if has(&["grainy", "gritty", "curdled"]) {
            return "A grainy cheesecake has curdled: the oven was too hot, or it was beaten too long after the eggs went in. Bake it at 150C (130C fan) in a water bath, mix only until each thing is just in, and let it cool in the oven with the door ajar so it sets slowly. It is still good to eat - a grainy one is a texture, not a fault to throw away.".to_string();
        }
        if has(&["watery", "soggy", "set"]) {
            return "A watery cheesecake was underbaked or it went into the fridge too soon: it needs to set in the oven until only the middle wobbles, then cool in the tin an hour before it sees the cold. Bake it in a water bath at 150C (130C fan), because the steam is what makes it dense and creamy rather than grainy, and drain the tin well before it is chilled.".to_string();
        }
        return "A cheesecake cracks when it is baked too hot or cooled too fast: bake it at 150C (130C fan) in a water bath, and when only the middle still wobbles turn the oven off and leave it in there with the door ajar for an hour. A crack is cosmetic - a cold cheesecake and a spoon of sour cream over the top covers it - and the next one will not crack if it cools slowly.".to_string();
    }
    if has(&["ganache"]) {
        return "A split ganache got too hot, or cold cream went into warm chocolate: warm it gently over barely simmering water, off the heat, and stir in a tablespoon of warm milk or cream until it comes back together. Next time pour hot cream over chopped chocolate, wait a minute, then stir from the middle outwards, and keep every spoon and bowl dry.".to_string();
    }
    if has(&["meringue", "meringues"]) {
        let weeping = "A meringue weeps when the sugar was not fully dissolved or it was underbaked: rub a little between your fingers and it should feel smooth, not gritty, before it goes in. Bake it low and slow, 100C (80C fan) for an hour and a half, then leave it in the oven with the door ajar to cool, and make it on a dry day.".to_string();
        if has(&["weeping", "weeped", "weeps", "wept", "beading", "beaded", "beads", "gritty", "grainy"]) {
            return weeping.clone();
        }
        if has(&["soft", "softened", "soggy", "sticky", "chewy", "floppy", "limp", "collapsed"]) {
            return "A meringue goes soft when it has not dried out enough, or it was stored where the air is damp: bake it at 100C (80C fan) for an hour and a half and lift one off the paper to check it comes away clean, then cool it in the oven with the door ajar and keep it in an airtight tin - never the fridge, which is what makes it collapse. A sticky one wants 15 minutes more at the same heat, and a chewy one wants the sugar whisked in more slowly and a longer, slower bake.".to_string();
        }
        return weeping.clone();
    }
    if has(&["macaron", "macarons"]) {
        if has(&["hollow"]) {
            return "A hollow macaron was underbaked, or the whites were whipped too stiff and the shell set before the inside could come up: whip to soft to medium peaks, fold the batter until it ribbons and settles back in about 20 seconds, and bake at 150C (130C fan) for a minute or two longer, resting the shells only until a skin forms - much longer than that and they go hollow instead of getting feet.".to_string();
        }
        if has(&["soft", "softened", "softening", "soggy", "sticky", "chewy", "limp", "floppy", "collapsed", "sank"]) {
            return "A macaron that goes soft a day after it is filled is the filling's moisture working into the shell, which is what a macaron is meant to do: keep them in an airtight box and eat them within two or three days, and take them out of the fridge 20 minutes before you do, because cold makes a shell chewy. One that comes out of the oven soft was underbaked or the shells were not left long enough to form a skin: rest them 30 to 60 minutes until they are dry to the touch, then bake at 150C (130C fan) for 2 minutes longer than you did and check one at the back of the tray.".to_string();
        }
        return "Macarons crack when the oven is too hot or the piped shells were not rested: tap the tray, leave them 30 to 60 minutes until a skin forms, then bake at 150C (130C fan) with the door propped a crack. Fold the batter until it ribbons slowly off the spatula - overmixed and they spread, undermixed and they crack - and a hollow one usually wants 5C less and two minutes more.".to_string();
    }
    if has(&["choux", "profiteroles", "profiterole", "eclairs", "eclair"]) {
        return "Choux does not puff when the dough is too wet or the oven door is opened: dry the pan over the heat for a minute once the flour is in, beat in the eggs until it just falls off the spoon in a V, and keep the door shut for the first 20 minutes at 200C (180C fan). A wet dough steams instead of setting, and a cool oven lets it slide.".to_string();
    }
    if has(&["brownie", "brownies"]) {
        return "Brownies that will not set are underbaked: 20 to 25 minutes at 180C (160C fan), until a skewer comes out with a few moist crumbs and no wet batter, then cool them completely in the tin - an hour in the fridge if you are in a hurry - because they firm up as they cool. A runny middle is warm butter and sugar, so give them 5 more minutes rather than more flour.".to_string();
    }
    if (has(&["pastry", "tart", "tarts", "pie"]) || says(&["pie crust", "pie shell", "pastry case"])) {
        if has(&["soft", "softened", "soggy", "sticky", "greasy", "wet", "limp"]) {
            return "Soft pastry is too much fat or water, or the fat got warm: use about half the weight of butter to flour, cut it in cold until it looks like breadcrumbs, and add iced water a spoon at a time until it only just comes together. Handle it as little as you can, rest it 20 minutes in the fridge, and roll it on a floured surface rather than pressing it - warm butter is what makes it limp, so cool the bowl and your hands if the kitchen is hot.".to_string();
        }
        if (has(&["crack", "cracked", "cracks"]) && !has(&["shrank", "shrunk", "shrinking"])) {
            return "".to_string();
        }
        return "Pastry shrinks when the gluten is worked too much or it is stretched into the tin: rest it 20 minutes in the fridge before rolling, ease it in rather than pulling it, and let it hang over the edge instead of pressing it up the sides. Chill the lined tin 15 minutes before it bakes and it will hold its shape.".to_string();
    }
    if (has(&["cake mixture", "cake batter", "sponge mixture"]) || says(&["cake mixture", "cake batter"])) {
        return "A curdled cake mixture is cold eggs going into warm butter too fast, and it comes back easily: beat in a tablespoon of the weighed flour, and stand the bowl in warm water for a minute. Next time use eggs at room temperature and add them a little at a time, beating well after each one, and it will not split.".to_string();
    }
    return "".to_string();
}

/// The cause and the next-time fix for the fault in "everything i bake comes out ...".
pub fn _bake_always(_memory: &Memory, understood: &Understood) -> String {
    let said = understood.words.iter().map(|w| text::lower(w)).collect::<Vec<_>>().join(" ");
    let line = text::lower(if understood.text.is_empty() { &understood.clean } else { &understood.text });
    let both = format!("{said} {line}");
    for (words, cause, fix) in _BAKE_FAULT_FAMILIES.iter() {
        if words.iter().any(|w| both.contains(w.as_str())) {
            return format!("{cause}. Next time: {fix}.");
        }
    }
    String::new()
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.bake_faults:_bake_fault:39", |_, m, u| Some(_bake_fault(m, u))),
    ("bot.knowledge.bake_faults:_bake_always:213", |_, m, u| Some(_bake_always(m, u))),
];
