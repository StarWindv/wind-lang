use std::num::NonZeroU64;
use crate::modules::types::WindValueID;

impl WindValueID {
    pub fn new(id: u64) -> Self {
        Self(NonZeroU64::new(id).expect("ValueId must be non-zero"))
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}
