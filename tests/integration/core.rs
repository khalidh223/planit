use std::process::{Command, Stdio};

use crate::common::{
    binary_path, make_temp_dir, run_with_input, run_without_input, write_valid_config,
};

#[test]
fn main_exits_successfully_with_valid_config() {
    let dir = make_temp_dir("core");
    write_valid_config(&dir);
    let output = run_with_input(&dir, "exit\n");
    assert!(output.status.success());
}

#[test]
fn main_fails_when_config_missing() {
    let dir = make_temp_dir("core");
    let output = run_without_input(&dir);
    assert!(
        !output.status.success(),
        "expected failure when config is missing"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Configuration file"),
        "stderr did not mention missing config: {}",
        stderr
    );
}

#[test]
fn main_fails_when_config_is_malformed() {
    let dir = make_temp_dir("core");
    let cfg = r#"{
      "range": { "value": "8;0a", "description": "bad" }
    }"#;
    std::fs::write(dir.join("config.json"), cfg).unwrap();

    let output = run_without_input(&dir);
    assert!(
        !output.status.success(),
        "expected failure on malformed config"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("invalid") || stderr.to_lowercase().contains("parse"),
        "stderr did not mention parse error: {}",
        stderr
    );
}

#[test]
fn main_fails_when_required_key_missing() {
    let dir = make_temp_dir("core");
    let cfg = r#"{
      "range": { "value": "8:00AM-6:00PM", "description": "Daily hours" },
      "task_overflow_policy": { "value": "allow", "description": "overflow" }
    }"#;
    std::fs::write(dir.join("config.json"), cfg).unwrap();

    let output = run_without_input(&dir);
    assert!(
        !output.status.success(),
        "expected failure when required config key is missing"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("task_scheduling_order"),
        "stderr did not mention missing key: {}",
        stderr
    );
}

#[test]
fn main_fails_on_unknown_cli_arg() {
    let dir = make_temp_dir("core");
    write_valid_config(&dir);
    let output = Command::new(binary_path())
        .current_dir(&dir)
        .arg("--nope")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to run binary");

    assert!(
        !output.status.success(),
        "expected failure on unknown cli arg"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown argument"),
        "stderr did not mention unknown argument: {}",
        stderr
    );
}

#[test]
fn main_fails_on_missing_cli_arg_value() {
    let dir = make_temp_dir("core");
    write_valid_config(&dir);
    let output = Command::new(binary_path())
        .current_dir(&dir)
        .arg("--config")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to run binary");

    assert!(
        !output.status.success(),
        "expected failure on missing cli arg value"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Missing value for --config"),
        "stderr did not mention missing value: {}",
        stderr
    );
}
