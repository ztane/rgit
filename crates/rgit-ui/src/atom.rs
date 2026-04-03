use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::git;
use crate::shared::*;

/// Print the Atom feed for a repository.
pub fn print_atom(ctx: &mut CgitContext) {
    let repo_idx = ctx.repo.unwrap();
    let repo_path = ctx.repolist.repos[repo_idx].path.clone().unwrap_or_default();
    let repo_url = ctx.repolist.repos[repo_idx].url.clone();

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
    let max_count = if ctx.cfg.max_atom_items > 0 {
        ctx.cfg.max_atom_items as usize
    } else {
        10
    };

    let gix_repo = match gix_repo {
        Some(r) => r,
        None => return,
    };

    let commits = git::commit::walk_log(&gix_repo, &head, max_count, 0);

    ctx.page.mimetype = "text/xml".to_string();
    print_http_headers(ctx);

    html("<?xml version='1.0' encoding='UTF-8'?>\n");
    html("<feed xmlns='http://www.w3.org/2005/Atom'>\n");

    // Feed title
    html("<title>");
    html_txt(&repo_url);
    if let Some(ref p) = path {
        html("/");
        html_txt(p);
    }
    html("</title>\n");

    // Feed subtitle (repo description)
    let desc = &ctx.repolist.repos[repo_idx].desc;
    if !desc.is_empty() && desc != "[no description]" {
        html("<subtitle>");
        html_txt(desc);
        html("</subtitle>\n");
    }

    // Self link
    let host = ctx.env.server_name.as_deref().unwrap_or("localhost");
    let script = &ctx.cfg.script_name;
    let base_url = format!("http://{}{}", host, script);

    html("<link rel='alternate' type='text/html' href='");
    html_attr(&base_url);
    html("/");
    html_url_path(&repo_url);
    html("/'/>\n");

    // Updated (from most recent commit)
    if let Some(latest) = commits.first() {
        html("<updated>");
        html_txt(&format_iso8601_atom(latest.committer_date, latest.committer_tz));
        html("</updated>\n");
    }

    // Feed ID
    html("<id>");
    html_txt(&base_url);
    html("/");
    html_txt(&repo_url);
    html("/</id>\n");

    // Entries
    for commit in &commits {
        html("<entry>\n");

        html("<title>");
        html_txt(&commit.subject);
        html("</title>\n");

        html("<updated>");
        html_txt(&format_iso8601_atom(commit.committer_date, commit.committer_tz));
        html("</updated>\n");

        html("<author>\n<name>");
        html_txt(&commit.author);
        html("</name>\n<email>");
        // Strip angle brackets from email
        let email = commit.author_email.trim_start_matches('<').trim_end_matches('>');
        html_txt(email);
        html("</email>\n</author>\n");

        html("<published>");
        html_txt(&format_iso8601_atom(commit.author_date, commit.author_tz));
        html("</published>\n");

        html("<link rel='alternate' type='text/html' href='");
        html_attr(&base_url);
        html("/");
        html_url_path(&repo_url);
        html("/commit/?id=");
        html_url_arg(&commit.oid);
        html("'/>\n");

        html("<id>urn:sha1:");
        html_txt(&commit.oid);
        html("</id>\n");

        html("<content type='text'>");
        html_txt(&commit.msg);
        html("</content>\n");

        html("</entry>\n");
    }

    html("</feed>\n");
}

/// Format timestamp as ISO 8601 for Atom feeds (with T separator and timezone).
fn format_iso8601_atom(timestamp: i64, tz: i32) -> String {
    let tz_hours = tz / 100;
    let tz_mins = (tz % 100).abs();
    let offset_secs = (tz_hours as i64) * 3600 + (tz_mins as i64) * 60;
    let local_ts = timestamp + offset_secs;
    let (year, month, day, hour, min, sec, _wday) = unix_to_gmt(local_ts);
    let sign = if tz >= 0 { '+' } else { '-' };
    let tz_abs = tz.unsigned_abs();
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{:02}:{:02}",
        year, month + 1, day, hour, min, sec, sign, tz_abs / 100, tz_abs % 100)
}
