use crate::{
    BitPackError, BitPackResult, ReadArrayValue, ReadPackedArrayValue, ReadPackedValue, ReadValue,
};

/// A BitPack reader that can be used to read game packets.
///
/// While this guarantees not to panic, it should not be used anymore after any of
/// its functions returns an error because the bit stream may be corrupted at that
/// point.
///
/// This implementation is very inefficient and could be optimized.
pub struct BitPackReader<'a> {
    /// The buffer from which the reader is reading.
    buffer: &'a [u8],
    /// Represents the position of the reader in bits.
    position: usize,
}

impl<'a> BitPackReader<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub fn with_position(buffer: &'a [u8], position: usize) -> Self {
        Self { buffer, position }
    }

    /// Returns the current position of this reader, in bits.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Advances the reader to the next full byte ((pos % 8) == 0).
    /// If the reader is already aligned, this does nothing.
    pub fn align(&mut self) -> BitPackResult {
        while self.position % 8 != 0 {
            self.read_bit()?;
        }

        Ok(())
    }

    pub fn read_bit(&mut self) -> BitPackResult<bool> {
        let pos_in_buffer = self.position / 8;
        let pos_in_byte = self.position % 8;

        match self.buffer.get(pos_in_buffer) {
            Some(byte) => {
                let value = (byte >> pos_in_byte) & 1 != 0;
                self.position += 1;

                Ok(value)
            }
            None => Err(BitPackError::OutOfBounds),
        }
    }

    pub fn read_f32(&mut self) -> BitPackResult<f32> {
        self.read_u64(32).map(|v| f32::from_bits(v as u32))
    }

    pub fn read_u64(&mut self, bits: usize) -> BitPackResult<u64> {
        let mut value = 0;

        for i in 0..bits {
            if self.read_bit()? {
                value |= 1 << i;
            }
        }

        Ok(value)
    }

    // todo: move this to support read<&mut [u8]>
    pub fn read_bytes(&mut self, buf: &mut [u8]) -> BitPackResult {
        for byte in buf.iter_mut() {
            *byte = self.read_u64(8)? as u8;
        }

        Ok(())
    }

    pub fn read<T>(&mut self) -> BitPackResult<T>
    where
        T: ReadValue,
    {
        ReadValue::read(self)
    }

    pub fn read_packed<T>(&mut self, bits: usize) -> BitPackResult<T>
    where
        T: ReadPackedValue,
    {
        ReadPackedValue::read_packed(self, bits)
    }

    pub fn read_array<T>(&mut self, length: usize) -> BitPackResult<T>
    where
        T: ReadArrayValue,
    {
        ReadArrayValue::read_array(self, length)
    }

    pub fn read_packed_array<T>(&mut self, length: usize, bits: usize) -> BitPackResult<T>
    where
        T: ReadPackedArrayValue,
    {
        ReadPackedArrayValue::read_packed_array(self, length, bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_position_and_alignment() {
        let data = hex::decode("ffffffff").unwrap();

        // aligning while at 0 stays at 0
        let mut reader = BitPackReader::new(&data);
        assert!(reader.align().is_ok());
        assert_eq!(reader.position(), 0);

        // reading 1 bit advances cursor to second byte position.
        let mut reader = BitPackReader::new(&data);
        assert!(reader.read_bit().is_ok());

        // aligning doesn't consume a byte, but advances bit position.
        let mut reader = BitPackReader::new(&data);
        assert!(reader.read_bit().is_ok());
        assert_eq!(reader.position(), 1);
        assert!(reader.align().is_ok());
        assert_eq!(reader.position(), 8);

        // reading 9 bits advances cursor to third byte position.
        let mut reader = BitPackReader::new(&data);
        assert!(reader.read_u64(9).is_ok());
        assert_eq!(reader.position(), 9);
    }

    #[test]
    fn test_simple_message() {
        let data = "2f00000240c00000000000008800000000000000000000\
            00000000000000489208b89c000000000000000000000000";
        let data = hex::decode(data).unwrap();
        let mut reader = BitPackReader::new(&data);

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
        assert_eq!(reader.position(), 47 * 8);
    }
}
