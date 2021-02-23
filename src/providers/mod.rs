use std::sync::RwLockReadGuard;
use std::error::Error;

use crate::cache::Cache;

pub trait Provider {
    fn read_cache(&self) -> Result<RwLockReadGuard<Cache>, Box<dyn Error>>;
    fn join(self);
}

mod fs;
pub use fs::*;
