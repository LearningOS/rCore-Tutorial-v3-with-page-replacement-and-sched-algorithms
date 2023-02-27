#![no_std]
mod scheduler;
mod manager;
pub use scheduler::Schedule;
pub use manager::Manage;

extern crate alloc;

mod default_manager;
mod sjf;
mod syscall_args;
mod args_handler;

#[cfg(feature = "seq")]
pub use default_manager::DefaultManager as Manager;
#[cfg(feature = "sjf")]
pub use sjf::SJFManager as Manager;
pub use args_handler::SyscallHooks;