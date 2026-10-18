//! The site's static files, served by sapwood-assets with fingerprinted URLs:
//! the images and CSS from `assets/`. In development the index notices
//! edited files on the next page view. (The islands' WebAssembly has its
//! own versioned URLs, see `wasm.rs`.)

use std::path::Path;

pub use sapwood_assets::Assets;

pub fn load(root: &Path, watch: bool) -> std::io::Result<Assets> {
    Assets::builder()
        .dir(root.join("assets/images"))
        .dir(root.join("assets/stylesheets"))
        .watch(watch)
        .build()
}
