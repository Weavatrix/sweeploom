//! Minimal property-list reads: top-level scalar values of XML or binary plists.
use std::{fs, path::Path};

/// Top-level value from a small plist file; never follows symlinks.
pub(crate) fn read(path: &Path, key: &str) -> Option<String> {
    let meta = fs::symlink_metadata(path).ok()?;
    if !meta.is_file() || meta.len() > 4 << 20 {
        return None;
    }
    value(&fs::read(path).ok()?, key)
}

pub(crate) fn value(bytes: &[u8], key: &str) -> Option<String> {
    if bytes.starts_with(b"bplist00") {
        binary(bytes, key)
    } else {
        xml(std::str::from_utf8(bytes).ok()?, key)
    }
}

fn xml(text: &str, key: &str) -> Option<String> {
    let mut depth = 0usize;
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];
        let end = rest.find('>')?;
        let tag = &rest[..end];
        rest = &rest[end + 1..];
        match tag {
            "dict" => depth += 1,
            "/dict" => depth = depth.saturating_sub(1),
            "key" => {
                let close = rest.find("</key>")?;
                let name = unescape(&rest[..close]);
                rest = &rest[close + 6..];
                if depth == 1 && name == key {
                    return element(rest);
                }
            }
            _ => {}
        }
    }
    None
}

fn element(text: &str) -> Option<String> {
    let body = text.trim_start().strip_prefix('<')?;
    let end = body.find('>')?;
    let tag = &body[..end];
    match tag {
        "true/" => return Some("true".into()),
        "false/" => return Some("false".into()),
        "string/" => return Some(String::new()),
        "string" | "date" | "integer" | "real" => {}
        _ => return None,
    }
    let body = &body[end + 1..];
    let close = body.find(&format!("</{tag}>"))?;
    Some(unescape(&body[..close]))
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn be(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(0, |acc, byte| (acc << 8) | u64::from(*byte))
}

fn binary(bytes: &[u8], key: &str) -> Option<String> {
    let trailer = bytes.get(bytes.len().checked_sub(32)?..)?;
    let offset_size = usize::from(trailer[6]);
    let ref_size = usize::from(trailer[7]);
    if !(1..=8).contains(&offset_size) || !(1..=8).contains(&ref_size) {
        return None;
    }
    let count = usize::try_from(be(&trailer[8..16])).ok()?;
    let top = usize::try_from(be(&trailer[16..24])).ok()?;
    let table = usize::try_from(be(&trailer[24..32])).ok()?;
    let offset = |index: usize| -> Option<usize> {
        if index >= count {
            return None;
        }
        let start = table.checked_add(index.checked_mul(offset_size)?)?;
        usize::try_from(be(bytes.get(start..start.checked_add(offset_size)?)?)).ok()
    };
    let (marker, len, body) = object(bytes, offset(top)?)?;
    if marker >> 4 != 0xD {
        return None;
    }
    let refs = bytes.get(body..body.checked_add(len.checked_mul(ref_size * 2)?)?)?;
    let reference = |slot: usize| usize::try_from(be(&refs[slot * ref_size..][..ref_size])).ok();
    (0..len).find_map(|index| {
        (string_at(bytes, offset(reference(index)?)?)? == key)
            .then(|| string_at(bytes, offset(reference(len + index)?)?))
            .flatten()
    })
}

/// Marker, element count and body offset of one object.
fn object(bytes: &[u8], at: usize) -> Option<(u8, usize, usize)> {
    let marker = *bytes.get(at)?;
    let low = usize::from(marker & 0x0F);
    if low != 0x0F {
        return Some((marker, low, at + 1));
    }
    let int = *bytes.get(at + 1)?;
    let width = 1usize << (int & 0x0F);
    if int >> 4 != 1 || width > 8 {
        return None;
    }
    let len = usize::try_from(be(bytes.get(at + 2..at + 2 + width)?)).ok()?;
    Some((marker, len, at + 2 + width))
}

fn string_at(bytes: &[u8], at: usize) -> Option<String> {
    let (marker, len, body) = object(bytes, at)?;
    match marker >> 4 {
        0x5 => Some(String::from_utf8_lossy(bytes.get(body..body.checked_add(len)?)?).into_owned()),
        0x6 => {
            let raw = bytes.get(body..body.checked_add(len.checked_mul(2)?)?)?;
            let units: Vec<u16> = raw
                .chunks_exact(2)
                .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
                .collect();
            Some(String::from_utf16_lossy(&units))
        }
        _ => None,
    }
}
