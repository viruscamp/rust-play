use std::{cell::RefCell, ops::{Deref, DerefMut}, sync::{Mutex, MutexGuard}};
use std::rc::Rc;
use lockfree::tls::ThreadLocal;

/// add extention method reentrant_lock for Mutex<T: ?Sized>
///```rust
/// extern crate reentrant_lock;
/// use std::sync::Mutex;
/// use reentrant_lock::{ReentrantMutex, ReentrantMutexGuard};
/// let vec = ReentrantMutex::new(Mutex::new(vec![1,2,3]));
/// while let Some(num) = vec.reentrant_lock().unwrap().pop() {
///    if num == 2 {
///        vec.reentrant_lock().unwrap().push(4);
///    }
///    println!("got {}", num);
/// }
///```
pub struct ReentrantMutex<'a, T> {
    mutex: Mutex<T>,
    guard: ThreadLocal<RefCell<Option<Rc<MutexGuard<'a, T>>>>>,
}

impl<'a, T> Drop for ReentrantMutex<'a, T> {
    fn drop(&mut self) {
        self.guard.clear();
    }
}

impl<'a, T> ReentrantMutex<'a, T> {
    pub fn new(mutex: Mutex<T>) -> Self {
        Self {
            mutex,
            guard: ThreadLocal::new(),
        }
    }
    pub fn reentrant_lock(&'a self) -> Result<ReentrantMutexGuard<'a, T>, ()> {
        if let Ok(mut x) = self.guard.with_default().try_borrow_mut() {
            if let Some(mg) = &*x  {
                Ok(ReentrantMutexGuard::new(&self, mg.clone()))
            } else {
                match self.mutex.lock() {
                    Ok(mg) => {
                        let mg = Rc::new(mg);
                        *x = Some(mg.clone());
                        Ok(ReentrantMutexGuard::new(&self, mg))
                    }
                    Err(e) => {
                        Err(())
                    }
                }
            }
        } else {
            Err(())
        }
    }
}

pub struct ReentrantMutexGuard<'a, T:'a> {
    lock: &'a ReentrantMutex<'a, T>,
    guard: Rc<MutexGuard<'a, T>>,
}

impl<'a, T:'a> Deref for ReentrantMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &**self.guard
    }
}

impl<'a, T:'a> DerefMut for ReentrantMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut **self.guard
    }
}

impl<'a, T: 'a> ReentrantMutexGuard<'a, T> {
    fn new(lock: &'a ReentrantMutex<'a, T>, guard: Rc<MutexGuard<'a, T>>) -> Self {
        ReentrantMutexGuard {
            lock,
            guard,
        }
    }
}

impl<'a, T:'a> Drop for ReentrantMutexGuard<'a, T> {
    fn drop(&mut self) {
        let z = self.lock.guard.with_default();
        if let Ok(mut x) = z.try_borrow_mut() {
            if let Some(mg) = &*x {
                if Rc::strong_count(&mg) == 2 {
                    *x = None;
                }
            }
        }
    }
}
