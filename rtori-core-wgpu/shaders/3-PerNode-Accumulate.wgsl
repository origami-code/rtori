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
    error: f32
}

fn compute_force(
    node_index: u32,
    local_offset: u32,
    local_chunk_size: u32
) -> ConstraintResult {
    var geometry: NodeGeometry = node_geometry[node_index];

    var force_acc: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    var error_acc: f32 = 0.0;

    // Beam constraints
    for (var b: u32 = local_offset; b < min(geometry.beam_count, local_offset + local_chunk_size); b++) {
        var node_beam_index = geometry.beam_offset + b;

        var constraint_result = node_beam_constraint_forces[node_beam_index];
        force_acc += constraint_result.xyz;
        error_acc += constraint_result.w;
    }

    // Crease Constraints
    for (var c: u32 = local_offset; c < min(geometry.crease_count, local_offset + local_chunk_size); c++) {
        var node_beam_index = geometry.crease_offset + c;
        
        var constraint_result = node_crease_constraint_forces[node_beam_index];
        force_acc += constraint_result;
    }

    // Face Constraints
    for (var f: u32  = local_offset; f < min(geometry.face_count, local_offset + local_chunk_size); f++) {
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

// workgroup_size * chunk_size should be larger than any of:
// - crease per node count
// - beam per node count
// - face per node count
//
// And at a minimum (for performance), workgroup_size should be 64
const workgroup_size: i32 = 64; // @id(0) override workgroup_size: i32 = 64;

// The number of 'crease', 'beam' or 'face' processed per local invocation
// @id(1) override chunk_size: i32 = 1;

@compute
@workgroup_size(workgroup_size)
fn per_node_accumulate(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    //@builtin(local_invocation_index) local_index: u32
) {
    let node_index: u32 = global_id.x;// - local_index;
    if (node_index > arrayLength(&node_configs)) {
        return;
    }

    var current_velocity = node_velocity[node_index];
    var config = node_configs[node_index];

    // Workgroup synchronisation: we take the accumulated resut
    var result: ConstraintResult = compute_force(
        node_index,
        u32(0),//local_index * chunk_size,
        u32(100)
    );
    //atomicAdd(force_acc, result.force);
    //atomicAdd(error_acc, result.error_accor);
    //workgroupBarrier();

    // We only let index 0 write to the final destination
    // if (local_index != 0) {
    //     return;
    // }
    
    let force_acc = result.force;
    let error_acc = result.error;

    var external_force = node_external_forces[node_index];
    var force = external_force + force_acc;
    var new_velocity = force * dt / config.mass + current_velocity;
    node_velocity[node_index] = new_velocity;

    var is_fixed = config.fixed != 0;
    if (is_fixed) {
        // Don't do anything else
        node_error[node_index] = 0.0;
    } else {
        node_position_offset[node_index] += new_velocity * dt;

        var error = error_acc;
        node_error[node_index] = error;
    }
}