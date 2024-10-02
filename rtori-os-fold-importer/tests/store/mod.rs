extern crate alloc;
use alloc::vec::Vec;
use rtori_os_model as model;

mod loader;
pub use loader::*;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
pub struct Store {
    pub node_positions: Vec<model::Vector3F>,
    pub node_config: Vec<model::NodeConfig>,
    pub node_geometry: Vec<model::NodeGeometry>,
    pub crease_parameters: Vec<model::CreaseParameters>,
    pub crease_geometry: Vec<model::CreaseGeometry>,
    pub face_indices: Vec<model::Vector3U>,
    pub face_nominal_angles: Vec<model::Vector3F>,
    pub node_creases: Vec<model::NodeCreaseSpec>,
    pub node_beams: Vec<model::NodeBeamSpec>,
    pub node_faces: Vec<model::NodeFaceSpec>,
}

fn assert_size_slice(to_compare: &[(&str, usize)]) {
    let mut expected = None;
    for (expr, len) in to_compare {
        match expected {
            Some(size) if size != len => {
                panic!("Expected size to match between expressions, was expecting len: {} as previous slices, but got len: {} for '{}'",
                    size,
                    len,
                    expr
                );
            }
            None => expected = Some(len),
            _ => (),
        }
    }
}

macro_rules! assert_same_size(
    ($($slice:expr),+) => (
        $crate::store::assert_size_slice(&[$((stringify!($slice), $slice.len())),+])
    )
);

impl Store {
    pub fn size(&self) -> model::ModelSize {
        assert_same_size!(self.node_positions, self.node_config, self.node_geometry);
        assert_same_size!(self.crease_parameters, self.crease_geometry);
        assert_same_size!(self.face_indices, self.face_nominal_angles);

        model::ModelSize {
            nodes: u32::try_from(self.node_positions.len()).unwrap(),
            creases: u32::try_from(self.crease_geometry.len()).unwrap(),
            faces: u32::try_from(self.face_indices.len()).unwrap(),
            node_creases: u32::try_from(self.node_creases.len()).unwrap(),
            node_beams: u32::try_from(self.node_beams.len()).unwrap(),
            node_faces: u32::try_from(self.node_faces.len()).unwrap(),
        }
    }

    pub fn with_size(size: model::ModelSize) -> Self {
        fn create<T: Default>(size: usize) -> Vec<T> {
            let mut v = Vec::with_capacity(size);
            v.resize_with(size, || T::default());
            v
        }

        Self {
            node_positions: create(size.nodes as usize),
            node_config: create(size.nodes as usize),
            node_geometry: create(size.nodes as usize),
            crease_parameters: create(size.creases as usize),
            crease_geometry: create(size.creases as usize),
            face_indices: create(size.faces as usize),
            face_nominal_angles: create(size.faces as usize),
            node_creases: create(size.node_creases as usize),
            node_beams: create(size.node_beams as usize),
            node_faces: create(size.node_faces as usize),
        }
    }
}
