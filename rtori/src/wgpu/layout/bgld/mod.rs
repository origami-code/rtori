// bgld: Bind Group Layout Descriptor
// As some bing group layouts are reused through different pipelines, they are split from the pass description proper

pub mod positions_ro;
pub mod per_face;
pub mod per_crease_common;
pub mod per_crease_physics;
pub mod per_crease_fold_angles;
pub mod per_node_crease;
pub mod per_node_beam;
pub mod per_node_face;
pub mod per_node_accumulate;