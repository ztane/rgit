use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::git;
use rgit_core::git::commit::CommitInfo;
use rgit_core::git::refs::{BranchInfo, TagInfo};
use crate::shared::*;

/// Print the summary page for a repository.
pub fn print_summary(ctx: &mut CgitContext) {
    let repo_idx = ctx.repo.unwrap();
    let repo_path = ctx.repolist.repos[repo_idx].path.clone().unwrap_or_default();

    let gix_repo = git::open_repo(&repo_path);

    // Resolve default branch and set qry.head if not already set
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

    // Pre-fetch all data from the git repo
    let (branches, tags, commits) = if let Some(ref gix_repo) = gix_repo {
        let branches = git::refs::list_branches(gix_repo);
        let tags = git::refs::list_tags(gix_repo);
        let head = ctx.qry.head.as_deref().unwrap_or("master");
        let commits = if ctx.cfg.summary_log > 0 {
            git::commit::walk_log(gix_repo, head, ctx.cfg.summary_log as usize, 0)
        } else {
            Vec::new()
        };
        (branches, tags, commits)
    } else {
        (Vec::new(), Vec::new(), Vec::new())
    };

    ctx.page.title = Some(format!("{} - {}", ctx.cfg.root_title, ctx.repolist.repos[repo_idx].name));
    print_layout_start(ctx);

    let columns = calc_columns(ctx);

    html("<table summary='repository info' class='list nowrap'>");

    // Branches
    print_branches(ctx, &branches, ctx.cfg.summary_branches, columns);

    // Spacer
    html(&format!("<tr class='nohover'><td colspan='{}'>&nbsp;</td></tr>", columns));

    // Tags
    print_tags(ctx, &tags, ctx.cfg.summary_tags, columns);

    // Log
    if ctx.cfg.summary_log > 0 {
        html(&format!("<tr class='nohover'><td colspan='{}'>&nbsp;</td></tr>", columns));
        print_summary_log(ctx, &commits, ctx.cfg.summary_log as usize, columns);
    }

    // Clone URLs
    print_clone_urls(ctx, columns);

    html("</table>");
    print_layout_end(ctx);
}

fn calc_columns(ctx: &CgitContext) -> i32 {
    let repo_idx = ctx.repo.unwrap();
    let repo = &ctx.repolist.repos[repo_idx];
    let mut columns = 3;
    if repo.enable_log_filecount != 0 || ctx.cfg.enable_log_filecount != 0 {
        columns += 1;
    }
    if repo.enable_log_linecount != 0 || ctx.cfg.enable_log_linecount != 0 {
        columns += 1;
    }
    columns
}

fn print_branches(ctx: &CgitContext, branches: &[BranchInfo], max_count: i32, _columns: i32) {
    html("<tr class='nohover'><th class='left'>Branch</th>\
          <th class='left'>Commit message</th>\
          <th class='left'>Author</th>\
          <th class='left' colspan='2'>Age</th></tr>\n");

    let mut branches: Vec<&BranchInfo> = branches.iter().collect();

    // Sort by committer date (most recent first)
    branches.sort_by(|a, b| {
        let date_a = a.commit.as_ref().map(|c| c.committer_date).unwrap_or(0);
        let date_b = b.commit.as_ref().map(|c| c.committer_date).unwrap_or(0);
        date_b.cmp(&date_a)
    });

    let max = if max_count == 0 || max_count as usize > branches.len() {
        branches.len()
    } else {
        max_count as usize
    };

    // If branch_sort == 0, sort the displayed subset by name
    if ctx.cfg.branch_sort == 0 {
        branches[..max].sort_by(|a, b| a.name.cmp(&b.name));
    }

    for branch in &branches[..max] {
        let Some(info) = &branch.commit else { continue };
        html("<tr><td>");
        log_link(ctx, &branch.name, None, None, Some(&branch.name));
        html("</td><td>");
        commit_link(ctx, &info.subject, None, None, Some(&branch.name), None, None);
        html("</td><td>");
        html_txt(&info.author);
        html("</td><td colspan='2'>");
        print_age(info.committer_date, info.committer_tz, -1);
        html("</td></tr>\n");
    }

    if max < branches.len() {
        html("<tr class='nohover'><td colspan='5'>");
        refs_link(ctx, "[...]", None, None, ctx.qry.head.as_deref(), None, Some("heads"));
        html("</td></tr>");
    }
}

fn print_tags(ctx: &CgitContext, tags: &[TagInfo], max_count: i32, _columns: i32) {
    if tags.is_empty() {
        return;
    }

    let mut tags: Vec<&TagInfo> = tags.iter().collect();

    // Sort by date (most recent first)
    tags.sort_by(|a, b| {
        let date_a = if a.tagger_date > 0 { a.tagger_date } else {
            a.commit.as_ref().map(|c| c.committer_date).unwrap_or(0)
        };
        let date_b = if b.tagger_date > 0 { b.tagger_date } else {
            b.commit.as_ref().map(|c| c.committer_date).unwrap_or(0)
        };
        date_b.cmp(&date_a)
    });

    let max = if max_count == 0 || max_count as usize > tags.len() {
        tags.len()
    } else {
        max_count as usize
    };

    html("<tr class='nohover'><th class='left'>Tag</th>\
          <th class='left'>Download</th>\
          <th class='left'>Author</th>\
          <th class='left' colspan='2'>Age</th></tr>\n");

    for tag in &tags[..max] {
        html("<tr><td>");
        tag_link(ctx, &tag.name, None, None, &tag.name);
        html("</td><td>");
        // TODO: snapshot links when snapshots enabled
        html("</td><td>");
        if let Some(tagger) = &tag.tagger {
            html_txt(tagger);
        } else if let Some(commit) = &tag.commit {
            html_txt(&commit.author);
        }
        html("</td><td colspan='2'>");
        if tag.tagger_date > 0 {
            print_age(tag.tagger_date, tag.tagger_tz, -1);
        } else if let Some(commit) = &tag.commit {
            print_age(commit.committer_date, 0, -1);
        }
        html("</td></tr>\n");
    }

    if max < tags.len() {
        html("<tr class='nohover'><td colspan='5'>");
        refs_link(ctx, "[...]", None, None, ctx.qry.head.as_deref(), None, Some("tags"));
        html("</td></tr>");
    }
}

fn print_summary_log(ctx: &CgitContext, commits: &[CommitInfo], count: usize, columns: i32) {
    html("<tr class='nohover'>");
    html("<th class='left'>Age</th>");
    html("<th class='left'>Commit message</th>");
    html("<th class='left'>Author</th>");

    let repo_idx = ctx.repo.unwrap();
    let repo = &ctx.repolist.repos[repo_idx];
    if repo.enable_log_filecount != 0 || ctx.cfg.enable_log_filecount != 0 {
        html("<th class='left'>Files</th>");
    }
    if repo.enable_log_linecount != 0 || ctx.cfg.enable_log_linecount != 0 {
        html("<th class='left'>Lines</th>");
    }
    html("</tr>\n");

    let two_weeks: i64 = 7 * 24 * 60 * 60 * 2;
    for commit in commits {
        html("<tr><td>");
        print_age(commit.committer_date, commit.committer_tz, two_weeks);
        html("</td><td>");
        commit_link(ctx, &commit.subject, None, None, ctx.qry.head.as_deref(), Some(&commit.oid), ctx.qry.vpath.as_deref());
        html("</td><td>");
        html_txt(&commit.author);

        if repo.enable_log_filecount != 0 || ctx.cfg.enable_log_filecount != 0 {
            html("</td><td>");
        }
        if repo.enable_log_linecount != 0 || ctx.cfg.enable_log_linecount != 0 {
            html("</td><td>");
        }

        html("</td></tr>\n");
    }

    if commits.len() >= count {
        html(&format!("<tr class='nohover'><td colspan='{}'>", columns));
        log_link(ctx, "[...]", None, None, ctx.qry.head.as_deref());
        html("</td></tr>\n");
    }
}

fn print_clone_urls(ctx: &CgitContext, columns: i32) {
    let repo_idx = ctx.repo.unwrap();
    let repo = &ctx.repolist.repos[repo_idx];

    let urls = get_clone_urls(ctx, repo);
    if urls.is_empty() {
        return;
    }

    let mut first = true;
    for url in &urls {
        if first {
            html(&format!("<tr class='nohover'><td colspan='{}'>&nbsp;</td></tr>", columns));
            html(&format!("<tr class='nohover'><th class='left' colspan='{}'>Clone</th></tr>\n", columns));
            first = false;
        }
        html(&format!("<tr><td colspan='{}'><a rel='vcs-git' href='", columns));
        html_url_path(url);
        html("' title='");
        html_attr(&repo.name);
        html(" Git repository'>");
        html_txt(url);
        html("</a></td></tr>\n");
    }
}

fn get_clone_urls(ctx: &CgitContext, repo: &rgit_core::repo::CgitRepo) -> Vec<String> {
    let mut urls = Vec::new();

    let overrides = [
        ("CGIT_REPO_URL", repo.url.as_str()),
        ("CGIT_REPO_NAME", repo.name.as_str()),
        ("CGIT_REPO_PATH", repo.path.as_deref().unwrap_or("")),
        ("CGIT_REPO_OWNER", repo.owner.as_deref().unwrap_or("")),
        ("CGIT_REPO_DEFBRANCH", repo.defbranch.as_deref().unwrap_or("")),
        ("CGIT_REPO_SECTION", repo.section.as_str()),
        ("CGIT_REPO_CLONE_URL", repo.clone_url.as_deref().unwrap_or("")),
    ];

    if let Some(clone_url) = &repo.clone_url {
        let expanded = rgit_core::macros::expand_macros_with(clone_url, &overrides);
        for part in expanded.split_whitespace() {
            if !part.is_empty() {
                urls.push(part.to_string());
            }
        }
    } else if let Some(clone_prefix) = &ctx.cfg.clone_prefix {
        for part in clone_prefix.split_whitespace() {
            if !part.is_empty() {
                urls.push(format!("{}/{}", part, repo.url));
            }
        }
    }

    urls
}
