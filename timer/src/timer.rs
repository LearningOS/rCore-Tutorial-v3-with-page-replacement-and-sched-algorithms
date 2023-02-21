use sbi_rt::*;
use riscv::register::{time, sie};

// config for timer here
const CLOCK_FREQ: usize = 12500000;
const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;
const NSEC_PER_SEC: usize = 1000_000_000;

pub struct TrapTimer {
    use_timer: bool
}

impl TrapTimer {
    pub const fn new() -> Self {
        if cfg!(feature = "timer") {
            Self { use_timer: true }
        } else {
            Self { use_timer: false }
        }
    }

    pub fn init(&self) {
        if self.use_timer {
            unsafe {
                sie::set_stimer();
                self.set_next_trigger();
            }
        }
    }

    pub fn get_time() -> usize {
        time::read()
    }

    pub fn get_time_ms() -> usize {
        time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
    }
    
    pub fn set_next_trigger(&self) {
        if self.use_timer {
            set_timer((Self::get_time() + CLOCK_FREQ / TICKS_PER_SEC) as u64);
        } else {
            panic!("Shouldn't set timer when stimer is not enabled!");
        }
    }

    pub fn is_timer_enabled(&self) -> bool{
        self.use_timer
    }
}

