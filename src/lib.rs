// offset-allocator/src/lib.rs

#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod allocator;
mod bins_map;
mod node_index;
mod small_float;

#[cfg(test)]
mod tests;
