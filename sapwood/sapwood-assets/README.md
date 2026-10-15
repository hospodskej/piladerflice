# sapwood-assets

Fingerprinted static files for Rust web apps. Part of [Sapwood](https://crates.io/crates/sapwood).

Images, CSS and JavaScript get URLs with a hash of their content (`/assets/logo-1a2b3c4d.webp`), so browsers may cache them forever and a changed file simply gets a new URL. In development the index notices edited files. Also: a JavaScript import map (so `import "controllers/hello"` works without a bundler) and, with the `axum` feature (on by default), a router serving `/assets`.

```rust
use std::sync::Arc;
use sapwood_assets::{Assets, Importmap};

let assets = Arc::new(
    Assets::builder()
        .dir("assets")
        .dir("javascript")
        .watch(cfg!(debug_assertions))
        .build()?,
);
let logo = assets.url("logo.webp"); // "/assets/logo-1a2b3c4d.webp"
let importmap = Importmap::new().pin("application", "application.js").pin_all_from("controllers");
let app: axum::Router = axum::Router::new().merge(Assets::router(assets.clone()));
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise, any contribution you submit for inclusion in this crate shall be dual licensed as above, without any additional terms or conditions.
