use super::common::Vec;

pub type Handful<'alloc, T, const N: usize> = Vec<'alloc, T>;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct VecNURange {
    pub idx: u32,
    pub count: u32,
}

/// Non-uniform lockstep
#[derive(Debug, Clone, serde::Serialize)]
pub struct VecNU<'alloc, T> {
    pub backing: Vec<'alloc, T>,
    pub indices: Vec<'alloc, VecNURange>,
}

pub type LockstepNU<'alloc, T> = Option<VecNU<'alloc, T>>;

