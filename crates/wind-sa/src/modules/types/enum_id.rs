use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindEnumId(pub(crate) NonZeroU64);
