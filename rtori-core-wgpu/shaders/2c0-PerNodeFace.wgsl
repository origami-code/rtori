// 2a0-PerNodeFace
// - EntryPoint: "perNodeFace"
// - Granularity: 1x per node_face item
// - Bindings:
//  - Groups: 2
//  - Buffers: 8 (G0: 2, G1: 6)
//  - Variables: 0

const TAU: f32 = 6.283185307179586476925286766559;

// Common
@group(0)
@binding(0)
var<storage, read> node_positions_unchanging: array<vec3<f32>>;

@group(0)
@binding(1)
var<storage, read> node_positions_offset: array<vec3<f32>>;

/// The indices into the vertices
@group(1)
@binding(0)
var<storage, read_write> node_faces_constraint_forces: array<vec4<f32>>; // x, y, z, error

@group(1)
@binding(1)
var<storage, read_write> node_velocity: array<vec3<f32>>;

struct NodeFaceSpec {
    node_index: u32,
    face_index: u32,
    angles: vec3<f32>
}

@group(1)
@binding(2)
var<storage, read> node_faces: array<NodeFaceSpec>;

@group(1)
@binding(3)
var<storage, read> face_indices: array<vec3<u32>>;

@group(1)
@binding(4)
var<storage, read> face_normals: array<vec3<f32>>;

@group(1)
@binding(5)
var<storage, read> face_nominal_angles: array<vec3<f32>>;


struct ConstraintResult {
    force: vec3<f32>,
    error: f32
}

fn compute_face_constraint(
    node_face_index: u32
) -> ConstraintResult {
    var force_acc: vec3<f32> = vec3<f32>(0, 0, 0);
    var err = 0.0;

    var node_face_spec: NodeFaceSpec = node_faces[node_face_index];

    return ConstraintResult(force_acc, err);
}

@compute
@workgroup_size(1)
fn per_node_face(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var node_face_index = global_id.x;

    var constraint_results = compute_face_constraint(
        node_face_index
    );

    node_faces_constraint_forces[node_face_index] = vec4<f32>(constraint_results.force, constraint_results.error);
}