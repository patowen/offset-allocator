// offset-allocator/src/node_index.rs

use std::fmt::{Debug, Display};

use nonmax::{NonMaxU16, NonMaxU32};

/// Determines the number of allocations that the allocator supports.
///
/// By default, [`Allocator`] and related functions use `u32`, which allows for
/// `u32::MAX` allocations. You can, however, use `u16` instead, which
/// causes the allocator to use less memory but limits the number of allocations
/// within a single allocator to at most 65,535.
pub trait NodeIndex: Clone + Copy + Default {
    /// The `NonMax` version of this type.
    ///
    /// This is used extensively to optimize `enum` representations.
    type NonMax: NodeIndexNonMax + TryFrom<Self> + Into<Self>;

    /// The maximum value representable in this type. This is also equal to the
    /// number of valid values that can be held by the `NonMax` type
    const MAX: u32;
}

/// The `NonMax` version of the [`NodeIndex`].
///
/// For example, for `u32`, the `NonMax` version is [`NonMaxU32`].
pub trait NodeIndexNonMax: Clone + Copy + PartialEq + Default + Debug + Display {
    /// Converts from a unsigned 32-bit integer to an instance of this type.
    fn from_u32(val: u32) -> Self;

    /// Converts this type to an unsigned machine word.
    fn to_usize(self) -> usize;
}

impl NodeIndex for u32 {
    type NonMax = NonMaxU32;
    const MAX: u32 = u32::MAX;
}

impl NodeIndex for u16 {
    type NonMax = NonMaxU16;
    const MAX: u32 = u16::MAX as u32;
}

impl NodeIndexNonMax for NonMaxU32 {
    fn from_u32(val: u32) -> Self {
        NonMaxU32::new(val).unwrap()
    }

    fn to_usize(self) -> usize {
        u32::from(self) as usize
    }
}

impl NodeIndexNonMax for NonMaxU16 {
    fn from_u32(val: u32) -> Self {
        NonMaxU16::new(u16::try_from(val).unwrap()).unwrap()
    }

    fn to_usize(self) -> usize {
        u16::from(self) as usize
    }
}
