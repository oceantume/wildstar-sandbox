use ws_bitpack::{BitPackReader, BitPackReaderError, BitPackWriter, BitPackWriterError};
mod macros;
pub use macros::*;
mod values;
pub use values::*;

pub trait Message {
    fn id() -> u32;
}

pub trait MessageStruct
where
    Self: Sized,
{
    fn unpack(reader: &mut BitPackReader) -> Result<Self, BitPackReaderError>;
    fn pack(&self, writer: &mut BitPackWriter) -> Result<(), BitPackWriterError>;
}

pub trait MessageUnion
where
    Self: Sized,
{
    fn variant_id(&self) -> usize;
    fn unpack(reader: &mut BitPackReader, variant_id: usize) -> Result<Self, BitPackReaderError>;
    fn pack(&self, writer: &mut BitPackWriter) -> Result<(), BitPackWriterError>;
}

