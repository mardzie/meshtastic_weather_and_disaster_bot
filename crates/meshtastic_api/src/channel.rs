use std::{num::NonZero, ops::Deref};

const DEFAULT_CHANNEL: u8 = 0;
const MAX_CHANNEL: u8 = 7;

#[derive(Debug, Default, Clone, Copy)]
pub enum Channel {
    #[default]
    Primary,
    Other(NonZero<u8>),
}

impl Channel {
    pub fn into_channel(&self) -> u8 {
        match self {
            Self::Primary => DEFAULT_CHANNEL,
            Self::Other(channel) => channel.get(),
        }
    }
}

impl From<u32> for Channel {
    fn from(channel: u32) -> Self {
        if channel == DEFAULT_CHANNEL as u32 {
            Self::Primary
        } else if channel <= 7 {
            Self::Other(
                NonZero::new(channel as u8).expect(
                    "Fatal Error: Check code first and after that contact Rust compiler team.",
                ),
            )
        } else {
            panic!(
                "Channel cant be greater than {}: Found {}",
                MAX_CHANNEL, channel
            );
        }
    }
}

impl From<Channel> for meshtastic::types::MeshChannel {
    fn from(channel: Channel) -> Self {
        Self::new(channel.into_channel() as u32).expect("`Channel` must be in range 0..7")
    }
}
