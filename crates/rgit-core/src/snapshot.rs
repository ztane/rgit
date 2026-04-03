/// Snapshot format definitions matching C cgit's cgit_snapshot_formats[].
pub struct SnapshotFormat {
    pub suffix: &'static str,
    pub mimetype: &'static str,
    pub bit: u32,
}

pub static SNAPSHOT_FORMATS: &[SnapshotFormat] = &[
    SnapshotFormat { suffix: ".tar", mimetype: "application/x-tar", bit: 1 << 0 },
    SnapshotFormat { suffix: ".tar.gz", mimetype: "application/x-gzip", bit: 1 << 1 },
    SnapshotFormat { suffix: ".tar.bz2", mimetype: "application/x-bzip2", bit: 1 << 2 },
    SnapshotFormat { suffix: ".tar.lz", mimetype: "application/x-lzip", bit: 1 << 3 },
    SnapshotFormat { suffix: ".tar.xz", mimetype: "application/x-xz", bit: 1 << 4 },
    SnapshotFormat { suffix: ".tar.zst", mimetype: "application/x-zstd", bit: 1 << 5 },
    SnapshotFormat { suffix: ".zip", mimetype: "application/x-zip", bit: 1 << 6 },
];

/// Parse a snapshots mask from a string like "tar.gz tar.bz zip" or "all".
/// Matches C cgit's cgit_parse_snapshots_mask.
pub fn parse_snapshots_mask(s: &str) -> i32 {
    // Legacy: if it's a plain integer, return that
    if let Ok(n) = s.parse::<i32>() {
        if n != 0 {
            return 1;
        }
    }

    if s == "all" {
        return i32::MAX;
    }

    let mut rv: i32 = 0;
    for token in s.split_whitespace() {
        for f in SNAPSHOT_FORMATS {
            if token == f.suffix || token == &f.suffix[1..] {
                rv |= f.bit as i32;
                break;
            }
        }
    }
    rv
}
