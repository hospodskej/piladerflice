//! Calls from the islands back to the server.

use leptos::prelude::*;

/// Saves the signed-in user's dark/light choice.
#[server(endpoint = "theme")]
pub async fn save_theme(theme: String) -> Result<(), ServerFnError> {
    use crate::visitor::{Theme, visitor};
    let theme = Theme::parse(&theme).ok_or_else(|| ServerFnError::new("unknown theme"))?;
    let Some(user) = visitor().user else {
        return Ok(());
    };
    let state = expect_context::<crate::state::AppState>();
    let theme = theme.as_str();
    sqlx::query!("UPDATE users SET theme = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?", theme, user.id)
        .execute(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
