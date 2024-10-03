use std::ops::{Add, AddAssign};

#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct M31(u32);

impl M31 {
    const P: u32 = (1 << 31) - 1;
}

impl From<u32> for M31 {
    fn from(value: u32) -> Self {
        Self(value % Self::P)
    }
}
impl From<u64> for M31 {
    fn from(value: u64) -> Self {
        Self((value % (Self::P as u64)) as u32)
    }
}

impl Add for M31 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        ((self.0 as u64) + (rhs.0 as u64)).into()
    }
}
impl AddAssign for M31 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
