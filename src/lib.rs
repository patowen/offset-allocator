// offset-allocator/src/lib.rs

#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]
#![warn(missing_docs)]

mod allocator;
mod bins_map;
mod node_index;
mod small_float;
pub use allocator::*;
