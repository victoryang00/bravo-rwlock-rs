use criterion::criterion_main;
use criterion::{criterion_group, BenchmarkId, Criterion, Fun, ParameterizedBenchmark};

use std::{
    collections::binary_heap::Iter,
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
    thread::spawn,
};

use bravo_rwlock_rs::{BravoRWlock, BravoRWlockWriteGuard};
use std::borrow::{Borrow, BorrowMut};

// start 0:0:0
// end 0:i:i
struct ListRwLock {
    head: i32,
    tail: Option<Arc<RwLock<ListRwLock>>>,
}

impl ListRwLock {
    fn new() -> Self {
        let node3 = Some(Arc::new(RwLock::new(ListRwLock {
            head: 0,
            tail: None,
        })));
        let node2 = Some(Arc::new(RwLock::new(ListRwLock {
            head: 0,
            tail: node3,
        })));
        ListRwLock {
            head: 0,
            tail: node2,
        }
    }
    // simulate select * from db;
    fn get_all(&mut self) -> (i32, i32, i32) {
        let h1 = self.head;
        let t1 = self.tail.as_ref().unwrap().as_ref().read().unwrap();
        let h2 = t1.head;
        let t2 = t1.tail.as_ref().unwrap().as_ref().read().unwrap();
        let h3 = t2.head;
        (h1, h2, h3)
    }
    // simutale select * from db where id = 2
    fn get2(&mut self) -> i32 {
        let t1 = self.tail.as_ref().unwrap().as_ref().read().unwrap();
        let h2 = t1.head;
        h2
    }
    // simulate update id = 2
    fn set2(&mut self) {
        let mut t1 = self.tail.as_ref().unwrap().as_ref().write().unwrap();
        t1.head += 1;
    }
    // simulate update id = 3
    fn set3(&mut self) {
        let t1 = self.tail.as_ref().unwrap().as_ref().read().unwrap();
        let mut t2 = t1.tail.as_ref().unwrap().as_ref().write().unwrap();
        t2.head += 1;
    }
}

// start 0:0:0
// end 0:i:i
struct ListMutex {
    head: i32,
    tail: Option<Arc<Mutex<ListMutex>>>,
}

impl Default for ListMutex {
    fn default() -> Self {
        Self::new()
    }
}

impl ListMutex {
    fn new() -> Self {
        let node3 = Some(Arc::new(Mutex::new(ListMutex {
            head: 0,
            tail: None,
        })));
        let node2 = Some(Arc::new(Mutex::new(ListMutex {
            head: 0,
            tail: node3,
        })));
        ListMutex {
            head: 0,
            tail: node2,
        }
    }
    // simulate select * from db;
    fn get_all(&mut self) -> (i32, i32, i32) {
        let h1 = self.head;
        let t1 = self.tail.as_ref().unwrap().as_ref().lock().unwrap();
        let h2 = t1.head;
        let t2 = t1.tail.as_ref().unwrap().as_ref().lock().unwrap();
        let h3 = t2.head;
        (h1, h2, h3)
    }
    // simutale select * from db where id = 2
    fn get2(&mut self) -> i32 {
        let t1 = self.tail.as_ref().unwrap().as_ref().lock().unwrap();
        let h2 = t1.head;
        h2
    }
    // simulate update id = 2
    fn set2(&mut self) {
        let mut t1 = self.tail.as_ref().unwrap().as_ref().lock().unwrap();
        t1.head += 1;
    }
    // simulate update id = 3
    fn set3(&mut self) {
        let t1 = self.tail.as_ref().unwrap().as_ref().lock().unwrap();
        let mut t2 = t1.tail.as_ref().unwrap().as_ref().lock().unwrap();
        t2.head += 1;
    }
}

struct ListOLock {
    head: i32,
    tail: Option<Arc<BravoRWlock<ListOLock>>>,
}

impl Default for ListOLock {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq<Self> for ListOLock {
    fn eq(&self, other: &Self) -> bool {
        self.head == other.head
    }
}


impl ListOLock {
    fn new() -> Self {
        let node3 = Some(Arc::new(BravoRWlock::new(ListOLock {
            head: 0,
            tail: None,
        })));
        let node2 = Some(Arc::new(BravoRWlock::new(ListOLock {
            head: 0,
            tail: node3,
        })));
        ListOLock {
            head: 0,
            tail: node2,
        }
    }
    // simulate select * from db;
    fn get_all(&self) -> (i32, i32, i32) {
        loop {
            let h1 = self.head;
            // let t1 = self.tail.borrow_mut().as_mut().unwrap().as_ref().read();
            let sb = Arc::into_raw(self.tail.clone().unwrap());

            match unsafe { sb.read().read() } {
                Ok(t1) => {
                    let h2 = t1.head;
                    // let t2 = t1.tail.as_mut().unwrap().as_ref().read();
                    let sb = Arc::into_raw(t1.tail.clone().unwrap());
                    match unsafe { sb.read().read() } {
                        Ok(t2) => {
                            let h3 = t2.head;
                            match t2.try_sync() {
                                Ok(_) => {}
                                Err(_) => {
                                    continue;
                                }
                            }
                            match t1.try_sync() {
                                Ok(_) => {
                                    return (h1, h2, h3);
                                }
                                Err(_) => {
                                    continue;
                                }
                            }
                        }
                        Err(_) => {
                            continue;
                        }
                    };
                }
                Err(_) => {
                    continue;
                }
            }
        }
    }
    // simutale select * from db where id = 2
    fn get2(&mut self) -> i32 {
        loop {
            let sb = Arc::into_raw(self.tail.clone().unwrap());
            match unsafe { sb.read().read() } {
                Ok(t1) => {
                    let h2 = t1.head;
                    match t1.try_sync() {
                        Ok(_) => return h2,
                        Err(_) => {
                            continue;
                        }
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
    }
    // simulate update id = 2
    fn set2(&mut self) {
        let sb = Arc::into_raw(self.tail.clone().unwrap());
        unsafe { sb.read().write().unwrap().head += 1; }
    }
    // simulate update id = 3
    fn set3(&mut self) {
        loop {
            // let t1 = self.tail.as_mut().unwrap().as_ref().read();
            let sb = Arc::into_raw(self.tail.clone().unwrap());

            match unsafe { sb.read().read() } {
                Ok(t1) => {
                    let sb = Arc::into_raw(t1.tail.clone().unwrap());
                    unsafe { sb.read().write().unwrap().head += 1; }
                    match t1.try_sync() {
                        Ok(_) => {
                            return;
                        }
                        Err(_) => {
                            continue;
                        }
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
    }
}


fn heavy_read_mutex(i: i32) {
    static mut lock: Option<Mutex<i32>> = None;
    unsafe { lock = Some(Mutex::from(0)) };

    let write_fn = move || unsafe {
        for _i in 0..i {
            std::thread::sleep_ms(10);
            match lock.as_ref().unwrap().lock() {
                Ok(mut guard) => {
                    *guard += 1;
                }
                Err(_poison) => {
                    panic!(" fuck me! ")
                }
            }
        }
    };
    let read_fn = move || unsafe {
        for _i in 0..i {
            std::thread::sleep_ms(8);
            match lock.as_ref().unwrap().lock() {
                Ok(guard) => {
                    let _ = *guard + 1;
                }
                Err(_poison) => {
                    panic!(" fuck me! ")
                }
            }
        }
    };
    let thread1 = spawn(write_fn);
    let thread2 = spawn(read_fn);
    let thread3 = spawn(read_fn);

    let _ = thread1.join();
    let _ = thread2.join();
    let _ = thread3.join();
}

fn heavy_read_rwlock(i: i32) {
    static mut lock: Option<RwLock<i32>> = None;
    unsafe { lock = Some(RwLock::from(0)) };

    let write_fn = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(10);
            match lock.as_ref().unwrap().write() {
                Ok(mut guard) => {
                    *guard += 1;
                }
                Err(_poison) => {
                    panic!(" fuck me! ")
                }
            }
        }
    };
    let read_fn = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(8);
            match lock.as_ref().unwrap().read() {
                Ok(guard) => {
                    let _ = *guard + 1;
                }
                Err(_poison) => {
                    panic!(" fuck me! ")
                }
            }
        }
    };
    let thread1 = spawn(write_fn);
    let thread2 = spawn(read_fn);
    let thread3 = spawn(read_fn);

    let _ = thread1.join();
    let _ = thread2.join();
    let _ = thread3.join();
    unsafe { assert_eq!(*(lock.as_mut().unwrap().write().unwrap().deref()), i) }
}

fn heavy_read_optimistic_lock_coupling(i: i32) {
    static mut lock: Option<BravoRWlock<i32>> = None;
    unsafe { lock = Some(BravoRWlock::from(0)) };
    let write_fn = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(10);
            loop {
                match lock.as_mut().unwrap().write() {
                    Ok(mut guard) => {
                        *guard += 1;
                        break;
                    }
                    Err(_err) => {
                        continue;
                    }
                }
            }
        }
    };
    let read_fn = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(8);
            loop {
                let g = lock.as_mut().unwrap();
                match g.read() {
                    Ok(guard) => {
                        let _ = *guard + 1;
                        match guard.try_sync() {
                            Ok(_) => {
                                break;
                            }
                            Err(_) => {
                                continue;
                            }
                        }
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        }
    };
    use std::thread::spawn;
    let thread1 = spawn(write_fn);
    let thread2 = spawn(read_fn);
    let thread3 = spawn(read_fn);

    let _ = thread1.join();
    let _ = thread2.join();
    let _ = thread3.join();
    unsafe { assert_eq!(*(lock.as_mut().unwrap().write().unwrap().deref()), i) }
}

fn heavy_read_list_rwlock(i: i32) {
    static mut lock: Option<ListRwLock> = None;
    unsafe { lock = Some(ListRwLock::new()) };
    let write_fn = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(10);
            lock.as_mut().unwrap().set2();
            // std::thread::sleep_ms(12);
            lock.as_mut().unwrap().set3();
        }
    };
    let read_fn1 = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(8);
            lock.as_mut().unwrap().get_all();
        }
    };
    let read_fn2 = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(8);
            lock.as_mut().unwrap().get2();
        }
    };
    use std::thread::spawn;
    let thread1 = spawn(write_fn);
    let thread2 = spawn(read_fn1);
    let thread3 = spawn(read_fn2);

    let _ = thread1.join();
    let _ = thread2.join();
    let _ = thread3.join();
    unsafe { assert_eq!(lock.as_mut().unwrap().get2(), i) }
}

fn heavy_read_list_mutex(i: i32) {
    static mut lock: Option<ListMutex> = None;
    unsafe { lock = Some(ListMutex::new()) };
    let write_fn = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(10);
            lock.as_mut().unwrap().set2();
            // std::thread::sleep_ms(12);
            lock.as_mut().unwrap().set3();
        }
    };
    let read_fn1 = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(8);
            lock.as_mut().unwrap().get_all();
        }
    };
    let read_fn2 = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(8);
            lock.as_mut().unwrap().get2();
        }
    };
    use std::thread::spawn;
    let thread1 = spawn(write_fn);
    let thread2 = spawn(read_fn1);
    let thread3 = spawn(read_fn2);

    let _ = thread1.join();
    let _ = thread2.join();
    let _ = thread3.join();
    unsafe { assert_eq!(lock.as_mut().unwrap().get2(), i) }
}

fn heavy_read_list_optimistic_lock_coupling(i: i32) {
    static mut lock: Option<ListOLock> = None;
    unsafe { lock = Some(ListOLock::new()) };
    let write_fn = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(10);
            lock.as_mut().unwrap().set2();
            // std::thread::sleep_ms(12);
            lock.as_mut().unwrap().set3();
        }
    };
    let read_fn1 = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(8);
            lock.as_mut().unwrap().get_all();
        }
    };
    let read_fn2 = move || unsafe {
        for _i in 0..i {
            // std::thread::sleep_ms(8);
            lock.as_mut().unwrap().get2();
        }
    };
    use std::thread::spawn;
    let thread1 = spawn(write_fn);
    let thread2 = spawn(read_fn1);
    let thread3 = spawn(read_fn2);

    let _ = thread1.join();
    let _ = thread2.join();
    let _ = thread3.join();
    unsafe { assert_eq!(lock.as_mut().unwrap().get2(), i) }
}

fn lock_heavy_read_i32(c: &mut Criterion) {
    // let mx = Fun::new("Mutex", |b, i| b.iter(|| heavy_read_mutex(*i)));
    let rw = Fun::new("RwLock", |b, i| b.iter(|| heavy_read_rwlock(*i)));
    let ol = Fun::new("BravoRWlock", |b, i| {
        b.iter(|| heavy_read_optimistic_lock_coupling(*i))
    });

    let functions = vec![ol, rw];

    c.bench_functions("Lock Heavy Read I32 Compare", functions, 100000);
}

fn lock_heavy_read_list(c: &mut Criterion) {
    let mx = Fun::new("Mutex", |b, i| b.iter(|| heavy_read_list_mutex(*i)));
    let rw = Fun::new("RwLock", |b, i| b.iter(|| heavy_read_list_rwlock(*i)));
    let ol = Fun::new("BravoRWlock", |b, i| {
        b.iter(|| heavy_read_list_optimistic_lock_coupling(*i))
    });

    let functions = vec![ol, rw, mx];

    c.bench_functions("Lock Heavy Read List Compare", functions, 100000);
}

criterion_group!(name = lock_heavy_read; config = Criterion::default().sample_size(100); targets = lock_heavy_read_list);


criterion_main! {
   lock_heavy_read
}

