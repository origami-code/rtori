use crate::collections::VecU;
use crate::layers::{EdgeOrder, FaceOrder};

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<Alloc>), bounds(Alloc: Clone)))]
pub struct LayerInformation<Alloc: core::alloc::Allocator> {
    #[serde(rename = "faceOrders")]
    pub face_orders: VecU<FaceOrder, Alloc>,

    #[serde(rename = "edgeOrders")]
    pub edge_orders: VecU<EdgeOrder, Alloc>,
}
crate::assert_deserializable!(assert_layers, LayerInformation<Alloc>);
