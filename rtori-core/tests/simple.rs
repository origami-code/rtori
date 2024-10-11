#![feature(allocator_api)]

extern crate alloc;
use pollster::FutureExt as _;
use rtori_os_fold_importer::{import_in, transform::transform_in};
use rtori_os_model::ExtractorDyn;

const SIMPLE_FOLD: &'static str = include_str!("../../fold/testdata/simple.fold");

#[test]
fn test_step() {
    let parsed_input = serde_json::from_str::<fold::File>(SIMPLE_FOLD)
        .expect("source deserialization (json/fold file) failed");

    let allocator = alloc::alloc::Global;
    let mut solver =
        rtori_core::os_solver::Solver::create(rtori_core::os_solver::BackendFlags::CPU)
            .block_on()
            .unwrap();

    solver.load_fold_in(&parsed_input.key_frame, allocator);

    solver.step(1).expect("Step failed");

    let result = solver
        .extract(rtori_os_model::ExtractFlags::all())
        .expect("extract call failed");

    let mut positions = Vec::new();
    positions.resize(
        parsed_input.frame(0).unwrap().get().vertices.count(),
        rtori_os_model::Vector3F([6.9f32, 42.0f32, 6009.0f32]),
    );
    result.copy_node_position(&mut positions[..], 0);

    println!("Positions: {:?}", positions);
}
