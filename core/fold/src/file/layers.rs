use crate::collections::Lockstep;
use crate::layers::{FaceOrder, EdgeOrder};

#[derive(serde_seeded::DeserializeSeeded, Debug, Clone, serde::Serialize)]
#[seeded(de(seed(crate::deser::Seed<'alloc>)))]
pub struct LayerInformation<'alloc> {
    #[serde(rename = "faceOrders")]
    pub face_orders: Lockstep<'alloc, FaceOrder>,

    #[serde(rename = "edgeOrders")]
    pub edge_orders: Lockstep<'alloc, EdgeOrder>,
}