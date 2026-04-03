use rgit_core::context::CgitContext;
use rgit_core::git;
use crate::shared::*;
use std::io::Write;
use std::process::Command;

/// Print the rawdiff page (plain text diff output).
pub fn print_rawdiff(ctx: &mut CgitContext) {
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

    ctx.page.mimetype = "text/plain".to_string();
    print_http_headers(ctx);

    let stdout_handle = std::io::stdout();
    let mut stdout = stdout_handle.lock();

    if let Some(ref old) = old_oid {
        // Normal diff between two commits
        let output = Command::new("git")
            .arg("--git-dir").arg(&repo_path)
            .arg("diff")
            .arg(&format!("{}..{}", old, new_oid))
            .output();
        if let Ok(o) = output {
            let _ = stdout.write_all(&o.stdout);
        }
    } else {
        // Root commit: use diff-tree
        let output = Command::new("git")
            .arg("--git-dir").arg(&repo_path)
            .arg("diff-tree")
            .arg("-p")
            .arg("--no-commit-id")
            .arg("--root")
            .arg(&new_oid)
            .output();
        if let Ok(o) = output {
            let _ = stdout.write_all(&o.stdout);
        }
    }
}
