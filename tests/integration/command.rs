use planit::core::types::{CardColor, DayOfWeek};

use crate::common::{
    ArgParser, CommandParser, build_context, execute_command, make_temp_dir, normalized_lines,
    read_log_contents, run_with_input, write_valid_config,
};

#[test]
fn unknown_command_reports_error_and_continues() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let output = run_with_input(&dir, "frobnicate\nexit\n");

    assert!(output.status.success());
    let stderr_lines = normalized_lines(&output.stderr);
    let expected = "Command resolution failed for 'frobnicate'. Unknown command: frobnicate";
    assert!(
        stderr_lines.iter().any(|line| line == expected),
        "stderr did not include expected error. stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn main_can_schedule_after_adding_task() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let input = "task \"Test\" 1 @ 2099-01-01\nschedule\nexit\n";
    let output = run_with_input(&dir, input);
    assert!(output.status.success(), "schedule run should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SCHEDULE"),
        "stdout did not include schedule output"
    );
}

#[test]
fn man_command_prints_general_manual() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let output = run_with_input(&dir, "man\nexit\n");

    assert!(output.status.success());
    let stdout_lines = normalized_lines(&output.stdout);
    assert!(stdout_lines.iter().any(|line| line == "NAME"));
    assert!(stdout_lines
        .iter()
        .any(|line| line == "planit - Personal scheduling CLI."));
}

#[test]
fn man_command_prints_task_manual() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let output = run_with_input(&dir, "man task\nexit\n");

    assert!(output.status.success());
    let stdout_lines = normalized_lines(&output.stdout);
    assert!(stdout_lines
        .iter()
        .any(|line| line == "task \"<name>\" <hours> [cardId] @ <date>"));
}

#[test]
fn task_add_modify_and_delete_flow_succeeds() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let arg_parser = ArgParser::new();
    let command_parser = CommandParser::new();
    let mut ctx = build_context(&dir);

    execute_command(
        "task \"Test\" 1 @ 2099-01-01",
        &arg_parser,
        &command_parser,
        &mut ctx,
    );
    assert_eq!(ctx.tasks.len(), 1, "task should be inserted");
    let t = ctx.tasks.get(1).expect("task 1 should exist");
    assert_eq!(t.name, "Test");
    assert!((t.hours - 1.0).abs() < f32::EPSILON);
    assert_eq!(t.date.to_string(), "2099-01-01");

    execute_command(
        "mod task 1 \"Updated\" 2 @ 2099-02-02",
        &arg_parser,
        &command_parser,
        &mut ctx,
    );
    assert_eq!(
        ctx.tasks.len(),
        1,
        "task count should remain 1 after modify"
    );
    let t = ctx.tasks.get(1).expect("task 1 should still exist");
    assert_eq!(t.name, "Updated");
    assert!((t.hours - 2.0).abs() < f32::EPSILON);
    assert_eq!(t.date.to_string(), "2099-02-02");

    execute_command("del task 1", &arg_parser, &command_parser, &mut ctx);
    assert_eq!(ctx.tasks.len(), 0, "task should be deleted from repository");
}

#[test]
fn event_add_modify_and_delete_flow_succeeds() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let arg_parser = ArgParser::new();
    let command_parser = CommandParser::new();
    let mut ctx = build_context(&dir);

    execute_command(
        "event true \"Standup\" @ mon 9:00AM-10:00AM",
        &arg_parser,
        &command_parser,
        &mut ctx,
    );
    assert_eq!(ctx.events.len(), 1, "event should be inserted");
    let e = ctx.events.get(1).expect("event 1 should exist");
    assert_eq!(e.name, "Standup");
    assert_eq!(e.days, vec![DayOfWeek::Mon]);
    assert!(e.recurring);
    assert_eq!(e.time_range.to_string(), "9:00AM-10:00AM");

    execute_command(
        "mod event 1 false \"Retro\" @ tue 1:00PM-2:00PM",
        &arg_parser,
        &command_parser,
        &mut ctx,
    );
    assert_eq!(
        ctx.events.len(),
        1,
        "event count should remain 1 after modify"
    );
    let e = ctx.events.get(1).expect("event 1 should still exist");
    assert_eq!(e.name, "Retro");
    assert_eq!(e.days, vec![DayOfWeek::Tue]);
    assert!(!e.recurring);
    assert_eq!(e.time_range.to_string(), "1:00PM-2:00PM");

    execute_command("del event 1", &arg_parser, &command_parser, &mut ctx);
    assert_eq!(
        ctx.events.len(),
        0,
        "event should be deleted from repository"
    );
}

#[test]
fn card_add_modify_delete_flow_succeeds() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let arg_parser = ArgParser::new();
    let command_parser = CommandParser::new();
    let mut ctx = build_context(&dir);

    execute_command(
        "card \"Backlog\" red",
        &arg_parser,
        &command_parser,
        &mut ctx,
    );
    assert_eq!(ctx.cards.len(), 1, "card should be inserted");
    let c = ctx.cards.get(1).expect("card 1 should exist");
    assert_eq!(c.name, "Backlog");
    assert_eq!(c.color, CardColor::Red);

    execute_command(
        "mod card 1 \"Doing\" blue",
        &arg_parser,
        &command_parser,
        &mut ctx,
    );
    assert_eq!(
        ctx.cards.len(),
        1,
        "card count should remain 1 after modify"
    );
    let c = ctx.cards.get(1).expect("card 1 should still exist");
    assert_eq!(c.name, "Doing");
    assert_eq!(c.color, CardColor::Blue);

    execute_command("del card 1", &arg_parser, &command_parser, &mut ctx);
    assert_eq!(ctx.cards.len(), 0, "card should be deleted from repository");
}

#[test]
fn parse_error_does_not_stop_followup_command() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let output = run_with_input(
        &dir,
        "task \"Bad\" 0 @ 2099-01-01\ntask \"Good\" 1 @ 2099-01-01\nexit\n",
    );
    assert!(output.status.success(), "session should complete");

    let stderr_lines = normalized_lines(&output.stderr);
    let count = stderr_lines
        .iter()
        .filter(|l| l.contains("Hours must be greater than 0"))
        .count();
    assert_eq!(
        count, 1,
        "error should be reported exactly once: {stderr_lines:?}"
    );

    let stdout_lines = normalized_lines(&output.stdout);
    assert!(
        stdout_lines
            .iter()
            .any(|l| l.contains("Added task with id 1: Task(id=1, name='Good'")),
        "stdout did not include successful add after error:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn schedule_help_flag_prints_usage() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let output = run_with_input(&dir, "schedule -h\nexit\n");

    assert!(output.status.success());
    let stdout_lines = normalized_lines(&output.stdout);
    let expected = "schedule      # Schedule tasks";
    assert!(
        stdout_lines.iter().any(|l| l == expected),
        "stdout did not include schedule usage:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn colors_help_command_lists_valid_colors() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let output = run_with_input(&dir, "colors -h\nexit\n");

    assert!(output.status.success());
    let stdout_lines = normalized_lines(&output.stdout);
    let expected = "Valid colors: RED, ORANGE, YELLOW, GREEN, LIGHT_BLUE, BLUE, INDIGO, VIOLET, BLACK, LIGHT_GREEN, LIGHT_CORAL";
    assert!(
        stdout_lines.iter().any(|l| l == expected),
        "stdout did not include colors help:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn type_help_commands_show_usage() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let output = run_with_input(&dir, "date -h\ntime -h\nexit\n");
    assert!(output.status.success(), "type help should succeed");

    let stdout_lines = normalized_lines(&output.stdout);
    let help_lines: Vec<&String> = stdout_lines
        .iter()
        .filter(|l| l.contains("Supported formats"))
        .collect();
    assert!(
        help_lines.len() >= 2,
        "expected at least two help lines for date/time, got {help_lines:?}"
    );
}

#[test]
fn log_command_without_log_file_prints_no_logs() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let output = run_with_input(&dir, "log\nexit\n");
    assert!(output.status.success(), "log command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No logs"),
        "stdout should indicate missing logs:\n{stdout}"
    );
    assert!(
        !dir.join("logs").exists(),
        "log command should not create a log file when none existed"
    );
}

#[test]
fn log_command_prints_session_log_without_logging_itself() {
    let dir = make_temp_dir("command");
    write_valid_config(&dir);
    let output = run_with_input(&dir, "task \"Logged\" 1 @ 2099-01-01\nlog\nexit\n");
    assert!(output.status.success(), "session should complete");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[20"),
        "stdout should include printed log file contents:\n{stdout}"
    );
    let log_text =
        read_log_contents(&dir).expect("log file should exist after running a logging command");
    assert!(
        !log_text.contains("Command run: log"),
        "log file should not contain the log command itself:\n{log_text}"
    );
    assert!(
        log_text.contains("Added task with id 1"),
        "log file should include the logged command:\n{log_text}"
    );
}
