#![feature(core_intrinsics)]
#![feature(new_uninit)]

use libc::c_char;
use std::cmp::{max, min};
use std::mem;
use std::num::*;
use std::sync::atomic::{fence, AtomicUsize};
use std::sync::{RwLock, RwLockWriteGuard, LockResult};
use std::thread::sleep;

extern crate gettid;

use gettid::gettid;

extern crate coarsetime;

use coarsetime::Instant;
use std::time::Duration;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering::{Release, Relaxed};
use std::fmt::{Debug, Display};
use std::borrow::{BorrowMut, Borrow};

const NR_ENTIES: usize = 4096;

pub fn mix32(mut z: u64) -> u32 {
    z = (z ^ (z >> 33)) * 0xff51afd7ed558ccdu64;
    z = (z ^ (z >> 33)) * 0xc4ceb9fe1a85ec53u64;
    return (z >> 32) as u32;
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

pub struct BravoRWlock<T: Default + ?Sized> {
    rbias: bool,
    underlying: RwLock<T>,
    inhibit_until: u64,
}

// only one instance because the data is locked
// implemented `Deref` and `DerefMut`
// release the lock on drop
pub struct BravoRWlockWriteGuard<'a, T: ?Sized + 'a + Default> {
    lock: &'a BravoRWlock<T>,
}


unsafe impl<T: ?Sized + Sync + Default> Sync for BravoRWlockWriteGuard<'_, T> {}

impl<T: ?Sized + Default> Deref for BravoRWlockWriteGuard<'_, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe {
            &self.lock.underlying.into_inner().unwrap()
        }
    }
}

impl<T: ?Sized + Default> DerefMut for BravoRWlockWriteGuard<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut self.lock.underlying.into_inner().unwrap()
        }
    }
}

impl<T: ?Sized + Default> Drop for BravoRWlockWriteGuard<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {}
}

impl<T: Debug + Default> Debug for BravoRWlockWriteGuard<'_, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BravoRWlockWriteGuard")
            .field("data", self.deref())
            .finish()
    }
}

impl<T: Debug + Display + Default> Display for BravoRWlockWriteGuard<'_, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "BravoRWlockWriteGuard  {}",
            self.deref()
        ))
    }
}

impl<'a, T: ?Sized + Default> BravoRWlockWriteGuard<'a, T> {
    #[inline(always)]
    pub fn new(lock: &'a BravoRWlockWriteGuard<T>) -> Self {
        Self { lock: lock.lock }
    }
}

pub struct BravoRWlockReadGuard<'a, T: ?Sized + 'a + Default> {
    lock: &'a BravoRWlock<T>,
}


impl<T: ?Sized + Default + PartialEq> Default for BravoRWlock<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: ?Sized + Default + PartialEq> PartialEq for BravoRWlock<T> {
    fn eq(&self, other: &Self) -> bool {
        self.underlying.into_inner().borrow().unwrap() == other.underlying.into_inner().borrow().unwrap()
    }

    fn ne(&self, other: &Self) -> bool {
        self.underlying.into_inner().borrow().unwrap() != other.underlying.into_inner().borrow().unwrap()
    }
}

impl<T: Sized + Default + PartialEq> From<T> for BravoRWlock<T> {
    #[inline(always)]
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

unsafe impl<T: Default + ?Sized> Sync for BravoRWlock<T> {}

unsafe impl<T: Default + ?Sized> Send for BravoRWlock<T> {}

fn get_visible_reader<T: ?Sized + Default>() -> [BravoRWlock<T>; 4096] {
    [BravoRWlock { rbias: false, underlying: RwLock::default(), inhibit_until: 0 }; NR_ENTIES]
}

// static VISIBLE_READERS: [BravoRWlock<T>; NR_ENTIES] = [BravoRWlock { rbias: false, underlying: RwLock::new(0), inhibit_until: 0 }; NR_ENTIES];

impl<T: ?Sized + Default + PartialEq> BravoRWlock<T> {
    #[inline(always)]
    pub fn new(t: T) -> Self {
        let s = RwLock::new(t);
        Self {
            rbias: false,
            underlying: s,
            inhibit_until: 0,
        }
    }
    #[inline]
    pub fn hash(&mut self) -> u32 {
        let a: u64 = gettid();
        mix32(a % (NR_ENTIES as u64))
    }
    // make self destroy
    // usually used when the container grows and this pointer point to this structure is replaced
    #[inline]
    pub fn destroy(&self) {
        unimplemented!()
    }

    // try to aquire the lock but only internal use
    #[inline]
    fn try_write(&mut self) -> BravoRWlockResult<u64> {
        self.underlying.borrow_mut().try_write().unwrap();
        if self.rbias {
            self.revocate()
        }
        Ok(0)
    }
    #[inline]
    fn try_read(&mut self) -> BravoRWlockResult<u64> {
        unimplemented!()
    }
    // I suggest you redo the hole function when error occurs
    #[inline]
    pub fn read(&self) -> BravoRWlockResult<BravoRWlockReadGuard<'_, T>> {
        // BravoRWlockReadGuard::new(self)
        unimplemented!()
    }
    // get your RAII write guard
    #[inline]
    pub fn write(&mut self) -> BravoRWlockResult<BravoRWlockWriteGuard<'_, T>> {
        let mut s = self.underlying.borrow_mut().write();
        if self.rbias {
            self.revocate()
        }
        match s {
            Ok(_) => Ok(BravoRWlockWriteGuard { lock: self }),
            Err(_) => Err(BravoRWlockErrorType::RWLockWLockFail),
        }
    }
    #[inline]
    pub fn revocate(&mut self) {
        let ts = Instant::recent();
        self.rbias = false;
        for i in 0..NR_ENTIES {
            while get_visible_reader::<T>()[i].borrow_mut() == self {
                sleep(Duration::from_millis(1));
            }
        };
        self.inhibit_until = ts.elapsed().as_millis();
    }
}

#[test]
fn hello_world() {
    println!("Hello, world!");
}
