#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{get_time};

#[no_mangle]
pub fn main() -> i32 {
    let start = get_time();
    println!("I am sjf5");
    println!("current time_msec = {}", start);
    let mut a: usize = 1;
    let mut b: usize = 1;
    let mut c:usize = 0;
    for i in 0..20000000{
        c = (a + b) % 1000007;
        a = b;
        b = c; 
        if i % 5000000 == 0{
            println!("sjf5 running...");
        }
    }
    println!("{}",c);
    let end = get_time();
    println!(
        "time_msec = {}, delta = {}ms, sjf5 OK!",
        end,
        end - start
    );
    0
}
