// 2a0-PerNodeBeam
// - EntryPoint: "perNodeBeam"
// - Granularity: 1x per node_beams item
// - Bindings:
//  - Groups: 2
//  - Buffers: 5 (G0: 2, G1: 3)

const TAU: f32 = core::f32::consts::TAU;

// Common
@group(0)
@binding(0)
var<storage, read> node_positions_unchanging: array<vec3<f32>>;

@group(0)
@binding(1)
var<storage, read> node_positions_offset: array<vec3<f32>>;

/// Per Node-Beam
@group(1)
@binding(0)
var<storage, read_write> node_beam_constraint_forces: array<vec4<f32>>; // x, y, z, error

@group(1)
@binding(1)
var<storage, read> node_velocity: array<vec3<f32>>;

struct BeamSpec {
    node_index: u32,
    k: f32,
    d: f32,
    length: f32,
    neighbour_index: u32
}

@group(1)
@binding(2)
var<storage, read> node_beams: array<BeamSpec>;

struct DeltaPResult {
    delta_p: vec3<f32>,
    err: f32
}

fn compute_delta_p(
    spec: BeamSpec,
    position_unchanging: vec3<f32>,
    current_position_offset: vec3<f32>,
) -> DeltaPResult {
    var nb_position_unchanging: vec3<f32> = node_positions_unchanging[spec.neighbour_index];
    var nb_position_offset: vec3<f32> = node_positions_offset[spec.neighbour_index];
    var nominal_dist: vec3<f32> = nb_position_unchanging - position_unchanging;

    var delta_p: vec3<f32> = (nb_position_offset - current_position_offset) + nominal_dist;
    var delta_p_length: f32 = length(delta_p);
    delta_p -= delta_p * (spec.length / delta_p_length);
        
    var error = abs(delta_p_length  / length(nominal_dist) - 1.0);

    return DeltaPResult(delta_p, error);
}

fn compute_delta_v(
    neighbour_index: u32,
    current_velocity: vec3<f32>
) -> vec3<f32> {
    var nb_velocity: vec3<f32> = node_velocity[neighbour_index];
    var delta_v: vec3<f32> = nb_velocity - current_velocity;
    return delta_v;
}

struct ConstraintResult {
    force: vec3<f32>,
    error: f32
}

fn compute_beam_constraint(
    node_beam_index: u32,
    position_unchanging: vec3<f32>,
    current_position_offset: vec3<f32>,
    current_velocity: vec3<f32>
) -> ConstraintResult {
    var spec: BeamSpec = node_beams[node_beam_index];

    // Neighbour position
    var delta_p_result: DeltaPResult = compute_delta_p(spec, position_unchanging, current_position_offset);
    var delta_v: vec3<f32> = compute_delta_v(spec.neighbour_index, current_velocity);
    var force: vec3<f32> = delta_p_result.delta_p * spec.k + delta_v * spec.d;

    return ConstraintResult(force, delta_p_result.err);
}

const workgroup_size: i32 = 64; // @id(0) override workgroup_size: i32 = 64;

@compute
@workgroup_size(workgroup_size)
fn per_node_beam(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var node_beam_index = global_id.x;
    if (node_beam_index > arrayLength(&node_beams)) {
        return;
    }

    var node_beam: BeamSpec = node_beams[node_beam_index];
    var node_index = node_beam.node_index;

    var position_unchanging = node_positions_unchanging[node_index];
    var current_position_offset = node_positions_offset[node_index];
    var current_velocity = node_velocity[node_index];
    
    var constraint_results = compute_beam_constraint(
        node_beam_index,
        position_unchanging,
        current_position_offset,
        current_velocity
    );

    node_beam_constraint_forces[node_beam_index] = vec4<f32>(constraint_results.force, constraint_results.error);
}