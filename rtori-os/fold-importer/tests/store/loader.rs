use crate::Store;

impl rtori_os_model::LoaderDyn<'_> for Store {
    fn model(&self) -> rtori_os_model::ModelSize {
        todo!()
    }

    fn copy_node_position(
        &mut self,
        from: &[rtori_os_model::Vector3F],
        offset: rtori_os_model::NodeIndex,
    ) {
        self.node_positions[(offset as usize)..(offset as usize) + from.len()]
            .copy_from_slice(from);
    }

    fn copy_node_external_forces(
        &mut self,
        from: &[rtori_os_model::Vector3F],
        offset: rtori_os_model::NodeIndex,
    ) {
        unimplemented!()
    }

    fn copy_node_config(
        &mut self,
        from: &[rtori_os_model::NodeConfig],
        offset: rtori_os_model::NodeIndex,
    ) {
        self.node_config[(offset as usize)..(offset as usize) + from.len()].copy_from_slice(from);
    }

    fn copy_node_geometry(
        &mut self,
        from: &[rtori_os_model::NodeGeometry],
        offset: rtori_os_model::NodeIndex,
    ) {
        self.node_geometry[(offset as usize)..(offset as usize) + from.len()].copy_from_slice(from);
    }

    fn copy_crease_geometry(
        &mut self,
        from: &[rtori_os_model::CreaseGeometry],
        offset: rtori_os_model::CreaseIndex,
    ) {
        self.crease_geometry[(offset as usize)..(offset as usize) + from.len()]
            .copy_from_slice(from);
    }

    fn copy_crease_parameters(
        &mut self,
        from: &[rtori_os_model::CreaseParameters],
        offset: rtori_os_model::CreaseIndex,
    ) {
        self.crease_parameters[(offset as usize)..(offset as usize) + from.len()]
            .copy_from_slice(from);
    }

    fn copy_face_indices(
        &mut self,
        from: &[rtori_os_model::Vector3U],
        offset: rtori_os_model::FaceIndex,
    ) {
        self.face_indices[(offset as usize)..(offset as usize) + from.len()].copy_from_slice(from);
    }

    fn copy_face_nominal_angles(
        &mut self,
        from: &[rtori_os_model::Vector3F],
        offset: rtori_os_model::FaceIndex,
    ) {
        self.face_nominal_angles[(offset as usize)..(offset as usize) + from.len()]
            .copy_from_slice(from);
    }

    fn copy_node_crease(
        &mut self,
        from: &[rtori_os_model::NodeCreaseSpec],
        offset: rtori_os_model::NodeCreaseIndex,
    ) {
        self.node_creases[(offset as usize)..(offset as usize) + from.len()].copy_from_slice(from);
    }

    fn copy_node_beam(
        &mut self,
        from: &[rtori_os_model::NodeBeamSpec],
        offset: rtori_os_model::NodeBeamIndex,
    ) {
        self.node_beams[(offset as usize)..(offset as usize) + from.len()].copy_from_slice(from);
    }

    fn copy_node_face(
        &mut self,
        from: &[rtori_os_model::NodeFaceSpec],
        offset: rtori_os_model::NodeFaceIndex,
    ) {
        self.node_faces[(offset as usize)..(offset as usize) + from.len()].copy_from_slice(from);
    }
}
