/// A tree entry with extracted information.
#[derive(Clone, Debug)]
pub struct TreeEntry {
    pub name: String,
    pub mode: u32,
    pub is_dir: bool,
    pub is_file: bool,
    pub is_symlink: bool,
    pub is_submodule: bool,
    pub size: u64,
    pub oid: String,
}

/// List entries in a tree at a given path within a commit.
/// If path is None or empty, lists the root tree.
pub fn list_tree(repo: &gix::Repository, rev: &str, path: Option<&str>) -> Option<Vec<TreeEntry>> {
    let reference = repo.find_reference(rev)
        .or_else(|_| repo.find_reference(&format!("refs/heads/{}", rev)))
        .ok()?;
    let commit_obj = reference.id().object().ok()?;
    let commit = commit_obj.try_to_commit_ref().ok()?;
    let tree_oid = commit.tree();
    let tree_obj = repo.find_object(tree_oid).ok()?;

    // If we have a path, navigate to that subtree
    let target_tree_data = if let Some(p) = path {
        if p.is_empty() {
            tree_obj.data.to_vec()
        } else {
            // Use the tree's lookup_entry to find the entry at the path
            let tree = repo.find_object(tree_oid).ok()?.peel_to_tree().ok()?;
            let entry = tree.lookup_entry_by_path(p).ok()??;
            let obj = entry.object().ok()?;
            obj.data.to_vec()
        }
    } else {
        tree_obj.data.to_vec()
    };

    let mut entries = Vec::new();
    for entry in gix::objs::TreeRefIter::from_bytes(&target_tree_data) {
        let Ok(entry) = entry else { continue };
        let mode = entry.mode.value() as u32;
        let is_dir = entry.mode.is_tree();
        let is_symlink = entry.mode.is_link();
        let is_submodule = entry.mode.is_commit();
        let is_file = !is_dir && !is_symlink && !is_submodule;

        let size = if is_file || is_symlink {
            if let Ok(obj) = repo.find_object(entry.oid) {
                obj.data.len() as u64
            } else {
                0
            }
        } else {
            0
        };

        entries.push(TreeEntry {
            name: entry.filename.to_string(),
            mode,
            is_dir,
            is_file,
            is_symlink,
            is_submodule,
            size,
            oid: entry.oid.to_hex().to_string(),
        });
    }
    Some(entries)
}

/// Read a blob at a given path within a commit.
/// Returns (blob_oid, content_bytes).
pub fn read_blob(repo: &gix::Repository, rev: &str, path: &str) -> Option<(String, Vec<u8>)> {
    let reference = repo.find_reference(rev)
        .or_else(|_| repo.find_reference(&format!("refs/heads/{}", rev)))
        .ok()?;
    let commit_obj = reference.id().object().ok()?;
    let commit = commit_obj.try_to_commit_ref().ok()?;
    let tree_oid = commit.tree();
    let tree = repo.find_object(tree_oid).ok()?.peel_to_tree().ok()?;
    let entry = tree.lookup_entry_by_path(path).ok()??;
    let oid = entry.oid().to_hex().to_string();
    let obj = entry.object().ok()?;
    Some((oid, obj.data.to_vec()))
}
