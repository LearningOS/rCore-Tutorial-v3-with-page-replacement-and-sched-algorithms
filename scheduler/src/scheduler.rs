use core::marker::Copy;
use core::cmp::Ord;
use core::option::Option;

/// Scheduler
pub trait Schedule<I: Copy + Ord> {
    /// 入队
    fn add(&mut self, id: I);
    /// 出队
    fn fetch(&mut self) -> Option<I>;
}
