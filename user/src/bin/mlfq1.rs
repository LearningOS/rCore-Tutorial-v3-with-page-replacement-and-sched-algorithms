#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main() -> i32 {
    let mut a: usize = 1;
    let mut b: usize = 1;
    let mut c:usize = 0;
    for i in 0..60000000{
        c = (a + b) % 1000007;
        a = b;
        b = c; 
        if i % 4000000 == 0{
            println!("mlfq1 running...");
        }
    }
    println!("{}",c);
    println!("mlfq1 OK");
    0
}