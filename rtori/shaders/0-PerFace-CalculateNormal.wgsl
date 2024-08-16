@group(0)
@binding(0)
var<storage, read> vertices: array<vec3<f32>>;

/// The indices into the vertices
@group(0)
@binding(1)
var<storage, read> face_indices: array<vec3<u32>>;

@group(0)
@binding(2)
var<storage, read_write> face_normals: array<vec3<f32>>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var face_index: u32 = global_id.x;
    var face: vec3<u32> = face_indices[face_index];
    
    var a: vec3<f32> = vertices[face.x];
    var b: vec3<f32> = vertices[face.y];
    var c: vec3<f32> = vertices[face.z];

    var normal: vec3<f32> = normalize(cross(b-a, c-a));
    face_normals[face_index] = normal;
}