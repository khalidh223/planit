use super::{Config, ConfigKey, models::*};
use crate::core::types::{Bool, TaskOverflowPolicy, TaskSchedulingOrder, TimeRange};
use crate::errors::Error;
use crate::extensions::enums::valid_csv;
use chrono::NaiveDate;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let uniq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("planit-config-test-{nanos}-{uniq}.json"))
}

fn sample_config_file(path: &std::path::Path) {
    let json = r#"{
  "range": { "value": "8:00AM-5:00PM", "description": "hours" },
  "task_overflow_policy": { "value": "allow", "description": "overflow" },
  "task_scheduling_order": { "value": "longest-task-first", "description": "order" },
  "schedule_start_date": { "value": "2099-01-01", "description": "start date" },
  "file_logging_enabled": { "value": "True", "description": "file logging" }
}"#;
    fs::write(path, json).unwrap();
}

#[test]
fn load_from_reads_config_and_rows() {
    let path = temp_path();
    sample_config_file(&path);
    let cfg = Config::load_from(&path).expect("config should load");

    assert_eq!(
        cfg.range(),
        &TimeRange::try_from_str("8:00AM-5:00PM").unwrap()
    );
    assert_eq!(cfg.task_overflow_policy(), &TaskOverflowPolicy::Allow);
    assert_eq!(
        cfg.task_scheduling_order(),
        &TaskSchedulingOrder::LongestTaskFirst
    );
    assert_eq!(
        cfg.schedule_start_date(),
        &Some(NaiveDate::from_ymd_opt(2099, 1, 1).unwrap())
    );
    assert!(cfg.file_logging_enabled());

    let rows = cfg.rows();
    assert_eq!(rows.len(), 5);
    assert!(rows.iter().any(|(k, _, _)| k == "RANGE"));
}

#[test]
fn load_from_reports_missing_file() {
    let path = temp_path();
    let err = Config::load_from(&path).unwrap_err();
    match err {
        Error::Parse(msg) => {
            let expected = format!("Configuration file '{}' not found.", path.display());
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn load_from_reports_invalid_json() {
    let path = temp_path();
    fs::write(&path, "{").unwrap();
    let err = Config::load_from(&path).unwrap_err();
    match err {
        Error::Parse(msg) => {
            let prefix = format!("Invalid JSON in '{}':", path.display());
            assert!(msg.starts_with(&prefix));
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn load_from_reports_read_error_for_directory() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("planit-config-read-dir-{nanos}"));
    fs::create_dir_all(&path).unwrap();
    let io_err = fs::read_to_string(&path).unwrap_err();

    let err = Config::load_from(&path).unwrap_err();
    match err {
        Error::Parse(msg) => {
            let expected = format!("Failed to read {}: {}", path.display(), io_err);
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn set_key_updates_values_and_tracks_last_change() {
    let path = temp_path();
    sample_config_file(&path);
    let mut cfg = Config::load_from(&path).unwrap();

    cfg.set_key(ConfigKey::Range, "9:00AM-3:00PM").unwrap();
    let new_range_str = cfg.range().to_string();
    let old_range_str = "8:00AM-5:00PM".to_string();
    assert_eq!(
        cfg.range(),
        &TimeRange::try_from_str("9:00AM-3:00PM").unwrap()
    );
    let change = cfg.take_last_change().unwrap();
    assert_eq!(change.0, "RANGE");
    assert_eq!(change.1, old_range_str); // old
    assert_eq!(change.2, new_range_str); // new
}

#[test]
fn set_key_updates_file_logging_enabled() {
    let path = temp_path();
    sample_config_file(&path);
    let mut cfg = Config::load_from(&path).unwrap();

    cfg.set_key(ConfigKey::FileLoggingEnabled, "False").unwrap();
    assert!(!cfg.file_logging_enabled());
}

#[test]
fn set_by_index_rejects_invalid_id() {
    let path = temp_path();
    sample_config_file(&path);
    let mut cfg = Config::load_from(&path).unwrap();

    let err = cfg.set_by_index(99, "allow").unwrap_err();
    match err {
        Error::Parse(msg) => assert_eq!(msg, "Invalid ID: 99"),
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn set_by_index_and_set_many_work() {
    let path = temp_path();
    sample_config_file(&path);
    let mut cfg = Config::load_from(&path).unwrap();

    cfg.set_by_index(1, "block").unwrap(); // TaskOverflowPolicy
    assert_eq!(cfg.task_overflow_policy(), &TaskOverflowPolicy::Block);

    cfg.set_many([
        ("TASK_SCHEDULING_ORDER", "shortest-task-first"),
        ("SCHEDULE_START_DATE", "2099-02-02"),
    ])
    .unwrap();

    assert_eq!(
        cfg.task_scheduling_order(),
        &TaskSchedulingOrder::ShortestTaskFirst
    );
    assert_eq!(
        cfg.schedule_start_date(),
        &Some(NaiveDate::from_ymd_opt(2099, 2, 2).unwrap())
    );
}

#[test]
fn set_rejects_unknown_key() {
    let path = temp_path();
    sample_config_file(&path);
    let mut cfg = Config::load_from(&path).unwrap();

    let err = cfg.set("NOPE", "1").unwrap_err();
    match err {
        Error::Parse(msg) => {
            let expected = format!(
                "Unknown configuration key 'NOPE'. Valid keys: {}",
                valid_csv::<ConfigKey>()
            );
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn set_many_rejects_unknown_key() {
    let path = temp_path();
    sample_config_file(&path);
    let mut cfg = Config::load_from(&path).unwrap();

    let err = cfg.set_many([("BOGUS", "1")]).unwrap_err();
    match err {
        Error::Parse(msg) => {
            let expected = format!(
                "Unknown configuration key 'BOGUS'. Valid keys: {}",
                valid_csv::<ConfigKey>()
            );
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn config_rows_get_returns_first_row() {
    let path = temp_path();
    sample_config_file(&path);
    let cfg = Config::load_from(&path).unwrap();

    let rows = cfg.rows();
    let first = rows.get(0).unwrap();
    assert_eq!(first, &rows[0]);
}

#[test]
fn config_rows_is_not_empty_for_sample_config() {
    let path = temp_path();
    sample_config_file(&path);
    let cfg = Config::load_from(&path).unwrap();

    let rows = cfg.rows();
    assert!(!rows.is_empty());
}

#[test]
fn config_view_exposes_loaded_values() {
    let path = temp_path();
    sample_config_file(&path);
    let cfg = Config::load_from(&path).unwrap();

    let view = cfg.view();
    assert_eq!(
        view.range.get_value(),
        &TimeRange::try_from_str("8:00AM-5:00PM").unwrap()
    );
}

#[test]
fn config_items_validate_and_set() {
    let mut range = RangeConfigItem {
        value: TimeRange::try_from_str("8AM-9AM").unwrap(),
        description: "range".into(),
    };
    assert!(range.set_value("9AM-10AM").is_ok());

    let mut overflow = TaskOverflowPolicyConfigItem {
        value: TaskOverflowPolicy::Allow,
        description: "overflow".into(),
    };
    assert!(overflow.set_value("block").is_ok());

    let mut order = TaskSchedulingOrderConfigItem {
        value: TaskSchedulingOrder::LongestTaskFirst,
        description: "order".into(),
    };
    assert!(order.set_value("shortest-task-first").is_ok());

    let mut start = StartDateConfigItem {
        value: None,
        description: "start".into(),
    };
    assert!(start.set_value("2099-03-03").is_ok());
    assert_eq!(
        start.get_value(),
        &Some(NaiveDate::from_ymd_opt(2099, 3, 3).unwrap())
    );
    assert!(start.set_value("").is_ok()); // clears
    assert_eq!(start.get_value(), &None);

    let mut file_logging = FileLoggingConfigItem {
        value: Bool(false),
        description: "file logging".into(),
    };
    assert!(file_logging.set_value("True").is_ok());
    assert_eq!(file_logging.get_value(), &Bool(true));
}

#[test]
fn set_key_does_not_update_last_change_on_error() {
    let path = temp_path();
    sample_config_file(&path);
    let mut cfg = Config::load_from(&path).unwrap();

    let err = cfg.set_key(ConfigKey::Range, "not-a-range").unwrap_err();
    match err {
        Error::Parse(msg) => {
            let expected = TimeRange::try_from_str("not-a-range").unwrap_err();
            match expected {
                Error::Parse(expected_msg) => assert_eq!(msg, expected_msg),
                other => panic!("expected parse error, got {other:?}"),
            }
        }
        other => panic!("expected parse error, got {other:?}"),
    }
    assert!(cfg.take_last_change().is_none());
}

#[test]
fn set_many_rejects_invalid_value() {
    let path = temp_path();
    sample_config_file(&path);
    let mut cfg = Config::load_from(&path).unwrap();

    let err = cfg
        .set_many([("TASK_OVERFLOW_POLICY", "not-a-policy")])
        .unwrap_err();
    match err {
        Error::Parse(msg) => {
            let expected = TaskOverflowPolicy::try_from("not-a-policy").unwrap_err();
            match expected {
                Error::Parse(expected_msg) => assert_eq!(msg, expected_msg),
                other => panic!("expected parse error, got {other:?}"),
            }
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn set_key_reports_write_error() {
    let path = temp_path();
    sample_config_file(&path);
    let mut cfg = Config::load_from(&path).unwrap();

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let write_dir = std::env::temp_dir().join(format!("planit-config-write-dir-{nanos}"));
    fs::create_dir_all(&write_dir).unwrap();
    let io_err = fs::write(&write_dir, "noop").unwrap_err();
    cfg.path = write_dir.clone();

    let err = cfg.set_key(ConfigKey::Range, "9AM-10AM").unwrap_err();
    match err {
        Error::Parse(msg) => {
            let expected = format!("Failed to write {}: {}", write_dir.display(), io_err);
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}
