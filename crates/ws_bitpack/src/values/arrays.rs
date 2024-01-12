use crate::*;

impl<Item> ReadArrayValue for Vec<Item>
where
    Item: ReadValue,
{
    fn read_array(reader: &mut BitPackReader, length: usize) -> BitPackResult<Self> {
        let mut vec = Vec::with_capacity(length);
        while vec.len() < length {
            vec.push(ReadValue::read(reader)?);
        }
        Ok(vec)
    }
}

impl<Item> WriteArrayValue for Vec<Item>
where
    Item: WriteValue,
{
    fn write_array(&self, writer: &mut BitPackWriter) -> BitPackResult {
        self.iter()
            .try_for_each(|item| WriteValue::write(item, writer))
    }

    fn bits_array(&self) -> usize {
        self.iter()
            .fold(0, |bits, item| bits + WriteValue::bits(item))
    }
}

impl<Item> ReadPackedArrayValue for Vec<Item>
where
    Item: ReadPackedValue,
{
    fn read_packed_array(
        reader: &mut BitPackReader,
        length: usize,
        bits: usize,
    ) -> BitPackResult<Self> {
        let mut vec = Vec::with_capacity(length);
        while vec.len() < length {
            vec.push(ReadPackedValue::read_packed(reader, bits)?);
        }
        Ok(vec)
    }
}

impl<Item> WritePackedArrayValue for Vec<Item>
where
    Item: WritePackedValue,
{
    fn write_packed_array(&self, writer: &mut BitPackWriter, bits: usize) -> BitPackResult {
        self.iter()
            .try_for_each(|item| WritePackedValue::write_packed(item, writer, bits))
    }

    fn bits_packed_array(&self, bits: usize) -> usize {
        self.len() * bits
    }
}
