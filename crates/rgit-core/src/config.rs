use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::context::{CgitConfig, CgitContext};
use crate::macros::expand_macros;
use crate::repo::{CgitRepo, ensure_end, trim_end};
use crate::snapshot::parse_snapshots_mask;

/// Parse a cgitrc-style config file and call `callback` for each key=value pair.
/// Handles comments (#, ;), blank lines, \r\n line endings.
/// Supports nesting up to depth 8 via "include" directives.
pub fn parse_configfile(filename: &str, callback: &mut dyn FnMut(&str, &str)) {
    parse_configfile_inner(filename, callback, 0);
}

fn parse_configfile_inner(filename: &str, callback: &mut dyn FnMut(&str, &str), depth: u32) {
    if depth > 8 {
        return;
    }
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(_) => return,
    };
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let line = line.trim_start();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(eq_pos) = line.find('=') {
            let name = &line[..eq_pos];
            let value = &line[eq_pos + 1..];
            callback(name, value);
        }
    }
}

/// The main config callback that populates a CgitContext from cgitrc entries.
/// This is the equivalent of C cgit's config_cb().
pub fn apply_config(ctx: &mut CgitContext, name: &str, value: &str) {
    // Handle repo.* settings
    if name == "repo.url" {
        let repo = CgitRepo::new(value, &ctx.cfg);
        let idx = ctx.repolist.add_repo(repo);
        ctx.repo = Some(idx);
        return;
    }

    if let Some(repo_idx) = ctx.repo {
        if name == "repo.path" {
            ctx.repolist.repos[repo_idx].path = Some(trim_end(value, '/'));
            return;
        }
        if let Some(arg) = name.strip_prefix("repo.") {
            apply_repo_config(&mut ctx.repolist.repos[repo_idx], arg, value, &ctx.cfg);
            return;
        }
    }

    match name {
        "section" => ctx.cfg.section = value.to_string(),
        "readme" => ctx.cfg.readme.push(value.to_string()),
        "root-title" => ctx.cfg.root_title = value.to_string(),
        "root-desc" => ctx.cfg.root_desc = value.to_string(),
        "root-readme" => ctx.cfg.root_readme = Some(value.to_string()),
        "css" => ctx.cfg.css.push(value.to_string()),
        "js" => ctx.cfg.js.push(value.to_string()),
        "favicon" => ctx.cfg.favicon = value.to_string(),
        "footer" => ctx.cfg.footer = Some(value.to_string()),
        "head-include" => ctx.cfg.head_include = Some(value.to_string()),
        "header" => ctx.cfg.header = Some(value.to_string()),
        "logo" => ctx.cfg.logo = value.to_string(),
        "logo-link" => ctx.cfg.logo_link = Some(value.to_string()),
        "module-link" => ctx.cfg.module_link = Some(value.to_string()),
        "strict-export" => ctx.cfg.strict_export = Some(value.to_string()),
        "virtual-root" => ctx.cfg.virtual_root = Some(ensure_end(value, '/')),
        "noplainemail" => ctx.cfg.noplainemail = value.parse().unwrap_or(0),
        "noheader" => ctx.cfg.noheader = value.parse().unwrap_or(0),
        "snapshots" => ctx.cfg.snapshots = parse_snapshots_mask(value),
        "enable-filter-overrides" => ctx.cfg.enable_filter_overrides = value.parse().unwrap_or(0),
        "enable-follow-links" => ctx.cfg.enable_follow_links = value.parse().unwrap_or(0),
        "enable-http-clone" => ctx.cfg.enable_http_clone = value.parse().unwrap_or(0),
        "enable-index-links" => ctx.cfg.enable_index_links = value.parse().unwrap_or(0),
        "enable-index-owner" => ctx.cfg.enable_index_owner = value.parse().unwrap_or(0),
        "enable-blame" => ctx.cfg.enable_blame = value.parse().unwrap_or(0),
        "enable-commit-graph" => ctx.cfg.enable_commit_graph = value.parse().unwrap_or(0),
        "enable-log-filecount" => ctx.cfg.enable_log_filecount = value.parse().unwrap_or(0),
        "enable-log-linecount" => ctx.cfg.enable_log_linecount = value.parse().unwrap_or(0),
        "enable-remote-branches" => ctx.cfg.enable_remote_branches = value.parse().unwrap_or(0),
        "enable-subject-links" => ctx.cfg.enable_subject_links = value.parse().unwrap_or(0),
        "enable-html-serving" => ctx.cfg.enable_html_serving = value.parse().unwrap_or(0),
        "enable-tree-linenumbers" => ctx.cfg.enable_tree_linenumbers = value.parse().unwrap_or(0),
        "enable-git-config" => ctx.cfg.enable_git_config = value.parse().unwrap_or(0),
        "cache-size" => ctx.cfg.cache_size = value.parse().unwrap_or(0),
        "cache-root" => ctx.cfg.cache_root = expand_macros(value),
        "cache-root-ttl" => ctx.cfg.cache_root_ttl = value.parse().unwrap_or(0),
        "cache-repo-ttl" => ctx.cfg.cache_repo_ttl = value.parse().unwrap_or(0),
        "cache-scanrc-ttl" => ctx.cfg.cache_scanrc_ttl = value.parse().unwrap_or(0),
        "cache-static-ttl" => ctx.cfg.cache_static_ttl = value.parse().unwrap_or(0),
        "cache-dynamic-ttl" => ctx.cfg.cache_dynamic_ttl = value.parse().unwrap_or(0),
        "cache-about-ttl" => ctx.cfg.cache_about_ttl = value.parse().unwrap_or(0),
        "cache-snapshot-ttl" => ctx.cfg.cache_snapshot_ttl = value.parse().unwrap_or(0),
        "case-sensitive-sort" => ctx.cfg.case_sensitive_sort = value.parse().unwrap_or(0),
        "embedded" => ctx.cfg.embedded = value.parse().unwrap_or(0),
        "max-atom-items" => ctx.cfg.max_atom_items = value.parse().unwrap_or(0),
        "max-message-length" => ctx.cfg.max_msg_len = value.parse().unwrap_or(0),
        "max-repodesc-length" => ctx.cfg.max_repodesc_len = value.parse().unwrap_or(0),
        "max-blob-size" => ctx.cfg.max_blob_size = value.parse().unwrap_or(0),
        "max-repo-count" => {
            let v: i32 = value.parse().unwrap_or(0);
            ctx.cfg.max_repo_count = if v <= 0 { i32::MAX } else { v };
        }
        "max-commit-count" => ctx.cfg.max_commit_count = value.parse().unwrap_or(0),
        "summary-log" => ctx.cfg.summary_log = value.parse().unwrap_or(0),
        "summary-branches" => ctx.cfg.summary_branches = value.parse().unwrap_or(0),
        "summary-tags" => ctx.cfg.summary_tags = value.parse().unwrap_or(0),
        "agefile" => ctx.cfg.agefile = value.to_string(),
        "mimetype-file" => ctx.cfg.mimetype_file = Some(value.to_string()),
        "renamelimit" => ctx.cfg.renamelimit = value.parse().unwrap_or(0),
        "remove-suffix" => ctx.cfg.remove_suffix = value.parse().unwrap_or(0),
        "robots" => ctx.cfg.robots = value.to_string(),
        "clone-prefix" => ctx.cfg.clone_prefix = Some(value.to_string()),
        "clone-url" => ctx.cfg.clone_url = Some(value.to_string()),
        "local-time" => ctx.cfg.local_time = value.parse().unwrap_or(0),
        "commit-sort" => {
            if value == "date" { ctx.cfg.commit_sort = 1; }
            if value == "topo" { ctx.cfg.commit_sort = 2; }
        }
        "branch-sort" => {
            if value == "age" { ctx.cfg.branch_sort = 1; }
            if value == "name" { ctx.cfg.branch_sort = 0; }
        }
        "repository-sort" => ctx.cfg.repository_sort = value.to_string(),
        "section-sort" => ctx.cfg.section_sort = value.parse().unwrap_or(0),
        "section-from-path" => ctx.cfg.section_from_path = value.parse().unwrap_or(0),
        "scan-hidden-path" => ctx.cfg.scan_hidden_path = value.parse().unwrap_or(0),
        "about-filter" => ctx.cfg.about_filter = Some(value.to_string()),
        "commit-filter" => ctx.cfg.commit_filter = Some(value.to_string()),
        "source-filter" => ctx.cfg.source_filter = Some(value.to_string()),
        "email-filter" => ctx.cfg.email_filter = Some(value.to_string()),
        "owner-filter" => ctx.cfg.owner_filter = Some(value.to_string()),
        "auth-filter" => ctx.cfg.auth_filter = Some(value.to_string()),
        "side-by-side-diffs" => {
            ctx.cfg.difftype = if value.parse::<i32>().unwrap_or(0) != 0 { 1 } else { 0 };
        }
        "project-list" => ctx.cfg.project_list = Some(expand_macros(value)),
        "scan-path" => {
            let expanded = expand_macros(value);
            if let Some(ref project_list) = ctx.cfg.project_list.clone() {
                crate::scan_tree::scan_projects(ctx, &expanded, project_list);
            } else {
                crate::scan_tree::scan_tree(ctx, &expanded);
            }
        }
        "include" => {
            let expanded = expand_macros(value);
            parse_configfile(&expanded, &mut |n, v| apply_config(ctx, n, v));
        }
        _ => {
            // Handle mimetype.* entries
            if let Some(ext) = name.strip_prefix("mimetype.") {
                let _ = (ext, value); // TODO: store in mimetypes map
            }
        }
    }
}

/// Apply a repo config setting from scan-tree's per-repo cgitrc (needs ctx for cfg reference).
pub fn apply_repo_config_public(ctx: &mut CgitContext, name: &str, value: &str) {
    if let Some(repo_idx) = ctx.repo {
        apply_repo_config(&mut ctx.repolist.repos[repo_idx], name, value, &ctx.cfg);
    }
}

/// Apply a repo config setting with standalone repo reference (for git config parsing).
pub fn apply_repo_config_standalone(repo: &mut CgitRepo, name: &str, value: &str, cfg: &CgitConfig) {
    apply_repo_config(repo, name, value, cfg);
}

fn apply_repo_config(repo: &mut CgitRepo, name: &str, value: &str, cfg: &CgitConfig) {
    match name {
        "name" => repo.name = value.to_string(),
        "clone-url" => repo.clone_url = Some(value.to_string()),
        "desc" => repo.desc = value.to_string(),
        "owner" => repo.owner = Some(value.to_string()),
        "homepage" => repo.homepage = Some(value.to_string()),
        "defbranch" => repo.defbranch = Some(value.to_string()),
        "extra-head-content" => repo.extra_head_content = Some(value.to_string()),
        "snapshots" => repo.snapshots = cfg.snapshots & parse_snapshots_mask(value),
        "enable-blame" => repo.enable_blame = value.parse().unwrap_or(0),
        "enable-commit-graph" => repo.enable_commit_graph = value.parse().unwrap_or(0),
        "enable-follow-links" => repo.enable_follow_links = value.parse().unwrap_or(0),
        "enable-log-filecount" => repo.enable_log_filecount = value.parse().unwrap_or(0),
        "enable-log-linecount" => repo.enable_log_linecount = value.parse().unwrap_or(0),
        "enable-remote-branches" => repo.enable_remote_branches = value.parse().unwrap_or(0),
        "enable-subject-links" => repo.enable_subject_links = value.parse().unwrap_or(0),
        "enable-html-serving" => repo.enable_html_serving = value.parse().unwrap_or(0),
        "branch-sort" => {
            if value == "age" { repo.branch_sort = 1; }
            if value == "name" { repo.branch_sort = 0; }
        }
        "commit-sort" => {
            if value == "date" { repo.commit_sort = 1; }
            if value == "topo" { repo.commit_sort = 2; }
        }
        "module-link" => repo.module_link = Some(value.to_string()),
        "section" => repo.section = value.to_string(),
        "snapshot-prefix" => repo.snapshot_prefix = Some(value.to_string()),
        "readme" => {
            // If readme list is still shared with global config, start fresh
            repo.readme.push(value.to_string());
        }
        "logo" => repo.logo = Some(value.to_string()),
        "logo-link" => repo.logo_link = Some(value.to_string()),
        "hide" => repo.hide = value.parse().unwrap_or(0),
        "ignore" => repo.ignore = value.parse().unwrap_or(0),
        "about-filter" if cfg.enable_filter_overrides != 0 => {
            repo.about_filter = Some(value.to_string());
        }
        "commit-filter" if cfg.enable_filter_overrides != 0 => {
            repo.commit_filter = Some(value.to_string());
        }
        "source-filter" if cfg.enable_filter_overrides != 0 => {
            repo.source_filter = Some(value.to_string());
        }
        "email-filter" if cfg.enable_filter_overrides != 0 => {
            repo.email_filter = Some(value.to_string());
        }
        "owner-filter" if cfg.enable_filter_overrides != 0 => {
            repo.owner_filter = Some(value.to_string());
        }
        _ => {}
    }
}
