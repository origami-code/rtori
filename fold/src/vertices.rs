use super::common::*;
use super::indices::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct Vertex(pub [f32; 3]);

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct VertexInformation {
    #[serde(rename = "vertices_coords")]
    pub coords: Option<Vec<Vertex>>,

    #[serde(rename = "vertices_vertices")]
    pub adjacent: Option<Vec<SmallVec<[VertexIndex; 8]>>>,

    #[serde(rename = "vertices_edges")]
    pub edges: Option<Vec<SmallVec<[EdgeIndex; 8]>>>,

    /// For each vertex, an array of face IDs for the faces incident to the vertex
    /// Possibly including None (null).
    #[serde(rename = "vertices_faces")]
    pub faces: Option<Vec<SmallVec<[Option<FaceIndex>; 8]>>>,
}

pub struct PerVertexInformation<'a> {
    pub coords: &'a Vertex,
    pub adjacent: Option<&'a SmallVec<[VertexIndex; 8]>>,
    pub edges: Option<&'a SmallVec<[EdgeIndex; 8]>>,
    pub faces: Option<&'a SmallVec<[Option<FaceIndex>; 8]>>,
}

impl VertexInformation {
    pub fn count(&self) -> usize {
        self.coords.as_ref().map(|c| c.len()).unwrap_or(0)
    }

    pub fn query(&self, index: VertexIndex) -> PropertyResult<PerVertexInformation> {
        let coords = match self.coords.as_ref().and_then(|v| v.get(index as usize)) {
            Some(coords) => coords,
            None => return Ok(None),
        };

        Ok(Some(PerVertexInformation {
            coords,
            adjacent: get_property(
                &self.adjacent,
                index as usize,
                Some(DebugInfo {
                    container: "VertexInformation",
                    core_property_name: "coords",
                    queried_property_name: "vertices_vertices",
                }),
            )?,
            edges: get_property(
                &self.edges,
                index as usize,
                Some(DebugInfo {
                    container: "VertexInformation",
                    core_property_name: "coords",
                    queried_property_name: "vertices_edges",
                }),
            )?,
            faces: get_property(
                &self.faces,
                index as usize,
                Some(DebugInfo {
                    container: "VertexInformation",
                    core_property_name: "coords",
                    queried_property_name: "vertices_faces",
                }),
            )?,
        }))
    }
}
