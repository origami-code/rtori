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
var<storage, read> face_normals: array<vec3<f32>>;

struct EdgeSpec {
    vertices: vec2<u32>,
    faces: vec2<u32>,
    k: f32,
    d: f32,
    target_theta: f32
}

@group(0)
@binding(4)
var<storage, read> edges: array<EdgeSpec>;

@group(0)
@binding(5)
var<storage, read> last_fold_angle: array<f32>;

@group(0)
@binding(6)
var<storage, read> new_fold_angle: array<f32>;

/// per-vertex
@group(0)
@binding(7)
var<storage, read> masses: array<f32>;

/// per-vertex
@group(0)
@binding(8)
var<storage, read> external_forces: array<vec3<f32>>;

@group(0)
@binding(9)
var<storage, read> last_velocities: array<vec3<f32>>;

struct VertexSpec {
    beam_offset: u32,
    beam_count: u32,
    crease_offset: u32,
    crease_count: u32,
    face_offset: u32,
    face_count: u32
}

@group(0)
@binding(10)
var<storage, read> vertices_spec: array<VertexSpec>;

/// per-node-beam: redirects to the beam spec
@group(0)
@binding(11)
var<storage, read> node_beams: array<u32>;

struct NodeCreaseSpec {
    index: u32,
    node_number: u32
}

/// per-node-crease: redirects to the crease spec
@group(0)
@binding(12)
var<storage, read> node_creases: array<NodeCreaseSpec>;


struct BeamSpec {
    neighbour: u32,
    k: f32,
    d: f32,
    length: f32
}

/// per-beam-spec: redirects to the crease spec
@group(0)
@binding(13)
var<storage, read> beam_specs: array<BeamSpec>;



struct CreaseGeoPart {
    height: f32,
    coef: f32
}

struct CreaseGeo {
    inner: array<CreaseGeoPart, 2>
}

@group(0)
@binding(15)
var<storage, read> crease_geo: array<CreaseGeo>;

@group(0)
@binding(16)
var<storage, read> crease_percentage: f32;

@group(0)
@binding(17)
var<storage, read> node_faces: array<u32>;


fn get_position(index: u32) -> vec3<f32> {
    return vertices_unchanging[index] + vertices_offset[index];
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var vertex_index: u32 = global_id.x;
    var vertex_mass = masses[vertex_index];
    var external_force: vec3<f32> = external_forces[vertex_index];
    var last_velocity: vec3<f32> = last_velocities[vertex_index];
    var original_position: vec3<f32> = vertices_unchanging[vertex_index];
    var offset_position: vec3<f32> = vertices_offset[vertex_index];
    var spec: VertexSpec = vertices_spec[vertex_index];
    
    var force_acc = external_force;
    var node_error: f32 = 0.0;

    // Beam Constraints
    // Go through every beam connected to the current vertex, and calculate the forces
    // coming from that
    for (var i: u32 = 0; i < 100; i ++) {
        if (i >= spec.beam_count) {
            break;
        }

        var beam_spec_index: u32 = node_beams[spec.beam_offset + i];
        var beam_spec: BeamSpec = beam_specs[beam_spec_index];

        // nb: neighbour
        var nb_offset_position: vec3<f32> = vertices_offset[beam_spec.neighbour];
        var nb_last_velocity: vec3<f32> = last_velocities[beam_spec.neighbour];
        var nb_original_position: vec3<f32> = vertices_unchanging[beam_spec.neighbour];

        var nominal_dist: vec3<f32> = nb_original_position - original_position;
        var delta_p: vec3<f32> = (nb_offset_position - offset_position) + nominal_dist;
        var delta_p_length: f32 = length(delta_p);
        delta_p -= delta_p * (beam_spec.length / delta_p_length);
        node_error += abs(delta_p_length  / length(nominal_dist) - 1.0);

        var delta_v: vec3<f32> = nb_last_velocity - last_velocity; 
        
        var force: vec3<f32> = delta_p * beam_spec.k + delta_v * beam_spec.d;
        force_acc += force;
    }

    // Crease Constraints
    for (var i: u32; i < 100; i++) {
        if (i >= spec.crease_count) {
            break;
        }

        var node_crease_spec: NodeCreaseSpec = node_creases[spec.crease_offset + i];

        var crease_index = node_crease_spec.index;
        var crease_spec: EdgeSpec = edges[crease_index];
        var crease_geo: CreaseGeo = crease_geo[crease_index];
        if (crease_geo.inner[0].height < 0.0) {
            //crease disabled bc it has collapsed too much
            continue;
        }

        // The global theta value influences
        var target_theta: f32 = crease_spec.target_theta * crease_percentage;
        var angular_force: f32 = crease_spec.k * (target_theta - last_fold_angle[crease_index]);
        
        var node_number = node_crease_spec.node_number;
        if (node_number > 2) {
            //crease reaction, node is on a crease
            var normal_a: vec3<f32> = face_normals[crease_spec.faces.x];
            var normal_b: vec3<f32> = face_normals[crease_spec.faces.y];
            var coef_a = crease_geo.inner[0].coef;
            var coef_b = crease_geo.inner[1].coef;
            if (node_number == 3) {
                coef_a = 1.0 - coef_a;
                coef_b = 1.0 - coef_b;
            }
            var force: vec3<f32> = - angular_force * (
                (coef_a / crease_geo.inner[0].height * normal_a)
                + (coef_b / crease_geo.inner[1].height * normal_b)
            );
            force_acc += force;
        } else {
            var face_index: u32 = crease_spec.faces[node_number];
            var normal: vec3<f32> = face_normals[face_index];

            var crease_geo_part: CreaseGeoPart = crease_geo.inner[node_number];
            var moment_arm: f32 = crease_geo_part.coef;

            var force: vec3<f32> = (angular_force / moment_arm) * normal;

            force_acc += force;
        }
    }

    // Face Constraints
    for (var i: u32; i < 100; i++) {
        if (i >= spec.face_offset) {
            break;
        }

        var face_index = node_faces[spec.face_offset + i];
        var face_vertex_indices: vec3<u32> = face_indices[face_index];

        var normal: vec3<f32> = face_normals[face_index];
            // TODO: end of face constraints

    }

}