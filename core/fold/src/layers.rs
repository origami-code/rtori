use super::indices::*;
use super::Lockstep;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
)]
#[repr(i8)]
pub enum Ordering {
    Above = 1,
    Below = -1,
    Unknown = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq /*, serde::Deserialize, serde::Serialize*/)]
pub struct FaceOrder {
    pub f: FaceIndex,
    pub g: FaceIndex,
    pub s: Ordering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq /*, serde::Deserialize, serde::Serialize*/)]
pub struct EdgeOrder {
    pub e: EdgeIndex,
    pub f: EdgeIndex,
    pub s: Ordering,
}

#[derive(Debug, Clone /*, serde::Deserialize, serde::Serialize*/)]
pub struct LayerInformation<'alloc> {
    //#[serde(rename = "faceOrders")]
    pub face_orders: Lockstep<'alloc, FaceOrder>,

    //#[serde(rename = "edgeOrders")]
    pub edge_orders: Lockstep<'alloc, EdgeOrder>,
}
