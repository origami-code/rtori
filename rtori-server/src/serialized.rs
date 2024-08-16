#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Serialized {
    pub vertices: Vec<f32>,
    pub faces: Vec<u32>,
}
