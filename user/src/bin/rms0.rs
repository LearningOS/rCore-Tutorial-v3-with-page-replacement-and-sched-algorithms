#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{get_time, sleep, sleep_noblock};

const PEROID: isize = 2000;
const CPU_TIME: usize = 300;
const INIT_DDL: isize = 700;
const INIT_DIFF: isize = 400;
const EPS: isize = 50;


#[no_mangle]
pub fn main() -> i32 {
    let start = get_time() - INIT_DIFF;
    for i in 0..4 {
        println!("rms0 begin: iter={} time={} st={}", i, get_time(), start);

        for t in 0..(CPU_TIME / 100) {
            sleep(100);
        }
        
        let end = get_time();
        let _st = start + i * PEROID;
        
        if _st + INIT_DDL + EPS < end {
            panic!("rms0 timeout: iter={} time={} ddl={}", i, end, _st + INIT_DDL);
        } else {
            println!("rms0 end: iter={} time={} ddl={}", i, end, _st + INIT_DDL);
        }

        sleep_noblock((_st + PEROID - end) as usize);
    }
    0
}