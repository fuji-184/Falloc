#![cfg_attr(feature = "nightly", feature(allocator_api))]

#[cfg(feature = "nightly")]
mod heap_arena;

mod stack_arena;

#[cfg(feature = "nightly")]
pub use heap_arena::HeapArena;

pub use stack_arena::StackArena;
