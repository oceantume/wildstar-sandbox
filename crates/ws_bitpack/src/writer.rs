use crate::{BitPackError, BitPackResult, WriteArrayValue, WritePackedValue, WriteValue, WritePackedArrayValue};

/// A BitPack writer that can be used to write game packets.
///
/// While this guarantees not to panic, it should not be used anymore after any of
/// its functions returns an error because the bit stream may be corrupted at that
/// point.
///
/// This implementation is very simple and could be optimized.
pub struct BitPackWriter<'a> {
    /// The buffer to which this is writing.
    buffer: &'a mut [u8],
    /// Represents the position of the writer in bits.
    position: usize,
}

impl<'a> BitPackWriter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub fn with_position(buffer: &'a mut [u8], position: usize) -> Self {
        Self { buffer, position }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    /// Aligns the writer's position to the next byte by finishing the current byte
    /// with 0's.
    ///
    /// If the writer is already aligned, this does nothing.
    pub fn align(&mut self) -> BitPackResult {
        while self.position % 8 != 0 {
            self.write_bit(false)?;
        }

        Ok(())
    }

    pub fn write_bit(&mut self, bit: bool) -> BitPackResult {
        let pos_in_buffer = self.position / 8;
        let pos_in_byte = self.position % 8;

        match self.buffer.get_mut(pos_in_buffer) {
            Some(byte) => {
                let rhs = 1 << pos_in_byte;
                if bit {
                    *byte |= rhs;
                } else {
                    *byte &= !rhs
                }
                self.position += 1;

                Ok(())
            }
            None => Err(BitPackError::OutOfBounds),
        }
    }

    pub fn write_u64(&mut self, value: u64, bits: usize) -> BitPackResult {
        for i in 0..bits {
            self.write_bit(((value >> i) & 1) != 0)?;
        }

        Ok(())
    }

    pub fn write_f32(&mut self, value: f32) -> BitPackResult {
        self.write_u64(value.to_bits() as u64, 32)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> BitPackResult {
        for byte in bytes {
            self.write_u64(*byte as u64, 8)?;
        }

        Ok(())
    }

    pub fn write<T>(&mut self, value: &T) -> BitPackResult
    where
        T: WriteValue,
    {
        WriteValue::write(value, self)
    }

    pub fn write_packed<T>(&mut self, value: &T, bits: usize) -> BitPackResult
    where
        T: WritePackedValue,
    {
        WritePackedValue::write_packed(value, self, bits)
    }

    pub fn write_array<T>(&mut self, value: &T) -> BitPackResult
    where
        T: WriteArrayValue,
    {
        WriteArrayValue::write_array(value, self)
    }

    pub fn write_packed_array<T>(&mut self, value: &T, bits: usize) -> BitPackResult
    where
        T: WritePackedArrayValue,
    {
        WritePackedArrayValue::write_packed_array(value, self, bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for more complex packets

    #[test]
    fn test_write_position_and_alignment() {
        // aligning while at 0 stays at 0
        let mut buffer = vec![0; 10];
        let mut writer = BitPackWriter::new(&mut buffer);
        assert!(writer.align().is_ok());
        assert_eq!(writer.position(), 0);

        // writing 1 bit doesn't write a byte.
        let mut buffer = vec![0; 10];
        let mut writer = BitPackWriter::new(&mut buffer);
        assert!(writer.write_bit(true).is_ok());

        // aligning writes the byte and advances to second byte position.
        let mut buffer = vec![0; 10];
        let mut writer = BitPackWriter::new(&mut buffer);
        assert!(writer.write_bit(true).is_ok());
        assert_eq!(writer.position(), 1);
        assert!(writer.align().is_ok());
        assert_eq!(writer.position(), 8);

        // writing 9 bits advances cursor to second byte position.
        let mut buffer = vec![0; 10];
        let mut writer = BitPackWriter::new(&mut buffer);
        assert!(writer.write_u64(0, 9).is_ok());
        assert_eq!(writer.position(), 9);
    }

    #[test]
    #[should_panic(expected = "OutOfBounds")]
    fn test_write_out_of_bounds() {
        let mut buffer = vec![0; 1];
        let mut writer = BitPackWriter::new(&mut buffer);
        assert!(writer.write_u64(0, 8).is_ok());
        writer.write_u64(0, 32).unwrap();
    }

    #[test]
    fn test_simple_message() {
        let mut buffer = vec![0; 47];
        let mut writer = BitPackWriter::new(&mut buffer);

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
        assert_eq!(
            hex::encode(&buffer),
            "2f00000240c00000000000008800000000000000000000\
            00000000000000489208b89c000000000000000000000000"
        );
    }
}
