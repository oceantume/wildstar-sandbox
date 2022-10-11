use ws_bitpack::{BitPackReader, BitPackReaderError, BitPackWriter, BitPackWriterError};

use crate::MessageStruct;

/// Message field metadata.
#[derive(Default)]
pub struct MessageFieldMetadata {
    /// Defines a specific number of bits in which to represent an integer.
    ///
    /// In vecs and slices of integers, this determines the bits used by each item.
    pub bits: Option<usize>,
    /// Defines the length of a vec.
    pub length: Option<usize>,
    /// Defines whether a string should be read as an ascii string.
    pub ascii: bool,
}

pub trait MessageValue
where
    Self: Sized,
{
    fn unpack(
        reader: &mut BitPackReader,
        metadata: &MessageFieldMetadata,
    ) -> Result<Self, BitPackReaderError>;
    fn pack(
        &self,
        writer: &mut BitPackWriter,
        metadata: &MessageFieldMetadata,
    ) -> Result<(), BitPackWriterError>;
}

macro_rules! impl_int_message_field_value {
    ( $type: ident, $bits: expr ) => {
        impl MessageValue for $type {
            fn unpack(
                reader: &mut BitPackReader,
                metadata: &MessageFieldMetadata,
            ) -> Result<Self, BitPackReaderError> {
                reader
                    .read_u64(metadata.bits.unwrap_or($bits))
                    .map(|v| v as $type)
            }

            fn pack(
                &self,
                writer: &mut BitPackWriter,
                metadata: &MessageFieldMetadata,
            ) -> Result<(), BitPackWriterError> {
                let bits = metadata.bits.unwrap_or($bits);
                debug_assert!((*self as u128) < (1u128 << bits));
                writer.write_u64(*self as u64, bits)
            }
        }
    };
}

impl_int_message_field_value!(u8, 8);
impl_int_message_field_value!(i8, 8);
impl_int_message_field_value!(u16, 16);
impl_int_message_field_value!(i16, 16);
impl_int_message_field_value!(u32, 32);
impl_int_message_field_value!(i32, 32);
impl_int_message_field_value!(u64, 64);
impl_int_message_field_value!(i64, 64);

impl MessageValue for bool {
    fn unpack(
        reader: &mut BitPackReader,
        _metadata: &MessageFieldMetadata,
    ) -> Result<Self, BitPackReaderError> {
        reader.read_bit()
    }

    fn pack(
        &self,
        writer: &mut BitPackWriter,
        _metadata: &MessageFieldMetadata,
    ) -> Result<(), BitPackWriterError> {
        writer.write_bit(*self)
    }
}

impl MessageValue for f32 {
    fn unpack(
        reader: &mut BitPackReader,
        _metadata: &MessageFieldMetadata,
    ) -> Result<Self, BitPackReaderError> {
        reader.read_f32()
    }

    fn pack(
        &self,
        writer: &mut BitPackWriter,
        _metadata: &MessageFieldMetadata,
    ) -> Result<(), BitPackWriterError> {
        writer.write_f32(*self)
    }
}

impl<T> MessageValue for Vec<T>
where
    T: MessageValue,
{
    fn unpack(
        reader: &mut BitPackReader,
        metadata: &MessageFieldMetadata,
    ) -> Result<Self, BitPackReaderError> {
        let length = metadata.length.unwrap_or(0);
        let value = Self::with_capacity(length);

        while value.len() < length {
            T::unpack(
                reader,
                &MessageFieldMetadata {
                    length: None,
                    ..*metadata
                },
            )?;
        }

        Ok(value)
    }

    fn pack(
        &self,
        writer: &mut BitPackWriter,
        metadata: &MessageFieldMetadata,
    ) -> Result<(), BitPackWriterError> {
        // TODO: use an Error instead to prevent parser panics.
        debug_assert_eq!(
            metadata.length.expect("Length metadata must be defined."),
            self.len()
        );

        for item in self.iter() {
            item.pack(writer, &MessageFieldMetadata {
                length: None,
                ..*metadata
            })?;
        }

        Ok(())
    }
}

impl MessageValue for String {
    fn unpack(
        _reader: &mut BitPackReader,
        _metadata: &MessageFieldMetadata,
    ) -> Result<Self, BitPackReaderError> {
        todo!("Implement string unpacking.")
    }

    fn pack(
        &self,
        _writer: &mut BitPackWriter,
        _metadata: &MessageFieldMetadata,
    ) -> Result<(), BitPackWriterError> {
        todo!("Implement string packing.")
    }
}

impl<T> MessageValue for T
where
    T: MessageStruct,
{
    fn unpack(
        reader: &mut BitPackReader,
        _metadata: &MessageFieldMetadata,
    ) -> Result<Self, BitPackReaderError> {
        MessageStruct::unpack(reader)
    }

    fn pack(
        &self,
        writer: &mut BitPackWriter,
        _metadata: &MessageFieldMetadata,
    ) -> Result<(), BitPackWriterError> {
        MessageStruct::pack(self, writer)
    }
}
