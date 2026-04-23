#![cfg_attr(feature = "nightly", feature(allocator_api))]

#[cfg(feature = "nightly")]
pub mod heap_arena;

mod stack_arena;

pub use stack_arena::StackArena;