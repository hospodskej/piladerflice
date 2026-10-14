//! Small text helpers shared by the views.

pub fn escape(text: &str) -> String {
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

/// Collapses runs of whitespace, like Rails' `squish`.
pub fn squish(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Rails' `truncate(length, separator: " ")`: at most `length` characters,
/// cut at a space, ending in "...".
pub fn truncate(text: &str, length: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= length {
        return text.to_string();
    }
    let stop = length - 3;
    let cut = chars[..=stop.min(chars.len() - 1)]
        .iter()
        .rposition(|c| *c == ' ')
        .unwrap_or(stop);
    let mut out: String = chars[..cut].iter().collect();
    out.push_str("...");
    out
}

/// "neco jineho" -> "Neco jineho"
pub fn capitalize(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut chars = lower.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_like_rails() {
        let text = "a ".repeat(100);
        let cut = truncate(text.trim(), 160);
        assert!(cut.chars().count() <= 160);
        assert!(cut.ends_with("a..."));
        assert_eq!(truncate("short", 160), "short");
    }

    #[test]
    fn capitalizes_unknown_segments() {
        assert_eq!(capitalize("neco jineho"), "Neco jineho");
    }

    #[test]
    fn escapes_like_rails() {
        assert_eq!(
            escape(r#"<a href="x">'&'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
        );
        assert_eq!(escape("/cesta"), "/cesta");
    }
}
