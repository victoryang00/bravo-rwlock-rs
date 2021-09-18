#![feature(core_intrinsics)]

use libc::c_char;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::marker::{PhantomData, Sized};
use std::mem;
use std::sync::atomic::{fence, AtomicUsize};
use std::sync::RwLock;

pub fn iff_likely<T>(cond: bool, x: T) -> T {
    if unsafe { std::intrinsics::likely(cond) } { x } else {}
}

pub fn iff_unlikely<T>(cond: bool, x: T) -> T {
    if unsafe { std::intrinsics::unlikely(cond) } { x } else {}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BravoRWlockErrorType {
    RWLockInitFail,
    RWLockRLockFail,
    RWLockWLockFail,
    RuntimeFail,
}

type BravoRWlockResult<T> = Result<T, BravoRWlockErrorType>;

type ExchangeData<T> = Option<(usize, T)>;

pub struct BravoRWlock<T: Default + Copy + ?Sized> {
    rbias: bool,
    underlying: RwLock<T>,
    inhibit_until: u64,
}

// only one instance because the data is locked
// implemented `Deref` and `DerefMut`
// release the lock on drop
pub struct BravoRWlockWriteGuard<'a, T: ?Sized + 'a> {
    lock: &'a BravoRWLock<T>,
}

pub struct BravoRWlockReadGuard<'a, T: ?Sized + 'a> {
    lock: &'a BravoRWLock<T>,
}

impl<T: Default + Copy> BravoRWlock<T> {
    #[inline(always)]
    fn new(t: T) -> Self {
        let s = RwLock::new(T);
        Self {
            rbias: false,
            underlying: s,
            inhibit_until: 0,
        }
    }
}

unsafe impl<T: Default + Copy + ?Sized> Sync for BravoRWlock<T> {}

unsafe impl<T: Default + Copy + ?Sized> Send for BravoRWlock<T> {}

impl<T: Sized> From<T> for BravoRWlock<T> {
    #[inline(always)]
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

impl<T: ?Sized> BravoRWlock<T> {
    // make self destroy
    // usually used when the container grows and this pointer point to this structure is replaced
    #[inline]
    pub fn destroy(&self) {}

    // try to aquire the lock but only internal use
    #[inline]
    fn try_lock(&self) -> BravoRWlockResult<u64> {
        unimplemented!()
    }
    // I suggest you redo the hole function when error occurs
    #[inline]
    pub fn read(&self) -> BravoRWlockResult<BravoRWlockReadGuard<'_, T>> {
        BravoRWlockReadGuard::new(self)
    }
    // get your RAII write guard
    #[inline]
    pub fn write(&mut self) -> BravoRWlockResult<BravoRWlockWriteGuard<'_, T>> {
        let mut s = self.underlying.write().unwrap();
        iff_unlikely(s != 0,  Err(BravoRWlockErrorType::RWLockWLockFail); )
        if self.rbias!=0{
            self.revocate();
        }
    }
    // #[inline]
    // pub fn write(&mut self) -> BravoRWlockResult<BravoRWlockWriteGuard<'_, T>> {
    //     let mut s = self.underlying.write().unwrap();
    //     iff_unlikely(s != 0,  Err(BravoRWlockErrorType::RWLockWLockFail); )
    //     if self.rbias!=0{
    //         self.revocate();
    //     }
    // }
    #[inline]
    pub fn revocate(&self) -> BravoRWlockResult<BravoRWlockWriteGuard<'_, T>>{

    }
}

#[test]
fn hello_world() {
    println!("Hello, world!");
}
