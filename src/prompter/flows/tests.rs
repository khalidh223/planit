use super::{config_edit::ConfigEditFlow, main_flow::MainFlow};
use crate::config::Config;
use crate::core::{
    context::AppContext,
    repository::Repository,
    types::{TaskOverflowPolicy, TimeRange},
};
use crate::logging::Logger;
use crate::{core::models::Task, prompter::models::Flow};
use std::fs;
use std::path::PathBuf;

fn temp_config_path() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("planit-input-flow-{nanos}.json"))
}

fn write_sample_config(path: &PathBuf) {
    let json = r#"{
  "range": { "value": "8:00AM-6:00PM", "description": "Daily hours" },
  "task_overflow_policy": { "value": "allow", "description": "overflow" },
  "task_scheduling_order": { "value": "longest-task-first", "description": "order" },
  "schedule_start_date": { "value": null, "description": "start date" },
  "file_logging_enabled": { "value": "True", "description": "File logging" }
}"#;
    fs::write(path, json).unwrap();
}

fn make_ctx() -> AppContext {
    let path = temp_config_path();
    write_sample_config(&path);
    let config = Config::load_from(&path).unwrap();
    let logger = Logger::new();
    let schedules_dir = std::env::temp_dir().join("planit-input-schedules");
    let logs_dir = std::env::temp_dir().join("planit-input-logs");
    logger.set_log_dir(&logs_dir);
    logger.set_file_logging_enabled(config.file_logging_enabled());
    AppContext {
        config,
        tasks: Repository::<Task>::new(),
        events: Repository::new(),
        cards: Repository::new(),
        logger,
        startup_displayed: false,
        config_path: path,
        schedules_dir,
        logs_dir,
    }
}

#[test]
fn main_flow_render_sets_startup_and_prompts() {
    let mut ctx = make_ctx();
    let mut flow = MainFlow::new(&mut ctx);
    flow.render().unwrap();
    assert!(ctx.startup_displayed);
}

#[test]
fn main_flow_handles_exit_and_empty() {
    let mut ctx = make_ctx();
    let mut flow = MainFlow::new(&mut ctx);
    // empty
    let ctrl = flow.handle_input("").unwrap();
    matches!(ctrl, crate::prompter::models::FlowCtrl::Continue);
    // exit
    let ctrl = flow.handle_input("exit").unwrap();
    matches!(ctrl, crate::prompter::models::FlowCtrl::Finish);
}

#[test]
fn main_flow_parses_and_executes_task_add() {
    let mut ctx = make_ctx();
    let mut flow = MainFlow::new(&mut ctx);
    // add a task
    let ctrl = flow.handle_input(r#"task "Test" 2 @ 2099-01-01"#).unwrap();
    matches!(ctrl, crate::prompter::models::FlowCtrl::Continue);
    assert_eq!(ctx.tasks.len(), 1);
}

#[test]
fn config_edit_flow_walks_states_and_updates_value() {
    let mut ctx = make_ctx();
    let mut flow = ConfigEditFlow::new(&mut ctx);

    // initial render; still ShowTable
    flow.render().unwrap();
    assert!(matches!(
        flow.state(),
        crate::prompter::models::ConfigState::ShowTable
    ));
    // answer yes
    assert!(matches!(
        flow.handle_input("Y").unwrap(),
        crate::prompter::models::FlowCtrl::Continue
    ));
    assert!(matches!(
        flow.state(),
        crate::prompter::models::ConfigState::SelectId
    ));
    // select id 0 (range)
    flow.render().unwrap();
    flow.handle_input("0").unwrap();
    assert_eq!(flow.selected_index(), Some(0));
    // render show current -> moves to AskNewValue
    flow.render().unwrap();
    assert!(matches!(
        flow.state(),
        crate::prompter::models::ConfigState::AskNewValue
    ));
    // apply new value
    flow.handle_input("9:00AM-5:00PM").unwrap();

    // range should change
    assert_eq!(
        ctx.config.range(),
        &TimeRange::try_from_str("9:00AM-5:00PM").unwrap()
    );
}

#[test]
fn config_edit_flow_handles_invalid_inputs_gracefully() {
    let mut ctx = make_ctx();
    let mut flow = ConfigEditFlow::new(&mut ctx);
    // invalid Y/N
    flow.render().unwrap();
    flow.handle_input("maybe").unwrap();
    assert!(matches!(
        flow.state(),
        crate::prompter::models::ConfigState::ShowTable
    ));
    // move to select id
    flow.handle_input("Y").unwrap();
    // invalid id
    flow.render().unwrap();
    flow.handle_input("999").unwrap();
    // valid id, then bad value triggers error branch
    flow.handle_input("1").unwrap(); // TaskOverflowPolicy
    flow.render().unwrap();
    flow.handle_input("not-a-policy").unwrap();
    // capture state before dropping flow (which holds &mut ctx)
    let stayed_in_ask_new =
        matches!(flow.state(), crate::prompter::models::ConfigState::AskNewValue);
    drop(flow);

    // value should remain unchanged
    let policy = ctx.config.task_overflow_policy().clone();
    assert!(stayed_in_ask_new);
    assert_eq!(policy, TaskOverflowPolicy::Allow);
}

#[test]
fn config_edit_flow_updates_file_logging_setting_in_logger() {
    let mut ctx = make_ctx();
    let mut flow = ConfigEditFlow::new(&mut ctx);

    flow.render().unwrap();
    flow.handle_input("Y").unwrap();
    flow.render().unwrap();
    flow.handle_input("4").unwrap(); // File logging enabled
    flow.render().unwrap();
    flow.handle_input("False").unwrap();

    assert!(!ctx.config.file_logging_enabled());
    assert!(!ctx.logger.file_logging_enabled());
}
