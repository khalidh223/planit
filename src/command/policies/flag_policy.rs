use crate::arg::args::Arg;
use crate::core::types::Flag;

#[derive(Debug)]
pub enum FlagDecision {
    /// Stop execution and just print usage()
    ShortCircuitUsage,
    /// Stop with a custom message (e.g., version)
    ShortCircuitMsg(String),
    /// Continue command execution
    Continue,
    /// Turn into an error
    Error(crate::errors::Error),
}

pub trait FlagRule {
    fn check(&self, args: &[Arg]) -> FlagDecision;
}

pub struct HelpAtIdx(pub usize);
impl FlagRule for HelpAtIdx {
    fn check(&self, args: &[Arg]) -> FlagDecision {
        match args.get(self.0) {
            Some(Arg::Flag(Flag::Help)) => FlagDecision::ShortCircuitUsage,
            _ => FlagDecision::Continue,
        }
    }
}

pub struct FlagPolicy {
    rules: Vec<Box<dyn FlagRule>>,
}
impl FlagPolicy {
    pub fn new(rules: Vec<Box<dyn FlagRule>>) -> Self {
        Self { rules }
    }
    pub fn none() -> Self {
        Self { rules: vec![] }
    }

    pub fn evaluate(&self, args: &[Arg]) -> FlagDecision {
        // First matching short-circuit wins; otherwise Continue.
        for r in &self.rules {
            match r.check(args) {
                FlagDecision::Continue => continue,
                other => return other,
            }
        }
        FlagDecision::Continue
    }
}
