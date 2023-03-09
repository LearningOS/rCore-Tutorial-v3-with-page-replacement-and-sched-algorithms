#![no_std]
mod scheduler;
mod manager;
pub use scheduler::Schedule;
pub use manager::Manage;

extern crate alloc;
extern crate rcore_utils;

mod syscall_args;
mod args_handler;

#[cfg(feature = "seq")]
mod default_manager;
#[cfg(feature = "seq")]
pub use default_manager::DefaultManager as Manager;

#[cfg(feature = "sjf")]
mod sjf;
#[cfg(feature = "sjf")]
pub use sjf::SJFManager as Manager;

#[cfg(feature = "stcf")]
mod stcf;
#[cfg(feature = "stcf")]
pub use stcf::STCFManager as Manager;

#[cfg(feature = "hrrn")]
mod hrrn;
#[cfg(feature = "hrrn")]
pub use hrrn::HRRNManager as Manager;

pub use args_handler::{SyscallHooks, KernelHook};