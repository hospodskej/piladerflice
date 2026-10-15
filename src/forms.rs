//! Submitted forms, read with sapwood-forms: every extractor checks the
//! `Origin` and the form's CSRF token first, so a handler never sees a
//! forged request (it gets `public/422.html` instead, see
//! `routes/rejections.rs`).

pub use sapwood_forms::{Errors, Form, FormData, Rejected, UploadForm, UploadedFile};

/// Profile pictures: a little over the 2 MB a picture may have, so a too
/// big picture still gets a friendly message instead of a bare error.
pub const MAX_UPLOAD_BYTES: usize = 3 * 1024 * 1024;

/// Admin forms with a photo (straight from a phone camera).
pub const MAX_ADMIN_UPLOAD_BYTES: usize = 25 * 1024 * 1024;
