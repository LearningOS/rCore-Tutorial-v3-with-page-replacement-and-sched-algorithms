#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use::user_lib::sleep_noblock;

#[no_mangle]
pub fn main() -> i32 {
    let mut a: usize = 1;
    let mut b: usize = 1;
    let mut c:usize = 0;
    for i in 0..600000 {
        c = (a + b) % 1000007;
        a = b;
        b = c; 

        if i % 100000 == 0{
            sleep_noblock(1);
            println!("mlfq4 running...");
        }
    }
    println!("{}",c);
    println!("mlfq4 OK");
    0
}