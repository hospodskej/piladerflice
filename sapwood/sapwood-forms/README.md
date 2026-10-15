# sapwood-forms

HTML forms for [Axum](https://docs.rs/axum). Part of [Sapwood](https://crates.io/crates/sapwood).

Every extractor checks the request first: the `Origin` must be the site itself and the form must carry the session's CSRF token (from [sapwood-session](https://crates.io/crates/sapwood-session)). A forged form never reaches your handler; the request gets a 422.

- `FormData`: the fields as they came (repeated fields too);
- `Form<T>`: the fields as your own `#[derive(Deserialize)]` struct;
- `UploadForm`: a `multipart/form-data` form with files;
- `Errors`: messages by field, ready for a template (`errors.fields.email`, `errors.messages`).

```rust
use axum::response::Redirect;
use sapwood_forms::{Errors, Form};
use serde::Deserialize;

#[derive(Deserialize)]
struct Signup {
    email: String,
    #[serde(default, deserialize_with = "sapwood_forms::de::checkbox")]
    newsletter: bool,
}

async fn signup(Form(form): Form<Signup>) -> Result<Redirect, String> {
    let mut errors = Errors::new();
    if !form.email.contains('@') {
        errors.add("email", "Enter your e-mail address");
    }
    if !errors.is_empty() {
        return Err(errors.messages().join(", "));
    }
    Ok(Redirect::to("/thanks"))
}
```

In each form: `<input type="hidden" name="csrf_token" value="{{ csrf_token }}">`, with the token from `Session::csrf_token()`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise, any contribution you submit for inclusion in this crate shall be dual licensed as above, without any additional terms or conditions.
