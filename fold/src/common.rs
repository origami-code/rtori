use tinyvec::TinyVec;

pub type SmallVec<T> = TinyVec<T>;

#[derive(Debug, Clone)]
pub struct PropertyNotMatchingError {
    pub container: &'static str,
    pub core_property_name: &'static str,
    pub queried_property_name: &'static str,
    pub index: usize,
}

impl std::fmt::Display for PropertyNotMatchingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl std::error::Error for PropertyNotMatchingError {}

pub type PropertyResult<T> = std::result::Result<Option<T>, PropertyNotMatchingError>;

pub struct DebugInfo {
    pub container: &'static str,
    pub core_property_name: &'static str,
    pub queried_property_name: &'static str,
}

pub fn get_property<'a, T: tinyvec::Array>(
    o: &'a Option<Vec<SmallVec<T>>>,
    idx: usize,
    debug_info: Option<DebugInfo>,
) -> PropertyResult<&'a SmallVec<T>> {
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
