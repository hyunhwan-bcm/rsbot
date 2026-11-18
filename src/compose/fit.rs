//! Port of `bot/compose/fit.py`: the shape of answer a line wants (`wants` and every function it
//! reaches). The rest of fit.py (how a reply fits the shape) belongs to the reply side.

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
