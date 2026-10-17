//! A Sortiment category tile (picture, name, text, link), on the home page
//! and the Sortiment overview.

use crate::AppState;
use crate::models::content;
use crate::request::Req;
use crate::ui::shared::CategoryTile;

pub async fn all(state: &AppState, req: &Req) -> sqlx::Result<Vec<CategoryTile>> {
    Ok(content::category_tiles(&state.db)
        .await?
        .iter()
        .map(|tile| CategoryTile {
            title: tile.title(req.locale),
            text: tile.text(req.locale),
            image: tile.image.clone(),
            link: req.path_for(tile.link.as_deref().unwrap_or("/")),
        })
        .collect())
}
