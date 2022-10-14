#[link(name = "simd")]
extern "C" {
    fn asm_i32_from_str16_sse (s: &[u8]) -> i32;
}

use std::time::{Instant};
use rust_io;

const ITERATIONS: usize = 1_000_000;

fn time <T> (fun: &dyn Fn(&String) -> T, target: T)
    where T: Default + std::cmp::PartialEq + std::fmt::Display,
{
    let stringify = "0".repeat(1) + &target.to_string();
    let mut n = T::default();
    
    for _ in 0..ITERATIONS {
        n = fun(&stringify);
        assert!(n == target);
    }

    let start = Instant::now();

    for _ in 0..ITERATIONS {
        n = fun(&stringify);
    }

    let elapsed = start.elapsed();

    println!("average time: {:?}", elapsed / ITERATIONS as u32);
    print!("sanity check: ");
    assert!(n == target);
    println!("passed");
}

#[inline]
fn i32_from_str16_sse (s: &String) -> i32 {
    unsafe { asm_i32_from_str16_sse(&s.as_bytes()) }
}

#[inline]
fn i32_from_str16_intrin (s: &String) -> i32 {
    unsafe { rust_io::i32_from_str16_sse(&s.as_bytes()) }
}

#[inline]
fn i32_from_str (s: &String) -> i32 {
    s.parse::<i32>().unwrap()
}

pub fn main () {
    let target: i32 = 1234567890;
    println!("i32_from_str16_see");
    time(&i32_from_str16_sse, target);
    println!("i32_from_str16_intrin");
    time(&i32_from_str16_intrin, target);
    println!("i32_from_str");
    time(&i32_from_str, target);
}