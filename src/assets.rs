//! The site's static files, served by sapwood-assets with fingerprinted URLs:
//! images and CSS from `assets/`, the JavaScript from `javascript/` and, for
//! Turbo and Stimulus, `vendor/javascript/`. In development the index
//! notices edited files on the next page view.

use std::path::Path;

pub use sapwood_assets::{Assets, Importmap};

pub fn load(root: &Path, watch: bool) -> std::io::Result<Assets> {
    Assets::builder()
        .dir(root.join("assets/images"))
        .dir(root.join("assets/stylesheets"))
        .dir(root.join("javascript"))
        .dir(root.join("vendor/javascript"))
        .watch(watch)
        .build()
}

/// Which file each JavaScript module name loads.
pub fn importmap() -> Importmap {
    Importmap::new()
        .pin("application", "application.js")
        .pin("@hotwired/turbo", "turbo.min.js")
        .pin("@hotwired/stimulus", "stimulus.min.js")
        .pin("@hotwired/stimulus-loading", "stimulus-loading.js")
        .pin_all_from("controllers")
        .pin_all_from("custom")
}
