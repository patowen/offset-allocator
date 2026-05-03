// offset-allocator/src/small_float.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmallFloat(u32);

impl SmallFloat {
    #[inline]
    pub fn reinterpret_as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn reinterpret_u32(data: u32) -> Self {
        Self(data)
    }
}

pub const MANTISSA_BITS: u32 = 3;
pub const MANTISSA_VALUE: u32 = 1 << MANTISSA_BITS;
pub const MANTISSA_MASK: u32 = MANTISSA_VALUE - 1;

// Bin sizes follow floating point (exponent + mantissa) distribution (piecewise linear log approx)
// This ensures that for each size class, the average overhead percentage stays the same
pub fn uint_to_float_round_up(size: u32) -> SmallFloat {
    let mut exp = 0;
    let mut mantissa;

    if size < MANTISSA_VALUE {
        // Denorm: 0..(MANTISSA_VALUE-1)
        mantissa = size
    } else {
        // Normalized: Hidden high bit always 1. Not stored. Just like float.
        let leading_zeros = size.leading_zeros();
        let highest_set_bit = 31 - leading_zeros;

        let mantissa_start_bit = highest_set_bit - MANTISSA_BITS;
        exp = mantissa_start_bit + 1;
        mantissa = (size >> mantissa_start_bit) & MANTISSA_MASK;

        let low_bits_mask = (1 << mantissa_start_bit) - 1;

        // Round up!
        if (size & low_bits_mask) != 0 {
            mantissa += 1;
        }
    }

    // + allows mantissa->exp overflow for round up
    SmallFloat((exp << MANTISSA_BITS) + mantissa)
}

pub fn uint_to_float_round_down(size: u32) -> SmallFloat {
    let mut exp = 0;
    let mantissa;

    if size < MANTISSA_VALUE {
        // Denorm: 0..(MANTISSA_VALUE-1)
        mantissa = size
    } else {
        // Normalized: Hidden high bit always 1. Not stored. Just like float.
        let leading_zeros = size.leading_zeros();
        let highest_set_bit = 31 - leading_zeros;

        let mantissa_start_bit = highest_set_bit - MANTISSA_BITS;
        exp = mantissa_start_bit + 1;
        mantissa = (size >> mantissa_start_bit) & MANTISSA_MASK;
    }

    SmallFloat((exp << MANTISSA_BITS) | mantissa)
}

pub fn float_to_uint(float_value: SmallFloat) -> u32 {
    let exponent = float_value.0 >> MANTISSA_BITS;
    let mantissa = float_value.0 & MANTISSA_MASK;
    if exponent == 0 {
        mantissa
    } else {
        (mantissa | MANTISSA_VALUE) << (exponent - 1)
    }
}
