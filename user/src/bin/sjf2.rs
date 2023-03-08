#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{sleep_noblock};

#[no_mangle]
pub fn main() -> i32 {
    println!("I am sjf2");
    sleep_noblock(100000);
    0
}
