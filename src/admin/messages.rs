//! The admin's messages about a form that didn't save, e.g. "Klíč je
//! povinné pole" for the field labelled "Klíč".

pub fn blank(label: &str) -> String {
    format!("{label} je povinné pole")
}

pub fn taken(label: &str) -> String {
    format!("{label} už je použitý")
}

pub fn invalid(label: &str) -> String {
    format!("{label} má neplatnou hodnotu")
}

pub fn not_a_number(label: &str) -> String {
    format!("{label} musí být číslo")
}

pub fn not_above(label: &str, minimum: i64) -> String {
    format!("{label} musí být větší než {minimum}")
}

/// "A, B a C": messages as one sentence (for a flash message).
pub fn sentence(messages: &[&str]) -> String {
    match messages {
        [] => String::new(),
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} a {last}", rest.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_read_as_czech_sentences() {
        assert_eq!(blank("Klíč"), "Klíč je povinné pole");
        assert_eq!(not_above("Cena", 0), "Cena musí být větší než 0");
        assert_eq!(sentence(&["A"]), "A");
        assert_eq!(sentence(&["A", "B"]), "A a B");
        assert_eq!(sentence(&["A", "B", "C"]), "A, B a C");
    }
}
