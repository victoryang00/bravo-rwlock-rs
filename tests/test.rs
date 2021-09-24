use bravo_rwlock_rs::*;
use std::sync::atomic::Ordering::Acquire;
use std::borrow::BorrowMut;

const BILLION: usize = 1000000000;
const N: usize = 9;

#[test]
fn read_lock() {
    env_logger::try_init();
    let mut lock = BravoRWlock::new(1);
    let r = lock.read().and_then(|r| {
        println!("{}", r);
        assert_eq!(*r, 1);
        r.try_sync()
    });
    assert_eq!(r, Ok(()));
}

#[test]
fn write_lock() {
    env_logger::try_init();
    let mut lock = BravoRWlock::new(1);
    let _w = lock.write().and_then(|mut w| {
        *w += 1;
        Ok(())
    });
    assert_eq!(lock.rbias.load(Acquire), true);
    let _r = lock.read().and_then(|r| {
        println!("{}", r);
        assert_eq!(*r, 2);
        r.try_sync()
    });
}

#[test]
#[should_panic]
fn read_while_write() {
    env_logger::try_init();
    let mut lock = BravoRWlock::new(1);
    let _w = lock.write().unwrap();
    // will fail due to its blocked
    let _r = lock.read().unwrap();
}

#[test]
fn lots_thread() {
    env_logger::try_init();
    static mut lock: Option<BravoRWlock<i32>> = None;
    unsafe { lock = Some(BravoRWlock::from(0)) };
    let add_10000 = move || {
        println!("{:?} started!", std::thread::current().id());
        for _i in 0..10000 {
            loop {
                unsafe {
                    match (lock.as_mut()).unwrap().write() {
                        Ok(mut guard) => {
                            *guard += 1;
                            break;
                        }
                        Err(err) => {
                            println!("{:?}: {:?}", std::thread::current().id(), err);
                            continue;
                        }
                    }
                }
            }
        }
        println!("{:?} finished!", std::thread::current().id());
    };
    std::thread::spawn(add_10000);
    std::thread::spawn(add_10000);
    std::thread::spawn(add_10000);
    // should be finished
    std::thread::sleep_ms(5000);
    unsafe {
        let read = lock.as_mut().unwrap().read().unwrap();
        assert_eq!(*read, 30000);
        read.try_sync().unwrap();
    }
}
