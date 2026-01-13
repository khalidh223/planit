use crate::arg::arg_emitter::{
    ArgEmitContext, ArgEmitter, CardArgEmitter, EventArgEmitter, NoRefEmitContext, SaveEmitContext,
    TaskArgEmitter,
};
use crate::arg::arg_parser::ArgParser;
use crate::arg::args::Arg;
use crate::command::command_parser::CommandParser;
use crate::core::aliases::{IdLookup, TokenList, TokenMatrix};
use crate::core::context::AppContext;
use crate::core::models::{Card, Event, Task};
use crate::core::repository::{Repository, Sort};
use crate::core::transaction::CommandQueue;
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFile {
    #[serde(default)]
    pub cards: TokenMatrix,
    #[serde(default)]
    pub events: TokenMatrix,
    #[serde(default)]
    pub tasks: TokenMatrix,
}

pub fn save_state(
    tasks: &Repository<Task>,
    events: &Repository<Event>,
    cards: &Repository<Card>,
    path: &Path,
) -> Result<PathBuf> {
    let file = build_save_file(tasks, events, cards)?;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let contents = serde_json::to_string_pretty(&file)?;
    fs::write(path, contents)?;
    Ok(path.to_path_buf())
}

pub fn load_state(ctx: &mut AppContext, path: &Path) -> Result<()> {
    let save_file = load_save_file(path)?;
    let arg_parser = ArgParser::new();
    let command_parser = CommandParser::new();

    let mut queue = CommandQueue::new();

    for tokens in &save_file.cards {
        let args = arg_parser.parse(tokens)?;
        queue.push("card", args);
    }
    for tokens in &save_file.events {
        let args = arg_parser.parse(tokens)?;
        queue.push("event", args);
    }
    for tokens in &save_file.tasks {
        let args = arg_parser.parse(tokens)?;
        queue.push("task", args);
    }

    queue.execute(ctx, &command_parser, true)
}

fn emit_tokens<E>(
    emitter: &dyn ArgEmitter<E>,
    entity: &E,
    ctx: &dyn ArgEmitContext,
) -> Result<TokenList> {
    let args = emitter.with_entity(entity, ctx)?;
    Ok(args_to_tokens(&args))
}

fn serialize_cards_for_save(
    cards: &[&Card],
    emitter: &CardArgEmitter,
) -> Result<(TokenMatrix, IdLookup)> {
    let mut saved_id_lookup: IdLookup = HashMap::new();
    let mut card_tokens: TokenMatrix = Vec::new();

    for (idx, card) in cards.iter().enumerate() {
        let new_idx = idx + 1;
        saved_id_lookup.insert(card.id, new_idx as i32);
        let tokens = emit_tokens(emitter, card, &NoRefEmitContext)?;
        card_tokens.push(tokens);
    }

    Ok((card_tokens, saved_id_lookup))
}

fn build_save_file(
    tasks: &Repository<Task>,
    events: &Repository<Event>,
    cards: &Repository<Card>,
) -> Result<SaveFile> {
    let card_emitter = CardArgEmitter::new();
    let event_emitter = EventArgEmitter::new();
    let task_emitter = TaskArgEmitter::new();
    let cards_sorted = cards.values(Sort::IdAsc);
    let (card_tokens, card_id_map) = serialize_cards_for_save(&cards_sorted, &card_emitter)?;

    let emit_context = SaveEmitContext {
        id_lookup: &card_id_map,
    };

    let events_tokens = events
        .values(Sort::IdAsc)
        .into_iter()
        .map(|event| emit_tokens(&event_emitter, &event, &emit_context))
        .collect::<Result<Vec<TokenList>>>()?;

    let tasks_tokens = tasks
        .values(Sort::IdAsc)
        .into_iter()
        .map(|task| emit_tokens(&task_emitter, &task, &emit_context))
        .collect::<Result<Vec<TokenList>>>()?;

    Ok(SaveFile {
        cards: card_tokens,
        events: events_tokens,
        tasks: tasks_tokens,
    })
}

fn load_save_file(path: &Path) -> Result<SaveFile> {
    let contents = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

fn args_to_tokens(args: &[Arg]) -> TokenList {
    args.iter()
        .flat_map(|arg| arg.to_tokens())
        .collect::<Vec<_>>()
}
