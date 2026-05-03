// offset-allocator/src/lib.rs

#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod ext;
pub mod allocator2;

mod small_float;

#[cfg(test)]
mod tests;
