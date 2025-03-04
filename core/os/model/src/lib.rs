#![no_std]

#[cfg(feature = "define_proxy")]
pub mod proxy;

mod model;
pub use model::*;

mod extractor;
pub use extractor::*;

mod loader;
pub use loader::*;

mod extract_flags;
pub use extract_flags::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct ModelSize {
    pub nodes: NodeIndex,
    pub creases: CreaseIndex,
    pub faces: FaceIndex,
    pub node_creases: NodeCreaseIndex,
    pub node_beams: NodeBeamIndex,
    pub node_faces: NodeFaceIndex,
}

#[cfg(feature = "define_proxy")]
mod proxy_sa {
    use crate::proxy::*;
    use static_assertions as sa;

    macro_rules! check_proxy{
        ($t:ty) => {
            sa::assert_impl_all!(
                Proxy<&'_ mut [u8], $t>: core::ops::DerefMut<Target=[$t]>
            );
        }
    }
    check_proxy!(crate::Vector3F);
    check_proxy!(crate::Vector3U);
    check_proxy!(crate::Vector2U);
    check_proxy!(crate::NodeConfig);
    check_proxy!(crate::NodeGeometry);
    check_proxy!(crate::CreaseGeometry);
    check_proxy!(crate::CreaseParameters);
    check_proxy!(crate::NodeCreaseSpec);
    check_proxy!(crate::NodeBeamSpec);
    check_proxy!(crate::NodeFaceSpec);
}
