#define_import_path types::crease_physics
#import types::base

struct CreasePhysicsHalf {
    height: f32,
    coef: f32
}

struct CreasePhysics {
    inner: array<CreasePhysicsHalf, 2>
}