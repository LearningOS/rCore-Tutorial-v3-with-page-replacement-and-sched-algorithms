#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

static TESTS: &[&str] = &[
    "lottery0",
    "lottery1",
    "lottery2",
    "lottery3",
    "lottery4",
    "lottery5",
];

static PRIORITY: [usize; 6] = [
    5, 6, 7, 8, 9, 10
];


use user_lib::{exec_with_args, fork, get_time, sleep};

#[no_mangle]
pub fn main() -> i32 {
    for (i, &test) in TESTS.iter().enumerate() {     
        let start = get_time();
        println!("{} Arrive at {}", test, start);
        let pid = fork();
        if pid == 0 {
            exec_with_args(test, (&PRIORITY[i]) as *const _ as usize);
            panic!("unreachable!");
        }
    }
    sleep(50000);
    for (i, &test) in TESTS.iter().enumerate() {     
        let start = get_time();
        println!("{} Arrive at {}", test, start);
        let pid = fork();
        if pid == 0 {
            exec_with_args(test, (&PRIORITY[i]) as *const _ as usize);
            panic!("unreachable!");
        }
    }
    0
}
