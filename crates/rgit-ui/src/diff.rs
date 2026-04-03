use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::git;
use crate::shared::*;

/// Print the standalone diff page.
pub fn print_diff(ctx: &mut CgitContext) {
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
    let rev_input = ctx.qry.oid.clone().unwrap_or_else(|| head.clone());
    let rev = gix_repo.as_ref()
        .and_then(|r| git::diff::resolve_rev(r, &rev_input))
        .unwrap_or(rev_input);
    let old_rev = ctx.qry.oid2.clone().or_else(|| {
        gix_repo.as_ref().and_then(|r| git::diff::get_parent(r, &rev))
    });

    ctx.page.title = Some(format!("{} - diff", ctx.repolist.repos[repo_idx].name));
    print_layout_start(ctx);

    print_diff_body(ctx, &repo_path, &rev, old_rev.as_deref(), ctx.qry.vpath.as_deref());

    print_layout_end(ctx);
}

/// Shared diff rendering used by both commit and diff pages.
pub fn print_diff_body(ctx: &CgitContext, repo_path: &str, new_rev: &str, old_rev: Option<&str>, path: Option<&str>) {
    let diff_stat = git::diff::diff_stat(repo_path, new_rev, old_rev);
    let unified = git::diff::unified_diff(repo_path, new_rev, old_rev, path);

    // Diffstat
    print_diffstat(ctx, &diff_stat, new_rev);

    // Unified diff output
    print_unified_diff(ctx, &unified, new_rev);
}

fn print_diffstat(ctx: &CgitContext, diff: &git::diff::DiffResult, rev: &str) {
    if diff.files.is_empty() {
        return;
    }

    html("<table summary='diffstat' class='diffstat'>");
    for file in &diff.files {
        let class = match file.status {
            'A' => "add",
            'D' => "del",
            'R' => "rename",
            'C' => "copy",
            _ => "upd",
        };
        html(&format!("<tr><td class='mode'>", ));
        match file.status {
            'A' => html("new file mode "),
            'D' => html("deleted file mode "),
            _ => {}
        }
        if file.status == 'A' {
            html_txt(&file.new_mode);
        }
        if file.status == 'D' {
            html_txt(&file.old_mode);
        }
        html("</td>");
        html(&format!("<td class='{}'>", class));
        // Link to the file in the tree
        reporevlink(ctx, "diff", &file.new_path, None, None,
                    ctx.qry.head.as_deref(), Some(rev), Some(&file.new_path));
        html("</td><td class='right'>");
        if !file.binary {
            html(&format!("{}", file.added + file.removed));
        } else {
            html("bin");
        }
        html("</td><td class='graph'>");
        let total = file.added + file.removed;
        if total > 0 && !file.binary {
            let max_width = 30;
            let add_width = (file.added * max_width / total).max(if file.added > 0 { 1 } else { 0 });
            let rem_width = (file.removed * max_width / total).max(if file.removed > 0 { 1 } else { 0 });
            for _ in 0..add_width {
                html("<span class='ins'>+</span>");
            }
            for _ in 0..rem_width {
                html("<span class='del'>-</span>");
            }
        }
        html("</td></tr>\n");
    }
    html("<tr><td colspan='4' class='summary'>");
    html(&format!("{} files changed, {} insertions, {} deletions",
                  diff.files.len(), diff.total_adds, diff.total_rems));
    html("</td></tr>\n");
    html("</table>");
}

fn print_unified_diff(ctx: &CgitContext, diff_output: &str, rev: &str) {
    if diff_output.is_empty() {
        return;
    }

    for line in diff_output.lines() {
        if line.starts_with("diff --git ") {
            html("<div class='head'>");
            html_txt(line);
            html("</div>");
        } else if line.starts_with("index ") {
            html("<div class='head'>");
            html_txt(line);
            html("</div>");
        } else if line.starts_with("new file mode ") || line.starts_with("deleted file mode ") ||
                  line.starts_with("old mode ") || line.starts_with("new mode ") ||
                  line.starts_with("similarity index ") || line.starts_with("rename from ") ||
                  line.starts_with("rename to ") || line.starts_with("copy from ") ||
                  line.starts_with("copy to ") {
            html("<div class='head'>");
            html_txt(line);
            html("</div>");
        } else if line.starts_with("--- ") {
            html("<div class='hunk'>");
            // Link to old file
            if line.starts_with("--- a/") {
                let path = &line[6..];
                html("--- a/");
                reporevlink(ctx, "tree", path, None, None, ctx.qry.head.as_deref(), Some(rev), Some(path));
            } else {
                html_txt(line);
            }
            html("</div>");
        } else if line.starts_with("+++ ") {
            html("<div class='hunk'>");
            if line.starts_with("+++ b/") {
                let path = &line[6..];
                html("+++ b/");
                reporevlink(ctx, "tree", path, None, None, ctx.qry.head.as_deref(), Some(rev), Some(path));
            } else {
                html_txt(line);
            }
            html("</div>");
        } else if line.starts_with("@@ ") {
            html("<div class='hunk'>");
            html_txt(line);
            html("</div>");
        } else if line.starts_with('+') {
            html("<div class='add'>");
            html_txt(line);
            html("</div>");
        } else if line.starts_with('-') {
            html("<div class='del'>");
            html_txt(line);
            html("</div>");
        } else if line.starts_with('\\') {
            html("<div class='ctx'>");
            html_txt(line);
            html("</div>");
        } else {
            html("<div class='ctx'>");
            html_txt(line);
            html("</div>");
        }
    }
}
