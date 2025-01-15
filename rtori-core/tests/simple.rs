#![feature(allocator_api)]

extern crate alloc;
use pollster::FutureExt as _;
use rtori_os_fold_importer::{import_in, transform::transform_in};
use rtori_os_model::ExtractorDyn;

const DIAGONAL_FOLD: &'static str = include_str!("../testdata/diagonal/diagonal-cp_0.fold");
const SIMPLE_FOLD_RESULTS: [(f32, &'static str); 9] = [
    (0.00, include_str!("../testdata/simple/simple_0.fold")),
    (0.25, include_str!("../testdata/simple/simple_25.fold")),
    (0.50, include_str!("../testdata/simple/simple_50.fold")),
    (0.75, include_str!("../testdata/simple/simple_75.fold")),
    (1.00, include_str!("../testdata/simple/simple_100.fold")),
    (-0.25, include_str!("../testdata/simple/simple_-25.fold")),
    (-0.50, include_str!("../testdata/simple/simple_-50.fold")),
    (-0.75, include_str!("../testdata/simple/simple_-75.fold")),
    (-1.00, include_str!("../testdata/simple/simple_-100.fold")),
];

#[test]
fn test_onestep() {
    let parsed_input = serde_json::from_str::<fold::File>(DIAGONAL_FOLD)
        .expect("source deserialization (json/fold file) failed");

    let allocator = alloc::alloc::Global;
    let mut solver =
        rtori_core::os_solver::Solver::create(rtori_core::os_solver::BackendFlags::CPU)
            .block_on()
            .unwrap();

    solver.load_fold_in(&parsed_input.key_frame, allocator);

    for i in 0..1000 {
        solver.step(1).expect(&format!("Step {i} failed"));

        let mut positions = Vec::new();
        positions.resize(
            parsed_input.frame(0).unwrap().get().vertices.count(),
            rtori_os_model::Vector3F([6.9f32, 42.0f32, 6009.0f32]),
        );

        let result = solver
            .extract(rtori_os_model::ExtractFlags::all())
            .expect("extract call failed");
        result.copy_node_position(&mut positions[..], 0);

        for (i, pos) in positions.iter().enumerate() {
            assert!(
                pos.0.iter().all(|v| !v.is_nan()),
                "Step {i} failed : got a NaN in vertex {i} (got position: {pos:?})"
            );
        }
    }
}

#[test]
fn test_stability() {
    let allocator = alloc::alloc::Global;
    let mut solver =
        rtori_core::os_solver::Solver::create(rtori_core::os_solver::BackendFlags::CPU)
            .block_on()
            .unwrap();

    for (fold_percentage, expected) in &SIMPLE_FOLD_RESULTS {
        let parsed_input = serde_json::from_str::<fold::File>(expected)
            .expect("source deserialization (json/fold file) failed");

        solver.load_fold_in(&parsed_input.key_frame, allocator);

        let mut positions = Vec::new();
        positions.resize(
            parsed_input.frame(0).unwrap().get().vertices.count(),
            rtori_os_model::Vector3F([6.9f32, 42.0f32, 6009.0f32]),
        );

        println!("Testing simple with fold ratio {}", *fold_percentage);
        solver.set_fold_percentage(*fold_percentage).unwrap();
        solver.step(1).expect(&format!(
            "Step failed for fold percentage {}",
            *fold_percentage
        ));

        let result = solver
            .extract(rtori_os_model::ExtractFlags::all())
            .expect("extract call failed");

        result.copy_node_position(&mut positions[..], 0);
        for (i, pos) in positions.iter().enumerate() {
            assert!(pos.0.iter().all(|v| !v.is_nan()), "fold percentage {fold_percentage}: got a NaN in vertex {i} (got position: {pos:?})");
        }
        println!("Diff vector {positions:?} (expected 0)");
    }
}
