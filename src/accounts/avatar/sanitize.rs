//! Turns an untrusted uploaded picture into a safe profile picture.
//!
//! We never store or serve what the user sent. The upload is checked by its
//! actual bytes (not the file name or the type the browser claims), limited
//! in size and pixel count (decompression bombs), decoded, turned upright by
//! its EXIF orientation, cropped to a centred square and re-encoded as a
//! fresh 256×256 WebP. Re-encoding drops anything hidden in the file.
//! EXIF metadata (camera, date, GPS...) is deliberately kept; it is
//! attacker-controlled text, so it is never shown as HTML.

use std::io::Cursor;

use image::imageops::FilterType;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits};

use super::webp_exif;

pub const MAX_BYTES: usize = 2 * 1024 * 1024;
/// ~5000×5000: a tiny file may decode to hundreds of MB.
pub const MAX_PIXELS: u64 = 25_000_000;
pub const SIZE: u32 = 256;
const QUALITY: f32 = 85.0;

/// Why a picture was refused; each has a message (`auth.avatar_errors.*`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Invalid {
    TooLarge,
    UnsupportedType,
    TooManyPixels,
    Unreadable,
}

impl Invalid {
    pub fn reason(self) -> &'static str {
        match self {
            Invalid::TooLarge => "too_large",
            Invalid::UnsupportedType => "unsupported_type",
            Invalid::TooManyPixels => "too_many_pixels",
            Invalid::Unreadable => "unreadable",
        }
    }
}

/// What the first bytes say the file is: only JPEG, PNG and WebP pass.
pub fn sniff(bytes: &[u8]) -> Option<ImageFormat> {
    match bytes {
        [0xFF, 0xD8, 0xFF, ..] => Some(ImageFormat::Jpeg),
        [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some(ImageFormat::Png),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Some(ImageFormat::WebP),
        _ => None,
    }
}

/// The sanitized picture as WebP bytes.
pub fn sanitize(bytes: &[u8]) -> Result<Vec<u8>, Invalid> {
    if bytes.len() > MAX_BYTES {
        return Err(Invalid::TooLarge);
    }
    let format = sniff(bytes).ok_or(Invalid::UnsupportedType)?;

    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_alloc = Some(512 * 1024 * 1024);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|_| Invalid::Unreadable)?;

    // The header alone tells the size; refuse before decoding any pixels.
    let (width, height) = decoder.dimensions();
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(Invalid::TooManyPixels);
    }
    let exif = decoder.exif_metadata().ok().flatten();
    let orientation = decoder.orientation().map_err(|_| Invalid::Unreadable)?;
    let mut image = DynamicImage::from_decoder(decoder).map_err(|_| Invalid::Unreadable)?;
    image.apply_orientation(orientation);

    let square = image.resize_to_fill(SIZE, SIZE, FilterType::Lanczos3);
    let encoded = if square.color().has_alpha() {
        let rgba = square.to_rgba8();
        webp::Encoder::from_rgba(&rgba, SIZE, SIZE).encode(QUALITY).to_vec()
    } else {
        let rgb = square.to_rgb8();
        webp::Encoder::from_rgb(&rgb, SIZE, SIZE).encode(QUALITY).to_vec()
    };
    Ok(match exif {
        Some(exif) => webp_exif::with_exif(&encoded, &exif, SIZE, SIZE),
        None => encoded,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageEncoder, RgbImage, codecs::jpeg::JpegEncoder};

    /// A JPEG of `w`×`h`, with an EXIF block (orientation `orientation`) if given.
    pub fn jpeg(w: u32, h: u32, orientation: Option<u16>) -> Vec<u8> {
        let image = RgbImage::from_fn(w, h, |x, _| {
            if x < w / 2 {
                [200, 30, 30].into()
            } else {
                [30, 30, 200].into()
            }
        });
        let mut out = Vec::new();
        JpegEncoder::new(&mut out)
            .write_image(&image, w, h, image::ExtendedColorType::Rgb8)
            .unwrap();
        match orientation {
            None => out,
            Some(value) => {
                // APP1 "Exif\0\0" + little-endian TIFF with one IFD0 entry: Orientation.
                let mut tiff = b"II*\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0".to_vec();
                tiff.extend_from_slice(&value.to_le_bytes());
                tiff.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
                let mut app1 = b"Exif\0\0".to_vec();
                app1.extend_from_slice(&tiff);
                let mut segment = vec![0xFF, 0xE1];
                segment.extend_from_slice(&((app1.len() + 2) as u16).to_be_bytes());
                segment.extend_from_slice(&app1);
                let mut with_exif = out[..2].to_vec();
                with_exif.extend_from_slice(&segment);
                with_exif.extend_from_slice(&out[2..]);
                with_exif
            }
        }
    }

    #[test]
    fn makes_a_256_square_webp() {
        let webp = sanitize(&jpeg(600, 400, None)).unwrap();
        assert_eq!(sniff(&webp), Some(ImageFormat::WebP));
        let decoded = image::load_from_memory(&webp).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (256, 256));
    }

    #[test]
    fn keeps_exif_and_turns_the_picture_upright() {
        // Orientation 6: the camera was turned; the left half (red) ends up on top.
        let webp = sanitize(&jpeg(400, 200, Some(6))).unwrap();
        let mut decoder = ImageReader::new(Cursor::new(&webp))
            .with_guessed_format()
            .unwrap()
            .into_decoder()
            .unwrap();
        let exif = decoder.exif_metadata().unwrap().expect("EXIF kept");
        assert_eq!(
            image::metadata::Orientation::from_exif_chunk(&exif),
            Some(image::metadata::Orientation::NoTransforms)
        );
        let decoded = image::load_from_memory(&webp).unwrap().to_rgb8();
        let top = decoded.get_pixel(128, 10);
        let bottom = decoded.get_pixel(128, 245);
        assert!(
            top[0] > 150 && bottom[2] > 150,
            "rotated: top {top:?}, bottom {bottom:?}"
        );
    }

    #[test]
    fn refuses_what_is_not_a_picture() {
        assert_eq!(sanitize(b"<svg onload=alert(1)>"), Err(Invalid::UnsupportedType));
        assert_eq!(sanitize(b"GIF89a....."), Err(Invalid::UnsupportedType));
        assert_eq!(sanitize(&[0xFF, 0xD8, 0xFF, 0x00, 0x01]), Err(Invalid::Unreadable));
        assert_eq!(sanitize(&vec![0u8; MAX_BYTES + 1]), Err(Invalid::TooLarge));
    }

    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }

    #[test]
    fn refuses_decompression_bombs_before_decoding() {
        // A PNG header claiming 20000×20000 pixels.
        let mut png = Vec::new();
        let image = image::GrayImage::new(1, 1);
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(&image, 1, 1, image::ExtendedColorType::L8)
            .unwrap();
        png[16..20].copy_from_slice(&20000u32.to_be_bytes());
        png[20..24].copy_from_slice(&20000u32.to_be_bytes());
        // The IHDR chunk's CRC must match, or the header is just "unreadable".
        let crc = crc32(&png[12..29]);
        png[29..33].copy_from_slice(&crc.to_be_bytes());
        assert_eq!(sanitize(&png), Err(Invalid::TooManyPixels));
    }
}
