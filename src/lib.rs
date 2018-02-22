#![feature(allocator_api)]
// clippy
#![allow(unknown_lints)]
#![allow(should_implement_trait)]

pub mod vec;
pub mod into_iter;

mod owned_ptr;
mod raw_vec;

#[cfg(test)]
mod tests {}
