#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

static TESTS: &[&str] = &[
    "sjf1",
    "sjf2",
    "sjf3",
    "sjf4",
    "sjf5",
];

static TIMES: [usize;5] = [
    10000,
    100000,
    3000,
    1200,
    1100,
];

use user_lib::{exec_with_args, fork, sleep_noblock, get_time};

#[no_mangle]
pub fn main() -> i32 {
    let mut i = 0;
    for test in TESTS { 
        if i == 3 || i == 4{
            sleep_noblock(1000);
        }
        let start = get_time();
        println!("{} Arrive at {}", test, start);
        let pid = fork();
        if pid == 0 {
            exec_with_args(*test, (&TIMES[i]) as *const _ as usize);
            panic!("unreachable!");
        }
        i += 1;
    }
    0
}
