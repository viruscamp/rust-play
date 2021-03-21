use std::{collections::HashMap, sync::{Arc, LockResult, Mutex, MutexGuard}};
use std::rc::Rc;
use lockfree::tls::ThreadLocal;

/*
/// add extention method reentrant_lock for Mutex<T: ?Sized>
///```rust
/// extern crate reentrant_lock;
/// use reentrant_lock::{ReentrantMutex, ReentrantMutexGuard};
/// let vec = Mutex::new(vec![1,2,3]);
/// while let Some(num) = vec.reentrant_lock().unwrap().pop() {
///    if num == 2 {
///        vec.reentrant_lock().unwrap().push(4);
///    }
///    println!("got {}", num);
/// }
///```
pub trait ReentrantMutex<T: ?Sized> {
    fn reentrant_lock(&self) -> LockResult<ReentrantMutexGuard<'_, T>>;
}

impl<T: ?Sized> ReentrantMutex<T> for Mutex<T> {
    fn reentrant_lock(&self) {
        self.lock()
    }
}
*/

pub struct ReentrantMutex<T: ?Sized> {
    tlmg: ThreadLocal<Option<Rc<MutexGuard<'a, T>>>>,
    mutex: Mutex<T>,
}

pub struct ReentrantMutexGuard<'a, T: ?Sized + 'a>(
    Rc<MutexGuard<'a, T>>,
);

impl Drop for ReentrantMutexGuard<'a, T: ?Sized + 'a> {
    fn drop(&mut self) {

    }
}