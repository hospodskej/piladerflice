# sapwood-session

Encrypted cookie sessions for [Axum](https://docs.rs/axum). Part of [Sapwood](https://crates.io/crates/sapwood).

- Your own session type (any `#[derive(Clone, Default, Serialize, Deserialize)]` struct), in a cookie encrypted and authenticated with AES-256-GCM: visitors can't read or change it, and the server stores nothing.
- A **CSRF token** per session, a **flash message** shown once on the next page, and an optional **login cookie** that outlives the browser session.
- Requests that don't touch the session (images, CSS) set no cookie.

```rust
use axum::{Router, routing::get};
use sapwood_session::{Session, SessionLayer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
struct Visit {
    pages_seen: u32,
}

async fn page(session: Session<Visit>) -> String {
    let seen = session.update(|visit| {
        visit.pages_seen += 1;
        visit.pages_seen
    });
    format!("page {seen} of your visit")
}

let secret = std::env::var("SECRET_KEY_BASE").unwrap(); // at least 64 random bytes
let app: Router = Router::new()
    .route("/", get(page))
    .layer(SessionLayer::<Visit>::new(secret.as_bytes()).cookie_name("_shop_session"));
```

To check forms against the token, use [sapwood-forms](https://crates.io/crates/sapwood-forms).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise, any contribution you submit for inclusion in this crate shall be dual licensed as above, without any additional terms or conditions.
