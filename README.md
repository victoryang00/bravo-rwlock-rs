# WIP bravo-rwlock
The scalable reader-writer lock is introduced in the paper below
* [BRAVO—Biased Locking for Reader-Writer Locks](https://www.usenix.org/conference/atc19/presentation/dice) (USENIX ATC '19)

## How to use
The current implementation provides the following functions.
They are similar in usage to RWLock.

```rust
use bravo_rwlock_rs::{BravoRWlock,BravoRWlockErrorType};

fn read_txn(lock: &BravoRWlock<i32>)->Result<(), BravoRWlockErrorType>{
    let read_guard = lock.read()?;
    // do your stuff
    println!("status: {}", read_guard);
    println!("\tmy operations: {} + 1 = {}", *read_guard, *read_guard + 1);
    let res = read_guard.try_sync();
    println!("safely synced");
    res
}
```