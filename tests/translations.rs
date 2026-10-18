//! Every translation key the components and the code use exists in Czech and
//! in German, so no page ever shows "[missing: ...]". Only keys written out
//! in full are checked; keys put together at run time are tested where
//! they are used.

use std::path::{Path, PathBuf};

use piladerflice::locale::Locale;
use regex::Regex;

fn files(dir: &Path, extension: &str, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(&path, extension, found);
        } else if path.extension().is_some_and(|ext| ext == extension) {
            found.push(path);
        }
    }
}

fn keys_used() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let code_call = Regex::new(r#"\.text\(\s*[a-z_.]+,\s*"([a-z0-9_]+(?:\.[a-z0-9_]+)+)""#).unwrap();
    // The components: t("key"), t_html("key"), t_with("key", ...), t_list("key").
    let component_call =
        Regex::new(r#"\bt(?:_html|_with|_html_with|_list)?\(\s*"([a-z0-9_]+(?:\.[a-z0-9_]+)+)"\s*[,)]"#).unwrap();
    let mut used = Vec::new();
    for (dir, extension, pattern) in [("src", "rs", &code_call), ("src", "rs", &component_call)] {
        let mut paths = Vec::new();
        files(&root.join(dir), extension, &mut paths);
        for path in paths {
            let text = std::fs::read_to_string(&path).unwrap();
            for capture in pattern.captures_iter(&text) {
                let place = path.strip_prefix(root).unwrap().display().to_string();
                used.push((capture[1].to_string(), place));
            }
        }
    }
    used
}

#[test]
fn every_key_used_exists_in_both_languages() {
    let translations = piladerflice::i18n::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("locales")).unwrap();
    let used = keys_used();
    assert!(
        used.len() > 200,
        "found only {} keys: is the pattern still right?",
        used.len()
    );
    let missing: Vec<String> = used
        .iter()
        .flat_map(|(key, place)| {
            [Locale::Cs, Locale::De]
                .into_iter()
                .filter(|locale| !translations.has(*locale, key))
                .map(move |locale| format!("{}.{key} (used in {place})", locale.code()))
        })
        .collect();
    assert!(missing.is_empty(), "missing translations:\n{}", missing.join("\n"));
}
