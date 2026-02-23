use std::ops::Deref;

pub const MAX_HOPS: u32 = 7;

#[derive(Debug, Clone)]
pub struct Hops(u32);

impl Hops {
    pub fn new(hops: u32) -> Result<Self, ()> {
        if hops <= MAX_HOPS {
            Ok(Self(hops))
        } else {
            Err(())
        }
    }

    pub fn inner(self) -> u32 {
        self.0
    }
}

impl Deref for Hops {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
