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

// TODO: remove tests below (or put them in test mod)

struct TestStruct {
    field: u64,
    other_field: Vec<u64>,
    other_field2: Vec<u64>,
}

impl SimpleReader<TestStruct> for MessageReader<TestStruct> {
    fn read(reader: &mut BitPackReader) -> Result<TestStruct, BitPackReaderError> {
        Ok(TestStruct {
            field: MessageReader::read(reader)?,
            other_field: MessageReader::read_list(reader, 10)?,
            other_field2: MessageReader::read_packed_list(reader, 10, 10)?,
        })
    }
}

enum TestUnion {
    First { field: u64 },
    Second { field: String },
}

impl UnionReader<TestUnion> for MessageReader<TestUnion> {
    fn read_union(
        reader: &mut BitPackReader,
        variant: usize,
    ) -> Result<TestUnion, BitPackReaderError> {
        Ok(match variant {
            0 => TestUnion::First {
                field: MessageReader::read(reader)?,
            },
            1 => TestUnion::Second {
                field: MessageReader::read_ascii(reader)?,
            },
            _ => panic!("Invalid union variant."), // TODO: use an Error instead
        })
    }
}

fn test_it() -> Result<String, BitPackReaderError> {
    let mut cursor = std::io::Cursor::new(vec![10]);
    let mut reader = BitPackReader::new(&mut cursor);
    MessageReader::<u64>::read(&mut reader).unwrap();
    MessageReader::<u64>::read_packed(&mut reader, 30).unwrap();
    MessageReader::<String>::read_ascii(&mut reader).unwrap();
    MessageReader::<Vec<u64>>::read_list(&mut reader, 10).unwrap();
    MessageReader::<Vec<u64>>::read_packed_list(&mut reader, 10, 30).unwrap();
    MessageReader::<String>::read(&mut reader)
}
