use crate::syscall_args::*;
use crate::Manager;
use crate::Schedule;

pub struct SyscallHooks {}
impl SyscallHooks {
    pub fn handle_exec<T, I: Copy + Ord>(id: I, args: &ExecArgs, manager:&mut Manager<T, I>) {
        manager.update_exec(id, args);
    }

    pub fn handle_fork<T, I: Copy + Ord>(parent_id: I, child_id: I, manager:&mut Manager<T, I>) {
        manager.update_fork(parent_id, child_id);
    }
}

