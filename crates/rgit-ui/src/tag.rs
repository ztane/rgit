use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::git;
use crate::shared::*;

/// Print the tag detail page.
pub fn print_tag(ctx: &mut CgitContext) {
    let repo_idx = ctx.repo.unwrap();
    let repo_path = ctx.repolist.repos[repo_idx].path.clone().unwrap_or_default();

    let gix_repo = match git::open_repo(&repo_path) {
        Some(r) => r,
        None => {
            print_error_page(ctx, 500, "Internal server error", "Cannot open repository");
            return;
        }
    };

    let oid_str = match ctx.qry.oid.as_deref() {
        Some(o) => o.to_string(),
        None => {
            print_error_page(ctx, 400, "Bad request", "No tag specified");
            return;
        }
    };

    // Try to find the tag
    let tags = git::refs::list_tags(&gix_repo);
    let tag_info = tags.iter().find(|t| t.name == oid_str || t.tagged_oid == oid_str);

    ctx.page.title = Some(format!("{} - {} - tag", ctx.cfg.root_title, ctx.repolist.repos[repo_idx].name));
    print_layout_start(ctx);

    if let Some(tag) = tag_info {
        html("<table summary='tag info' class='commit-info'>\n");
        html("<tr><th>tag name</th><td>");
        html_txt(&tag.name);
        html("</td></tr>\n");
        html("<tr><th>tag object</th><td class='oid'>");
        html_txt(&tag.tagged_oid);
        html("</td></tr>\n");

        if let Some(ref tagger) = tag.tagger {
            html("<tr><th>tagged by</th><td>");
            html_txt(tagger);
            if let Some(ref email) = tag.tagger_email {
                html(" ");
                html_txt(email);
            }
            html("</td></tr>\n");
            if tag.tagger_date > 0 {
                html("<tr><th>tagged at</th><td>");
                html_txt(&format_iso8601_full(tag.tagger_date, tag.tagger_tz));
                html("</td></tr>\n");
            }
        }

        html("</table>\n");

        if let Some(ref msg) = tag.msg {
            html("<div class='commit-subject'>");
            let (subject, body) = match msg.find('\n') {
                Some(pos) => (&msg[..pos], msg[pos..].trim_start_matches('\n')),
                None => (msg.as_str(), ""),
            };
            html_txt(subject);
            html("</div>\n");
            if !body.is_empty() {
                html("<div class='commit-msg'>");
                html_txt(body);
                html("</div>\n");
            }
        }
    } else {
        html("<div class='error'>Tag not found</div>\n");
    }

    print_layout_end(ctx);
}
