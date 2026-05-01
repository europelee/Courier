use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LogEntry {
    HttpRequest {
        tunnel_id: String,
        method: String,
        path: String,
        status: u16,
        duration_ms: u64,
        timestamp: String,
    },
    TunnelConnected {
        tunnel_id: String,
        subdomain: String,
        local_port: u16,
        timestamp: String,
    },
    TunnelDisconnected {
        tunnel_id: String,
        timestamp: String,
    },
}

pub struct AccessLogWriter {
    log_dir: PathBuf,
    current_date: NaiveDate,
    writer: BufWriter<File>,
}

impl AccessLogWriter {
    pub fn new(log_dir: PathBuf) -> Self {
        fs::create_dir_all(&log_dir).ok();
        let today = Local::now().date_naive();
        let file = Self::open_file(&log_dir, today);
        AccessLogWriter {
            log_dir,
            current_date: today,
            writer: BufWriter::new(file),
        }
    }

    fn open_file(log_dir: &PathBuf, date: NaiveDate) -> File {
        let path = log_dir.join(format!("access-{}.log", date.format("%Y-%m-%d")));
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            // 不变量：log_dir 已由 create_dir_all 保证存在
            .expect("failed to open log file")
    }

    pub fn write(&mut self, entry: &LogEntry) -> std::io::Result<()> {
        let today = Local::now().date_naive();
        if today != self.current_date {
            self.writer.flush()?;
            self.rotate(today);
        }
        let line = serde_json::to_string(entry).unwrap_or_default();
        writeln!(self.writer, "{}", line)?;
        self.writer.flush()
    }

    fn rotate(&mut self, new_date: NaiveDate) {
        self.current_date = new_date;
        let file = Self::open_file(&self.log_dir, new_date);
        self.writer = BufWriter::new(file);
        self.cleanup_old_files();
    }

    pub fn cleanup_old_files(&self) {
        let cutoff = Local::now().date_naive() - chrono::Duration::days(7);
        let Ok(entries) = fs::read_dir(&self.log_dir) else { return };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(date_str) = name
                .strip_prefix("access-")
                .and_then(|s| s.strip_suffix(".log"))
            {
                if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    if date < cutoff {
                        fs::remove_file(entry.path()).ok();
                    }
                }
            }
        }
    }
}

pub fn start_log_writer(rx: mpsc::Receiver<LogEntry>, log_dir: PathBuf) {
    tokio::spawn(async move {
        let mut rx = rx;
        let mut writer = AccessLogWriter::new(log_dir);
        while let Some(entry) = rx.recv().await {
            if let Err(e) = writer.write(&entry) {
                warn!("访问日志写入失败: {}", e);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_log_entry_serializes_to_json_line() {
        let entry = LogEntry::HttpRequest {
            tunnel_id: "abc123".to_string(),
            method: "GET".to_string(),
            path: "/hello".to_string(),
            status: 200,
            duration_ms: 42,
            timestamp: "2026-04-20T10:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"tunnel_id\":\"abc123\""));
        assert!(json.contains("\"status\":200"));
    }

    #[test]
    fn test_writer_creates_log_file() {
        let dir = tempdir().unwrap();
        let mut writer = AccessLogWriter::new(dir.path().to_path_buf());
        let entry = LogEntry::HttpRequest {
            tunnel_id: "t1".to_string(),
            method: "POST".to_string(),
            path: "/api".to_string(),
            status: 201,
            duration_ms: 10,
            timestamp: "2026-04-20T10:00:00Z".to_string(),
        };
        writer.write(&entry).unwrap();
        let files: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(files.len(), 1);
    }
}
