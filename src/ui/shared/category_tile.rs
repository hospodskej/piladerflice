//! A Sortiment category tile (picture, name, text, link), on the home page
//! and the Sortiment overview.

#[derive(Clone, Debug)]
pub struct CategoryTile {
    pub title: String,
    pub text: String,
    /// A logical asset name ("sortiment/palivove-drevo.webp").
    pub image: Option<String>,
    pub link: String,
}
