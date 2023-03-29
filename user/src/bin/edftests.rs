#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

static TESTS: &[&str] = &[
    "edf0",
    "edf1",
    "edf2"
];

static DEADLINES:[isize; 3] = [
    700, 400, 800
];

static PEROIDS: [isize; 3] = [
    2000, 500, 1000
];

const INIT_DIFF: isize = 4327 - 1771;

use user_lib::{exec_with_args, fork, get_time}; 

#[no_mangle]
pub fn main() -> i32 {
    let st = get_time();
    for (i, test) in TESTS.iter().enumerate() {
        println!("{} Arriving at {}", test, get_time());
        let pid = fork();
        if pid == 0 {
            exec_with_args(*test,(&(PEROIDS[i], DEADLINES[i] + st + INIT_DIFF)) as *const _ as usize);
            panic!("unreachable!");
        }
    }
    0
}