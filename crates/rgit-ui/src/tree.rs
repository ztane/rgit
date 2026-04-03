use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::filter;
use rgit_core::git;
use crate::shared::*;

/// Print the tree page for a repository.
pub fn print_tree(ctx: &mut CgitContext) {
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
    let path = ctx.qry.path.clone();

    let gix_repo = match gix_repo {
        Some(r) => r,
        None => {
            print_error_page(ctx, 500, "Internal server error", "Cannot open repository");
            return;
        }
    };

    // Check if path refers to a blob (file) or tree (directory)
    if let Some(ref p) = path {
        if !p.is_empty() {
            // Try to read as blob first
            if let Some((blob_oid, data)) = git::tree::read_blob(&gix_repo, &head, p) {
                print_blob(ctx, &head, p, &blob_oid, &data);
                return;
            }
        }
    }

    // Show directory listing
    let entries = git::tree::list_tree(&gix_repo, &head, path.as_deref());
    match entries {
        Some(entries) => print_tree_listing(ctx, &head, path.as_deref(), &entries),
        None => {
            print_error_page(ctx, 404, "Not found", "Path not found");
        }
    }
}

fn print_tree_listing(ctx: &mut CgitContext, head: &str, path: Option<&str>, entries: &[git::tree::TreeEntry]) {
    ctx.page.title = Some(format!("{} - {}", ctx.cfg.root_title, ctx.repolist.repos[ctx.repo.unwrap()].name));
    print_layout_start(ctx);

    html("<table summary='tree listing' class='list'>\n");
    html("<tr class='nohover'>");
    html("<th class='left'>Mode</th>");
    html("<th class='left'>Name</th>");
    html("<th class='right'>Size</th>");
    html("<th/>");
    html("</tr>\n");

    for entry in entries {
        html("<tr><td class='ls-mode'>");
        print_filemode(entry.mode);
        html("</td><td>");

        let fullpath = if let Some(p) = path {
            if p.is_empty() {
                entry.name.clone()
            } else {
                format!("{}/{}", p, entry.name)
            }
        } else {
            entry.name.clone()
        };

        if entry.is_dir {
            reporevlink(ctx, "tree", &entry.name, None, Some("ls-dir"), Some(head), None, Some(&fullpath));
        } else if entry.is_submodule {
            html_txt(&entry.name);
        } else {
            let class = if let Some(dot_pos) = entry.name.rfind('.') {
                format!("ls-blob {}", &entry.name[dot_pos + 1..])
            } else {
                "ls-blob".to_string()
            };
            reporevlink(ctx, "tree", &entry.name, None, Some(&class), Some(head), None, Some(&fullpath));
        }

        if entry.is_symlink {
            // TODO: show symlink target
        }

        html(&format!("</td><td class='ls-size'>{}</td>", entry.size));

        html("<td>");
        // Log button
        cgit_log_link_button(ctx, head, &fullpath);
        if !entry.is_submodule {
            cgit_plain_link_button(ctx, head, &fullpath);
        }
        html("</td></tr>\n");
    }

    html("</table>\n");
    print_layout_end(ctx);
}

fn print_blob(ctx: &mut CgitContext, head: &str, path: &str, blob_oid: &str, data: &[u8]) {
    let basename = path.rsplit('/').next().unwrap_or(path);
    let is_binary = data.iter().take(8000).any(|&b| b == 0);
    let source_filter = ctx.repolist.repos[ctx.repo.unwrap()].source_filter.clone();

    ctx.page.title = Some(format!("{} - {}", ctx.cfg.root_title, ctx.repolist.repos[ctx.repo.unwrap()].name));
    print_layout_start(ctx);

    html(&format!("blob: {} (", blob_oid));
    reporevlink(ctx, "plain", "plain", None, None, Some(head), None, Some(path));
    html(")\n");

    if ctx.cfg.max_blob_size > 0 && (data.len() / 1024) > ctx.cfg.max_blob_size as usize {
        html(&format!("<div class='error'>blob size ({}KB) exceeds display size limit ({}KB).</div>",
                      data.len() / 1024, ctx.cfg.max_blob_size));
        print_layout_end(ctx);
        return;
    }

    if is_binary {
        print_binary_buffer(data);
    } else {
        let text = String::from_utf8_lossy(data);
        print_text_buffer(basename, &text, ctx.cfg.enable_tree_linenumbers != 0, source_filter.as_deref());
    }

    print_layout_end(ctx);
}

fn print_text_buffer(name: &str, text: &str, show_linenumbers: bool, source_filter: Option<&str>) {
    html("<table summary='blob content' class='blob'>\n");

    if show_linenumbers {
        html("<tr><td class='linenumbers'><pre>");
        if !text.is_empty() {
            let mut lineno = 1u32;
            html(&format!("<a id='n{0}' href='#n{0}'>{0}</a>\n", lineno));
            // Count newlines except the very last one
            let bytes = text.as_bytes();
            for i in 0..bytes.len().saturating_sub(1) {
                if bytes[i] == b'\n' {
                    lineno += 1;
                    html(&format!("<a id='n{0}' href='#n{0}'>{0}</a>\n", lineno));
                }
            }
        }
        html("</pre></td>\n");
    } else {
        html("<tr>\n");
    }

    if source_filter.is_some() {
        html("<td class='lines'><pre><code>");
        filter::with_filter(source_filter, &[name], || {
            html_raw(text.as_bytes());
        });
        html("</code></pre></td></tr></table>\n");
    } else {
        html("<td class='lines'><pre><code>");
        html_txt(text);
        html("</code></pre></td></tr></table>\n");
    }
}

fn print_binary_buffer(data: &[u8]) {
    const ROWLEN: usize = 32;
    html("<table summary='blob content' class='bin-blob'>\n");
    html("<tr><th>ofs</th><th>hex dump</th><th>ascii</th></tr>");
    let mut ofs = 0;
    while ofs < data.len() {
        html(&format!("<tr><td class='right'>{:04x}</td><td class='hex'>", ofs));
        let end = std::cmp::min(ofs + ROWLEN, data.len());
        for idx in 0..(end - ofs) {
            let sep = if idx == 16 { "    " } else { " " };
            html(&format!("{}{:02x}", sep, data[ofs + idx]));
        }
        html(" </td><td class='hex'>");
        let mut ascii = String::new();
        for idx in 0..(end - ofs) {
            let b = data[ofs + idx];
            if b.is_ascii_graphic() {
                ascii.push(b as char);
            } else {
                ascii.push('.');
            }
        }
        html_txt(&ascii);
        html("</td></tr>\n");
        ofs += ROWLEN;
    }
    html("</table>\n");
}

fn print_filemode(mode: u32) {
    let file_type = mode & 0o170000;
    let perms = mode & 0o7777;

    match file_type {
        0o040000 => html("d"),
        0o120000 => html("l"),
        0o160000 => html("m"), // submodule
        _ => html("-"),
    }

    // rwx for owner, group, other
    for shift in [6, 3, 0] {
        let bits = (perms >> shift) & 7;
        html(if bits & 4 != 0 { "r" } else { "-" });
        html(if bits & 2 != 0 { "w" } else { "-" });
        html(if bits & 1 != 0 { "x" } else { "-" });
    }
}

fn cgit_log_link_button(ctx: &CgitContext, head: &str, path: &str) {
    let delim = repolink(ctx, None, Some("button"), Some("log"), Some(head), Some(path));
    let _ = delim;
    html("'>");
    html_txt("log");
    html("</a>");
}

fn cgit_plain_link_button(ctx: &CgitContext, head: &str, path: &str) {
    reporevlink(ctx, "plain", "plain", None, Some("button"), Some(head), None, Some(path));
}
