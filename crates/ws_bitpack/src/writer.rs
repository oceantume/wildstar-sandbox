#[derive(Debug)]
pub enum BitPackWriterError {
    IoError(std::io::Error),
}

/// A BitPack writer that can be used to write game packets.
///
/// This implementation is very simple and could be optimized.
pub struct BitPackWriter<'a> {
    /// Represents the position of the writer in bits.
    pos: usize,
    /// Contains the byte currently being written to.
    byte: u8,
    /// The underlying writer.
    writer: &'a mut dyn std::io::Write,
}

impl<'a> BitPackWriter<'a> {
    /// Creates a [`BitPackWriter`] from an IO writer.
    pub fn new(writer: &'a mut dyn std::io::Write) -> Self {
        Self {
            pos: 0,
            byte: 0,
            writer,
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Flushes the current byte by adding 0's until aligned with the next byte.
    /// If the writer is already aligned, this does nothing.
    pub fn align(&mut self) -> Result<(), BitPackWriterError> {
        while self.pos % 8 != 0 {
            self.write_bit(false)?;
        }

        Ok(())
    }

    pub fn write_bit(&mut self, bit: bool) -> Result<(), BitPackWriterError> {
        let pos_in_byte = self.pos % 8;

        let mut byte = self.byte;
        if bit {
            byte |= 1 << pos_in_byte;
        }

        // if we're on the last bit in the byte, attempt to write it.
        if pos_in_byte == 7 {
            self.writer
                .write_all(&[byte])
                .map_err(|err| BitPackWriterError::IoError(err))?;
            byte = 0;
        }

        self.byte = byte;
        self.pos += 1;
        Ok(())
    }

    pub fn write_u64(&mut self, value: u64, bits: usize) -> Result<(), BitPackWriterError> {
        for i in 0..bits {
            self.write_bit(((value >> i) & 1) != 0)?;
        }

        Ok(())
    }

    pub fn write_f32(&mut self, value: f32) -> Result<(), BitPackWriterError> {
        self.write_u64(value.to_bits() as u64, 32)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), BitPackWriterError> {
        for byte in bytes {
            self.write_u64(*byte as u64, 8)?;
        }

        Ok(())
    }

    pub fn write_string(&mut self, _value: String, _wide: bool) -> Result<(), BitPackWriterError> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use super::*;

    // TODO: add tests for more complex packets

    #[test]
    fn test_write_position_and_alignment() {
        // aligning while at 0 stays at 0
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = BitPackWriter::new(&mut cursor);
        assert!(writer.align().is_ok());
        assert_eq!(writer.pos(), 0);
        assert_eq!(cursor.position(), 0);

        // writing 1 bit doesn't write a byte.
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = BitPackWriter::new(&mut cursor);
        assert!(writer.write_bit(true).is_ok());
        assert_eq!(cursor.position(), 0);

        // aligning writes the byte and advances to second byte position.
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = BitPackWriter::new(&mut cursor);
        assert!(writer.write_bit(true).is_ok());
        assert_eq!(writer.pos(), 1);
        assert!(writer.align().is_ok());
        assert_eq!(writer.pos(), 8);
        assert_eq!(cursor.position(), 1);

        // writing 9 bits advances cursor to second byte position.
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = BitPackWriter::new(&mut cursor);
        assert!(writer.write_u64(0, 9).is_ok());
        assert_eq!(writer.pos(), 9);
        assert_eq!(cursor.position(), 1);
    }

    #[test]
    fn test_simple_message() {
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = BitPackWriter::new(&mut cursor);

        // header
        assert!(writer.write_u64(47, 24).is_ok());
        assert!(writer.write_u64(2, 11).is_ok());

        // content
        assert!(writer.write_u64(6152, 32).is_ok());
        assert!(writer.write_u64(0, 32).is_ok());
        assert!(writer.write_u64(17, 32).is_ok());
        assert!(writer.write_u64(0, 32).is_ok());
        assert!(writer.write_u64(0, 64).is_ok());
        assert!(writer.write_u64(0, 16).is_ok());
        assert!(writer.write_u64(9, 5).is_ok());
        assert!(writer.write_u64(2629306514, 32).is_ok());
        assert!(writer.write_u64(0, 32).is_ok());
        assert!(writer.write_u64(0, 64).is_ok());

        // data is fully read
        assert!(writer.align().is_ok());
        assert_eq!(cursor.position(), 47);

        cursor.set_position(0);
        let mut vec = Vec::new();
        cursor.read_to_end(&mut vec).unwrap();
        assert_eq!(
            hex::encode(&vec),
            "2f00000240c00000000000008800000000000000000000\
            00000000000000489208b89c000000000000000000000000"
        );
    }
}
