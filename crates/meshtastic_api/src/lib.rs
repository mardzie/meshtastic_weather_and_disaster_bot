use meshtastic::protobufs;
use tokio::sync::mpsc::UnboundedReceiver;

pub use meshtastic::protobufs::MyNodeInfo;

use crate::{channel::Channel, node_id::NodeId, packet::Packet};

pub mod channel;
pub mod error;
pub mod node_id;
pub mod packet;

pub const MAX_PAYLOAD_SIZE: usize = 200;

#[derive(Debug)]
pub struct MeshtasticApi {
    stream_api: meshtastic::api::ConnectedStreamApi,
    node_id: NodeId,

    listener_task: tokio::task::JoinHandle<()>,
    exit_sender: tokio::sync::broadcast::Sender<()>,
}

impl MeshtasticApi {
    pub async fn new(
        serial_path: String,
        packet_sender: tokio::sync::mpsc::Sender<Packet>,
    ) -> Result<Self, error::Error> {
        let stream_api = meshtastic::api::StreamApi::new();
        tracing::trace!("Creating serial stream...");
        let stream_handle =
            meshtastic::utils::stream::build_serial_stream(serial_path, None, None, None)?;
        tracing::trace!("Serial stream created.");
        let (decoded_listener, stream_api) = stream_api.connect(stream_handle).await;

        let my_info_task = tokio::task::spawn(async { Self::wait_for_my_info(decoded_listener) });

        let config_id = meshtastic::utils::generate_rand_id();
        let stream_api = stream_api.configure(config_id).await?;

        let (my_node_info, decoded_listener) = my_info_task.await?.await;

        let (exit_sender, mut rx) = tokio::sync::broadcast::channel(1);
        let listener_task = tokio::task::spawn(async move {
            tokio::select! {
                _ = rx.recv() => {
                    tracing::info!("Exiting listener...");
                }
                _ = Self::listener_task(decoded_listener, packet_sender) => {
                    tracing::error!("Meshtastic Listener closed unexpected.");
                }
            }
        });

        Ok(Self {
            stream_api,
            node_id: NodeId::from(my_node_info),

            listener_task,
            exit_sender,
        })
    }

    async fn wait_for_my_info(
        mut listener: UnboundedReceiver<meshtastic::protobufs::FromRadio>,
    ) -> (
        meshtastic::protobufs::MyNodeInfo,
        UnboundedReceiver<meshtastic::protobufs::FromRadio>,
    ) {
        while let Some(from_radio) = listener.recv().await {
            if let Some(payload_variant) = from_radio.payload_variant {
                match payload_variant {
                    protobufs::from_radio::PayloadVariant::MyInfo(my_node_info) => {
                        return (my_node_info, listener);
                    }
                    _ => {}
                }
            };
        }

        panic!("Meshtastic connection lost: Failed to get `MyNodeInfo`.");
    }

    async fn listener_task(
        mut listener: UnboundedReceiver<meshtastic::protobufs::FromRadio>,
        sender: tokio::sync::mpsc::Sender<Packet>,
    ) {
        while let Some(from_radio) = listener.recv().await {
            if let Some(payload_variant) = from_radio.payload_variant {
                match payload_variant {
                    protobufs::from_radio::PayloadVariant::Packet(mesh_packet) => {
                        if let Err(_) = Self::handle_mesh_packet(mesh_packet, &sender).await {
                            return;
                        };
                    }
                    _ => {}
                }
            }
        }

        tracing::error!("Failed to listen: Meshtastic disconnected.");
    }

    async fn handle_mesh_packet(
        mesh_packet: meshtastic::protobufs::MeshPacket,
        sender: &tokio::sync::mpsc::Sender<Packet>,
    ) -> Result<(), ()> {
        if let Some(variation) = &mesh_packet.payload_variant {
            match &variation {
                protobufs::mesh_packet::PayloadVariant::Decoded(data) => {
                    tracing::debug!("Decoded Packet: {:?}", data);
                    tracing::debug!("Payload: {}", String::from_utf8_lossy(&data.payload));

                    if data.emoji == 0
                        && let Err(_) = sender.send(Packet::new(&mesh_packet, data)).await
                    {
                        tracing::warn!(
                            "All Meshtastic packet receivers have been closed. Meshtastic sender stopping..."
                        );
                        return Err(());
                    };
                }
                _ => {}
            }
        };

        Ok(())
    }

    /// Disconnect from Meshtastic device.
    pub async fn disconnect(self) {
        if self.exit_sender.send(()).is_err() {
            tracing::warn!("All tasks have stopped already.");
        };

        if let Err(e) = self.stream_api.disconnect().await {
            tracing::error!("Failed to disconnect from Meshtastic Device: {}", e);
        };

        if self.listener_task.await.is_err() {
            tracing::error!("Meshtastic listener task shut down unexpectedly.");
        };
    }

    pub fn get_node_id(&self) -> NodeId {
        self.node_id
    }

    pub async fn send_message(
        &self,
        text: String,
        target: packet::Target,
        channel: Option<Channel>,
    ) -> Result<(), error::SendError> {
        if text.len() > MAX_PAYLOAD_SIZE {
            return Err(error::SendError::TooBig(text.len()));
        };

        match self
            .stream_api
            .send_text(
                packet_router,
                text,
                target.into(),
                true,
                channel.unwrap_or_default().into(),
            )
            .await
        {
            _ => {}
        };

        Ok(())
    }
}
