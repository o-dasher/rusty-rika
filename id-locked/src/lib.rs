use std::{collections::HashSet, sync::Arc};

use parking_lot::Mutex;
use strum::Display;
use thiserror::Error;

#[derive(Debug)]
pub struct IDLocker(Arc<Mutex<HashSet<String>>>);

#[derive(Error, Debug, Display)]
pub enum IDLockerError {
    AlreadyLocked,
    AlreadyUnlocked,
}

pub type IDLockerResult = Result<(), IDLockerError>;

#[derive(Debug)]
pub struct IDLockGuard<'a> {
    locker: &'a IDLocker,
    locking: String,
}

impl IDLockGuard<'_> {
    pub fn unlock(&self) -> IDLockerResult {
        self.locker
            .0
            .lock()
            .remove(&self.locking)
            .then_some(())
            .ok_or(IDLockerError::AlreadyUnlocked)
    }
}

impl<'a> Drop for IDLockGuard<'a> {
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}

impl Default for IDLocker {
    fn default() -> Self {
        Self::new()
    }
}

impl IDLocker {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashSet::new())))
    }

    pub fn lock(&self, locking: String) -> Result<IDLockGuard, IDLockerError> {
        self.0
            .lock()
            .insert(locking.clone())
            .then_some(IDLockGuard {
                locking,
                locker: self,
            })
            .ok_or(IDLockerError::AlreadyLocked)
    }
}
