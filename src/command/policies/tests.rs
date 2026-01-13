use super::flag_policy::{FlagDecision, FlagPolicy, FlagRule, HelpAtIdx};
use crate::arg::args::Arg;
use crate::core::types::Flag;

struct AlwaysError;
impl FlagRule for AlwaysError {
    fn check(&self, _: &[Arg]) -> FlagDecision {
        FlagDecision::Error(crate::errors::Error::parse("boom"))
    }
}

#[test]
fn help_rule_short_circuits_by_position() {
    let policy = FlagPolicy::new(vec![Box::new(HelpAtIdx(0))]);
    let args = vec![Arg::Flag(Flag::Help)];
    match policy.evaluate(&args) {
        FlagDecision::ShortCircuitUsage => {}
        other => panic!("expected usage short-circuit, got {other:?}"),
    }

    let args = vec![Arg::Flag(Flag::Help)];
    assert!(matches!(
        policy.evaluate(&args[1..]),
        FlagDecision::Continue
    ));
}

#[test]
fn policy_defaults_to_continue_when_no_rules_trigger() {
    let policy = FlagPolicy::none();
    let args: Vec<Arg> = vec![];
    assert!(matches!(policy.evaluate(&args), FlagDecision::Continue));
}

#[test]
fn first_rule_wins_in_order() {
    let policy = FlagPolicy::new(vec![Box::new(AlwaysError), Box::new(HelpAtIdx(0))]);
    let args: Vec<Arg> = vec![];
    match policy.evaluate(&args) {
        FlagDecision::Error(_) => {}
        other => panic!("expected error from first rule, got {other:?}"),
    }
}
