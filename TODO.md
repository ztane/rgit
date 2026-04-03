# rgit TODO

## Unimplemented pages
- [ ] blame — annotated source view (`ui-blame.c`, ~317 lines)
- [ ] stats — commit statistics/graphs (`ui-stats.c`, ~435 lines)

## Missing features
- [ ] Side-by-side diff view (`difftype=1` / `side-by-side-diffs=1`)
- [ ] Submodule links in tree view (`module-link` config)
- [ ] Syntax highlighting in tree/blob view (source filter or built-in)
- [ ] Owner auto-detection from file uid in scan-tree (getpwuid)
- [ ] `mimetype.*` config entries (currently parsed but ignored)
- [ ] `project-list` file support in scan-path (partially wired, untested)

## Performance / architecture
- [ ] Replace `diff_stat` / `unified_diff` git CLI calls with gix tree diff API
- [ ] Replace `commit_stats` per-commit git subprocess with gix diffstat
- [ ] Native FastCGI support (avoid fcgiwrap overhead) — original cgit doesn't have this either
- [ ] Async I/O / connection pooling (only relevant for FastCGI mode)

## Polish
- [ ] `max-blob-size` enforcement in tree/blob views
- [ ] Proper `Content-Length` header for blob/plain/clone responses
- [ ] `strict-export` validation in clone endpoints
- [ ] `owner-filter` support
- [ ] `auth-filter` support
- [ ] Commit graph (`enable-commit-graph`)
- [ ] Follow mode in log (`enable-follow-links`)
