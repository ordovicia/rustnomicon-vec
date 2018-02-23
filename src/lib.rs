#![feature(allocator_api)]
// clippy
#![allow(unknown_lints)]
#![allow(should_implement_trait)]

pub mod vec;
pub mod into_iter;
pub mod drain;

mod owned_ptr;
mod raw_vec;
mod raw_val_iter;

#[cfg(test)]
mod tests {}
