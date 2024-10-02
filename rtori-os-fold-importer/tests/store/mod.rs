use alloc::vec::Vec;
use rtori_os_model as model;

pub struct ComparableOutput {
    pub model_size: model::ModelSize,
    pub node_position: Vec<model::Vector3F>,
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
