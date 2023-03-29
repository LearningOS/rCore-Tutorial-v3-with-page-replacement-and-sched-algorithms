#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{get_time, sleep, sleep_noblock};

const PEROID: isize = 1000;
const CPU_TIME: usize = 200;
const INIT_DIFF: isize = 200;
const INIT_DDL: isize = 800;
const EPS: isize = 50;


#[no_mangle]
pub fn main() -> i32 {
    let start = get_time() - INIT_DIFF;
    for i in 0..4 {
        println!("rms2 begin: iter={} time={} st={}", i, get_time(), start);
        sleep(CPU_TIME);
        let end = get_time();
        let _st = start + i * PEROID;
        
        if _st + INIT_DDL + EPS < end {
            panic!("rms2 timeout: iter={} time={} ddl={}", i, end, _st + INIT_DDL);
        } else {
            println!("rms2 end: iter={} time={} ddl={}", i, end, _st + INIT_DDL);
        }

        sleep_noblock((_st + PEROID - end) as usize);
    }
    0
}