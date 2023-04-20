#![no_std]
extern crate kernel_vm;
extern crate alloc;

mod config;
mod plugins;
mod manager;
mod frame_allocator;
mod virt_frame_swapper;

pub use plugins::Manage;
pub use manager::PageFaultHandler;
pub use frame_allocator::{FRAME_ALLOCATOR, frame_alloc};

pub const MEMORY_END: usize = 0x81000000;

#[cfg(feature = "fifo")]
mod fifo;
#[cfg(feature = "fifo")]
pub use fifo::FIFOManager as FrameManager;