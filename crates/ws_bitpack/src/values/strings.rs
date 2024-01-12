use crate::*;

impl ReadValue for String {
    fn read(reader: &mut BitPackReader) -> BitPackResult<Self> {
        let extended: bool = reader.read()?;
        let length_bits = if extended { 15 } else { 7 };
        let length: usize = reader.read_packed(length_bits)?;
        let vec: Vec<u16> = reader.read_array(length)?;
        String::from_utf16(&vec).map_err(BitPackError::FromUtf16)
    }
}

impl WriteValue for String {
    fn write(&self, writer: &mut BitPackWriter) -> BitPackResult {
        WriteValue::write(self.as_str(), writer)
    }

    fn bits(&self) -> usize {
        WriteValue::bits(self.as_str())
    }
}

impl WriteValue for str {
    fn write(&self, writer: &mut BitPackWriter) -> BitPackResult {
        debug_assert!(self.len() < 32768);
        let extended = self.len() > 127;
        let length_bits = if extended { 15 } else { 7 };
        extended.write(writer)?;
        self.len().write_packed(writer, length_bits)?;
        self.encode_utf16()
            .try_for_each(|part| part.write(writer))?;
        Ok(())
    }

    fn bits(&self) -> usize {
        debug_assert!(self.len() < 32768);
        let extended = self.len() > 127;
        let length_bits = if extended { 15 } else { 7 };
        let content_bits = 16 * self.encode_utf16().count();
        length_bits + content_bits
    }
}
