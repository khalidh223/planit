#[cfg(test)]
mod tests;

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Local;

#[derive(Debug, Copy, Clone)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

trait LogSink: Send + Sync {
    fn log(&self, level: LogLevel, line: &str);
}

#[derive(Default)]
struct StdoutSink;
impl LogSink for StdoutSink {
    fn log(&self, level: LogLevel, line: &str) {
        if matches!(level, LogLevel::Info) {
            println!("{line}");
        }
    }
}

#[derive(Default)]
struct StderrSink;
impl LogSink for StderrSink {
    fn log(&self, level: LogLevel, line: &str) {
        if matches!(level, LogLevel::Warn | LogLevel::Error) {
            eprintln!("{line}");
        }
    }
}

struct FileSink {
    file: Mutex<File>,
}

impl FileSink {
    fn new(dir: impl AsRef<Path>) -> std::io::Result<(Self, PathBuf)> {
        fs::create_dir_all(&dir)?;
        let stamp = Local::now().format("%Y%m%d-%H%M%S");
        let path = dir.as_ref().join(format!("session-{stamp}.log"));
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok((
            Self {
                file: Mutex::new(file),
            },
            path,
        ))
    }
}

impl LogSink for FileSink {
    fn log(&self, _level: LogLevel, line: &str) {
        if let Ok(mut file) = self.file.lock() {
            let _ = writeln!(file, "{line}");
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LogTarget {
    ConsoleOnly,
    ConsoleAndFile,
    FileOnly,
}

impl Default for LogTarget {
    fn default() -> Self {
        LogTarget::ConsoleAndFile
    }
}

#[derive(Clone)]
pub struct Logger {
    console_sinks: Arc<Vec<Arc<dyn LogSink>>>,
    file_state: Arc<Mutex<FileState>>,
    file_enabled: Arc<AtomicBool>,
}

struct FileState {
    sink: Option<Arc<dyn LogSink>>,
    log_path: Option<PathBuf>,
    attempted: bool,
    log_dir: PathBuf,
}

impl Default for FileState {
    fn default() -> Self {
        Self {
            sink: None,
            log_path: None,
            attempted: false,
            log_dir: PathBuf::from("logs"),
        }
    }
}

impl Logger {
    pub fn new() -> Self {
        let console_sinks: Vec<Arc<dyn LogSink>> = vec![
            Arc::new(StdoutSink::default()),
            Arc::new(StderrSink::default()),
        ];

        Self {
            console_sinks: Arc::new(console_sinks),
            file_state: Arc::new(Mutex::new(FileState::default())),
            file_enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    fn ensure_file_sink(&self) -> Option<Arc<dyn LogSink>> {
        let mut state = self.file_state.lock().ok()?;
        if state.attempted {
            return state.sink.clone();
        }
        state.attempted = true;

        match FileSink::new(&state.log_dir) {
            Ok((sink, path)) => {
                let arc: Arc<dyn LogSink> = Arc::new(sink);
                state.log_path = Some(path);
                state.sink = Some(arc.clone());
                Some(arc)
            }
            Err(err) => {
                eprintln!("WARN: File logging unavailable; continuing without a log file. ({err})");
                None
            }
        }
    }

    fn log(&self, level: LogLevel, message: &str, target: LogTarget) {
        let console_line = message.to_string();

        if matches!(target, LogTarget::ConsoleOnly | LogTarget::ConsoleAndFile) {
            for sink in self.console_sinks.iter() {
                sink.log(level, &console_line);
            }
        }

        if matches!(target, LogTarget::ConsoleAndFile | LogTarget::FileOnly)
            && self.file_enabled.load(Ordering::SeqCst)
        {
            if let Some(file_sink) = self.ensure_file_sink() {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                let file_line = format!("[{timestamp}] {:<5} {message}", level);
                file_sink.log(level, &file_line);
            }
        }
    }

    pub fn info(&self, message: impl AsRef<str>, target: LogTarget) {
        self.log(LogLevel::Info, message.as_ref(), target);
    }

    pub fn warn(&self, message: impl AsRef<str>, target: LogTarget) {
        self.log(LogLevel::Warn, message.as_ref(), target);
    }

    pub fn error(&self, message: impl AsRef<str>, target: LogTarget) {
        self.log(LogLevel::Error, message.as_ref(), target);
    }

    pub fn set_file_logging_enabled(&self, enabled: bool) {
        self.file_enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn set_log_dir(&self, dir: impl AsRef<Path>) {
        if let Ok(mut state) = self.file_state.lock() {
            if state.sink.is_none() && !state.attempted {
                state.log_dir = dir.as_ref().to_path_buf();
            }
        }
    }

    pub fn log_dir(&self) -> Option<PathBuf> {
        self.file_state.lock().ok().map(|s| s.log_dir.clone())
    }

    pub fn file_logging_enabled(&self) -> bool {
        self.file_enabled.load(Ordering::SeqCst)
    }

    pub fn log_path(&self) -> Option<PathBuf> {
        self.file_state.lock().ok().and_then(|s| s.log_path.clone())
    }
}

impl fmt::Debug for Logger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.log_path();
        f.debug_struct("Logger").field("log_path", &path).finish()
    }
}
