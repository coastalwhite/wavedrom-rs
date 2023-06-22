use std::borrow::Cow;

pub fn escape_str(s: &str) -> Cow<str> {
    if !s.contains(['<', '>', '"', '&']) {
        return Cow::Borrowed(s);
    }

    let mut output = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '&' => output.push_str("&amp;"),
            _ => output.push(c),
        }
    }

    Cow::Owned(output)
}

