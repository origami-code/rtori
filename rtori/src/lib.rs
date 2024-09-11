// Velocity Buffer For Every Node (Vector3)
// Position Buffer For Every Node (Vector3)
// Step 1.
// Calculate the face normal for all triangular faces in the mesh (one face per 'thread')
// Calculate the current fold angle for all edges in the mesh (one edge per 'thread')
// Calculate the coefficients of eq 3-6 for all edges in the mesh (one edge per 'thread')
// Calculate the forces & velocities for all nodes in the mesh (one one per 'thread')
// Calculate positions for all nodes in mesh (one node per 'thread')

// There is essentially five index namespace
// per "node" - a vertex
// per "edge" - every two dirrectly connected nodes, they are all beams
// per "crease" - a subselection of edges who are of (M)ontain, (V)alley or (F) typ
// per "face" - a face is made up of three node index
/*#![feature(iter_array_chunks)]
#![feature(portable_simd)]
mod simd;*/

mod wgpu;
