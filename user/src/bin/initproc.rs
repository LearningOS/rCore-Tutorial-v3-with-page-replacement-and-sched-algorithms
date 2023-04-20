#![no_std]
#![no_main]

extern crate user_lib;

use rcore_console::log::info;
use user_lib::{exec_with_args, fork, sched_yield, wait, get_args};

#[no_mangle]
fn main() -> i32 {
    let proc_name = if cfg!(feature = "seq") {
        "user_shell"
    }
    else if cfg!(feature = "sjf") || cfg!(feature = "stcf") || cfg!(feature = "hrrn") {
        "sjftests"
    } else if cfg!(feature = "stride") {
        "stridetests"
    } else if cfg!(feature = "lottery") {
        "lotterytests"
    } else if cfg!(feature = "mlfq") {
        "mlfqtests"
    } else if cfg!(feature = "edf") {
        "edftests"
    } else if cfg!(feature = "rms") {
        "rmstests"
    } else {
        panic!("unsupported sched method!");
    };

    if fork() == 0 {
        exec_with_args(proc_name, get_args(proc_name));
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
