mod reader;
mod writer;
mod values;

pub use reader::*;
pub use writer::*;
pub use values::*;

#[derive(Debug)]
pub enum BitPackError {
    FromUtf16(std::string::FromUtf16Error),
    OutOfBounds,
}

pub type BitPackResult<T = ()> = Result<T, BitPackError>;
