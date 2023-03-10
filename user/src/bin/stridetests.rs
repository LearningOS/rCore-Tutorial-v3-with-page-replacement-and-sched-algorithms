#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

static TESTS: &[&str] = &[
    "stride0",
    "stride1",
    "stride2",
    "stride3",
    "stride4",
    "stride5",
];

static PRIORITY: [usize; 6] = [
    5, 6, 7, 8, 9, 10
];

use user_lib::{exec_with_args, fork, get_time};

#[no_mangle]
pub fn main() -> i32 {
    let mut i = 0;
    for test in TESTS {     
        let start = get_time();
        println!("{} Arrive at {}", test, start);
        let pid = fork();
        if pid == 0 {
            exec_with_args(*test, (&PRIORITY[i]) as *const _ as usize);
            panic!("unreachable!");
        }
        i += 1;
    }
    0
}
