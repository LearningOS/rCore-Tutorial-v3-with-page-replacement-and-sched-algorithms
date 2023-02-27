#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{sleep};

#[no_mangle]
pub fn main() -> i32 {
    println!("I am sjf1");
    sleep(100000);
    0
}
