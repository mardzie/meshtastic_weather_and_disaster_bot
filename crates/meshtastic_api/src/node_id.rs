use std::ops::Deref;

use crate::packet::Target;

#[derive(Debug, Clone, Copy)]
pub struct NodeId(u32);

impl NodeId {
    pub fn inner(&self) -> u32 {
        self.0
    }
}

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

impl From<&Target> for NodeId {
    fn from(target: &Target) -> Self {
        NodeId(target.into_id())
    }
}

impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for NodeId {}

impl Deref for NodeId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
