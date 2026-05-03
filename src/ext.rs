//! Extension functions not present in the original C++ `OffsetAllocator`.

use crate::small_float::SmallFloat;

/// Returns the minimum allocator size needed to hold an object of the given
/// size.
pub fn min_allocator_size(needed_object_size: u32) -> u32 {
    SmallFloat::from_u32_round_up(needed_object_size).to_u32()
}
