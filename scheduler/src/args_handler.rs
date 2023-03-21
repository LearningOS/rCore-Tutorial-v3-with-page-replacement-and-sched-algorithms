use crate::syscall_args::*;
use crate::Manager;
use crate::Schedule;
use crate::Manage;

pub struct SyscallHooks {}
impl SyscallHooks {
    pub fn handle_exec<T, I: Copy + Ord>(id: I, args: &ExecArgs, manager:&mut Manager<T, I>) {
        manager.update_exec(id, args);
    }

    pub fn handle_fork<T, I: Copy + Ord>(parent_id: I, child_id: I, manager:&mut Manager<T, I>) {
        manager.update_fork(parent_id, child_id);
    }

    pub fn handle_thread_create<T, I: Copy + Ord>(parent_id: I, child_id: I, manager:&mut Manager<T, I>) {
        SyscallHooks::handle_fork(parent_id, child_id, manager);
    }

    pub fn handle_sleep<T, I: Copy + Ord>(id: I, manager:&mut Manager<T, I>) {
        manager.update_sleep(id);
    }
}

pub struct KernelHook {}
impl KernelHook {
    pub fn handle_sched_to<T, I: Copy + Ord, MT: Manage<T, I> + Schedule<I>>(id: I, manager:&mut MT, time: usize) {
        manager.update_sched_to(id, time);
    }

    pub fn handle_suspend<T, I: Copy + Ord, MT: Manage<T, I> + Schedule<I>>(id: I, manager:&mut MT, time: usize) {
        manager.update_suspend(id, time)
    }
}