//! Native-messaging frames: 4-byte little-endian length + UTF-8 JSON.

use std::io::{self, Read, Write};

/// Chrome/Firefox native-messaging payload cap.
const MAX_FRAME: u32 = 1024 * 1024;

/// Read one frame. `Ok(None)` is a clean disconnect (EOF).
pub fn read_frame(reader: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut len_bytes = [0_u8; 4];
    match reader.read_exact(&mut len_bytes) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }
    let len = u32::from_le_bytes(len_bytes);
    if len == 0 || len > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "native-messaging frame size out of range",
        ));
    }
    let mut buf = vec![0_u8; len as usize];
    reader.read_exact(&mut buf)?;
    Ok(Some(buf))
}

/// Write one frame.
pub fn write_frame(writer: &mut impl Write, payload: &[u8]) -> io::Result<()> {
    let len = u32::try_from(payload.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "native-messaging payload too large",
        )
    })?;
    if len == 0 || len > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "native-messaging frame size out of range",
        ));
    }
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn roundtrip_frame() {
        let payload = br#"{"type":"hello"}"#;
        let mut buf = Vec::new();
        write_frame(&mut buf, payload).expect("write");
        let mut cursor = Cursor::new(buf);
        let got = read_frame(&mut cursor).expect("read").expect("eof");
        assert_eq!(got, payload);
    }

    #[test]
    fn eof_is_none() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        assert!(read_frame(&mut cursor).expect("read").is_none());
    }
}
