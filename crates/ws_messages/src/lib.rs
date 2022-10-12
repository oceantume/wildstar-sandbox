mod macros;
pub use macros::*;
pub mod reader;
pub mod writer;

pub trait Message {
    fn id() -> u32;
}

pub trait MessageStruct
where
    Self: Sized,
{
}

pub trait MessageUnion
where
    Self: Sized,
{
}
