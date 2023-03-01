#![no_std]
extern crate sbi_rt;
extern crate riscv;
extern crate rcore_utils;

mod timer;

pub use timer::TrapTimer;
pub static mut TIMER: TrapTimer = TrapTimer::new();