use crate::context::CgitConfig;

#[derive(Clone, Debug)]
pub struct CgitRepo {
    pub url: String,
    pub name: String,
    pub path: Option<String>,
    pub desc: String,
    pub extra_head_content: Option<String>,
    pub owner: Option<String>,
    pub homepage: Option<String>,
    pub defbranch: Option<String>,
    pub module_link: Option<String>,
    pub readme: Vec<String>,
    pub section: String,
    pub clone_url: Option<String>,
    pub logo: Option<String>,
    pub logo_link: Option<String>,
    pub snapshot_prefix: Option<String>,
    pub snapshots: i32,
    pub enable_blame: i32,
    pub enable_commit_graph: i32,
    pub enable_follow_links: i32,
    pub enable_log_filecount: i32,
    pub enable_log_linecount: i32,
    pub enable_remote_branches: i32,
    pub enable_subject_links: i32,
    pub enable_html_serving: i32,
    pub max_stats: i32,
    pub branch_sort: i32,
    pub commit_sort: i32,
    pub mtime: i64,
    pub about_filter: Option<String>,
    pub commit_filter: Option<String>,
    pub source_filter: Option<String>,
    pub email_filter: Option<String>,
    pub owner_filter: Option<String>,
    pub hide: i32,
    pub ignore: i32,
}

impl CgitRepo {
    pub fn new(url: &str, cfg: &CgitConfig) -> Self {
        let trimmed_url = trim_end(url, '/');
        CgitRepo {
            name: trimmed_url.clone(),
            url: trimmed_url,
            path: None,
            desc: "[no description]".to_string(),
            extra_head_content: None,
            owner: None,
            homepage: None,
            defbranch: None,
            module_link: cfg.module_link.clone(),
            readme: cfg.readme.clone(),
            section: cfg.section.clone(),
            clone_url: cfg.clone_url.clone(),
            logo: None,
            logo_link: None,
            snapshot_prefix: None,
            snapshots: cfg.snapshots,
            enable_blame: cfg.enable_blame,
            enable_commit_graph: cfg.enable_commit_graph,
            enable_follow_links: cfg.enable_follow_links,
            enable_log_filecount: cfg.enable_log_filecount,
            enable_log_linecount: cfg.enable_log_linecount,
            enable_remote_branches: cfg.enable_remote_branches,
            enable_subject_links: cfg.enable_subject_links,
            enable_html_serving: cfg.enable_html_serving,
            max_stats: cfg.max_stats,
            branch_sort: cfg.branch_sort,
            commit_sort: cfg.commit_sort,
            mtime: -1,
            about_filter: cfg.about_filter.clone(),
            commit_filter: cfg.commit_filter.clone(),
            source_filter: cfg.source_filter.clone(),
            email_filter: cfg.email_filter.clone(),
            owner_filter: cfg.owner_filter.clone(),
            hide: 0,
            ignore: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CgitRepoList {
    pub repos: Vec<CgitRepo>,
}

impl CgitRepoList {
    pub fn add_repo(&mut self, repo: CgitRepo) -> usize {
        self.repos.push(repo);
        self.repos.len() - 1
    }

    pub fn get_repoinfo(&self, url: &str) -> Option<usize> {
        for (i, repo) in self.repos.iter().enumerate() {
            if repo.ignore != 0 {
                continue;
            }
            if repo.url == url {
                return Some(i);
            }
        }
        None
    }
}

/// Get the mtime for a repo (cached). Uses ref file stat as fallback.
pub fn get_repo_modtime(repo: &mut CgitRepo, agefile: &str) -> i64 {
    if repo.mtime != -1 {
        return repo.mtime;
    }
    let Some(ref path) = repo.path else { return 0 };

    // Try agefile first
    if !agefile.is_empty() {
        let agefile_path = std::path::Path::new(path).join(agefile);
        if let Ok(contents) = std::fs::read_to_string(&agefile_path) {
            if let Ok(ts) = contents.trim().parse::<i64>() {
                if ts > 0 {
                    repo.mtime = ts;
                    return ts;
                }
            }
        }
    }

    // Try stat on refs/heads/{defbranch} or refs/heads/master
    let defbranch = repo.defbranch.as_deref().unwrap_or("master");
    let ref_path = std::path::Path::new(path).join("refs/heads").join(defbranch);
    if let Ok(meta) = std::fs::metadata(&ref_path) {
        if let Ok(modified) = meta.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                repo.mtime = duration.as_secs() as i64;
                return repo.mtime;
            }
        }
    }

    // Try packed-refs
    let packed_path = std::path::Path::new(path).join("packed-refs");
    if let Ok(meta) = std::fs::metadata(&packed_path) {
        if let Ok(modified) = meta.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                repo.mtime = duration.as_secs() as i64;
                return repo.mtime;
            }
        }
    }

    0
}

pub fn trim_end(s: &str, c: char) -> String {
    let trimmed = s.trim_end_matches(c);
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.to_string()
    }
}

pub fn ensure_end(s: &str, c: char) -> String {
    if s.ends_with(c) {
        s.to_string()
    } else {
        format!("{}{}", s, c)
    }
}
