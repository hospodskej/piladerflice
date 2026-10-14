//! Putting the EXIF block into a WebP file. The encoder writes a plain WebP
//! (one image chunk); metadata needs the "extended" layout: a VP8X header
//! chunk that announces it, then the image, then an EXIF chunk.
//! See https://developers.google.com/speed/webp/docs/riff_container

const EXIF_FLAG: u8 = 0x08;

/// `webp` with `exif` attached. The picture was already turned upright, so
/// the EXIF orientation is reset to "normal" (or viewers would turn it again).
pub fn with_exif(webp: &[u8], exif: &[u8], width: u32, height: u32) -> Vec<u8> {
    let Some(chunks) = chunks(webp) else {
        return webp.to_vec();
    };
    let mut exif = exif.strip_prefix(b"Exif\0\0").unwrap_or(exif).to_vec();
    reset_orientation(&mut exif);

    let mut body: Vec<u8> = b"WEBP".to_vec();
    match chunks.iter().find(|(id, _)| id == b"VP8X") {
        Some((_, data)) => {
            let mut header = data.to_vec();
            header[0] |= EXIF_FLAG;
            push_chunk(&mut body, b"VP8X", &header);
        }
        None => {
            let mut header = vec![EXIF_FLAG, 0, 0, 0];
            header.extend_from_slice(&(width - 1).to_le_bytes()[..3]);
            header.extend_from_slice(&(height - 1).to_le_bytes()[..3]);
            push_chunk(&mut body, b"VP8X", &header);
        }
    }
    for (id, data) in chunks.iter().filter(|(id, _)| id != b"VP8X" && id != b"EXIF") {
        push_chunk(&mut body, id, data);
    }
    push_chunk(&mut body, b"EXIF", &exif);

    let mut file = b"RIFF".to_vec();
    file.extend_from_slice(&(body.len() as u32).to_le_bytes());
    file.extend_from_slice(&body);
    file
}

/// The chunks of a RIFF/WEBP file: (id, data).
fn chunks(webp: &[u8]) -> Option<Vec<([u8; 4], &[u8])>> {
    if webp.len() < 12 || &webp[0..4] != b"RIFF" || &webp[8..12] != b"WEBP" {
        return None;
    }
    let mut chunks = Vec::new();
    let mut at = 12;
    while at + 8 <= webp.len() {
        let id: [u8; 4] = webp[at..at + 4].try_into().ok()?;
        let size = u32::from_le_bytes(webp[at + 4..at + 8].try_into().ok()?) as usize;
        let data = webp.get(at + 8..at + 8 + size)?;
        chunks.push((id, data));
        at += 8 + size + (size % 2);
    }
    Some(chunks)
}

fn push_chunk(out: &mut Vec<u8>, id: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(id);
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
    if data.len() % 2 == 1 {
        out.push(0);
    }
}

/// Sets the Orientation tag (0x0112) of the first IFD of a TIFF/EXIF block
/// to 1 ("normal"). Anything unexpected is left alone.
pub fn reset_orientation(tiff: &mut [u8]) {
    let little = match tiff.get(0..2) {
        Some(b"II") => true,
        Some(b"MM") => false,
        _ => return,
    };
    let read16 = |b: &[u8], at: usize| -> Option<u16> {
        let bytes: [u8; 2] = b.get(at..at + 2)?.try_into().ok()?;
        Some(if little {
            u16::from_le_bytes(bytes)
        } else {
            u16::from_be_bytes(bytes)
        })
    };
    let read32 = |b: &[u8], at: usize| -> Option<u32> {
        let bytes: [u8; 4] = b.get(at..at + 4)?.try_into().ok()?;
        Some(if little {
            u32::from_le_bytes(bytes)
        } else {
            u32::from_be_bytes(bytes)
        })
    };
    let Some(ifd) = read32(tiff, 4).map(|offset| offset as usize) else {
        return;
    };
    let Some(count) = read16(tiff, ifd) else { return };
    for entry in 0..count as usize {
        let at = ifd + 2 + entry * 12;
        if read16(tiff, at) == Some(0x0112) && read16(tiff, at + 2) == Some(3) {
            let one = if little { 1u16.to_le_bytes() } else { 1u16.to_be_bytes() };
            if let Some(value) = tiff.get_mut(at + 8..at + 10) {
                value.copy_from_slice(&one);
            }
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resets_orientation_in_both_byte_orders() {
        let mut little = b"II*\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0".to_vec();
        reset_orientation(&mut little);
        assert_eq!(&little[18..20], &[1, 0]);
        let mut big = b"MM\0*\0\0\0\x08\0\x01\x01\x12\0\x03\0\0\0\x01\0\x08\0\0".to_vec();
        reset_orientation(&mut big);
        assert_eq!(&big[18..20], &[0, 1]);
        let mut garbage = b"nonsense".to_vec();
        reset_orientation(&mut garbage);
        assert_eq!(garbage, b"nonsense");
    }
}
