#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelSize {
    pub node_count: u16,
    pub crease_count: u16,
    pub face_count: u16,
    pub node_beam_count: u16,
    pub node_crease_count: u16,
    pub node_face_count: u16,
}
