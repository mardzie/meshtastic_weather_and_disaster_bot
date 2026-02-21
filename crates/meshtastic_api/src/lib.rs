use meshtastic::protobufs;
use tokio::sync::mpsc::UnboundedReceiver;

pub use meshtastic::protobufs::MyNodeInfo;

pub mod error;

#[derive(Debug)]
pub struct MeshtasticApi {
    stream_api: meshtastic::api::ConnectedStreamApi,
    my_node_info: MyNodeInfo,

    listener_task: tokio::task::JoinHandle<()>,
    exit_sender: tokio::sync::broadcast::Sender<()>,
}

impl MeshtasticApi {
    pub async fn new(serial_path: String) -> Result<Self, error::Error> {
        let available_ports = meshtastic::utils::stream::available_serial_ports()?;
        tracing::info!("Available Serial Ports: {:?}", available_ports);

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
                _ = Self::listener_task(decoded_listener) => {
                    tracing::error!("Meshtastic Listener closed unexpected.");
                }
            }
        });

        Ok(Self {
            stream_api,
            my_node_info,

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

    async fn listener_task(mut listener: UnboundedReceiver<meshtastic::protobufs::FromRadio>) {
        while let Some(from_radio) = listener.recv().await {
            if let Some(payload_variant) = from_radio.payload_variant {
                match payload_variant {
                    protobufs::from_radio::PayloadVariant::Packet(mesh_packet) => {
                        Self::handle_mesh_packet(mesh_packet).await;
                    }
                    _ => {}
                }
            }
        }

        tracing::error!("Failed to listen: Meshtastic disconnected.");
    }

    async fn handle_mesh_packet(mesh_packet: meshtastic::protobufs::MeshPacket) {
        if let Some(variation) = mesh_packet.payload_variant {
            match variation {
                protobufs::mesh_packet::PayloadVariant::Decoded(packet) => {
                    tracing::debug!("Decoded Packet: {:?}", packet);
                }
                _ => {}
            }
        }
    }
}
