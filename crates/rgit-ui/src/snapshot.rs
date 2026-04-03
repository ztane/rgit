use rgit_core::context::CgitContext;
use rgit_core::snapshot::SNAPSHOT_FORMATS;
use crate::shared::*;
use std::io::Write;
use std::process::{Command, Stdio};

/// Print the snapshot page (serves an archive).
pub fn print_snapshot(ctx: &mut CgitContext) {
    let repo_idx = ctx.repo.unwrap();
    let repo_path = ctx.repolist.repos[repo_idx].path.clone().unwrap_or_default();
    let repo_url = ctx.repolist.repos[repo_idx].url.clone();

    let filename = match ctx.qry.path.as_deref() {
        Some(f) if !f.is_empty() => f.to_string(),
        _ => {
            print_error_page(ctx, 400, "Bad request", "No snapshot name specified");
            return;
        }
    };

    // Find matching format
    let format = SNAPSHOT_FORMATS.iter().find(|f| filename.ends_with(f.suffix));
    let format = match format {
        Some(f) => f,
        None => {
            print_error_page(ctx, 400, "Bad request",
                &format!("Unsupported snapshot format: {}", filename));
            return;
        }
    };

    // Check if this format is enabled for the repo
    let repo_snapshots = ctx.repolist.repos[repo_idx].snapshots;
    if repo_snapshots & (format.bit as i32) == 0 {
        print_error_page(ctx, 400, "Bad request",
            &format!("Unsupported snapshot format: {}", filename));
        return;
    }

    // Determine the revision and prefix
    let (rev, prefix) = if let Some(ref oid) = ctx.qry.oid {
        // Explicit revision specified
        let prefix = snapshot_prefix(&repo_url);
        (oid.clone(), prefix)
    } else {
        // DWIM: guess revision from filename
        match get_ref_from_filename(&repo_path, &filename, format.suffix) {
            Some((rev, prefix)) => (rev, prefix),
            None => {
                print_error_page(ctx, 404, "Not found", "Not found");
                return;
            }
        }
    };

    // Emit HTTP headers
    ctx.page.mimetype = format.mimetype.to_string();
    ctx.page.filename = Some(filename.clone());
    print_http_headers(ctx);

    // Generate the archive
    write_archive(&repo_path, &rev, &prefix, format.suffix);
}

/// Get the snapshot prefix (directory name in archive) for a repo.
fn snapshot_prefix(repo_url: &str) -> String {
    let mut s = repo_url.to_string();
    // Strip trailing slashes
    while s.ends_with('/') {
        s.pop();
    }
    // Strip trailing .git
    if s.ends_with(".git") {
        s.truncate(s.len() - 4);
    }
    // Strip trailing slashes again
    while s.ends_with('/') {
        s.pop();
    }
    // Take last component
    if let Some(pos) = s.rfind('/') {
        s[pos + 1..].to_string()
    } else {
        s
    }
}

/// Try to guess the revision from the filename (DWIM mode).
/// Returns (rev, prefix) on success.
fn get_ref_from_filename(repo_path: &str, filename: &str, suffix: &str) -> Option<(String, String)> {
    let base = &filename[..filename.len() - suffix.len()];

    // Try the base name directly as a ref
    if rev_exists(repo_path, base) {
        return Some((base.to_string(), base.to_string()));
    }

    // TODO: strip repo basename prefix and try again, also try v/V prefix
    None
}

/// Check if a revision exists in the repo.
fn rev_exists(repo_path: &str, rev: &str) -> bool {
    Command::new("git")
        .arg("--git-dir").arg(repo_path)
        .arg("rev-parse")
        .arg("--verify")
        .arg(rev)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Write an archive to stdout.
fn write_archive(repo_path: &str, rev: &str, prefix: &str, suffix: &str) {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    match suffix {
        ".zip" => {
            // git archive --format=zip
            let output = Command::new("git")
                .arg("--git-dir").arg(repo_path)
                .arg("archive")
                .arg("--format=zip")
                .arg(&format!("--prefix={}/", prefix))
                .arg(rev)
                .output();
            if let Ok(o) = output {
                let _ = stdout.write_all(&o.stdout);
            }
        }
        ".tar" => {
            let output = Command::new("git")
                .arg("--git-dir").arg(repo_path)
                .arg("archive")
                .arg("--format=tar")
                .arg(&format!("--prefix={}/", prefix))
                .arg(rev)
                .output();
            if let Ok(o) = output {
                let _ = stdout.write_all(&o.stdout);
            }
        }
        _ => {
            // Compressed tar: pipe git archive through compressor
            let compressor = match suffix {
                ".tar.gz" => vec!["gzip", "-n"],
                ".tar.bz2" => vec!["bzip2"],
                ".tar.lz" => vec!["lzip"],
                ".tar.xz" => vec!["xz"],
                ".tar.zst" => vec!["zstd", "-T0"],
                _ => return,
            };

            let archive = Command::new("git")
                .arg("--git-dir").arg(repo_path)
                .arg("archive")
                .arg("--format=tar")
                .arg(&format!("--prefix={}/", prefix))
                .arg(rev)
                .stdout(Stdio::piped())
                .spawn();

            let archive = match archive {
                Ok(a) => a,
                Err(_) => return,
            };

            let compress = Command::new(compressor[0])
                .args(&compressor[1..])
                .stdin(archive.stdout.unwrap())
                .output();

            if let Ok(o) = compress {
                let _ = stdout.write_all(&o.stdout);
            }
        }
    }
}
