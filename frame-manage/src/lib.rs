#![no_std]
extern crate kernel_vm;
extern crate alloc;

mod config;
mod clock_queue;
mod plugins;
mod manager;
mod frame_allocator;
mod virt_frame_swapper;

pub use plugins::Manage;
pub use manager::PageFaultHandler;
pub use frame_allocator::{FRAME_ALLOCATOR, frame_alloc};

const ACCESS_FLAG: usize = 1 << 6;
const DIRTY_FLAG: usize = 1 << 7;

pub const PFF_T: usize = 720000; // 100000 or 720000
pub const WORKSET_NUM: usize = 20; // 5 or 20

#[cfg(feature = "fifo")]
mod fifo;
#[cfg(feature = "fifo")]
pub use fifo::FIFOManager as FrameManager;

#[cfg(feature = "clock")]
mod clock;
#[cfg(feature = "clock")]
pub use clock::ClockManager as FrameManager;

#[cfg(feature = "clock-improve")]
mod clock_improve;
#[cfg(feature = "clock-improve")]
pub use clock_improve::ClockImproveManager as FrameManager;

#[cfg(feature = "pff")]
extern crate rcore_utils;
#[cfg(feature = "pff")]
mod pff;
#[cfg(feature = "pff")]
pub use pff::PffManager as FrameManager;

#[cfg(feature = "work-set")]
mod work_set;
#[cfg(feature = "work-set")]
pub use work_set::WorkSetManager as FrameManager;