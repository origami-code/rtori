use std::num::NonZeroU64;

use encase::ShaderSize;

use super::ModelSize;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, encase::ShaderType)]
pub struct NodeFaceSpec {
    node_index: u32,
    face_index: u32,
    angles: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parameters {
    pub parameters: ModelSize,
    pub min_storage_alignment: u32,
}


impl Parameters {
    pub const ORDER_NODE_POSITIONS_OFFSETS: usize = 0;
    pub const ORDER_NODE_POSITIONS_UNCHANGING: usize = 1;
    pub const ORDER_NODE_VELOCITY: usize = 2;
    pub const ORDER_NODE_ERROR: usize = 3;
    pub const ORDER_NODE_EXTERNAL_FORCES: usize = 4;
    pub const ORDER_NODE_CONFIGS: usize = 5;
    pub const ORDER_NODE_GEOMETRY: usize = 6;
    pub const ORDER_CREASE_GEOMETRY: usize = 7;
    pub const ORDER_CREASE_FOLD_ANGLES: usize = 8;
    pub const ORDER_CREASE_PHYSICS: usize = 9;
    pub const ORDER_CREASE_PARAMETERS: usize = 10;
    pub const ORDER_FACE_INDICES: usize = 11;
    pub const ORDER_FACE_NORMALS: usize = 12;
    pub const ORDER_FACE_NOMINAL_TRIANGLES: usize = 13;
    pub const ORDER_NODE_CREASES : usize = 14;
    pub const ORDER_NODE_CREASES_CONSTRAINT_FORCES : usize = 15;
    pub const ORDER_NODE_BEAMS : usize = 16;
    pub const ORDER_NODE_BEAMS_CONSTRAINT_FORCES : usize = 17;
    pub const ORDER_NODE_FACES : usize = 18;
    pub const ORDER_NODE_FACES_CONSTRAINT_FORCES : usize = 19;

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

    pub fn binding_ranges(&self) -> ([(u64, NonZeroU64); super::StorageBindings::COUNT], NonZeroU64)
    {
        let mut ranges: [Option<(u64, NonZeroU64)>; super::StorageBindings::COUNT] =
            [const { None }; super::StorageBindings::COUNT];
        
        let mut f = {
            let mut idx = 0;
            
            let mut append = move |offset, size| {
                (&mut ranges)[idx] = Some((offset, size));
                *(&mut idx) += 1;
            };

            let compute_and_append
                = move |order: usize, cur: u64, count: u16, stride: u64| -> u64 {
                if idx != order {
                    panic!("invalid order");
                }

                let offset = self.align_storage(cur);
                let size = u64::from(count) * stride;
                
                append(offset, NonZeroU64::new(size).unwrap());
                offset + size
            };

            compute_and_append
        };

        let cur = 0;

        let mut f_nodes = |order, cur, stride| f(order, cur, self.parameters.node_count, stride);
        let cur = f_nodes(Self::ORDER_NODE_POSITIONS_OFFSETS, cur, Self::STRIDE_NODE_POSITIONS_OFFSETS);
        let cur = f_nodes(Self::ORDER_NODE_POSITIONS_UNCHANGING, cur, Self::STRIDE_NODE_POSITIONS_UNCHANGING);
        let cur = f_nodes(Self::ORDER_NODE_VELOCITY, cur, Self::STRIDE_NODE_VELOCITY);
        let cur = f_nodes(Self::ORDER_NODE_ERROR, cur, Self::STRIDE_NODE_ERROR);
        let cur = f_nodes(Self::ORDER_NODE_EXTERNAL_FORCES, cur, Self::STRIDE_NODE_EXTERNAL_FORCES);
        let cur = f_nodes(Self::ORDER_NODE_CONFIGS, cur, Self::STRIDE_NODE_CONFIGS);
        let cur = f_nodes(Self::ORDER_NODE_GEOMETRY, cur, Self::STRIDE_NODE_GEOMETRY);

        let mut f_crease = |order, cur, stride| f(order, cur, self.parameters.crease_count, stride);
        let cur = f_crease(Self::ORDER_CREASE_GEOMETRY, cur, Self::STRIDE_CREASE_GEOMETRY);
        let cur = f_crease(Self::ORDER_CREASE_FOLD_ANGLES, cur, Self::STRIDE_CREASE_FOLD_ANGLES);
        let cur = f_crease(Self::ORDER_CREASE_PHYSICS, cur, Self::STRIDE_CREASE_PHYSICS);
        let cur = f_crease(Self::ORDER_CREASE_PARAMETERS, cur, Self::STRIDE_CREASE_PARAMETERS);

        let mut f_faces = |order, cur, stride| f(order, cur, self.parameters.face_count, stride);
        let cur = f_faces(Self::ORDER_FACE_INDICES, cur, Self::STRIDE_FACE_INDICES);
        let cur = f_faces(Self::ORDER_FACE_NORMALS, cur, Self::STRIDE_FACE_NORMALS);
        let cur = f_faces(Self::ORDER_FACE_NOMINAL_TRIANGLES, cur, Self::STRIDE_FACE_NOMINAL_TRIANGLES);

        let cur = f(
            Self::ORDER_NODE_CREASES,
            cur,
            self.parameters.node_crease_count,
            Self::STRIDE_NODE_CREASES,
        );
        let cur = f(
            Self::ORDER_NODE_CREASES_CONSTRAINT_FORCES,
            cur,
            self.parameters.node_crease_count,
            Self::STRIDE_NODE_CREASES_CONSTRAINT_FORCES,
        );

        let cur = f(
            Self::ORDER_NODE_BEAMS,
            cur,
            self.parameters.node_beam_count,
            Self::STRIDE_NODE_BEAMS,
        );
        let cur = f(
            Self::ORDER_NODE_BEAMS_CONSTRAINT_FORCES,
            cur,
            self.parameters.node_beam_count,
            Self::STRIDE_NODE_BEAM_CONSTRAINT_FORCES,
        );

        let cur = f(
            Self::ORDER_NODE_FACES,
            cur,
            self.parameters.node_face_count,
            Self::STRIDE_NODE_FACES,
        );
        let cur = f(
            Self::ORDER_NODE_FACES_CONSTRAINT_FORCES,
            cur,
            self.parameters.node_face_count,
            Self::STRIDE_NODE_FACES_CONSTRAINT_FORCES,
        );

        let unwrapped = ranges.map(|opt| opt.unwrap());
        let total_size =NonZeroU64::new(cur).unwrap();

        (unwrapped, total_size)
    }

    pub fn min_size(&self) -> NonZeroU64 {
        self.binding_ranges().1
    }
}
