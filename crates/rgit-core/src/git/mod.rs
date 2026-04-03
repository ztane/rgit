pub mod commit;
pub mod refs;
pub mod tree;
pub mod diff;

use std::path::Path;

/// Open a git repository at the given path using gix.
/// Uses isolated open options to avoid reading system/user git config.
pub fn open_repo(path: &str) -> Option<gix::Repository> {
    let path = Path::new(path);
    let mut open_opts = gix::open::Options::isolated();
    open_opts.permissions.config.git_binary = false;
    gix::open_opts(path, open_opts).ok()
}
