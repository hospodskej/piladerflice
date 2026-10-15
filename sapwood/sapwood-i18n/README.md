# sapwood-i18n

Translations for server-rendered Rust sites. Part of [Sapwood](https://crates.io/crates/sapwood).

One YAML file per language (`locales/en.yml`, `locales/de.yml`...), looked up by dotted key (`"nav.home"`), with `{name}` placeholders and a fallback language for keys a translation doesn't have yet. With the `minijinja` feature (on by default), templates get a `t("key", name=value)` function; keys ending in `_html` hold trusted HTML.

```yaml
# locales/en.yml
nav:
  home: Home
cart:
  added: "{product} is in your cart"
```

```rust
use sapwood_i18n::Translations;

let translations = Translations::load_dir("locales")?.with_fallback("en");
assert_eq!(translations.text("de", "cart.added", &[("product", "Oak")]), "Oak is in your cart");
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise, any contribution you submit for inclusion in this crate shall be dual licensed as above, without any additional terms or conditions.
