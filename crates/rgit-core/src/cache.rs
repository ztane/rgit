use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use crate::html::HtmlOutput;

/// FNV-1 hash matching C cgit's hash_str().
fn hash_str(s: &str) -> u32 {
    const FNV_OFFSET: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;
    let mut h = FNV_OFFSET;
    for b in s.bytes() {
        h = h.wrapping_mul(FNV_PRIME);
        h ^= b as u32;
    }
    h
}

/// Compute the cache filename for a given key and cache size.
fn cache_filename(path: &str, key: &str, size: i32) -> PathBuf {
    let mut hash = hash_str(key) % (size as u32);
    let mut name = String::with_capacity(8);
    for _ in 0..8 {
        name.push(char::from_digit(hash & 0xf, 16).unwrap());
        hash >>= 4;
    }
    let mut p = PathBuf::from(path);
    if !p.as_os_str().is_empty() {
        p.push(name);
    }
    p
}

/// Process a request with caching.
/// If the cache contains a valid entry for this key, serve it.
/// Otherwise, call `generate_fn` to produce the output, cache it, and serve it.
pub fn cache_process<F>(size: i32, path: &str, key: Option<&str>, ttl: i32, generate_fn: F)
where
    F: FnOnce(),
{
    let key = key.unwrap_or("");

    // If caching is disabled, just generate
    if size <= 0 || ttl == 0 || path.is_empty() {
        generate_fn();
        return;
    }

    let cache_file = cache_filename(path, key, size);
    let lock_file = {
        let mut l = cache_file.clone().into_os_string();
        l.push(".lock");
        PathBuf::from(l)
    };

    // Try to read existing cache
    if let Ok(data) = fs::read(&cache_file) {
        // Find the key in the cache file (key + NUL + content)
        if let Some(nul_pos) = data.iter().position(|&b| b == 0) {
            if &data[..nul_pos] == key.as_bytes() {
                // Key matches - check TTL
                let expired = if ttl < 0 {
                    false
                } else if let Ok(metadata) = fs::metadata(&cache_file) {
                    if let Ok(modified) = metadata.modified() {
                        let age = modified.elapsed().unwrap_or_default();
                        age.as_secs() > (ttl as u64) * 60
                    } else {
                        true
                    }
                } else {
                    true
                };

                if !expired {
                    // Serve from cache
                    let stdout = io::stdout();
                    let mut stdout = stdout.lock();
                    let _ = stdout.write_all(&data[nul_pos + 1..]);
                    return;
                }
            }
        }
    }

    // Generate the content with capture
    HtmlOutput::start_capture();
    generate_fn();
    let output = HtmlOutput::stop_capture();

    // Write to cache file via lock file
    // Ensure cache directory exists
    let _ = fs::create_dir_all(path);

    if let Ok(mut f) = fs::File::create(&lock_file) {
        let _ = f.write_all(key.as_bytes());
        let _ = f.write_all(&[0]);
        let _ = f.write_all(&output);
        let _ = f.flush();
        // Atomically replace cache file
        let _ = fs::rename(&lock_file, &cache_file);
    }

    // Write output to stdout
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let _ = stdout.write_all(&output);
}

/// List cache entries (for ls_cache command).
pub fn cache_ls(path: &str) {
    let dir = match fs::read_dir(path) {
        Ok(d) => d,
        Err(_) => return,
    };

    let mut entries: Vec<_> = dir.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // Only show 8-character filenames (cache slots)
        if name_str.len() != 8 {
            continue;
        }

        let full_path = entry.path();
        let metadata = match fs::metadata(&full_path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        // Read the key from the cache file
        let mut file = match fs::File::open(&full_path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let mut buf = vec![0u8; 4096];
        let n = match file.read(&mut buf) {
            Ok(n) => n,
            Err(_) => continue,
        };
        buf.truncate(n);

        let key = if let Some(nul_pos) = buf.iter().position(|&b| b == 0) {
            String::from_utf8_lossy(&buf[..nul_pos]).to_string()
        } else {
            String::new()
        };

        // Format modified time
        let mtime_str = if let Ok(modified) = metadata.modified() {
            let duration = modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            format_datetime(duration.as_secs() as i64)
        } else {
            "                   ".to_string()
        };

        crate::html::html(&format!(
            "{}/{} {} {:>10} {}\n",
            path,
            name_str,
            mtime_str,
            metadata.len(),
            key
        ));
    }
}

fn format_datetime(timestamp: i64) -> String {
    let secs = ((timestamp % 86400) + 86400) % 86400;
    let mut days = timestamp / 86400;
    if timestamp % 86400 < 0 {
        days -= 1;
    }

    let hour = secs / 3600;
    let min = (secs % 3600) / 60;
    let sec = secs % 60;

    let mut y = 1970i64;
    loop {
        let diy = if is_leap(y) { 366 } else { 365 };
        if days < diy {
            break;
        }
        days -= diy;
        y += 1;
    }

    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y,
        month + 1,
        days + 1,
        hour,
        min,
        sec
    )
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
