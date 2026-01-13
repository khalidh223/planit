use crate::arg::arg_extractor::ArgExtractor;
use crate::arg::arg_matcher::ArgMatcher;
use crate::arg::args::Arg;
use crate::core::context::AppContext;
use crate::core::types::EntityActionType;
use crate::errors::{Error, Result};
use std::fmt::Display;

pub type ArgPattern = Vec<ArgSlot>;

pub struct ArgSlot {
    matcher: fn(&Arg) -> Result<()>,
    validator: Option<Box<dyn Fn(&Arg, &AppContext) -> Result<()>>>,
    optional: bool,
}
#[derive(Debug)]
pub enum SlotMatch {
    Match,                // kind ok (+ validator ok)
    KindMismatch(Error),  // type didn't match
    ValidatorFail(Error), // kind matched, validator failed
}

impl ArgSlot {
    pub fn is_of_arg_type<A: ArgMatcher + ?Sized>() -> Self {
        Self {
            matcher: |a| {
                if A::matches_variant(a) {
                    Ok(())
                } else {
                    Err(A::expected_error(a))
                }
            },
            validator: None,
            optional: false,
        }
    }

    pub fn classify(&self, actual: &Arg, ctx: &AppContext) -> SlotMatch {
        match (self.matcher)(actual) {
            Err(e) => SlotMatch::KindMismatch(e),
            Ok(()) => {
                if let Some(v) = &self.validator {
                    match v(actual, ctx) {
                        Ok(()) => SlotMatch::Match,
                        Err(e) => SlotMatch::ValidatorFail(e),
                    }
                } else {
                    SlotMatch::Match
                }
            }
        }
    }

    #[inline]
    pub fn kind_matches(&self, actual: &Arg) -> bool {
        (self.matcher)(actual).is_ok()
    }

    pub fn with_validator(mut self, v: impl Fn(&Arg) -> Result<()> + 'static) -> Self {
        self.validator = Some(Box::new(move |a, _ctx: &AppContext| v(a)));
        self
    }
    pub fn with_validator_ctx(
        mut self,
        v: impl Fn(&Arg, &AppContext) -> Result<()> + 'static,
    ) -> Self {
        self.validator = Some(Box::new(v));
        self
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    fn is_optional(&self) -> bool {
        self.optional
    }
}

#[derive(Debug)]
pub struct ArgMatchFailure {
    position: usize,
    error: Error,
}

pub trait PatternIdExt {
    fn pattern(&self) -> ArgPattern;
}

pub trait ArgSchema {
    type PatternId: Copy + Eq + PatternIdExt + Display;

    fn patterns_for(&self, action: EntityActionType) -> Vec<Self::PatternId>;
}

pub trait ArgValidator {
    type PatternId: Copy + Eq + PatternIdExt + Display;

    fn validate(
        &self,
        _args: &[Arg],
        _action: EntityActionType,
        _pat_id: Self::PatternId,
    ) -> Result<()> {
        Ok(())
    }
}

pub trait EntityBuilder<E> {
    type PatternId: Copy + Eq + PatternIdExt + Display;
    fn create(&self, args: &[Arg], pat_id: Self::PatternId) -> Result<E>;
    fn modify<'a>(
        &self,
        existing: &'a mut E,
        args: &[Arg],
        pat_id: Self::PatternId,
    ) -> Result<&'a E>;
}

pub trait EntitySpec<E> {
    type PatternId: Copy + Eq + PatternIdExt + Display;

    fn arg_schema(&self) -> &dyn ArgSchema<PatternId = Self::PatternId>;
    fn arg_validator(&self) -> &dyn ArgValidator<PatternId = Self::PatternId>;
    fn entity_builder(&self) -> &dyn EntityBuilder<E, PatternId = Self::PatternId>;

    fn get_mut<'a>(&self, ctx: &'a mut AppContext, id: i32) -> Result<&'a mut E>;

    fn create(&self, ctx: &AppContext, args: &[Arg]) -> Result<E> {
        let pat_id = self.assert_matches_pattern(ctx, args, EntityActionType::Add)?;
        self.arg_validator()
            .validate(args, EntityActionType::Add, pat_id)?;
        self.entity_builder().create(args, pat_id)
    }

    fn modify<'a>(&self, ctx: &'a mut AppContext, args: &[Arg], id: i32) -> Result<&'a E> {
        let pid = {
            let pid = self.assert_matches_pattern(&*ctx, args, EntityActionType::Modify)?;
            self.arg_validator()
                .validate(args, EntityActionType::Modify, pid)?;
            pid
        };

        let existing = self.get_mut(ctx, id)?;
        self.entity_builder().modify(existing, args, pid)
    }

    fn can_delete(&self, ctx: &AppContext, args: &[Arg]) -> Result<()> {
        let pat_id = self.assert_matches_pattern(ctx, args, EntityActionType::Delete)?;
        self.arg_validator()
            .validate(args, EntityActionType::Delete, pat_id)
    }

    fn assert_matches_pattern(
        &self,
        ctx: &AppContext,
        args: &[Arg],
        action: EntityActionType,
    ) -> Result<Self::PatternId> {
        let mut deepest: Option<ArgMatchFailure> = None;
        for pid in self.arg_schema().patterns_for(action) {
            match self.match_pattern(ctx, args, &pid, &action) {
                Ok(()) => return Ok(pid),
                Err(fail) => {
                    if deepest.as_ref().map(|d| d.position).unwrap_or(0) <= fail.position {
                        deepest = Some(fail);
                    }
                }
            }
        }

        if let Some(f) = deepest {
            Err(f.error)
        } else {
            Err(Error::Parse(
                "No matching pattern of arguments found.".into(),
            ))
        }
    }

    fn match_pattern(
        &self,
        ctx: &AppContext,
        args: &[Arg],
        pid: &Self::PatternId,
        action: &EntityActionType,
    ) -> std::result::Result<(), ArgMatchFailure> {
        fn normalize_parse(err: &Error) -> String {
            if let Error::Parse(msg) = err {
                msg.clone()
            } else {
                err.to_string()
            }
        }
        let pattern = pid.pattern();
        let mut args_it = args.iter().peekable();
        let mut last_slot_idx = 0;

        for (slot_idx, slot) in pattern.iter().enumerate() {
            last_slot_idx = slot_idx;

            let Some(arg) = args_it.peek() else {
                if slot.is_optional() {
                    continue;
                }
                return Err(ArgMatchFailure {
                    position: slot_idx,
                    error: Error::Parse(format!("Missing argument(s).\nUsage: {} {}", action, pid)),
                });
            };

            match slot.classify(arg, ctx) {
                SlotMatch::Match => {
                    // consume arg + slot
                    let _ = args_it.next();
                }
                SlotMatch::KindMismatch(e) => {
                    if slot.is_optional() {
                        continue;
                    } else {
                        let msg = normalize_parse(&e);
                        return Err(ArgMatchFailure {
                            position: slot_idx,
                            error: Error::Parse(format!("{msg}.\nUsage: {} {}", action, pid)),
                        });
                    }
                }
                SlotMatch::ValidatorFail(e) => {
                    let msg = normalize_parse(&e);
                    return Err(ArgMatchFailure {
                        position: slot_idx,
                        error: Error::Parse(format!("{msg}.\nUsage: {} {}", action, pid)),
                    });
                }
            }
        }

        if args_it.peek().is_some() {
            return Err(ArgMatchFailure {
                position: last_slot_idx,
                error: Error::Parse(format!(
                    "Too many arguments provided.\nUsage: {} {}",
                    action, pid
                )),
            });
        }

        Ok(())
    }
}

pub struct ColumnIndexer<'a> {
    args: &'a [Arg],
    slots: &'a [ArgSlot],
    arg_idx: usize,
    slot_idx: usize,
}

impl<'a> ColumnIndexer<'a> {
    pub fn new(args: &'a [Arg], pattern: &'a [ArgSlot]) -> Self {
        Self {
            args,
            slots: pattern,
            arg_idx: 0,
            slot_idx: 0,
        }
    }

    pub fn advance(&mut self) -> &mut Self {
        self.advance_once();
        self
    }

    pub fn advance_times(&mut self, n: usize) -> &mut Self {
        for _ in 0..n {
            self.advance_once();
        }
        self
    }

    fn advance_once(&mut self) {
        loop {
            if self.slot_idx >= self.slots.len() {
                break;
            }
            let slot = &self.slots[self.slot_idx];

            if self.arg_idx >= self.args.len() {
                self.slot_idx += 1;
                break;
            }

            let a = &self.args[self.arg_idx];
            if slot.kind_matches(a) {
                self.slot_idx += 1;
                self.arg_idx += 1;
                break;
            } else if slot.is_optional() {
                self.slot_idx += 1;
                continue;
            } else {
                self.slot_idx += 1;
                break;
            }
        }
    }

    pub fn advance_until_arg<E: ArgExtractor<'a>>(&mut self) -> Option<E::Out> {
        loop {
            if self.slot_idx >= self.slots.len() {
                return None;
            }
            if self.arg_idx >= self.args.len() {
                if self.slots[self.slot_idx].is_optional() {
                    self.slot_idx += 1;
                    continue;
                } else {
                    return None;
                }
            }

            if let Some(v) = E::try_extract(&self.args[self.arg_idx]) {
                self.slot_idx += 1;
                self.arg_idx += 1;
                return Some(v);
            }

            if self.slots[self.slot_idx].is_optional() {
                self.slot_idx += 1;
            } else {
                self.slot_idx += 1;
                self.arg_idx += 1;
            }
        }
    }

    pub fn next_opt<E: ArgExtractor<'a>>(&mut self) -> Option<E::Out> {
        loop {
            if self.slot_idx >= self.slots.len() {
                return None;
            }
            let slot = &self.slots[self.slot_idx];

            if self.arg_idx >= self.args.len() {
                if slot.is_optional() {
                    self.slot_idx += 1;
                    continue;
                } else {
                    return None;
                }
            }

            let a = &self.args[self.arg_idx];
            if let Some(v) = E::try_extract(a) {
                self.slot_idx += 1;
                self.arg_idx += 1;
                return Some(v);
            }

            if slot.is_optional() {
                self.slot_idx += 1;
                continue;
            } else {
                return None;
            }
        }
    }

    pub fn next<E: ArgExtractor<'a>>(&mut self) -> E::Out {
        self.next_opt::<E>()
            .expect("pattern matched; required slot must be present")
    }
}
