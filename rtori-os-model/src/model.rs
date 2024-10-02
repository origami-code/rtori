pub type NodeIndex = u32;
pub type CreaseIndex = u32;
pub type FaceIndex = u32;
pub type NodeCreaseIndex = u32;
pub type NodeBeamIndex = u32;
pub type NodeFaceIndex = u32;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(transparent)]
pub struct Vector3F(pub [f32; 3]);

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(transparent)]
pub struct Vector3U(pub [u32; 3]);

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(transparent)]
pub struct Vector2U(pub [u32; 3]);

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct NodeConfig {
    pub mass: f32,
    pub fixed: u8,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved: [u8; 3],
}

impl NodeConfig {
    pub const fn new() -> Self {
        Self {
            mass: 1.0,
            fixed: 0,
            _reserved: [0; 3],
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct NodeCreasePointer {
    pub offset: NodeCreaseIndex,
    pub count: u32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct NodeBeamPointer {
    pub offset: NodeBeamIndex,
    pub count: u32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct NodeFacePointer {
    pub offset: NodeFaceIndex,
    pub count: u32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct NodeGeometry {
    pub crease: NodeCreasePointer,
    pub beam: NodeBeamPointer,
    pub face: NodeFacePointer,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct CreaseGeometry {
    pub face_indices: [FaceIndex; 2],
    pub complementary_node_indices: [NodeIndex; 2],
    pub adjacent_node_indices: [NodeIndex; 2],
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct CreaseParameters {
    pub k: f32,
    pub d: f32,
    pub target_fold_angle: f32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct NodeCreaseSpec {
    pub crease_index: CreaseIndex,
    pub node_number: u32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct NodeBeamSpec {
    pub node_index: NodeIndex,
    pub k: f32,
    pub d: f32,
    pub length: f32,
    pub neighbour_index: u32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct NodeFaceSpec {
    pub node_index: NodeIndex,
    pub face_index: FaceIndex,
}
