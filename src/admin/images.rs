//! Photos uploaded in the admin (products, variants, promos): checked to be
//! a JPEG, PNG or WebP picture and cleaned of personal data before they are
//! stored and shown on the site.
//!
//! Phone photos carry EXIF data: where they were taken (GPS), when, with
//! which phone. Rails strips it with ImageOptim; here the metadata blocks
//! are dropped from the file without touching the picture itself (no
//! re-encoding, so no quality is lost). Unlike ImageOptim, a JPEG keeps its
//! orientation, so a photo taken with the phone upright isn't shown lying
//! on its side.

use std::io::Cursor;
use std::path::Path;

use image::{ImageDecoder, ImageFormat, ImageReader};

use crate::db::Db;
use crate::forms::upload::UploadForm;
use crate::models::upload::Owner;
use crate::storage::{self, NewFile};

/// A picture ready to store.
#[derive(Debug)]
pub struct Image {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NotAnImage;

/// Checks and cleans an uploaded file.
pub fn prepare(bytes: &[u8]) -> Result<Image, NotAnImage> {
    let format = image::guess_format(bytes).map_err(|_| NotAnImage)?;
    let content_type = match format {
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::Png => "image/png",
        ImageFormat::WebP => "image/webp",
        _ => return Err(NotAnImage),
    };
    let reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut decoder = reader.into_decoder().map_err(|_| NotAnImage)?;
    let (mut width, mut height) = decoder.dimensions();
    let orientation = decoder.orientation().map(|o| o.to_exif()).unwrap_or(1);
    // Turned a quarter: shown the other way round, as Rails records it.
    if (5..=8).contains(&orientation) {
        std::mem::swap(&mut width, &mut height);
    }
    let bytes = match format {
        ImageFormat::Jpeg => strip_jpeg(bytes, orientation.into()),
        ImageFormat::Png => strip_png(bytes),
        _ => strip_webp(bytes),
    }
    .ok_or(NotAnImage)?;
    Ok(Image {
        bytes,
        content_type,
        width,
        height,
    })
}

/// The message when the file isn't a picture we take.
pub const NOT_AN_IMAGE: &str = "Obrázek musí být fotka ve formátu JPG, PNG nebo WebP.";

/// Checks the photo chosen in a form's `field`; Ok(None) when none was
/// chosen (the current picture stays).
pub async fn from_form(form: &UploadForm, field: &str) -> Result<Option<(Image, String)>, NotAnImage> {
    let Some(file) = form.file(field) else {
        return Ok(None);
    };
    let bytes = file.bytes.clone();
    // Reading the picture is CPU work: off the async threads.
    let image = tokio::task::spawn_blocking(move || prepare(&bytes))
        .await
        .map_err(|_| NotAnImage)??;
    Ok(Some((image, sanitize_filename(&file.file_name))))
}

/// Stores a checked photo as the record's image, replacing the old one.
pub async fn attach(
    db: &Db,
    storage_root: &Path,
    (owner, record_id): (Owner, i64),
    (image, filename): &(Image, String),
) -> anyhow::Result<()> {
    let file = NewFile {
        bytes: &image.bytes,
        filename,
        content_type: image.content_type,
    };
    storage::attach(db, storage_root, owner, record_id, file).await?;
    Ok(())
}

/// The file name as stored, made safe like Active Storage does (no
/// folders, no characters that mean something in a path or URL).
pub fn sanitize_filename(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name).trim();
    let cleaned: String = base
        .chars()
        .map(|c| match c {
            '\u{202E}' | '%' | '$' | '|' | ':' | ';' | '<' | '>' | '?' | '*' | '"' | '\t' | '\r' | '\n' => '-',
            c if c.is_control() => '-',
            c => c,
        })
        .collect();
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        "obrazek".into()
    } else {
        cleaned
    }
}

/// JPEG: drops comments and the APP blocks that hold metadata (EXIF, XMP,
/// IPTC...). Kept: APP0 (JFIF), APP2 (colour profile), APP14 (Adobe colour
/// info). When the photo is rotated, a tiny EXIF block with only the
/// orientation is put back.
fn strip_jpeg(data: &[u8], orientation: u16) -> Option<Vec<u8>> {
    if !data.starts_with(&[0xFF, 0xD8]) {
        return None;
    }
    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(&[0xFF, 0xD8]);
    let mut orientation_written = orientation <= 1;
    let mut i = 2;
    while i + 1 < data.len() {
        if data[i] != 0xFF {
            return None;
        }
        let marker = data[i + 1];
        if marker == 0xFF {
            i += 1;
            continue;
        }
        if !orientation_written && marker != 0xE0 {
            out.extend_from_slice(&orientation_segment(orientation));
            orientation_written = true;
        }
        // Start of the picture data (or the end): the rest is copied as is.
        if marker == 0xDA || marker == 0xD9 {
            out.extend_from_slice(&data[i..]);
            return Some(out);
        }
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            out.extend_from_slice(&data[i..i + 2]);
            i += 2;
            continue;
        }
        let length = u16::from_be_bytes([*data.get(i + 2)?, *data.get(i + 3)?]) as usize;
        let end = i + 2 + length;
        let segment = data.get(i..end)?;
        let metadata = marker == 0xFE || ((0xE1..=0xEF).contains(&marker) && marker != 0xE2 && marker != 0xEE);
        if !metadata {
            out.extend_from_slice(segment);
        }
        i = end;
    }
    None
}

/// An APP1 EXIF block holding just the orientation (tag 0x0112).
fn orientation_segment(orientation: u16) -> Vec<u8> {
    let mut exif = b"Exif\0\0MM\0\x2A\0\0\0\x08".to_vec();
    exif.extend_from_slice(&1u16.to_be_bytes()); // one entry
    exif.extend_from_slice(&0x0112u16.to_be_bytes());
    exif.extend_from_slice(&3u16.to_be_bytes()); // SHORT
    exif.extend_from_slice(&1u32.to_be_bytes());
    exif.extend_from_slice(&orientation.to_be_bytes());
    exif.extend_from_slice(&[0, 0]);
    exif.extend_from_slice(&0u32.to_be_bytes()); // no next directory
    let mut segment = vec![0xFF, 0xE1];
    segment.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
    segment.extend_from_slice(&exif);
    segment
}

/// PNG: drops text, EXIF and timestamp chunks.
fn strip_png(data: &[u8]) -> Option<Vec<u8>> {
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
    if !data.starts_with(SIGNATURE) {
        return None;
    }
    let mut out = SIGNATURE.to_vec();
    let mut i = SIGNATURE.len();
    while i < data.len() {
        let length = u32::from_be_bytes(data.get(i..i + 4)?.try_into().ok()?) as usize;
        let end = i.checked_add(12)?.checked_add(length)?;
        let chunk = data.get(i..end)?;
        let kind = &chunk[4..8];
        if !matches!(kind, b"tEXt" | b"zTXt" | b"iTXt" | b"eXIf" | b"tIME") {
            out.extend_from_slice(chunk);
        }
        i = end;
    }
    Some(out)
}

/// WebP: drops the EXIF and XMP chunks and says so in the header.
fn strip_webp(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WEBP" {
        return None;
    }
    let mut body = b"WEBP".to_vec();
    let mut i = 12;
    while i + 8 <= data.len() {
        let size = u32::from_le_bytes(data[i + 4..i + 8].try_into().ok()?) as usize;
        let end = (i + 8).checked_add(size + size % 2)?.min(data.len());
        let chunk = &data[i..end];
        match &chunk[0..4] {
            b"EXIF" | b"XMP " => {}
            b"VP8X" if chunk.len() > 8 => {
                let mut chunk = chunk.to_vec();
                chunk[8] &= !(0x08 | 0x04); // the EXIF and XMP flags
                body.extend_from_slice(&chunk);
            }
            _ => body.extend_from_slice(chunk),
        }
        i = end;
    }
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&body);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpeg() -> Vec<u8> {
        let picture = image::RgbImage::from_fn(40, 20, |x, _| image::Rgb([x as u8 * 5, 90, 30]));
        let mut out = Cursor::new(Vec::new());
        picture.write_to(&mut out, ImageFormat::Jpeg).unwrap();
        out.into_inner()
    }

    /// A JPEG with an EXIF block (orientation 6 plus a fake GPS note) and a
    /// comment, as a phone would make it.
    fn phone_jpeg() -> Vec<u8> {
        let plain = jpeg();
        let mut exif = orientation_segment(6);
        exif.extend_from_slice(b"GPS 48.85N 16.05E"); // trailing data inside the block
        let length = (exif.len() - 2) as u16;
        exif[2..4].copy_from_slice(&length.to_be_bytes());
        let mut comment = vec![0xFF, 0xFE, 0, 9];
        comment.extend_from_slice(b"Pixel 9");
        let mut data = plain[..2].to_vec();
        data.extend_from_slice(&exif);
        data.extend_from_slice(&comment);
        data.extend_from_slice(&plain[2..]);
        data
    }

    #[test]
    fn jpeg_metadata_goes_but_the_orientation_stays() {
        let image = prepare(&phone_jpeg()).unwrap();
        assert_eq!(image.content_type, "image/jpeg");
        assert!(!image.bytes.windows(3).any(|w| w == b"GPS"));
        assert!(!image.bytes.windows(7).any(|w| w == b"Pixel 9"));
        // Still readable, still on its side, and recorded as upright (20×40).
        let mut decoder = ImageReader::new(Cursor::new(&image.bytes))
            .with_guessed_format()
            .unwrap()
            .into_decoder()
            .unwrap();
        assert_eq!(decoder.orientation().unwrap().to_exif(), 6);
        assert_eq!((image.width, image.height), (20, 40));
    }

    #[test]
    fn a_plain_jpeg_is_kept_as_it_is() {
        let plain = jpeg();
        assert_eq!(prepare(&plain).unwrap().bytes, plain);
    }

    #[test]
    fn png_text_chunks_are_dropped() {
        let picture = image::RgbImage::new(4, 4);
        let mut out = Cursor::new(Vec::new());
        picture.write_to(&mut out, ImageFormat::Png).unwrap();
        let plain = out.into_inner();
        // A tEXt chunk after the header (CRC isn't checked by the stripper).
        let mut text = 18u32.to_be_bytes().to_vec();
        text.extend_from_slice(b"tEXtComment\0secret GPS");
        text.extend_from_slice(&[0, 0, 0, 0]);
        let mut data = plain[..33].to_vec();
        data.extend_from_slice(&text);
        data.extend_from_slice(&plain[33..]);
        let image = prepare(&data).unwrap();
        assert_eq!(image.bytes, plain);
        assert_eq!(image.content_type, "image/png");
    }

    #[test]
    fn other_files_are_refused() {
        assert_eq!(prepare(b"<svg onload=alert(1)>").unwrap_err(), NotAnImage);
        assert_eq!(prepare(b"GIF89a....").unwrap_err(), NotAnImage);
        assert_eq!(prepare(&jpeg()[..50]).unwrap_err(), NotAnImage);
    }

    #[test]
    fn file_names_lose_folders_and_odd_characters() {
        assert_eq!(sanitize_filename("C:\\fotky\\dub štípaný.jpg"), "dub štípaný.jpg");
        assert_eq!(sanitize_filename("../a?b.png"), "a-b.png");
        assert_eq!(sanitize_filename(".."), "obrazek");
    }
}
