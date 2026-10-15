//! The signed-in customer of a request (or none), with their picture.

use super::avatar::{self, Avatar};
use crate::db::Db;
use crate::models::login_session;
use crate::models::user::User;

#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub user: User,
    pub avatar: Option<Avatar>,
    /// The `sessions` row of this login.
    pub session_id: i64,
}

impl CurrentUser {
    pub async fn load(db: &Db, session_id: Option<i64>) -> sqlx::Result<Option<CurrentUser>> {
        let Some(session_id) = session_id else { return Ok(None) };
        let Some(user) = login_session::user_of(db, session_id).await? else {
            return Ok(None);
        };
        let avatar = avatar::find(db, user.id).await?;
        Ok(Some(CurrentUser {
            user,
            avatar,
            session_id,
        }))
    }

    /// The picture's URL ("/muj-ucet/profilovy-obrazek?v=…") in a language.
    pub fn avatar_path(&self, localize: impl Fn(&str) -> String) -> Option<String> {
        self.avatar
            .as_ref()
            .map(|avatar| format!("{}?v={}", localize("/muj-ucet/profilovy-obrazek"), avatar.version()))
    }
}
