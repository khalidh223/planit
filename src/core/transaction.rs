use crate::arg::args::Arg;
use crate::command::command_parser::CommandParser;
use crate::core::context::AppContext;
use crate::core::models::{BaseEntity, Card, Event, Task};
use crate::core::repository::{PreparedRepo, Repository};
use crate::errors::{Error, Result};
use std::collections::HashSet;

pub trait ParticipantOps {
    fn begin_stage(&mut self, ctx: &mut AppContext, clear_existing: bool) -> Result<()>;
    fn id_pool(&mut self, ctx: &mut AppContext) -> Result<HashSet<i32>>;
    fn references(&mut self, ctx: &mut AppContext) -> Result<Vec<i32>>;
    fn prepare_commit(&mut self, ctx: &mut AppContext) -> Result<()>;
    fn apply_prepared(&mut self, ctx: &mut AppContext) -> Result<()>;
    fn discard_stage(&mut self, ctx: &mut AppContext);
}

struct RepoParticipant<T: BaseEntity + Clone> {
    accessor: fn(&mut AppContext) -> &mut Repository<T>,
    ref_extractor: Option<fn(&T) -> Option<i32>>,
    prepared: Option<PreparedRepo<T>>,
}

impl<T: BaseEntity + Clone> RepoParticipant<T> {
    fn new(
        accessor: fn(&mut AppContext) -> &mut Repository<T>,
        ref_extractor: Option<fn(&T) -> Option<i32>>,
    ) -> Self {
        Self {
            accessor,
            ref_extractor,
            prepared: None,
        }
    }
}

impl<T: BaseEntity + Clone + 'static> ParticipantOps for RepoParticipant<T> {
    fn begin_stage(&mut self, ctx: &mut AppContext, clear_existing: bool) -> Result<()> {
        (self.accessor)(ctx).begin_stage(clear_existing)
    }

    fn id_pool(&mut self, ctx: &mut AppContext) -> Result<HashSet<i32>> {
        if self.ref_extractor.is_none() {
            (self.accessor)(ctx).staged_effective_ids()
        } else {
            Ok(HashSet::new())
        }
    }

    fn references(&mut self, ctx: &mut AppContext) -> Result<Vec<i32>> {
        let Some(extractor) = self.ref_extractor else {
            return Ok(Vec::new());
        };
        let repo = (self.accessor)(ctx);

        let mut ids = Vec::new();
        if let Some(pending) = repo.staged_pending() {
            for entity in pending {
                if let Some(cid) = extractor(entity) {
                    ids.push(cid);
                }
            }
        }
        Ok(ids)
    }

    fn prepare_commit(&mut self, ctx: &mut AppContext) -> Result<()> {
        let prepared = (self.accessor)(ctx).prepare_commit()?;
        self.prepared = Some(prepared);
        Ok(())
    }

    fn apply_prepared(&mut self, ctx: &mut AppContext) -> Result<()> {
        if let Some(prep) = self.prepared.take() {
            (self.accessor)(ctx).apply_prepared(prep);
            Ok(())
        } else {
            Err(Error::Parse("No prepared state to apply.".into()))
        }
    }

    fn discard_stage(&mut self, ctx: &mut AppContext) {
        (self.accessor)(ctx).discard_stage();
        self.prepared = None;
    }
}

pub struct Transaction {
    participants: Vec<Box<dyn ParticipantOps>>,
}

impl Transaction {
    pub fn new() -> Self {
        let mut participants: Vec<Box<dyn ParticipantOps>> = Vec::new();
        participants.push(Box::new(RepoParticipant::new(AppContext::cards_repo, None)));
        participants.push(Box::new(RepoParticipant::new(
            AppContext::events_repo,
            Some(|e: &Event| e.card_id),
        )));
        participants.push(Box::new(RepoParticipant::new(
            AppContext::tasks_repo,
            Some(|t: &Task| t.card_id),
        )));
        Self { participants }
    }

    pub fn run<F>(&mut self, ctx: &mut AppContext, clear_existing: bool, f: F) -> Result<()>
    where
        F: FnOnce(&mut AppContext) -> Result<()>,
    {
        self.begin_all(ctx, clear_existing)?;
        let outcome = f(ctx);
        match outcome {
            Ok(()) => {
                self.validate_associations(ctx)?;
                self.prepare_all(ctx)?;
                self.apply_all(ctx)?;
                Ok(())
            }
            Err(e) => {
                self.discard_all(ctx);
                Err(e)
            }
        }
    }

    fn begin_all(&mut self, ctx: &mut AppContext, clear_existing: bool) -> Result<()> {
        for p in self.participants.iter_mut() {
            p.begin_stage(ctx, clear_existing)?;
        }
        Ok(())
    }

    fn validate_associations(&mut self, ctx: &mut AppContext) -> Result<()> {
        let mut id_pool = HashSet::new();
        for p in self.participants.iter_mut() {
            id_pool.extend(p.id_pool(ctx)?);
        }

        for p in self.participants.iter_mut() {
            for cid in p.references(ctx)? {
                if !id_pool.contains(&cid) {
                    return Err(Error::Parse(format!(
                        "Referenced id {} not present in transaction.",
                        cid
                    )));
                }
            }
        }
        Ok(())
    }

    fn prepare_all(&mut self, ctx: &mut AppContext) -> Result<()> {
        for p in self.participants.iter_mut() {
            p.prepare_commit(ctx)?;
        }
        Ok(())
    }

    fn apply_all(&mut self, ctx: &mut AppContext) -> Result<()> {
        for p in self.participants.iter_mut() {
            p.apply_prepared(ctx)?;
        }
        Ok(())
    }

    fn discard_all(&mut self, ctx: &mut AppContext) {
        for p in self.participants.iter_mut() {
            p.discard_stage(ctx);
        }
    }
}

impl AppContext {
    fn cards_repo(&mut self) -> &mut Repository<Card> {
        &mut self.cards
    }
    fn events_repo(&mut self) -> &mut Repository<Event> {
        &mut self.events
    }
    fn tasks_repo(&mut self) -> &mut Repository<Task> {
        &mut self.tasks
    }
}

#[derive(Debug)]
struct CommandOp {
    name: String,
    args: Vec<Arg>,
}

#[derive(Debug, Default)]
pub struct CommandQueue {
    ops: Vec<CommandOp>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    pub fn push(&mut self, name: &str, args: Vec<Arg>) {
        self.ops.push(CommandOp {
            name: name.to_string(),
            args,
        });
    }

    pub fn execute(
        self,
        ctx: &mut AppContext,
        parser: &CommandParser,
        clear_existing: bool,
    ) -> Result<()> {
        let mut tx = Transaction::new();
        tx.run(ctx, clear_existing, |ctx| {
            for op in &self.ops {
                let cmd = parser.parse(&op.name, &op.args)?;
                cmd.execute(ctx)?;
            }
            Ok(())
        })
    }
}
