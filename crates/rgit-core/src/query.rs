use crate::context::CgitContext;
use crate::html::http_parse_querystring;
use crate::repo::trim_end;

/// Parse the querystring and populate ctx.qry fields.
/// Matches C cgit's querystring_cb.
pub fn parse_querystring(ctx: &mut CgitContext) {
    let raw = match ctx.qry.raw.clone() {
        Some(r) => r,
        None => return,
    };
    http_parse_querystring(&raw, |name, value| {
        querystring_cb(ctx, name, value);
    });
}

fn querystring_cb(ctx: &mut CgitContext, name: &str, value: &str) {
    let value = if value.is_empty() && name != "url" { "" } else { value };

    match name {
        "r" => {
            ctx.qry.repo = Some(value.to_string());
            ctx.repo = ctx.repolist.get_repoinfo(value);
        }
        "p" => ctx.qry.page = Some(value.to_string()),
        "url" => {
            let v = if value.starts_with('/') { &value[1..] } else { value };
            ctx.qry.url = Some(v.to_string());
            cgit_parse_url(ctx, v);
        }
        "qt" => ctx.qry.grep = Some(value.to_string()),
        "q" => ctx.qry.search = Some(value.to_string()),
        "h" => {
            ctx.qry.head = Some(value.to_string());
            ctx.qry.has_symref = true;
        }
        "id" => {
            ctx.qry.oid = Some(value.to_string());
            ctx.qry.has_oid = true;
        }
        "id2" => {
            ctx.qry.oid2 = Some(value.to_string());
            ctx.qry.has_oid = true;
        }
        "ofs" => ctx.qry.ofs = value.parse().unwrap_or(0),
        "path" => ctx.qry.path = Some(trim_end(value, '/')),
        "name" => ctx.qry.name = Some(value.to_string()),
        "s" => ctx.qry.sort = Some(value.to_string()),
        "showmsg" => ctx.qry.showmsg = value.parse().unwrap_or(0),
        "period" => ctx.qry.period = Some(value.to_string()),
        "dt" => {
            ctx.qry.difftype = value.parse().unwrap_or(0);
            ctx.qry.has_difftype = true;
        }
        "ss" => {
            ctx.qry.difftype = if value.parse::<i32>().unwrap_or(0) != 0 { 1 } else { 0 };
            ctx.qry.has_difftype = true;
        }
        "all" => ctx.qry.show_all = value.parse().unwrap_or(0),
        "context" => ctx.qry.context = value.parse().unwrap_or(0),
        "ignorews" => ctx.qry.ignorews = value.parse().unwrap_or(0),
        "follow" => ctx.qry.follow = value.parse().unwrap_or(0),
        _ => {}
    }
}

/// Parse a virtual URL path: [repo ['/' cmd ['/' path]]]
/// Matches C cgit's cgit_parse_url.
pub fn cgit_parse_url(ctx: &mut CgitContext, url: &str) {
    if url.is_empty() {
        return;
    }

    ctx.qry.page = None;

    // First try the full URL as a repo name
    if let Some(idx) = ctx.repolist.get_repoinfo(url) {
        ctx.repo = Some(idx);
        ctx.qry.repo = Some(ctx.repolist.repos[idx].url.clone());
        return;
    }

    // Try progressively shorter prefixes
    let mut best_repo: Option<usize> = None;
    let mut best_cmd_pos: usize = 0;

    let mut search_from = 0;
    while let Some(slash_pos) = url[search_from..].find('/') {
        let slash_pos = search_from + slash_pos;
        let prefix = &url[..slash_pos];
        if let Some(idx) = ctx.repolist.get_repoinfo(prefix) {
            best_repo = Some(idx);
            best_cmd_pos = slash_pos;
        }
        search_from = slash_pos + 1;
    }

    if let Some(idx) = best_repo {
        ctx.repo = Some(idx);
        ctx.qry.repo = Some(ctx.repolist.repos[idx].url.clone());
        let after_repo = &url[best_cmd_pos + 1..];
        if let Some(slash_pos) = after_repo.find('/') {
            let cmd = &after_repo[..slash_pos];
            if !cmd.is_empty() {
                ctx.qry.page = Some(cmd.to_string());
            }
            let path = &after_repo[slash_pos + 1..];
            if !path.is_empty() {
                ctx.qry.path = Some(trim_end(path, '/'));
            }
        } else if !after_repo.is_empty() {
            ctx.qry.page = Some(after_repo.to_string());
        }
    }
}
