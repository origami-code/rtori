const TAU: f32 = core::f32::consts::TAU;

// Common

@group(0)
@binding(0)
var<storage, read> node_positions_unchanging: array<vec3<f32>>;

@group(0)
@binding(1)
var<storage, read> node_positions_offset: array<vec3<f32>>;

fn get_position(index: u32) -> vec3<f32> {
    return node_positions_unchanging[index] + node_positions_offset[index];
}

// Crease-related
struct CreaseGeometry {
    complement_node_indices: array<u32, 2>,
    adjacent_node_indices: array<u32, 2>,
    face_indices: array<u32, 2>,
}

@group(1)
@binding(0)
var<storage, read> crease_geometry: array<CreaseGeometry>;

// Stage-specific
@group(2)
@binding(0)
var<storage, read> face_normals: array<vec3<f32>>;

@group(2)
@binding(1)
var<storage, read_write> crease_fold_angles: array<f32>;


fn update_fold_angle(crease_index: u32) {
    var spec: CreaseGeometry = crease_geometry[crease_index];

    // Face access
    var face_index_a: u32 = spec.face_indices[0];
    var normal_a: vec3<f32> = face_normals[face_index_a];

    var face_index_b: u32 = spec.face_indices[1];
    var normal_b: vec3<f32> = face_normals[face_index_b];

    var normals_dot_unclamped: f32 = dot(normal_a, normal_b); // normals are already normalized, we don't normalize by dividing by length
    var normals_dot_clamped: f32 = clamp(normals_dot_unclamped, -1.0, 1.0);

    // Node access
    var vertex_a: vec3<f32> = get_position(spec.adjacent_node_indices[0]);
    var vertex_b: vec3<f32> = get_position(spec.adjacent_node_indices[1]);

    var crease_vector: vec3<f32> = normalize(vertex_b - vertex_a);
    var x: f32 = normals_dot_clamped;
    var y: f32 = dot(cross(normal_a, crease_vector), normal_b); 

    var fold_angle: f32 = atan2(y, x);

    // Correct it (why ?)
    var previous_fold_angle: f32 = crease_fold_angles[crease_index];
    var diff: f32 = fold_angle - previous_fold_angle;
    var diff_corrected: f32 = diff;
    if (diff < -5.0) {
        diff_corrected += TAU;
    } else if (diff > 5.0) {
        diff_corrected -= TAU;
    }
    var fold_angle_corrected: f32 = previous_fold_angle + diff_corrected;

    crease_fold_angles[crease_index] = fold_angle_corrected;
}

const workgroup_size: i32 = 64; // @id(0) override workgroup_size: i32 = 64;

@compute
@workgroup_size(workgroup_size)
fn per_crease_fold_angles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var crease_index: u32 = global_id.x;
    if (crease_index > arrayLength(&crease_geometry)) {
        return;
    }

    update_fold_angle(crease_index);
}