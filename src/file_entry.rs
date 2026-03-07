use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::SystemTime;

use chrono::{Datelike, Local, NaiveDate};

/// Represents a single file or directory entry.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_executable: bool,
    pub is_parent: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub permissions: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Size,
    Date,
}

impl FileEntry {
    /// Read all entries from the given directory path.
    pub fn read_directory(path: &Path) -> Vec<FileEntry> {
        let mut entries = Vec::new();

        // Parent entry (..)
        if path.parent().is_some() {
            entries.push(FileEntry {
                name: "../".to_string(),
                is_dir: true,
                is_executable: false,
                is_parent: true,
                size: 0,
                modified: None,
                permissions: "drwxr-xr-x".to_string(),
            });
        }

        if let Ok(read_dir) = fs::read_dir(path) {
            for entry_result in read_dir {
                if let Ok(entry) = entry_result {
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    let name_raw = entry.file_name().to_string_lossy().to_string();
                    let is_dir = metadata.is_dir();
                    let name = if is_dir {
                        format!("{}/", name_raw)
                    } else {
                        name_raw
                    };

                    let mode = metadata.mode();
                    let is_executable = !is_dir && (mode & 0o111 != 0);
                    let permissions = format_permissions(mode, is_dir);
                    let size = if is_dir {
                        metadata.size()
                    } else {
                        metadata.len()
                    };
                    let modified = metadata.modified().ok();

                    entries.push(FileEntry {
                        name,
                        is_dir,
                        is_executable,
                        is_parent: false,
                        size,
                        modified,
                        permissions,
                    });
                }
            }
        }

        // Sort: parent first, then dirs, then files
        entries.sort_by(|a, b| {
            if a.is_parent {
                return std::cmp::Ordering::Less;
            }
            if b.is_parent {
                return std::cmp::Ordering::Greater;
            }
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        entries
    }

    /// Format size for display.
    pub fn format_size(&self) -> String {
        if self.is_parent {
            return "-".to_string();
        }
        format_human_size(self.size)
    }

    /// Format date for display.
    pub fn format_date(&self) -> String {
        if self.is_parent {
            return "--".to_string();
        }
        match self.modified {
            Some(time) => format_date(time),
            None => "--".to_string(),
        }
    }

    /// Get display name (with * prefix for executables).
    pub fn display_name(&self) -> String {
        if self.is_executable {
            format!("*{}", self.name)
        } else {
            self.name.clone()
        }
    }
}

/// Sort entries in-place by the given column.
pub fn sort_entries(entries: &mut [FileEntry], column: SortColumn, ascending: bool) {
    entries.sort_by(|a, b| {
        // Parent always first
        if a.is_parent {
            return std::cmp::Ordering::Less;
        }
        if b.is_parent {
            return std::cmp::Ordering::Greater;
        }
        // Directories always before files
        match (a.is_dir, b.is_dir) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        let ord = match column {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Size => a.size.cmp(&b.size),
            SortColumn::Date => {
                let a_time = a.modified.unwrap_or(SystemTime::UNIX_EPOCH);
                let b_time = b.modified.unwrap_or(SystemTime::UNIX_EPOCH);
                a_time.cmp(&b_time)
            }
        };

        if ascending {
            ord
        } else {
            ord.reverse()
        }
    });
}

fn format_permissions(mode: u32, is_dir: bool) -> String {
    let mut s = String::with_capacity(10);
    s.push(if is_dir { 'd' } else { '-' });

    let flags = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];

    for (bit, ch) in flags {
        s.push(if mode & bit != 0 { ch } else { '-' });
    }

    s
}

fn format_human_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.1}G", b / GB)
    } else if b >= MB {
        format!("{:.1}M", b / MB)
    } else if b >= KB {
        format!("{:.1}K", b / KB)
    } else {
        format!("{}B", bytes)
    }
}

fn format_date(time: SystemTime) -> String {
    let datetime: chrono::DateTime<Local> = time.into();
    let today = Local::now().date_naive();
    let file_date = datetime.date_naive();

    let yesterday = today
        .pred_opt()
        .unwrap_or(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());

    if file_date == today {
        format!("TODAY {}", datetime.format("%H:%M"))
    } else if file_date == yesterday {
        "YESTERDAY".to_string()
    } else {
        let month = match datetime.month() {
            1 => "JAN",
            2 => "FEB",
            3 => "MAR",
            4 => "APR",
            5 => "MAY",
            6 => "JUN",
            7 => "JUL",
            8 => "AUG",
            9 => "SEP",
            10 => "OCT",
            11 => "NOV",
            12 => "DEC",
            _ => "???",
        };
        format!("{} {} {}", month, datetime.day(), datetime.format("%H:%M"))
    }
}
