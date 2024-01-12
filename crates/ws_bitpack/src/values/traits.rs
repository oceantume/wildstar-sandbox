use crate::*;

pub trait ReadValue
where
    Self: Sized,
{
    fn read(reader: &mut BitPackReader) -> BitPackResult<Self>;
}

pub trait WriteValue {
    fn write(&self, writer: &mut BitPackWriter) -> BitPackResult;
    fn bits(&self) -> usize;
}

pub trait ReadPackedValue
where
    Self: Sized,
{
    fn read_packed(reader: &mut BitPackReader, bits: usize) -> BitPackResult<Self>;
}

pub trait WritePackedValue {
    fn write_packed(&self, writer: &mut BitPackWriter, bits: usize) -> BitPackResult;
    fn bits_packed(&self, bits: usize) -> usize {
        // this is obvious, but was added for completedness with a default implementation
        bits
    }
}

pub trait ReadArrayValue
where
    Self: Sized,
{
    fn read_array(reader: &mut BitPackReader, length: usize) -> BitPackResult<Self>;
}

pub trait WriteArrayValue {
    fn write_array(&self, writer: &mut BitPackWriter) -> BitPackResult;
    fn bits_array(&self) -> usize;
}

pub trait ReadPackedArrayValue
where
    Self: Sized,
{
    fn read_packed_array(
        reader: &mut BitPackReader,
        length: usize,
        bits: usize,
    ) -> BitPackResult<Self>;
}

pub trait WritePackedArrayValue {
    fn write_packed_array(&self, writer: &mut BitPackWriter, bits: usize) -> BitPackResult;
    fn bits_packed_array(&self, bits: usize) -> usize;
}

pub trait ReadUnionValue
where
    Self: Sized,
{
    fn read_union(reader: &mut BitPackReader, variant: usize) -> BitPackResult<Self>;
}

pub trait UnionVariant
where
    Self: Sized,
{
    /// Returns the variant index for current union value.
    fn variant(&self) -> usize;
}
