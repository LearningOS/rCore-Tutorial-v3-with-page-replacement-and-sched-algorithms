#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::{exec_with_args, fork, sched_yield, wait};

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        exec_with_args("mlfqtests", &() as *const _ as usize);
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                sched_yield();
                continue;
            }

            // println!(
            //     "[initproc] Released a zombie process, pid={}, exit_code={}",
            //     pid,
            //     exit_code,
            // );
        }
    }
    0
}
