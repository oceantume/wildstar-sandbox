use std::marker::PhantomData;

use ws_bitpack::{BitPackWriter, BitPackWriterError};

pub struct MessageWriter<T>(PhantomData<*const T>);

pub trait SimpleWriter<T> {
    /// Writes a simple value.
    fn write(writer: &mut BitPackWriter, value: &T) -> Result<(), BitPackWriterError>;
}

pub trait PackedWriter<T> {
    /// Writes a bit-packed value.
    fn write_packed(
        writer: &mut BitPackWriter,
        value: &T,
        bits: usize,
    ) -> Result<(), BitPackWriterError>;
}

pub trait AsciiWriter<T> {
    /// Writes an ascii string.
    fn write_ascii(writer: &mut BitPackWriter, value: &T) -> Result<(), BitPackWriterError>;
}

pub trait ListLength<T> {
    /// Returns the length of the list.
    fn list_length(value: &T) -> usize;
}

pub trait UnionVariant<T> {
    /// Returns the 0-based variant index for that union value.
    fn variant(value: &T) -> usize;
}

impl SimpleWriter<String> for MessageWriter<String> {
    fn write(writer: &mut BitPackWriter, value: &String) -> Result<(), BitPackWriterError> {
        writer.write_string(value, true)
    }
}

impl AsciiWriter<String> for MessageWriter<String> {
    fn write_ascii(writer: &mut BitPackWriter, value: &String) -> Result<(), BitPackWriterError> {
        writer.write_string(value, false)
    }
}

impl<T> SimpleWriter<Vec<T>> for MessageWriter<Vec<T>>
where
    MessageWriter<T>: SimpleWriter<T>,
{
    fn write(writer: &mut BitPackWriter, value: &Vec<T>) -> Result<(), BitPackWriterError> {
        for item in value {
            MessageWriter::<T>::write(writer, item)?;
        }
        Ok(())
    }
}

impl<T> ListLength<Vec<T>> for MessageWriter<Vec<T>>
where
    MessageWriter<T>: SimpleWriter<T>,
{
    fn list_length(value: &Vec<T>) -> usize {
        value.len()
    }
}

impl<T> PackedWriter<Vec<T>> for MessageWriter<T>
where
    MessageWriter<T>: PackedWriter<T>,
{
    fn write_packed(
        writer: &mut BitPackWriter,
        value: &Vec<T>,
        bits: usize,
    ) -> Result<(), BitPackWriterError> {
        for item in value {
            MessageWriter::<T>::write_packed(writer, item, bits)?;
        }
        Ok(())
    }
}

macro_rules! impl_int_writers {
    ( $t: ident, $bits: literal ) => {
        impl SimpleWriter<$t> for MessageWriter<$t> {
            fn write(writer: &mut BitPackWriter, value: &$t) -> Result<(), BitPackWriterError> {
                writer.write_u64(*value as u64, $bits)
            }
        }

        impl PackedWriter<$t> for MessageWriter<$t> {
            fn write_packed(
                writer: &mut BitPackWriter,
                value: &$t,
                bits: usize,
            ) -> Result<(), BitPackWriterError> {
                writer.write_u64(*value as u64, bits)
            }
        }
    };
}

impl_int_writers!(u8, 8);
impl_int_writers!(i8, 8);
impl_int_writers!(u16, 16);
impl_int_writers!(i16, 16);
impl_int_writers!(u32, 32);
impl_int_writers!(i32, 32);
impl_int_writers!(u64, 64);
impl_int_writers!(i64, 64);
