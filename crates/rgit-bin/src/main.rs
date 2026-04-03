#![forbid(unsafe_code)]

use rgit_core::config::{apply_config, parse_configfile};
use rgit_core::context::CgitContext;
use rgit_core::macros::expand_macros;
use rgit_core::query::parse_querystring;
use rgit_core::repo::ensure_end;
use rgit_ui::shared;

fn main() {
    // NOTE: We do NOT call set_var/remove_var (unsafe in edition 2024).
    // Instead, env overrides for git subprocesses are applied via Command::env().
    // The gix library will be configured to skip system config via open options.
    // We read env vars at startup (safe) and store them in CgitContext.

    let args: Vec<String> = std::env::args().collect();

    // Handle --version before anything else
    for arg in &args[1..] {
        if arg == "--version" {
            shared::print_version();
            std::process::exit(0);
        }
    }

    let mut ctx = CgitContext::new();
    ctx.prepare_from_env();
    let mut scan = false;

    // Parse command-line arguments (matching C cgit's cgit_parse_args)
    for arg in &args[1..] {
        if let Some(val) = arg.strip_prefix("--cache=") {
            ctx.cfg.cache_root = val.to_string();
        } else if arg == "--nohttp" {
            ctx.env.no_http = Some("1".to_string());
        } else if let Some(val) = arg.strip_prefix("--query=") {
            ctx.qry.raw = Some(val.to_string());
        } else if let Some(val) = arg.strip_prefix("--repo=") {
            ctx.qry.repo = Some(val.to_string());
        } else if let Some(val) = arg.strip_prefix("--page=") {
            ctx.qry.page = Some(val.to_string());
        } else if let Some(val) = arg.strip_prefix("--head=") {
            ctx.qry.head = Some(val.to_string());
            ctx.qry.has_symref = true;
        } else if let Some(val) = arg.strip_prefix("--oid=") {
            ctx.qry.oid = Some(val.to_string());
            ctx.qry.has_oid = true;
        } else if let Some(val) = arg.strip_prefix("--ofs=") {
            ctx.qry.ofs = val.parse().unwrap_or(0);
        } else if let Some(val) = arg.strip_prefix("--scan-tree=")
            .or_else(|| arg.strip_prefix("--scan-path="))
        {
            ctx.cfg.snapshots = 0xFF;
            scan = true;
            rgit_core::scan_tree::scan_tree(&mut ctx, val);
        }
    }
    if scan {
        print_scan_result(&ctx);
        std::process::exit(0);
    }

    // Parse config file
    let config_path = expand_macros(&ctx.env.cgit_config);
    parse_configfile(&config_path, &mut |name, value| {
        apply_config(&mut ctx, name, value);
    });
    ctx.repo = None;

    // Parse querystring
    parse_querystring(&mut ctx);

    // If virtual-root isn't set, derive from script_name
    if ctx.cfg.virtual_root.is_none() {
        ctx.cfg.virtual_root = Some(ensure_end(&ctx.cfg.script_name, '/'));
    }

    // If no URL from querystring, try PATH_INFO
    if ctx.qry.url.is_none() {
        let path_info = ctx.env.path_info.clone();
        if let Some(ref path) = path_info {
            let path = if path.starts_with('/') {
                &path[1..]
            } else {
                path.as_str()
            };
            ctx.qry.url = Some(path.to_string());
            if let Some(ref raw) = ctx.qry.raw {
                ctx.qry.raw = Some(format!("{}?{}", path, raw));
            } else {
                ctx.qry.raw = Some(path.to_string());
            }
            rgit_core::query::cgit_parse_url(&mut ctx, path);
        }
    }

    // TTL calculation
    let ttl = calc_ttl(&ctx);
    if ttl < 0 {
        ctx.page.expires += 10 * 365 * 24 * 60 * 60;
    } else {
        ctx.page.expires += (ttl as i64) * 60;
    }

    // Disable caching for unauthenticated or HEAD requests (matching C cgit)
    if !ctx.env.authenticated {
        ctx.cfg.cache_size = 0;
    }
    if let Some(ref method) = ctx.env.request_method {
        if method == "HEAD" {
            ctx.cfg.cache_size = 0;
        }
    }

    let cache_size = ctx.cfg.cache_size;
    let cache_root = ctx.cfg.cache_root.clone();
    let cache_key = ctx.qry.raw.clone();

    rgit_core::cache::cache_process(
        cache_size,
        &cache_root,
        cache_key.as_deref(),
        ttl,
        || process_request(&mut ctx),
    );
}

fn calc_ttl(ctx: &CgitContext) -> i32 {
    if ctx.repo.is_none() {
        return ctx.cfg.cache_root_ttl;
    }
    if ctx.qry.page.is_none() {
        return ctx.cfg.cache_repo_ttl;
    }
    if let Some(ref page) = ctx.qry.page {
        if page == "about" {
            return ctx.cfg.cache_about_ttl;
        }
        if page == "snapshot" {
            return ctx.cfg.cache_snapshot_ttl;
        }
    }
    if ctx.qry.has_oid {
        return ctx.cfg.cache_static_ttl;
    }
    if ctx.qry.has_symref {
        return ctx.cfg.cache_dynamic_ttl;
    }
    ctx.cfg.cache_repo_ttl
}

fn process_request(ctx: &mut CgitContext) {
    // Look up the command
    let page = ctx.qry.page.clone();
    let has_repo = ctx.repo.is_some();
    let cmd = match rgit_core::cmd::get_cmd(page.as_deref(), has_repo) {
        Some(c) => c,
        None => {
            ctx.page.title = Some("cgit error".to_string());
            shared::print_error_page(ctx, 404, "Not found", "Invalid request");
            return;
        }
    };

    if ctx.cfg.enable_http_clone == 0 && cmd.is_clone {
        ctx.page.title = Some("cgit error".to_string());
        shared::print_error_page(ctx, 404, "Not found", "Invalid request");
        return;
    }

    if cmd.want_repo && ctx.repo.is_none() {
        shared::print_error_page(ctx, 400, "Bad request", "No repository selected");
        return;
    }

    // Set vpath
    if cmd.want_vpath {
        ctx.qry.vpath = ctx.qry.path.clone();
    }

    // Update page defaults for this command
    let resolved_page = page.as_deref().unwrap_or(if has_repo { "summary" } else { "repolist" });
    ctx.qry.page = Some(resolved_page.to_string());

    // Dispatch to the appropriate page renderer
    match resolved_page {
        "repolist" => rgit_ui::repolist::print_repolist(ctx),
        "summary" => {
            rgit_ui::summary::print_summary(ctx);
        }
        "log" => {
            rgit_ui::log::print_log(ctx);
        }
        "tree" => {
            rgit_ui::tree::print_tree(ctx);
        }
        "commit" => {
            rgit_ui::commit::print_commit(ctx);
        }
        "diff" => {
            rgit_ui::diff::print_diff(ctx);
        }
        "snapshot" => {
            rgit_ui::snapshot::print_snapshot(ctx);
        }
        "patch" => {
            rgit_ui::patch::print_patch(ctx);
        }
        "rawdiff" => {
            rgit_ui::rawdiff::print_rawdiff(ctx);
        }
        "about" => {
            rgit_ui::about::print_about(ctx);
        }
        "plain" => {
            rgit_ui::plain::print_plain(ctx);
        }
        "refs" => {
            rgit_ui::refs::print_refs(ctx);
        }
        "atom" => {
            rgit_ui::atom::print_atom(ctx);
        }
        "blob" => {
            rgit_ui::blob::print_blob(ctx);
        }
        "tag" => {
            rgit_ui::tag::print_tag(ctx);
        }
        "HEAD" => {
            rgit_ui::clone::print_head(ctx);
        }
        "info" => {
            rgit_ui::clone::print_info(ctx);
        }
        "objects" => {
            rgit_ui::clone::print_objects(ctx);
        }
        "ls_cache" => {
            ctx.page.mimetype = "text/plain".to_string();
            shared::print_http_headers(ctx);
            rgit_core::cache::cache_ls(&ctx.cfg.cache_root);
        }
        _ => {
            shared::print_error_page(
                ctx,
                500,
                "Not implemented",
                &format!("Page '{}' not yet implemented", resolved_page),
            );
        }
    }
}

/// Print discovered repos as cgitrc format (for --scan-path/--scan-tree).
fn print_scan_result(ctx: &CgitContext) {
    for repo in &ctx.repolist.repos {
        println!("repo.url={}", repo.url);
        if let Some(ref path) = repo.path {
            println!("repo.path={}", path);
        }
        if repo.owner.is_some() {
            println!("repo.owner={}", repo.owner.as_deref().unwrap_or(""));
        }
        if repo.desc != "[no description]" {
            println!("repo.desc={}", repo.desc);
        }
        if !repo.section.is_empty() {
            println!("repo.section={}", repo.section);
        }
        if repo.defbranch.is_some() {
            println!("repo.defbranch={}", repo.defbranch.as_deref().unwrap_or(""));
        }
        println!();
    }
}
