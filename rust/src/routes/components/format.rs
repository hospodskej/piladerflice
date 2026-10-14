//! Turning text typed in the admin into HTML, like Rails' text helpers.

/// Rails' `simple_format`: the text made safe, blank lines become
/// paragraphs and single line breaks become <br>.
pub fn simple_format(text: &str) -> String {
    let clean = ammonia::clean(text);
    let normalized = clean.replace("\r\n", "\n").replace('\r', "\n");
    let paragraphs: Vec<String> = normalized
        .split("\n\n")
        .map(|paragraph| paragraph.trim_matches('\n'))
        .filter(|paragraph| !paragraph.trim().is_empty())
        .map(|paragraph| format!("<p>{}</p>", paragraph.replace('\n', "\n<br />")))
        .collect();
    if paragraphs.is_empty() {
        "<p></p>".to_string()
    } else {
        paragraphs.join("\n\n")
    }
}

/// The text of an HTML fragment, without its tags (Rails' `strip_tags`).
pub fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_format_makes_paragraphs() {
        assert_eq!(simple_format("a\nb\n\nc"), "<p>a\n<br />b</p>\n\n<p>c</p>");
        assert_eq!(simple_format("x <br /> y"), "<p>x <br> y</p>");
        assert_eq!(simple_format("<script>alert(1)</script>ok"), "<p>ok</p>");
    }

    #[test]
    fn strips_tags() {
        assert_eq!(strip_tags("Suché <strong>dřevo</strong><br>"), "Suché dřevo");
    }
}
