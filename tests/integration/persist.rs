use planit::core::persist::SaveFile;

use crate::common::{
    ArgParser, CommandParser, build_context, execute_command, make_temp_dir, run_with_input,
    write_valid_config,
};
use std::fs;
use std::path::PathBuf;

fn write_save_file(path: &PathBuf, save_file: &SaveFile) {
    let contents = serde_json::to_string_pretty(save_file).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn save_command_creates_schedule_file() {
    let dir = make_temp_dir("persist");
    write_valid_config(&dir);

    let output = run_with_input(&dir, "save \"snap\"\nexit\n");
    assert!(output.status.success());

    let path = dir.join("schedules").join("snap.json");
    assert!(path.exists());
}

#[test]
fn read_command_loads_cards() {
    let dir = make_temp_dir("persist");
    write_valid_config(&dir);
    let save_path = dir.join("cards.json");
    let save_file = SaveFile {
        cards: vec![vec!["\"Card\"".into(), "RED".into()]],
        events: Vec::new(),
        tasks: Vec::new(),
    };
    write_save_file(&save_path, &save_file);

    let arg_parser = ArgParser::new();
    let command_parser = CommandParser::new();
    let mut ctx = build_context(&dir);
    let line = format!("read \"{}\"", save_path.display());
    execute_command(&line, &arg_parser, &command_parser, &mut ctx);

    assert_eq!(ctx.cards.len(), 1);
    let card = ctx.cards.get(1).unwrap();
    assert_eq!(card.name, "Card");
}

#[test]
fn read_command_loads_tasks_with_card_mapping() {
    let dir = make_temp_dir("persist");
    write_valid_config(&dir);
    let save_path = dir.join("tasks.json");
    let save_file = SaveFile {
        cards: vec![vec!["\"Tag\"".into(), "GREEN".into()]],
        events: Vec::new(),
        tasks: vec![vec![
            "\"Task\"".into(),
            "1".into(),
            "+C1".into(),
            "@".into(),
            "2099-01-01".into(),
        ]],
    };
    write_save_file(&save_path, &save_file);

    let arg_parser = ArgParser::new();
    let command_parser = CommandParser::new();
    let mut ctx = build_context(&dir);
    let line = format!("read \"{}\"", save_path.display());
    execute_command(&line, &arg_parser, &command_parser, &mut ctx);

    assert_eq!(ctx.tasks.len(), 1);
    let task = ctx.tasks.get(1).unwrap();
    assert_eq!(task.card_id, Some(1));
}
