use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct TransferInfo {
    pub src: String,
    pub dst: String,
    pub bytes_moved: u64,
    pub bytes_total: u64,
    pub files_moved: usize,
    pub files_total: usize,
    pub history: Vec<f64>, // MB/s over last 15 ticks
    pub current_mbs: f64,
    pub is_finished: bool,
    pub is_cut: bool,
    pub current_file: String,
    pub status: String,
    pub error: Option<String>,
}

pub fn get_size_and_count(path: &Path) -> (u64, usize) {
    let mut size = 0;
    let mut count = 0;
    if path.is_file() {
        if let Ok(meta) = path.metadata() {
            size += meta.len();
            count += 1;
        }
    } else if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let (s, c) = get_size_and_count(&entry.path());
                size += s;
                count += c;
            }
        }
    }
    (size, count)
}

pub fn start_transfer(
    src: PathBuf,
    dst: PathBuf,
    is_cut: bool,
    info_mutex: Arc<Mutex<TransferInfo>>,
) -> Arc<AtomicBool> {
    let abort_flag = Arc::new(AtomicBool::new(false));
    let abort_clone = abort_flag.clone();

    thread::spawn(move || {
        let mut last_update = Instant::now();
        let mut bytes_since_last_update: u64 = 0;

        fn copy_recursive(
            src: &Path,
            dst: &Path,
            is_cut: bool,
            abort: &Arc<AtomicBool>,
            info_mutex: &Arc<Mutex<TransferInfo>>,
            last_update: &mut Instant,
            bytes_since_last_update: &mut u64,
        ) -> Result<(), String> {
            if abort.load(Ordering::SeqCst) {
                return Err("Transfer aborted".to_string());
            }

            if src.is_dir() {
                fs::create_dir_all(dst)
                    .map_err(|e| format!("Failed to create {}: {}", dst.display(), e))?;
                let entries = fs::read_dir(src)
                    .map_err(|e| format!("Failed to read {}: {}", src.display(), e))?;

                for entry in entries.flatten() {
                    let new_dst = dst.join(entry.file_name());
                    copy_recursive(
                        &entry.path(),
                        &new_dst,
                        is_cut,
                        abort,
                        info_mutex,
                        last_update,
                        bytes_since_last_update,
                    )?;
                }

                if is_cut {
                    fs::remove_dir(src)
                        .map_err(|e| format!("Failed to remove {}: {}", src.display(), e))?;
                }

                return Ok(());
            }

            if !src.is_file() {
                return Ok(());
            }

            use std::io::{Read, Write};

            let mut sf = fs::File::open(src)
                .map_err(|e| format!("Failed to open {}: {}", src.display(), e))?;
            let mut df = fs::File::create(dst)
                .map_err(|e| format!("Failed to create {}: {}", dst.display(), e))?;

            if let Ok(mut info) = info_mutex.lock() {
                info.current_file = src
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                info.status = if is_cut {
                    "Moving".to_string()
                } else {
                    "Copying".to_string()
                };
            }

            let mut buffer = [0; 1024 * 512];
            loop {
                if abort.load(Ordering::SeqCst) {
                    return Err("Transfer aborted".to_string());
                }

                match sf.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        df.write_all(&buffer[..n])
                            .map_err(|e| format!("Failed writing {}: {}", dst.display(), e))?;

                        *bytes_since_last_update += n as u64;
                        let now = Instant::now();
                        let elapsed = now.duration_since(*last_update).as_secs_f64();

                        if elapsed >= 0.1 {
                            let mbs = (*bytes_since_last_update as f64 / 1_048_576.0) / elapsed;
                            if let Ok(mut info) = info_mutex.lock() {
                                info.bytes_moved += *bytes_since_last_update;
                                info.current_mbs = mbs;
                                info.history.push(mbs);
                                if info.history.len() > 15 {
                                    info.history.remove(0);
                                }
                            }
                            *bytes_since_last_update = 0;
                            *last_update = now;
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed reading {}: {}", src.display(), e));
                    }
                }
            }

            if let Ok(mut info) = info_mutex.lock() {
                info.files_moved += 1;
            }

            if is_cut {
                fs::remove_file(src)
                    .map_err(|e| format!("Failed removing {}: {}", src.display(), e))?;
            }

            Ok(())
        }

        let result = copy_recursive(
            &src,
            &dst,
            is_cut,
            &abort_clone,
            &info_mutex,
            &mut last_update,
            &mut bytes_since_last_update,
        );

        if let Ok(mut info) = info_mutex.lock() {
            info.bytes_moved += bytes_since_last_update;
            info.current_mbs = 0.0;
            info.is_finished = true;
            match result {
                Ok(()) => info.status = "Completed".to_string(),
                Err(err) if err == "Transfer aborted" => info.status = "Aborted".to_string(),
                Err(err) => {
                    info.status = "Failed".to_string();
                    info.error = Some(err);
                }
            }
        }
    });

    abort_flag
}
