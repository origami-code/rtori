use crate::Handful;

pub use bumpalo::collections::{String, Vec};

#[derive(Debug, Clone)]
pub struct PropertyNotMatchingError {
    pub container: &'static str,
    pub core_property_name: &'static str,
    pub queried_property_name: &'static str,
    pub index: usize,
}

impl core::fmt::Display for PropertyNotMatchingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PropertyNotMatchingError: in object '{}' with {}[{}] existing, the property '{}' exist but does not contain at least {} elements",
            self.container,
            self.core_property_name,
            self.index,
            self.queried_property_name,
            self.index
        )
    }
}

impl core::error::Error for PropertyNotMatchingError {}

pub type PropertyResult<T> = core::result::Result<Option<T>, PropertyNotMatchingError>;

pub struct DebugInfo {
    pub container: &'static str,
    pub core_property_name: &'static str,
    pub queried_property_name: &'static str,
}

pub fn get_property<'a, T>(
    o: &'a Option<Vec<T>>,
    idx: usize,
    debug_info: Option<DebugInfo>,
) -> PropertyResult<&'a T> {
    o.as_ref()
        .map(|v| {
            v.get(idx as usize).ok_or_else(|| PropertyNotMatchingError {
                index: idx,
                container: debug_info.as_ref().map_or("unknown", |i| i.container),
                core_property_name: debug_info
                    .as_ref()
                    .map_or("unknown", |i| i.core_property_name),
                queried_property_name: debug_info
                    .as_ref()
                    .map_or("unknown", |i| i.queried_property_name),
            })
        })
        .map_or(Ok(None), |v| v.map(Some))
}

// The goal is to put in the vertexInformation type declaration in there
/*macro_rules! create_type {

}*/
