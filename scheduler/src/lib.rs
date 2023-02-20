#![no_std]
mod scheduler;
mod manager;
pub use scheduler::Schedule;
pub use manager::Manage;

extern crate alloc;

mod default_manager;


#[cfg(feature = "seq")]
pub use default_manager::DefaultManager as Manager;