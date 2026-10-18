# Sapwood

A small web framework for server-rendered Rust sites on [Axum](https://docs.rs/axum): the part of a site that isn't about your business.

- **Sessions**: your own session type in an encrypted cookie, a CSRF token, flash messages, a long-lived login cookie. Nothing stored on the server.
- **Forms**: extractors that refuse forged forms before your handler runs; plain, typed (serde) and file uploads; validation errors to show next to fields.
- **Translations**: one YAML file per language, `{name}` placeholders, a fallback language; with the `minijinja` feature, `t()` for MiniJinja templates.
- **Assets**: images, CSS and JavaScript at URLs with a content hash, cached forever; an import map for JavaScript without a bundler.

```toml
[dependencies]
sapwood = "0.1"
```

Each part is also its own crate: [`sapwood-session`](https://docs.rs/sapwood-session), [`sapwood-forms`](https://docs.rs/sapwood-forms), [`sapwood-i18n`](https://docs.rs/sapwood-i18n), [`sapwood-assets`](https://docs.rs/sapwood-assets).

```sh
cargo run --example guestbook --features minijinja    # a small complete app: http://127.0.0.1:3000
```

Sapwood doesn't mind how you render HTML: templates, components or `format!`. It grew out of a real shop, [Pila Derflice](https://github.com/hospodskej/piladerflice) (a sawmill's site and e-shop), which uses its sessions, forms, translations and assets with Leptos components. Version 0.1: the API may still change.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise, any contribution you submit for inclusion in this crate shall be dual licensed as above, without any additional terms or conditions.
