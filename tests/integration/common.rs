use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

pub use planit::arg::arg_parser::ArgParser;
pub use planit::command::command_parser::CommandParser;
use planit::config::Config;
use planit::core::context::AppContext;
use planit::core::models::{Card, Event, Task};
use planit::core::repository::Repository;
use planit::logging::Logger;

pub fn binary_path() -> String {
    let raw = PathBuf::from(env!("CARGO_BIN_EXE_planit"));
    if raw.is_absolute() {
        return raw.to_string_lossy().to_string();
    }
    let from_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&raw);
    if from_manifest.exists() {
        return from_manifest.to_string_lossy().to_string();
    }
    raw.to_string_lossy().to_string()
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn make_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn write_valid_config(dir: &PathBuf) {
    let cfg = r#"{
      "range": { "value": "8:00AM-6:00PM", "description": "Daily hours" },
      "task_overflow_policy": { "value": "allow", "description": "overflow" },
      "task_scheduling_order": { "value": "longest-task-first", "description": "order" },
      "schedule_start_date": { "value": null, "description": "start" },
      "file_logging_enabled": { "value": "True", "description": "file logging" }
    }"#;
    fs::write(dir.join("config.json"), cfg).unwrap();
}

pub fn write_config_with_start(dir: &PathBuf, start: &str) {
    let cfg = format!(
        r#"{{
      "range": {{ "value": "8:00AM-6:00PM", "description": "Daily hours" }},
      "task_overflow_policy": {{ "value": "allow", "description": "overflow" }},
      "task_scheduling_order": {{ "value": "longest-task-first", "description": "order" }},
      "schedule_start_date": {{ "value": "{}", "description": "start" }},
      "file_logging_enabled": {{ "value": "True", "description": "file logging" }}
    }}"#,
        start
    );
    fs::write(dir.join("config.json"), cfg).unwrap();
}

pub fn run_with_input(dir: &PathBuf, input: &str) -> Output {
    let mut child = Command::new(binary_path())
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn binary");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    child.wait_with_output().unwrap()
}

pub fn run_without_input(dir: &PathBuf) -> Output {
    Command::new(binary_path())
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to run binary")
}

fn strip_ansi_and_control(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut bytes = s.bytes().peekable();

    while let Some(b) = bytes.next() {
        if b == 0x1B {
            if matches!(bytes.peek(), Some(b'[')) {
                let _ = bytes.next();
                while let Some(nb) = bytes.next() {
                    if (nb as char).is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
        }

        if b.is_ascii_control() {
            continue;
        }

        out.push(b as char);
    }

    out
}

pub fn normalized_lines(buf: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(buf)
        .lines()
        .map(|l| {
            let stripped = strip_ansi_and_control(l);
            let trimmed = stripped.trim();
            if let Some(rest) = trimmed.strip_prefix('>') {
                rest.trim().to_string()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|l| !l.is_empty())
        .collect()
}

pub fn build_context(dir: &PathBuf) -> AppContext {
    let config_path = dir.join("config.json");
    let schedules_dir = dir.join("schedules");
    let logs_dir = dir.join("logs");
    let config = Config::load_from(&config_path).expect("config should load");
    let logger = Logger::new();
    logger.set_log_dir(&logs_dir);
    logger.set_file_logging_enabled(config.file_logging_enabled());
    AppContext {
        config,
        tasks: Repository::<Task>::new(),
        events: Repository::<Event>::new(),
        cards: Repository::<Card>::new(),
        logger,
        startup_displayed: false,
        config_path,
        schedules_dir,
        logs_dir,
    }
}

pub fn execute_command(
    line: &str,
    arg_parser: &ArgParser,
    command_parser: &CommandParser,
    ctx: &mut AppContext,
) {
    let mut parts = line.split_whitespace();
    let command = parts.next().unwrap_or("");
    let raw_args: Vec<String> = parts.map(|s| s.to_string()).collect();

    let args = arg_parser
        .parse(&raw_args)
        .unwrap_or_else(|e| panic!("arg parse failed for '{}': {}", line, e));
    let cmd = command_parser
        .parse(command, &args)
        .unwrap_or_else(|e| panic!("command parse failed for '{}': {}", line, e));
    cmd.execute(ctx)
        .unwrap_or_else(|e| panic!("command execute failed for '{}': {}", line, e));
}

pub fn read_log_contents(dir: &PathBuf) -> Option<String> {
    let logs_dir = dir.join("logs");
    let mut entries = fs::read_dir(&logs_dir).ok()?;
    let entry = entries.find_map(|e| e.ok())?;
    fs::read_to_string(entry.path()).ok()
}
