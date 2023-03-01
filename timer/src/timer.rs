use sbi_rt::set_timer;
use riscv::register::{time, sie};
use rcore_utils::{CLOCK_FREQ, TICKS_PER_SEC, get_time};

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

    pub fn set_next_trigger(&self) {
        if self.use_timer {
            set_timer((get_time() + CLOCK_FREQ / TICKS_PER_SEC) as u64);
        } else {
            panic!("Shouldn't set timer when stimer is not enabled!");
        }
    }

    pub fn is_timer_enabled(&self) -> bool{
        self.use_timer
    }
}

