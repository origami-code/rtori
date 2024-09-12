use std::num::NonZeroU64;

use encase::ShaderSize;
use wgpu::Buffer;

use super::ModelSize;

#[derive(encase::ShaderType)]
pub struct NodeFaceSpec {
    node_index: u32,
    face_index: u32,
    angles: [f32; 3],
}

pub struct Parameters {
    pub parameters: ModelSize,
    pub min_storage_alignment: u32,
}

impl Parameters {
    const STRIDE_VEC3: u64 = 4 /* f32 */ * 3 /* x, y, z */;
    const STRIDE_VEC4: u64 = 4 /* f32 */ * 4 /* x, y, z, error */;
    const STRIDE_NODE_POSITIONS_OFFSETS: u64 = Self::STRIDE_VEC3;
    const STRIDE_NODE_POSITIONS_UNCHANGING: u64 = Self::STRIDE_VEC3; // could be a uniform
    const STRIDE_NODE_VELOCITY: u64 = Self::STRIDE_VEC3;
    const STRIDE_NODE_ERROR: u64 = 4 /* f32 */ * 1 /* just one */;
    const STRIDE_NODE_EXTERNAL_FORCES: u64 = Self::STRIDE_VEC3;
    const STRIDE_NODE_CONFIGS: u64 = 4 /* u32 */ * 2 /* mass & fixed */;
    const STRIDE_NODE_GEOMETRY: u64 = 4 /* u32 */ * 6 /* node_crease,node_beam,node_face offsets & counts */;

    const STRIDE_CREASE_GEOMETRY: u64 = 4 /* u32 */ * 2  /* 2 per array */ * 3 /* 3 arrays */;
    const STRIDE_CREASE_FOLD_ANGLES: u64 = 4 /* f32 */ * 1 /* 1 per unit */;
    const STRIDE_CREASE_PHYSICS: u64 = 4 /* f32 */ * 2  /* 2 per halves */ * 2 /* 2 halves */;
    const STRIDE_CREASE_PARAMETERS: u64 = 4 /* f32 */ * 3 /* 3 per struct (k, d, target_fold_angle) */;

    const STRIDE_FACE_INDICES: u64 = 4 /* u32 */ * 3 /* a, b, c */;
    const STRIDE_FACE_NORMALS: u64 = Self::STRIDE_VEC3;
    const STRIDE_FACE_NOMINAL_TRIANGLES: u64 = Self::STRIDE_VEC3;

    const STRIDE_NODE_CREASES: u64 = 4 /* u32 */ * 2;
    const STRIDE_NODE_CREASES_CONSTRAINT_FORCES: u64 = Self::STRIDE_VEC3;

    const STRIDE_NODE_BEAMS: u64 = 4 /* f32/u32 */ * 5 /* components */;
    const STRIDE_NODE_BEAM_CONSTRAINT_FORCES: u64 = Self::STRIDE_VEC4;

    const STRIDE_NODE_FACES: u64 = NodeFaceSpec::SHADER_SIZE.get();
    const STRIDE_NODE_FACES_CONSTRAINT_FORCES: u64 = Self::STRIDE_VEC4;

    const fn align_address(cursor: u64, alignment: u64) -> u64 {
        let multiple = u64::div_ceil(cursor, alignment);

        multiple * alignment
    }

    const fn align_storage(&self, cursor: u64) -> u64 {
        Self::align_address(cursor, self.min_storage_alignment as u64)
    }

    pub fn apply_for_each_binding<F>(&self, mut f: F) -> u64
    where
        F: FnMut(/* offset */ u64, /* length */ u64),
    {
        let mut f_wrap = |cur: u64, count: u16, stride: u64| -> u64 {
            let offset = self.align_storage(cur);
            let size = u64::from(count) * stride;
            f(offset, size);
            offset + size
        };

        let cur = 0;

        let mut f_nodes = |cur, stride| f_wrap(cur, self.parameters.node_count, stride);
        let cur = f_nodes(cur, Self::STRIDE_NODE_POSITIONS_OFFSETS);
        let cur = f_nodes(cur, Self::STRIDE_NODE_POSITIONS_UNCHANGING);
        let cur = f_nodes(cur, Self::STRIDE_NODE_VELOCITY);
        let cur = f_nodes(cur, Self::STRIDE_NODE_ERROR);
        let cur = f_nodes(cur, Self::STRIDE_NODE_EXTERNAL_FORCES);
        let cur = f_nodes(cur, Self::STRIDE_NODE_CONFIGS);
        let cur = f_nodes(cur, Self::STRIDE_NODE_GEOMETRY);

        let mut f_crease = |cur, stride| f_wrap(cur, self.parameters.crease_count, stride);
        let cur = f_crease(cur, Self::STRIDE_CREASE_GEOMETRY);
        let cur = f_crease(cur, Self::STRIDE_CREASE_FOLD_ANGLES);
        let cur = f_crease(cur, Self::STRIDE_CREASE_PHYSICS);
        let cur = f_crease(cur, Self::STRIDE_CREASE_PARAMETERS);

        let mut f_faces = |cur, stride| f_wrap(cur, self.parameters.face_count, stride);
        let cur = f_faces(cur, Self::STRIDE_FACE_INDICES);
        let cur = f_faces(cur, Self::STRIDE_FACE_NORMALS);
        let cur = f_faces(cur, Self::STRIDE_FACE_NOMINAL_TRIANGLES);

        let cur = f_wrap(
            cur,
            self.parameters.node_crease_count,
            Self::STRIDE_NODE_CREASES,
        );
        let cur = f_wrap(
            cur,
            self.parameters.node_crease_count,
            Self::STRIDE_NODE_CREASES_CONSTRAINT_FORCES,
        );

        let cur = f_wrap(
            cur,
            self.parameters.node_beam_count,
            Self::STRIDE_NODE_BEAMS,
        );
        let cur = f_wrap(
            cur,
            self.parameters.node_beam_count,
            Self::STRIDE_NODE_BEAM_CONSTRAINT_FORCES,
        );

        let cur = f_wrap(
            cur,
            self.parameters.node_face_count,
            Self::STRIDE_NODE_FACES,
        );
        let cur = f_wrap(
            cur,
            self.parameters.node_face_count,
            Self::STRIDE_NODE_FACES_CONSTRAINT_FORCES,
        );

        cur
    }

    pub fn min_size(&self) -> u64 {
        return self.apply_for_each_binding(|_, _| {});
    }
}
