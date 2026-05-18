use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindTypeId(pub(crate) NonZeroU64);
