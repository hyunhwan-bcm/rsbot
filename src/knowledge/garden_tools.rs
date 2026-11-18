//! Port of `bot/knowledge/garden_tools.py`: the garden machines - a hedge trimmer, a strimmer, a
//! leaf blower, a chainsaw, a rotavator: what each is, how it cuts, how it keeps.

use crate::data;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;

/// A phrase list of the module's, by name.
fn consts(name: &str) -> Vec<String> {
    data::get(&format!("bot.knowledge.garden_tools.{name}")).strs()
}

/// True for the safety question: what to wear, and how not to cut yourself.
pub fn _the_guard_question(words: &HashSet<String>, line: &str) -> bool {
    if ["safety", "protective", "goggles", "glasses", "visor", "chaps", "helmet", "defenders", "gloves", "ppe"].iter().any(|w| words.contains(*w)) {
        return true;
    }
    [
        "safety gear", "safety equipment", "protective gear", "protective equipment", "what should i wear", "what do i wear", "cut my fingers",
        "cutting my fingers", "cut my leg", "cutting my leg", "keep catching", "slice my", "kickback", "kick back",
    ]
    .iter()
    .any(|p| line.contains(p))
}

/// A hedge trimmer, a strimmer, a blower or a chainsaw: what it is, how it cuts, how it keeps.
#[allow(unused_mut, unused_assignments)]
pub fn _garden_machine(_memory: &Memory, understood: &Understood) -> String {
    let said: Vec<String> = understood.words.iter().map(|w| text::lower(w)).collect();
    let words: HashSet<String> = said.iter().cloned().collect();
    let line = format!(" {} ", said.join(" "));
    let says = |phrases: &[&str]| phrases.iter().any(|p| line.contains(&format!(" {p} ")));
    let meets = |set: &[&str]| set.iter().any(|w| words.contains(*w));
    macro_rules! says {
        (&consts($n:expr)) => {{
            let list = consts($n);
            list.iter().any(|p| line.contains(&format!(" {p} ")))
        }};
    }
    let mut hedge = (meets(&["hedge", "hedges", "hedge trimmer", "hedge trimmers", "hedge cutters", "trimming", "topiary"]) || says(&["a hedge", "the hedge", "my hedge", "hedges"]));
    let mut strimmer = (meets(&["strimmer", "strimmers", "grass trimmer", "grass trimmers"]) || says(&["strimmer", "grass trimmer", "edge my lawn", "edge the lawn", "edging", "lawn edge", "lawn edges", "edges of my lawn"]));
    if (meets(&["edge", "edges", "edging"]) && meets(&["lawn", "lawns", "grass", "path", "paths", "drive", "driveway", "border", "borders", "kerb", "paving"])) {
        strimmer = true;
    }
    let mut blower = (meets(&["leaf blower", "leafblower", "leaf blowers"]) || says(&["leaf blower", "leafblower", "blow the leaves", "blow leaves", "leaves off"]));
    let mut tiller = (meets(&["rotavator", "rotovator", "rotavating", "rotovating", "tiller", "cultivator"]) || says(&["rotavator", "rotovator", "dig over", "break up the soil"]));
    let mut saw = (meets(&["chainsaw", "chain saw", "chainsaws"]) || says(&["chainsaw", "chain saw", "fell a tree", "cut a log", "cut logs"]));
    if (says!(&consts("_LINE_IT")) && (strimmer || !(hedge || blower || saw))) {
        return "Take the spool off the head, wind the new line on in the direction of the arrow on it, and hook each end through the eyelet on opposite sides so it cuts level. Leave about 10 to 15cm of line out on each side and let the head spin for a moment before you cut, because a line wound the wrong way will not feed. If it keeps snapping, it is usually too long, or the line is old and brittle.".to_string();
    }
    if says!(&consts("_FUEL_IT")) {
        if (strimmer || saw) {
            return "Two-stroke machines are nine times in ten old fuel: the mix goes off in about a month, so tip it out, put fresh petrol with the right two-stroke oil in it (50:1 on most) and try again with the choke on for two pulls only, or the engine floods. Then the spark plug out and dried, the air filter washed, and on a strimmer the line trimmed to the right length so it is not grabbing the guard.".to_string();
        }
        return "Nine times in ten it is the fuel: petrol goes off in about a month, so drain last year's half tank and refill. Then the spark plug out and cleaned or replaced, the air filter shaken out, and the blades checked for a jammed stem or a build-up of cuttings behind the guard, which drags the motor down and makes it stall.".to_string();
    }
    let mut putting_away = says(&["store", "storing", "stored", "storage", "put away", "put it away", "over winter", "for the winter", "winter", "wintering", "winterise", "winterize", "shed", "garage", "rust", "rusty", "away until"]);
    if (says!(&consts("_STORE_IT")) && (putting_away || !says!(&consts("_CARE_IT")))) {
        if blower {
            return "Empty the bag and wipe the inside of the tube, because leaves left in it go soggy and rot the housing. Run the tank dry if it is petrol, or charge the battery to about half if it is not, and hang it in a dry shed rather than standing it on a damp concrete floor.".to_string();
        }
        if saw {
            return "Empty the fuel tank and run it dry, because petrol left in a carburettor over winter turns to varnish and that is what turns a spring start into a repair. Clean the bar groove, oil the chain, slacken the chain tension so it is not stretched over months, and store it with the bar in its cover in a dry shed.".to_string();
        }
        if (strimmer || hedge) {
            return "Empty the fuel tank and run the engine until it stops, because petrol left in a carburettor over winter turns to varnish and that is what makes a spring start-up a repair. Wipe the blades or the cutting head clean, oil the blades with a rag and a few drops of three-in-one, and put the guard back on. Hang it in a dry shed rather than standing it on a concrete floor, and charge a battery tool to about half and keep it indoors, since a battery left flat in a cold shed does not come back.".to_string();
        }
        return "Store garden machines the same way: fuel out and the engine run dry, blades wiped and oiled, guards on, and everything up off a damp concrete floor in a dry shed. Batteries come indoors at about half charge, because the cold is what kills them.".to_string();
    }
    if says!(&consts("_USE_IT")) {
        if tiller {
            return "Go over the ground twice, the first pass shallow and slow with the tines just breaking the surface and the second deeper, and only when the soil is damp but not wet enough to smear. Take it in strips the width of the machine, keep the tines clear of stones and roots, and let the soil settle a week or two before planting or sowing.".to_string();
        }
        if hedge {
            return "Cut the sides first, bottom to top in a sweep with the blades at about 10 to 15 degrees to the hedge, then the top flat - working upwards throws the cuttings clear instead of back in your face. A string line along the top and one down each end is what makes a hedge look straight, and take the top back to the same width as the bottom, so light reaches the base.".to_string();
        }
        if strimmer {
            return "Hold the head just off the ground and sweep it side to side from left to right with the line hitting the grass at full speed, and work away from anything you do not want cut. Edge along a path by walking it with the head turned 90 degrees, and tap the line out on the ground rather than pulling it by hand.".to_string();
        }
        if blower {
            return "Work with the wind and from the edges inwards, holding the tube just off the ground and using short sweeps rather than full power - and never blow cuttings into a drain, a road or a neighbour's garden. Damp leaves want a rake first, because a blower only moves them.".to_string();
        }
        if saw {
            return "Start the cut with the saw running at full speed and let the chain do the work, keeping the bar out of the dirt and your body to one side of the cut. Cut from the top down on a branch, keep the chain oiled and tensioned, and never cut above shoulder height or with the tip, which is what causes kickback.".to_string();
        }
    }
    if (says!(&consts("_WET_IT")) && (meets(&["grass", "lawn", "lawns", "cut", "cutting", "strim", "strimming", "edge", "edging", "mow", "mowing"]) || strimmer)) {
        return "Wet grass clogs the guard and welts the line, so the head drags and the motor bogs down; leave it until it is dry and it cuts in half the time. If you have to, raise the head a notch, take a narrower strip, use the guard and clean the grass out of the spool housing before it sets. A petrol machine also cuts long wet grass badly, so a couple of passes at different heights beat one deep one.".to_string();
    }
    if says!(&consts("_NOT_CUTTING")) {
        if hedge {
            return "A trimmer bar that tears rather than cuts is blunt, or the blades are loose or jammed with sap: clean the bar with white spirit, check the bolts are nipped up, and file each blade at the angle it already has. A hedge also tears if you are cutting woody growth more than a finger thick, which wants loppers or a saw first.".to_string();
        }
        if strimmer {
            return "A line that will not cut is usually too long, too old, or wound the wrong way: trim it to about 12cm out of the head, wind it on in the direction of the arrow, and if it is brittle, replace it. Check too that the line is not welded into a lump in the spool, which happens when the head is held in long grass.".to_string();
        }
        if saw {
            return "A chain that will not cut is blunt or too loose: file every tooth on the angle marked on it, take the depth gauges down a touch behind the teeth, and check the tension cold so it lifts about 3mm off the bar. A dry chain also stretches and burns the bar, so keep the oil tank full.".to_string();
        }
        return "Clean the cutting head, check the blade or the line, and look for a jam: a build-up of cuttings behind the guard drags the motor down, a blunt blade or an old line tears instead of cutting, and a loose blade judders. Nothing sharp on a garden machine is worth using blunt, because a dull edge is what makes people push and slip.".to_string();
    }
    if _the_guard_question(&words, &line) {
        if saw {
            return "Chaps or tough trousers over the legs, a helmet with a visor and ear defenders, gloves and boots - and a first-aid kit within reach, because the cut that goes wrong is the one on your own leg. Kickback comes from the tip of the bar, so keep the tip out of the wood and stand to one side of the cut.".to_string();
        }
        if hedge {
            return "Gloves, eye protection and ear defenders, and the guard back on the blades before you start. Keep the lead behind you and over your shoulder rather than trailing in front, hold the trimmer with both hands and never reach over the hedge to the far side, which is how fingers get between the blades.".to_string();
        }
        if strimmer {
            return "Eye protection, gloves and trousers rather than shorts, because the line throws stones as well as grass. The guard stays on, the head stays below waist height, and nobody else - child or pet - is in the same part of the garden while it spins.".to_string();
        }
        return "Eye protection, gloves, ear defenders and boots, with the guard on and the lead behind you rather than in front, and nobody else in the garden while it is running.".to_string();
    }
    if says!(&consts("_START_IT")) {
        if (saw || strimmer) {
            return "Cold start: choke fully out, two or three pulls only - any more and it floods - then the choke half in and it should catch. If it fires and dies, the mix is stale or the idle is set too low; if it does not fire at all, the plug is wet or the air filter is clogged. A hedgetrimmer with a battery or a lead has none of that: hold the trigger lock, squeeze, and if it is dead check the lead and the breaker.".to_string();
        }
        return "Cold start: choke out, a couple of pulls, then choke half in. If it fires and dies the fuel is old, and if it does not fire the plug or the air filter is the next thing to look at.".to_string();
    }
    if says!(&consts("_CARE_IT")) {
        if hedge {
            return "Take the blades off with the plug lead pulled or the battery out, clamp them in a vice and run a flat file along each blade at the angle it already has, about a dozen strokes, then a drop of oil along the bar and a run for a minute to spread it. Clean the sap off with a rag and white spirit first, because sap is what makes a trimmer bind.".to_string();
        }
        if saw {
            return "File every tooth on the same angle with a round file of the size the chain says, two or three strokes each, and take the depth gauges down a touch behind them so the teeth can bite. Check the tension cold - it should lift about 3mm off the bar - and keep the oil tank full, because a dry chain stretches and then breaks.".to_string();
        }
        return "Clean the cutting head and the air intake of grass and sap, oil the moving parts with a few drops, and check the blade or the line: a strimmer line that is too long drags the motor down and a blunt blade tears rather than cuts. A chain wants its tension checked every use and a file across the teeth every few tanks.".to_string();
    }
    if (says(&["should i buy", "which one", "worth it", "to buy", "buy", "buying", "worth buying", "best one", "best to get", "should i get", "what should i get", "do i need", "need a", "or a", "instead of", "which is better", "recommend", "recommendation", "petrol or", "battery or", "corded or", "mains or"]) || meets(&["which"])) {
        if hedge {
            return "Get a corded trimmer for one hedge under about 10 metres and a battery one if the garden has no outdoor socket or the hedge is longer, because a cable overhead is the thing that makes people put it off. Petrol only for a hedge over about 2 metres or a long run of it.".to_string();
        }
        if strimmer {
            return "A mains strimmer is the cheapest and lightest for a small lawn, battery for anything over about 50 metres of edge or where there is no outdoor socket, and petrol only for rough ground and long sessions. A bump-feed head is worth more than extra power for ordinary garden edging.".to_string();
        }
        if blower {
            return "You need one if leaves sit on gravel, decking or a lawn you want tidy, and you do not if the garden is small enough to rake in ten minutes. A battery blower is quieter and enough for a suburban garden; petrol only for a big garden and a lot of leaves at once.".to_string();
        }
        if saw {
            return "A battery saw is enough for logs and a fallen branch up to about 20cm and is the one most people use; petrol only for a lot of timber and a whole afternoon of cutting. Get one with a chain brake whatever else you choose.".to_string();
        }
        return "Match the tool to the job and the size: mains for a small garden and a short run, battery where there is no outdoor socket or the run is long, and petrol only for a big garden where the noise and the fuel are worth it.".to_string();
    }
    if (says!(&consts("_CHAIN_OFF")) && (saw || meets(&["chain", "chainsaw", "chain saw"]))) {
        return "A chain that keeps coming off is loose tension or a worn drive sprocket: check the two bar nuts are nipped up, set the tension cold so the chain lifts only about 3mm off the bar, then turn the bar over and look at the nose sprocket, because a worn one lets the chain climb out of the groove. If it still jumps the chain is stretched or the sprocket is hooked, and either will wear the bar out if you keep cutting with it.".to_string();
    }
    if says!(&consts("_NOT_FEEDING")) {
        return "A head that will not feed is usually wound the wrong way or the line has welded into one lump: take the spool off, pull all the line out, rewind it in the direction of the arrow moulded on the spool and leave about 10 to 15cm out on each side. Old line goes brittle and fuses when it is held in long grass, so cut off the fused part and rewind the rest, or fit new line of the thickness the head says.".to_string();
    }
    if (says!(&consts("_CUTS_PLANTS")) || (strimmer && says(&["cutting the", "cuts the"]) && meets(&["plants", "flowers", "shrubs", "border", "borders", "beds"]))) {
        return "Keep the line off the beds by cutting the edge in two passes: the first with the head tilted so the line works the grass and not the soil, sweeping away from the border rather than along it. A line about 10cm long throws less than a long one, and a plant guard or a board stood along the edge catches what still flicks over; string a line between two canes where you cannot see the boundary.".to_string();
    }
    if (says!(&consts("_FAULT_IT")) && !says!(&consts("_DEFINITION_IT"))) {
        return "Treat it as three things in order: is it getting fuel and spark, is the cutting edge free, is anything loose. Tip out old petrol before anything else, dry or change the plug, wash the air filter, then take the guard off and clear the cuttings and grass packed behind it, which is what makes a head drag and a motor bog down. Check every nut, bolt and the bar or blade mounting for slack, because a machine that rattles is one that is about to throw something.".to_string();
    }

    let mut named: Vec<&str> = Vec::new();
    if strimmer {
        named.push("A strimmer cuts grass and edges of a lawn with a nylon line on a spinning head");
    }
    if hedge {
        named.push("a hedge trimmer is a bar of blades for the flat sides and top of a hedge");
    }
    if blower {
        named.push("a leaf blower is a fan in a tube that shifts leaves off gravel, decking and grass");
    }
    if saw {
        named.push("a chainsaw is a loop of chain on a bar, for timber only");
    }
    if tiller {
        named.push("a rotavator is a small engine turning a row of curved tines to break up and mix soil before planting");
    }
    if let Some(first) = named.first() {
        let rest = &named[1..];
        let said_them = if rest.is_empty() { String::new() } else { format!(" The others in the shed: {}.", rest.join("; ")) };
        let warning = if saw { " It is not a hedge tool - it chews small growth instead of cutting it clean, and it kicks." } else { "" };
        return format!("{}.{said_them}{warning}", text::upper_first(first));
    }
    "A strimmer cuts grass and edges of a lawn with a nylon line on a spinning head, a hedge trimmer is a bar of blades for the flat sides and top of a hedge, a leaf blower is a fan in a tube, and a chainsaw is a loop of chain on a bar for timber. The strimmer is for grass, the trimmer for a hedge's shape, and the chainsaw only for wood - not for a hedge, where it chews the growth instead of cutting it clean.".to_string()
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.garden_tools:_garden_machine:110", |_, m, u| Some(_garden_machine(m, u)))];
