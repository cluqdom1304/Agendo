//! A minimal file logger (D-90): one line per event in `agendo.log`, written by a background
//! thread so callers never wait on the disk.
//!
//! Only technical events and errors go here. Never pass the content of a note or a task: the
//! callers hand in static text plus ids, counts and error messages.

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, OnceLock,
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    time::Duration,
};

use crate::scheduler::clock::Clock;

pub const LOG_FILE: &str = "agendo.log";
/// A file is rotated when the next line would take it past this size.
pub const MAX_BYTES: u64 = 1024 * 1024;
/// Rotated files kept next to the current one (`agendo.log.1`, `agendo.log.2`).
pub const KEEP: usize = 2;
/// Longest message kept, in characters; the rest is cut.
pub const MAX_MESSAGE_CHARS: usize = 500;
/// Lines waiting for the writer; past this, new lines are dropped instead of blocking the caller.
const QUEUE: usize = 1024;
/// Reads and writes slower than this are logged.
pub const SLOW_QUERY: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
}

impl Level {
    fn label(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
        }
    }
}

enum Message {
    Line(String),
    Flush(SyncSender<()>),
}

pub struct Logger {
    queue: SyncSender<Message>,
    clock: Arc<dyn Clock>,
}

impl Logger {
    pub fn open(path: &Path, clock: Arc<dyn Clock>) -> std::io::Result<Self> {
        Self::open_with(path, clock, MAX_BYTES, KEEP)
    }

    pub fn open_with(
        path: &Path,
        clock: Arc<dyn Clock>,
        max_bytes: u64,
        keep: usize,
    ) -> std::io::Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let sink = Sink::open(path.to_path_buf(), max_bytes, keep)?;
        let (queue, inbox) = sync_channel(QUEUE);
        std::thread::Builder::new()
            .name("agendo-log".into())
            .spawn(move || sink.run(&inbox))?;
        Ok(Self { queue, clock })
    }

    pub fn log(&self, level: Level, target: &str, message: &str) {
        let line = format_line(&self.clock.now_string(), level, target, message);
        // A full queue means the disk is far behind: losing a line beats stalling the caller.
        let _ = self.queue.try_send(Message::Line(line));
    }

    /// Waits until every line sent so far is on disk.
    pub fn flush(&self) {
        let (done, wait) = sync_channel(1);
        if self.queue.send(Message::Flush(done)).is_ok() {
            let _ = wait.recv_timeout(Duration::from_secs(2));
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.flush();
    }
}

/// `2026-03-10T10:00:00Z WARN [scheduler] message`, on a single line and of bounded length.
pub fn format_line(timestamp: &str, level: Level, target: &str, message: &str) -> String {
    let mut clean: String = message
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take(MAX_MESSAGE_CHARS + 1)
        .collect();
    if clean.chars().count() > MAX_MESSAGE_CHARS {
        clean = clean.chars().take(MAX_MESSAGE_CHARS).collect();
        clean.push('…');
    }
    format!("{timestamp} {} [{target}] {clean}\n", level.label())
}

struct Sink {
    path: PathBuf,
    file: File,
    size: u64,
    max_bytes: u64,
    keep: usize,
}

impl Sink {
    fn open(path: PathBuf, max_bytes: u64, keep: usize) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let size = file.metadata()?.len();
        Ok(Self {
            path,
            file,
            size,
            max_bytes,
            keep,
        })
    }

    fn run(mut self, inbox: &Receiver<Message>) {
        while let Ok(message) = inbox.recv() {
            match message {
                Message::Line(line) => self.write(&line),
                Message::Flush(done) => {
                    let _ = self.file.flush();
                    let _ = done.send(());
                }
            }
        }
    }

    fn write(&mut self, line: &str) {
        let length = line.len() as u64;
        if self.size > 0 && self.size + length > self.max_bytes {
            self.rotate();
        }
        if self.file.write_all(line.as_bytes()).is_ok() {
            self.size += length;
        }
    }

    fn numbered(&self, n: usize) -> PathBuf {
        let mut name = self.path.clone().into_os_string();
        name.push(format!(".{n}"));
        PathBuf::from(name)
    }

    /// `agendo.log` becomes `.1`, `.1` becomes `.2`, and the oldest is deleted. Best effort: if
    /// a step fails the log just keeps growing until the next try.
    fn rotate(&mut self) {
        let _ = self.file.flush();
        if self.keep == 0 {
            let _ = fs::remove_file(&self.path);
        } else {
            let _ = fs::remove_file(self.numbered(self.keep));
            for n in (1..self.keep).rev() {
                let _ = fs::rename(self.numbered(n), self.numbered(n + 1));
            }
            if fs::rename(&self.path, self.numbered(1)).is_err() {
                return;
            }
        }
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            self.file = file;
            self.size = 0;
        }
    }
}

static GLOBAL: OnceLock<Logger> = OnceLock::new();

/// Starts logging to `path`. A failure to open the file is not worth stopping the app for.
pub fn init(path: &Path, clock: Arc<dyn Clock>) {
    if let Ok(logger) = Logger::open(path, clock) {
        let _ = GLOBAL.set(logger);
    }
}

/// Logs through the global logger; a no-op until [`init`] has run (and in unit tests).
pub fn log(level: Level, target: &str, message: &str) {
    if let Some(logger) = GLOBAL.get() {
        logger.log(level, target, message);
    }
}

pub fn error(target: &str, message: &str) {
    log(Level::Error, target, message);
}

pub fn warn(target: &str, message: &str) {
    log(Level::Warn, target, message);
}

pub fn info(target: &str, message: &str) {
    log(Level::Info, target, message);
}

pub fn flush() {
    if let Some(logger) = GLOBAL.get() {
        logger.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::dates, scheduler::clock::ManualClock, test_support::TempDir};

    fn clock() -> Arc<ManualClock> {
        Arc::new(ManualClock::new(
            dates::parse_instant("2026-03-10T10:00:00Z").expect("instant"),
        ))
    }

    #[test]
    fn line_has_timestamp_level_and_target() {
        let line = format_line("2026-03-10T10:00:00Z", Level::Warn, "scheduler", "hola");
        assert_eq!(line, "2026-03-10T10:00:00Z WARN [scheduler] hola\n");
    }

    #[test]
    fn newlines_and_controls_become_spaces_so_a_line_stays_a_line() {
        let line = format_line("t", Level::Info, "x", "a\nb\r\tc");
        assert_eq!(line, "t INFO [x] a b  c\n");
    }

    #[test]
    fn long_messages_are_truncated_on_a_character_boundary() {
        let line = format_line("t", Level::Error, "x", &"ñ".repeat(MAX_MESSAGE_CHARS + 50));
        let text = line.trim_end();
        assert!(text.ends_with('…'));
        assert_eq!(
            text.chars().filter(|c| *c == 'ñ').count(),
            MAX_MESSAGE_CHARS
        );
    }

    #[test]
    fn lines_are_written_in_order_with_the_clock_time() {
        let dir = TempDir::new();
        let path = dir.path().join(LOG_FILE);
        let clock = clock();
        let logger = Logger::open(&path, clock.clone()).expect("open");
        logger.log(Level::Info, "a", "one");
        clock.advance(chrono::Duration::seconds(5));
        logger.log(Level::Error, "b", "two");
        logger.flush();
        let text = fs::read_to_string(&path).expect("read");
        assert_eq!(
            text,
            "2026-03-10T10:00:00Z INFO [a] one\n2026-03-10T10:00:05Z ERROR [b] two\n"
        );
    }

    #[test]
    fn the_file_rotates_by_size_and_keeps_only_the_configured_history() {
        let dir = TempDir::new();
        let path = dir.path().join(LOG_FILE);
        let logger = Logger::open_with(&path, clock(), 200, 2).expect("open");
        for n in 0..40 {
            logger.log(Level::Info, "t", &format!("line number {n:03}"));
        }
        logger.flush();
        let read = |name: &str| fs::read_to_string(dir.path().join(name)).unwrap_or_default();
        let (current, first, second) = (read(LOG_FILE), read("agendo.log.1"), read("agendo.log.2"));
        assert!(!first.is_empty() && !second.is_empty());
        assert!(!dir.path().join("agendo.log.3").exists());
        for text in [&current, &first, &second] {
            assert!(text.len() <= 200, "{} bytes", text.len());
        }
        assert!(current.contains("line number 039"));
        assert!(!second.contains("line number 000"));
    }

    #[test]
    fn an_existing_log_is_appended_to_after_a_restart() {
        let dir = TempDir::new();
        let path = dir.path().join(LOG_FILE);
        for text in ["first", "second"] {
            let logger = Logger::open(&path, clock()).expect("open");
            logger.log(Level::Info, "t", text);
            logger.flush();
        }
        let lines = fs::read_to_string(&path).expect("read");
        assert_eq!(lines.lines().count(), 2);
    }

    #[test]
    fn logging_before_init_is_a_no_op() {
        info("t", "nobody listens");
        flush();
    }
}
