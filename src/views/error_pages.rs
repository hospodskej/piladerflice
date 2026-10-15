//! The static error pages of `public/` (shared with Rails): "422 The
//! change you wanted was rejected" and "500 Something went wrong". Read
//! once at startup, so they can be shown even when something else fails.

use std::path::Path;
use std::sync::OnceLock;

static PAGES: OnceLock<Pages> = OnceLock::new();

struct Pages {
    rejected: String,
    failed: String,
}

/// Reads the pages from the `public/` folder (the first call wins).
pub fn load(public_dir: &Path) {
    let read = |name: &str, fallback: &str| {
        std::fs::read_to_string(public_dir.join(name)).unwrap_or_else(|error| {
            tracing::warn!("{name} not found in {}: {error}", public_dir.display());
            fallback.to_string()
        })
    };
    let _ = PAGES.set(Pages {
        rejected: read("422.html", "The change you wanted was rejected."),
        failed: read("500.html", "Something went wrong."),
    });
}

/// 422: a form that was forged or came from another site.
pub fn rejected() -> &'static str {
    PAGES
        .get()
        .map_or("The change you wanted was rejected.", |pages| &pages.rejected)
}

/// 500: something failed on our side.
pub fn failed() -> &'static str {
    PAGES.get().map_or("Something went wrong.", |pages| &pages.failed)
}
