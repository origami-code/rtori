const TAU: f32 = 6.283185307179586476925286766559;
const TOL: f32 = 0.000001;
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

// Specific

struct CreasePhysicsHalf {
    height: f32,
    coef: f32
}

struct CreasePhysics {
    inner: array<CreasePhysicsHalf, 2>
}

@group(2)
@binding(0)
var<storage, read_write> crease_physics: array<CreasePhysics>;

fn update_crease_phyics(crease_index: u32) {
    var output: CreasePhysics = CreasePhysics(array(
        CreasePhysicsHalf(-1, -1),
        CreasePhysicsHalf(-1, -1)
    ));

    var geometry: CreaseGeometry = crease_geometry[crease_index];
    
    var complement_a: u32 = geometry.complement_node_indices[0];
    var node_fa: vec3<f32> = get_position(complement_a);

    var complement_b: u32 = geometry.complement_node_indices[1];
    var node_fb: vec3<f32> = get_position(complement_b);

    var adjacent_a: u32 = geometry.adjacent_node_indices[0];
    var node_ea: vec3<f32> = get_position(adjacent_a);

    var adjacent_b: u32 = geometry.adjacent_node_indices[1];
    var node_eb: vec3<f32> = get_position(adjacent_b);

    var crease_vector: vec3<f32> = node_eb - node_ea;
    var crease_length: f32 = length(crease_vector);
    if (abs(crease_length) < TOL) {
        // disable
        crease_physics[crease_index] = output;
        return;
    }
    var crease_vector_normalized = crease_vector / crease_length;
    
    var vector_a: vec3<f32> = node_fa - node_ea;
    var proj_a_length: f32 = dot(crease_vector_normalized, vector_a);

    var vector_b: vec3<f32> = node_fb - node_ea; // not a typo ('ea')
    var proj_b_length: f32 = dot(crease_vector_normalized, vector_b);

    var dist_a: f32 = sqrt(abs(vector_a.x*vector_a.x+vector_a.y*vector_a.y+vector_a.z*vector_a.z-proj_a_length*proj_a_length));
    var dist_b: f32 = sqrt(abs(vector_b.x*vector_b.x+vector_b.y*vector_b.y+vector_b.z*vector_b.z-proj_b_length*proj_b_length));
    if (dist_a<TOL || dist_b<TOL){
        // disable
        crease_physics[crease_index] = output;
        return;
    }

    output = CreasePhysics(array(
        CreasePhysicsHalf(
            dist_a, proj_a_length / crease_length
        ),
        CreasePhysicsHalf(
            dist_b, proj_b_length / crease_length
        )
    ));  
    crease_physics[crease_index] = output;
}

const workgroup_size: i32 = 64; // @id(0) override workgroup_size: i32 = 64;

@compute
@workgroup_size(workgroup_size)
fn per_crease_physics(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var crease_index: u32 = global_id.x;
    if (crease_index > arrayLength(&crease_geometry)) {
        return;
    }

    update_crease_phyics(crease_index);
}