// 3-PerNode
// - EntryPoint: "perNode"
// - Granularity: 1x per node item
// - Bindings:
//  - Groups: 1
//  - Buffers: 10 (G0: 10)
//  - Variables: 1 (G0)

const TAU: f32 = 6.283185307179586476925286766559;

/// Output
@group(0)
@binding(0)
var<storage, read_write> node_position_offset: array<vec3<f32>>;

@group(0)
@binding(1)
var<storage, read_write> node_velocity: array<vec3<f32>>;

@group(0)
@binding(2)
var<storage, read_write> node_error: array<f32>;

@group(0)
@binding(3)
var<storage, read> node_crease_constraint_forces: array<vec3<f32>>; // x, y, z

@group(0)
@binding(4)
var<storage, read> node_beam_constraint_forces: array<vec4<f32>>; // x, y, z, error_accor

@group(0)
@binding(5)
var<storage, read> node_faces_constraint_forces: array<vec4<f32>>; // x, y, z, error_accor

struct NodeGeometry {
    // Offset/Count into node_beams
    beam_offset: u32,
    beam_count: u32,

    // Offset/Count into creases
    crease_offset: u32,
    crease_count: u32,

    // Offset/count into node_face
    face_offset: u32,
    face_count: u32,
}

@group(0)
@binding(6)
var<storage, read> node_geometry: array<NodeGeometry>;


struct NodeSimulationConfig {
    mass: f32,
    fixed: u32
}

/// per-node
@group(0)
@binding(7)
var<storage, read> node_configs: array<NodeSimulationConfig>;

// TODO: make it optional
@group(0)
@binding(8)
var<storage, read> node_external_forces: array<vec3<f32>>;

@group(0)
@binding(9)
var<uniform> dt: f32;

struct ConstraintResult {
    force: vec3<f32>,
    error_accor: f32
}


fn compute_force(node_index: u32) -> ConstraintResult {
    var external_force = node_external_forces[node_index];

    var geometry: NodeGeometry = node_geometry[node_index];

    var force_acc: vec3<f32> = external_force;
    var error_acc: f32 = 0.0;

    // Beam constraints
    for (var b: u32; b < geometry.beam_count; b++) {
        var node_beam_index = geometry.beam_offset + b;

        var constraint_result = node_beam_constraint_forces[node_beam_index];
        force_acc += constraint_result.xyz;
        error_acc += constraint_result.w;
    }

    // Crease Constraints
    for (var c: u32; c < geometry.crease_count; c++) {
        var node_beam_index = geometry.crease_offset + c;
        
        var constraint_result = node_crease_constraint_forces[node_beam_index];
        force_acc += constraint_result;
    }

    // Face Constraints
    for (var f: u32; f < geometry.face_count; f++) {
        var node_beam_index = geometry.face_offset + f;

        var constraint_result = node_faces_constraint_forces[node_beam_index];
        force_acc += constraint_result.xyz;
        error_acc += constraint_result.w;
    }

    // TODO: something about the error_accor being divided by the number of faces for the node
    return ConstraintResult(
        force_acc,
        error_acc
    );
}

@compute
@workgroup_size(1)
fn per_node_accumulate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var node_index = global_id.x;

    var current_velocity = node_velocity[node_index];
    var config = node_configs[node_index];

    var constraint_results = compute_force(node_index);

    var force = constraint_results.force;
    var new_velocity = force * dt / config.mass + current_velocity;
    node_velocity[node_index] = new_velocity;

    var is_fixed = config.fixed != 0;
    if (is_fixed) {
        // Don't do anything else
        node_error[node_index] = 0.0;
    } else {
        node_position_offset[node_index] += new_velocity * dt;
        node_error[node_index] = constraint_results.error_accor;
    }
}