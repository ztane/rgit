use rgit_core::context::CgitContext;
use rgit_core::html::*;
use crate::shared::*;

/// Print the repository index page.
/// Matches C cgit's cgit_print_repolist().
pub fn print_repolist(ctx: &mut CgitContext) {
    let mut hits = 0;
    let mut header_printed = false;
    let mut columns = 3;

    // Check if any repos are visible
    let any_visible = ctx.repolist.repos.iter().any(|r| is_visible(r, ctx));
    if !any_visible {
        print_error_page(ctx, 404, "Not found", "No repositories found");
        return;
    }

    if ctx.cfg.enable_index_links != 0 {
        columns += 1;
    }
    if ctx.cfg.enable_index_owner != 0 {
        columns += 1;
    }

    ctx.page.title = Some(ctx.cfg.root_title.clone());
    print_http_headers(ctx);
    print_docstart(ctx);
    print_pageheader(ctx);

    // Sort repos if needed
    let sort_field = ctx.qry.sort.clone();
    if let Some(ref sort) = sort_field {
        sort_repolist(ctx, sort);
    } else if ctx.cfg.section_sort != 0 {
        sort_repolist(ctx, "section");
    }

    let sorted = sort_field.is_some();
    let max_repodesc_len = ctx.cfg.max_repodesc_len;
    let max_repo_count = ctx.cfg.max_repo_count;
    let enable_index_owner = ctx.cfg.enable_index_owner;
    let enable_index_links = ctx.cfg.enable_index_links;
    let ofs = ctx.qry.ofs;
    let virtual_root = ctx.cfg.virtual_root.clone();
    let mut last_section: Option<String> = None;

    html("<table summary='repository list' class='list nowrap'>");

    for i in 0..ctx.repolist.repos.len() {
        if !is_visible_idx(ctx, i) {
            continue;
        }
        hits += 1;
        if hits <= ofs {
            continue;
        }
        if hits > ofs + max_repo_count {
            continue;
        }

        if !header_printed {
            print_repolist_header(ctx);
            header_printed = true;
        }

        let repo = &ctx.repolist.repos[i];
        let section = if repo.section.is_empty() {
            None
        } else {
            Some(repo.section.clone())
        };

        // Print section header if changed
        if !sorted {
            if section != last_section {
                if let Some(ref s) = section {
                    html(&format!(
                        "<tr class='nohover-highlight'><td colspan='{}' class='reposection'>",
                        columns
                    ));
                    html_txt(s);
                    html("</td></tr>");
                }
                last_section = section.clone();
            }
        }

        let sublevel = !sorted && section.is_some();
        html(&format!(
            "<tr><td class='{}'>",
            if sublevel { "sublevel-repo" } else { "toplevel-repo" }
        ));

        // Repository name link -> summary (uses html_url_path for encoding)
        let repo_url = &ctx.repolist.repos[i].url;
        let repo_name = &ctx.repolist.repos[i].name;
        html("<a href='");
        emit_repo_url(&virtual_root, repo_url);
        html("'>");
        html_txt(repo_name);
        html("</a>");
        html("</td><td>");

        // Description link
        html("<a href='");
        emit_repo_url(&virtual_root, repo_url);
        html("'>");
        let desc = &ctx.repolist.repos[i].desc;
        if html_ntxt(desc, max_repodesc_len as usize) < 0 {
            html("...");
        }
        html_link_close();
        html("</td><td>");

        if enable_index_owner != 0 {
            let owner = ctx.repolist.repos[i].owner.as_deref().unwrap_or("");
            if !owner.is_empty() {
                let currenturl = current_url(ctx);
                html("<a href='");
                html_attr(&currenturl);
                html("?q=");
                html_url_arg(owner);
                html("'>");
                html_txt(owner);
                html("</a>");
            }
            html("</td><td>");
        }

        // Idle column
        let mtime = rgit_core::repo::get_repo_modtime(&mut ctx.repolist.repos[i], &ctx.cfg.agefile);
        if mtime > 0 {
            print_age(mtime, 0, -1);
        }
        html("</td>");

        if enable_index_links != 0 {
            html("<td>");
            let repo_url = &ctx.repolist.repos[i].url;
            html("<a class='button' href='");
            emit_repo_url(&virtual_root, repo_url);
            html("'>summary</a>");

            html("<a class='button' href='");
            emit_repo_page_url(&virtual_root, repo_url, "log");
            html("'>log</a>");

            html("<a class='button' href='");
            emit_repo_page_url(&virtual_root, repo_url, "tree");
            html("'>tree</a>");
            html("</td>");
        }
        html("</tr>\n");
    }
    html("</table>");

    if hits > max_repo_count {
        print_pager(ctx, hits, max_repo_count);
    }

    print_docend(ctx);
}

fn is_visible(repo: &rgit_core::repo::CgitRepo, ctx: &CgitContext) -> bool {
    if repo.hide != 0 || repo.ignore != 0 {
        return false;
    }
    if let Some(ref search) = ctx.qry.search {
        let search_lower = search.to_lowercase();
        let matches = repo.url.to_lowercase().contains(&search_lower)
            || repo.name.to_lowercase().contains(&search_lower)
            || repo.desc.to_lowercase().contains(&search_lower)
            || repo
                .owner
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(&search_lower);
        if !matches {
            return false;
        }
    }
    if let Some(ref url) = ctx.qry.url {
        if !repo.url.starts_with(url.as_str()) {
            return false;
        }
    }
    true
}

fn is_visible_idx(ctx: &CgitContext, idx: usize) -> bool {
    is_visible(&ctx.repolist.repos[idx], ctx)
}

fn print_repolist_header(ctx: &CgitContext) {
    html("<tr class='nohover'>");
    print_sort_header(ctx, "Name", "name");
    print_sort_header(ctx, "Description", "desc");
    if ctx.cfg.enable_index_owner != 0 {
        print_sort_header(ctx, "Owner", "owner");
    }
    print_sort_header(ctx, "Idle", "idle");
    if ctx.cfg.enable_index_links != 0 {
        html("<th class='left'>Links</th>");
    }
    html("</tr>\n");
}

fn print_sort_header(ctx: &CgitContext, title: &str, sort: &str) {
    let currenturl = current_url(ctx);
    html("<th class='left'><a href='");
    html_attr(&currenturl);
    html(&format!("?s={}", sort));
    if let Some(ref search) = ctx.qry.search {
        html("&amp;q=");
        html_url_arg(search);
    }
    html(&format!("'>{}</a></th>", title));
}

fn sort_repolist(ctx: &mut CgitContext, field: &str) {
    let case_sensitive = ctx.cfg.case_sensitive_sort != 0;
    match field {
        "name" => ctx.repolist.repos.sort_by(|a, b| {
            cmp_str(&a.name, &b.name, case_sensitive)
        }),
        "desc" => ctx.repolist.repos.sort_by(|a, b| {
            cmp_str(&a.desc, &b.desc, case_sensitive)
        }),
        "owner" => ctx.repolist.repos.sort_by(|a, b| {
            cmp_str(
                a.owner.as_deref().unwrap_or(""),
                b.owner.as_deref().unwrap_or(""),
                case_sensitive,
            )
        }),
        "section" => ctx.repolist.repos.sort_by(|a, b| {
            let result = cmp_str(&a.section, &b.section, case_sensitive);
            if result == std::cmp::Ordering::Equal {
                cmp_str(&a.name, &b.name, case_sensitive)
            } else {
                result
            }
        }),
        "idle" => {
            let agefile = ctx.cfg.agefile.clone();
            ctx.repolist.repos.sort_by(|a, b| {
                let mut a = a.clone();
                let mut b = b.clone();
                let ma = rgit_core::repo::get_repo_modtime(&mut a, &agefile);
                let mb = rgit_core::repo::get_repo_modtime(&mut b, &agefile);
                mb.cmp(&ma)
            });
        }
        _ => {}
    }
}

fn cmp_str(a: &str, b: &str, case_sensitive: bool) -> std::cmp::Ordering {
    if case_sensitive {
        a.cmp(b)
    } else {
        a.to_lowercase().cmp(&b.to_lowercase())
    }
}

/// Emit a URL-encoded repo URL into an href attribute.
/// Uses html_url_path for proper encoding (spaces → %20, etc).
fn emit_repo_url(virtual_root: &Option<String>, repo_url: &str) {
    if let Some(vr) = virtual_root {
        html_url_path(vr);
        html_url_path(repo_url);
        if !repo_url.ends_with('/') {
            html("/");
        }
    } else {
        html("?r=");
        html_url_arg(repo_url);
    }
}

/// Emit a URL-encoded repo+page URL into an href attribute.
fn emit_repo_page_url(virtual_root: &Option<String>, repo_url: &str, page: &str) {
    if let Some(vr) = virtual_root {
        html_url_path(vr);
        html_url_path(repo_url);
        if !repo_url.ends_with('/') {
            html("/");
        }
        html_url_path(page);
        html("/");
    } else {
        html("?url=");
        html_url_arg(repo_url);
        html("/");
        html_url_arg(page);
        html("/");
    }
}

fn print_pager(ctx: &CgitContext, items: i32, pagelen: i32) {
    html("<ul class='pager'>");
    let mut i = 0;
    let mut ofs = 0;
    while ofs < items {
        let class = if ctx.qry.ofs == ofs {
            " class='current'"
        } else {
            ""
        };
        html("<li>");
        let rooturl = ctx.rooturl();
        html(&format!(
            "<a{} href='",
            class
        ));
        html_attr(rooturl);
        html(&format!("?ofs={}", ofs));
        if let Some(ref search) = ctx.qry.search {
            html("&amp;q=");
            html_url_arg(search);
        }
        if let Some(ref sort) = ctx.qry.sort {
            html("&amp;s=");
            html_url_arg(sort);
        }
        html(&format!("'>[{}]</a>", i + 1));
        html("</li>");
        i += 1;
        ofs = i * pagelen;
    }
    html("</ul>");
}
