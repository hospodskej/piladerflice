//! What the server functions start from (on the server only).

use leptos::prelude::*;

use crate::AppState;
use crate::locale::Locale;
use crate::request::Req;

/// The site, this visitor's request (session, signed-in user) and the
/// page's language from its code ("cs", "de"; anything else is Czech).
pub async fn request(locale: &str) -> Result<(AppState, Req, Locale), ServerFnError> {
    let state = expect_context::<AppState>();
    let req: Req = leptos_axum::extract_with_state(&state).await?;
    Ok((state, req, Locale::from_code(locale).unwrap_or(Locale::Cs)))
}

/// A failure on our side: logged; the island gets an error without details.
pub fn failed(error: anyhow::Error) -> ServerFnError {
    tracing::error!("server function failed: {error:#}");
    ServerFnError::new("failed")
}
