use rgit_core::context::CgitContext;
use rgit_core::html::*;

const CGIT_VERSION: &str = "v1.3";

/// Format a time_t as an HTTP date string.
fn http_date(t: i64) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let dt = UNIX_EPOCH + Duration::from_secs(t as u64);
    // Format: "Sun, 06 Nov 1994 08:49:37 GMT"
    let secs = t;
    let days = secs / 86400;
    // Use a simple algorithm to compute the date components
    // We need to match the C gmtime_r output exactly
    let (year, month, day, hour, min, sec, wday) = unix_to_gmt(secs);
    let day_names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let month_names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let _ = (dt, days);
    format!(
        "{}, {:02} {} {:04} {:02}:{:02}:{:02} GMT",
        day_names[wday as usize],
        day,
        month_names[month as usize],
        year,
        hour,
        min,
        sec
    )
}

fn unix_to_gmt(timestamp: i64) -> (i32, i32, i32, i32, i32, i32, i32) {
    let secs = timestamp % 86400;
    let mut days = timestamp / 86400;
    if secs < 0 {
        days -= 1;
    }
    let secs = ((timestamp % 86400) + 86400) % 86400;

    let hour = (secs / 3600) as i32;
    let min = ((secs % 3600) / 60) as i32;
    let sec = (secs % 60) as i32;

    // Day of week: Jan 1 1970 was Thursday (4)
    let wday = ((days % 7 + 4 + 7) % 7) as i32;

    // Civil date from days since epoch
    let mut y = 1970i64;
    loop {
        let days_in_year = if is_leap(y as i32) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        y += 1;
    }

    let leap = is_leap(y as i32);
    let month_days = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0i32;
    for md in &month_days {
        if days < *md {
            break;
        }
        days -= md;
        month += 1;
    }

    (y as i32, month, days as i32 + 1, hour, min, sec, wday)
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Print CGI HTTP headers.
pub fn print_http_headers(ctx: &CgitContext) {
    if let Some(ref no_http) = ctx.env.no_http {
        if no_http == "1" {
            return;
        }
    }

    if let Some(status) = ctx.page.status {
        let msg = ctx.page.statusmsg.as_deref().unwrap_or("OK");
        html(&format!("Status: {} {}\n", status, msg));
    }

    if !ctx.page.mimetype.is_empty() && !ctx.page.charset.is_empty() {
        html(&format!(
            "Content-Type: {}; charset={}\n",
            ctx.page.mimetype, ctx.page.charset
        ));
    } else if !ctx.page.mimetype.is_empty() {
        html(&format!("Content-Type: {}\n", ctx.page.mimetype));
    }

    if ctx.page.size > 0 {
        html(&format!("Content-Length: {}\n", ctx.page.size));
    }

    if let Some(ref filename) = ctx.page.filename {
        html("Content-Disposition: inline; filename=\"");
        html_header_arg_in_quotes(filename);
        html("\"\n");
    }

    if !ctx.env.authenticated {
        html("Cache-Control: no-cache, no-store\n");
    }

    html(&format!("Last-Modified: {}\n", http_date(ctx.page.modified)));
    html(&format!("Expires: {}\n", http_date(ctx.page.expires)));

    if let Some(ref etag) = ctx.page.etag {
        html(&format!("ETag: \"{}\"\n", etag));
    }

    html("\n");

    if let Some(ref method) = ctx.env.request_method {
        if method == "HEAD" {
            std::process::exit(0);
        }
    }
}

/// Print the HTML document start (DOCTYPE, <html>, <head>, etc).
pub fn print_docstart(ctx: &CgitContext) {
    if ctx.cfg.embedded != 0 {
        if let Some(ref header) = ctx.cfg.header {
            let _ = html_include(header);
        }
        return;
    }

    html("<!DOCTYPE html>\n");
    html("<html lang='en'>\n");
    html("<head>\n");
    html("<title>");
    if let Some(ref title) = ctx.page.title {
        html_txt(title);
    }
    html("</title>\n");
    html(&format!(
        "<meta name='generator' content='cgit {}'/>\n",
        CGIT_VERSION
    ));
    if !ctx.cfg.robots.is_empty() {
        html(&format!(
            "<meta name='robots' content='{}'/>\n",
            ctx.cfg.robots
        ));
    }

    if !ctx.cfg.css.is_empty() {
        for css in &ctx.cfg.css {
            if css.is_empty() {
                continue;
            }
            html("<link rel='stylesheet' type='text/css' href='");
            html_attr(css);
            html("'/>\n");
        }
    } else {
        html("<link rel='stylesheet' type='text/css' href='");
        html_attr("/cgit.css");
        html("'/>\n");
    }

    if !ctx.cfg.js.is_empty() {
        for js in &ctx.cfg.js {
            if js.is_empty() {
                continue;
            }
            html("<script type='text/javascript' src='");
            html_attr(js);
            html("'></script>\n");
        }
    } else {
        html("<script type='text/javascript' src='");
        html_attr("/cgit.js");
        html("'></script>\n");
    }

    if !ctx.cfg.favicon.is_empty() {
        html("<link rel='shortcut icon' href='");
        html_attr(&ctx.cfg.favicon);
        html("'/>\n");
    }

    if let Some(ref head_include) = ctx.cfg.head_include {
        let _ = html_include(head_include);
    }

    if let Some(repo_idx) = ctx.repo {
        let repo = &ctx.repolist.repos[repo_idx];
        if let Some(ref extra) = repo.extra_head_content {
            html(extra);
        }
    }

    html("</head>\n");
    html("<body>\n");
    if let Some(ref header) = ctx.cfg.header {
        let _ = html_include(header);
    }
}

/// Print the HTML document end.
pub fn print_docend(ctx: &CgitContext) {
    html("</div> <!-- class=content -->\n");
    if ctx.cfg.embedded != 0 {
        html("</div> <!-- id=cgit -->\n");
        if let Some(ref footer) = ctx.cfg.footer {
            let _ = html_include(footer);
        }
        return;
    }
    if let Some(ref footer) = ctx.cfg.footer {
        let _ = html_include(footer);
    } else {
        html(&format!(
            "<div class='footer'>generated by <a href='https://git.zx2c4.com/cgit/about/'>cgit {}</a> ",
            CGIT_VERSION
        ));
        html("(<a href='https://git-scm.com/'>git</a>) at ");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        html_txt(&format_iso8601(now, 0));
        html("</div>\n");
    }
    html("</div> <!-- id=cgit -->\n");
    html("</body>\n</html>\n");
}

/// Print the page header (tabs, navigation).
pub fn print_pageheader(ctx: &CgitContext) {
    html("<div id='cgit'>");
    if ctx.env.authenticated && ctx.cfg.noheader == 0 {
        print_header_table(ctx);
    }

    html("<table class='tabs'><tr><td>\n");
    if ctx.env.authenticated && ctx.repo.is_some() {
        // Repo page tabs
        let current_page = ctx.qry.page.as_deref().unwrap_or("summary");
        let hc = |name: &str| -> Option<&str> {
            if name == current_page { Some("tab") } else { None }
        };

        summary_link(ctx, "summary", None, hc("summary"), ctx.qry.head.as_deref());
        refs_link(ctx, "refs", None, hc("refs"), ctx.qry.head.as_deref(), ctx.qry.oid.as_deref(), None);
        log_link(ctx, "log", None, hc("log"), ctx.qry.head.as_deref());
        reporevlink(ctx, "tree", "tree", None, hc("tree"), ctx.qry.head.as_deref(), ctx.qry.oid.as_deref(), ctx.qry.vpath.as_deref());
        commit_link(ctx, "commit", None, hc("commit"), ctx.qry.head.as_deref(), ctx.qry.oid.as_deref(), ctx.qry.vpath.as_deref());
        reporevlink(ctx, "diff", "diff", None, hc("diff"), ctx.qry.head.as_deref(), ctx.qry.oid.as_deref(), ctx.qry.vpath.as_deref());
        html("</td><td class='form'>");
        html("<form class='right' method='get' action='");
        if let Some(vr) = &ctx.cfg.virtual_root {
            let repo_idx = ctx.repo.unwrap();
            let repo = &ctx.repolist.repos[repo_idx];
            html_url_path(vr);
            html_url_path(&repo.url);
            if !repo.url.ends_with('/') {
                html("/");
            }
            html_url_path("log");
            html("/");
            if let Some(vpath) = &ctx.qry.vpath {
                html_url_path(vpath);
            }
        }
        html("'>\n");
        // Hidden fields
        if let Some(h) = &ctx.qry.head {
            if let Some(repo_idx) = ctx.repo {
                let repo = &ctx.repolist.repos[repo_idx];
                if let Some(db) = &repo.defbranch {
                    if h != db {
                        html_hidden("h", h);
                    }
                }
            }
        }
        html("<select name='qt'>\n");
        html_option("grep", "log msg", ctx.qry.grep.as_deref());
        html_option("author", "author", ctx.qry.grep.as_deref());
        html_option("committer", "committer", ctx.qry.grep.as_deref());
        html_option("range", "range", ctx.qry.grep.as_deref());
        html("</select>\n");
        html("<input class='txt' type='search' size='10' name='q' value='");
        if let Some(search) = &ctx.qry.search {
            html_attr(search);
        }
        html("'/>\n");
        html("<input type='submit' value='search'/>\n");
        html("</form>");
    } else if ctx.env.authenticated {
        // Index page tabs
        let rooturl = ctx.rooturl();
        html("<a href='");
        html_attr(rooturl);
        html("'>index</a>");
        if ctx.cfg.root_readme.is_some() {
            html("<a href='");
            html_attr(rooturl);
            html("?p=about'>about</a>");
        }
        html("</td><td class='form'>");
        let currenturl = current_url(ctx);
        html("<form method='get' action='");
        html_attr(&currenturl);
        html("'>\n");
        html("<input type='search' name='q' size='10' value='");
        if let Some(search) = &ctx.qry.search {
            html_attr(search);
        }
        html("'/>\n");
        html("<input type='submit' value='search'/>\n");
        html("</form>");
    }
    html("</td></tr></table>\n");
    html("<div class='content'>");
}

/// Print layout start (headers + doc start + page header).
pub fn print_layout_start(ctx: &CgitContext) {
    print_http_headers(ctx);
    print_docstart(ctx);
    print_pageheader(ctx);
}

/// Print layout end (doc end).
pub fn print_layout_end(ctx: &CgitContext) {
    print_docend(ctx);
}

/// Helper: emit a repolink opening tag and return the query delimiter.
/// This matches C cgit's repolink() function.
pub fn repolink(ctx: &CgitContext, title: Option<&str>, class: Option<&str>, page: Option<&str>, head: Option<&str>, path: Option<&str>) -> &'static str {
    let repo_idx = ctx.repo.unwrap();
    let repo = &ctx.repolist.repos[repo_idx];

    html("<a");
    if let Some(t) = title {
        html(" title='");
        html_attr(t);
        html("'");
    }
    if let Some(c) = class {
        html(" class='");
        html_attr(c);
        html("'");
    }
    html(" href='");
    let delim;
    if let Some(vr) = &ctx.cfg.virtual_root {
        html_url_path(vr);
        html_url_path(&repo.url);
        if !repo.url.ends_with('/') {
            html("/");
        }
        if let Some(p) = page {
            html_url_path(p);
            html("/");
            if let Some(path) = path {
                html_url_path(path);
            }
        }
        delim = "?";
    } else {
        html_url_path(&ctx.cfg.script_name);
        html("?url=");
        html_url_arg(&repo.url);
        if !repo.url.ends_with('/') {
            html("/");
        }
        if let Some(p) = page {
            html_url_arg(p);
            html("/");
            if let Some(path) = path {
                html_url_arg(path);
            }
        }
        delim = "&amp;";
    }
    if let Some(h) = head {
        if let Some(db) = &repo.defbranch {
            if h != db {
                html(delim);
                html("h=");
                html_url_arg(h);
                return "&amp;";
            }
        }
    }
    delim
}

/// Emit a reporevlink (page link with optional revision).
pub fn reporevlink(ctx: &CgitContext, page: &str, name: &str, title: Option<&str>, class: Option<&str>, head: Option<&str>, rev: Option<&str>, path: Option<&str>) {
    let delim = repolink(ctx, title, class, Some(page), head, path);
    if let Some(r) = rev {
        if let Some(h) = &ctx.qry.head {
            if r != h {
                html(delim);
                html("id=");
                html_url_arg(r);
            }
        }
    }
    html("'>");
    html_txt(name);
    html("</a>");
}

/// Emit a summary link.
pub fn summary_link(ctx: &CgitContext, name: &str, title: Option<&str>, class: Option<&str>, head: Option<&str>) {
    reporevlink(ctx, "", name, title, class, head, None, None);
}

/// Emit a log link (simplified for summary page use).
pub fn log_link(ctx: &CgitContext, name: &str, title: Option<&str>, class: Option<&str>, head: Option<&str>) {
    let _delim = repolink(ctx, title, class, Some("log"), head, None);
    html("'>");
    html_txt(name);
    html("</a>");
}

/// Emit a commit link with subject truncation.
pub fn commit_link(ctx: &CgitContext, name: &str, title: Option<&str>, class: Option<&str>, head: Option<&str>, rev: Option<&str>, path: Option<&str>) {
    let delim = repolink(ctx, title, class, Some("commit"), head, path);
    if let Some(r) = rev {
        if let Some(h) = &ctx.qry.head {
            if r != h {
                html(delim);
                html("id=");
                html_url_arg(r);
            }
        }
    }
    html("'>");
    if name.is_empty() {
        html_txt("(no commit message)");
    } else if name.len() > ctx.cfg.max_msg_len as usize && ctx.cfg.max_msg_len >= 15 {
        html_ntxt(name, (ctx.cfg.max_msg_len - 3) as usize);
        html("...");
    } else {
        html_txt(name);
    }
    html("</a>");
}

/// Emit a tag link.
pub fn tag_link(ctx: &CgitContext, name: &str, title: Option<&str>, class: Option<&str>, tag: &str) {
    reporevlink(ctx, "tag", name, title, class, Some(tag), None, None);
}

/// Emit a refs link.
pub fn refs_link(ctx: &CgitContext, name: &str, title: Option<&str>, class: Option<&str>, head: Option<&str>, rev: Option<&str>, path: Option<&str>) {
    reporevlink(ctx, "refs", name, title, class, head, rev, path);
}

/// Print age in human-readable format.
pub fn print_age(t: i64, _tz: i32, max_relative: i64) {
    if t == 0 {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let mut secs = now - t;
    if secs < 0 {
        secs = 0;
    }

    const TM_MIN: i64 = 60;
    const TM_HOUR: i64 = 60 * TM_MIN;
    const TM_DAY: i64 = 24 * TM_HOUR;
    const TM_WEEK: i64 = 7 * TM_DAY;
    const TM_MONTH: i64 = 30 * TM_DAY;
    const TM_YEAR: i64 = 365 * TM_DAY;

    if secs > max_relative && max_relative >= 0 {
        html("<span title='");
        html_attr(&format_iso8601(t, _tz));
        html("'>");
        html_txt(&format_short_date(t, _tz));
        html("</span>");
        return;
    }

    if secs < TM_HOUR * 2 {
        print_rel_date(t, _tz, secs as f64 / TM_MIN as f64, "age-mins", "min.");
    } else if secs < TM_DAY * 2 {
        print_rel_date(t, _tz, secs as f64 / TM_HOUR as f64, "age-hours", "hours");
    } else if secs < TM_WEEK * 2 {
        print_rel_date(t, _tz, secs as f64 / TM_DAY as f64, "age-days", "days");
    } else if secs < TM_MONTH * 2 {
        print_rel_date(t, _tz, secs as f64 / TM_WEEK as f64, "age-weeks", "weeks");
    } else if secs < TM_YEAR * 2 {
        print_rel_date(t, _tz, secs as f64 / TM_MONTH as f64, "age-months", "months");
    } else {
        print_rel_date(t, _tz, secs as f64 / TM_YEAR as f64, "age-years", "years");
    }
}

fn print_rel_date(t: i64, tz: i32, value: f64, class: &str, unit: &str) {
    html(&format!("<span class='{}' title='", class));
    html_attr(&format_iso8601(t, tz));
    html(&format!("'>{:.0} {}</span>", value, unit));
}

fn format_short_date(timestamp: i64, _tz: i32) -> String {
    let (year, month, day, _hour, _min, _sec, _wday) = unix_to_gmt(timestamp);
    format!("{:04}-{:02}-{:02}", year, month + 1, day)
}

fn print_header_table(ctx: &CgitContext) {
    html("<table id='header'>\n<tr>\n");

    let logo = if let Some(repo_idx) = ctx.repo {
        let repo = &ctx.repolist.repos[repo_idx];
        if let Some(ref l) = repo.logo {
            if !l.is_empty() { Some(l.as_str()) } else { None }
        } else {
            None
        }
    } else {
        None
    }
    .unwrap_or_else(|| if ctx.cfg.logo.is_empty() { "" } else { &ctx.cfg.logo });

    let logo_link = if let Some(repo_idx) = ctx.repo {
        let repo = &ctx.repolist.repos[repo_idx];
        if let Some(ref l) = repo.logo_link {
            if !l.is_empty() { Some(l.as_str()) } else { None }
        } else {
            None
        }
    } else {
        None
    }
    .or(ctx.cfg.logo_link.as_deref());

    if !logo.is_empty() {
        html("<td class='logo' rowspan='2'><a href='");
        if let Some(ll) = logo_link {
            html_attr(ll);
        } else {
            html_attr(ctx.rooturl());
        }
        html("'><img src='");
        html_attr(logo);
        html("' alt='cgit logo'/></a></td>\n");
    }

    html("<td class='main'>");
    if ctx.repo.is_some() {
        // Repo page: show "index : reponame"
        let rooturl = ctx.rooturl();
        html("<a href='");
        html_attr(rooturl);
        html("'>index</a> : ");
        if let Some(repo_idx) = ctx.repo {
            let repo = &ctx.repolist.repos[repo_idx];
            html("<a href='");
            if let Some(vr) = &ctx.cfg.virtual_root {
                html_url_path(vr);
                html_url_path(&repo.url);
                if !repo.url.ends_with('/') {
                    html("/");
                }
            } else {
                html("?r=");
                html_url_arg(&repo.url);
            }
            html("'>");
            html_txt(&repo.name);
            html("</a>");
        }
    } else {
        html_txt(&ctx.cfg.root_title);
    }
    html("</td></tr>\n");

    html("<tr><td class='sub'>");
    if let Some(repo_idx) = ctx.repo {
        let repo = &ctx.repolist.repos[repo_idx];
        html_txt(&repo.desc);
        html("</td><td class='sub right'>");
        if let Some(ref owner) = repo.owner {
            html_txt(owner);
        }
    } else {
        html_txt(&ctx.cfg.root_desc);
    }
    html("</td></tr></table>\n");
}

pub fn current_url(ctx: &CgitContext) -> String {
    let root = ctx.rooturl();
    match &ctx.qry.url {
        None => root.to_string(),
        Some(url) => {
            if root.ends_with('/') {
                format!("{}{}", root, url)
            } else {
                format!("{}/{}", root, url)
            }
        }
    }
}

pub fn print_error(msg: &str) {
    html("<div class='error'>");
    html_txt(msg);
    html("</div>\n");
}

pub fn print_error_page(ctx: &mut CgitContext, code: u16, status_msg: &str, error_msg: &str) {
    ctx.page.status = Some(code);
    ctx.page.statusmsg = Some(status_msg.to_string());
    print_http_headers(ctx);
    print_docstart(ctx);
    print_pageheader(ctx);
    print_error(error_msg);
    print_docend(ctx);
}

/// Format a timestamp as ISO 8601.
fn format_iso8601(timestamp: i64, _tz: i32) -> String {
    let (year, month, day, hour, min, sec, _wday) = unix_to_gmt(timestamp);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} +0000",
        year,
        month + 1,
        day,
        hour,
        min,
        sec
    )
}

/// Format a timestamp as ISO 8601 with the original timezone offset.
/// The tz parameter is in git's HHMM format (e.g. 100 = +0100, -500 = -0500).
pub fn format_iso8601_full(timestamp: i64, tz: i32) -> String {
    // Apply timezone offset to get local time
    let tz_hours = tz / 100;
    let tz_mins = tz % 100;
    let offset_secs = (tz_hours as i64) * 3600 + (tz_mins as i64) * 60;
    let local_ts = timestamp + offset_secs;
    let (year, month, day, hour, min, sec, _wday) = unix_to_gmt(local_ts);
    let sign = if tz >= 0 { '+' } else { '-' };
    let tz_abs = tz.unsigned_abs();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}{:02}{:02}",
        year,
        month + 1,
        day,
        hour,
        min,
        sec,
        sign,
        tz_abs / 100,
        tz_abs % 100
    )
}

/// Print the version string matching C cgit's --version output.
pub fn print_version() {
    println!(
        "CGit {} | https://git.zx2c4.com/cgit/\n\nCompiled in features:\n[+] Lua scripting\n[+] Linux sendfile() usage",
        CGIT_VERSION
    );
}
