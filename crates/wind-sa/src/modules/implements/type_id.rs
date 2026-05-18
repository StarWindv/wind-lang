use std::num::NonZeroU64;
use crate::modules::types::WindTypeId;

impl WindTypeId {
    pub fn new(id: u64) -> Self {
        Self(NonZeroU64::new(id).expect("TypeId must be non-zero"))
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}
