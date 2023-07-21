use std::{collections::HashSet, sync::Arc};

use strum::Display;
use thiserror::Error;
use tokio::sync::Mutex;

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
    pub async fn unlock(&self) -> IDLockerResult {
        if !self.locker.0.lock().await.remove(&self.locking) {
            return Err(IDLockerError::AlreadyUnlocked);
        };

        Ok(())
    }
}

impl IDLocker {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashSet::new())))
    }

    pub async fn lock(&self, locking: String) -> Result<IDLockGuard, IDLockerError> {
        if !self.0.lock().await.insert(locking.clone()) {
            return Err(IDLockerError::AlreadyLocked)?;
        };

        Ok(IDLockGuard {
            locking,
            locker: self,
        })
    }
}
