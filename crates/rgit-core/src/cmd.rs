/// Command definition matching C cgit's struct cgit_cmd.
pub struct CgitCmd {
    pub name: &'static str,
    pub want_repo: bool,
    pub want_vpath: bool,
    pub is_clone: bool,
}

pub static COMMANDS: &[CgitCmd] = &[
    CgitCmd { name: "HEAD", want_repo: true, want_vpath: false, is_clone: true },
    CgitCmd { name: "atom", want_repo: true, want_vpath: false, is_clone: false },
    CgitCmd { name: "about", want_repo: false, want_vpath: false, is_clone: false },
    CgitCmd { name: "blame", want_repo: true, want_vpath: true, is_clone: false },
    CgitCmd { name: "blob", want_repo: true, want_vpath: false, is_clone: false },
    CgitCmd { name: "commit", want_repo: true, want_vpath: true, is_clone: false },
    CgitCmd { name: "diff", want_repo: true, want_vpath: true, is_clone: false },
    CgitCmd { name: "info", want_repo: true, want_vpath: false, is_clone: true },
    CgitCmd { name: "log", want_repo: true, want_vpath: true, is_clone: false },
    CgitCmd { name: "ls_cache", want_repo: false, want_vpath: false, is_clone: false },
    CgitCmd { name: "objects", want_repo: true, want_vpath: false, is_clone: true },
    CgitCmd { name: "patch", want_repo: true, want_vpath: true, is_clone: false },
    CgitCmd { name: "plain", want_repo: true, want_vpath: false, is_clone: false },
    CgitCmd { name: "rawdiff", want_repo: true, want_vpath: true, is_clone: false },
    CgitCmd { name: "refs", want_repo: true, want_vpath: false, is_clone: false },
    CgitCmd { name: "repolist", want_repo: false, want_vpath: false, is_clone: false },
    CgitCmd { name: "snapshot", want_repo: true, want_vpath: false, is_clone: false },
    CgitCmd { name: "stats", want_repo: true, want_vpath: true, is_clone: false },
    CgitCmd { name: "summary", want_repo: true, want_vpath: false, is_clone: false },
    CgitCmd { name: "tag", want_repo: true, want_vpath: false, is_clone: false },
    CgitCmd { name: "tree", want_repo: true, want_vpath: true, is_clone: false },
];

/// Look up a command by page name, defaulting to "summary" (repo) or "repolist" (no repo).
pub fn get_cmd(page: Option<&str>, has_repo: bool) -> Option<&'static CgitCmd> {
    let page = page.unwrap_or(if has_repo { "summary" } else { "repolist" });
    COMMANDS.iter().find(|c| c.name == page)
}
