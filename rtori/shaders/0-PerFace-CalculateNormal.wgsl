// Common

@group(0)
@binding(0)
var<storage, read> node_positions_unchanging: array<vec3<f32>>;

@group(0)
@binding(1)
var<storage, read> node_positions_offset: array<vec3<f32>>;

fn get_position(idx: u32) -> vec3<f32> {
    return node_positions_unchanging[idx] + node_positions_offset[idx];
}

// Specific

@group(1)
@binding(0)
var<storage, read> face_indices: array<vec3<u32>>;

@group(1)
@binding(1)
var<storage, read_write> face_normals: array<vec3<f32>>;

@compute
@workgroup_size(1)
fn per_face_calculate_normal(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var face_index: u32 = global_id.x;
    var face: vec3<u32> = face_indices[face_index];
    
    var a: vec3<f32> = get_position(face.x);
    var b: vec3<f32> = get_position(face.y);
    var c: vec3<f32> = get_position(face.z);

    var normal: vec3<f32> = normalize(cross(b-a, c-a));
    face_normals[face_index] = normal;
}