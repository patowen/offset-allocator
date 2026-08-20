// offset-allocator/src/small_float.rs

//! This module handles operations around small-floats, which are 8-bit unsigned floating point
//! values that represents integer values using a 3-bit mantissa and a 5-bit exponent. Each of
//! these 256 values correspond to a specific bin, which determines the size of the allocations
//! supported by each bin.

/// The number of bits that represent the mantissa
pub const MANTISSA_BITS: u32 = 3;

/// The number of possible mantissa values. This number is a power of 2.
pub const MANTISSA_VALUE: u32 = 1 << MANTISSA_BITS;

/// A mask that can be bitwise-anded with the float to get just the mantissa
pub const MANTISSA_MASK: u32 = MANTISSA_VALUE - 1;

// Bin sizes follow floating point (exponent + mantissa) distribution (piecewise linear log approx)
// This ensures that for each size class, the average overhead percentage stays the same

/// The least small-float greater than or equal to the given value
pub fn uint_to_float_round_up(size: u32) -> u32 {
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

    // Using `+` instead of `|` allows mantissa->exp overflow for round up
    (exp << MANTISSA_BITS) + mantissa
}

/// The greatest small-float less than or equal to the given value
pub fn uint_to_float_round_down(size: u32) -> u32 {
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

    (exp << MANTISSA_BITS) | mantissa
}

/// The `u32` that holds the same value as the small-float
pub fn float_to_uint(float_value: u32) -> u32 {
    let exponent = float_value >> MANTISSA_BITS;
    let mantissa = float_value & MANTISSA_MASK;
    if exponent == 0 {
        mantissa
    } else {
        (mantissa | MANTISSA_VALUE) << (exponent - 1)
    }
}
