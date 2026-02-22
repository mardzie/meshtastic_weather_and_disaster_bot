#[derive(Debug, Clone)]
pub struct Packet {
    /// Source
    pub from: u32,
    /// Target
    pub to: Target,

    /// The local channel.
    pub channel: u32,
    /// Unique Packet ID.
    pub id: u32,
    pub via_mqtt: bool,

    /// Hop limit with which the packet started.
    pub hop_start: u8,
    /// Current hop limit.
    ///
    /// At 0 this is the last receiving node. At 1 this packet can be resend one time etc.
    pub hop_limit: u8,
    /// The already traveled hops. (hop_start - hop_limit)
    pub hops_traveled: u8,

    pub payload: String,
}

#[derive(Debug, Clone)]
pub enum Target {
    PrimaryChannel,
    NodeId(u32),
}

impl Packet {
    pub fn new(
        mesh_packet: &meshtastic::protobufs::MeshPacket,
        data: &meshtastic::protobufs::Data,
    ) -> Self {
        Self {
            from: mesh_packet.from,
            to: Target::from(mesh_packet.to),

            channel: mesh_packet.channel,
            id: mesh_packet.id,
            via_mqtt: mesh_packet.via_mqtt,

            hop_start: mesh_packet.hop_start as u8,
            hop_limit: mesh_packet.hop_limit as u8,
            hops_traveled: mesh_packet.hop_start as u8 - mesh_packet.hop_limit as u8,

            payload: String::from_utf8_lossy(&data.payload).to_string(),
        }
    }
}

impl Target {
    pub const PRIMARY_CHANNEL_ID: u32 = 0xFFFFFFFF;

    pub fn into_id(&self) -> u32 {
        match self {
            Self::PrimaryChannel => Self::PRIMARY_CHANNEL_ID,
            Self::NodeId(id) => id.clone(),
        }
    }
}

impl From<u32> for Target {
    fn from(id: u32) -> Self {
        if id == Self::PRIMARY_CHANNEL_ID {
            Self::PrimaryChannel
        } else {
            Self::NodeId(id)
        }
    }
}

impl From<Target> for meshtastic::packet::PacketDestination {
    fn from(target: Target) -> Self {
        match target {
            Target::PrimaryChannel => meshtastic::packet::PacketDestination::Broadcast,
            Target::NodeId(id) => {
                meshtastic::packet::PacketDestination::Node(meshtastic::types::NodeId::new(id))
            }
        }
    }
}
