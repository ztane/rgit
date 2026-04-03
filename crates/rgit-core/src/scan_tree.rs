use std::fs;
use std::path::Path;

use crate::config::parse_configfile;
use crate::context::CgitContext;
use crate::repo::{CgitRepo, trim_end};

/// Check if a directory looks like a git repository (has objects/ dir and HEAD file).
fn is_git_dir(path: &Path) -> bool {
    let objects = path.join("objects");
    let head = path.join("HEAD");
    objects.is_dir() && head.is_file()
}

/// Scan a filesystem tree for git repositories, adding them to ctx.repolist.
pub fn scan_tree(ctx: &mut CgitContext, base: &str) {
    scan_path(ctx, base, base, 0);
}

const MAX_SCAN_DEPTH: u32 = 20;

/// Scan a project list file: each line is a relative path under `base`.
pub fn scan_projects(ctx: &mut CgitContext, base: &str, projectsfile: &str) {
    let contents = match fs::read_to_string(projectsfile) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error opening projectsfile {}: {}", projectsfile, e);
            return;
        }
    };
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let full = format!("{}/{}", base, line);
        scan_path(ctx, base, &full, 0);
    }
}

fn scan_path(ctx: &mut CgitContext, base: &str, path: &str, depth: u32) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    let path_buf = Path::new(path);

    let Ok(_) = fs::read_dir(path_buf) else {
        eprintln!("Error opening directory {}", path);
        return;
    };

    // Check if path itself is a git dir
    if is_git_dir(path_buf) {
        add_repo(ctx, base, path);
        return;
    }

    // Check for .git subdirectory
    let dotgit = path_buf.join(".git");
    if is_git_dir(&dotgit) {
        add_repo(ctx, base, &dotgit.to_string_lossy());
        return;
    }

    // Recurse into subdirectories
    let Ok(entries) = fs::read_dir(path_buf) else { return };
    for entry in entries {
        let Ok(entry) = entry else { continue };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip . and ..
        if name_str == "." || name_str == ".." {
            continue;
        }
        // Skip hidden unless scan_hidden_path
        if name_str.starts_with('.') && ctx.cfg.scan_hidden_path == 0 {
            continue;
        }

        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() || ft.is_symlink() {
            let child = entry.path();
            // For symlinks, verify it points to a directory
            if ft.is_symlink() && !child.is_dir() {
                continue;
            }
            scan_path(ctx, base, &child.to_string_lossy(), depth + 1);
        }
    }
}

fn add_repo(ctx: &mut CgitContext, base: &str, path: &str) {
    let path_with_slash = if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{}/", path)
    };

    // Check for noweb marker
    let noweb = Path::new(&path_with_slash).join("noweb");
    if noweb.exists() {
        return;
    }

    // Check strict_export
    if let Some(ref strict_export) = ctx.cfg.strict_export {
        let export_file = Path::new(&path_with_slash).join(strict_export);
        if !export_file.exists() {
            return;
        }
    }

    // Compute relative URL from base
    let base_with_slash = if base.ends_with('/') {
        base.to_string()
    } else {
        format!("{}/", base)
    };

    let rel = if path_with_slash.starts_with(&base_with_slash) {
        &path_with_slash[base_with_slash.len()..]
    } else {
        &path_with_slash
    };

    // Strip trailing /
    let rel = rel.trim_end_matches('/');
    // Strip /.git suffix
    let rel = rel.strip_suffix("/.git").unwrap_or(rel);

    let mut url = rel.to_string();

    // Create the repo
    let mut repo = CgitRepo::new(&url, &ctx.cfg);

    // Read git config if enabled
    if ctx.cfg.enable_git_config != 0 {
        let config_path = format!("{}config", path_with_slash);
        if Path::new(&config_path).exists() {
            read_git_config(&config_path, &mut repo, &ctx.cfg);
        }
    }

    // Apply remove-suffix
    if ctx.cfg.remove_suffix != 0 {
        if let Some(stripped) = url.strip_suffix(".git") {
            url = stripped.trim_end_matches('/').to_string();
            repo.url = url.clone();
            repo.name = url.clone();
        }
    }

    // Set path
    repo.path = Some(trim_end(&path_with_slash, '/'));

    // Try to read owner from file ownership
    // (skip getpwuid as it's platform-specific and the common case is config-based)

    // Read description file if desc is still default
    if repo.desc == "[no description]" {
        let desc_path = format!("{}description", path_with_slash);
        if let Ok(desc) = fs::read_to_string(&desc_path) {
            let desc = desc.trim();
            if !desc.is_empty() && !desc.starts_with("Unnamed repository") {
                repo.desc = desc.to_string();
            }
        }
    }

    // Apply section-from-path
    if ctx.cfg.section_from_path != 0 {
        if let Some(section) = section_from_path(rel, ctx.cfg.section_from_path) {
            if repo.name.starts_with(&section) {
                let rest = &repo.name[section.len()..];
                if rest.starts_with('/') {
                    repo.name = rest[1..].to_string();
                }
            }
            repo.section = section;
        }
    }

    // Read per-repo cgitrc
    let cgitrc_path = format!("{}cgitrc", path_with_slash);
    if Path::new(&cgitrc_path).exists() {
        let idx = ctx.repolist.add_repo(repo);
        ctx.repo = Some(idx);
        parse_configfile(&cgitrc_path, &mut |name, value| {
            if let Some(arg) = name.strip_prefix("repo.") {
                crate::config::apply_repo_config_public(ctx, arg, value);
            }
        });
    } else {
        ctx.repolist.add_repo(repo);
    }
}

/// Extract section from relative path.
fn section_from_path(rel: &str, n: i32) -> Option<String> {
    if n > 0 {
        // Take first N path components
        let mut count = 0;
        let mut end = 0;
        for (i, c) in rel.char_indices() {
            if c == '/' {
                count += 1;
                if count == n {
                    end = i;
                    break;
                }
            }
        }
        if count == n && end > 0 {
            Some(rel[..end].to_string())
        } else {
            None
        }
    } else {
        // Take from the end: n is negative, strip last |n| components
        let mut count = 0;
        let mut pos = rel.len();
        for (i, c) in rel.char_indices().rev() {
            if c == '/' {
                count += 1;
                if count == -n {
                    pos = i;
                    break;
                }
            }
        }
        if count == -n && pos > 0 {
            Some(rel[..pos].to_string())
        } else {
            None
        }
    }
}

/// Read gitweb.* and cgit.* keys from a git config file.
fn read_git_config(config_path: &str, repo: &mut CgitRepo, cfg: &crate::context::CgitConfig) {
    let Ok(contents) = fs::read_to_string(config_path) else { return };

    // Simple INI-style parser for git config
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') || line.starts_with('[') {
            continue;
        }
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();
            // Strip surrounding quotes from value if present
            let value = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                &value[1..value.len()-1]
            } else {
                value
            };

            match key {
                "owner" | "gitweb.owner" => { repo.owner = Some(value.to_string()); }
                "description" | "gitweb.description" => { repo.desc = value.to_string(); }
                "category" | "gitweb.category" => { repo.section = value.to_string(); }
                "homepage" | "gitweb.homepage" => { repo.homepage = Some(value.to_string()); }
                _ => {
                    if let Some(cgit_key) = key.strip_prefix("cgit.") {
                        crate::config::apply_repo_config_standalone(repo, cgit_key, value, cfg);
                    }
                }
            }
        }
    }
}
