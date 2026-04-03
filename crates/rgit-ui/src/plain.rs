use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::git;
use crate::shared::*;

/// Print the plain page (serve raw file content).
pub fn print_plain(ctx: &mut CgitContext) {
    let repo_idx = ctx.repo.unwrap();
    let repo_path = ctx.repolist.repos[repo_idx].path.clone().unwrap_or_default();

    let gix_repo = git::open_repo(&repo_path);

    // Resolve default branch
    if let Some(ref gix_repo) = gix_repo {
        let defbranch = ctx.repolist.repos[repo_idx].defbranch.clone();
        if let Some(branch) = git::refs::find_default_branch(gix_repo, defbranch.as_deref()) {
            if ctx.repolist.repos[repo_idx].defbranch.is_none() {
                ctx.repolist.repos[repo_idx].defbranch = Some(branch.clone());
            }
            if ctx.qry.head.is_none() {
                ctx.qry.head = Some(branch);
            }
        }
    }

    let head = ctx.qry.head.clone().unwrap_or_else(|| "master".to_string());
    let path = ctx.qry.path.clone().unwrap_or_default();

    let gix_repo = match gix_repo {
        Some(r) => r,
        None => {
            print_error_page(ctx, 500, "Internal server error", "Cannot open repository");
            return;
        }
    };

    if path.is_empty() {
        // Directory listing at root
        let entries = git::tree::list_tree(&gix_repo, &head, None);
        print_dir_listing(ctx, &head, None, entries);
        return;
    }

    // Try as blob first
    if let Some((_oid, data)) = git::tree::read_blob(&gix_repo, &head, &path) {
        print_blob_plain(ctx, &path, &data);
        return;
    }

    // Try as directory
    let entries = git::tree::list_tree(&gix_repo, &head, Some(&path));
    if let Some(ref e) = entries {
        if !e.is_empty() {
            print_dir_listing(ctx, &head, Some(&path), entries);
            return;
        }
    }

    print_error_page(ctx, 404, "Not found", "Not found");
}

fn print_blob_plain(ctx: &mut CgitContext, path: &str, data: &[u8]) {
    let is_binary = data.iter().take(8000).any(|&b| b == 0);
    let basename = path.rsplit('/').next().unwrap_or(path);

    // Determine mimetype from filename
    ctx.page.mimetype = guess_mimetype(basename, is_binary);

    // Security headers when html serving is disabled
    if ctx.cfg.enable_html_serving == 0 {
        // Block dangerous mimetypes
        if ctx.page.mimetype.starts_with("text/") || ctx.page.mimetype.starts_with("application/") {
            if ctx.page.mimetype != "application/pdf" {
                ctx.page.mimetype = if is_binary {
                    "application/octet-stream".to_string()
                } else {
                    "text/plain".to_string()
                };
            }
        }
    }

    if ctx.cfg.enable_html_serving == 0 {
        ctx.page.extra_headers.push("X-Content-Type-Options: nosniff".to_string());
        ctx.page.extra_headers.push("Content-Security-Policy: default-src 'none'".to_string());
    }

    print_http_headers(ctx);

    html_raw(data);
}

fn print_dir_listing(ctx: &mut CgitContext, head: &str, path: Option<&str>, entries: Option<Vec<git::tree::TreeEntry>>) {
    let entries = match entries {
        Some(e) => e,
        None => {
            print_error_page(ctx, 404, "Not found", "Path not found");
            return;
        }
    };

    ctx.page.mimetype = "text/html".to_string();
    print_http_headers(ctx);

    html("<html><head><title>");
    html_txt(&ctx.repolist.repos[ctx.repo.unwrap()].name);
    html(" - ");
    html_txt(path.unwrap_or("/"));
    html("</title></head>\n<body>\n<ul>\n");

    // Parent directory link
    if path.is_some() {
        let parent = path.and_then(|p| {
            if let Some(pos) = p.rfind('/') {
                Some(&p[..pos])
            } else {
                None
            }
        });
        html("<li><a href='");
        repolink(ctx, None, None, Some("plain"), Some(head), parent);
        html("'>../</a></li>\n");
    }

    for entry in &entries {
        html("<li><a href='");
        let fullpath = if let Some(p) = path {
            format!("{}/{}", p, entry.name)
        } else {
            entry.name.clone()
        };
        repolink(ctx, None, None, Some("plain"), Some(head), Some(&fullpath));
        html("'>");
        html_txt(&entry.name);
        if entry.is_dir {
            html("/");
        }
        html("</a></li>\n");
    }

    html("</ul>\n</body></html>\n");
}

fn guess_mimetype(filename: &str, is_binary: bool) -> String {
    if let Some(ext) = filename.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "html" | "htm" => return "text/html".to_string(),
            "css" => return "text/css".to_string(),
            "js" => return "application/javascript".to_string(),
            "json" => return "application/json".to_string(),
            "xml" => return "text/xml".to_string(),
            "svg" => return "image/svg+xml".to_string(),
            "png" => return "image/png".to_string(),
            "jpg" | "jpeg" => return "image/jpeg".to_string(),
            "gif" => return "image/gif".to_string(),
            "ico" => return "image/x-icon".to_string(),
            "pdf" => return "application/pdf".to_string(),
            "txt" | "md" | "rst" => return "text/plain".to_string(),
            "c" | "h" | "rs" | "py" | "rb" | "pl" | "sh" | "java" | "go" | "cpp" | "hpp" => {
                return "text/plain".to_string()
            }
            "woff" => return "font/woff".to_string(),
            "woff2" => return "font/woff2".to_string(),
            "ttf" => return "font/ttf".to_string(),
            "zip" => return "application/zip".to_string(),
            "gz" | "tgz" => return "application/gzip".to_string(),
            "tar" => return "application/x-tar".to_string(),
            _ => {}
        }
    }
    if is_binary {
        "application/octet-stream".to_string()
    } else {
        "text/plain".to_string()
    }
}
