use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::git;
use rgit_core::git::commit::CommitInfo;
use crate::shared::*;

/// Print the log page for a repository.
pub fn print_log(ctx: &mut CgitContext) {
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
    let ofs = if ctx.qry.ofs < 0 { 0 } else { ctx.qry.ofs as usize };
    let cnt = ctx.cfg.max_commit_count as usize;

    // Get one extra to detect if there are more
    let all_commits = if let Some(ref gix_repo) = gix_repo {
        git::commit::walk_log(gix_repo, &head, cnt + ofs + 1, 0)
    } else {
        Vec::new()
    };

    // Apply grep filter if specified
    let filtered_commits = apply_grep_filter(ctx, &all_commits);

    // Paginate
    let total = filtered_commits.len();
    let display_commits: Vec<&CommitInfo> = filtered_commits.into_iter().skip(ofs).take(cnt).collect();
    let has_more = total > ofs + cnt;

    ctx.page.title = Some(format!("{} - {} - log", ctx.cfg.root_title, ctx.repolist.repos[repo_idx].name));
    print_layout_start(ctx);

    let repo = &ctx.repolist.repos[repo_idx];
    let mut columns = 3;
    if repo.enable_log_filecount != 0 || ctx.cfg.enable_log_filecount != 0 {
        columns += 1;
    }
    if repo.enable_log_linecount != 0 || ctx.cfg.enable_log_linecount != 0 {
        columns += 1;
    }

    html("<table class='list nowrap'>");

    // Header row
    html("<tr class='nohover'>");
    html("<th class='left'>Age</th>");
    html("<th class='left'>Commit message");
    // Expand/Collapse link
    html(" (");
    cgit_log_link_full(ctx, if ctx.qry.showmsg != 0 { "Collapse" } else { "Expand" },
                       None, None, ctx.qry.head.as_deref(), ctx.qry.oid.as_deref(),
                       ctx.qry.vpath.as_deref(), ctx.qry.ofs,
                       ctx.qry.grep.as_deref(), ctx.qry.search.as_deref(),
                       if ctx.qry.showmsg != 0 { 0 } else { 1 }, ctx.qry.follow);
    html(")");
    html("</th><th class='left'>Author</th>");
    if repo.enable_log_filecount != 0 || ctx.cfg.enable_log_filecount != 0 {
        html("<th class='left'>Files</th>");
    }
    if repo.enable_log_linecount != 0 || ctx.cfg.enable_log_linecount != 0 {
        html("<th class='left'>Lines</th>");
    }
    html("</tr>\n");

    let two_weeks: i64 = 7 * 24 * 60 * 60 * 2;
    for commit in &display_commits {
        let tr_class = if ctx.qry.showmsg != 0 { " class='logheader'" } else { "" };
        html(&format!("<tr{}>", tr_class));
        html("<td>");
        print_age(commit.committer_date, commit.committer_tz, two_weeks);
        html("</td>");

        let td_class = if ctx.qry.showmsg != 0 { " class='logsubject'" } else { "" };
        html(&format!("<td{}>", td_class));
        commit_link(ctx, &commit.subject, None, None, ctx.qry.head.as_deref(), Some(&commit.oid), ctx.qry.vpath.as_deref());
        html("</td><td>");
        html_txt(&commit.author);

        let show_filecount = repo.enable_log_filecount != 0 || ctx.cfg.enable_log_filecount != 0;
        let show_linecount = repo.enable_log_linecount != 0 || ctx.cfg.enable_log_linecount != 0;
        if show_filecount || show_linecount {
            let (files, added, removed) = git::diff::commit_stats(&repo_path, &commit.oid);
            if show_filecount {
                html(&format!("</td><td>{}", files));
            }
            if show_linecount {
                html(&format!("</td><td><span class='deletions'>-{}</span>/<span class='insertions'>+{}</span>", removed, added));
            }
        }

        html("</td></tr>\n");

        if ctx.qry.showmsg != 0 {
            html("<tr class='nohover-highlight'>");
            html("<td/>"); // Empty Age column
            html(&format!("<td colspan='{}' class='logmsg'>\n", columns - 1));
            if !commit.msg.is_empty() {
                html_txt(&commit.msg);
            }
            html("</td></tr>\n");
        }
    }

    // Pager
    html("</table><ul class='pager'>");
    if ofs > 0 {
        html("<li>");
        cgit_log_link_full(ctx, "[prev]", None, None, ctx.qry.head.as_deref(),
                           ctx.qry.oid.as_deref(), ctx.qry.vpath.as_deref(),
                           (ofs as i32) - (cnt as i32), ctx.qry.grep.as_deref(),
                           ctx.qry.search.as_deref(), ctx.qry.showmsg, ctx.qry.follow);
        html("</li>");
    }
    if has_more {
        html("<li>");
        cgit_log_link_full(ctx, "[next]", None, None, ctx.qry.head.as_deref(),
                           ctx.qry.oid.as_deref(), ctx.qry.vpath.as_deref(),
                           (ofs as i32) + (cnt as i32), ctx.qry.grep.as_deref(),
                           ctx.qry.search.as_deref(), ctx.qry.showmsg, ctx.qry.follow);
        html("</li>");
    }
    html("</ul>");

    print_layout_end(ctx);
}

fn apply_grep_filter<'a>(ctx: &CgitContext, commits: &'a [CommitInfo]) -> Vec<&'a CommitInfo> {
    let search = match &ctx.qry.search {
        Some(s) if !s.is_empty() => s,
        _ => return commits.iter().collect(),
    };
    let grep_type = ctx.qry.grep.as_deref().unwrap_or("grep");
    let search_lower = search.to_lowercase();

    commits.iter().filter(|c| {
        match grep_type {
            "grep" => c.subject.to_lowercase().contains(&search_lower)
                || c.msg.to_lowercase().contains(&search_lower),
            "author" => c.author.to_lowercase().contains(&search_lower)
                || c.author_email.to_lowercase().contains(&search_lower),
            "committer" => c.committer.to_lowercase().contains(&search_lower)
                || c.committer_email.to_lowercase().contains(&search_lower),
            _ => true,
        }
    }).collect()
}

/// Full log link with all parameters (matching C cgit's cgit_log_link).
fn cgit_log_link_full(ctx: &CgitContext, name: &str, title: Option<&str>, class: Option<&str>,
                      head: Option<&str>, _rev: Option<&str>, _path: Option<&str>,
                      ofs: i32, grep: Option<&str>, pattern: Option<&str>,
                      showmsg: i32, follow: i32) {
    let delim = repolink(ctx, title, class, Some("log"), head, None);
    if let (Some(g), Some(p)) = (grep, pattern) {
        if !p.is_empty() {
            html(delim);
            html("qt=");
            html_url_arg(g);
            html("&amp;q=");
            html_url_arg(p);
        }
    }
    if ofs > 0 {
        html("&amp;ofs=");
        html(&format!("{}", ofs));
    }
    if showmsg != 0 {
        html("&amp;showmsg=1");
    }
    if follow != 0 {
        html("&amp;follow=1");
    }
    html("'>");
    html_txt(name);
    html("</a>");
}
