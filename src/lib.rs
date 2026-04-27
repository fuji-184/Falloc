#![cfg_attr(feature = "nightly", feature(allocator_api))]

#![feature(array_try_from_fn)]

#[cfg(feature = "nightly")]
pub mod heap_arena;

mod stack_arena;

pub use stack_arena::StackArena;

pub mod heap_pool;

mod error;
pub use error::Error;