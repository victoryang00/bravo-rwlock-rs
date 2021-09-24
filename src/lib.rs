#![feature(core_intrinsics)]
#![feature(new_uninit)]
#![feature(thread_id_value)]
#![feature(in_band_lifetimes)]

use std::sync::atomic::{fence, AtomicUsize, AtomicBool};
use std::sync::{RwLock, RwLockWriteGuard, LockResult};
use std::thread::sleep;
use lazy_static::lazy_static;

extern crate coarsetime;

use coarsetime::Instant;
use std::time::Duration;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering::{Release, Relaxed};
use std::fmt::{Debug, Display};
use std::borrow::{BorrowMut, Borrow};
use std::marker::PhantomData;
use std::cell::UnsafeCell;
use std::intrinsics::copy_nonoverlapping;

const NR_ENTIES: usize = 4096;

pub fn mix32(mut z: u64) -> u32 {
    z = (z.borrow() ^ (z >> 33)) * 0xff51afd7ed558ccdu64;
    z = (z.borrow() ^ (z >> 33)) * 0xc4ceb9fe1a85ec53u64;
    return (z >> 32) as u32;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BravoRWlockErrorType {
    RWLockInitFail,
    RWLockRLockFail,
    RWLockWLockFail,
    RWLockSyncFail,
    RuntimeFail,
}

type BravoRWlockResult<T> = Result<T, BravoRWlockErrorType>;

type ExchangeData<T> = Option<(usize, T)>;

pub struct BravoRWlock<T: Default + ?Sized> {
    pub rbias: AtomicBool,
    underlying: RwLock<T>,
    inhibit_until: u64,
    data: UnsafeCell<T>,
}

// only one instance because the data is locked
// implemented `Deref` and `DerefMut`
// release the lock on drop
pub struct BravoRWlockWriteGuard<'a, T: ?Sized + Default> {
    lock: &'a mut BravoRWlock<T>,
}


unsafe impl<T: ?Sized + Sync + Default> Sync for BravoRWlockWriteGuard<'_, T> {}

impl<T: ?Sized + Default> Deref for BravoRWlockWriteGuard<'_, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized + Default> DerefMut for BravoRWlockWriteGuard<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}
impl<T: ?Sized + Default> DerefMut for BravoRWlockReadGuard<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}
//
// impl<T: ?Sized + Default> Drop for BravoRWlockWriteGuard<'_, T> {
//     #[inline(always)]
//     fn drop(&mut self) {}
// }

impl<T: Debug + Default> Debug for BravoRWlockWriteGuard<'_, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BravoRWlockWriteGuard")
            .field("data", self.deref())
            .field("poison", self.deref())
            .field("marker", self.deref())
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

impl<T: ?Sized + Default> BravoRWlockWriteGuard<'a, T> {
    #[inline(always)]
    pub fn new(lock: &'a mut BravoRWlockWriteGuard<'a, T>) -> Self {
        Self { lock: &mut lock.lock }
    }
}

impl<T: ?Sized + Default> BravoRWlockWriteGuard<'_, T> {
    pub fn try_sync(self) -> BravoRWlockResult<()> {
        Ok(()) // what time to get ready
    }
}

pub struct BravoRWlockReadGuard<'a, T: ?Sized + Default> {
    lock: &'a mut BravoRWlock<T>,
}

impl<T: ?Sized + Default> BravoRWlockReadGuard<'a, T> {
    #[inline(always)]
    pub fn new(lock: &'a mut BravoRWlockReadGuard<'a, T>) -> Self {
        Self { lock: &mut lock.lock }
    }
}

impl<T: ?Sized + Default> BravoRWlockReadGuard<'_, T> {
    pub fn try_sync(self) -> BravoRWlockResult<()> {
        Ok(()) // what time to get ready
        // if self.lock.try_lock()? {
        //     drop(self);
        //     Ok(())
        // } else {
        //     use crate::BravoRWlockErrorType::*;
        //     Err(RWLockSyncFail)
        // }
    }
}

impl<T: ?Sized + Default> Deref for BravoRWlockReadGuard<'_, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}


impl<T: Debug + Default> Debug for BravoRWlockReadGuard<'_, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BravoRWlockReadGuard")
            .field("data", self.deref())
            .finish()
    }
}

impl<T: Debug + Display + Default> Display for BravoRWlockReadGuard<'_, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "BravoRWlockReadGuard  {}",
            self.deref()
        ))
    }
}

impl<T: ?Sized + Default + PartialEq> Default for BravoRWlock<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: ?Sized + Default + PartialEq> PartialEq for BravoRWlock<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.underlying.read().unwrap() == *other.underlying.read().unwrap()
    }

    fn ne(&self, other: &Self) -> bool {
        *self.underlying.read().unwrap() != *other.underlying.read().unwrap()
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

fn get_visible_reader<T: ?Sized + Default>() -> Vec<BravoRWlock<T>> {
    std::iter::repeat_with(|| BravoRWlock { rbias: AtomicBool::from(false), underlying: RwLock::default(), inhibit_until: 0, data: UnsafeCell::new(T::default()) }).take(NR_ENTIES).collect()
}
// static VISIBLE_READERS: [BravoRWlock<T>; NR_ENTIES] = [BravoRWlock { rbias: false, underlying: RwLock::new(0), inhibit_until: 0 }; NR_ENTIES];

impl<T: ?Sized + Default + PartialEq> BravoRWlock<T> {
    #[inline(always)]
    pub fn new(mut t: T) -> Self {
        let mut u = UnsafeCell::new(T::default());
        unsafe { copy_nonoverlapping(t.borrow_mut(), u.get(), 1) }
        Self {
            rbias: AtomicBool::from(false),
            data: u,
            underlying: RwLock::new(t),
            inhibit_until: 0,
        }
    }
    #[inline]
    pub fn hash(&mut self) -> u32 {
        let a: u64 = std::thread::current().id().as_u64().into();
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
        if self.rbias.load(Relaxed) {
            self.revocate()
        }
        Ok(0)
    }
    // get your RAII write guard
    #[inline]
    pub fn write(&mut self) -> BravoRWlockResult<BravoRWlockWriteGuard<T>> {
        unimplemented!()
    }
    #[inline]
    fn try_read(&mut self) -> BravoRWlockResult<u64> {
        unimplemented!()
    }
    // I suggest you redo the hole function when error occurs
    #[inline]
    pub fn read(&mut self) -> BravoRWlockResult<BravoRWlockReadGuard<T>> {
        // BravoRWlockReadGuard::new(self)
        unimplemented!()
    }

    #[inline]
    pub fn revocate(&mut self) {
        let ts = Instant::recent();
        self.rbias.store(false, Relaxed);
        for i in 0..NR_ENTIES {
            while get_visible_reader::<T>()[i].borrow_mut() == self {
                sleep(Duration::from_millis(1));
            }
        };
        self.inhibit_until = ts.elapsed().as_millis();
    }
    pub fn get_mut(&mut self) -> BravoRWlockResult<&mut T>{
            let data = self.data.get_mut();
            Ok(data)
    }
}

#[test]
fn hello_world() {
    println!("Hello, world!");
}
