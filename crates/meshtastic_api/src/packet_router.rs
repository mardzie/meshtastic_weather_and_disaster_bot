#[derive(Debug)]
pub struct Router {
    node_id: meshtastic::types::NodeId,
}

impl Router {
    pub fn new(node_id: u32) -> Self {
        Self {
            node_id: meshtastic::types::NodeId::new(node_id),
        }
    }
}

impl meshtastic::packet::PacketRouter<(), error::Error> for Router {
    fn handle_packet_from_radio(
        &mut self,
        packet: meshtastic::protobufs::FromRadio,
    ) -> Result<(), error::Error> {
        tracing::trace!("Router: Handle Packet from Radio: {:?}", packet);

        Ok(())
    }

    fn handle_mesh_packet(
        &mut self,
        packet: meshtastic::protobufs::MeshPacket,
    ) -> Result<(), error::Error> {
        tracing::trace!("Router: Handle Mesh Packet: {:?}", packet);

        Ok(())
    }

    fn source_node_id(&self) -> meshtastic::types::NodeId {
        self.node_id
    }
}

pub mod error {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {}
}
