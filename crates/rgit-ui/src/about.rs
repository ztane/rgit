use rgit_core::context::CgitContext;
use rgit_core::html::*;
use rgit_core::filter;
use rgit_core::git;
use crate::shared::*;

/// Print the about/readme page for a repository.
pub fn print_about(ctx: &mut CgitContext) {
    let repo_idx = match ctx.repo {
        Some(idx) => idx,
        None => {
            // Site-level about page (root readme)
            ctx.page.title = Some(ctx.cfg.root_title.clone());
            print_layout_start(ctx);
            if let Some(ref root_readme) = ctx.cfg.root_readme.clone() {
                html("<div id='summary'>");
                filter::with_filter(ctx.cfg.about_filter.as_deref(), &[root_readme], || {
                    let _ = html_include(root_readme);
                });
                html("</div>");
            }
            print_layout_end(ctx);
            return;
        }
    };

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

    print_layout_start(ctx);

    let readme_list = ctx.repolist.repos[repo_idx].readme.clone();
    if readme_list.is_empty() {
        print_layout_end(ctx);
        return;
    }

    let readme_spec = &readme_list[0];
    let about_filter = ctx.repolist.repos[repo_idx].about_filter.clone();

    // Parse "ref:path" format
    let (git_ref, filename) = if let Some(colon_pos) = readme_spec.find(':') {
        (Some(&readme_spec[..colon_pos]), &readme_spec[colon_pos + 1..])
    } else {
        (None, readme_spec.as_str())
    };

    html("<div id='summary'>");

    if let Some(git_ref) = git_ref {
        // Read file from git repo
        if let Some(ref gix_repo) = gix_repo {
            if let Some((_oid, data)) = git::tree::read_blob(gix_repo, git_ref, filename) {
                filter::with_filter(about_filter.as_deref(), &[filename], || {
                    html_raw(&data);
                });
            }
        }
    } else {
        // Read file from filesystem
        filter::with_filter(about_filter.as_deref(), &[filename], || {
            let _ = html_include(filename);
        });
    }

    html("</div>");

    print_layout_end(ctx);
}
