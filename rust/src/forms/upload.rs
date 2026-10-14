//! A form with a file (multipart). Like [`FormData`](super::FormData): the
//! token is checked before the handler sees anything. The whole request
//! may be at most [`MAX_UPLOAD_BYTES`]; the route sets the same limit.

use axum::extract::{FromRequest, Multipart, Request};
use axum::http::StatusCode;

use super::{Refused, verify};
use crate::AppState;
use crate::request::Req;

/// A little over the 2 MB a picture may have, so a too big picture still
/// gets a friendly message instead of a bare error.
pub const MAX_UPLOAD_BYTES: usize = 3 * 1024 * 1024;

#[derive(Debug)]
pub struct UploadedFile {
    pub field: String,
    pub file_name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct UploadForm {
    fields: Vec<(String, String)>,
    pub files: Vec<UploadedFile>,
}

impl UploadForm {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .rev()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The chosen file of a field; None when no file was chosen.
    pub fn file(&self, name: &str) -> Option<&UploadedFile> {
        self.files
            .iter()
            .find(|file| file.field == name && !file.file_name.is_empty())
    }
}

impl FromRequest<AppState> for UploadForm {
    type Rejection = Refused;

    async fn from_request(request: Request, state: &AppState) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = request.into_parts();
        let req = <Req as axum::extract::FromRequestParts<AppState>>::from_request_parts(&mut parts, state)
            .await
            .map_err(|_| Refused(StatusCode::BAD_REQUEST))?;
        let headers = parts.headers.clone();
        let mut multipart = Multipart::from_request(Request::from_parts(parts, body), state)
            .await
            .map_err(|_| Refused(StatusCode::BAD_REQUEST))?;

        let mut form = UploadForm::default();
        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| Refused(StatusCode::PAYLOAD_TOO_LARGE))?
        {
            let name = field.name().unwrap_or_default().to_string();
            match field.file_name().map(str::to_string) {
                Some(file_name) => {
                    let bytes = field
                        .bytes()
                        .await
                        .map_err(|_| Refused(StatusCode::PAYLOAD_TOO_LARGE))?;
                    form.files.push(UploadedFile {
                        field: name,
                        file_name,
                        bytes: bytes.to_vec(),
                    });
                }
                None => {
                    let value = field.text().await.map_err(|_| Refused(StatusCode::BAD_REQUEST))?;
                    form.fields.push((name, value));
                }
            }
        }
        verify(&req, &headers, form.get("authenticity_token"))?;
        Ok(form)
    }
}
