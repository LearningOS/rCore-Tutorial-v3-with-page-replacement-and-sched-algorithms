#![no_std]
extern crate sbi_rt;
extern crate riscv;
extern crate rcore_utils;
extern crate alloc;

mod timer;

pub use timer::TrapTimer;
pub static mut TIMER: TrapTimer = TrapTimer::new();