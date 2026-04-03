use super::commit::CommitInfo;

/// Information about a branch reference.
#[derive(Clone, Debug)]
pub struct BranchInfo {
    pub name: String,
    pub commit: Option<CommitInfo>,
}

/// Information about a tag reference.
#[derive(Clone, Debug)]
pub struct TagInfo {
    pub name: String,
    pub tagged_oid: String,
    pub tagger: Option<String>,
    pub tagger_email: Option<String>,
    pub tagger_date: i64,
    pub tagger_tz: i32,
    pub msg: Option<String>,
    pub commit: Option<CommitInfo>,
}

/// List all local branches with their latest commit info.
pub fn list_branches(repo: &gix::Repository) -> Vec<BranchInfo> {
    let mut branches = Vec::new();
    let Ok(refs) = repo.references() else {
        return branches;
    };
    let Ok(branch_refs) = refs.local_branches() else {
        return branches;
    };
    for reference in branch_refs.flatten() {
        let name = reference.name().shorten().to_string();
        let commit = resolve_commit(repo, reference.id().detach());
        branches.push(BranchInfo { name, commit });
    }
    branches
}

/// List all tags with their info.
pub fn list_tags(repo: &gix::Repository) -> Vec<TagInfo> {
    let mut tags = Vec::new();
    let Ok(refs) = repo.references() else {
        return tags;
    };
    let Ok(tag_refs) = refs.tags() else {
        return tags;
    };
    for reference in tag_refs.flatten() {
        let name = reference.name().shorten().to_string();
        let id = reference.id().detach();
        let Ok(object) = repo.find_object(id) else { continue };

        match object.kind {
            gix::object::Kind::Tag => {
                let Ok(tag_ref) = object.try_to_tag_ref() else { continue };
                let tagger_sig = tag_ref.tagger().ok().flatten();
                let (tagger, tagger_email, tagger_date, tagger_tz) = match tagger_sig {
                    Some(sig) => {
                        let time = sig.time().unwrap_or_default();
                        (
                            Some(sig.name.to_string()),
                            Some(format!("<{}>", sig.email)),
                            time.seconds,
                            time.offset,
                        )
                    }
                    None => (None, None, 0, 0),
                };
                let msg = if tag_ref.message.is_empty() {
                    None
                } else {
                    Some(tag_ref.message.to_string())
                };
                let target_id = tag_ref.target();
                let commit = resolve_commit(repo, target_id);
                tags.push(TagInfo {
                    name,
                    tagged_oid: target_id.to_hex().to_string(),
                    tagger,
                    tagger_email,
                    tagger_date,
                    tagger_tz,
                    msg,
                    commit,
                });
            }
            gix::object::Kind::Commit => {
                let commit = resolve_commit(repo, id);
                tags.push(TagInfo {
                    name,
                    tagged_oid: id.to_hex().to_string(),
                    tagger: None,
                    tagger_email: None,
                    tagger_date: 0,
                    tagger_tz: 0,
                    msg: None,
                    commit,
                });
            }
            _ => {}
        }
    }
    tags
}

/// Guess the default branch by looking at HEAD.
pub fn guess_default_branch(repo: &gix::Repository) -> Option<String> {
    let head = repo.head().ok()?;
    let name = head.referent_name()?;
    let short = name.shorten().to_string();
    Some(short)
}

/// Find the default branch: if the specified defbranch exists, use it;
/// otherwise find the first branch, or guess from HEAD.
pub fn find_default_branch(repo: &gix::Repository, defbranch: Option<&str>) -> Option<String> {
    if let Some(db) = defbranch {
        // Check if this branch exists
        let ref_name = format!("refs/heads/{}", db);
        if repo.find_reference(&ref_name).is_ok() {
            return Some(db.to_string());
        }
    }
    // Try HEAD
    if let Some(branch) = guess_default_branch(repo) {
        return Some(branch);
    }
    // Fall back to first branch
    let branches = list_branches(repo);
    branches.first().map(|b| b.name.clone())
}

fn resolve_commit(repo: &gix::Repository, oid: gix::ObjectId) -> Option<CommitInfo> {
    let object = repo.find_object(oid).ok()?;
    // Peel to commit if needed
    let commit_obj = if object.kind == gix::object::Kind::Commit {
        object
    } else {
        // Try to peel tag -> commit
        object.peel_to_kind(gix::object::Kind::Commit).ok()?
    };
    let commit_ref = commit_obj.try_to_commit_ref().ok()?;
    Some(super::commit::parse_commit(&commit_ref, oid))
}
