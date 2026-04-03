# rgit

A Rust rewrite of [cgit](https://git.zx2c4.com/cgit/about/), the fast web
interface for Git repositories.

rgit is a drop-in CGI replacement for cgit. It reads the same `cgitrc`
configuration file, produces the same HTML output, and passes cgit's original
shell test suite unmodified.

## Status

All 14 original cgit test files pass (155 tests total):

| Test | Description |
|------|-------------|
| t0001 | Git version validation |
| t0010 | HTML validation (tidy) |
| t0020 | Cache validation |
| t0101 | Index / repository list |
| t0102 | Summary page |
| t0103 | Log page |
| t0104 | Tree browser |
| t0105 | Commit detail |
| t0106 | Diff page |
| t0107 | Snapshot archives |
| t0108 | Patch (format-patch) |
| t0109 | No $HOME access |
| t0110 | Raw diff |
| t0111 | Exec + Lua filters |

## Building

```sh
cargo build --release
```

The binary is named `cgit` (for compatibility with the original test suite and
CGI deployments).

## Architecture

The project is a Cargo workspace with three crates:

- **rgit-core** -- data types, configuration, HTML output, git abstraction,
  caching, filter infrastructure
- **rgit-ui** -- page renderers (repolist, summary, log, tree, commit, diff,
  snapshot, patch, rawdiff, about)
- **rgit-bin** -- CGI binary entry point

### Key design decisions

- **No unsafe code** -- `#![forbid(unsafe_code)]` in all crates
- **Rust 2024 edition** -- environment variables are never mutated
- **[gix](https://github.com/Byron/gitoxide)** for native Git repository
  access (refs, commits, trees, blobs)
- **git CLI** for byte-exact output where tests compare output verbatim
  (format-patch, raw diff, archives) -- with `GIT_CONFIG_NOSYSTEM=1` and
  `HOME` removed to prevent config leakage
- **[mlua](https://github.com/mlua-rs/mlua)** (Lua 5.4, vendored) for Lua
  filter scripting support
- Thread-local output buffer stack supporting nested capture (cache + filters)
- FNV-1 hash for cache slot filenames (matching the C implementation exactly)

## Running tests

The test suite lives in the original cgit source tree. A Makefile is provided
to build, symlink the binary, and run the tests:

```sh
make test
```

Or run individual tests:

```sh
make build
cd ../cgit/tests && ./t0111-filter.sh -v
```

## Configuration

rgit reads the standard `cgitrc` file. Set `CGIT_CONFIG` to point to your
configuration:

```sh
CGIT_CONFIG=/etc/cgitrc ./cgit
```

See the [cgit documentation](https://git.zx2c4.com/cgit/tree/cgitrc.5.txt)
for the full list of configuration options.

## License

Licensed under the GNU General Public License v2, the same license as cgit.
See [COPYING](COPYING) for the full text.
