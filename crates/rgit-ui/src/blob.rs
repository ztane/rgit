use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::git;
use crate::shared::*;

/// Print the blob page (serve raw blob content by OID or path).
pub fn print_blob(ctx: &mut CgitContext) {
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
    let path = ctx.qry.path.clone();

    let gix_repo = match gix_repo {
        Some(r) => r,
        None => {
            print_error_page(ctx, 500, "Internal server error", "Cannot open repository");
            return;
        }
    };

    // Try to resolve blob
    let data = if let Some(ref p) = path {
        git::tree::read_blob(&gix_repo, &head, p).map(|(_, d)| d)
    } else {
        None
    };

    let data = match data {
        Some(d) => d,
        None => {
            print_error_page(ctx, 404, "Not found", "Blob not found");
            return;
        }
    };

    let is_binary = data.iter().take(8000).any(|&b| b == 0);
    let filename = path.as_deref()
        .and_then(|p| p.rsplit('/').next())
        .unwrap_or("blob");

    ctx.page.mimetype = if is_binary {
        "application/octet-stream".to_string()
    } else {
        "text/plain".to_string()
    };
    ctx.page.filename = Some(filename.to_string());

    if ctx.cfg.enable_html_serving == 0 {
        ctx.page.extra_headers.push("X-Content-Type-Options: nosniff".to_string());
        ctx.page.extra_headers.push("Content-Security-Policy: default-src 'none'".to_string());
    }

    print_http_headers(ctx);
    html_raw(&data);
}
