#![feature(allocator_api)]
#![feature(test)]
extern crate test;

mod owned_ptr;
pub mod vec;

#[cfg(test)]
mod tests {}
