use crate::logging::{LogTarget, Logger};
use std::fs;

#[test]
fn logger_defers_file_creation_until_needed() {
    let logger = Logger::new();
    assert!(logger.log_path().is_none());

    // Console-only should not create a log file.
    logger.info("console only", LogTarget::ConsoleOnly);
    assert!(logger.log_path().is_none());

    // First file-targeted log should create the file.
    logger.info("file line", LogTarget::FileOnly);
    let path = logger.log_path().expect("log path should be set");
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("file line"));
    assert!(contents.contains("INFO"));
}

#[test]
fn logger_writes_levels_and_combined_targets() {
    let logger = Logger::new();

    logger.warn("warn line", LogTarget::FileOnly);
    logger.error("error line", LogTarget::ConsoleAndFile);

    let path = logger.log_path().expect("log path should be set");
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("WARN"));
    assert!(contents.contains("warn line"));
    assert!(contents.contains("ERROR"));
    assert!(contents.contains("error line"));
}

#[test]
fn logger_skips_file_logging_when_disabled() {
    let logger = Logger::new();
    logger.set_file_logging_enabled(false);

    logger.info("file should not exist", LogTarget::ConsoleAndFile);
    assert!(logger.log_path().is_none());

    logger.set_file_logging_enabled(true);
    logger.info("now write", LogTarget::FileOnly);
    assert!(logger.log_path().is_some());
}
