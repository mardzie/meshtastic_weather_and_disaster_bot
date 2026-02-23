use std::ops::Deref;

pub const MAX_PAYLOAD_SIZE: usize = 200;

#[derive(Debug, Clone)]
pub struct Payload(String);

impl Payload {
    /// Creates a new `Payload`.
    ///
    /// Fails if the payload `String` is longer than `MAX_PAYLOAD_SIZE`.
    pub fn new(payload: String) -> Result<Self, ()> {
        if payload.len() <= MAX_PAYLOAD_SIZE {
            Ok(Self(payload))
        } else {
            Err(())
        }
    }

    pub fn new_unchecked(payload: String) -> Self {
        Self(payload)
    }

    pub fn inner(self) -> String {
        self.0
    }
}

impl Deref for Payload {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
