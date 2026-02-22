use std::ops::Deref;

#[derive(Debug, Clone, Copy)]
pub struct NodeId(u32);

impl From<u32> for NodeId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<NodeId> for u32 {
    fn from(id: NodeId) -> Self {
        id.0
    }
}

impl From<meshtastic::protobufs::MyNodeInfo> for NodeId {
    fn from(my_node_info: meshtastic::protobufs::MyNodeInfo) -> Self {
        Self::from(my_node_info.my_node_num)
    }
}

impl Deref for NodeId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
