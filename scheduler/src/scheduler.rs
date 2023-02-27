use core::marker::Copy;
use core::cmp::Ord;
use core::option::Option;
use crate::syscall_args::*;

/// Scheduler
pub trait Schedule<I: Copy + Ord> {
    /// 入队
    fn add(&mut self, id: I);
    /// 出队
    fn fetch(&mut self) -> Option<I>;
    
    /// info update
    fn update_exec(&mut self, id: I, args: &ExecArgs);
    /// copy info for fork
    fn update_fork(&mut self, parent_id: I, child_id: I);
}
