use std::{collections::HashSet, sync::Arc};

use parking_lot::Mutex;
use strum::Display;
use thiserror::Error;

pub struct IDLocker(Arc<Mutex<HashSet<String>>>);

#[derive(Error, Debug, Display)]
pub enum IDLockerError {
    AlreadyLocked,
    AlreadyUnlocked,
}

pub type IDLockerResult = Result<(), IDLockerError>;

pub struct IDLockGuard<'a> {
    locker: &'a IDLocker,
    locking: String,
}

impl IDLockGuard<'_> {
    pub fn unlock(&self) -> IDLockerResult {
        if !self.locker.0.lock().remove(&self.locking) {
            return Err(IDLockerError::AlreadyUnlocked);
        };

        Ok(())
    }
}

impl<'a> Drop for IDLockGuard<'a> {
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}

impl IDLocker {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashSet::new())))
    }

    pub fn lock(&self, locking: String) -> Result<IDLockGuard, IDLockerError> {
        if !self.0.lock().insert(locking.clone()) {
            return Err(IDLockerError::AlreadyLocked)?;
        };

        Ok(IDLockGuard {
            locking,
            locker: self,
        })
    }
}
