// offset-allocator/src/lib.rs

#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod allocator2;
pub mod ext;

mod bins_map;
mod small_float;
mod node_index;

#[cfg(test)]
mod tests;
