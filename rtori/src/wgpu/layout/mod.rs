mod bgld;

mod pass;

mod pass_per_face;
use pass_per_face::PassPerFace;

mod pass_per_crease_fold_angle;
use pass_per_crease_fold_angle::PassPerCreaseFoldAngle;

mod pass_per_crease_physics;
use pass_per_crease_physics::PassPerCreasePhysics;

mod pass_per_node_crease;
use pass_per_node_crease::PassPerNodeCrease;

mod pass_per_node_beam;
use pass_per_node_beam::PassPerNodeBeam;

mod pass_per_node_face;
use pass_per_node_face::PassPerNodeFace;

mod pass_per_node_accumulate;
use pass_per_node_accumulate::PassPerNodeAccumulate;

pub struct PipelineSetLayout {
    // Out-of-pass BGL
    pub bgl_positions_ro: wgpu::BindGroupLayout,
    pub bgl_per_crease_common: wgpu::BindGroupLayout,

    // Passes
    pub pass_per_face: PassPerFace,
    pub pass_per_crease_fold_angle: PassPerCreaseFoldAngle,
    pub pass_per_crease_physics: PassPerCreasePhysics,
    pub pass_per_node_crease: PassPerNodeCrease,
    pub pass_per_node_beam: PassPerNodeBeam,
    pub pass_per_node_face: PassPerNodeFace,
    pub pass_per_node_accumulate: PassPerNodeAccumulate
}

impl PipelineSetLayout {
    pub fn new(device: &wgpu::Device) -> Self {
        let bgl_positions_ro = device.create_bind_group_layout(&bgld::positions_ro::BIND_GROUP_POSITIONS_RO);
        let bgl_per_crease_common = device.create_bind_group_layout(&bgld::per_crease_common::BIND_GROUP_PER_CREASE_GEOMETRY);

        let pass_per_face = PassPerFace::new(device, &bgl_positions_ro);
        let pass_per_crease_fold_angle = PassPerCreaseFoldAngle::new(device, &bgl_positions_ro, &bgl_per_crease_common);
        let pass_per_crease_physics = PassPerCreasePhysics::new(device, &bgl_positions_ro, &bgl_per_crease_common);
        let pass_per_node_crease = PassPerNodeCrease::new(device, &bgl_positions_ro, &bgl_per_crease_common);
        let pass_per_node_beam = PassPerNodeBeam::new(&device, &bgl_positions_ro);
        let pass_per_node_face = PassPerNodeFace::new(&device, &bgl_positions_ro);
        let pass_per_node_accumulate = PassPerNodeAccumulate::new(&device);

        Self {
            bgl_positions_ro,
            bgl_per_crease_common,
            pass_per_face,
            pass_per_crease_fold_angle,
            pass_per_crease_physics,
            pass_per_node_crease,
            pass_per_node_beam,
            pass_per_node_face,
            pass_per_node_accumulate
        }
    }
}

