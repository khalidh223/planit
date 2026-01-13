use crate::arg::args::Arg;
use crate::core::aliases::{IdLookup, ResolvedId, SourceId};
use crate::core::models::{Card, Event, Task};
use crate::core::types::Bool;
use crate::errors::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmitRefKind {
    Card,
}

pub trait ArgEmitContext {
    fn translate_ref(&self, kind: EmitRefKind, id: SourceId) -> Result<ResolvedId>;
}
pub struct SaveEmitContext<'a> {
    pub id_lookup: &'a IdLookup,
}

impl<'a> ArgEmitContext for SaveEmitContext<'a> {
    fn translate_ref(&self, kind: EmitRefKind, id: SourceId) -> Result<ResolvedId> {
        match kind {
            EmitRefKind::Card => self.id_lookup.get(&id).copied().ok_or_else(|| {
                Error::Parse(format!(
                    "Reference to missing card id {} when building save file.",
                    id
                ))
            }),
        }
    }
}

pub struct NoRefEmitContext;
impl ArgEmitContext for NoRefEmitContext {
    fn translate_ref(&self, _kind: EmitRefKind, id: SourceId) -> Result<ResolvedId> {
        Err(Error::Parse(format!(
            "Unexpected reference mapping for id {}.",
            id
        )))
    }
}

pub trait ArgEmitter<E> {
    fn with_entity(&self, entity: &E, ctx: &dyn ArgEmitContext) -> Result<Vec<Arg>> {
        let mut out = Vec::new();
        self.fill_args(entity, ctx, &mut out)?;
        Ok(out)
    }

    fn fill_args(
        &self,
        entity: &E,
        ctx: &dyn ArgEmitContext,
        out: &mut Vec<Arg>,
    ) -> Result<()>;
}

pub struct CardArgEmitter;
impl CardArgEmitter {
    pub fn new() -> Self {
        Self
    }
}
impl ArgEmitter<Card> for CardArgEmitter {
    fn fill_args(
        &self,
        card: &Card,
        _ctx: &dyn ArgEmitContext,
        out: &mut Vec<Arg>,
    ) -> Result<()> {
        out.push(Arg::Name(card.name.clone()));
        out.push(Arg::CardColor(card.color));
        Ok(())
    }
}

pub struct TaskArgEmitter;
impl TaskArgEmitter {
    pub fn new() -> Self {
        Self
    }
}
impl ArgEmitter<Task> for TaskArgEmitter {
    fn fill_args(
        &self,
        task: &Task,
        ctx: &dyn ArgEmitContext,
        out: &mut Vec<Arg>,
    ) -> Result<()> {
        out.push(Arg::Name(task.name.clone()));
        out.push(Arg::Int(task.hours.round() as i32));
        if let Some(card_id) = task.card_id {
            let mapped = ctx.translate_ref(EmitRefKind::Card, card_id)?;
            out.push(Arg::CardColorId(mapped));
        }
        out.push(Arg::AtSymbol);
        out.push(Arg::Date(task.date.clone()));
        Ok(())
    }
}

pub struct EventArgEmitter;
impl EventArgEmitter {
    pub fn new() -> Self {
        Self
    }
}
impl ArgEmitter<Event> for EventArgEmitter {
    fn fill_args(
        &self,
        event: &Event,
        ctx: &dyn ArgEmitContext,
        out: &mut Vec<Arg>,
    ) -> Result<()> {
        out.push(Arg::Bool(Bool(event.recurring)));
        out.push(Arg::Name(event.name.clone()));
        if let Some(card_id) = event.card_id {
            let mapped = ctx.translate_ref(EmitRefKind::Card, card_id)?;
            out.push(Arg::CardColorId(mapped));
        }
        out.push(Arg::AtSymbol);
        if !event.days.is_empty() {
            out.push(Arg::DaysOfWeek(event.days.clone()));
        }
        out.push(Arg::TimeRange(event.time_range.clone()));
        Ok(())
    }
}
