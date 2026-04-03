# rgit -- Rust rewrite of cgit

## Project overview

Drop-in CGI replacement for cgit (C web interface for Git repos). The binary
is named `cgit` for test compatibility. Source lives in `crates/`.

## Key rules

### No unsafe code
- `#![forbid(unsafe_code)]` in all crates. No exceptions.
- `std::env::set_var` / `remove_var` are unsafe in edition 2024. Never mutate
  the process environment.
  - Git subprocesses: pass overrides via `Command::env()` / `env_remove()`.
  - gix: configure via `open::Options::isolated()` to skip system config.
  - Virtual env stored in `CgitContext`.

### Binary name
- Compiled binary: `cgit` (test suite expects it).
- Crate names: `rgit-core`, `rgit-ui`, `rgit-bin`.

### Testing
- Acceptance tests: original shell tests in `../cgit/tests/`.
- Run all: `make test`
- Run one: `make build && cd ../cgit/tests && ./t0111-filter.sh -v`

### Build
- `cargo build --release`
- `make build` also symlinks `target/release/cgit` to `../cgit/cgit`

### Version output
`cgit --version` must output exactly:
```
CGit v1.3 | https://git.zx2c4.com/cgit/about/

Compiled in features:
[+] Lua scripting
[+] Linux sendfile() usage
```

### Git access
- Use **gix** for repository access (refs, commits, trees, blobs).
- Shell out to `git` CLI only for byte-exact output (format-patch, raw diff,
  archives). Always use `git::git_command()` helper which sets
  `GIT_CONFIG_NOSYSTEM=1`, `GIT_ATTR_NOSYSTEM=1`, and removes `HOME` /
  `XDG_CONFIG_HOME`.

### Filters
- Exec filters: subprocess with stdin/stdout piping.
- Lua filters: `mlua` crate (Lua 5.4 vendored). Functions `filter_open`,
  `filter_write`, `filter_close` with `html()` etc. registered as globals.
- Use `filter::with_filter()` helper which captures HtmlOutput, runs the
  filter, and writes output back.

### Output buffering
- `HtmlOutput` uses a thread-local stack of `Vec<u8>` buffers.
- `start_capture()` / `stop_capture()` push/pop the stack.
- Nesting is supported (cache capture + filter capture).

## Crate structure

```
crates/
  rgit-core/src/
    config.rs    -- cgitrc parser
    context.rs   -- CgitContext, CgitConfig, CgitEnvironment, CgitPage
    query.rs     -- querystring parsing, URL routing
    repo.rs      -- CgitRepo, CgitRepoList
    html.rs      -- HTML/URL encoding, output buffering
    cmd.rs       -- command dispatch table
    cache.rs     -- FNV-1 file cache with TTL and locking
    filter.rs    -- exec + Lua filter runners
    snapshot.rs  -- snapshot format registry
    macros.rs    -- $VAR expansion
    git/
      mod.rs     -- git_command() helper, open_repo()
      commit.rs  -- CommitInfo, walk_log()
      refs.rs    -- branch/tag iteration
      diff.rs    -- diff_stat, unified_diff, resolve_rev
      tree.rs    -- tree listing, blob reading
  rgit-ui/src/
    shared.rs    -- HTTP headers, HTML layout, link helpers
    repolist.rs  -- index page
    summary.rs   -- summary page
    log.rs       -- log page
    tree.rs      -- tree browser + blob display
    commit.rs    -- commit detail
    diff.rs      -- diff page
    snapshot.rs  -- archive downloads
    patch.rs     -- format-patch output
    rawdiff.rs   -- raw diff output
    about.rs     -- readme display
  rgit-bin/src/
    main.rs      -- CGI entry point
```
