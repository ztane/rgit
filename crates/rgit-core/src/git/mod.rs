pub mod commit;
pub mod refs;
pub mod tree;
pub mod diff;

use std::path::Path;
use std::process::Command;

/// Open a git repository at the given path using gix.
/// Uses isolated open options to avoid reading system/user git config.
pub fn open_repo(path: &str) -> Option<gix::Repository> {
    let path = Path::new(path);
    let mut open_opts = gix::open::Options::isolated();
    open_opts.permissions.config.git_binary = false;
    gix::open_opts(path, open_opts).ok()
}

/// Create a `git` Command with environment overrides to prevent
/// accessing $HOME/.gitconfig and system git config.
pub fn git_command(repo_path: &str) -> Command {
    let mut cmd = git_command_bare();
    cmd.arg("--git-dir").arg(repo_path);
    cmd
}

/// Create a bare `git` Command with environment overrides but no --git-dir.
pub fn git_command_bare() -> Command {
    let mut cmd = Command::new("git");
    cmd.env("GIT_CONFIG_NOSYSTEM", "1");
    cmd.env("GIT_ATTR_NOSYSTEM", "1");
    cmd.env_remove("HOME");
    cmd.env_remove("XDG_CONFIG_HOME");
    cmd
}
