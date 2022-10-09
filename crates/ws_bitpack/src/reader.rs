use std::io::{Read, Result};

/// A BitPack reader that can be used to read game packets.
///
/// This implementation is very simple and could be optimized.
pub struct BitPackReader<'a> {
    /// Represents the position of the reader in bits.
    pos: usize,
    /// Contains the byte currently being read from.
    byte: u8,
    /// The underlying reader.
    reader: &'a mut dyn Read,
}

impl<'a> BitPackReader<'a> {
    /// Creates a [`BitPackReader`] from an IO reader.
    pub fn new(reader: &'a mut dyn Read) -> Self {
        Self {
            pos: 0,
            byte: 0,
            reader,
        }
    }

    /// Returns the current position of this reader, in bits.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Advances the reader to the next full byte (pos % 8).
    /// If the reader is already aligned, this does nothing.
    pub fn align(&mut self) -> Result<()> {
        while self.pos % 8 != 0 {
            self.read_bit()?;
        }

        Ok(())
    }

    pub fn read_bit(&mut self) -> Result<bool> {
        let pos_in_byte = self.pos % 8;

        // at the start of new byte, so we need to read it first.
        if pos_in_byte == 0 {
            let mut buf: [u8; 1] = [0];
            self.reader.read_exact(&mut buf)?;
            self.byte = buf[0];
        }

        let value = (self.byte >> pos_in_byte) & 1 != 0;
        self.pos += 1;

        Ok(value)
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        self.read_u64(32).map(|v| f32::from_bits(v as u32))
    }

    pub fn read_u64(&mut self, bits: usize) -> Result<u64> {
        let mut value = 0;

        for i in 0..bits {
            if self.read_bit()? {
                value |= 1 << i;
            }
        }

        Ok(value)
    }

    pub fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        for byte in buf.iter_mut() {
            *byte = self.read_u64(8)? as u8;
        }

        Ok(())
    }

    pub fn read_string(&mut self, _wide: bool) -> Result<String> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    // TODO: add tests for more complex packets

    #[test]
    fn test_read_position_and_alignment() {
        let data = hex::decode("ffffffff").unwrap();

        // aligning while at 0 stays at 0
        let mut cursor = Cursor::new(&data);
        let mut reader = BitPackReader::new(&mut cursor);
        assert!(reader.align().is_ok());
        assert_eq!(reader.pos(), 0);
        assert_eq!(cursor.position(), 0);

        // reading 1 bit advances cursor to second byte position.
        let mut cursor = Cursor::new(&data);
        let mut reader = BitPackReader::new(&mut cursor);
        assert!(reader.read_bit().is_ok());
        assert_eq!(cursor.position(), 1);

        // aligning doesn't consume a byte, but advances bit position.
        let mut cursor = Cursor::new(&data);
        let mut reader = BitPackReader::new(&mut cursor);
        assert!(reader.read_bit().is_ok());
        assert_eq!(reader.pos(), 1);
        assert!(reader.align().is_ok());
        assert_eq!(reader.pos(), 8);
        assert_eq!(cursor.position(), 1);

        // reading 9 bits advances cursor to third byte position.
        let mut cursor = Cursor::new(&data);
        let mut reader = BitPackReader::new(&mut cursor);
        assert!(reader.read_u64(9).is_ok());
        assert_eq!(reader.pos(), 9);
        assert_eq!(cursor.position(), 2);
    }

    #[test]
    fn test_simple_message() {
        let data = "2f00000240c00000000000008800000000000000000000\
            00000000000000489208b89c000000000000000000000000";
        let data = hex::decode(data).unwrap();
        let mut cursor = Cursor::new(data);
        let mut reader = BitPackReader::new(&mut cursor);

        // header
        assert_eq!(reader.read_u64(24).unwrap(), 47);
        assert_eq!(reader.read_u64(11).unwrap(), 2);

        // content
        assert_eq!(reader.read_u64(32).unwrap(), 6152);
        assert_eq!(reader.read_u64(32).unwrap(), 0);
        assert_eq!(reader.read_u64(32).unwrap(), 17);
        assert_eq!(reader.read_u64(32).unwrap(), 0);
        assert_eq!(reader.read_u64(64).unwrap(), 0);
        assert_eq!(reader.read_u64(16).unwrap(), 0);
        assert_eq!(reader.read_u64(5).unwrap(), 9);
        assert_eq!(reader.read_u64(32).unwrap(), 2629306514);
        assert_eq!(reader.read_u64(32).unwrap(), 0);
        assert_eq!(reader.read_u64(64).unwrap(), 0);

        // data is fully read
        assert!(reader.align().is_ok());
        assert_eq!(cursor.position(), 47);
    }
}
