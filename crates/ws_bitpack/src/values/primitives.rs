use crate::*;

impl ReadValue for bool {
    fn read(reader: &mut BitPackReader) -> BitPackResult<Self> {
        reader.read_bit()
    }
}

impl WriteValue for bool {
    fn write(&self, writer: &mut BitPackWriter) -> BitPackResult {
        writer.write_bit(*self)
    }

    fn bits(&self) -> usize {
        1
    }
}

impl ReadValue for f32 {
    fn read(reader: &mut BitPackReader) -> BitPackResult<Self> {
        reader.read_f32()
    }
}

impl WriteValue for f32 {
    fn write(&self, writer: &mut BitPackWriter) -> BitPackResult {
        writer.write_f32(*self)
    }

    fn bits(&self) -> usize {
        32
    }
}

macro_rules! impl_int_readers {
    ( $($t: ident)* ) => {$(
        impl ReadValue for $t {
            fn read(reader: &mut BitPackReader) -> BitPackResult<$t> {
                reader.read_u64($t::BITS as usize).map(|v| v as $t)
            }
        }

        impl WriteValue for $t {
            fn write(&self, writer: &mut BitPackWriter) -> BitPackResult {
                writer.write_u64(*self as u64, $t::BITS as usize)
            }

            fn bits(&self) -> usize {
                $t::BITS as usize
            }
        }

        impl ReadPackedValue for $t {
            fn read_packed(reader: &mut BitPackReader, bits: usize) -> BitPackResult<$t> {
                reader.read_u64(bits).map(|v| v as $t)
            }
        }

        impl WritePackedValue for $t {
            fn write_packed(&self, writer: &mut BitPackWriter, bits: usize) -> BitPackResult {
                writer.write_u64(*self as u64, bits)
            }
        }
    )+};
}

impl_int_readers!(u8 i8 u16 i16 u32 i32 u64 i64 usize isize);
