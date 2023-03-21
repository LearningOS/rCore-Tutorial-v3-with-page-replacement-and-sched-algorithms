#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

static TESTS: &[&str] = &[
    // "mlfq1",
    // "mlfq2",
    "mlfq3",
    "mlfq4",
    "mlfq5",
];


use user_lib::{exec_with_args, fork, sleep_noblock, get_time};

#[no_mangle]
pub fn main() -> i32 {
    let mut i = 0;
    for test in TESTS {     
        let start = get_time();
        println!("{} Arrive at {}", test, start);
        let pid = fork();
        if pid == 0 {
            exec_with_args(*test, &() as *const _ as usize);
            panic!("unreachable!");
        }
        i += 1;
    }
    0
}