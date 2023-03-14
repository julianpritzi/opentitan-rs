//! Synchronization capabilities for the opentitan platform
//!
//! Note: Because opentitan is currently single core & does not support atomic operations
//! synchronization logic only needs to "protect" from interrupts, yet the capabilities are designed
//! to be extendable to multiple cores in the future

use core::ops::{Deref, DerefMut};

pub struct Lock {
    locked: bool,
}

impl Lock {
    pub const fn new() -> Lock {
        Lock { locked: false }
    }

    pub fn try_lock(&mut self) -> Result<(), ()> {
        unsafe {
            riscv::interrupt::disable();
            let res = if !self.locked {
                self.locked = true;
                Ok(())
            } else {
                Err(())
            };
            // TODO: enable once interrupts are supported
            // riscv::interrupt::enable();
            res
        }
    }

    /// Unlocks this lock
    ///
    /// # Safety:
    ///  - only call if currently in possesion of the lock
    pub fn unlock(&mut self) {
        self.locked = false;
    }
}

pub struct Mutex<T> {
    lock: *mut Lock,
    elem: *mut T,
}

impl<T> Mutex<T> {
    pub const fn new(lock: *mut Lock, elem: *mut T) -> Mutex<T> {
        Mutex { lock, elem }
    }

    pub fn try_acquire(&mut self) -> Result<MutexHandle<T>, ()> {
        unsafe {
            if (*self.lock).try_lock().is_ok() {
                Ok(MutexHandle { mutex: self })
            } else {
                Err(())
            }
        }
    }

    /// Releases an acquired ressource
    /// Automatically called by the MutexHandle
    ///
    /// # Safety:
    ///  - only call if ressource is currently acquired by caller
    unsafe fn release(&mut self) {
        (*self.lock).unlock()
    }
}

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

pub struct MutexHandle<'a, T> {
    mutex: &'a mut Mutex<T>,
}

impl<'a, T> Drop for MutexHandle<'a, T> {
    fn drop(&mut self) {
        unsafe { self.mutex.release() }
    }
}

impl<'a, T> Deref for MutexHandle<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.mutex.elem) }
    }
}

impl<'a, T> DerefMut for MutexHandle<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.mutex.elem) }
    }
}
