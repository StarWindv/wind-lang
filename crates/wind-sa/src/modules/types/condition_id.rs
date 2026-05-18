use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConditionId(pub(crate) NonZeroU64);
