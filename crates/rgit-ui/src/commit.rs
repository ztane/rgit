use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::filter;
use rgit_core::git;
use crate::shared::*;
use crate::diff::print_diff_body;

/// Print the commit detail page.
pub fn print_commit(ctx: &mut CgitContext) {
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
    let rev = ctx.qry.oid.clone().unwrap_or_else(|| head.clone());

    // Get commit info
    let gix_repo = match gix_repo {
        Some(r) => r,
        None => {
            print_error_page(ctx, 500, "Internal server error", "Cannot open repository");
            return;
        }
    };

    let commits = git::commit::walk_log(&gix_repo, &rev, 1, 0);
    let commit = match commits.first() {
        Some(c) => c.clone(),
        None => {
            print_error_page(ctx, 404, "Not found", "Bad commit reference");
            return;
        }
    };

    let parent = git::diff::get_parent(&gix_repo, &rev);
    let tree_oid = git::diff::get_commit_tree(&gix_repo, &rev);

    ctx.page.title = Some(format!("{} - {} - commit {}", commit.subject, ctx.repolist.repos[repo_idx].name, &commit.oid[..8]));
    print_layout_start(ctx);

    // Commit info table
    html("<table summary='commit info' class='commit-info'>\n");

    let email_filter = ctx.repolist.repos[repo_idx].email_filter.clone();

    // Author
    html("<tr><th>author</th><td>");
    {
        let author = commit.author.clone();
        let author_email = commit.author_email.clone();
        let noplainemail = ctx.cfg.noplainemail;
        filter::with_filter(email_filter.as_deref(), &[&author_email, "commit"], || {
            html_txt(&author);
            if noplainemail == 0 {
                html(" ");
                html_txt(&author_email);
            }
        });
    }
    html("</td><td class='right'>");
    html_txt(&format_iso8601_full(commit.author_date, commit.author_tz));
    html("</td></tr>\n");

    // Committer
    html("<tr><th>committer</th><td>");
    {
        let committer = commit.committer.clone();
        let committer_email = commit.committer_email.clone();
        let noplainemail = ctx.cfg.noplainemail;
        filter::with_filter(email_filter.as_deref(), &[&committer_email, "commit"], || {
            html_txt(&committer);
            if noplainemail == 0 {
                html(" ");
                html_txt(&committer_email);
            }
        });
    }
    html("</td><td class='right'>");
    html_txt(&format_iso8601_full(commit.committer_date, commit.committer_tz));
    html("</td></tr>\n");

    // Commit OID
    html("<tr><th>commit</th><td colspan='2' class='oid'>");
    commit_link(ctx, &commit.oid, None, None, ctx.qry.head.as_deref(), Some(&commit.oid), ctx.qry.vpath.as_deref());
    html(" (");
    reporevlink(ctx, "patch", "patch", None, None, ctx.qry.head.as_deref(), Some(&commit.oid), None);
    html(")</td></tr>\n");

    // Tree
    html("<tr><th>tree</th><td colspan='2' class='oid'>");
    if let Some(tree) = &tree_oid {
        reporevlink(ctx, "tree", tree, None, None, ctx.qry.head.as_deref(), Some(&rev), None);
    }
    html("</td></tr>\n");

    // Parent
    if let Some(parent_oid) = &parent {
        html("<tr><th>parent</th><td colspan='2' class='oid'>");
        commit_link(ctx, parent_oid, None, None, ctx.qry.head.as_deref(), Some(parent_oid), ctx.qry.vpath.as_deref());
        html(" (");
        reporevlink(ctx, "diff", "diff", None, None, ctx.qry.head.as_deref(), Some(&commit.oid), None);
        html(")</td></tr>");
    }

    html("</table>\n");

    let commit_filter = ctx.repolist.repos[repo_idx].commit_filter.clone();

    // Subject and message
    html("<div class='commit-subject'>");
    {
        let subject = commit.subject.clone();
        filter::with_filter(commit_filter.as_deref(), &[], || {
            html_txt(&subject);
        });
    }
    html("</div>");
    html("<div class='commit-msg'>");
    {
        let msg = commit.msg.clone();
        filter::with_filter(commit_filter.as_deref(), &[], || {
            html_txt(&msg);
        });
    }
    html("</div>");

    // Diff
    print_diff_body(ctx, &repo_path, &commit.oid, parent.as_deref(), ctx.qry.vpath.as_deref());

    print_layout_end(ctx);
}
