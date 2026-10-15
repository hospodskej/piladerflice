//! Filling in `{name}` placeholders.

/// Replaces each `{name}` with its argument. Unknown names, and braces that
/// aren't a placeholder, stay as they are.
///
/// ```
/// assert_eq!(sapwood_i18n::interpolate("{km} km at {rate}", &[("km", "50"), ("rate", "40 Kč")]), "50 km at 40 Kč");
/// assert_eq!(sapwood_i18n::interpolate("100 {unknown}", &[]), "100 {unknown}");
/// ```
pub fn interpolate(text: &str, args: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let value = after.find('}').and_then(|end| {
            args.iter()
                .find(|(name, _)| *name == &after[..end])
                .map(|(_, value)| (end, value))
        });
        match value {
            Some((end, value)) => {
                out.push_str(value);
                rest = &after[end + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Escapes text for HTML: `& < > " '`.
pub fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_named_placeholders() {
        assert_eq!(interpolate("{a} and {b}", &[("a", "1"), ("b", "2")]), "1 and 2");
        assert_eq!(interpolate("{a}{a}", &[("a", "x")]), "xx");
    }

    #[test]
    fn leaves_other_braces_alone() {
        assert_eq!(interpolate("open {", &[]), "open {");
        assert_eq!(interpolate("{not closed", &[("not closed", "x")]), "{not closed");
        assert_eq!(interpolate("css { color: red }", &[]), "css { color: red }");
        assert_eq!(interpolate("{{a}}", &[("a", "x")]), "{x}");
    }

    #[test]
    fn escapes_html() {
        assert_eq!(
            escape_html(r#"<a href="x">Tom & 'Jerry'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;Tom &amp; &#39;Jerry&#39;&lt;/a&gt;"
        );
    }
}
