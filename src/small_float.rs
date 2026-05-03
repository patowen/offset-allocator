// offset-allocator/src/small_float.rs

pub const MANTISSA_BITS: u32 = 3;
pub const MANTISSA_VALUE: u32 = 1 << MANTISSA_BITS;
pub const MANTISSA_MASK: u32 = MANTISSA_VALUE - 1;

pub const NUM_LEAF_BINS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmallFloat(u32);

impl SmallFloat {
    pub fn values() -> impl ExactSizeIterator<Item = Self> {
        (0..(NUM_LEAF_BINS as u32)).map(|i| Self(i))
    }

    pub fn from_u32_round_up(value: u32) -> Self {
        let mut exp = 0;
        let mut mantissa;

        if value < MANTISSA_VALUE {
            // Denorm: 0..(MANTISSA_VALUE-1)
            mantissa = value
        } else {
            // Normalized: Hidden high bit always 1. Not stored. Just like float.
            let highest_set_bit = value.ilog2();
            let mantissa_start_bit = highest_set_bit - MANTISSA_BITS;
            exp = mantissa_start_bit + 1;
            mantissa = (value >> mantissa_start_bit) & MANTISSA_MASK;

            let low_bits_mask = (1 << mantissa_start_bit) - 1;

            // Round up!
            if (value & low_bits_mask) != 0 {
                mantissa += 1;
            }
        }

        // + allows mantissa->exp overflow for round up
        SmallFloat((exp << MANTISSA_BITS) + mantissa)
    }

    pub fn from_u32_round_down(value: u32) -> Self {
        let mut exp = 0;
        let mantissa;

        if value < MANTISSA_VALUE {
            // Denorm: 0..(MANTISSA_VALUE-1)
            mantissa = value
        } else {
            // Normalized: Hidden high bit always 1. Not stored. Just like float.
            let highest_set_bit = value.ilog2();
            let mantissa_start_bit = highest_set_bit - MANTISSA_BITS;
            exp = mantissa_start_bit + 1;
            mantissa = (value >> mantissa_start_bit) & MANTISSA_MASK;
        }

        SmallFloat((exp << MANTISSA_BITS) | mantissa)
    }

    pub fn to_u32(self) -> u32 {
        let exponent = self.0 >> MANTISSA_BITS;
        let mantissa = self.0 & MANTISSA_MASK;
        if exponent == 0 {
            mantissa
        } else {
            (mantissa | MANTISSA_VALUE) << (exponent - 1)
        }
    }

    #[inline]
    pub fn reinterpret_as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn reinterpret_u32(data: u32) -> Self {
        Self(data)
    }
}

#[derive(Debug)]
pub struct SmallFloatMap<T>([T; NUM_LEAF_BINS]);

impl<T: Default + Copy> Default for SmallFloatMap<T> {
    fn default() -> Self {
        Self([T::default(); NUM_LEAF_BINS])
    }
}

impl<T> std::ops::Index<SmallFloat> for SmallFloatMap<T> {
    type Output = T;

    fn index(&self, index: SmallFloat) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

impl<T> std::ops::IndexMut<SmallFloat> for SmallFloatMap<T> {
    fn index_mut(&mut self, index: SmallFloat) -> &mut Self::Output {
        &mut self.0[index.0 as usize]
    }
}
