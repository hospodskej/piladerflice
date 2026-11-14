//! Build steps for the site:
//! - the translations in `locales/*.yml` become typed Rust functions (codon);
//! - the stylesheets listed in `style/main.css` (its `@import` lines, in
//!   order) are bundled into one file, named after a hash of its content so
//!   browsers can keep it for a year.

use std::fmt::Write as _;
use std::path::Path;
use std::{env, fs};

fn main() {
    translations();
    stylesheet();
}

/// Typed functions for every key in locales/*.yml (see the codon crate).
fn translations() {
    let locales = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../locales");
    if let Err(e) = codon::Build::new()
        .locale("cs", locales.join("cs.yml"))
        .locale("de", locales.join("de.yml"))
        .url_prefix("de", "at")
        .generate()
    {
        panic!("translations: {e}");
    }
}

fn stylesheet() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../style");
    let manifest = root.join("main.css");
    println!("cargo:rerun-if-changed={}", manifest.display());

    let mut bundle = String::new();
    for line in fs::read_to_string(&manifest)
        .expect("style/main.css")
        .lines()
    {
        let Some(path) = line
            .trim()
            .strip_prefix("@import \"")
            .and_then(|l| l.strip_suffix("\";"))
        else {
            continue;
        };
        let file = root.join(path);
        println!("cargo:rerun-if-changed={}", file.display());
        let css = fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        writeln!(bundle, "/* {path} */\n{css}").unwrap();
    }

    // FNV-1a: stable across builds and Rust versions, good enough for cache busting.
    let hash = bundle.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
    });
    let out = Path::new(&env::var("OUT_DIR").unwrap()).to_owned();
    fs::write(out.join("site.css"), &bundle).unwrap();
    fs::write(
        out.join("site_css_href.txt"),
        format!("/assets/site-{:012x}.css", hash & 0xffff_ffff_ffff),
    )
    .unwrap();
}
