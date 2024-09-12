# rtori-core

## How it works

Every fold pattern is split into several buffers

| **Name**                  | **Kind**     | **R/W**                   | **Scope**  | **Description**                                                                                                                           | **Content**                 |
|---------------------------|--------------|---------------------------|------------|-------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------|
| node_positions_unchanging | geometry     | copy-dest / ro            | per-node   | The positions of each node at the original pattern                                                                                        |                             |
| node_positions_offsets    | output       | copy-dest / copy-src / rw | per-node   | The position offsets calculated                                                                                                           |                             |
| node_velocities           | intermediate | gpu-only / rw             | per-node   | The velocities computed                                                                                                                   |                             |
| node_configs              | parameters   | copy-dest / ro            | per-node   | Mass & fixed state                                                                                                                        |                             |
| node_external_forces      | parameters   | copy-dest / ro            | per-node   | External forces applied to each node                                                                                                      |                             |
| crease_geometry           | geometry     | copy-dest / ro            | per-crease | Describes the geometry of the crease: which nodes are on the crease, and which ones are on the complement, as well as the faces involved  | struct CreaseGeometry       |
| crease_physics            | intermediate | gpu-only / rw             | per-crease | An intermediate representation of the physics of the creases, containing the coefficients derived from its geometry                       |                             |
| crease_parameters         | parameters   | copy-dest / ro            | per-crease | Parameters related to the simulation of the creases                                                                                       |                             |
| crease_target_fold_angle  | parameters   | copy-dest / ro            | per-crease | The target fold angle for each crease. This is split from crease_parameters as they could be set at a different (higher) frequency        |                             |
| face_indices              | geometry     | copy-dest / ro            | per-face   | The indices that define each face                                                                                                         |                             |
| face_nominal_angles       | geometry     | copy-dest / ro            | per-face   | The inner angles of each face. This is split from face_indices as they are not always used together in the same stages                    |                             |
| face_normals              | intermediate | gpu-only / ro             | per-face   | The normals for each face, as calculated from the winding order                                                                           |                             |

## TODO

- Make CreaseGeometry group 1 (it's used in everything except normals)
