const TAU: f32 = 6.283185307179586476925286766559;

@group(0)
@binding(0)
var<storage, read> vertices_unchanging: array<vec3<f32>>;

@group(0)
@binding(1)
var<storage, read> vertices_offset: array<vec3<f32>>;

/// The indices into the vertices
@group(0)
@binding(2)
var<storage, read> face_indices: array<vec3<u32>>;

@group(0)
@binding(3)
var<storage, read_write> face_normals: array<vec3<f32>>;

struct EdgeSpec {
    vertices: vec2<u32>,
    faces: vec2<u32>
}

@group(0)
@binding(4)
var<storage, read> edges: array<EdgeSpec>;

@group(0)
@binding(5)
var<storage, read> last_fold_angles: array<f32>;

@group(0)
@binding(6)
var<storage, read_write> new_fold_angle: array<f32>;

struct CreaseGeoPart {
    height: f32,
    coef: f32
}

struct CreaseGeo {
    inner: array<CreaseGeoPart, 2>
}

@group(0)
@binding(15)
var<storage, read_write> crease_geo: array<CreaseGeo>;

fn get_position(index: u32) -> vec3<f32> {
    return vertices_unchanging[index] + vertices_offset[index];
}

fn update_fold_angle(edge_index: u32) {
    var spec: EdgeSpec = edges[edge_index];

    // Face access
    var normal_a: vec3<f32> = face_normals[spec.faces.x];
    var normal_b: vec3<f32> = face_normals[spec.faces.y];

    var normals_dot_unclamped: f32 = dot(normal_a, normal_b); // normals are already normalized, we don't normalize by dividing by length
    var normals_dot_clamped: f32 = clamp(normals_dot_unclamped, -1.0, 1.0);

    // Node access
    var vertex_a: vec3<f32> = get_position(spec.vertices.x);
    var vertex_b: vec3<f32> = get_position(spec.vertices.y);

    var crease_vector: vec3<f32> = normalize(vertex_b - vertex_a);
    var x: f32 = normals_dot_clamped;
    var y: f32 = dot(cross(normal_a, crease_vector), normal_b); 

    var fold_angle: f32 = atan2(y, x);

    // Correct it (why ?)
    var previous_fold_angle: f32 = last_fold_angles[edge_index];
    var diff: f32 = fold_angle - previous_fold_angle;
    var diff_corrected: f32 = diff;
    if (diff < -5.0) {
        diff_corrected += TAU;
    } else if (diff > 5.0) {
        diff_corrected -= TAU;
    }
    var fold_angle_corrected: f32 = previous_fold_angle + diff_corrected;

    new_fold_angle[edge_index] = fold_angle_corrected;
}

fn get_complement_vertex_index_for_face_index(face_index: u32, not: vec2<u32>) -> u32 {
    var vertices_in_face = face_indices[face_index];

    var x_is: vec3<bool> = vec3<bool>(
        not.x == vertices_in_face.x,
        not.x == vertices_in_face.y,
        not.x == vertices_in_face.z
    );
    
    var y_is: vec3<bool> = vec3<bool>(
        not.y == vertices_in_face.x,
        not.y == vertices_in_face.y,
        not.y == vertices_in_face.z
    );

    var one_is = x_is || y_is;

    var other: u32 = 0;
    if (one_is.x && one_is.y) {
        other = vertices_in_face.z;
    } else if (one_is.y && one_is.z) {
        other = vertices_in_face.x;
    } else if (one_is.x && one_is.z) {
        other = vertices_in_face.y;
    }
    return other;
}

fn update_crease_geo(edge_index: u32) {
    var output: CreaseGeo = CreaseGeo(array(
        CreaseGeoPart(-1, -1),
        CreaseGeoPart(-1, -1)
    ));

    var edge: EdgeSpec = edges[edge_index];
    
    var face_vertex_a: u32 = get_complement_vertex_index_for_face_index(
        edge.faces.x,
        edge.vertices
    );
    var face_vertex_b: u32 = get_complement_vertex_index_for_face_index(
        edge.faces.y,
        edge.vertices
    );
    var edge_vertex_a: u32 = edge.vertices.x;
    var edge_vertex_b: u32 = edge.vertices.y;

    var node_fa: vec3<f32> = get_position(face_vertex_a);
    var node_fb: vec3<f32> = get_position(face_vertex_b);
    var node_ea: vec3<f32> = get_position(edge_vertex_a);
    var node_eb: vec3<f32> = get_position(edge_vertex_b);

    var tol: f32 = 0.000001;

    var crease_vector: vec3<f32> = node_eb - node_ea;
    var crease_length: f32 = length(crease_vector);
    if (abs(crease_length) < tol) {
        // disable
        crease_geo[edge_index] = output;
        return;
    }
    var crease_vector_normalized = crease_vector / crease_length;
    
    var vector_a: vec3<f32> = node_fa - node_ea;
    var proj_a_length: f32 = dot(crease_vector_normalized, vector_a);

    var vector_b: vec3<f32> = node_fb - node_ea; // not a typo ('ea')
    var proj_b_length: f32 = dot(crease_vector_normalized, vector_b);

    var dist_a: f32 = sqrt(abs(vector_a.x*vector_a.x+vector_a.y*vector_a.y+vector_a.z*vector_a.z-proj_a_length*proj_a_length));
    var dist_b: f32 = sqrt(abs(vector_b.x*vector_b.x+vector_b.y*vector_b.y+vector_b.z*vector_b.z-proj_b_length*proj_b_length));
    if (dist_a<tol || dist_b<tol){
        // disable
        crease_geo[edge_index] = output;
        return;
    }

    output = CreaseGeo(array(
        CreaseGeoPart(
            dist_a, proj_a_length / crease_length
        ),
        CreaseGeoPart(
            dist_b, proj_b_length / crease_length
        )
    ));  
    crease_geo[edge_index] = output;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var edge_index: u32 = global_id.x;
    update_fold_angle(edge_index);
    update_crease_geo(edge_index);
}