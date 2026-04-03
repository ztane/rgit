use rgit_core::context::CgitContext;
use rgit_core::html::html_raw;
use rgit_core::git;
use crate::shared::*;

const CGIT_VERSION: &str = "v1.3";

/// Print the patch page (format-patch output).
pub fn print_patch(ctx: &mut CgitContext) {
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
    let new_rev = ctx.qry.oid.clone().unwrap_or_else(|| head.clone());
    let old_rev = ctx.qry.oid2.clone();

    // Resolve new_rev to OID
    let new_oid = gix_repo.as_ref()
        .and_then(|r| git::diff::resolve_rev(r, &new_rev))
        .unwrap_or_else(|| new_rev.clone());

    // Resolve old_rev to OID, or get parent
    let old_oid = if let Some(ref old) = old_rev {
        gix_repo.as_ref().and_then(|r| git::diff::resolve_rev(r, old))
    } else {
        gix_repo.as_ref().and_then(|r| git::diff::get_parent(r, &new_oid))
    };

    // Build rev range for filename
    let rev_range = if let Some(ref old) = old_oid {
        format!("{}..{}", old, new_oid)
    } else {
        new_oid.clone()
    };

    ctx.page.mimetype = "text/plain".to_string();
    ctx.page.filename = Some(format!("{}.patch", rev_range));
    print_http_headers(ctx);

    // Use git format-patch to generate the output
    let mut cmd = git::git_command(&repo_path);
    cmd.arg("format-patch");
    cmd.arg("--stdout");
    cmd.arg("-N");
    cmd.arg("--subject-prefix=");
    cmd.arg(&format!("--signature=cgit {}", CGIT_VERSION));

    if let Some(ref old) = old_oid {
        cmd.arg(&format!("{}..{}", old, new_oid));
    } else {
        // Root commit: format-patch needs --root
        cmd.arg("--root");
        cmd.arg(&new_oid);
    };

    let output = cmd.output();
    if let Ok(o) = output {
        html_raw(&o.stdout);
    }
}
