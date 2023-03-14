#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{get_time, sleep, sleep_noblock};

const PEROID: usize = 1000;
const CPU_TIME: usize = 200;


#[no_mangle]
pub fn main() -> i32 {
    for i in 0..4 {
        let start = get_time();
        println!("edf2 begin: iter={} time={}", i, start);
        sleep(CPU_TIME);
        println!("edf2 end: iter={} time={}", i, get_time());
        sleep_noblock(PEROID - CPU_TIME);
    }
    0
}