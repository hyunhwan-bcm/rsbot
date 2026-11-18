//! Port of `bot/compose/fit.py`: the shape of answer a line wants (`wants` and every function it
//! reaches), and whether a reply gives that shape (`gives` and every reader it reaches).

#![allow(non_upper_case_globals)]

use crate::py::re::Re;
use crate::py::text;
use crate::understanding::{rx, rx_flags, t};
use crate::understood::Understood;
use indexmap::IndexMap;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.fit.";

static AMOUNT: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "AMOUNT"));
static ASKS_FOR_MEANING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_FOR_MEANING"));
static ASKS_ITS_OWN_YES_NO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_ITS_OWN_YES_NO"));
static ASKS_OUT_LOUD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_OUT_LOUD"));
static ASKS_TO_BE_TOLD_WHERE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_TO_BE_TOLD_WHERE"));
static ASKS_YES_OR_NO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_YES_OR_NO"));
static ASK_TO_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASK_TO_ASK"));
static A_DURATION_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_DURATION_WORD"));
static A_FEW_THINGS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_FEW_THINGS"));
static A_FIGURE_NAMED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_FIGURE_NAMED"));
static A_FIGURE_OF_A_MONEY_VERB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_FIGURE_OF_A_MONEY_VERB"));
static A_GOOD_TIME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_GOOD_TIME"));
static A_PRICE_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_PRICE_ASK"));
static A_RECALL_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_RECALL_ASK"));
static A_SIZE_LONG: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_SIZE_LONG"));
static A_THING_TO_GIVE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_THING_TO_GIVE"));
static BARE_A_RECALL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BARE_A_RECALL"));
static BARE_A_SECOND_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BARE_A_SECOND_QUESTION"));
static BARE_NOTHING_ASKED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BARE_NOTHING_ASKED"));
static BARE_NOT_A_THING_ANYWHERE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BARE_NOT_A_THING_ANYWHERE"));
static BARE_NOT_A_THING_AT_THE_START: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BARE_NOT_A_THING_AT_THE_START"));
static BARE_VALUE_NOUN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BARE_VALUE_NOUN"));
static BARE_WHAT_AS_IS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BARE_WHAT_AS_IS"));
static BARE_WHAT_IS_PLAIN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BARE_WHAT_IS_PLAIN"));
static BEST_OF_MANY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BEST_OF_MANY"));
static BEST_WAY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BEST_WAY"));
static COMPARE: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "COMPARE"));
static COMPARED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "COMPARED"));
static COMPARE_MYSELF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "COMPARE_MYSELF"));
static COMPARISON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "COMPARISON"));
static CONFIRM_CAUSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONFIRM_CAUSE"));
static CONVERSION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONVERSION"));
static CONVERTED_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONVERTED_FIGURE"));
static COSTS_NOTHING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "COSTS_NOTHING"));
static COST_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "COST_ASK"));
static COST_STANDS_IN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COST_STANDS_IN"));
static COUNTED_STEPS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "COUNTED_STEPS"));
static DIFFERENCE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DIFFERENCE"));
static DOING_WRONG: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DOING_WRONG"));
static DO_WITH_A_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DO_WITH_A_PERSON"));
static EXPLICIT_COMPARE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "EXPLICIT_COMPARE"));
static FIX_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FIX_ASK"));
static FOLLOWUP: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FOLLOWUP"));
static GIVE_ME_AN_IDIOM: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GIVE_ME_AN_IDIOM"));
static GIVE_ME_A_FEW: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GIVE_ME_A_FEW"));
static GIVE_ME_LEAD_IN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIVE_ME_LEAD_IN"));
static GIVE_ME_MADE_UP: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GIVE_ME_MADE_UP"));
static GIVE_ME_NOT_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GIVE_ME_NOT_A_THING"));
static GIVE_ME_ONE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GIVE_ME_ONE"));
static GIVING_FRAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GIVING_FRAME"));
static GOOD_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GOOD_FIGURE"));
static GOOD_FIGURE_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GOOD_FIGURE_WORD"));
static GOOD_MEASURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GOOD_MEASURE"));
static GOOD_ONE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GOOD_ONE"));
static GOOD_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GOOD_THING"));
static GOOD_WAY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GOOD_WAY"));
static HAVE_RECALL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HAVE_RECALL"));
static HELP_ASK_ANY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HELP_ASK_ANY"));
static HELP_ASK_NO_TASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HELP_ASK_NO_TASK"));
static HELP_ME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HELP_ME"));
static HELP_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HELP_WORD"));
static HOW_ARE_YOU: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_ARE_YOU"));
static HOW_ASK_FORM: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_ASK_FORM"));
static HOW_COME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_COME"));
static HOW_I: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_I"));
static HOW_IT_HAPPENED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_IT_HAPPENED"));
static HOW_IT_WENT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_IT_WENT"));
static HOW_IT_WORKS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_IT_WORKS"));
static HOW_MANY_TIME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_MANY_TIME"));
static HOW_MEASURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_MEASURE"));
static HOW_MEASURE_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_MEASURE_HEAD"));
static HOW_SOON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_SOON"));
static HOW_TO_ANSWER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_TO_ANSWER"));
static HOW_TO_DO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HOW_TO_DO"));
static IF_IT_HAPPENS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "IF_IT_HAPPENS"));
static JUDGED_MEASURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "JUDGED_MEASURE"));
static KINDS_OF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "KINDS_OF"));
static LEADING_JOIN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LEADING_JOIN"));
static LINE_TEXT_FIELDS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "LINE_TEXT_FIELDS"));  // t, 5 items
static LONG_TIME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LONG_TIME"));
static MADE_OF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MADE_OF"));
static MEANING: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "MEANING"));
static MEASURE_LABELS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEASURE_LABELS"));
static MEMBERS_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MEMBERS_ASK"));
static METHOD_OPENS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "METHOD_OPENS"));
static MONEY_VERB_NOT_A_PRICE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MONEY_VERB_NOT_A_PRICE"));
static MY_THING_CALLED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MY_THING_CALLED"));
static NEED_TO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NEED_TO"));
static NEED_TO_ASKED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NEED_TO_ASKED"));
static NEED_TO_DO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NEED_TO_DO"));
static NEW_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NEW_ASK"));
static NOT_A_COMPARISON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_A_COMPARISON"));
static NOT_A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_A_FIGURE"));
static NOT_A_GIVING_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_A_GIVING_THING"));
static NOT_A_PRICE_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_A_PRICE_ASK"));
static NOT_KINDS_OF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_KINDS_OF"));
static NOT_STEPS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_STEPS"));
static ON_THE_CLOCK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ON_THE_CLOCK"));
static OPEN_WHO_HELP: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OPEN_WHO_HELP"));
static OPTIONS: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "OPTIONS"));
static ORIGIN_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ORIGIN_ASK"));
static ORIGIN_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ORIGIN_PERSON"));
static OTHER_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OTHER_QUESTION"));
static PAIRING_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PAIRING_ASK"));
static PERSON: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "PERSON"));
static PICK: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "PICK"));
static PICK_ONE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PICK_ONE"));
static PICK_ONE_OF_A_KIND: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PICK_ONE_OF_A_KIND"));
static PICK_SIDES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PICK_SIDES"));
static PLACE: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "PLACE"));
static POINT_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "POINT_ASK"));
static POINT_NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POINT_NOT_A_THING"));
static PRICE_ADJECTIVE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PRICE_ADJECTIVE"));
static PRICE_ADJECTIVE_KEEPS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PRICE_ADJECTIVE_KEEPS"));
static PRICE_ADJECTIVE_THE_ASK_IS_LATER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PRICE_ADJECTIVE_THE_ASK_IS_LATER"));
static PRICE_ADJECTIVE_WHY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PRICE_ADJECTIVE_WHY"));
static PRICE_IDIOM: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PRICE_IDIOM"));
static QUESTION_IN_THE_LINE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "QUESTION_IN_THE_LINE"));
static REASON: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "REASON"));
static REASON_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "REASON_ASK"));
static RECALL_AMOUNT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RECALL_AMOUNT"));
static RECALL_KIND_NOUN: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "RECALL_KIND_NOUN"));
static RECALL_KIND_NOUN_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RECALL_KIND_NOUN_ASK"));
static RECALL_NOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RECALL_NOT"));
static RECALL_OF_MINE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RECALL_OF_MINE"));
static RECALL_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RECALL_PERSON"));
static RECALL_PLACE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RECALL_PLACE"));
static RECALL_PRONOUN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RECALL_PRONOUN"));
static RECALL_SAID: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RECALL_SAID"));
static RECALL_SHAPE_BEFORE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RECALL_SHAPE_BEFORE"));
static RECALL_TIME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RECALL_TIME"));
static REPORTED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "REPORTED"));
static SIDE_PAIRS: LazyLock<Vec<&'static Re>> = LazyLock::new(|| t::res(M, "SIDE_PAIRS"));
static SPECIFIC_ASKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SPECIFIC_ASKS"));
static STEPS: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "STEPS"));
static STEPS_ARE_COUNTED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STEPS_ARE_COUNTED"));
static STEPS_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STEPS_ASK"));
static STEPS_AS_A_DISTANCE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STEPS_AS_A_DISTANCE"));
static SUBSTITUTE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SUBSTITUTE"));
static SUM_ASKS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SUM_ASKS"));
static SUM_OUT_LOUD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SUM_OUT_LOUD"));
static TAIL_VERDICT_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TAIL_VERDICT_ASK"));
static THING_THAT_EXTENDS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THING_THAT_EXTENDS"));
static TIME: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "TIME"));
static VERDICT_FRAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_FRAME"));
static WHAT_AGE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_AGE"));
static WHAT_ARE_MINE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_ARE_MINE"));
static WHAT_ARE_TIMES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_ARE_TIMES"));
static WHAT_CAN_I_DO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_CAN_I_DO"));
static WHAT_CAN_I_DO_WITH: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_CAN_I_DO_WITH"));
static WHAT_DOES_MEASURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_DOES_MEASURE"));
static WHAT_ELSE_STEPS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_ELSE_STEPS"));
static WHAT_ELSE_WITH: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_ELSE_WITH"));
static WHAT_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_FIGURE"));
static WHAT_HELPS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_HELPS"));
static WHAT_IS_PER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_IS_PER"));
static WHAT_I_GROW: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_I_GROW"));
static WHAT_I_NEED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_I_NEED"));
static WHAT_I_WRITE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_I_WRITE"));
static WHAT_SHOULD_I_DO_WITH: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_SHOULD_I_DO_WITH"));
static WHAT_THING_FRAGMENT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_THING_FRAGMENT"));
static WHAT_TIME_NAMED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_TIME_NAMED"));
static WHAT_TO_BAKE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_TO_BAKE"));
static WHAT_TO_DO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_TO_DO"));
static WHAT_TO_OPENS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_TO_OPENS"));
static WHAT_TO_PICK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_TO_PICK"));
static WHAT_TO_STEPS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_TO_STEPS"));
static WHAT_YOU_DO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_YOU_DO"));
static WHAT_YOU_TAKE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_YOU_TAKE"));
static WHEN_ASKS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHEN_ASKS"));
static WHERE_ALONE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHERE_ALONE"));
static WHERE_MONEY_GOES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHERE_MONEY_GOES"));
static WHERE_POINT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHERE_POINT"));
static WHERE_RECALL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHERE_RECALL"));
static WHERE_RELATIVE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHERE_RELATIVE"));
static WHERE_START: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHERE_START"));
static WHICH_DAY_IS_BEST: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHICH_DAY_IS_BEST"));
static WHO_AND_MEASURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHO_AND_MEASURE"));
static WHO_OF_A_NOUN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHO_OF_A_NOUN"));
static WHO_OF_A_PRONOUN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHO_OF_A_PRONOUN"));
static WHO_OPENS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHO_OPENS"));
static WRITTEN_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WRITTEN_THING"));
static WRONG_ASKED_ABOUT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WRONG_ASKED_ABOUT"));
static WRONG_RECALL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WRONG_RECALL"));
static YESNO: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "YESNO"));
static _ASK_CLAUSE_OPENS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ASK_CLAUSE_OPENS"));
static _ASK_CLAUSE_SPLIT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ASK_CLAUSE_SPLIT"));
static _MEASURE_THEN_VERB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_MEASURE_THEN_VERB"));
static _TALKING_ABOUT_A_LENGTH: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_TALKING_ABOUT_A_LENGTH"));
static _TRAILING_MEASURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_TRAILING_MEASURE"));
/// `ASKED_FOR`: (shape, the ask labels that want it), in order.
static ASKED_FOR: LazyLock<Vec<(String, HashSet<String>)>> =
    LazyLock::new(|| t::value(M, "ASKED_FOR").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str_set())).collect());
/// `RECALL_SHAPE_BEFORE_KIND`: (the words, the shape).
static RECALL_SHAPE_BEFORE_KIND: LazyLock<Vec<(Vec<String>, String)>> =
    LazyLock::new(|| t::value(M, "RECALL_SHAPE_BEFORE_KIND").items().iter().map(|p| (p.items()[0].strs(), p.items()[1].str().to_string())).collect());

fn s(v: &LazyLock<&'static str>) -> String {
    v.to_string()
}

/// `_said(understood)`: norm, norm_text, clean and text in one lower-cased line.
fn _said(u: &Understood) -> String {
    let mut parts: Vec<String> = Vec::new();
    for value in [u.norm.join(" "), u.norm_text.clone(), u.clean.clone(), u.text.clone()] {
        if !value.is_empty() {
            parts.push(value);
        }
    }
    text::squash(&parts.join(" ").to_lowercase().replace('\u{2019}', "'"))
}

/// `_a_who_a_person_owns(understood)`: "my friend who ..." - the who is not the ask.
fn _a_who_a_person_owns(u: &Understood) -> bool {
    for e in &u.entities {
        if e.kind != "person" || e.at < 0 {
            continue;
        }
        let after = e.at as usize + text::split_ref(&e.text).len();
        let word = if after > 0 && after < u.words.len() { text::strip_chars(&u.words[after].to_lowercase(), "'\u{2019}").to_string() } else { String::new() };
        if word.starts_with("who") || word == "that" {
            return true;
        }
    }
    false
}

/// `_names_a_pair(said)`.
fn _names_a_pair(said: &str) -> bool {
    SIDE_PAIRS.iter().any(|p| p.is_match(said))
}

/// `_asks_about_them(understood, said)`.
fn _asks_about_them(u: &Understood, said: &str) -> bool {
    if u.intent == "news" {
        return false;
    }
    EXPLICIT_COMPARE.is_match(said) || u.kind == "question"
}

/// `_a_help_ask_that_names_nothing(said)`: "can you help me?" with no task named.
fn _a_help_ask_that_names_nothing(said: &str) -> bool {
    let m = HELP_ASK_ANY.search(said).or_else(|| HELP_WORD.search(said));
    let rest = match m {
        Some(m) => &said[m.end()..],
        None => said,
    };
    HELP_ASK_NO_TASK.is_match(rest)
}

/// `_whole(understood)`: the line as typed (plain, else text), lower-cased.
fn _whole(u: &Understood) -> String {
    for value in [&u.plain, &u.text] {
        if !text::strip(value).is_empty() {
            return text::squash(&value.to_lowercase().replace('\u{2019}', "'"));
        }
    }
    String::new()
}

/// `_the_give_is_to_the_bot(said, at)`.
fn _the_give_is_to_the_bot(said: &str, at: usize) -> bool {
    let before = text::split_ref(&said[..at]);
    before.last().is_none_or(|w| GIVE_ME_LEAD_IN.contains(*w))
}

/// `_recall_wants(said, line='')`: "what time did i say" -> time.
fn _recall_wants(said: &str, line: &str) -> Option<String> {
    let whole = text::squash(&(if line.is_empty() { said } else { line }).to_lowercase().replace('\u{2019}', "'"));
    if whole.is_empty() || !RECALL_SAID.is_match(&whole) || RECALL_NOT.is_match(&whole) {
        return None;
    }
    for m in RECALL_SAID.finditer(&whole) {
        let after = RECALL_PRONOUN.sub("", text::strip(&whole[m.end()..]));
        let kind = _recall_kind_in_front(&after).or_else(|| _recall_kind_named(&after)).or_else(|| _recall_kind_behind(&whole[..m.start()]));
        if kind.is_some() {
            return kind;
        }
    }
    None
}

/// `_recall_kind_in_front(after)`.
fn _recall_kind_in_front(after: &str) -> Option<String> {
    for (pattern, kind) in [(&RECALL_TIME, &TIME), (&RECALL_AMOUNT, &AMOUNT), (&RECALL_PLACE, &PLACE), (&RECALL_PERSON, &PERSON)] {
        if pattern.match_(after).is_some() {
            return Some(s(kind));
        }
    }
    None
}

/// `_recall_kind_named(after)`.
fn _recall_kind_named(after: &str) -> Option<String> {
    let noun = RECALL_KIND_NOUN_ASK.match_(after)?;
    let key = noun.str(1).to_lowercase();
    Some(RECALL_KIND_NOUN.get(&key).cloned().unwrap_or_else(|| panic!("KeyError: {key}")))
}

/// `_recall_kind_behind(before)`.
fn _recall_kind_behind(before: &str) -> Option<String> {
    let m = RECALL_SHAPE_BEFORE.search(before)?;
    let named = text::squash(&m.as_str().to_lowercase());
    for (words, kind) in RECALL_SHAPE_BEFORE_KIND.iter() {
        for word in words {
            if named == *word || named.starts_with(&format!("{word} ")) {
                return Some(kind.clone());
            }
        }
    }
    None
}

/// `asks_a_follow_up(understood)`: "and for brown rice?" asks the same shape again.
pub fn asks_a_follow_up(u: &Understood) -> bool {
    if u.asks_a_verdict {
        return false;
    }
    let said = _said(u);
    if !FOLLOWUP.is_match(&said) {
        return false;
    }
    if u.asks.iter().any(|a| a != "what" && a != "yesno") {
        return false;
    }
    if NEW_ASK.is_match(&said) {
        return false;
    }
    if u.kind == "question" || text::rstrip(&said).ends_with('?') {
        return true;
    }
    let line = if u.plain.is_empty() { &u.text } else { &u.plain };
    u.kind == "statement" && u.followup && text::split_ref(line).len() <= 6
}

/// `_a_fragment_asks_for_a_time(said)`: "the 10 minute walk" vs "walk for how long".
fn _a_fragment_asks_for_a_time(said: &str) -> bool {
    if _TALKING_ABOUT_A_LENGTH.is_match(said) {
        return false;
    }
    _TRAILING_MEASURE.is_match(said) || _MEASURE_THEN_VERB.is_match(said)
}

/// `_the_ask_clause(said)`: the last clause when it opens with a question word.
fn _the_ask_clause(said: &str) -> String {
    let parts: Vec<String> = _ASK_CLAUSE_SPLIT.split(said).into_iter().map(|p| text::strip(&p).to_string()).filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return String::new();
    }
    let last = parts.last().unwrap();
    if _ASK_CLAUSE_OPENS.match_(last).is_some() { last.clone() } else { String::new() }
}

/// `wants(understood, before=None)`: the shape of answer the line wants ("time", "steps", ...),
/// or None.
pub fn wants(u: &Understood, before: Option<&str>) -> Option<String> {
    if A_RECALL_ASK.is_match(&_said(u)) {
        return None;
    }
    let own = _wants_alone(u);
    let own = _the_shape_named_outside_the_reading(u, own);
    if own.as_deref().is_some_and(|o| !o.is_empty()) {
        return own;
    }
    if _a_bare_definition(u) {
        return Some(s(&MEANING));
    }
    if let Some(b) = before.filter(|b| !b.is_empty()) {
        if asks_a_follow_up(u) {
            return Some(b.to_string());
        }
    }
    None
}

/// `_the_shape_named_outside_the_reading(understood, own)`.
fn _the_shape_named_outside_the_reading(u: &Understood, own: Option<String>) -> Option<String> {
    let said = _said(u);
    if said.is_empty() {
        return own;
    }
    if IF_IT_HAPPENS.match_(&said).is_some() {
        return own;
    }
    if own.as_deref() == Some(*REASON) && CONFIRM_CAUSE.is_match(&said) {
        return Some(s(&YESNO));
    }
    if (own.is_none() || own.as_deref() == Some(*REASON) || own.as_deref() == Some(*MEANING)) && ASK_TO_ASK.is_match(&said) {
        return Some(s(&YESNO));
    }
    if own.is_none() && A_FIGURE_NAMED.is_match(&said) {
        return Some(if WHAT_TIME_NAMED.is_match(&said) { s(&TIME) } else { s(&AMOUNT) });
    }
    own
}

/// `_wants_alone(understood)`: the shape the reading alone asks for.
fn _wants_alone(u: &Understood) -> Option<String> {
    let said = _said(u);
    let whole = _whole(u);
    if let Some(asked_back) = _recall_wants(&said, &whole) {
        return Some(asked_back);
    }
    if u.about_bot {
        return None;
    }
    if u.asks_a_verdict {
        return Some(s(&YESNO));
    }
    if OPEN_WHO_HELP.is_match(&said) {
        return Some(s(&PERSON));
    }
    let typed = if whole.is_empty() { said.clone() } else { whole.clone() };
    if u.help_ask {
        return if _a_help_ask_that_names_nothing(&typed) { None } else { Some(s(&STEPS)) };
    }
    let mut asks: HashSet<String> = u.asks.iter().cloned().collect();
    if u.intent == "news" {
        asks.remove("choice");
        if !(SUM_OUT_LOUD.is_match(&said) || SUM_ASKS.is_match(&said)) {
            asks.remove("maths");
        }
    }
    let said = said.as_str();
    let has = |a: &HashSet<String>, k: &str| a.contains(k);
    let asking = u.kind == "question" || u.intent == "question" || u.intent == "request" || HOW_ASK_FORM.is_match(said) || (QUESTION_IN_THE_LINE.is_match(said) && !REPORTED.is_match(said));
    if HOW_ARE_YOU.match_(said).is_some() || HOW_IT_WENT.is_match(said) {
        return None;
    }
    if HOW_IT_HAPPENED.is_match(said) {
        return Some(s(&REASON));
    }
    if HELP_ME.is_match(said) {
        return Some(s(&STEPS));
    }
    if HELP_ASK_ANY.is_match(&typed) {
        return if _a_help_ask_that_names_nothing(&typed) { None } else { Some(s(&STEPS)) };
    }
    if asking && !REPORTED.is_match(said) && DOING_WRONG.is_match(said) && !WRONG_ASKED_ABOUT.is_match(said) && !WRONG_RECALL.is_match(said) {
        return Some(s(&REASON));
    }
    if FIX_ASK.is_match(said) && rx(r"\s*(?:is|are|can|could|will|would|do|does)\b").match_(said).is_none() {
        return Some(s(&STEPS));
    }
    if WHERE_START.is_match(said) || BEST_WAY.is_match(said) {
        return Some(s(&STEPS));
    }
    if NEED_TO.is_match(said) && !NEED_TO_ASKED.is_match(said) && asks.iter().all(|a| matches!(a.as_str(), "maths" | "choice" | "write")) {
        return Some(s(&STEPS));
    }
    if WHERE_POINT.is_match(said) {
        return None;
    }
    if WHERE_RECALL.is_match(said) || WHERE_ALONE.is_match(&whole) {
        return None;
    }
    if HOW_TO_ANSWER.is_match(said) {
        return Some(s(&STEPS));
    }
    if PRICE_IDIOM.is_match(said) && asking && rx(r"\b(?:what|how much)\b").is_match(said) {
        return Some(s(&AMOUNT));
    }
    if HOW_COME.is_match(said) {
        return Some(s(&REASON));
    }
    if has(&asks, "steps") && HOW_I.is_match(said) && !COMPARE_MYSELF.is_match(said) && !((has(&asks, "maths") || has(&asks, "how_much")) && rx("[0-9]").is_match(said)) {
        return Some(s(&STEPS));
    }
    if STEPS_ASK.is_match(said) && !(COUNTED_STEPS.is_match(said) && !rx(r"\bhow (?:do|can|should) i\b").is_match(said)) {
        return Some(s(&STEPS));
    }
    if MEMBERS_ASK.is_match(said) {
        return Some(s(&OPTIONS));
    }
    if KINDS_OF.is_match(said) && !NOT_KINDS_OF.is_match(said) {
        return Some(s(&OPTIONS));
    }
    if (COMPARISON.is_match(said) || COMPARED.is_match(said)) && !NOT_A_COMPARISON.is_match(said) && _names_a_pair(said) && _asks_about_them(u, said) {
        return Some(s(&COMPARE));
    }
    if (ORIGIN_ASK.is_match(said) && !ORIGIN_PERSON.is_match(said)) || MADE_OF.is_match(said) {
        return Some(s(&MEANING));
    }
    if BEST_OF_MANY.is_match(said) && !(COST_ASK.is_match(said) && asks.iter().all(|a| COST_STANDS_IN.contains(a))) {
        return Some(s(&OPTIONS));
    }
    if has(&asks, "definition") && WHAT_ARE_MINE.is_match(said) && !rx(r"\b(?:called|mean|means|meaning|stand for|stands for)\b").is_match(said) {
        return Some(s(&STEPS));
    }
    if MY_THING_CALLED.is_match(said) {
        return None;
    }
    if has(&asks, "definition") && rx(r"\bwhat does\b").is_match(said) && !ASKS_FOR_MEANING.is_match(said) {
        asks.remove("definition");
    }
    if has(&asks, "definition") && !asking && !ASKS_FOR_MEANING.is_match(said) {
        asks.remove("definition");
    }
    if GOOD_MEASURE.is_match(said) {
        return Some(s(&AMOUNT));
    }
    if GOOD_THING.is_match(said) && has(&asks, "definition") && !GOOD_FIGURE_WORD.is_match(said) {
        return Some(if GOOD_WAY.is_match(said) { s(&STEPS) } else { s(&OPTIONS) });
    }
    if HOW_IT_WORKS.is_match(said) && !rx_flags(r"\bhow do i\b", crate::py::re::I).is_match(said) {
        return Some(s(&MEANING));
    }
    if has(&asks, "who") && WHO_OPENS.match_(said).is_none() && _a_who_a_person_owns(u) {
        asks.remove("who");
    }
    if has(&asks, "who") && (WHO_AND_MEASURE.is_match(said) || WHO_OF_A_PRONOUN.is_match(said) || (WHO_OF_A_NOUN.is_match(said) && WHO_OPENS.match_(said).is_none())) {
        asks.remove("who");
    }
    if has(&asks, "where") && WHERE_MONEY_GOES.is_match(said) {
        return Some(s(&STEPS));
    }
    if has(&asks, "where") && (has(&asks, "yesno") || has(&asks, "recommend")) && WHERE_RELATIVE.is_match(said) && !ASKS_TO_BE_TOLD_WHERE.is_match(said) {
        asks.remove("where");
    }
    if LONG_TIME.is_match(said) {
        asks.remove("how_long");
        asks.remove("how_often");
    }
    if CONVERSION.is_match(said) || CONVERTED_FIGURE.is_match(said) {
        return Some(s(&AMOUNT));
    }
    if NOT_A_FIGURE.is_match(said) {
        return None;
    }
    if asks.iter().any(|a| MEASURE_LABELS.contains(a)) && asking {
        if has(&asks, "how_long") && WRITTEN_THING.is_match(said) {
            return Some(s(&AMOUNT));
        }
        if has(&asks, "how_long") && THING_THAT_EXTENDS.is_match(said) {
            return Some(s(&AMOUNT));
        }
        if has(&asks, "how_long") && has(&asks, "steps") && METHOD_OPENS.match_(&LEADING_JOIN.sub("", said)).is_some() {
            return Some(s(&STEPS));
        }
        if has(&asks, "how_long") && A_SIZE_LONG.is_match(said) && !A_DURATION_WORD.is_match(said) {
            if has(&asks, "how_much") || has(&asks, "how_many") || rx(r"\bhow (?:much|many)\b").is_match(said) {
                return Some(s(&AMOUNT));
            }
            return None;
        }
        return Some(if has(&asks, "how_long") || HOW_MANY_TIME.is_match(said) { s(&TIME) } else { s(&AMOUNT) });
    }
    if WHAT_DOES_MEASURE.is_match(said) && !ASKS_FOR_MEANING.is_match(said) && !COSTS_NOTHING.is_match(said) {
        return Some(s(&AMOUNT));
    }
    if WHAT_IS_PER.is_match(said) {
        return Some(s(&AMOUNT));
    }
    if has(&asks, "how") {
        if HOW_SOON.is_match(said) {
            return Some(s(&TIME));
        }
        if HOW_MEASURE.is_match(said) {
            return Some(s(&AMOUNT));
        }
        if HOW_MEASURE_HEAD.match_(&LEADING_JOIN.sub("", said)).is_some() {
            return Some(s(&AMOUNT));
        }
        if HOW_TO_DO.is_match(said) {
            asks.remove("definition");
        }
    }
    if _a_fragment_asks_for_a_time(said) {
        return Some(s(&TIME));
    }
    let asked = LEADING_JOIN.sub("", said);
    let focus = if u.asking.is_empty() { &u.focus } else { &u.asking };
    let focus_said = focus.join(" ");
    if (WHEN_ASKS.match_(&asked).is_some() || WHEN_ASKS.match_(&LEADING_JOIN.sub("", text::strip(&focus_said))).is_some()) && !WHICH_DAY_IS_BEST.is_match(said) {
        return Some(s(&TIME));
    }
    let ask_clause = if asking { _the_ask_clause(said) } else { String::new() };
    if !ask_clause.is_empty() && WHEN_ASKS.match_(&LEADING_JOIN.sub("", &ask_clause)).is_some() && !WHICH_DAY_IS_BEST.is_match(said) {
        return Some(s(&TIME));
    }
    if WHAT_ARE_TIMES.match_(said).is_some() && !REPORTED.is_match(said) && !rx(r"\b(?:called|mean|means|meaning|stand for)\b").is_match(said) {
        return Some(s(&TIME));
    }
    if A_GOOD_TIME.is_match(said) {
        return Some(s(&YESNO));
    }
    if GOOD_FIGURE.is_match(said) && VERDICT_FRAME.match_(said).is_none() {
        return Some(if rx(r"\b(?:time|timeframe)\b").is_match(said) { s(&TIME) } else { s(&AMOUNT) });
    }
    if JUDGED_MEASURE.is_match(said) && asking && !ON_THE_CLOCK.is_match(said) && !REPORTED.is_match(said) && ASKS_YES_OR_NO.match_(&LEADING_JOIN.sub("", said)).is_none() {
        return Some(s(&AMOUNT));
    }
    if GOOD_ONE.is_match(said) {
        return Some(s(&OPTIONS));
    }
    if asking && !GIVE_ME_NOT_A_THING.is_match(said) && !GIVE_ME_MADE_UP.is_match(said) && !GIVE_ME_AN_IDIOM.is_match(said) && !has(&asks, "list") {
        if let Some(found) = GIVE_ME_A_FEW.search(said) {
            if _the_give_is_to_the_bot(said, found.start()) {
                return Some(s(&OPTIONS));
            }
        }
        if let Some(found) = GIVE_ME_ONE.search(said) {
            if _the_give_is_to_the_bot(said, found.start()) {
                return Some(s(&PICK));
            }
        }
    }
    if A_THING_TO_GIVE.is_match(said) && GIVING_FRAME.is_match(said) && !has(&asks, "yesno") && !NOT_A_GIVING_THING.is_match(said) {
        return Some(s(&OPTIONS));
    }
    if WHAT_FIGURE.is_match(said) || WHAT_AGE.is_match(said) {
        return Some(s(&AMOUNT));
    }
    if NEED_TO_DO.is_match(said) {
        return Some(s(&STEPS));
    }
    if WHAT_HELPS.is_match(said) || WHAT_YOU_TAKE.is_match(said) || WHAT_I_NEED.is_match(said) {
        return Some(s(&OPTIONS));
    }
    if WHAT_YOU_DO.is_match(said) {
        return Some(s(&STEPS));
    }
    if !REPORTED.is_match(said) && OTHER_QUESTION.match_(said).is_none() {
        if WHAT_I_WRITE.is_match(said) {
            return Some(s(&STEPS));
        }
        if WHAT_I_GROW.is_match(said) {
            return Some(s(&OPTIONS));
        }
    }
    if WHAT_TO_BAKE.is_match(said) {
        return Some(s(&OPTIONS));
    }
    if WHAT_THING_FRAGMENT.is_match(said) && !SUBSTITUTE.is_match(said) {
        return Some(s(&PICK));
    }
    if DO_WITH_A_PERSON.is_match(said) {
        return None;
    }
    if asking && (WHAT_CAN_I_DO_WITH.is_match(said) || WHAT_SHOULD_I_DO_WITH.is_match(said)) {
        if asks.iter().any(|a| PICK_SIDES.contains(a)) {
            return Some(s(&PICK));
        }
        if WHAT_SHOULD_I_DO_WITH.is_match(said) {
            return Some(s(&STEPS));
        }
        return Some(s(&OPTIONS));
    }
    if WHAT_ELSE_WITH.is_match(said) {
        return Some(s(&OPTIONS));
    }
    if WHAT_ELSE_STEPS.is_match(said) {
        return Some(s(&STEPS));
    }
    if PICK_ONE.is_match(said) || PICK_ONE_OF_A_KIND.is_match(said) || (WHAT_TO_OPENS.is_match(said) && WHAT_TO_PICK.is_match(said)) {
        return if HAVE_RECALL.match_(said).is_some() { None } else { Some(s(&PICK)) };
    }
    if SUBSTITUTE.is_match(said) {
        return Some(s(&OPTIONS));
    }
    if PAIRING_ASK.is_match(said) && !REPORTED.is_match(said) {
        return Some(s(&OPTIONS));
    }
    if (WHAT_TO_DO.is_match(said) || (WHAT_TO_OPENS.is_match(said) && WHAT_TO_STEPS.is_match(said))) && !REPORTED.is_match(said) && !NOT_STEPS.is_match(said) && OTHER_QUESTION.match_(said).is_none() {
        return Some(s(&STEPS));
    }
    if asking && !REPORTED.is_match(said) && REASON_ASK.is_match(said) {
        return Some(s(&REASON));
    }
    if let Some(found) = POINT_ASK.search(said) {
        if !ASKS_ITS_OWN_YES_NO.is_match(said) {
            let g = found.get(1).filter(|g| !g.is_empty()).or(found.get(2)).unwrap_or("");
            let thing = text::split_ref(g);
            if thing.first().is_some_and(|t_| !POINT_NOT_A_THING.contains(*t_)) {
                return Some(s(&REASON));
            }
        }
    }
    if asks.iter().all(|a| a == "recommend" || MEASURE_LABELS.contains(a)) && !text::rstrip(said).ends_with('?') && ASKS_OUT_LOUD.match_(said).is_none() && !TAIL_VERDICT_ASK.is_match(&whole) {
        return None;
    }
    if DIFFERENCE.is_match(said) {
        return Some(s(&MEANING));
    }
    if asks.iter().any(|a| PICK_SIDES.contains(a)) {
        return Some(s(&PICK));
    }
    if STEPS_ARE_COUNTED.is_match(said) || STEPS_AS_A_DISTANCE.is_match(said) {
        asks.remove("steps");
        if STEPS_AS_A_DISTANCE.is_match(said) {
            return Some(s(&AMOUNT));
        }
    }
    if PRICE_ADJECTIVE.is_match(said) && !PRICE_ADJECTIVE_KEEPS.is_match(said) && !PRICE_ADJECTIVE_THE_ASK_IS_LATER.is_match(&_line_text(&u.text)) && !asks.iter().any(|a| PRICE_ADJECTIVE_WHY.contains(a)) {
        return Some(s(&AMOUNT));
    }
    if PRICE_ADJECTIVE.is_match(said) && asks.iter().any(|a| PRICE_ADJECTIVE_WHY.contains(a)) {
        return Some(s(&REASON));
    }
    if A_PRICE_ASK.is_match(said) && !NOT_A_PRICE_ASK.is_match(said) {
        return Some(s(&AMOUNT));
    }
    if RECALL_OF_MINE.is_match(said) {
        asks.remove("yesno");
    }
    if has(&asks, "yesno") && !asks.iter().any(|a| SPECIFIC_ASKS.contains(a)) {
        return Some(s(&YESNO));
    }
    if COST_ASK.is_match(said) && asks.iter().all(|a| COST_STANDS_IN.contains(a)) {
        return Some(s(&AMOUNT));
    }
    if A_FIGURE_OF_A_MONEY_VERB.is_match(said) && !MONEY_VERB_NOT_A_PRICE.is_match(said) {
        return Some(s(&AMOUNT));
    }
    if WHAT_CAN_I_DO.is_match(said) && !asks.iter().any(|a| PICK_SIDES.contains(a)) {
        return Some(s(&STEPS));
    }
    for (shape, labels) in ASKED_FOR.iter() {
        let hit = asks.iter().any(|a| (asking || !MEASURE_LABELS.contains(a)) && labels.contains(a));
        if hit {
            if shape == *PICK && labels.contains("recommend") && A_FEW_THINGS.is_match(said) {
                return Some(s(&OPTIONS));
            }
            if shape == *AMOUNT && has(&asks, "maths") && !has(&asks, "how_much") && u.kind != "question" && !(SUM_OUT_LOUD.is_match(said) || SUM_ASKS.is_match(said)) {
                continue;
            }
            return Some(shape.clone());
        }
    }
    if TAIL_VERDICT_ASK.is_match(&whole) {
        return Some(s(&YESNO));
    }
    None
}

/// `_a_thing_to_define(phrase, whole)`.
fn _a_thing_to_define(phrase: &str, whole: &str) -> bool {
    if text::strip(phrase).is_empty() {
        return false;
    }
    if BARE_NOTHING_ASKED.contains(&text::strip(phrase).to_lowercase()) {
        return false;
    }
    if BARE_NOT_A_THING_AT_THE_START.match_(phrase).is_some() {
        return false;
    }
    if BARE_VALUE_NOUN.match_(phrase).is_some() && !rx_flags(r"\b(?:mean|means|work|works)\b", crate::py::re::I).is_match(phrase) {
        return false;
    }
    if BARE_NOT_A_THING_ANYWHERE.is_match(whole) || BARE_A_SECOND_QUESTION.is_match(phrase) || BARE_A_RECALL.is_match(phrase) {
        return false;
    }
    if text::rstrip(phrase).to_lowercase().ends_with(" for") {
        return false;
    }
    true
}

/// `_a_bare_definition(understood)`: "photosynthesis?" asks what it is.
fn _a_bare_definition(u: &Understood) -> bool {
    if u.about_bot {
        return false;
    }
    let whole = BARE_WHAT_AS_IS.sub("what is", &LEADING_JOIN.sub("", &_whole(u)));
    match BARE_WHAT_IS_PLAIN.match_(&whole) {
        Some(plain) => _a_thing_to_define(plain.name("thing").unwrap_or(""), &whole),
        None => false,
    }
}

/// `_line_text(line)` for a text: lower-cased, whitespace collapsed.
fn _line_text(line: &str) -> String {
    text::squash(&line.to_lowercase().replace('\u{2019}', "'"))
}

/// The line-text fields a reading is read by (`LINE_TEXT_FIELDS`).
pub fn line_text_fields() -> &'static [String] {
    &LINE_TEXT_FIELDS
}

#[allow(dead_code)]
fn _unused(_: &Re, _: &IndexMap<String, String>) {}

// The tables the reply side reads.
static ABBREVIATION_END: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ABBREVIATION_END"));
static ANY_DO_WITH: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ANY_DO_WITH"));
static AN_INSTRUCTION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "AN_INSTRUCTION"));
static AN_OFFER_NOT_A_TEST: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "AN_OFFER_NOT_A_TEST"));
static AN_ORDINAL_OPENING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "AN_ORDINAL_OPENING"));
static ASKS_ABOUT_A_QUALITY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_ABOUT_A_QUALITY"));
static ASKS_AN_AGE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_AN_AGE"));
static ASKS_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_HEAD"));
static ASKS_IF_SERIOUS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_IF_SERIOUS"));
static ASKS_THE_BOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_THE_BOT"));
static ASKS_WHEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKS_WHEN"));
static A_BARE_NUMBER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_BARE_NUMBER"));
static A_BOUND_THAT_IS_THE_ANSWER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_BOUND_THAT_IS_THE_ANSWER"));
static A_CALCULATION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_CALCULATION"));
static A_COUNT_OF_THINGS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_COUNT_OF_THINGS"));
static A_COVER_LETTER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_COVER_LETTER"));
static A_HARM_IS_WHAT_IT_IS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_HARM_IS_WHAT_IT_IS"));
static A_HOLDER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_HOLDER"));
static A_JOB_LANDS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_JOB_LANDS"));
static A_JUDGEMENT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_JUDGEMENT"));
static A_LIMIT_ON_ITS_USE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_LIMIT_ON_ITS_USE"));
static A_QUANTITY_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_QUANTITY_ASK"));
static A_RULE_THAT_SETTLES_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_RULE_THAT_SETTLES_IT"));
static A_SIDE_WINS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_SIDE_WINS"));
static A_STANDING_AFTER_THE_TEST: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_STANDING_AFTER_THE_TEST"));
static A_SUBORDINATE_FRAGMENT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_SUBORDINATE_FRAGMENT"));
static A_TIME_COUNT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_TIME_COUNT"));
static A_TRADE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_TRADE"));
static A_UNIT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_UNIT"));
static A_VERB_THEN_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_VERB_THEN_THE_THING"));
static A_WELL_WISHING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_WELL_WISHING"));
static BACK_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BACK_HEAD"));
static BALD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BALD"));
static BALD_VERB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BALD_VERB"));
static BARE_RANGE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BARE_RANGE"));
static CALENDAR_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CALENDAR_WORD"));
static CAUSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CAUSE"));
static CAUSE_IS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CAUSE_IS"));
static CAUSE_WHEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CAUSE_WHEN"));
static CHOICE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CHOICE"));
static CLOCK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CLOCK"));
static CLOCK_SURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CLOCK_SURE"));
static COMPARATIVE_THAN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "COMPARATIVE_THAN"));
static CONTRACTION_OPENER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONTRACTION_OPENER"));
static CONTRAST: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONTRAST"));
static COPULA: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "COPULA"));
static COUNTED_LIST: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "COUNTED_LIST"));
static DATE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DATE"));
static DAY_NAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DAY_NAME"));
static DEFINES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DEFINES"));
static DIRECTED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DIRECTED"));
static DOES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DOES"));
static DOES_NOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DOES_NOT"));
static DOES_NOT_ANYWHERE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DOES_NOT_ANYWHERE"));
static DURATION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "DURATION"));
static ECHO_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ECHO_HEAD"));
static ECHO_VERDICT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ECHO_VERDICT"));
static EDGED_PART_OF_DAY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "EDGED_PART_OF_DAY"));
static EVIDENCE_SAYS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "EVIDENCE_SAYS"));
static EXPLAINS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "EXPLAINS"));
static EXTENT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "EXTENT"));
static FIGURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FIGURE"));
static FIGURE_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FIGURE_HEAD"));
static FIRST_PERSON_VERDICT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FIRST_PERSON_VERDICT"));
static FOR_A_LIST: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FOR_A_LIST"));
static FOR_A_TIME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FOR_A_TIME"));
static FRAME_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FRAME_HEAD"));
static GARDEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "GARDEN"));
static HALF_A_UNIT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HALF_A_UNIT"));
static HANDOFF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HANDOFF"));
static HYPOTHETICAL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "HYPOTHETICAL"));
static IMPERATIVE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "IMPERATIVE"));
static INSTRUCTS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "INSTRUCTS"));
static INSTRUCTS_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "INSTRUCTS_CLAUSE"));
static IS_IT_DONE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "IS_IT_DONE"));
static IT_HELPS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "IT_HELPS"));
static I_OFFER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "I_OFFER"));
static JOB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "JOB"));
static JOB_ANY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "JOB_ANY"));
static LABEL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LABEL"));
static LENGTH_ASKED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LENGTH_ASKED"));
static LIST_AFTER_COLON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LIST_AFTER_COLON"));
static MEANS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MEANS"));
static MEANS_ARTICLE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MEANS_ARTICLE"));
static MEANS_BY_DOING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MEANS_BY_DOING"));
static MEANS_INFINITIVE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MEANS_INFINITIVE"));
static MEANS_NAMED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MEANS_NAMED"));
static MEANS_OF: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MEANS_OF"));
static MEASURED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MEASURED"));
static MECHANISM: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MECHANISM"));
static MERIT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MERIT"));
static MINUTES_AS_A_LENGTH: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MINUTES_AS_A_LENGTH"));
static MONEY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MONEY"));
static MONEY_RATE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MONEY_RATE"));
static MUST_BE_A_TRADE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MUST_BE_A_TRADE"));
static NAMED_CLOCK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAMED_CLOCK"));
static NAMED_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAMED_HEAD"));
static NAMED_TIME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAMED_TIME"));
static NAMED_WORDS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAMED_WORDS"));
static NAMES_IN_EMPHASIS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAMES_IN_EMPHASIS"));
static NAME_STOP: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAME_STOP"));
static NAMING_STOP: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NAMING_STOP"));
static NEGATED_AUX: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NEGATED_AUX"));
static NEVER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NEVER"));
static NEXT_IDEA: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NEXT_IDEA"));
static NOTHING_TO_GO_ON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOTHING_TO_GO_ON"));
static NOT_AN_OPENER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_AN_OPENER"));
static NOT_A_LABEL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_A_LABEL"));
static NOT_A_STEP_BUT_REASSURANCE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_A_STEP_BUT_REASSURANCE"));
static NOT_A_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_A_SUBJECT"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_THING"));
static NOT_A_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_VERB"));
static NOT_WORDS_ANYWAY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_WORDS_ANYWAY"));
static NOT_WORDS_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_WORDS_ASK"));
static NO_FIXED_TIME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NO_FIXED_TIME"));
static NO_JOB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NO_JOB"));
static OPTIONS_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OPTIONS_ASK"));
static OUTCOME_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OUTCOME_ASK"));
static PAST_DEFINES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PAST_DEFINES"));
static PATH_THEN_VERB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PATH_THEN_VERB"));
static PERMISSION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PERMISSION"));
static PERMISSION_RULE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PERMISSION_RULE"));
static PERSON_FULL_NAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PERSON_FULL_NAME"));
static PERSON_NAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PERSON_NAME"));
static PERSON_NAMED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PERSON_NAMED"));
static PERSON_RELATION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PERSON_RELATION"));
static PERSON_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PERSON_WORD"));
static PICKS_BY_COMPARISON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PICKS_BY_COMPARISON"));
static PICK_BY_CASE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PICK_BY_CASE"));
static PLACE_AS_EVIDENCE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PLACE_AS_EVIDENCE"));
static PLACE_AT_HOME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PLACE_AT_HOME"));
static PLACE_KIND: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PLACE_KIND"));
static PLACE_NAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PLACE_NAME"));
static PLACE_NAME_ANY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PLACE_NAME_ANY"));
static PLACE_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PLACE_WORD"));
static PLAN_VERB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PLAN_VERB"));
static POINTS_BACK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "POINTS_BACK"));
static PREFERS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PREFERS"));
static PRICE_ASKED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PRICE_ASKED"));
static PRICE_GIVEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PRICE_GIVEN"));
static PUNT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PUNT"));
static PURPOSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PURPOSE"));
static PURPOSE_TO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PURPOSE_TO"));
static QUALITY_IS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "QUALITY_IS"));
static QUOTED_NAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "QUOTED_NAME"));
static RATE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RATE"));
static RATE_OF_A_COUNT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RATE_OF_A_COUNT"));
static RATE_PHRASE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RATE_PHRASE"));
static REASON_OPENS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "REASON_OPENS"));
static REFERS_BACK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "REFERS_BACK"));
static REPEAT_LEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "REPEAT_LEAD"));
static RULE_CONDITION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RULE_CONDITION"));
static RULE_DROP_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RULE_DROP_IT"));
static RULE_GO_NEW: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RULE_GO_NEW"));
static RULE_KEEP_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "RULE_KEEP_IT"));
static SAID_WORDS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SAID_WORDS"));
static SAYS_NO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SAYS_NO"));
static SAYS_NOTHING_ABOUT_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SAYS_NOTHING_ABOUT_IT"));
static SAYS_WHAT_IT_DOES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SAYS_WHAT_IT_DOES"));
static SEASON_WHEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SEASON_WHEN"));
static SEEKS_HELP: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SEEKS_HELP"));
static SENDS_THEM_ON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SENDS_THEM_ON"));
static SENTENCE_END: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SENTENCE_END"));
static SEQUENCES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SEQUENCES"));
static SETTLED_ON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SETTLED_ON"));
static SET_LABEL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SET_LABEL"));
static SIDES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SIDES"));
static SIDES_VS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SIDES_VS"));
static SIDE_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SIDE_FILLER"));
static SIGN_OFF_LEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SIGN_OFF_LEAD"));
static SOFT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SOFT"));
static SOURCE_COLON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SOURCE_COLON"));
static SOURCE_DOES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SOURCE_DOES"));
static SOURCE_LIST: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SOURCE_LIST"));
static SOURCE_MADE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SOURCE_MADE"));
static SOURCE_MAKES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SOURCE_MAKES"));
static SPOT_OF_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SPOT_OF_THE_THING"));
static SPOT_ON_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SPOT_ON_THE_THING"));
static SPOT_PART: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SPOT_PART"));
static STAGE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STAGE"));
static STAGE_WHEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STAGE_WHEN"));
static STATES_HEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STATES_HEAD"));
static STEPS_INSIDE_A_LIST: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "STEPS_INSIDE_A_LIST"));
static TELLS_A_TEST: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TELLS_A_TEST"));
static TELL_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TELL_ASK"));
static TEST_IS_THE_VERDICT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TEST_IS_THE_VERDICT"));
static THEIR_OWN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THEIR_OWN"));
static THE_SIGN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THE_SIGN"));
static THE_THING_ASKED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THE_THING_ASKED"));
static THE_WINNER_NAMED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THE_WINNER_NAMED"));
static THINGS_VERB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THINGS_VERB"));
static TIME_LEAD_IN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIME_LEAD_IN"));
static TRAILING_VERDICT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TRAILING_VERDICT"));
static UNSETTLED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "UNSETTLED"));
static USUALLY_IS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "USUALLY_IS"));
static VERDICT_ABOUT_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_ABOUT_IT"));
static VERDICT_AFTER_A_CASE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_AFTER_A_CASE"));
static VERDICT_AFTER_A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_AFTER_A_FIGURE"));
static VERDICT_BAND: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_BAND"));
static VERDICT_ECHO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_ECHO"));
static VERDICT_INSIDE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_INSIDE"));
static VERDICT_INSIDE_QUALITY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_INSIDE_QUALITY"));
static VERDICT_ON_A_SCALE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_ON_A_SCALE"));
static VERDICT_OPEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_OPEN"));
static VERDICT_RULE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "VERDICT_RULE"));
static WANTS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "WANTS"));
static WANTS_INFO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WANTS_INFO"));
static WANTS_OF_THE_PERSON: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WANTS_OF_THE_PERSON"));
static WANTS_THE_CAUSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WANTS_THE_CAUSE"));
static WORDS_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WORDS_ASK"));
static WOULD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WOULD"));
static WRITES_IN_A_DOCUMENT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WRITES_IN_A_DOCUMENT"));
static WRITES_TO_A_MESSAGE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WRITES_TO_A_MESSAGE"));
static _ADVISES_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ADVISES_THE_THING"));
static _AN_AGE_BARE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_AN_AGE_BARE"));
static _AN_AGE_BIRTHDAY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_AN_AGE_BIRTHDAY"));
static _AN_AGE_MONTHS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_AN_AGE_MONTHS"));
static _AN_AGE_NAMED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_AN_AGE_NAMED"));
static _AN_AGE_SPELLED_OPENS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_AN_AGE_SPELLED_OPENS"));
static _AN_AGE_YEARS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_AN_AGE_YEARS"));
static _ARTICLE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_ARTICLE"));
static _A_DOING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_DOING"));
static _A_FIGURE_AGAINST_THE_STANDARD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_FIGURE_AGAINST_THE_STANDARD"));
static _A_QUOTE: LazyLock<&'static str> = LazyLock::new(|| t::string(M, "_A_QUOTE"));
static _A_SOCIAL_OPEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_SOCIAL_OPEN"));
static _COPULA: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_COPULA"));
static _EMPTY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_EMPTY"));
static _ENDS_THE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_ENDS_THE_THING"));
static _FIGURES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_FIGURES"));
static _FIGURE_HEAD: LazyLock<usize> = LazyLock::new(|| t::int(M, "_FIGURE_HEAD") as usize);
static _JUDGES_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_JUDGES_THE_THING"));
static _LIST_OF_THINGS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_LIST_OF_THINGS"));
static _MEASURE_AFTER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_MEASURE_AFTER"));
static _NAMING_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_NAMING_WORD"));
static _NOT_A_ROLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_NOT_A_ROLE"));
static _NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_NOT_A_THING"));
static _NOT_A_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_NOT_A_WORD"));
static _NOT_THE_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_NOT_THE_SUBJECT"));
static _NO_WAY_TO_KNOW: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_NO_WAY_TO_KNOW"));
static _OPINION_OF_THE_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_OPINION_OF_THE_THING"));
static _PARTY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_PARTY"));
static _PREFERS_STATED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_PREFERS_STATED"));
static _PRONOUN_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_PRONOUN_SUBJECT"));
static _PUTS_THE_THING_TO_USE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_PUTS_THE_THING_TO_USE"));
static _QUALITIES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "_QUALITIES"));
static _QUALITY_OF_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_QUALITY_OF_A_THING"));
static _SAYS_WHAT_IT_IS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_SAYS_WHAT_IT_IS"));
static _SIDE_EMPTY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_SIDE_EMPTY"));
static _THE_VERDICT_IS_EARLY: LazyLock<usize> = LazyLock::new(|| t::int(M, "_THE_VERDICT_IS_EARLY") as usize);
static _WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_WORD"));
static _WORKING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_WORKING"));
/// `_QUALITY_SAID`: {quality: how it is said}.
static _QUALITY_SAID: LazyLock<IndexMap<String, &'static Re>> = LazyLock::new(|| t::map_re(M, "_QUALITY_SAID"));

// =============================================================================================
// fit.py, the reply side: whether a reply gives the shape the line wants (`gives`) and every
// reader it reaches. Python defines `gives`, `_steps`, `_steps_in` and `_an_instruction` twice;
// the later definition is the module's (the earlier `gives`, `_steps` and `_steps_in` live on
// as `_gives_as_read`, `_steps_by_the_verb_list` and `_STEPS_IN`; the earlier
// `_an_instruction` is shadowed and never reached).

/// `text[:-k]` over characters ("" when the text is shorter).
fn cut(word: &str, k: usize) -> &str {
    let n = word.chars().count();
    if n <= k {
        return "";
    }
    let end = word.char_indices().nth(n - k).map(|(i, _)| i).unwrap_or(word.len());
    &word[..end]
}

/// `re.split('[,;:]', text, maxsplit=1)[0]`.
fn first_clause(s: &str) -> &str {
    match s.find([',', ';', ':']) {
        Some(at) => &s[..at],
        None => s,
    }
}

/// `" ".join(str(x).split())`.
fn squeeze(s: &str) -> String {
    text::split(s).join(" ")
}

/// `_the_rule_settles_it(opening, line, reply=None)`: the reply's rule settles the ask.
pub fn _the_rule_settles_it(opening: &str, _line: &str, reply: &str) -> bool {
    let said = squeeze(if reply.is_empty() { opening } else { reply });
    if said.contains('?') {
        return false;
    }
    if AN_OFFER_NOT_A_TEST.is_match(opening) {
        return false;
    }
    let Some(found) = A_RULE_THAT_SETTLES_IT.search(opening) else { return false };
    let word = found.as_str().to_lowercase();
    if matches!(word.as_str(), "once" | "until" | "unless") && rx("[:;]").is_match(&opening[..found.start()]) {
        return false;
    }
    if text::strip(&opening[..found.start()]).is_empty() && matches!(word.as_str(), "if" | "when" | "whenever" | "whether") {
        return false;
    }
    A_STANDING_AFTER_THE_TEST.is_match(&opening[found.end()..])
}

/// `wanted(memory)`: the shape this turn's line wants, as the reading or the intent observer
/// left it.
pub fn wanted(memory: &crate::state::Memory) -> Option<String> {
    if let Some(read) = &memory.understood {
        if let Some(want) = &read.wants {
            if WANTS.contains(want) {
                return Some(want.clone());
            }
        }
    }
    memory.intent_wants.clone().filter(|w| WANTS.contains(w))
}

/// `_an_amount_with_a_unit(opening)`.
pub fn _an_amount_with_a_unit(opening: &str) -> bool {
    _an_amount(opening) && A_UNIT.is_match(opening)
}

/// `_about_the_thing_the_line_named(opening, said)`: the reply's head names a word of the line.
pub fn _about_the_thing_the_line_named(opening: &str, said: &str) -> bool {
    let words = _words(opening);
    let wide = POINTS_BACK.match_(opening).is_some() || _an_instruction(opening);
    let head: HashSet<&String> = if wide { words.iter().collect() } else { words.iter().take(5).collect() };
    let said_words: HashSet<String> = _words(said).into_iter().collect();
    for word in &said_words {
        if text::len(word) < 3 || _EMPTY.contains(word) || NOT_A_THING.contains(word) {
            continue;
        }
        if head.contains(word) {
            return true;
        }
        for stem in [cut(word, 1), cut(word, 2)] {
            if text::len(stem) < 3 || _EMPTY.contains(stem) || NOT_A_THING.contains(stem) {
                continue;
            }
            if head.iter().any(|h| h.as_str() == stem) {
                return true;
            }
        }
    }
    let digits = |s: &str| -> HashSet<String> { rx("\\d+").findall_str(s).into_iter().collect() };
    !digits(opening).is_disjoint(&digits(said))
}

/// `_do_with_ask(said)`.
pub fn _do_with_ask(said: &str) -> bool {
    ANY_DO_WITH.is_match(said)
}

/// `asks_for_words(understood)`: the line asks for words to say or write.
pub fn asks_for_words(understood: &Understood) -> bool {
    let plain = text::strip(&understood.plain).to_lowercase();
    let said = if plain.is_empty() { _said(understood) } else { plain };
    if said.is_empty() || !WORDS_ASK.is_match(&said) {
        return false;
    }
    if NAMED_WORDS.is_match(&said) {
        return !(TELLS_A_TEST.is_match(&said) || NOT_WORDS_ANYWAY.is_match(&said));
    }
    if understood.subject.as_deref().unwrap_or("").is_empty() && understood.followup {
        return false;
    }
    if WRITES_IN_A_DOCUMENT.is_match(&said) && !(WRITES_TO_A_MESSAGE.is_match(&said) && !A_COVER_LETTER.is_match(&said)) {
        return false;
    }
    !NOT_WORDS_ASK.is_match(&said)
}

/// `_CHECKS[want](opening)`: the reader that says a reply gives the shape.
fn check_for(want: &str, opening: &str) -> Option<bool> {
    Some(match want {
        "time" => _a_time(opening),
        "compare" => _a_comparison(opening),
        "amount" => _an_amount(opening),
        "place" => _a_place(opening),
        "person" => _a_person(opening),
        "meaning" => _a_meaning_or_a_statement(opening),
        "yesno" => _a_yesno(opening, ""),
        "pick" => _a_pick(opening),
        "steps" => _steps(opening, ""),
        "options" => _options(opening),
        "reason" => _a_reason(opening, None),
        _ => return None,
    })
}

/// `_CLEAR`: the shapes a reply clearly gives, each with its reader, in order.
fn clear_says(want: &str, opening: &str) -> bool {
    match want {
        "time" => _a_clear_time_at_the_head(opening),
        "amount" => _an_amount_at_the_head(opening),
        "steps" => _steps(opening, ""),
        "place" => PLACE_AS_EVIDENCE.is_match(opening),
        "person" => PERSON_WORD.is_match(opening),
        "meaning" => MEANS.is_match(opening),
        "yesno" => VERDICT_OPEN.match_(opening).is_some(),
        "pick" => _PREFERS_STATED.is_match(opening),
        "reason" => REASON_OPENS.match_(opening).is_some(),
        _ => false,
    }
}

const CLEAR_ORDER: [&str; 9] = ["time", "amount", "steps", "place", "person", "meaning", "yesno", "pick", "reason"];

/// The first `n` characters of a text (`text[:n]`).
fn chars_head(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

/// `_the_shape_a_reply_gives(reply, want, line=None)`: True when the reply gives the shape the
/// line wants, False when it clearly gives another, None when it cannot tell. `line` is the
/// line's text (a reading is read by its text, clean or plain first); None is no line.
pub fn _the_shape_a_reply_gives(reply: &str, want: &str, line: Option<&str>) -> Option<bool> {
    if !WANTS.iter().any(|w| w == want) || SOFT.contains(want) {
        return None;
    }
    let has_line = line.is_some_and(|l| !l.is_empty());
    let said = || _line_text(line.unwrap_or(""));
    let mut opening = first_sentence(reply);
    if opening.is_empty() {
        return None;
    }
    let mut repeated = false;
    if want == *YESNO {
        if let Some(repeat) = REPEAT_LEAD.match_(&opening) {
            let rest = text::strip(&opening[repeat.end()..]).to_string();
            if !rest.is_empty() {
                opening = rest;
                repeated = true;
            }
        }
    }
    let opening = opening.as_str();
    if is_a_punt(opening) {
        let head = chars_head(reply, 400);
        return if DIRECTED.is_match(head) || A_HOLDER.is_match(head) { None } else { Some(false) };
    }
    if want == *YESNO && _a_sign_off(opening) {
        return if _A_SOCIAL_OPEN.match_(&said()).is_some() { None } else { Some(false) };
    }
    if want == *TIME && _a_money_rate(opening) {
        let stripped = _without_a_money_rate(opening);
        if stripped != opening {
            return Some(_a_time(&stripped));
        }
    }
    if want == *STEPS {
        if _steps_in(reply) {
            return Some(true);
        }
        if _imperative(opening) {
            return Some(true);
        }
    } else if want == *TIME && has_line && (STAGE_WHEN.is_match(opening) || SEASON_WHEN.is_match(opening)) && ASKS_WHEN.is_match(&said()) {
        return Some(true);
    } else if want == *REASON {
        if _a_reason(opening, line) {
            return Some(true);
        }
        if has_line && _names(&_opening_two(reply), &said()) && _a_reason(&_opening_two(reply), line) {
            return Some(true);
        }
    } else if want == *YESNO {
        if _the_rule_settles_it(opening, line.unwrap_or(""), reply) {
            return Some(true);
        }
        if _a_yesno(opening, line.unwrap_or("")) {
            return Some(true);
        }
    } else if want == *AMOUNT && has_line && ASKS_AN_AGE.is_match(&said()) {
        if _an_age_is_given(opening) {
            return Some(true);
        }
    } else if check_for(want, opening) == Some(true) {
        if want == *AMOUNT && has_line && _a_price_asked(line.unwrap_or("")) && _a_time(opening) && !PRICE_GIVEN.is_match(opening) {
            return Some(false);
        }
        return Some(true);
    } else if want == *TIME && _a_time_a_sentence_on(reply) {
        return Some(true);
    }
    let l = line.unwrap_or("");
    if want == *YESNO && has_line && ASKS_THE_BOT.is_match(&said()) && I_OFFER.is_match(opening) && _about_the_thing_the_line_named(opening, &said()) {
        return Some(true);
    }
    if want == *YESNO
        && has_line
        && !_an_outcome_of_their_own_doing(&said())
        && _about_the_thing_the_line_named(opening, &said())
        && (_a_meaning(opening) || _a_reason(opening, None) || _a_time(opening) || _an_amount_with_a_unit(opening))
    {
        return Some(true);
    }
    if want == *YESNO && has_line && _the_quality_it_was_asked_about(opening, &said()) {
        return Some(true);
    }
    if want == *YESNO && !repeated && _the_job_answers(opening, l) {
        return Some(true);
    }
    if want == *YESNO && !repeated && has_line && ASKS_IF_SERIOUS.is_match(&said()) && SEEKS_HELP.is_match(reply) {
        return Some(true);
    }
    if want == *YESNO && has_line && _the_thing_is_named_as_what_it_usually_is(opening, l) {
        return Some(true);
    }
    if want == *YESNO && has_line && VERDICT_ABOUT_IT.is_match(opening) && _names_what_the_line_named(opening, &said()) {
        return Some(true);
    }
    if want == *YESNO && VERDICT_RULE.is_match(reply) {
        return Some(true);
    }
    if want == *YESNO && _says_no_about_the_asked_thing(opening, l) {
        return Some(true);
    }
    if want == *YESNO && has_line && _the_side_wins(opening, l) {
        return Some(true);
    }
    if want == *YESNO && _an_echoed_no(opening, l) {
        return Some(true);
    }
    if want == *YESNO && has_line && _the_figure_against_the_standard(opening, &said()) {
        return Some(true);
    }
    if want == *YESNO && _a_figure_then_the_verdict(opening, reply) {
        return Some(true);
    }
    if want == *TIME && has_line && EXTENT.is_match(opening) && LENGTH_ASKED.is_match(&said()) {
        return Some(true);
    }
    if want == *PICK && has_line && _names_a_side(opening, &said()) {
        return Some(true);
    }
    if want == *PICK && has_line && _an_instruction(opening) && _names(opening, l) {
        return Some(true);
    }
    if want == *OPTIONS && has_line && OPTIONS_ASK.is_match(&said()) && (INSTRUCTS.match_(opening).is_some() || JOB.match_(opening).is_some()) {
        return Some(true);
    }
    let both = if want == *COMPARE && has_line { _names_both_sides(reply, &said()) } else { None };
    if both == Some(true) {
        return Some(true);
    }
    if want == *COMPARE && both.is_none() {
        return None;
    }
    if want == *REASON && _head_explains(opening) {
        return Some(true);
    }
    if want == *MEANING && has_line && _defines_the_thing(opening, &said()) {
        return Some(true);
    }
    if want == *MEANING && has_line && _a_past_definition(opening) && _names_what_the_line_named(opening, &said()) {
        return Some(true);
    }
    if want == *MEANING && has_line && (ORIGIN_ASK.is_match(&said()) || MADE_OF.is_match(&said())) && _an_origin(opening) {
        return Some(true);
    }
    if want == *OPTIONS
        && !_imperative(opening)
        && VERDICT_OPEN.match_(opening).is_none()
        && !LIST_AFTER_COLON.is_match(opening)
        && (_a_pick(opening) || CHOICE.is_match(opening) || _marked_things(opening) >= 2)
    {
        return Some(true);
    }
    if want == *OPTIONS && COUNTED_LIST.is_match(opening) && !A_TIME_COUNT.is_match(opening) && !_an_instruction(opening) && _a_list_of_things(opening) {
        return Some(true);
    }
    if want == *STEPS && has_line && _options(opening) && _names_what_the_line_named(opening, &said()) {
        return Some(true);
    }
    if (want == *OPTIONS || want == *STEPS) && has_line && _do_with_ask(&said()) && (_options(opening) || _steps_in(reply) || _an_instruction(opening)) {
        return Some(true);
    }
    if (want == *YESNO || want == *PICK) && _the_rule_names_both_sides(reply) {
        return Some(true);
    }
    if want == *OPTIONS && !_an_instruction(opening) && VERDICT_OPEN.match_(opening).is_none() && _three_names(&_opening_two(reply)) {
        return Some(true);
    }
    if want == *YESNO && _the_same_subject(opening, l) {
        return None;
    }
    for other in CLEAR_ORDER {
        if other == want || (want == *STEPS && _FIGURES.contains(other)) {
            continue;
        }
        if other == *MEANING && (want == *PICK || want == *OPTIONS) {
            continue;
        }
        if other == *MEANING && FIGURE_HEAD.match_(opening).is_some() {
            continue;
        }
        if other == *AMOUNT && want == *YESNO {
            continue;
        }
        if other == *REASON && (want == *STEPS || want == *YESNO) {
            continue;
        }
        if clear_says(other, opening) {
            if is_directed(opening) {
                return None;
            }
            return Some(false);
        }
    }
    None
}

/// `_gives_as_read` (the first `gives`): the shape, but a reply that asks the person back gives
/// nothing.
fn _gives_as_read(reply: &str, want: &str, line: Option<&str>) -> Option<bool> {
    let answer = _the_shape_a_reply_gives(reply, want, line);
    if answer == Some(true) && is_a_question_back(reply, line.unwrap_or("")) {
        return Some(false);
    }
    answer
}

/// `_a_sign_answers(want, line, reply)`: the sign that settles a how-do-I-know or is-it-done ask.
pub fn _a_sign_answers(want: &str, line: &str, reply: &str) -> bool {
    let said = line;
    if want == *STEPS {
        if !TELL_ASK.is_match(said) {
            return false;
        }
    } else if want == *YESNO {
        if !IS_IT_DONE.is_match(said) {
            return false;
        }
    } else {
        return false;
    }
    let opening = first_sentence(reply);
    if opening.is_empty() || is_a_punt(&opening) {
        return false;
    }
    THE_SIGN.is_match(&opening)
}

/// `gives(reply, wants_, line=None)`: `gives`, plus the sign that settles a how-do-I-know or
/// is-it-done ask. (Contract fix: `line` is optional, as in Python - None is not "".)
pub fn gives(reply: &str, wants_: &str, line: Option<&str>) -> Option<bool> {
    let read = _gives_as_read(reply, wants_, line);
    if read == Some(false) && _a_sign_answers(wants_, line.unwrap_or(""), reply) {
        return Some(true);
    }
    read
}

/// `missing(reply, wants_, line=None)`: the wants (one label or a list) the reply clearly does
/// not give.
pub fn missing(reply: &str, wants_: &crate::data::Value, line: Option<&str>) -> crate::data::Value {
    use crate::data::Value;
    let wants: Vec<String> = match wants_ {
        Value::Str(s) => vec![s.to_string()],
        other => other.strs(),
    };
    let gone: Vec<Value> = wants.iter().filter(|w| gives(reply, w, line) == Some(false)).map(|w| Value::from(w.as_str())).collect();
    Value::List(std::sync::Arc::new(gone))
}

/// `second_sentence(reply)`: the sentence after the opening one, or "".
pub fn second_sentence(reply: &str) -> String {
    let said = squeeze(reply);
    let parts = rx("(?<=[.!?])\\s+").split(&said);
    // maxsplit=2: the second piece is whole up to the next break
    if parts.len() > 1 {
        text::strip(&parts[1]).to_string()
    } else {
        String::new()
    }
}

/// `_a_time_a_sentence_on(reply)`: the time is given in the sentence after the opening.
pub fn _a_time_a_sentence_on(reply: &str) -> bool {
    let said = squeeze(reply);
    let opening = first_sentence(&said);
    if opening.is_empty() {
        return false;
    }
    let rest = said.get(opening.len()..).unwrap_or("").trim_start_matches([' ', '.', '!', '?']);
    let following = if rest.is_empty() { String::new() } else { first_sentence(rest) };
    if following.is_empty() {
        return false;
    }
    for pattern in [&*DURATION, &*CLOCK, &*STAGE, &*NAMED_CLOCK] {
        for found in pattern.finditer(&following) {
            let before = text::split(&following[..found.start()]);
            let lead_in = &before[before.len().saturating_sub(4)..];
            if lead_in.iter().any(|w| TIME_LEAD_IN.contains(&rx("[^a-z]").sub("", &w.to_lowercase()))) {
                return true;
            }
        }
    }
    false
}

/// `first_sentence(reply)`: the reply's opening sentence, tidied.
pub fn first_sentence(reply: &str) -> String {
    let said = squeeze(reply);
    if said.is_empty() {
        return String::new();
    }
    for end in SENTENCE_END.finditer(&said) {
        let before = &said[..end.start()];
        if ABBREVIATION_END.is_match(before) {
            continue;
        }
        return text::strip(before).to_string();
    }
    said
}

/// `_opening_two(reply)`: the first two sentences.
pub fn _opening_two(reply: &str) -> String {
    let said = squeeze(reply);
    if said.is_empty() {
        return String::new();
    }
    let mut seen = 0;
    for end in SENTENCE_END.finditer(&said) {
        if ABBREVIATION_END.is_match(&said[..end.start()]) {
            continue;
        }
        seen += 1;
        if seen == 2 {
            return text::strip(&said[..end.start()]).to_string();
        }
    }
    said
}

/// `_three_names(text)`: a clause that lists three short names.
pub fn _three_names(said: &str) -> bool {
    let plain = squeeze(said);
    if plain.is_empty() {
        return false;
    }
    for clause in rx("(?<=[.!?;:])\\s+").split(&plain) {
        if clause.matches(',').count() < 2 {
            continue;
        }
        let opened = rx_flags("^[\\W]*(?:then|next|after that|afterwards|finally|lastly)\\b", crate::py::re::I).sub("", &clause);
        if STEPS_INSIDE_A_LIST.is_match(&opened) {
            continue;
        }
        for run in NAME_STOP.split(&opened) {
            let items: Vec<String> = rx("\\s*,\\s*|\\s+(?:and|or|plus)\\s+").split(&run).iter().map(|i| text::strip(i).to_string()).collect();
            let items: Vec<String> = items.into_iter().filter(|i| !i.is_empty()).map(|i| rx("^(?:and|or|plus)\\s+").sub("", &i)).filter(|i| !i.is_empty()).collect();
            if run.contains('(') {
                continue;
            }
            let short: Vec<&String> = items.iter().filter(|i| text::split(i).len() <= 4).collect();
            let trailing = items.len() - short.len() == 1 && text::split(&items[items.len() - 1]).len() > 4;
            if short.len() >= 3 && (short.len() == items.len() || trailing) {
                return true;
            }
        }
    }
    false
}

/// `_a_figure(opening)`.
pub fn _a_figure(opening: &str) -> bool {
    FIGURE.is_match(opening)
}

/// `_a_time(opening)`: the reply gives a time, a length of time or a date.
pub fn _a_time(opening: &str) -> bool {
    if NAMED_CLOCK.is_match(opening) || EDGED_PART_OF_DAY.is_match(opening) {
        return true;
    }
    let mut said = if RATE_PHRASE.match_(opening).is_some() { RATE_PHRASE.subn("", opening, 1) } else { opening.to_string() };
    if RATE_OF_A_COUNT.is_match(&said) {
        said = RATE_OF_A_COUNT.sub(" ", &said);
    }
    if NO_FIXED_TIME.match_(&said).is_some() {
        return true;
    }
    if DURATION.is_match(&said)
        || CLOCK.is_match(&said)
        || BARE_RANGE.match_(&said).is_some()
        || STAGE.is_match(&said)
        || NEVER.is_match(&said)
        || NO_FIXED_TIME.match_(&said).is_some()
        || MINUTES_AS_A_LENGTH.is_match(&said)
    {
        return true;
    }
    if INSTRUCTS.match_(opening).is_some() {
        return false;
    }
    let first = first_clause(&said);
    NAMED_TIME.is_match(first) || DAY_NAME.is_match(first) || DATE.is_match(first)
}

/// `_a_clear_time(opening)`.
pub fn _a_clear_time(opening: &str) -> bool {
    DURATION.is_match(opening) || CLOCK_SURE.is_match(opening) || BARE_RANGE.match_(opening).is_some()
}

/// `_an_age_is_given(opening)`.
pub fn _an_age_is_given(opening: &str) -> bool {
    _AN_AGE_YEARS.is_match(opening)
        || _AN_AGE_MONTHS.is_match(opening)
        || _AN_AGE_BIRTHDAY.is_match(opening)
        || _AN_AGE_NAMED.is_match(opening)
        || _AN_AGE_BARE.is_match(opening)
        || _AN_AGE_SPELLED_OPENS.match_(opening).is_some()
}

/// `_an_amount(opening)`.
pub fn _an_amount(opening: &str) -> bool {
    FIGURE.is_match(opening) || MONEY.is_match(opening) || RATE.is_match(opening)
}

/// `_a_price_asked(line)`.
pub fn _a_price_asked(line: &str) -> bool {
    PRICE_ASKED.is_match(&_line_text(line))
}

/// `_a_figure_that_counts(opening)`.
pub fn _a_figure_that_counts(opening: &str) -> bool {
    _an_amount_to_read(opening) || A_BARE_NUMBER.match_(text::strip(opening)).is_some()
}

/// `_an_amount_to_read(opening)`.
pub fn _an_amount_to_read(opening: &str) -> bool {
    if rx("\\d").is_match(opening) || MONEY.is_match(opening) || RATE.is_match(opening) {
        return true;
    }
    MEASURED.is_match(opening)
}

/// `_figure_at_the_head(opening, pattern)`: the pattern is found within the first few words.
pub fn _figure_at_the_head(opening: &str, pattern: &Re) -> bool {
    match pattern.search(opening) {
        Some(m) => text::split(&opening[..m.start()]).len() < *_FIGURE_HEAD,
        None => false,
    }
}

/// `_a_clear_time_at_the_head(opening)`.
pub fn _a_clear_time_at_the_head(opening: &str) -> bool {
    _figure_at_the_head(opening, &DURATION) || _figure_at_the_head(opening, &CLOCK_SURE) || BARE_RANGE.match_(opening).is_some()
}

/// `_an_amount_at_the_head(opening)`.
pub fn _an_amount_at_the_head(opening: &str) -> bool {
    if _figure_at_the_head(opening, &MONEY) || _figure_at_the_head(opening, &RATE) {
        return true;
    }
    let Some(m) = FIGURE.search(opening) else { return false };
    if text::split(&opening[..m.start()]).len() >= *_FIGURE_HEAD {
        return false;
    }
    if m.as_str().chars().next().is_some_and(crate::understanding::pyuni::ch_digit) {
        return true;
    }
    crate::understanding::match_at(&_MEASURE_AFTER, opening, m.end()).is_some()
}

/// `_a_party(opening)`: a role or body the reply sends them to.
pub fn _a_party(opening: &str) -> bool {
    for found in _PARTY.finditer(opening) {
        let last = text::split(found.as_str()).last().map(|w| w.to_lowercase()).unwrap_or_default();
        if !_NOT_A_ROLE.contains(&last) {
            return true;
        }
    }
    false
}

/// `_the_spot_a_thing_has(found)`: the part of the thing a spot is on.
pub fn _the_spot_a_thing_has(found: Option<&crate::py::re::Match>) -> Option<String> {
    let found = found?;
    Some(found.str(1).to_lowercase().trim_matches(['\'', '-']).to_string())
}

/// `_names_a_spot(opening)`.
pub fn _names_a_spot(opening: &str) -> bool {
    if SPOT_OF_THE_THING.is_match(opening) {
        return true;
    }
    for found in SPOT_ON_THE_THING.finditer(opening) {
        if _the_spot_a_thing_has(Some(&found)).is_some_and(|s| SPOT_PART.contains(&s)) {
            return true;
        }
    }
    false
}

/// `_a_place(opening)`.
pub fn _a_place(opening: &str) -> bool {
    let opening = CALENDAR_WORD.sub(" ", opening);
    PLACE_WORD.is_match(&opening)
        || PLACE_NAME.is_match(&opening)
        || PLACE_NAME_ANY.finditer(&opening).len() >= 2
        || SETTLED_ON.is_match(&opening)
        || _names_a_spot(&opening)
        || PLACE_KIND.is_match(&opening)
        || PLACE_AT_HOME.is_match(&opening)
}

/// `_a_person(opening)`.
pub fn _a_person(opening: &str) -> bool {
    let opening = CALENDAR_WORD.sub(" ", opening);
    if PERSON_RELATION.is_match(&opening) {
        return true;
    }
    if _a_party(&opening) || PERSON_NAMED.is_match(&opening) || PERSON_FULL_NAME.match_(&opening).is_some() {
        return true;
    }
    PERSON_WORD.is_match(&opening) || PERSON_NAME.is_match(&opening)
}

/// `_an_origin(opening)`.
pub fn _an_origin(opening: &str) -> bool {
    SOURCE_MAKES.match_(opening).is_some()
        || SOURCE_COLON.match_(opening).is_some()
        || SOURCE_MADE.match_(opening).is_some()
        || SOURCE_DOES.match_(opening).is_some()
        || SOURCE_LIST.match_(opening).is_some()
}

/// `_a_past_definition(opening)`.
pub fn _a_past_definition(opening: &str) -> bool {
    let Some(found) = PAST_DEFINES.search(opening) else { return false };
    let pieces = rx("[.;:!?,()\\u2014]").split(&opening[..found.start()]);
    let subject = text::strip(pieces.last().map(|s| s.as_str()).unwrap_or("")).to_string();
    if NOT_A_SUBJECT.is_match(&subject) {
        return false;
    }
    rx_flags("(?:my|your|his|her|its|our|their|mine|yours|theirs)\\b", crate::py::re::I).match_(&subject).is_none()
}

/// `_a_meaning(opening)`.
pub fn _a_meaning(opening: &str) -> bool {
    if _OPINION_OF_THE_THING.match_(text::strip(opening)).is_some() {
        return false;
    }
    if MECHANISM.is_match(opening) {
        return true;
    }
    if MEANS_BY_DOING.match_(opening).is_some() || MEANS_INFINITIVE.match_(opening).is_some() {
        return true;
    }
    if DEFINES.match_(opening).is_some() {
        return true;
    }
    let Some(hit) = MEANS.search(opening) else { return false };
    !(MEANS_ARTICLE.match_(hit.as_str()).is_some() && A_JUDGEMENT.match_(opening).is_some())
}

/// `_the_quality_is_about_the_thing(clause, line)`.
pub fn _the_quality_is_about_the_thing(clause: &str, line: &str) -> bool {
    let Some(found) = QUALITY_IS.search(clause) else { return false };
    let subject = text::strip(&clause[..found.start()]);
    if subject.is_empty() {
        return true;
    }
    let words = _words(subject);
    if words.last().is_some_and(|w| _PRONOUN_SUBJECT.contains(w)) {
        return true;
    }
    let said = _line_text(line);
    if said.is_empty() {
        return true;
    }
    let named: HashSet<&String> = words.iter().filter(|w| text::len(w) >= 4 && !_EMPTY.contains(*w)).collect();
    named.is_empty() || named.iter().any(|w| said.contains(w.as_str()))
}

/// `_the_tail_says_what_it_is_about(clause, line)`.
pub fn _the_tail_says_what_it_is_about(clause: &str, line: &str) -> bool {
    let Some(found) = VERDICT_INSIDE.search(clause) else { return false };
    let subject = text::strip(&clause[..found.start()]);
    if subject.is_empty() {
        return false;
    }
    let words = _words(subject);
    if words.last().is_some_and(|w| _PRONOUN_SUBJECT.contains(w)) {
        return true;
    }
    let said = _line_text(line);
    if said.is_empty() {
        return true;
    }
    let named: HashSet<&String> = words.iter().filter(|w| text::len(w) >= 3 && !_EMPTY.contains(*w)).collect();
    !named.is_empty() && named.iter().any(|w| _the_same_word(w, &said))
}

/// `_the_quality_it_was_asked_about(opening, said)`: "is it normal" answered with "it is normal".
pub fn _the_quality_it_was_asked_about(opening: &str, said: &str) -> bool {
    let quality_words: HashSet<String> = _QUALITIES.iter().flat_map(|q| text::split(q)).collect();
    let named: HashSet<String> = _words(said)
        .into_iter()
        .filter(|w| text::len(w) >= 4 && !_EMPTY.contains(w) && !NOT_A_THING.contains(w) && !quality_words.contains(w))
        .collect();
    let early = |s: &str| _words(s).into_iter().take(*_THE_VERDICT_IS_EARLY).collect::<Vec<_>>().join(" ");
    if named.is_empty() && !said.is_empty() && ASKS_ABOUT_A_QUALITY.is_match(said) {
        let asked = early(opening);
        return _QUALITIES.iter().any(|q| _QUALITY_SAID.get(q).is_some_and(|r| r.is_match(&asked)));
    }
    if said.is_empty() || !ASKS_ABOUT_A_QUALITY.is_match(said) {
        return false;
    }
    let head = early(opening);
    for quality in _QUALITIES.iter() {
        let Some(pattern) = _QUALITY_SAID.get(quality) else { continue };
        let Some(found) = pattern.search(&head) else { continue };
        if !pattern.is_match(said) {
            continue;
        }
        if _QUALITY_OF_A_THING.contains(quality) && !_the_quality_is_said_of_the_thing(&head, found.start() as i64, said) {
            continue;
        }
        return true;
    }
    false
}

/// `_the_quality_is_said_of_the_thing(head, at, said)`: the quality at `at` (a byte offset in
/// `head`, where Python has the match's start) is said of the line's thing.
pub fn _the_quality_is_said_of_the_thing(head: &str, at: i64, said: &str) -> bool {
    let words = text::split(&head[..at as usize]);
    let Some(last) = words.last() else { return true };
    if !_COPULA.contains(last) {
        return true;
    }
    let subject: Vec<&String> = words[..words.len() - 1].iter().filter(|w| crate::understanding::pyuni::is_alpha(w)).collect();
    let Some(last_subject) = subject.last() else { return true };
    if _PRONOUN_SUBJECT.contains(*last_subject) {
        return !said.is_empty() && _about_the_thing_the_line_named(head, said);
    }
    if said.is_empty() {
        return true;
    }
    subject[subject.len().saturating_sub(2)..].iter().any(|w| said.contains(w.as_str()))
}

/// `_the_verdict_is_inside(clause, line)`.
pub fn _the_verdict_is_inside(clause: &str, line: &str) -> bool {
    let Some(found) = VERDICT_INSIDE_QUALITY.search(clause) else { return VERDICT_INSIDE.is_match(clause) };
    let q = found.name_str("q");
    let at = clause[found.start()..].find(q).map(|i| i + found.start()).expect("ValueError: substring not found");
    if _the_quality_is_said_of_the_thing(clause, at as i64, line) {
        return true;
    }
    let rest = format!("{} {}", &clause[..found.start()], &clause[found.end()..]);
    VERDICT_INSIDE.is_match(&rest)
}

/// `_the_quality_answers(clause, line)`.
pub fn _the_quality_answers(clause: &str, line: &str) -> bool {
    if _the_quality_is_about_the_thing(clause, line) {
        return true;
    }
    IT_HELPS.is_match(clause) && _WORKING.is_match(&_line_text(line))
}

/// `is_a_punt(opening)`: the reply hands the question on.
pub fn is_a_punt(opening: &str) -> bool {
    PUNT.match_(opening).is_some() || HANDOFF.match_(opening).is_some() || REFERS_BACK.match_(opening).is_some()
}

/// `is_directed(opening)`.
pub fn is_directed(opening: &str) -> bool {
    DIRECTED.is_match(opening)
}

/// `_the_first_sentence(reply)`: up to the first stop outside quotes.
pub fn _the_first_sentence(reply: &str) -> String {
    let mut quoted = false;
    for (index, ch) in reply.char_indices() {
        if _A_QUOTE.contains(ch) {
            quoted = !quoted;
        } else if !quoted && ".!?".contains(ch) {
            let end = index + ch.len_utf8();
            let after = reply[end..].chars().next();
            if after.is_none_or(crate::py::text::is_space) {
                return reply[..end].to_string();
            }
        }
    }
    reply.to_string()
}

/// `_the_echo_question(opening, line)`: the reply only asks the line's own words back.
pub fn _the_echo_question(opening: &str, line: &str) -> bool {
    let said = _WORD.findall_str(&line.to_lowercase());
    if said.is_empty() {
        return false;
    }
    let mine: Vec<String> = _WORD.findall_str(&opening.to_lowercase()).into_iter().filter(|w| text::len(w) > 3 && !_NOT_A_WORD.contains(w)).collect();
    !mine.is_empty() && mine.iter().all(|w| said.contains(w))
}

/// `is_a_question_back(reply, line=None)`: the reply opens by asking the person.
pub fn is_a_question_back(reply: &str, line: &str) -> bool {
    let body = text::strip(reply);
    if body.is_empty() {
        return false;
    }
    let opening = _the_first_sentence(body);
    if !text::rstrip(&opening).ends_with('?') {
        return false;
    }
    if !_the_echo_question(&opening, line) {
        return true;
    }
    let rest = text::strip(&body[opening.len()..]);
    if rest.is_empty() {
        return true;
    }
    is_a_punt(&first_sentence(rest)) || _NO_WAY_TO_KNOW.is_match(rest)
}

/// `_an_outcome_of_their_own_doing(line)`.
pub fn _an_outcome_of_their_own_doing(line: &str) -> bool {
    HYPOTHETICAL.is_match(line) && OUTCOME_ASK.is_match(line)
}

/// `_names_the_lines_thing(opening, said)`.
pub fn _names_the_lines_thing(opening: &str, said: &str) -> bool {
    let reply: HashSet<String> = _words(opening).into_iter().collect();
    let said_words: HashSet<String> = _words(said).into_iter().collect();
    for word in &said_words {
        if text::len(word) < 3 || _EMPTY.contains(word) || NOT_A_THING.contains(word) {
            continue;
        }
        if reply.contains(word) || (text::len(word) > 4 && reply.contains(cut(word, 1))) {
            return true;
        }
    }
    false
}

/// `_says_no_about_the_asked_thing(opening, line)`.
pub fn _says_no_about_the_asked_thing(opening: &str, line: &str) -> bool {
    let said = _line_text(line);
    if said.is_empty() {
        return false;
    }
    if opening.trim_end_matches(['"', '\u{201d}', ' ']).ends_with('?') {
        return false;
    }
    for m in SAYS_NO.finditer(opening) {
        if _names_the_lines_thing(m.name_str("subject"), &said) {
            return true;
        }
    }
    let asked: HashSet<String> = _words(&said).into_iter().collect();
    for m in NO_JOB.finditer(opening) {
        let verb = m.str(1);
        if asked.contains(verb) || (text::len(verb) > 4 && asked.contains(cut(verb, 1))) {
            return true;
        }
    }
    false
}

/// `_the_job_is_split(opening)`.
pub fn _the_job_is_split(opening: &str) -> bool {
    if MUST_BE_A_TRADE.is_match(opening) {
        return true;
    }
    THEIR_OWN.is_match(opening) && A_TRADE.is_match(opening)
}

/// `_the_same_word(word, text)`: the word, or its plural, as a whole word.
pub fn _the_same_word(word: &str, said: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    rx(&format!("\\b{}(?:s|es)?\\b", crate::py::re::escape(word))).is_match(said)
}

/// `_names_what_the_line_named(opening, said)`.
pub fn _names_what_the_line_named(opening: &str, said: &str) -> bool {
    let words: HashSet<String> = _NAMING_WORD.findall_str(&said.to_lowercase()).into_iter().filter(|w| !NAMING_STOP.contains(w)).collect();
    if words.is_empty() {
        return false;
    }
    let first = first_clause(opening).to_lowercase();
    words.iter().any(|w| _the_same_word(w, &first))
}

/// `_the_rule_names_both_sides(reply)`.
pub fn _the_rule_names_both_sides(reply: &str) -> bool {
    if reply.is_empty() || !RULE_CONDITION.is_match(reply) {
        return false;
    }
    let drop = RULE_DROP_IT.is_match(reply) || RULE_GO_NEW.is_match(reply);
    RULE_KEEP_IT.is_match(reply) && drop
}

/// `_a_sign_off(opening)`: the reply opens on a sign-off with no verdict after it.
pub fn _a_sign_off(opening: &str) -> bool {
    let Some(m) = SIGN_OFF_LEAD.match_(opening) else { return false };
    let rest = opening[m.end()..].trim_matches([' ', '\t', '.', ',', ';', ':', '!', '?', '-', '\u{2014}']);
    let rest = rx_flags("^(?:but|and|so|though|however|yet|although|still)\\b[\\s,]*", crate::py::re::I).sub("", rest);
    if rest.is_empty() {
        return true;
    }
    !(_a_yesno(&rest, "")
        || VERDICT_INSIDE.is_match(&rest)
        || VERDICT_ECHO.match_(&rest).is_some()
        || VERDICT_RULE.is_match(&rest)
        || TEST_IS_THE_VERDICT.is_match(&rest)
        || TRAILING_VERDICT.is_match(&rest)
        || PERMISSION_RULE.is_match(&rest))
}

/// `_a_yesno(opening, line=None)`: the reply gives a yes or a no (`line` "" is no line).
pub fn _a_yesno(opening: &str, line: &str) -> bool {
    if is_a_punt(opening) {
        return false;
    }
    if FIRST_PERSON_VERDICT.match_(opening).is_some()
        || VERDICT_OPEN.match_(opening).is_some()
        || ECHO_VERDICT.match_(opening).is_some()
        || VERDICT_AFTER_A_CASE.match_(opening).is_some()
    {
        return true;
    }
    if PERMISSION_RULE.is_match(opening) {
        return true;
    }
    if _the_job_is_split(opening) {
        return true;
    }
    if VERDICT_ECHO.match_(opening).is_some() || VERDICT_RULE.is_match(opening) {
        return true;
    }
    let first = first_clause(opening);
    if _the_verdict_is_inside(first, line)
        || TEST_IS_THE_VERDICT.is_match(opening)
        || TRAILING_VERDICT.is_match(opening)
        || THE_WINNER_NAMED.is_match(first)
        || A_HARM_IS_WHAT_IT_IS.is_match(first)
        || VERDICT_ON_A_SCALE.is_match(first)
    {
        return true;
    }
    if _the_quality_answers(first, line) {
        return true;
    }
    if _a_rule_of_its_use(opening, line) {
        return true;
    }
    let mut tail = "";
    if let Some((_, after)) = opening.split_once(':') {
        tail = after;
    } else if let Some(found) = rx(",?\\s+so\\b").search(opening) {
        tail = &opening[found.end()..];
    }
    let tail = if tail.is_empty() {
        ""
    } else {
        match tail.find([',', ';']) {
            Some(at) => &tail[..at],
            None => tail,
        }
    };
    !tail.is_empty() && (_the_tail_says_what_it_is_about(tail, line) || _the_quality_answers(tail, line))
}

/// `_a_rule_of_its_use(opening, line)`.
pub fn _a_rule_of_its_use(opening: &str, line: &str) -> bool {
    if A_BOUND_THAT_IS_THE_ANSWER.is_match(opening) {
        return true;
    }
    let Some(found) = A_LIMIT_ON_ITS_USE.search(opening) else { return false };
    let said = _line_text(line);
    if said.is_empty() {
        return true;
    }
    if _names(found.as_str(), &said) || _names(opening, &said) {
        return true;
    }
    let words: HashSet<String> = _words(&said).into_iter().collect();
    !words.iter().any(|w| _long_enough_a_name(w, 4) && !_EMPTY.contains(w))
}

/// `_the_thing_is_named_as_what_it_usually_is(opening, line)`.
pub fn _the_thing_is_named_as_what_it_usually_is(opening: &str, line: &str) -> bool {
    let said = _line_text(line);
    if said.is_empty() {
        return false;
    }
    let Some(found) = USUALLY_IS.search(opening) else { return false };
    let rest = &opening[found.end()..];
    let words: HashSet<String> = rx("[a-z][a-z']*").findall_str(&said).into_iter().collect();
    for word in words {
        if text::len(&word) < 4 || _EMPTY.contains(&word) || NOT_A_THING.contains(&word) {
            continue;
        }
        if rx(&format!("\\b{}", crate::py::re::escape(&word))).is_match(rest) {
            return true;
        }
    }
    false
}

/// `_an_echoed_no(opening, line)`.
pub fn _an_echoed_no(opening: &str, line: &str) -> bool {
    let said = _line_text(line);
    if said.is_empty() || !NEGATED_AUX.is_match(opening) {
        return false;
    }
    let words: HashSet<String> = rx("[a-z][a-z']*").findall_str(&said).into_iter().collect();
    words.iter().any(|w| text::len(w) >= 4 && !_EMPTY.contains(w) && opening.contains(w.as_str()))
}

/// `_the_job_answers(opening, line)`.
pub fn _the_job_answers(opening: &str, line: &str) -> bool {
    if line.is_empty() || _an_outcome_of_their_own_doing(line) {
        return false;
    }
    if JOB.match_(opening).is_some() {
        return true;
    }
    if _imperative(opening) {
        return true;
    }
    _the_job_named(opening, &_line_text(line))
}

/// `_undecorated(head)`.
pub fn _undecorated(head: &str) -> String {
    text::strip(text::strip(head).trim_matches(['*', '_', '"', '\u{201c}', '\u{201d}', '\''])).to_string()
}

/// `_listed_after(label_lead)`.
pub fn _listed_after(label_lead: &str) -> bool {
    if rx_flags("\\bor\\b", crate::py::re::I).is_match(label_lead) || label_lead.contains(';') {
        return true;
    }
    rx(",\\s+\\S").finditer(label_lead).len() >= 2
}

/// `_marked_things(opening)`: how many things are marked out with stars or quotes.
pub fn _marked_things(opening: &str) -> i64 {
    (rx("\\*[^*\\n]{2,60}\\*").finditer(opening).len() + rx("\\u201c[^\\u201d\\n]{2,60}\\u201d|\\B\\\"[^\\\"\\n]{2,60}\\\"").finditer(opening).len()) as i64
}

/// `_a_pick(opening)`: the reply picks one.
pub fn _a_pick(opening: &str) -> bool {
    if PREFERS.is_match(opening) || PICKS_BY_COMPARISON.is_match(opening) || WOULD.is_match(opening) {
        return true;
    }
    if let Some(lead) = LABEL.match_(opening) {
        if !NOT_A_LABEL.is_match(lead.str(1)) && SET_LABEL.match_(text::strip(lead.str(1))).is_none() && _listed_after(lead.str(2)) {
            return true;
        }
    }
    static SUPERLATIVE: LazyLock<&'static Re> = LazyLock::new(|| {
        rx_flags(
            "\\b(?:is|are|was|were) (?:almost |nearly |usually |often |always |generally |probably |still |quite |about )?(?:the )?(?:(?!(?:request|interest|honest|guest|test|rest|nest|west|east|forest|harvest|protest|suggest|arrest|contest|digest|invest|earnest|chest|crest|tempest|midwest|northwest|southwest|northeast|southeast)\\b)[a-z]{3,}est|most [a-z]+|least [a-z]+)\\b|\\bis the (?:cheapest|dearest|worst|easiest|quickest|safest|hardest|cheaper|better)\\b|\\bwins?\\b(?: on| over| for| by)\\b|\\b(?:has|have|holds|gets) (?:the )?(?:most|highest|lowest|fewest|best)\\b",
            crate::py::re::I,
        )
    });
    if SUPERLATIVE.is_match(opening) {
        return true;
    }
    if NAMES_IN_EMPHASIS.match_(opening).is_some() {
        return true;
    }
    if QUOTED_NAME.finditer(opening).len() >= 2 && PICK_BY_CASE.is_match(opening) {
        return true;
    }
    if is_a_punt(opening) || VERDICT_OPEN.match_(opening).is_some() {
        return false;
    }
    let head = NAMED_HEAD.match_(opening);
    if let Some(h) = &head {
        let named = _undecorated(text::strip(h.str(1)));
        if ECHO_HEAD.match_(&named).is_some()
            || BACK_HEAD.match_(&named).is_some()
            || STATES_HEAD.is_match(&named)
            || FRAME_HEAD.match_(&named).is_some()
            || (rx("\\d").is_match(&named) && (_a_time(&named) || _an_amount(&named)))
        {
            return false;
        }
        let bare = _undecorated(h.str(1));
        if INSTRUCTS.match_(&bare).is_none() && ASKS_HEAD.match_(&bare).is_none() && SET_LABEL.match_(text::strip(&bare)).is_none() {
            return true;
        }
    }
    let words = text::split(opening.trim_end_matches(['.', '!']));
    words.len() <= 6
        && BALD.match_(opening).is_some()
        && !BALD_VERB.is_match(opening)
        && INSTRUCTS.match_(opening).is_none()
        && REASON_OPENS.match_(opening).is_none()
        && A_VERB_THEN_THE_THING.match_(opening).is_none()
        && !_a_clear_time(opening)
        && !_a_figure_that_counts(opening)
}

/// `_asks_the_person(said)`: the text asks the person something.
pub fn _asks_the_person(said: &str) -> bool {
    let said = squeeze(said);
    if said.is_empty() {
        return false;
    }
    if WANTS_INFO.match_(&said).is_some() {
        return true;
    }
    let first = match rx("(?<=[.?!])\\s").search(&said) {
        Some(m) => &said[..m.start()],
        None => said.as_str(),
    };
    first.contains('?')
}

/// `_an_instruction(sentence)`: the sentence tells the person to do something that is not
/// fetching things.
pub fn _an_instruction(sentence: &str) -> bool {
    _imperative(sentence) && THINGS_VERB.match_(sentence).is_none()
}

/// `_imperative(sentence)`: shaped like an instruction, a verb and the thing it acts on.
pub fn _imperative(sentence: &str) -> bool {
    let Some(m) = IMPERATIVE.match_(sentence) else { return false };
    let word = m.str(1).to_lowercase();
    let word = word.trim_matches('\'');
    !NOT_A_VERB.contains(word) && !word.ends_with("ing")
}

/// `_opens_on_a_job(sentence)`.
pub fn _opens_on_a_job(sentence: &str) -> bool {
    let Some(m) = JOB.match_(sentence) else { return false };
    let first = first_clause(&sentence[m.end()..]);
    let words: Vec<String> = rx("[A-Za-z']+").findall_str(first).into_iter().take(3).collect();
    if words.is_empty() {
        return false;
    }
    !words.iter().any(|w| SAYS_WHAT_IT_DOES.contains(&w.to_lowercase()))
}

/// `_stem(word)`.
pub fn _stem(word: &str) -> String {
    for end in ["ing", "ed", "es", "s"] {
        if word.ends_with(end) && text::len(word) >= text::len(end) + 4 {
            return word[..word.len() - end.len()].to_string();
        }
    }
    word.to_string()
}

/// `_subject_words(text)`: the stems of the words that can be a subject (a set, sorted here).
pub fn _subject_words(said: &str) -> Vec<String> {
    let mut words: HashSet<String> = HashSet::new();
    for m in rx("[A-Za-z']+").finditer(said) {
        let word = m.as_str();
        let low = word.to_lowercase();
        let low = low.trim_matches('\'');
        if text::len(low) < 4 || _NOT_THE_SUBJECT.contains(low) {
            continue;
        }
        let capital = word.chars().next().is_some_and(|c| crate::understanding::pyuni::ch_upper(c));
        if capital && m.start() > 0 && !said[..m.start()].chars().last().is_some_and(|c| ".!?".contains(c)) {
            continue;
        }
        words.insert(_stem(low));
    }
    let mut out: Vec<String> = words.into_iter().collect();
    out.sort();
    out
}

/// `_the_same_subject(opening, line)`.
pub fn _the_same_subject(opening: &str, line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    let mine: HashSet<String> = _subject_words(opening).into_iter().collect();
    _subject_words(line).iter().any(|w| mine.contains(w))
}

/// `_steps_by_the_verb_list(opening, after="")` (the first `_steps`): the reply is a method.
fn _steps_by_the_verb_list(opening: &str, after: &str) -> bool {
    if DOES_NOT.match_(opening).is_some()
        || DOES.is_match(opening)
        || MEANS_OF.is_match(opening)
        || MEANS_NAMED.match_(opening).is_some()
        || _opens_on_a_job(opening)
        || A_CALCULATION.match_(opening).is_some()
    {
        return true;
    }
    for part in rx("[:;]").split(opening).iter().skip(1) {
        let part = text::strip(part);
        if !part.is_empty() && (INSTRUCTS.match_(part).is_some() || DOES.is_match(part) || _imperative(part)) {
            return true;
        }
    }
    if !after.is_empty() && !_asks_the_person(after) && (INSTRUCTS.match_(after).is_some() || DOES.is_match(after) || _imperative(after)) {
        return true;
    }
    false
}

/// `_instruction_after_a_clause(text)`.
pub fn _instruction_after_a_clause(said: &str) -> bool {
    for m in INSTRUCTS_CLAUSE.finditer(said) {
        let verb_at = m.name_span("verb").map(|s| s.0).unwrap_or(0);
        if _asks_the_person(&said[verb_at..]) {
            continue;
        }
        if m.start() == 0 && !_opens_on_a_job(said) {
            continue;
        }
        return true;
    }
    false
}

/// `_STEPS_IN` (the first `_steps_in`).
fn _steps_in_first(reply: &str) -> bool {
    let said = squeeze(reply);
    if said.is_empty() {
        return false;
    }
    if _steps(&first_sentence(&said), &second_sentence(&said)) {
        return true;
    }
    _instruction_after_a_clause(&said)
}

/// `_a_comparison(opening)`.
pub fn _a_comparison(opening: &str) -> bool {
    CONTRAST.is_match(opening)
}

/// `_a_list_of_things(opening)`.
pub fn _a_list_of_things(opening: &str) -> bool {
    let Some(found) = _LIST_OF_THINGS.search(opening) else { return false };
    let after = found.as_str().split_once(':').map(|(_, a)| a).unwrap_or("");
    let commas = after.matches(',').count();
    commas >= 2 || (commas >= 1 && rx_flags("\\band\\b", crate::py::re::I).is_match(after))
}

/// `_options(opening)`: the reply gives more than one way.
pub fn _options(opening: &str) -> bool {
    if rx_flags("\\b(?:1\\.|1\\)|first option|option one|one idea)\\b", crate::py::re::I).is_match(opening) {
        return true;
    }
    if rx_flags("\\bor\\b", crate::py::re::I).is_match(opening) {
        return true;
    }
    if opening.contains(';') {
        return true;
    }
    if NEXT_IDEA.is_match(opening) {
        return true;
    }
    if FOR_A_LIST.is_match(opening) && !FOR_A_TIME.is_match(opening) && !SEQUENCES.is_match(opening) {
        return true;
    }
    if LIST_AFTER_COLON.is_match(opening) && !SEQUENCES.is_match(opening) {
        return true;
    }
    if LIST_AFTER_COLON.is_match(opening) && _three_things_after_the_colon(opening) {
        return true;
    }
    _two_things_to_do(opening)
}

/// `_two_things_to_do(opening)`.
pub fn _two_things_to_do(opening: &str) -> bool {
    let head = match rx("\\.\\s").search(opening) {
        Some(m) => &opening[..m.start()],
        None => opening,
    };
    let head = head.trim_end_matches('.');
    if SEQUENCES.is_match(head) {
        return false;
    }
    let mut found = 0;
    for part in rx(",?\\s+and\\s+").split(head) {
        let part = text::strip(&part);
        if text::split(part).len() < 2 || !(INSTRUCTS.match_(part).is_some() || PLAN_VERB.match_(part).is_some()) {
            break;
        }
        found += 1;
    }
    found >= 2
}

/// `_three_things_after_the_colon(opening)`.
pub fn _three_things_after_the_colon(opening: &str) -> bool {
    let Some((_, after_raw)) = opening.split_once(':') else { return false };
    let after = text::strip(after_raw);
    if INSTRUCTS.match_(after).is_some() || SEQUENCES.is_match(after_raw) {
        return false;
    }
    rx(",\\s+\\S").finditer(after_raw).len() >= 2
}

/// `_a_reason(opening, line=None)`: the reply gives a reason.
pub fn _a_reason(opening: &str, line: Option<&str>) -> bool {
    if CAUSE.is_match(opening) {
        return true;
    }
    if PURPOSE.is_match(opening) || PURPOSE_TO.match_(opening).is_some() || MERIT.is_match(opening) || (UNSETTLED.is_match(opening) && EVIDENCE_SAYS.is_match(opening)) {
        return true;
    }
    if INSTRUCTS.match_(opening).is_some() || DOES_NOT.match_(opening).is_some() {
        return false;
    }
    if SENDS_THEM_ON.is_match(opening) {
        return false;
    }
    if CAUSE_WHEN.is_match(opening) {
        return true;
    }
    if let Some(l) = line {
        if WANTS_THE_CAUSE.is_match(opening) && WANTS_OF_THE_PERSON.match_(opening).is_none() && _names(opening, l) {
            return true;
        }
    }
    if A_COUNT_OF_THINGS.is_match(opening) {
        return true;
    }
    CAUSE_IS.is_match(opening) && line.is_none_or(|l| _names_the_subject(opening, l))
}

/// `_head_explains(opening)`.
pub fn _head_explains(opening: &str) -> bool {
    let Some((head, _)) = opening.split_once(':') else { return false };
    if SENDS_THEM_ON.is_match(head) {
        return false;
    }
    EXPLAINS.is_match(head) && !_imperative(head)
}

/// `_a_meaning_or_a_statement(opening)`.
pub fn _a_meaning_or_a_statement(opening: &str) -> bool {
    if SAYS_NOTHING_ABOUT_IT.match_(opening).is_some() {
        return false;
    }
    if _a_meaning(opening) {
        return true;
    }
    _a_statement_answers_it(opening)
}

/// `_a_statement_answers_it(opening)`.
pub fn _a_statement_answers_it(opening: &str) -> bool {
    if opening.is_empty() || text::rstrip(opening).ends_with('?') {
        return false;
    }
    if is_a_punt(opening) || NOTHING_TO_GO_ON.match_(opening).is_some() || SAYS_NOTHING_ABOUT_IT.match_(opening).is_some() || _an_opening_job(opening) {
        return false;
    }
    if _an_origin(opening) {
        return false;
    }
    let words = rx("[A-Za-z0-9'\\u2019-]+").findall_str(opening);
    if words.len() <= 10 && _a_figure(opening) && !COPULA.is_match(opening) {
        return false;
    }
    words.len() >= 5
}

/// `_an_opening_job(opening)`.
pub fn _an_opening_job(opening: &str) -> bool {
    if _an_instruction(opening) {
        return true;
    }
    let rest = AN_ORDINAL_OPENING.sub("", opening);
    if rest != opening && _an_instruction(&rest) {
        return true;
    }
    let Some(m) = A_JOB_LANDS.match_(&rest) else { return false };
    let word = m.str(1).to_lowercase();
    let word = word.trim_matches('\'');
    !NOT_A_VERB.contains(word) && !word.ends_with("ing")
}

/// `fits(reply, want, line=None)`: `gives`.
pub fn fits(reply: &str, want: &str, line: Option<&str>) -> Option<bool> {
    gives(reply, want, line)
}

/// `_words(text)`: the lower-cased words of a text.
pub fn _words(said: &str) -> Vec<String> {
    rx("[a-z']+").findall_str(&said.to_lowercase().replace('\u{2019}', "'"))
}

/// `_the_thing_asked(said)`: the thing "what is ..." asks about, up to three words.
pub fn _the_thing_asked(said: &str) -> String {
    let Some(m) = THE_THING_ASKED.match_(said) else { return String::new() };
    let mut words: Vec<String> = Vec::new();
    for word in rx("[a-z0-9']+").findall_str(&m.str(1).to_lowercase().replace('\u{2019}', "'")) {
        if _ENDS_THE_THING.contains(&word) || _EMPTY.contains(&word) {
            break;
        }
        words.push(word);
        if words.len() == 3 {
            break;
        }
    }
    if words.is_empty() || _NOT_A_THING.contains(&words[0]) {
        return String::new();
    }
    if words.iter().map(|w| text::len(w)).max().unwrap_or(0) < 2 {
        return String::new();
    }
    words.join(" ")
}

/// `_uses_the_thing(opening, thing)`.
pub fn _uses_the_thing(opening: &str, thing: &str) -> bool {
    if thing.is_empty() || _PUTS_THE_THING_TO_USE.match_(opening).is_none() {
        return false;
    }
    let low = opening.to_lowercase();
    let Some(at) = low.find(thing) else { return false };
    rx("\\b(?:to|for|so)\\b").is_match(&low[at..])
}

/// `_defines_the_thing(opening, said)`: the reply opens on the thing asked, defined.
pub fn _defines_the_thing(opening: &str, said: &str) -> bool {
    let thing = _the_thing_asked(said);
    if thing.is_empty() {
        return false;
    }
    if _uses_the_thing(opening, &thing) {
        return true;
    }
    let head = _ARTICLE.sub("", text::strip(opening)).to_lowercase();
    if !head.starts_with(&format!("{thing} ")) && !head.starts_with(&format!("{thing}'")) {
        return false;
    }
    let rest = text::strip(&head[thing.len()..]);
    if _JUDGES_THE_THING.match_(rest).is_some() || _ADVISES_THE_THING.match_(rest).is_some() {
        return false;
    }
    if !thing.contains(' ') && !(_SAYS_WHAT_IT_IS.match_(rest).is_some() || _A_DOING.match_(rest).is_some()) {
        return false;
    }
    text::split(rest).len() >= 3
}

/// `_name_words(text)`.
pub fn _name_words(said: &str) -> Vec<String> {
    rx("[a-z0-9']+").findall_str(&said.to_lowercase().replace('\u{2019}', "'"))
}

/// `_long_enough_a_name(word, least=3)`.
pub fn _long_enough_a_name(word: &str, least: i64) -> bool {
    text::len(word) as i64 >= least || word.chars().any(crate::understanding::pyuni::ch_digit)
}

/// `_names(opening, said)`: the reply says a word of the line.
pub fn _names(opening: &str, said: &str) -> bool {
    let reply: HashSet<String> = _words(opening).into_iter().collect();
    let said_words: HashSet<String> = _words(said).into_iter().collect();
    for word in &said_words {
        if !_long_enough_a_name(word, 4) || _EMPTY.contains(word) {
            continue;
        }
        if reply.contains(word) || reply.contains(cut(word, 1)) || reply.contains(cut(word, 2)) {
            return true;
        }
    }
    false
}

/// `_names_the_subject(opening, said)`.
pub fn _names_the_subject(opening: &str, said: &str) -> bool {
    let reply: HashSet<String> = _words(opening).into_iter().collect();
    let said_words: HashSet<String> = _words(said).into_iter().collect();
    for word in &said_words {
        if !_long_enough_a_name(word, 3) || _EMPTY.contains(word) {
            continue;
        }
        if text::len(word) == 3 {
            if reply.contains(word) {
                return true;
            }
            continue;
        }
        if reply.contains(word) || reply.contains(cut(word, 1)) || reply.contains(cut(word, 2)) {
            return true;
        }
        if reply.iter().any(|r| r.starts_with(word.as_str())) {
            return true;
        }
    }
    false
}

/// `_opens_on_the_lines_verb(opening, said)`.
pub fn _opens_on_the_lines_verb(opening: &str, said: &str) -> bool {
    let words = text::split(opening);
    if words.is_empty() || words.len() > 5 {
        return false;
    }
    let first = _words(opening);
    if first.is_empty() || text::len(&first[0]) < 3 {
        return false;
    }
    let stem = &first[0];
    for m in rx_flags("\\b(?:should|must|need to|have to)\\b\\s+(?:i\\s+)?([a-z]{3,})", crate::py::re::I).finditer(said) {
        let verb = m.str(1).to_lowercase();
        if verb == *stem || (verb.starts_with(cut(stem, 1)) && text::len(stem) > 3) {
            return true;
        }
    }
    false
}

/// `_trimmed(words)`: the words without the filler at either end.
pub fn _trimmed(words: &[String]) -> Vec<String> {
    let (mut start, mut end) = (0, words.len());
    while start < end && SIDE_FILLER.contains(&words[start]) {
        start += 1;
    }
    while end > start && SIDE_FILLER.contains(&words[end - 1]) {
        end -= 1;
    }
    words[start..end].to_vec()
}

/// `_the_pair_up(text)`: the two sides a comparison names, as word lists.
pub fn _the_pair_up(said: &str) -> Option<(Vec<String>, Vec<String>)> {
    let clean = squeeze(said);
    if let Some(found) = COMPARATIVE_THAN.search(&clean) {
        let mut left = _trimmed(&_words(found.str(1)));
        let right = _trimmed(&_words(found.str(3)));
        let spent: HashSet<String> = _words(found.str(2)).into_iter().collect();
        while left.last().is_some_and(|w| spent.contains(w)) {
            left.pop();
        }
        if left.is_empty() || right.is_empty() {
            return None;
        }
        let l = left[left.len().saturating_sub(2)..].to_vec();
        let r = right[..right.len().min(2)].to_vec();
        return Some((l, r));
    }
    let found = SIDES.search(&clean).or_else(|| SIDES_VS.search(&clean))?;
    Some((_words(found.str(1)), _words(found.str(2))))
}

/// `_a_side_said_in_the_reply(side, ordered, words)`.
pub fn _a_side_said_in_the_reply(side: &[String], ordered: &[String], words: &[String]) -> bool {
    for word in side {
        if !_long_enough_a_name(word, 3) || _EMPTY.contains(word) {
            continue;
        }
        if _a_side_word_is_named(word, ordered, words) {
            return true;
        }
    }
    false
}

/// `_the_side_wins(opening, line)`: the reply says which side of the line's comparison wins.
pub fn _the_side_wins(opening: &str, line: &str) -> bool {
    let Some(pair) = _the_pair_up(&_line_text(line)) else { return false };
    let sides: Vec<&Vec<String>> = [&pair.0, &pair.1].into_iter().filter(|s| !s.is_empty()).collect();
    if sides.is_empty() {
        return false;
    }
    let ordered = _words(opening);
    let words: Vec<String> = ordered.iter().cloned().collect::<HashSet<_>>().into_iter().collect();
    let named = sides.iter().any(|side| _a_side_said_in_the_reply(side, &ordered, &words));
    if !(named || rx_flags("^both\\b", crate::py::re::I).match_(opening).is_some()) {
        return false;
    }
    A_SIDE_WINS.is_match(opening)
}

/// `_names_a_side(opening, said)`.
pub fn _names_a_side(opening: &str, said: &str) -> bool {
    let Some(sides) = SIDES.search(said) else { return false };
    let reply: Vec<String> = _name_words(opening).into_iter().take(3).collect();
    let side_words: Vec<String> = sides.groups().into_iter().flatten().filter(|s| !s.is_empty()).flat_map(_name_words).collect();
    let set: Vec<String> = reply.iter().cloned().collect::<HashSet<_>>().into_iter().collect();
    _a_side_said_in_the_reply(&side_words, &reply, &set)
}

/// `_a_side_word_is_named(word, ordered, words)`.
pub fn _a_side_word_is_named(word: &str, ordered: &[String], words: &[String]) -> bool {
    for reply_word in words {
        if reply_word == word {
            return true;
        }
        let (shorter, longer) = if text::len(reply_word) <= text::len(word) { (reply_word.as_str(), word) } else { (word, reply_word.as_str()) };
        if text::len(longer) > 4 && longer.starts_with(shorter) && !shorter.ends_with('s') {
            return true;
        }
        if text::len(shorter) >= 3 && shorter.trim_end_matches('s') == longer.trim_end_matches('s') {
            return true;
        }
    }
    for index in 0..ordered.len().saturating_sub(1) {
        if format!("{}{}", ordered[index], ordered[index + 1]) == word {
            return true;
        }
    }
    _initials_named(word, ordered)
}

/// `_initials_named(word, reply_words)`: "bbc" said as "british broadcasting corporation".
pub fn _initials_named(word: &str, reply_words: &[String]) -> bool {
    let w: Vec<char> = word.chars().collect();
    if !(2..=4).contains(&w.len()) || !crate::understanding::pyuni::is_alpha(word) {
        return false;
    }
    for start in 0..reply_words.len() {
        let mut pos = 0;
        for spoken in &reply_words[start..(start + w.len()).min(reply_words.len())] {
            if pos < w.len() && spoken.chars().next() == Some(w[pos]) {
                pos += 1;
            }
        }
        if pos == w.len() {
            return true;
        }
    }
    false
}

/// `_names_both_sides(reply, said)`: True when the reply names both sides of the comparison,
/// False when it misses one it could have named, None when the line names no two sides.
pub fn _names_both_sides(reply: &str, said: &str) -> Option<bool> {
    let said = said.to_lowercase().replace('\u{2019}', "'");
    let mut pairs: Vec<Vec<String>> = Vec::new();
    for pattern in SIDE_PAIRS.iter() {
        if let Some(sides) = pattern.search(&said) {
            pairs.push(vec![sides.get(1).unwrap_or("").to_string(), sides.get(2).unwrap_or("").to_string()]);
            break;
        }
    }
    if pairs.is_empty() {
        if let Some((a, b)) = _the_pair_up(&said) {
            if !a.is_empty() && !b.is_empty() {
                pairs.push(vec![a.join(" "), b.join(" ")]);
            }
        }
    }
    if pairs.is_empty() {
        return None;
    }
    let ordered = _words(reply);
    let words: Vec<String> = ordered.iter().cloned().collect::<HashSet<_>>().into_iter().collect();
    let mut named = 0;
    let mut missing: Vec<String> = Vec::new();
    for side in &pairs[0] {
        let side_words: Vec<String> = {
            let mut s: Vec<String> = _words(side).into_iter().filter(|w| text::len(w) >= 3 && !_SIDE_EMPTY.contains(w)).collect::<HashSet<_>>().into_iter().collect();
            s.sort();
            s
        };
        if side_words.is_empty() {
            continue;
        }
        if side_words.iter().any(|w| _a_side_word_is_named(w, &ordered, &words)) {
            named += 1;
        } else {
            // min(side_words, key=len): only its length is read below
            let shortest = side_words.iter().min_by_key(|w| text::len(w)).unwrap().clone();
            missing.push(shortest);
        }
    }
    if named + missing.len() < 2 {
        return None;
    }
    if named >= 2 {
        return Some(true);
    }
    if missing.iter().any(|w| text::len(w) <= 3) {
        return None;
    }
    Some(false)
}

/// `_the_job_named(opening, said)`: the reply opens on the job the line asks permission for.
pub fn _the_job_named(opening: &str, said: &str) -> bool {
    if !PERMISSION.is_match(said) {
        return false;
    }
    if !_names(opening, said) {
        return _opens_on_the_lines_verb(opening, said);
    }
    if _steps(opening, "") {
        return true;
    }
    let head: Vec<String> = text::split(opening).into_iter().take(12).collect();
    if JOB_ANY.is_match(&head.join(" ")) {
        return true;
    }
    if text::split(opening).len() > 9 {
        return false;
    }
    let first = _words(opening);
    if first.is_empty() || text::len(&first[0]) < 4 {
        return false;
    }
    let stem = &first[0];
    _words(said).iter().any(|w| text::len(w) >= 4 && (w == stem || w.starts_with(stem.as_str()) || stem.starts_with(w.as_str())))
}

/// `_a_step_told_as_what_not_to_do(text)`.
pub fn _a_step_told_as_what_not_to_do(said: &str) -> bool {
    for m in DOES_NOT_ANYWHERE.finditer(said) {
        let after = text::strip(m.name_str("after"));
        if after.is_empty() {
            continue;
        }
        if NOT_A_STEP_BUT_REASSURANCE.match_(after).is_some() {
            continue;
        }
        return true;
    }
    false
}

/// `_steps_in(reply)`: `_STEPS_IN`, and the methods a verb list cannot see.
pub fn _steps_in(reply: &str) -> bool {
    if _steps_in_first(reply) {
        return true;
    }
    let said = squeeze(reply);
    if _a_step_told_as_what_not_to_do(&said) {
        return true;
    }
    for sentence in rx("(?<=[.!?])\\s+").split(&said) {
        if GARDEN.match_(&sentence).is_some() || PATH_THEN_VERB.is_match(&sentence) {
            return true;
        }
    }
    _words_handed_over(&said)
}

/// `_words_handed_over(text)`: the reply opens by handing over the words to say.
pub fn _words_handed_over(said: &str) -> bool {
    SAID_WORDS.match_(said).is_some()
}

/// `_a_money_rate(opening)`.
pub fn _a_money_rate(opening: &str) -> bool {
    MONEY_RATE.is_match(opening)
}

/// `_without_a_money_rate(opening)`.
pub fn _without_a_money_rate(opening: &str) -> String {
    MONEY_RATE.sub(" ", opening)
}

/// `_the_figure_against_the_standard(opening, said)`.
pub fn _the_figure_against_the_standard(opening: &str, said: &str) -> bool {
    if !A_QUANTITY_ASK.is_match(said) {
        return false;
    }
    _A_FIGURE_AGAINST_THE_STANDARD.match_(opening).is_some()
}

/// `_a_figure_then_the_verdict(opening, reply)`.
pub fn _a_figure_then_the_verdict(opening: &str, reply: &str) -> bool {
    if opening.is_empty() || !(_a_time(opening) || _an_amount_with_a_unit(opening)) {
        return false;
    }
    let two = _opening_two(reply);
    let Some(rest) = two.strip_prefix(opening) else { return false };
    let second = text::strip(rest);
    !second.is_empty() && (VERDICT_AFTER_A_FIGURE.is_match(second) || VERDICT_BAND.is_match(second))
}

/// `_opens_on_a_job_to_do(sentence)`: shaped like an instruction, whatever verb it uses.
pub fn _opens_on_a_job_to_do(sentence: &str) -> bool {
    if AN_INSTRUCTION.match_(sentence).is_none() || WANTS_INFO.match_(sentence).is_some() {
        return false;
    }
    if CONTRACTION_OPENER.match_(sentence).is_some() || A_WELL_WISHING.match_(text::strip(sentence)).is_some() {
        return false;
    }
    if A_SUBORDINATE_FRAGMENT.match_(sentence).is_some() && !sentence.contains(',') {
        return false;
    }
    if HALF_A_UNIT.match_(sentence).is_some() {
        return false;
    }
    let word = rx("[A-Za-z']+").match_(sentence).map(|m| m.as_str().to_lowercase()).expect("AttributeError: no word");
    let word = word.trim_matches('\'');
    !NOT_A_VERB.contains(word) && !NOT_AN_OPENER.contains(word) && !word.ends_with("ing")
}

/// `_steps(opening, after="")`: by the verb list, and then by its shape.
pub fn _steps(opening: &str, after: &str) -> bool {
    _steps_by_the_verb_list(opening, after) || _opens_on_a_job_to_do(opening)
}
