use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::git;
use crate::shared::*;

/// Print the refs page (branches and tags listing).
pub fn print_refs(ctx: &mut CgitContext) {
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

    let gix_repo = match gix_repo {
        Some(r) => r,
        None => {
            print_error_page(ctx, 500, "Internal server error", "Cannot open repository");
            return;
        }
    };

    ctx.page.title = Some(format!("{} - {} - refs", ctx.cfg.root_title, ctx.repolist.repos[repo_idx].name));
    print_layout_start(ctx);

    let path = ctx.qry.path.as_deref().unwrap_or("");
    let show_branches = path.is_empty() || path == "heads";
    let show_tags = path.is_empty() || path == "tags";

    if show_branches {
        let mut branches = git::refs::list_branches(&gix_repo);
        branches.sort_by(|a, b| {
            let a_date = a.commit.as_ref().map(|c| c.committer_date).unwrap_or(0);
            let b_date = b.commit.as_ref().map(|c| c.committer_date).unwrap_or(0);
            b_date.cmp(&a_date)
        });

        html("<table summary='branches' class='list nowrap'>");
        html("<tr class='nohover'><th class='left'>Branch</th>");
        html("<th class='left'>Commit message</th>");
        html("<th class='left'>Author</th>");
        html("<th class='left' colspan='2'>Age</th></tr>\n");

        for branch in &branches {
            html("<tr><td>");
            reporevlink(ctx, "log", &branch.name, None, None, Some(&branch.name), None, None);
            html("</td><td>");
            if let Some(ref commit) = branch.commit {
                commit_link(ctx, &commit.subject, None, None, Some(&branch.name), Some(&commit.oid), None);
            }
            html("</td><td>");
            if let Some(ref commit) = branch.commit {
                html_txt(&commit.author);
            }
            html("</td><td colspan='2'>");
            if let Some(ref commit) = branch.commit {
                print_age(commit.committer_date, commit.committer_tz, -1);
            }
            html("</td></tr>\n");
        }

        html("</table>");
    }

    if show_tags {
        let mut tags = git::refs::list_tags(&gix_repo);
        tags.sort_by(|a, b| {
            let a_date = if a.tagger_date > 0 {
                a.tagger_date
            } else {
                a.commit.as_ref().map(|c| c.committer_date).unwrap_or(0)
            };
            let b_date = if b.tagger_date > 0 {
                b.tagger_date
            } else {
                b.commit.as_ref().map(|c| c.committer_date).unwrap_or(0)
            };
            b_date.cmp(&a_date)
        });

        if !tags.is_empty() {
            html("<table summary='tags' class='list nowrap'>");
            html("<tr class='nohover'><th class='left'>Tag</th>");
            html("<th class='left'>Download</th>");
            html("<th class='left'>Author</th>");
            html("<th class='left' colspan='2'>Age</th></tr>\n");

            let repo_snapshots = ctx.repolist.repos[repo_idx].snapshots;

            for tag in &tags {
                html("<tr><td>");
                reporevlink(ctx, "tag", &tag.name, None, None, None, Some(&tag.tagged_oid), None);
                html("</td><td>");
                if repo_snapshots != 0 {
                    print_snapshot_links(ctx, &tag.name);
                }
                html("</td><td>");
                if let Some(ref tagger) = tag.tagger {
                    html_txt(tagger);
                } else if let Some(ref commit) = tag.commit {
                    html_txt(&commit.author);
                }
                html("</td><td colspan='2'>");
                let date = if tag.tagger_date > 0 {
                    tag.tagger_date
                } else {
                    tag.commit.as_ref().map(|c| c.committer_date).unwrap_or(0)
                };
                if date > 0 {
                    print_age(date, 0, -1);
                }
                html("</td></tr>\n");
            }

            html("</table>");
        }
    }

    print_layout_end(ctx);
}

fn print_snapshot_links(ctx: &CgitContext, tag_name: &str) {
    use rgit_core::snapshot::SNAPSHOT_FORMATS;
    let repo_idx = ctx.repo.unwrap();
    let repo_snapshots = ctx.repolist.repos[repo_idx].snapshots;

    let mut first = true;
    for fmt in SNAPSHOT_FORMATS {
        if repo_snapshots & (fmt.bit as i32) == 0 {
            continue;
        }
        if !first {
            html(" ");
        }
        first = false;
        let filename = format!("{}{}", tag_name, fmt.suffix);
        reporevlink(ctx, "snapshot", &fmt.suffix[1..], None, None, None, None, Some(&filename));
    }
}
