use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::Instant;
use std::thread;

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

pub fn start_transfer(src: PathBuf, dst: PathBuf, is_cut: bool, info_mutex: Arc<Mutex<TransferInfo>>) -> Arc<AtomicBool> {
    let abort_flag = Arc::new(AtomicBool::new(false));
    let abort_clone = abort_flag.clone();
    
    thread::spawn(move || {
        let start_time = Instant::now();
        let mut last_update = start_time;
        // Total bytes moving within a single 100ms cycle
        let mut bytes_since_last_update: u64 = 0;
        
        fn copy_recursive(
            src: &Path, 
            dst: &Path, 
            is_cut: bool, 
            abort: &Arc<AtomicBool>, 
            info_mutex: &Arc<Mutex<TransferInfo>>,
            last_update: &mut Instant,
            bytes_since_last_update: &mut u64
        ) -> bool {
            if abort.load(Ordering::SeqCst) {
                return false;
            }
            if src.is_dir() {
                fs::create_dir_all(dst).unwrap_or(());
                if let Ok(entries) = fs::read_dir(src) {
                    for entry in entries.flatten() {
                        let obj_name = entry.file_name();
                        let new_dst = dst.join(obj_name);
                        if !copy_recursive(&entry.path(), &new_dst, is_cut, abort, info_mutex, last_update, bytes_since_last_update) {
                            return false;
                        }
                    }
                }
                if is_cut {
                    let _ = fs::remove_dir(src);
                }
            } else if src.is_file() {
                use std::io::{Read, Write};
                if let Ok(mut sf) = fs::File::open(src) {
                    if let Ok(mut df) = fs::File::create(dst) {
                        let mut buffer = [0; 1024 * 512]; // 512KB chunks
                        loop {
                            if abort.load(Ordering::SeqCst) {
                                return false;
                            }
                            match sf.read(&mut buffer) {
                                Ok(0) => break, // EOF
                                Ok(n) => {
                                    if df.write_all(&buffer[..n]).is_err() {
                                        break;
                                    }
                                    *bytes_since_last_update += n as u64;
                                    let now = Instant::now();
                                    
                                    let elapsed = now.duration_since(*last_update).as_secs_f64();
                                    if elapsed >= 0.1 { // Tick every 100ms
                                        let mbs = (*bytes_since_last_update as f64 / 1_048_576.0) / elapsed;
                                        
                                        let mut info = info_mutex.lock().unwrap();
                                        info.bytes_moved += *bytes_since_last_update;
                                        info.current_mbs = mbs;
                                        
                                        info.history.push(mbs);
                                        if info.history.len() > 15 {
                                            info.history.remove(0);
                                        }
                                        
                                        *bytes_since_last_update = 0;
                                        *last_update = now;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                        
                        let mut info = info_mutex.lock().unwrap();
                        info.files_moved += 1;
                    }
                }
                if is_cut {
                    let _ = fs::remove_file(src);
                }
            }
            true
        }
        
        copy_recursive(&src, &dst, is_cut, &abort_clone, &info_mutex, &mut last_update, &mut bytes_since_last_update);
        
        {
            let mut info = info_mutex.lock().unwrap();
            info.bytes_moved += bytes_since_last_update;
            info.is_finished = true;
            info.current_mbs = 0.0;
        }
    });
    
    abort_flag
}
