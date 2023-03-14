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

use user_lib::{exec_with_args, fork, get_time}; 

#[no_mangle]
pub fn main() -> i32 {
    for (i, test) in TESTS.iter().enumerate() {
        let st = get_time();
        println!("{} Arriving at {}", test, st);
        let pid = fork();
        if pid == 0 {
            exec_with_args(*test,(&(PEROIDS[i], DEADLINES[i] + st)) as *const _ as usize);
            panic!("unreachable!");
        }
    }
    0
}