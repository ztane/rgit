use std::io::{self, Write};

/// Percent-encoding table matching C cgit's url_escape_table exactly.
/// Characters NOT escaped: a-zA-Z0-9 ! $ ( ) * , . / : ; @ - ~
/// NULL entry means "pass through", Some means "replace with this string".
static URL_ESCAPE_TABLE: [Option<&str>; 256] = {
    let mut table: [Option<&str>; 256] = [None; 256];
    // 0x00..0x1f: all escaped
    table[0x00] = Some("%00"); table[0x01] = Some("%01"); table[0x02] = Some("%02"); table[0x03] = Some("%03");
    table[0x04] = Some("%04"); table[0x05] = Some("%05"); table[0x06] = Some("%06"); table[0x07] = Some("%07");
    table[0x08] = Some("%08"); table[0x09] = Some("%09"); table[0x0a] = Some("%0a"); table[0x0b] = Some("%0b");
    table[0x0c] = Some("%0c"); table[0x0d] = Some("%0d"); table[0x0e] = Some("%0e"); table[0x0f] = Some("%0f");
    table[0x10] = Some("%10"); table[0x11] = Some("%11"); table[0x12] = Some("%12"); table[0x13] = Some("%13");
    table[0x14] = Some("%14"); table[0x15] = Some("%15"); table[0x16] = Some("%16"); table[0x17] = Some("%17");
    table[0x18] = Some("%18"); table[0x19] = Some("%19"); table[0x1a] = Some("%1a"); table[0x1b] = Some("%1b");
    table[0x1c] = Some("%1c"); table[0x1d] = Some("%1d"); table[0x1e] = Some("%1e"); table[0x1f] = Some("%1f");
    // 0x20..0x2f
    table[0x20] = Some("%20"); // space
    // 0x21 '!' = None (pass through)
    table[0x22] = Some("%22"); // "
    table[0x23] = Some("%23"); // #
    // 0x24 '$' = None
    table[0x25] = Some("%25"); // %
    table[0x26] = Some("%26"); // &
    table[0x27] = Some("%27"); // '
    // 0x28 '(' = None
    // 0x29 ')' = None
    // 0x2a '*' = None
    table[0x2b] = Some("%2b"); // +
    // 0x2c ',' = None
    // 0x2d '-' = None
    // 0x2e '.' = None
    // 0x2f '/' = None
    // 0x30..0x39 '0'-'9' = None
    // 0x3a ':' = None
    // 0x3b ';' = None
    table[0x3c] = Some("%3c"); // <
    table[0x3d] = Some("%3d"); // =
    table[0x3e] = Some("%3e"); // >
    table[0x3f] = Some("%3f"); // ?
    // 0x40 '@' = None
    // 0x41..0x5a 'A'-'Z' = None
    // 0x5b '[' = None (not in table, passes through)
    table[0x5c] = Some("%5c"); // backslash
    // 0x5d ']' = None
    table[0x5e] = Some("%5e"); // ^
    // 0x5f '_' = None (not in C table either)
    table[0x60] = Some("%60"); // `
    // 0x61..0x7a 'a'-'z' = None
    table[0x7b] = Some("%7b"); // {
    table[0x7c] = Some("%7c"); // |
    table[0x7d] = Some("%7d"); // }
    // 0x7e '~' = None
    table[0x7f] = Some("%7f");
    // 0x80..0xff: all escaped
    table[0x80] = Some("%80"); table[0x81] = Some("%81"); table[0x82] = Some("%82"); table[0x83] = Some("%83");
    table[0x84] = Some("%84"); table[0x85] = Some("%85"); table[0x86] = Some("%86"); table[0x87] = Some("%87");
    table[0x88] = Some("%88"); table[0x89] = Some("%89"); table[0x8a] = Some("%8a"); table[0x8b] = Some("%8b");
    table[0x8c] = Some("%8c"); table[0x8d] = Some("%8d"); table[0x8e] = Some("%8e"); table[0x8f] = Some("%8f");
    table[0x90] = Some("%90"); table[0x91] = Some("%91"); table[0x92] = Some("%92"); table[0x93] = Some("%93");
    table[0x94] = Some("%94"); table[0x95] = Some("%95"); table[0x96] = Some("%96"); table[0x97] = Some("%97");
    table[0x98] = Some("%98"); table[0x99] = Some("%99"); table[0x9a] = Some("%9a"); table[0x9b] = Some("%9b");
    table[0x9c] = Some("%9c"); table[0x9d] = Some("%9d"); table[0x9e] = Some("%9e"); table[0x9f] = Some("%9f");
    table[0xa0] = Some("%a0"); table[0xa1] = Some("%a1"); table[0xa2] = Some("%a2"); table[0xa3] = Some("%a3");
    table[0xa4] = Some("%a4"); table[0xa5] = Some("%a5"); table[0xa6] = Some("%a6"); table[0xa7] = Some("%a7");
    table[0xa8] = Some("%a8"); table[0xa9] = Some("%a9"); table[0xaa] = Some("%aa"); table[0xab] = Some("%ab");
    table[0xac] = Some("%ac"); table[0xad] = Some("%ad"); table[0xae] = Some("%ae"); table[0xaf] = Some("%af");
    table[0xb0] = Some("%b0"); table[0xb1] = Some("%b1"); table[0xb2] = Some("%b2"); table[0xb3] = Some("%b3");
    table[0xb4] = Some("%b4"); table[0xb5] = Some("%b5"); table[0xb6] = Some("%b6"); table[0xb7] = Some("%b7");
    table[0xb8] = Some("%b8"); table[0xb9] = Some("%b9"); table[0xba] = Some("%ba"); table[0xbb] = Some("%bb");
    table[0xbc] = Some("%bc"); table[0xbd] = Some("%bd"); table[0xbe] = Some("%be"); table[0xbf] = Some("%bf");
    table[0xc0] = Some("%c0"); table[0xc1] = Some("%c1"); table[0xc2] = Some("%c2"); table[0xc3] = Some("%c3");
    table[0xc4] = Some("%c4"); table[0xc5] = Some("%c5"); table[0xc6] = Some("%c6"); table[0xc7] = Some("%c7");
    table[0xc8] = Some("%c8"); table[0xc9] = Some("%c9"); table[0xca] = Some("%ca"); table[0xcb] = Some("%cb");
    table[0xcc] = Some("%cc"); table[0xcd] = Some("%cd"); table[0xce] = Some("%ce"); table[0xcf] = Some("%cf");
    table[0xd0] = Some("%d0"); table[0xd1] = Some("%d1"); table[0xd2] = Some("%d2"); table[0xd3] = Some("%d3");
    table[0xd4] = Some("%d4"); table[0xd5] = Some("%d5"); table[0xd6] = Some("%d6"); table[0xd7] = Some("%d7");
    table[0xd8] = Some("%d8"); table[0xd9] = Some("%d9"); table[0xda] = Some("%da"); table[0xdb] = Some("%db");
    table[0xdc] = Some("%dc"); table[0xdd] = Some("%dd"); table[0xde] = Some("%de"); table[0xdf] = Some("%df");
    table[0xe0] = Some("%e0"); table[0xe1] = Some("%e1"); table[0xe2] = Some("%e2"); table[0xe3] = Some("%e3");
    table[0xe4] = Some("%e4"); table[0xe5] = Some("%e5"); table[0xe6] = Some("%e6"); table[0xe7] = Some("%e7");
    table[0xe8] = Some("%e8"); table[0xe9] = Some("%e9"); table[0xea] = Some("%ea"); table[0xeb] = Some("%eb");
    table[0xec] = Some("%ec"); table[0xed] = Some("%ed"); table[0xee] = Some("%ee"); table[0xef] = Some("%ef");
    table[0xf0] = Some("%f0"); table[0xf1] = Some("%f1"); table[0xf2] = Some("%f2"); table[0xf3] = Some("%f3");
    table[0xf4] = Some("%f4"); table[0xf5] = Some("%f5"); table[0xf6] = Some("%f6"); table[0xf7] = Some("%f7");
    table[0xf8] = Some("%f8"); table[0xf9] = Some("%f9"); table[0xfa] = Some("%fa"); table[0xfb] = Some("%fb");
    table[0xfc] = Some("%fc"); table[0xfd] = Some("%fd"); table[0xfe] = Some("%fe"); table[0xff] = Some("%ff");
    table
};

/// Global output writer. All html output goes through this.
/// We use a thread-local buffered writer wrapping stdout for performance.
pub struct HtmlOutput;

impl HtmlOutput {
    pub fn write_bytes(data: &[u8]) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(data).expect("write error on html output");
    }

    pub fn write_str(s: &str) {
        Self::write_bytes(s.as_bytes());
    }

    pub fn flush() {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.flush().expect("flush error on html output");
    }
}

/// Write raw HTML string (no escaping).
pub fn html(txt: &str) {
    HtmlOutput::write_str(txt);
}

/// Write raw bytes.
pub fn html_raw(data: &[u8]) {
    HtmlOutput::write_bytes(data);
}

/// Write formatted HTML (no escaping).
pub fn htmlf(args: std::fmt::Arguments<'_>) {
    let s = format!("{}", args);
    html(&s);
}

/// Write text with HTML entity escaping (< > &).
pub fn html_txt(txt: &str) {
    html_ntxt(txt, txt.len());
}

/// Write up to `maxlen` bytes of text with HTML entity escaping.
/// Returns remaining count (negative-ish if truncated).
pub fn html_ntxt(txt: &str, maxlen: usize) -> isize {
    let bytes = txt.as_bytes();
    let mut remaining = maxlen as isize;
    let mut last = 0;
    for (i, &c) in bytes.iter().enumerate() {
        if remaining <= 0 {
            break;
        }
        remaining -= 1;
        match c {
            b'<' => {
                HtmlOutput::write_bytes(&bytes[last..i]);
                html("&lt;");
                last = i + 1;
            }
            b'>' => {
                HtmlOutput::write_bytes(&bytes[last..i]);
                html("&gt;");
                last = i + 1;
            }
            b'&' => {
                HtmlOutput::write_bytes(&bytes[last..i]);
                html("&amp;");
                last = i + 1;
            }
            _ => {}
        }
    }
    let end = std::cmp::min(maxlen, bytes.len());
    if end > last {
        HtmlOutput::write_bytes(&bytes[last..end]);
    }
    remaining
}

/// Write text with HTML attribute escaping (< > & ' ").
pub fn html_attr(txt: &str) {
    let bytes = txt.as_bytes();
    let mut last = 0;
    for (i, &c) in bytes.iter().enumerate() {
        let replacement = match c {
            b'<' => Some("&lt;"),
            b'>' => Some("&gt;"),
            b'\'' => Some("&#x27;"),
            b'"' => Some("&quot;"),
            b'&' => Some("&amp;"),
            _ => None,
        };
        if let Some(r) = replacement {
            HtmlOutput::write_bytes(&bytes[last..i]);
            html(r);
            last = i + 1;
        }
    }
    if bytes.len() > last {
        HtmlOutput::write_bytes(&bytes[last..]);
    }
}

/// URL-encode for path segments: escapes everything in url_escape_table
/// EXCEPT '+' and '&' which are passed through in paths.
pub fn html_url_path(txt: &str) {
    let bytes = txt.as_bytes();
    let mut last = 0;
    for (i, &c) in bytes.iter().enumerate() {
        if c == b'+' || c == b'&' {
            continue;
        }
        if let Some(escaped) = URL_ESCAPE_TABLE[c as usize] {
            HtmlOutput::write_bytes(&bytes[last..i]);
            html(escaped);
            last = i + 1;
        }
    }
    if bytes.len() > last {
        HtmlOutput::write_bytes(&bytes[last..]);
    }
}

/// URL-encode for query parameters: escapes everything in url_escape_table,
/// and additionally replaces ' ' with '+'.
pub fn html_url_arg(txt: &str) {
    let bytes = txt.as_bytes();
    let mut last = 0;
    for (i, &c) in bytes.iter().enumerate() {
        let escaped = if c == b' ' {
            Some("+")
        } else {
            URL_ESCAPE_TABLE[c as usize]
        };
        if let Some(e) = escaped {
            HtmlOutput::write_bytes(&bytes[last..i]);
            html(e);
            last = i + 1;
        }
    }
    if bytes.len() > last {
        HtmlOutput::write_bytes(&bytes[last..]);
    }
}

/// Escape for use inside a quoted HTTP header value.
pub fn html_header_arg_in_quotes(txt: &str) {
    let bytes = txt.as_bytes();
    let mut last = 0;
    for (i, &c) in bytes.iter().enumerate() {
        let replacement = match c {
            b'\\' => Some("\\\\"),
            b'\r' => Some("\\r"),
            b'\n' => Some("\\n"),
            b'"' => Some("\\\""),
            _ => None,
        };
        if let Some(r) = replacement {
            HtmlOutput::write_bytes(&bytes[last..i]);
            html(r);
            last = i + 1;
        }
    }
    if bytes.len() > last {
        HtmlOutput::write_bytes(&bytes[last..]);
    }
}

/// Generate an HTML hidden input field.
pub fn html_hidden(name: &str, value: &str) {
    html("<input type='hidden' name='");
    html_attr(name);
    html("' value='");
    html_attr(value);
    html("'/>");
}

/// Generate an HTML select option.
pub fn html_option(value: &str, text: &str, selected_value: Option<&str>) {
    html("<option value='");
    html_attr(value);
    html("'");
    if let Some(sel) = selected_value {
        if sel == value {
            html(" selected='selected'");
        }
    }
    html(">");
    html_txt(text);
    html("</option>\n");
}

/// Generate an HTML link opening tag.
pub fn html_link_open(url: &str, title: Option<&str>, class: Option<&str>) {
    html("<a href='");
    html_attr(url);
    if let Some(t) = title {
        html("' title='");
        html_attr(t);
    }
    if let Some(c) = class {
        html("' class='");
        html_attr(c);
    }
    html("'>");
}

/// Generate a closing link tag.
pub fn html_link_close() {
    html("</a>");
}

/// Print file permission bits as rwx.
pub fn html_fileperm(mode: u16) {
    html(if mode & 4 != 0 { "r" } else { "-" });
    html(if mode & 2 != 0 { "w" } else { "-" });
    html(if mode & 1 != 0 { "x" } else { "-" });
}

/// Include a file's contents as raw HTML.
pub fn html_include(filename: &str) -> io::Result<()> {
    let content = std::fs::read(filename)?;
    html_raw(&content);
    Ok(())
}

/// Decode a hex digit.
fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// URL-decode a string, replacing '+' with ' ' and %XX with the byte.
pub fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                result.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                    result.push((hi << 4) | lo);
                    i += 3;
                } else {
                    result.push(b'%');
                    i += 1;
                }
            }
            c => {
                result.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&result).into_owned()
}

/// Parse a querystring and call the callback for each name=value pair.
/// Matches C cgit's http_parse_querystring behavior exactly.
pub fn http_parse_querystring(txt: &str, mut callback: impl FnMut(&str, &str)) {
    if txt.is_empty() {
        return;
    }
    for pair in txt.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (name, value) = if let Some(eq_pos) = pair.find('=') {
            (&pair[..eq_pos], &pair[eq_pos + 1..])
        } else {
            (pair, "")
        };
        let decoded_name = url_decode(name);
        let decoded_value = url_decode(value);
        if !decoded_name.is_empty() {
            callback(&decoded_name, &decoded_value);
        }
    }
}
