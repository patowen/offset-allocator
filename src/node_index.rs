// offset-allocator/src/node_index.rs

use std::fmt::{Debug, Display};

use nonmax::{NonMaxU16, NonMaxU32};

/// The index used to identify nodes in the allocator. Determines the number of allocations
/// that the allocator supports.
///
/// By default, [`Allocator`] and related functions use `u32`, which allows for
/// `u32::MAX` allocations. You can, however, use `u16` instead, which
/// causes the allocator to use less memory but limits the number of allocations
/// within a single allocator to at most 65,535.
pub trait NodeIndex: Display + Debug + Clone + Copy + PartialEq + Eq {
    /// The number of indexes, consecutive starting from 0, that are valid representations
    const NUM_VALID: u32;

    /// Converts from a unsigned 32-bit integer to an instance of this type.
    fn from_u32(val: u32) -> Self;

    /// Converts this type to an unsigned machine word.
    fn to_usize(self) -> usize;
}

impl NodeIndex for NonMaxU32 {
    const NUM_VALID: u32 = u32::MAX;

    fn from_u32(val: u32) -> Self {
        NonMaxU32::try_from(val).unwrap()
    }

    fn to_usize(self) -> usize {
        self.get() as usize
    }
}

impl NodeIndex for NonMaxU16 {
    const NUM_VALID: u32 = u16::MAX as u32;

    fn from_u32(val: u32) -> Self {
        NonMaxU16::try_from(u16::try_from(val).unwrap()).unwrap()
    }

    fn to_usize(self) -> usize {
        self.get() as usize
    }
}

/// A convenience trait that allows allocators to be defined with `u16` and `u32` instead
/// of `NonMaxU16` and NonMaxU32
pub trait RawNodeIndex {
    type NodeIndex: NodeIndex;
}

impl RawNodeIndex for u16 {
    type NodeIndex = NonMaxU16;
}

impl RawNodeIndex for u32 {
    type NodeIndex = NonMaxU32;
}
