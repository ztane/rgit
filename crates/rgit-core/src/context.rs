use crate::repo::{CgitRepo, CgitRepoList};

#[derive(Clone, Debug)]
pub struct CgitEnvironment {
    pub cgit_config: String,
    pub http_host: Option<String>,
    pub https: Option<String>,
    pub no_http: Option<String>,
    pub path_info: Option<String>,
    pub query_string: Option<String>,
    pub request_method: Option<String>,
    pub script_name: Option<String>,
    pub server_name: Option<String>,
    pub server_port: Option<String>,
    pub http_cookie: Option<String>,
    pub http_referer: Option<String>,
    pub content_length: u32,
    pub authenticated: bool,
}

impl Default for CgitEnvironment {
    fn default() -> Self {
        CgitEnvironment {
            cgit_config: String::new(),
            http_host: None,
            https: None,
            no_http: None,
            path_info: None,
            query_string: None,
            request_method: None,
            script_name: None,
            server_name: None,
            server_port: None,
            http_cookie: None,
            http_referer: None,
            content_length: 0,
            authenticated: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CgitQuery {
    pub has_symref: bool,
    pub has_oid: bool,
    pub has_difftype: bool,
    pub raw: Option<String>,
    pub repo: Option<String>,
    pub page: Option<String>,
    pub search: Option<String>,
    pub grep: Option<String>,
    pub head: Option<String>,
    pub oid: Option<String>,
    pub oid2: Option<String>,
    pub path: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub period: Option<String>,
    pub ofs: i32,
    pub nohead: bool,
    pub sort: Option<String>,
    pub showmsg: i32,
    pub difftype: i32,
    pub show_all: i32,
    pub context: i32,
    pub ignorews: i32,
    pub follow: i32,
    pub vpath: Option<String>,
}

impl Default for CgitQuery {
    fn default() -> Self {
        CgitQuery {
            has_symref: false,
            has_oid: false,
            has_difftype: false,
            raw: None,
            repo: None,
            page: None,
            search: None,
            grep: None,
            head: None,
            oid: None,
            oid2: None,
            path: None,
            name: None,
            url: None,
            period: None,
            ofs: 0,
            nohead: false,
            sort: None,
            showmsg: 0,
            difftype: 0,
            show_all: 0,
            context: 0,
            ignorews: 0,
            follow: 0,
            vpath: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CgitConfig {
    pub agefile: String,
    pub cache_root: String,
    pub clone_prefix: Option<String>,
    pub clone_url: Option<String>,
    pub favicon: String,
    pub footer: Option<String>,
    pub head_include: Option<String>,
    pub header: Option<String>,
    pub logo: String,
    pub logo_link: Option<String>,
    pub mimetype_file: Option<String>,
    pub module_link: Option<String>,
    pub project_list: Option<String>,
    pub readme: Vec<String>,
    pub css: Vec<String>,
    pub robots: String,
    pub root_title: String,
    pub root_desc: String,
    pub root_readme: Option<String>,
    pub script_name: String,
    pub section: String,
    pub repository_sort: String,
    pub virtual_root: Option<String>,
    pub strict_export: Option<String>,
    pub cache_size: i32,
    pub cache_dynamic_ttl: i32,
    pub cache_max_create_time: i32,
    pub cache_repo_ttl: i32,
    pub cache_root_ttl: i32,
    pub cache_scanrc_ttl: i32,
    pub cache_static_ttl: i32,
    pub cache_about_ttl: i32,
    pub cache_snapshot_ttl: i32,
    pub case_sensitive_sort: i32,
    pub embedded: i32,
    pub enable_filter_overrides: i32,
    pub enable_follow_links: i32,
    pub enable_http_clone: i32,
    pub enable_index_links: i32,
    pub enable_index_owner: i32,
    pub enable_blame: i32,
    pub enable_commit_graph: i32,
    pub enable_log_filecount: i32,
    pub enable_log_linecount: i32,
    pub enable_remote_branches: i32,
    pub enable_subject_links: i32,
    pub enable_html_serving: i32,
    pub enable_tree_linenumbers: i32,
    pub enable_git_config: i32,
    pub local_time: i32,
    pub max_atom_items: i32,
    pub max_repo_count: i32,
    pub max_commit_count: i32,
    pub max_lock_attempts: i32,
    pub max_msg_len: i32,
    pub max_repodesc_len: i32,
    pub max_blob_size: i32,
    pub max_stats: i32,
    pub noplainemail: i32,
    pub noheader: i32,
    pub renamelimit: i32,
    pub remove_suffix: i32,
    pub scan_hidden_path: i32,
    pub section_from_path: i32,
    pub snapshots: i32,
    pub section_sort: i32,
    pub summary_branches: i32,
    pub summary_log: i32,
    pub summary_tags: i32,
    pub difftype: i32,
    pub branch_sort: i32,
    pub commit_sort: i32,
    pub js: Vec<String>,
    pub about_filter: Option<String>,
    pub commit_filter: Option<String>,
    pub source_filter: Option<String>,
    pub email_filter: Option<String>,
    pub owner_filter: Option<String>,
    pub auth_filter: Option<String>,
}

impl Default for CgitConfig {
    fn default() -> Self {
        CgitConfig {
            agefile: "info/web/last-modified".to_string(),
            cache_root: "/var/cache/cgit".to_string(),
            clone_prefix: None,
            clone_url: None,
            favicon: "/favicon.ico".to_string(),
            footer: None,
            head_include: None,
            header: None,
            logo: "/cgit.png".to_string(),
            logo_link: None,
            mimetype_file: None,
            module_link: None,
            project_list: None,
            readme: Vec::new(),
            css: Vec::new(),
            robots: "index, nofollow".to_string(),
            root_title: "Git repository browser".to_string(),
            root_desc: "a fast webinterface for the git dscm".to_string(),
            root_readme: None,
            script_name: "/cgit".to_string(),
            section: String::new(),
            repository_sort: "name".to_string(),
            virtual_root: None,
            strict_export: None,
            cache_size: 0,
            cache_dynamic_ttl: 5,
            cache_max_create_time: 5,
            cache_repo_ttl: 5,
            cache_root_ttl: 5,
            cache_scanrc_ttl: 15,
            cache_static_ttl: -1,
            cache_about_ttl: 15,
            cache_snapshot_ttl: 5,
            case_sensitive_sort: 1,
            embedded: 0,
            enable_filter_overrides: 0,
            enable_follow_links: 0,
            enable_http_clone: 1,
            enable_index_links: 0,
            enable_index_owner: 1,
            enable_blame: 0,
            enable_commit_graph: 0,
            enable_log_filecount: 0,
            enable_log_linecount: 0,
            enable_remote_branches: 0,
            enable_subject_links: 0,
            enable_html_serving: 0,
            enable_tree_linenumbers: 1,
            enable_git_config: 0,
            local_time: 0,
            max_atom_items: 10,
            max_repo_count: 50,
            max_commit_count: 50,
            max_lock_attempts: 5,
            max_msg_len: 80,
            max_repodesc_len: 80,
            max_blob_size: 0,
            max_stats: 0,
            noplainemail: 0,
            noheader: 0,
            renamelimit: -1,
            remove_suffix: 0,
            scan_hidden_path: 0,
            section_from_path: 0,
            snapshots: 0,
            section_sort: 1,
            summary_branches: 10,
            summary_log: 10,
            summary_tags: 10,
            difftype: 0,
            branch_sort: 0,
            commit_sort: 0,
            js: Vec::new(),
            about_filter: None,
            commit_filter: None,
            source_filter: None,
            email_filter: None,
            owner_filter: None,
            auth_filter: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CgitPage {
    pub modified: i64,
    pub expires: i64,
    pub size: usize,
    pub mimetype: String,
    pub charset: String,
    pub filename: Option<String>,
    pub etag: Option<String>,
    pub title: Option<String>,
    pub status: Option<u16>,
    pub statusmsg: Option<String>,
}

impl Default for CgitPage {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        CgitPage {
            modified: now,
            expires: now,
            size: 0,
            mimetype: "text/html".to_string(),
            charset: "UTF-8".to_string(),
            filename: None,
            etag: None,
            title: None,
            status: None,
            statusmsg: None,
        }
    }
}

pub struct CgitContext {
    pub env: CgitEnvironment,
    pub qry: CgitQuery,
    pub cfg: CgitConfig,
    pub repo: Option<usize>,  // index into repolist
    pub page: CgitPage,
    pub repolist: CgitRepoList,
}

impl CgitContext {
    pub fn new() -> Self {
        CgitContext {
            env: CgitEnvironment::default(),
            qry: CgitQuery::default(),
            cfg: CgitConfig::default(),
            repo: None,
            page: CgitPage::default(),
            repolist: CgitRepoList::default(),
        }
    }

    /// Get the current repo, if any.
    pub fn current_repo(&self) -> Option<&CgitRepo> {
        self.repo.map(|i| &self.repolist.repos[i])
    }

    /// Prepare context from environment variables.
    pub fn prepare_from_env(&mut self) {
        fn getenv(name: &str) -> Option<String> {
            std::env::var(name).ok()
        }

        self.env.cgit_config = getenv("CGIT_CONFIG")
            .unwrap_or_else(|| "/etc/cgitrc".to_string());
        self.env.http_host = getenv("HTTP_HOST");
        self.env.https = getenv("HTTPS");
        self.env.no_http = getenv("NO_HTTP");
        self.env.path_info = getenv("PATH_INFO");
        self.env.query_string = getenv("QUERY_STRING");
        self.env.request_method = getenv("REQUEST_METHOD");
        self.env.script_name = getenv("SCRIPT_NAME");
        self.env.server_name = getenv("SERVER_NAME");
        self.env.server_port = getenv("SERVER_PORT");
        self.env.http_cookie = getenv("HTTP_COOKIE");
        self.env.http_referer = getenv("HTTP_REFERER");
        self.env.content_length = getenv("CONTENT_LENGTH")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        self.env.authenticated = true; // no auth filter in Rust version yet

        if let Some(ref sn) = self.env.script_name {
            self.cfg.script_name = sn.clone();
        }
        if let Some(ref qs) = self.env.query_string {
            self.qry.raw = Some(qs.clone());
        }
    }

    /// Get the root URL.
    pub fn rooturl(&self) -> &str {
        if let Some(ref vr) = self.cfg.virtual_root {
            vr
        } else {
            &self.cfg.script_name
        }
    }

    /// Get URL for a repository.
    pub fn repourl(&self, reponame: &str) -> String {
        if self.cfg.virtual_root.is_some() {
            format!("{}{}/", self.rooturl(), reponame)
        } else {
            format!("?r={}", reponame)
        }
    }
}
