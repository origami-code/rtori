use nalgebra::SimdComplexField;
use nalgebra::{self as na, SimdPartialOrd, SimdValue};
use simba::simd;

type SimdVector3 = na::Vector3<simd::f32x8>;

pub struct Nodes {
    pub position_base: Vec<SimdVector3>,
    pub position_offset: Vec<SimdVector3>,
    pub velocities: Vec<SimdVector3>,
}

#[inline(always)]
const fn convert_index_to_packed(input: usize) -> (usize, usize) {
    (input / 8, (input % 8))
}

#[inline(always)]
fn query_single(input: &[SimdVector3], idx: usize) -> na::Vector3<f32> {
    let (outer, inner) = convert_index_to_packed(idx);
    let arr = input[outer];
    na::Vector3::<f32>::new(arr.x.0[inner], arr.y.0[inner], arr.z.0[inner])
}

#[inline(always)]
fn accumulate_one(m: &[na::Vector3<f32>; 8]) -> SimdVector3 {
    let x = simd::f32x8::from(m.map(|v| v.x));
    let y = simd::f32x8::from(m.map(|v| v.y));
    let z = simd::f32x8::from(m.map(|v| v.z));

    SimdVector3::new(x, y, z)
}

fn accumulate<'a, I: ExactSizeIterator<Item = na::Vector3<f32>> + 'a>(
    input: I,
) -> impl Iterator<Item = SimdVector3> + 'a {
    if input.size_hint().0 % 8 != 0 {
        panic!("Size needs to be a multiple of 8");
    }

    let chunks = input.array_chunks::<8>();
    chunks.map(|m| accumulate_one(&m))
}

/// Nomenclature is the following
/// For a crease P2-P4:
/// ```ascii
///              P4
///               |
/// P3 --- h2 --- | --- h1 --- P1
///               |
///              P2
/// ``
/// The force applied to Px is F_{crease} = C * coeff_x
/// - with C calculated at a later point, but not relative to a node position
/// - and coeff_x the following:
///     - for P1 & P3 (complementary vertices of the folds adjacent to the crease),
///       `coeff_x = n / h` with n the face normal and h the height
///     - for P2 & P4 (crease vertices) it's a bit more complex (see the paper)
struct CreaseForceCoefficients {
    /// The force applied to the complementary vertices by the crease
    pub vertices_forces: [na::Vector3<f32>; 2],

    /// The force applied to the crease nodes by the crease
    pub neighbours: [na::Vector3<f32>; 2],
}

impl Nodes {}

pub struct Faces {
    pub node_indices: Vec<[u32; 3]>,
}

pub struct Crease {
    pub node_indices: Vec<[u32; 2]>,
    pub faces: Vec<[u32; 2]>,
    pub fold_angle: Vec<f32>,
    pub target: Vec<f32>,
}

pub struct Buffers {
    pub nodes: Nodes,
    pub faces: Faces,
    pub creases: Crease,
}

fn get_complement_vertex_index_for_face_index(face_indices: [u32; 3], not: [u32; 2]) -> u32 {
    *face_indices
        .iter()
        .find(|&&idx| idx != not[0] && idx != not[1])
        .unwrap()
}

impl Buffers {
    // STEP 0
    pub fn calculate_normals<'a>(
        &'a self,
        positions: &'a [SimdVector3],
    ) -> impl ExactSizeIterator<Item = na::Vector3<f32>> + 'a {
        // First, we calculate every combined position using SIMD
        self.faces.node_indices.as_slice().iter().map(move |idx| {
            let f = |i| query_single(positions, (idx[i] / 8) as usize);

            let a = f(0);
            let b = f(1);
            let c = f(2);

            let ba = b - a;
            let ca = c - a;

            let result = ba.cross(&ca).normalize();
            result
        })
    }

    /// STEP 1.a - Calculates the (current) fold angles from the face normals and vertex positions
    pub fn calculate_fold_angles<'a>(
        &'a self,
        normals: &'a [SimdVector3],
        positions: &'a [SimdVector3],
    ) -> impl ExactSizeIterator<Item = f32> + 'a {
        self.creases
            .faces
            .iter()
            .zip(self.creases.node_indices.iter())
            .zip(self.creases.fold_angle.iter())
            .map(move |((face_indices, node_indices), previous_fold_angle)| {
                let normal_a = query_single(normals, face_indices[0] as usize);
                let normal_b = query_single(normals, face_indices[1] as usize);
                let normals_dot = normal_a.dot(&normal_b).clamp(-1.0f32, 1.0f32);

                let vertex_a = query_single(positions, node_indices[0] as usize);
                let vertex_b = query_single(positions, node_indices[1] as usize);

                let crease_vector = (vertex_b - vertex_a).normalize();
                let x = normals_dot;
                let y = normal_a.cross(&crease_vector).dot(&normal_b);

                let calculated_fold_angle = f32::atan2(y, x);
                let diff = {
                    const TAU: f32 = 6.283185307179586476925286766559f32;
                    let uncorrected = calculated_fold_angle - previous_fold_angle;
                    if uncorrected < -5.0 {
                        uncorrected + TAU
                    } else if uncorrected > 5.0 {
                        uncorrected - TAU
                    } else {
                        uncorrected
                    }
                };
                let corrected = calculated_fold_angle + diff;
                corrected
            })
    }

    /// STEP 1.b - Calculates the four partial derivatives of the force constraints for each crease
    pub fn calculate_crease_forces<'a>(
        &'a self,
        positions: &'a [SimdVector3],
    ) -> impl ExactSizeIterator<Item = CreaseForceCoefficients> + 'a {
        let affected_node_iter = self
            .creases
            .faces
            .iter()
            .zip(self.creases.node_indices.iter())
            .map(|(face_indices, vertex_indices)| {
                let edge_node_a = query_single(positions, vertex_indices[0] as usize);
                let edge_node_b = query_single(positions, vertex_indices[1] as usize);

                let node_for_face = |n| {
                    let face_node_indices = self.faces.node_indices[face_indices[n] as usize];
                    let face_node_index = get_complement_vertex_index_for_face_index(
                        face_node_indices,
                        *vertex_indices,
                    );
                    query_single(positions, face_node_index as usize)
                };

                let face_node_a = node_for_face(0);
                let face_node_b = node_for_face(1);

                [edge_node_a, edge_node_b, face_node_a, face_node_b]
            })
            .array_chunks::<8>()
            .map(|chunk| {
                let acc = |inner| {
                    accumulate_one(&[
                        chunk[0][inner],
                        chunk[1][inner],
                        chunk[2][inner],
                        chunk[3][inner],
                        chunk[4][inner],
                        chunk[5][inner],
                        chunk[6][inner],
                        chunk[7][inner],
                    ])
                };

                (acc(0), acc(1), acc(2), acc(3))
            })
            .map(|(edge_node_a, edge_node_b, face_node_a, face_node_b)| {
                let tol = 0.000001f32;

                let crease_vector: SimdVector3 = edge_node_b - edge_node_a;
                let crease_length: simd::f32x8 = crease_vector.norm();

                let mask: <simd::f32x8 as simd::SimdValue>::SimdBool = {
                    let abs: simd::f32x8 = crease_length.simd_abs();
                    let mask = crease_length.simd_ge(simd::f32x8::splat(tol));
                    mask
                };

                if mask.0.all() == false {
                    // We skip this chunk
                    return;
                }

                let crease_vector_normalized: SimdVector3 = crease_vector / crease_length;

                let vector_a: SimdVector3 = face_node_a - edge_node_a;
                let proj_a_length: simd::f32x8 = crease_vector_normalized.dot(&vector_a);

                let vector_b: SimdVector3 = face_node_b - edge_node_b;
                let proj_b_length: simd::f32x8 = crease_vector_normalized.dot(&vector_b);

                let calculate_triangle_height = |v: SimdVector3, l: simd::f32x8| -> simd::f32x8 {
                    let sum = (v.x * v.x) + (v.y * v.y) + (v.z * v.z) - (l * l);
                    let absed = sum.simd_abs();
                    let sqrted = absed.simd_sqrt();
                    sqrted
                };

                let h1: simd::f32x8 = calculate_triangle_height(vector_a, proj_a_length);
                let h2: simd::f32x8 = calculate_triangle_height(vector_b, proj_b_length);
                if h1 < tol || h2 < tol {
                    return unimplemented!();
                }

                return unimplemented!();
            });
    }

    pub fn process(&self) {
        let positions = self
            .nodes
            .position_base
            .iter()
            .zip(self.nodes.position_offset.iter())
            .map(|(base, offset)| base + offset)
            .collect::<Vec<SimdVector3>>();

        let face_normals = {
            let it = self.calculate_normals(&positions);
            accumulate(it).collect::<Vec<_>>()
        };

        let fold_angles = self
            .calculate_fold_angles(&face_normals, &positions)
            .collect::<Vec<_>>();

        let crease_forces = self.calculate_crease_forces(&positions);
    }
}
