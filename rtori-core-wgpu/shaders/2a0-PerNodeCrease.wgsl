// 2a0-PerNodeCrease
// - EntryPoint: "perNodeCrease"
// - Granularity: 1x per node_creases item
// - Bindings:
//  - Groups: 3
//  - Buffers: 10 (G0: 2, G1: 1, G2: 7)
//  - Variables: 1 (G2: 1)

const TAU: f32 = 6.283185307179586476925286766559;

// Common
@group(0)
@binding(0)
var<storage, read> node_positions_unchanging: array<vec3<f32>>;

@group(0)
@binding(1)
var<storage, read> node_positions_offset: array<vec3<f32>>;

// Crease
struct CreaseGeometry {
    complement_node_indices: array<u32, 2>,
    adjacent_node_indices: array<u32, 2>,
    face_indices: array<u32, 2>,
}

@group(1)
@binding(0)
var<storage, read> crease_geometry: array<CreaseGeometry>;

/// Per Node-Crease
@group(2)
@binding(0)
var<storage, read_write> node_crease_constraint_forces: array<vec3<f32>>; // x, y, z

struct NodeCreaseSpec {
    index: u32,
    // 0, 1, 2, 3, 4
    node_number: u32
}

/// per-node-crease: redirects to the crease spec
@group(2)
@binding(1)
var<storage, read> node_creases: array<NodeCreaseSpec>;

@group(2)
@binding(2)
var<storage, read> crease_fold_angles: array<f32>;

struct CreasePhysicsHalf {
    height: f32,
    coef: f32
}

struct CreasePhysics {
    inner: array<CreasePhysicsHalf, 2>
}

@group(2)
@binding(3)
var<storage, read> crease_physics: array<CreasePhysics>;

struct CreaseParameters {
    k: f32,
    d: f32,
    target_fold_angle: f32
}

@group(2)
@binding(4)
var<storage, read> crease_parameters: array<CreaseParameters>;

@group(2)
@binding(5)
var<uniform> crease_percentage: f32;

@group(2)
@binding(6)
var<storage, read> face_indices: array<vec3<u32>>;

@group(2)
@binding(7)
var<storage, read> face_normals: array<vec3<f32>>;




// Crease constraints

fn compute_crease_constraint(node_crease_index: u32) -> vec3<f32> {
    var node_crease_spec: NodeCreaseSpec = node_creases[node_crease_index];
    
    var crease_index = node_crease_spec.index;
    
    var physics: CreasePhysics = crease_physics[crease_index];
    if (physics.inner[0].height < 0.0) {
        //crease disabled bc it has collapsed too much
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    // The global fold_angle value influences here
    var params: CreaseParameters = crease_parameters[crease_index];

    var target_fold_angle: f32 = params.target_fold_angle;
    var adjusted_target_fold_angle: f32 = target_fold_angle * crease_percentage;

    var angular_force: f32 = params.k * (adjusted_target_fold_angle - crease_fold_angles[crease_index]);

    var geometry: CreaseGeometry = crease_geometry[crease_index];
    
    var node_number = node_crease_spec.node_number;
    var force: vec3<f32>;
    if (node_number > 2) {
        //crease reaction, node is on a crease

        var face_index_a = geometry.face_indices[0];
        var normal_a: vec3<f32> = face_normals[face_index_a];
        var coef_a = physics.inner[0].coef;

        var face_index_b = geometry.face_indices[1];
        var normal_b: vec3<f32> = face_normals[face_index_b];
        var coef_b = physics.inner[1].coef;

        if (node_number == 3) {
            // oneminus, why ?
            coef_a = 1.0 - coef_a;
            coef_b = 1.0 - coef_b;
        }

        force = - angular_force * (
            (coef_a / physics.inner[0].height * normal_a)
            + (coef_b / physics.inner[1].height * normal_b)
        );
    } else {
        var face_index: u32 = geometry.face_indices[node_number];
        var normal: vec3<f32> = face_normals[face_index];

        var crease_physics_part: CreasePhysicsHalf = physics.inner[node_number];
        var moment_arm: f32 = crease_physics_part.coef;

        force = (angular_force / moment_arm) * normal;
    }

    return force;
}


@compute
@workgroup_size(1)
fn per_node_crease(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var node_crease_index = global_id.x;

    var constraint_force = compute_crease_constraint(
        node_crease_index
    );

    node_crease_constraint_forces[node_crease_index] = constraint_force;
}