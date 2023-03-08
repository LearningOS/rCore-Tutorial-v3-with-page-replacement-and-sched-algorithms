use alloc::collections::{BinaryHeap, VecDeque};
use sbi_rt::set_timer;
use riscv::register::{time, sie};
use rcore_utils::{CLOCK_FREQ, TICKS_PER_SEC, NSEC_PER_SEC, get_time};
use core::cmp::{Ord, Ordering};
use core::option::Option;


struct TimerBlock {
    id: usize,
    expire_ticks: usize
}

impl PartialOrd for TimerBlock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.expire_ticks.partial_cmp(&self.expire_ticks)
    }
}

impl PartialEq for TimerBlock {
    fn eq(&self, other: &Self) -> bool {
        self.expire_ticks == other.expire_ticks
    }
}

impl Ord for TimerBlock {
    fn cmp(&self, other: &Self) -> Ordering {
        other.expire_ticks.cmp(&self.expire_ticks)
    }
}

impl Eq for TimerBlock {}

pub struct TrapTimer {
    use_timer: bool,
    timer_queue: Option<BinaryHeap<TimerBlock>>
}

impl TrapTimer {
    pub const fn new() -> Self {
        if cfg!(feature = "timer") {
            Self { use_timer: true, timer_queue: None }
        } else {
            Self { use_timer: false, timer_queue: None }
        }
    }

    pub fn init(&mut self) {
        if self.use_timer {
            unsafe {
                sie::set_stimer();
                self.set_next_trigger();
            }
        }
        self.timer_queue = Some(BinaryHeap::new());
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

    pub fn add_timer(&mut self, task_id: usize, period_ms: usize) {
        self.timer_queue.as_mut().unwrap().push(TimerBlock { 
            id: task_id, 
            expire_ticks: get_time() + period_ms * 1000_000 / (NSEC_PER_SEC / CLOCK_FREQ)
        });
    }

    pub fn check_timer(&mut self) -> VecDeque<usize> {
        let current_time = get_time();
        let mut ret: VecDeque<usize> = VecDeque::new();
        while let Some(timer) = self.timer_queue.as_mut().unwrap().peek() {
            if timer.expire_ticks <= current_time {
                ret.push_back(self.timer_queue.as_mut().unwrap().pop().unwrap().id);
            } else {
                break;
            }
        }
        ret
    }
}

