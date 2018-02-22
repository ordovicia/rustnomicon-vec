#![feature(allocator_api)]
// clippy
#![allow(unknown_lints)]
#![allow(should_implement_trait)]
#![allow(new_without_default)]

pub mod vec;
pub mod into_iter;
mod owned_ptr;

#[cfg(test)]
mod tests {}
