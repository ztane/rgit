/// Expand $TOKEN macros in a string, replacing them with environment variable values.
/// Matches C cgit's expand_macros behavior.
pub fn expand_macros(txt: &str) -> String {
    expand_macros_with(txt, &[])
}

/// Expand $TOKEN macros with additional variable overrides.
/// Overrides are checked first before falling back to env vars.
pub fn expand_macros_with(txt: &str, overrides: &[(&str, &str)]) -> String {
    let mut result = String::with_capacity(txt.len());
    let bytes = txt.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'$' {
            i += 1;
            let start = i;
            while i < bytes.len() && is_token_char(bytes[i]) {
                i += 1;
            }
            if i > start {
                let name = &txt[start..i];
                // Check overrides first
                let mut found = false;
                for &(k, v) in overrides {
                    if k == name {
                        result.push_str(v);
                        found = true;
                        break;
                    }
                }
                if !found {
                    if let Ok(val) = std::env::var(name) {
                        result.push_str(&val);
                    }
                }
                continue;
            }
            // Lone $ with no token chars following
            result.push('$');
            continue;
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn is_token_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}
