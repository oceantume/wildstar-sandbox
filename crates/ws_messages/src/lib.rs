mod macros;
pub use macros::*;

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
    Self: Sized
{
    /// Returns the 0-based variant index for that union value.
    fn variant(&self) -> usize;
}
