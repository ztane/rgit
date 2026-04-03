use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::git;
use crate::shared::print_error_page;
use crate::shared::print_http_headers;

/// Serve HEAD file for dumb HTTP clone.
pub fn print_head(ctx: &mut CgitContext) {
    let repo_idx = ctx.repo.unwrap();
    let repo_path = ctx.repolist.repos[repo_idx].path.clone().unwrap_or_default();
    let head_path = format!("{}/HEAD", repo_path);
    send_file(ctx, &head_path, &repo_path);
}

/// Serve info/refs for dumb HTTP clone.
pub fn print_info(ctx: &mut CgitContext) {
    let path = ctx.qry.path.as_deref();
    if path != Some("refs") {
        print_error_page(ctx, 400, "Bad request", "Bad request");
        return;
    }

    let repo_idx = ctx.repo.unwrap();
    let repo_path = ctx.repolist.repos[repo_idx].path.clone().unwrap_or_default();

    ctx.page.mimetype = "text/plain".to_string();
    ctx.page.filename = Some("info/refs".to_string());
    print_http_headers(ctx);

    // Use git for-each-ref to list refs in info/refs format
    let mut cmd = git::git_command(&repo_path);
    cmd.args(["for-each-ref", "--format=%(objectname)\t%(refname)\n%(*objectname)\t%(refname)^{}"]);
    if let Ok(output) = cmd.output() {
        if output.status.success() {
            // Filter out lines with empty objectname (non-tag refs have no *objectname)
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if !line.starts_with('\t') && !line.is_empty() {
                    html(line);
                    html("\n");
                }
            }
        }
    }
}

/// Serve objects for dumb HTTP clone.
pub fn print_objects(ctx: &mut CgitContext) {
    let path = match ctx.qry.path.as_deref() {
        Some(p) if !p.is_empty() => p,
        _ => {
            print_error_page(ctx, 400, "Bad request", "Bad request");
            return;
        }
    };

    // Handle info/packs
    if path == "info/packs" {
        print_pack_info(ctx);
        return;
    }

    // Validate path: no "..", only alnum, /, ., -
    for (i, c) in path.char_indices() {
        if c == '.' && path.as_bytes().get(i + 1) == Some(&b'.') {
            print_error_page(ctx, 400, "Bad request", "Bad request");
            return;
        }
        if !c.is_alphanumeric() && c != '/' && c != '.' && c != '-' {
            print_error_page(ctx, 400, "Bad request", "Bad request");
            return;
        }
    }

    let repo_idx = ctx.repo.unwrap();
    let repo_path = ctx.repolist.repos[repo_idx].path.clone().unwrap_or_default();
    let full_path = format!("{}/objects/{}", repo_path, path);
    send_file(ctx, &full_path, &repo_path);
}

fn print_pack_info(ctx: &mut CgitContext) {
    let repo_idx = ctx.repo.unwrap();
    let repo_path = ctx.repolist.repos[repo_idx].path.clone().unwrap_or_default();
    let pack_dir = format!("{}/objects/pack", repo_path);

    ctx.page.mimetype = "text/plain".to_string();
    ctx.page.filename = Some("objects/info/packs".to_string());
    print_http_headers(ctx);

    // List .pack files
    if let Ok(entries) = std::fs::read_dir(&pack_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with(".pack") {
                html("P ");
                html_txt(&name_str);
                html("\n");
            }
        }
    }
}

fn send_file(ctx: &mut CgitContext, path: &str, repo_path: &str) {
    // Verify the resolved path stays within the repository directory
    if let (Ok(canonical), Ok(canonical_repo)) =
        (std::fs::canonicalize(path), std::fs::canonicalize(repo_path))
    {
        if !canonical.starts_with(&canonical_repo) {
            print_error_page(ctx, 403, "Forbidden", "Forbidden");
            return;
        }
    }

    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    print_error_page(ctx, 404, "Not found", "Not found");
                }
                std::io::ErrorKind::PermissionDenied => {
                    print_error_page(ctx, 403, "Forbidden", "Forbidden");
                }
                _ => {
                    print_error_page(ctx, 400, "Bad request", "Bad request");
                }
            }
            return;
        }
    };

    if !meta.is_file() {
        print_error_page(ctx, 404, "Not found", "Not found");
        return;
    }

    ctx.page.mimetype = "application/octet-stream".to_string();
    // Set filename as relative path within repo
    let filename = path.strip_prefix(repo_path)
        .map(|p| p.trim_start_matches('/'))
        .unwrap_or(path);
    ctx.page.filename = Some(filename.to_string());
    print_http_headers(ctx);

    if let Ok(data) = std::fs::read(path) {
        html_raw(&data);
    }
}
