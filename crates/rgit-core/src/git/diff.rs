/// A file changed in a commit.
#[derive(Clone, Debug)]
pub struct DiffFile {
    pub status: char,
    pub old_mode: String,
    pub new_mode: String,
    pub old_oid: String,
    pub new_oid: String,
    pub old_path: String,
    pub new_path: String,
    pub added: usize,
    pub removed: usize,
    pub binary: bool,
}

/// A diff between two commits (or a commit and null for root commits).
#[derive(Clone, Debug)]
pub struct DiffResult {
    pub files: Vec<DiffFile>,
    pub total_adds: usize,
    pub total_rems: usize,
}

// TODO: Replace diff_stat with gix tree diff (gix::diff::tree) + line counting via gix blob diffs.
// TODO: Replace unified_diff with gix-based blob diff (similar crate or gix::diff::blob).

/// Get the list of changed files between two commits (diffstat).
/// If old_rev is None, shows changes from the empty tree (root commit).
pub fn diff_stat(repo_path: &str, new_rev: &str, old_rev: Option<&str>) -> DiffResult {
    let mut cmd = super::git_command(repo_path);
    cmd.arg("diff-tree");
    cmd.arg("-r");
    cmd.arg("--numstat");
    cmd.arg("--raw");
    cmd.arg("-z"); // NUL-separated for parsing

    if let Some(old) = old_rev {
        cmd.arg(old);
        cmd.arg(new_rev);
    } else {
        // Root commit: diff against empty tree
        cmd.arg("--root");
        cmd.arg(new_rev);
    }

    let output = cmd.output().ok();
    let output = match output {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => return DiffResult { files: Vec::new(), total_adds: 0, total_rems: 0 },
    };

    parse_diff_output(&output)
}

fn parse_diff_output(output: &str) -> DiffResult {
    let mut files = Vec::new();
    let mut total_adds = 0;
    let mut total_rems = 0;

    // The -z output has NUL separators
    let parts: Vec<&str> = output.split('\0').collect();
    let mut i = 0;
    while i < parts.len() {
        let part = parts[i];
        if part.starts_with(':') {
            // Raw format: :old_mode new_mode old_oid new_oid status\0path[\0new_path]
            let fields: Vec<&str> = part.splitn(6, |c: char| c == ' ' || c == '\t').collect();
            if fields.len() >= 5 {
                let old_mode = fields[0].trim_start_matches(':').to_string();
                let new_mode = fields[1].to_string();
                let old_oid = fields[2].to_string();
                let new_oid = fields[3].to_string();
                let status = fields[4].chars().next().unwrap_or('M');

                i += 1;
                let old_path = if i < parts.len() { parts[i].to_string() } else { String::new() };
                let new_path = if status == 'R' || status == 'C' {
                    i += 1;
                    if i < parts.len() { parts[i].to_string() } else { old_path.clone() }
                } else {
                    old_path.clone()
                };

                files.push(DiffFile {
                    status,
                    old_mode,
                    new_mode,
                    old_oid,
                    new_oid,
                    old_path,
                    new_path,
                    added: 0,
                    removed: 0,
                    binary: false,
                });
            } else {
                i += 1;
            }
        } else if !part.is_empty() && !part.starts_with('\n') {
            // Numstat line: added\tremoved\tpath
            let numstat_fields: Vec<&str> = part.split('\t').collect();
            if numstat_fields.len() >= 3 {
                let added: usize = numstat_fields[0].parse().unwrap_or(0);
                let removed: usize = numstat_fields[1].parse().unwrap_or(0);
                let binary = numstat_fields[0] == "-";
                // Find matching file entry
                let path = numstat_fields[2];
                for f in &mut files {
                    if f.new_path == path || f.old_path == path {
                        f.added = added;
                        f.removed = removed;
                        f.binary = binary;
                        break;
                    }
                }
                total_adds += added;
                total_rems += removed;
            }
            i += 1;
        } else {
            i += 1;
        }
    }

    DiffResult { files, total_adds, total_rems }
}

/// Generate a unified diff for a commit.
/// Returns the raw diff output as a string.
pub fn unified_diff(repo_path: &str, new_rev: &str, old_rev: Option<&str>, path: Option<&str>) -> String {
    let mut cmd = super::git_command(repo_path);
    cmd.arg("diff-tree");
    cmd.arg("-p");

    if let Some(old) = old_rev {
        cmd.arg(old);
        cmd.arg(new_rev);
    } else {
        cmd.arg("--root");
        cmd.arg(new_rev);
    }

    cmd.arg("--");
    if let Some(p) = path {
        cmd.arg(p);
    }

    let output = cmd.output().ok();
    match output {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => String::new(),
    }
}

/// Lightweight diffstat: returns (file_count, lines_added, lines_removed) for a commit.
/// Used by log and summary pages for filecount/linecount columns.
pub fn commit_stats(repo_path: &str, commit_oid: &str) -> (usize, usize, usize) {
    let mut cmd = super::git_command(repo_path);
    cmd.arg("diff-tree");
    cmd.arg("--numstat");
    cmd.arg("--root"); // handles root commits too
    cmd.arg(commit_oid);

    let output = cmd.output().ok();
    let output = match output {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => return (0, 0, 0),
    };

    let mut files = 0;
    let mut added = 0;
    let mut removed = 0;
    for line in output.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            files += 1;
            if parts[0] != "-" {
                added += parts[0].parse::<usize>().unwrap_or(0);
                removed += parts[1].parse::<usize>().unwrap_or(0);
            }
        }
    }
    (files, added, removed)
}

/// Resolve a revision (ref name or OID prefix) to its full OID using gix.
pub fn resolve_rev(repo: &gix::Repository, rev: &str) -> Option<String> {
    // Try as full hex OID first
    if let Ok(oid) = gix::ObjectId::from_hex(rev.as_bytes()) {
        if repo.find_object(oid).is_ok() {
            return Some(oid.to_hex().to_string());
        }
    }
    // Try as reference
    if let Ok(reference) = repo.find_reference(rev)
        .or_else(|_| repo.find_reference(&format!("refs/heads/{}", rev)))
        .or_else(|_| repo.find_reference(&format!("refs/tags/{}", rev)))
    {
        let id = reference.id().detach();
        // Peel to commit if it's a tag
        if let Ok(obj) = repo.find_object(id) {
            if let Ok(commit_ref) = obj.try_to_commit_ref() {
                let _ = commit_ref;
                return Some(id.to_hex().to_string());
            }
            // Could be an annotated tag pointing to a commit
            if let Ok(tag_ref) = obj.try_to_tag_ref() {
                return Some(tag_ref.target().to_hex().to_string());
            }
        }
        return Some(id.to_hex().to_string());
    }
    None
}

/// Get parent commit OID for a given commit using gix.
pub fn get_parent(repo: &gix::Repository, rev: &str) -> Option<String> {
    let oid = resolve_to_oid(repo, rev)?;
    let object = repo.find_object(oid).ok()?;
    let commit = object.try_to_commit_ref().ok()?;
    let parents: Vec<_> = commit.parents().collect();
    parents.first().map(|id| id.to_hex().to_string())
}

/// Get the tree OID for a commit using gix.
pub fn get_commit_tree(repo: &gix::Repository, rev: &str) -> Option<String> {
    let oid = resolve_to_oid(repo, rev)?;
    let object = repo.find_object(oid).ok()?;
    let commit = object.try_to_commit_ref().ok()?;
    Some(commit.tree().to_hex().to_string())
}

/// Internal helper: resolve a rev string to a gix ObjectId.
fn resolve_to_oid(repo: &gix::Repository, rev: &str) -> Option<gix::ObjectId> {
    // Try as full hex OID
    if let Ok(oid) = gix::ObjectId::from_hex(rev.as_bytes()) {
        return Some(oid);
    }
    // Try as reference
    if let Ok(reference) = repo.find_reference(rev)
        .or_else(|_| repo.find_reference(&format!("refs/heads/{}", rev)))
        .or_else(|_| repo.find_reference(&format!("refs/tags/{}", rev)))
    {
        return Some(reference.id().detach());
    }
    None
}
