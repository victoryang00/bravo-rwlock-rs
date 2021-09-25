#![feature(core_intrinsics)]
#![feature(new_uninit)]
#![feature(thread_id_value)]
#![feature(in_band_lifetimes)]
#![feature(negative_impls)]

use std::sync::atomic::{AtomicBool};
use std::sync::RwLock;
use std::thread::sleep;
use log::info;

extern crate coarsetime;

use coarsetime::Instant;
use std::time::Duration;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering::Relaxed;
use std::fmt::{Debug, Display};
use std::borrow::{BorrowMut, Borrow};
use std::cell::UnsafeCell;
use std::intrinsics::copy_nonoverlapping;


const NR_ENTIES: usize = 4096;


pub fn mix32(mut z: u64) -> u32 {
    info!(" mix32(mut z: u64) -> u32 ");
    z = (z.borrow() ^ (z >> 33)) * 0xff51afd7ed558ccdu64;
    z = (z.borrow() ^ (z >> 33)) * 0xc4ceb9fe1a85ec53u64;
    return (z >> 32) as u32;
}

pub fn bravo_hash() -> u32 {
    info!(" bravo_hash() -> u32 ");
    let a: u64 = std::thread::current().id().as_u64().into();
    mix32(a % (NR_ENTIES as u64))
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

impl<T: Debug + Default> Debug for BravoRWlock<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BravoRWlock")
            .field("data", self.deref())
            .finish()
    }
}

impl<T: Debug + Display + Default> Display for BravoRWlock<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "BravoRWlock  {}",
            self.deref()
        ))
    }
}

impl<T: ?Sized + Default> BravoRWlockWriteGuard<'a, T> {
    #[inline(always)]
    pub fn new(lock: &'a mut BravoRWlockWriteGuard<'a, T>) -> Self {
        info!(" new(lock: &'a mut BravoRWlockWriteGuard<'a, T>) -> Self ");
        Self { lock: &mut lock.lock }
    }
}

impl<T: ?Sized + Default> BravoRWlockWriteGuard<'_, T> {
    pub fn try_sync(self) -> BravoRWlockResult<()> {
        info!(" try_sync(self) -> BravoRWlockResult<()> ");
        if !self.lock.underlying.is_poisoned() {
            drop(self);
            Ok(())
        } else {
            use crate::BravoRWlockErrorType::*;
            Err(RWLockSyncFail)
        }
    }
}

pub struct BravoRWlockReadGuard<'a, T: ?Sized + Default> {
    lock: &'a BravoRWlock<T>,
}

impl<T: ?Sized + Default> BravoRWlockReadGuard<'a, T> {
    #[inline(always)]
    pub fn new(lock: &'a mut BravoRWlockReadGuard<'a, T>) -> Self {
        info!(" new(lock: &'a mut BravoRWlockReadGuard<'a, T>) -> Self ");
        Self { lock: &lock.lock }
    }
}

impl<T: ?Sized + Default> BravoRWlockReadGuard<'_, T> {
    pub fn try_sync(self) -> BravoRWlockResult<()> {
        info!(" try_sync(self) -> BravoRWlockResult<()> ");
        if !self.lock.underlying.is_poisoned() {
            drop(self);
            Ok(())
        } else {
            use crate::BravoRWlockErrorType::*;
            Err(RWLockSyncFail)
        }
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

impl<T: Sized + Default + PartialEq + Debug> Default for BravoRWlock<T> {
    #[inline(always)]
    fn default() -> Self {
        BravoRWlock { rbias: AtomicBool::from(false), underlying: RwLock::default(), inhibit_until: 0, data: UnsafeCell::new(T::default()) }
    }
}


impl<T: ?Sized + Default + PartialEq + Debug> PartialEq for BravoRWlock<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.underlying.read().unwrap() == *other.underlying.read().unwrap()
    }

    fn ne(&self, other: &Self) -> bool {
        *self.underlying.read().unwrap() != *other.underlying.read().unwrap()
    }
}

impl<T: Sized + Default + PartialEq + Debug> From<T> for BravoRWlock<T> {
    #[inline(always)]
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

unsafe impl<T: Default + ?Sized> Sync for BravoRWlock<T> {}

unsafe impl<T: Default + ?Sized> Send for BravoRWlock<T> {}

impl<T: Default + ?Sized> ! Send for BravoRWlockWriteGuard<'_, T> {}

impl<T: Default + ?Sized> ! Send for BravoRWlockReadGuard<'_, T> {}


fn get_visible_reader<T: ?Sized + Default>() -> Vec<BravoRWlock<T>> {
    std::iter::repeat_with(|| BravoRWlock { rbias: AtomicBool::from(false), underlying: RwLock::default(), inhibit_until: 0, data: UnsafeCell::new(T::default()) }).take(NR_ENTIES).collect()
}
// static VISIBLE_READERS: [BravoRWlock<T>; NR_ENTIES] = [BravoRWlock { rbias: false, underlying: RwLock::new(0), inhibit_until: 0 }; NR_ENTIES];


impl<T: ?Sized + Default + PartialEq + Debug> BravoRWlock<T> {
    #[inline(always)]
    pub fn new(mut t: T) -> Self {
        info!(" new(mut t: T) -> Self ");
        let u = UnsafeCell::new(T::default());
        unsafe { copy_nonoverlapping(t.borrow_mut(), u.get(), 1) }
        Self {
            rbias: AtomicBool::from(false),
            data: u,
            underlying: RwLock::new(t),
            inhibit_until: 0,
        }
    }


    // try to aquire the lock but only internal use
    #[inline]
    fn try_write(&mut self) -> BravoRWlockResult<BravoRWlockWriteGuard<T>> {
        self.underlying.borrow_mut().try_write().unwrap();
        if self.rbias.load(Relaxed) {
            self.revocate()
        }
        Ok(BravoRWlockWriteGuard { lock: self })
    }
    // get your RAII write guard
    #[inline]
    pub fn write(&mut self) -> BravoRWlockResult<BravoRWlockWriteGuard<T>> {
        info!(" write(&mut self) -> BravoRWlockResult<BravoRWlockWriteGuard<T>> ");
        self.underlying.borrow_mut().write().unwrap();
        if self.rbias.load(Relaxed) {
            self.revocate()
        }
        for i in 0..4096 {
            dbg!(get_visible_reader::<T>()[i].inhibit_until);
        }
        Ok(BravoRWlockWriteGuard { lock: self })
    }
    #[inline]
    pub fn try_read(&self) -> BravoRWlockResult<BravoRWlockReadGuard<T>> {
        info!(" try_read(&self) -> BravoRWlockResult<BravoRWlockReadGuard<T>> ");
        let mut slot;
        if self.rbias.load(Relaxed) {
            slot = bravo_hash();
            if unsafe {
                std::intrinsics::atomic_cxchg(&mut (get_visible_reader::<T>()[slot as usize].inhibit_until)
                                              , BravoRWlock::<T>::default().inhibit_until, self.inhibit_until).1
            } {
                if self.rbias.load(Relaxed) {
                    return Ok(BravoRWlockReadGuard { lock: self });
                }
                get_visible_reader::<T>()[slot as usize] = BravoRWlock::default();
            }
        }
        self.underlying.try_read().unwrap();
        let ts = Instant::recent().as_u64();
        if self.rbias.load(Relaxed) && ts >= self.inhibit_until {
            self.rbias.store(true, Relaxed)
        }
        Ok(BravoRWlockReadGuard { lock: self })
    }

    // I suggest you redo the whole function when error occurs
    #[inline]
    pub fn read(&self) -> BravoRWlockResult<BravoRWlockReadGuard<T>> {
        info!(" read(&self) -> BravoRWlockResult<BravoRWlockReadGuard<T>> ");
        let mut slot;
        if self.rbias.load(Relaxed) {
            slot = bravo_hash();
            if unsafe {
                std::intrinsics::atomic_cxchg(&mut (get_visible_reader::<T>()[slot as usize].inhibit_until)
                                              , BravoRWlock::<T>::default().inhibit_until, self.inhibit_until).1
            } {
                if self.rbias.load(Relaxed) {
                    return Ok(BravoRWlockReadGuard { lock: self });
                }
                get_visible_reader::<T>()[slot as usize] = BravoRWlock::default();
            }
        }
        self.underlying.read().unwrap();
        let ts = Instant::recent().as_u64();
        if self.rbias.load(Relaxed) && ts >= self.inhibit_until {
            self.rbias.store(true, Relaxed)
        }
        Ok(BravoRWlockReadGuard { lock: self })
    }

    #[inline]
    pub fn revocate(&mut self) {
        info!(" revocate(&mut self) ");
        self.rbias.store(false, Relaxed);
        for i in 0..NR_ENTIES {
            while get_visible_reader::<T>()[i].borrow_mut() == self {
                sleep(Duration::from_millis(1));
            }
        };
        self.inhibit_until = Instant::recent().as_u64();
    }
    pub fn get_mut(&mut self) -> BravoRWlockResult<&mut T> {
        info!(" get_mut(&mut self) -> BravoRWlockResult<&mut T> ");
        let data = self.data.get_mut();
        Ok(data)
    }
}

