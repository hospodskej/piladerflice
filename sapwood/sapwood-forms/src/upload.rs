//! Forms with files.

use crate::FormData;

/// A chosen file.
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// The form field it came in.
    pub field: String,
    /// The name it had on the visitor's computer (not checked; clean it
    /// before using it).
    pub file_name: String,
    pub bytes: Vec<u8>,
}

/// A `multipart/form-data` form: its fields and its files. A plain form is
/// read too (fields, no files), so one handler can take both a "Save" form
/// with a photo and a plain "Delete" button.
///
/// How big a request may be is the route's `DefaultBodyLimit`.
#[derive(Debug, Default, Clone)]
pub struct UploadForm {
    pub fields: FormData,
    pub files: Vec<UploadedFile>,
}

impl UploadForm {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.fields.get(name)
    }

    /// The file chosen in `field`; None when none was chosen.
    pub fn file(&self, field: &str) -> Option<&UploadedFile> {
        self.files
            .iter()
            .find(|file| file.field == field && !file.file_name.is_empty())
    }
}
