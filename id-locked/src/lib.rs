use std::{collections::HashSet, sync::Arc};

use strum::Display;
use thiserror::Error;
use tokio::sync::Mutex;

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
    pub async fn unlock(self) -> IDLockerResult {
        self.locker
            .0
            .lock()
            .await
            .remove(&self.locking)
            .then_some(())
            .ok_or(IDLockerError::AlreadyUnlocked)
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

    pub async fn lock(&self, locking: String) -> Result<IDLockGuard, IDLockerError> {
        self.0
            .lock()
            .await
            .insert(locking.clone())
            .then_some(IDLockGuard {
                locking,
                locker: self,
            })
            .ok_or(IDLockerError::AlreadyLocked)
    }
}
