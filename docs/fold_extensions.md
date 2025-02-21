# Extensions to the FOLD format

## General

- `rtori:$`: comments

## Visuzalization extensions (`rtori:viz:...`)

- `rtori:viz:uvs`: an array of arrays that contains UVs that might be referred to by each face
    - if there is only one uv dimension, this can be a simple `[[u0 v0] [u1 v1]]`
    - if there are more than one, then we have a vector of those
- `rtori:viz:faces_uvs`: vector of vectors indices into `rtori:uvs` where each element corresponds to a vertex
    - assuming two faces with three vertices, if there is only one UV dimension, this means `faces_uvs: [[uvA, uvB, uvC], [uvD, uvE, uvF]]`
    - if there are more than one uvs, that means `faces_uvs: [[[uvA, uvX], [uvB, uvY], [uvC, uvZ]], [[uvD, uvR], [uvE, uvG], [uvF, uvB]]]`
- `rtori:viz:faces_normals`: normals used for visualization calculations
- `rtori:viz:faces_tangeants`: tangeants used for visualization calculations

## Solver-specific: Origami Simulator extensions

- `rtori:os:edges_creaseStiffness`: crease stiffness
- `rtori:os:edges_axialStiffness`: axial stiffness
- `rtori:os:faces_stiffness`: face stiffness
- `rtori:os:vertices_fixed`: boolean on whether or not a vertex is excluded from the calculation
- `rtori:os:vertices_mass`: mass of the vertices for the calculation