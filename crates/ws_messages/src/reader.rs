use std::marker::PhantomData;

use ws_bitpack::{BitPackReader, BitPackReaderError};

/// An empty type that only exposes static functions used to read message
/// values from a [`BitPackReader`].
///
/// This type doesn't do anything by itself, but it has implementations for
/// different reader traits, providing a type-safe way read complex values
/// from the bitpacked stream.
///
/// PhantomData is used to satisfy the compiler since we're not using `T`
/// directly and it's only used for trait implementation resolution.
pub struct MessageReader<T>(PhantomData<*const T>);

pub trait SimpleReader<T> {
    /// Reads a simple value.
    fn read(reader: &mut BitPackReader) -> Result<T, BitPackReaderError>;
}

pub trait PackedReader<T> {
    /// Reads a bit-packed value.
    fn read_packed(reader: &mut BitPackReader, bits: usize) -> Result<T, BitPackReaderError>;
}

pub trait AsciiReader<T> {
    /// Reads an ascii string.
    fn read_ascii(reader: &mut BitPackReader) -> Result<T, BitPackReaderError>;
}

pub trait ListReader<T> {
    /// Reads a list of simple values.
    ///
    /// Uses [`SimpleReader::read`] to read each item.
    fn read_list(reader: &mut BitPackReader, length: usize) -> Result<T, BitPackReaderError>;
}

pub trait PackedListReader<T> {
    /// Reads a list of packed values.
    ///
    /// Uses [`PackedReader::read_packed`] to read each item.
    fn read_packed_list(
        reader: &mut BitPackReader,
        length: usize,
        bits: usize,
    ) -> Result<T, BitPackReaderError>;
}

pub trait UnionReader<T> {
    /// Reads a union value.
    ///
    /// Unions are enums with structured variants that are resolved using
    /// a 0-based variant index.
    fn read_union(reader: &mut BitPackReader, variant: usize) -> Result<T, BitPackReaderError>;
}

impl SimpleReader<String> for MessageReader<String> {
    fn read(reader: &mut BitPackReader) -> Result<String, BitPackReaderError> {
        reader.read_string(true)
    }
}

impl AsciiReader<String> for MessageReader<String> {
    fn read_ascii(reader: &mut BitPackReader) -> Result<String, BitPackReaderError> {
        reader.read_string(false)
    }
}

impl<T> ListReader<Vec<T>> for MessageReader<Vec<T>>
where
    MessageReader<T>: SimpleReader<T>,
{
    fn read_list(reader: &mut BitPackReader, length: usize) -> Result<Vec<T>, BitPackReaderError> {
        let mut vec = Vec::<T>::with_capacity(length);
        while vec.len() < length {
            vec.push(MessageReader::<T>::read(reader)?);
        }
        Ok(vec)
    }
}

impl<T> PackedListReader<Vec<T>> for MessageReader<Vec<T>>
where
    MessageReader<T>: PackedReader<T>,
{
    fn read_packed_list(
        reader: &mut BitPackReader,
        length: usize,
        bits: usize,
    ) -> Result<Vec<T>, BitPackReaderError> {
        let mut vec = Vec::<T>::with_capacity(length);
        while vec.len() < length {
            vec.push(MessageReader::<T>::read_packed(reader, bits)?);
        }
        Ok(vec)
    }
}

macro_rules! impl_int_readers {
    ( $t: ident, $bits: literal ) => {
        impl SimpleReader<$t> for MessageReader<$t> {
            fn read(
                reader: &mut BitPackReader
            ) -> Result<$t, BitPackReaderError> {
                reader.read_u64($bits).map(|v| v as $t)
            }
        }

        impl PackedReader<$t> for MessageReader<$t> {
            fn read_packed(
                reader: &mut BitPackReader,
                bits: usize
            ) -> Result<$t, BitPackReaderError> {
                reader.read_u64(bits).map(|v| v as $t)
            }
        }
    }
}

impl_int_readers!(u8, 8);
impl_int_readers!(i8, 8);
impl_int_readers!(u16, 16);
impl_int_readers!(i16, 16);
impl_int_readers!(u32, 32);
impl_int_readers!(i32, 32);
impl_int_readers!(u64, 64);
impl_int_readers!(i64, 64);
