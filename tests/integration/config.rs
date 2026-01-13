use std::io::BufReader;

use planit::config::Config;
use planit::prompter::flows::config_edit::ConfigEditFlow;
use planit::prompter::prompter::Prompter;

use crate::common::{
    build_context, make_temp_dir, normalized_lines, run_with_input, write_config_with_start,
    write_valid_config,
};

#[test]
fn main_allows_running_config_command() {
    let dir = make_temp_dir("config");
    write_valid_config(&dir);

    let output = run_with_input(&dir, "config\nN\nexit\n");
    assert!(output.status.success(), "config run should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("CONFIG"),
        "stdout did not include CONFIG table"
    );
}

#[test]
fn config_edit_persists_range_change() {
    let dir = make_temp_dir("config");
    write_valid_config(&dir);
    let mut ctx = build_context(&dir);
    let flow = ConfigEditFlow::new(&mut ctx);
    let input = b"Y\n0\n9:00AM-5:00PM\nN\n";
    let reader = BufReader::new(&input[..]);
    Prompter::new()
        .run_with_reader(flow, false, reader)
        .expect("config flow should run");

    let cfg = Config::load_from(dir.join("config.json")).expect("config should reload");
    assert_eq!(
        cfg.range().to_string(),
        "9:00AM-5:00PM",
        "range should persist to disk"
    );
}

#[test]
fn schedule_start_date_blocks_earlier_tasks() {
    let dir = make_temp_dir("config");
    write_config_with_start(&dir, "2099-01-02");
    let output = run_with_input(
        &dir,
        "task \"Early\" 1 @ 2099-01-01\ntask \"OnTime\" 1 @ 2099-01-03\nschedule\nexit\n",
    );
    assert!(output.status.success(), "session should complete");

    let stderr_lines = normalized_lines(&output.stderr);
    let count = stderr_lines
        .iter()
        .filter(|l| l.contains("schedule start date"))
        .count();
    assert_eq!(
        count, 1,
        "expected a single schedule start date validation error, got {stderr_lines:?}"
    );

    let stdout_lines = normalized_lines(&output.stdout);
    assert!(
        stdout_lines
            .iter()
            .any(|l| l.contains("Added task with id 1: Task(id=1, name='OnTime'")),
        "stdout did not include successful add after validation error:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        stdout_lines.iter().any(|l| l == "SCHEDULE"),
        "stdout did not include schedule output:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn config_alt_screen_returns_to_main_flow() {
    let dir = make_temp_dir("config");
    write_valid_config(&dir);
    let output = run_with_input(
        &dir,
        "task \"Flow\" 1 @ 2099-01-01\nconfig\nN\nschedule\nexit\n",
    );
    assert!(output.status.success(), "session should complete");

    let stdout_lines = normalized_lines(&output.stdout);
    assert!(
        stdout_lines
            .iter()
            .any(|l| l.contains("Added task with id 1: Task(id=1, name='Flow'")),
        "stdout did not include task add:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        stdout_lines.iter().any(|l| l == "SCHEDULE"),
        "stdout did not include schedule output after config flow:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}
