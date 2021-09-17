use libc::c_char;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::marker::{PhantomData, Sized};
use std::mem;
use std::sync::atomic::{fence, AtomicUsize};
use std::sync::RwLock;

const BILLION: usize = 1000000000;
const N: usize = 9;
const NR_ENTIES: usize = 4096;

type ExchangeData<T> = Option<(usize, T)>;

pub struct BravoRWlock<T: Default + Copy + ?Sized> {
    rbias: bool,
    underlying: RwLock<T>,
    inhibit_until: u64,
}

impl<T: Default + Copy> BravoRWlock<T> {
    fn new() -> Self {
        let s= RwLock::new();
        Self {
            rbias: false,
            underlying: s,
            inhibit_until: 0,
        }
    }
}

unsafe impl<T: Default + Copy + ?Sized> Sync for BravoRWlock<T> {}
unsafe impl<T: Default + Copy + ?Sized> Send for BravoRWlock<T> {}

#[test]
fn hello_world() {
    println!("Hello, world!");
}
