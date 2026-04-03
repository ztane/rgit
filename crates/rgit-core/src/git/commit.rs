/// Information about a commit, extracted from git objects.
#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub oid: String,
    pub author: String,
    pub author_email: String,
    pub author_date: i64,
    pub author_tz: i32,
    pub committer: String,
    pub committer_email: String,
    pub committer_date: i64,
    pub committer_tz: i32,
    pub subject: String,
    pub msg: String,
}

/// Parse a commit from a gix commit object.
pub fn parse_commit(commit: &gix::objs::CommitRef<'_>, oid: gix::ObjectId) -> CommitInfo {
    let author = commit.author();
    let committer = commit.committer();

    let author_name = author.name.to_string();
    let author_email = format!("<{}>", author.email);
    let committer_name = committer.name.to_string();
    let committer_email = format!("<{}>", committer.email);

    let author_time = author.time().unwrap_or_default();
    let committer_time = committer.time().unwrap_or_default();

    let message = commit.message.to_string();
    let (subject, msg) = match message.find('\n') {
        Some(pos) => {
            let subj = message[..pos].to_string();
            let rest = message[pos..].trim_start_matches('\n').to_string();
            (subj, rest)
        }
        None => (message.clone(), String::new()),
    };

    CommitInfo {
        oid: oid.to_hex().to_string(),
        author: author_name,
        author_email,
        author_date: author_time.seconds,
        author_tz: author_time.offset,
        committer: committer_name,
        committer_email,
        committer_date: committer_time.seconds,
        committer_tz: committer_time.offset,
        subject,
        msg,
    }
}

/// Walk the commit log starting from a given reference, returning up to `max_count` commits.
pub fn walk_log(
    repo: &gix::Repository,
    head: &str,
    max_count: usize,
    skip: usize,
) -> Vec<CommitInfo> {
    let mut commits = Vec::new();

    let Ok(reference) = repo.find_reference(head)
        .or_else(|_| repo.find_reference(&format!("refs/heads/{}", head)))
    else {
        return commits;
    };

    let commit_id = reference.id();

    let Ok(walk) = commit_id.ancestors().all() else {
        return commits;
    };

    let mut count = 0;
    for info in walk {
        let Ok(info) = info else { break };
        count += 1;
        if count <= skip {
            continue;
        }
        let Ok(object) = repo.find_object(info.id) else { continue };
        let Ok(commit_ref) = object.try_to_commit_ref() else { continue };
        commits.push(parse_commit(&commit_ref, info.id));
        if commits.len() >= max_count {
            break;
        }
    }

    commits
}
