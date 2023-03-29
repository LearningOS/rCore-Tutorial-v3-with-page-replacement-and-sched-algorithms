#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

static TESTS: &[&str] = &[
    "rms0",
    "rms1",
    "rms2"
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
        println!("{} Arriving at {}", test, get_time());
        let pid = fork();
        if pid == 0 {
            exec_with_args(*test,(&PEROIDS[i]) as *const _ as usize);
            panic!("unreachable!");
        }
    }
    0
}