#![feature(allocator_api)]

extern crate alloc;
use pollster::FutureExt as _;
use rtori_os_fold_importer::{import_in, transform::transform_in};
use rtori_os_model::ExtractorDyn;

use rstest::rstest;
use rstest_reuse::{self, *};

static INIT: std::sync::Once = std::sync::Once::new();
fn initialize_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt::init();
    });
}

fn parse_path(path: &std::path::Path) -> (&str, f32) {
    let stem = path.file_stem().unwrap();
    let (name, percentage_str) = stem.to_str().unwrap().rsplit_once('_').unwrap();
    let percentage: f32 = percentage_str.parse().expect(&format!(
        "Tried to parse {percentage_str} as a percentage, but failed"
    ));

    (name, percentage / 100.0)
}

#[template]
#[rstest]
fn pair_test(#[files("testdata/*/*.fold")] fold_file: std::path::PathBuf) {}

/// test_onestep runs a single step for every test file available, checking only for crashes
#[apply(pair_test)]
fn test_onestep(fold_file: std::path::PathBuf) {
    initialize_tracing();

    let (_, fold_ratio) = parse_path(fold_file.as_ref());
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(fold_file)
        .unwrap();

    let parsed_input = serde_json::from_reader::<_, fold::File>(file)
        .expect("source deserialization (json/fold file) failed");

    let allocator = alloc::alloc::Global;
    let mut solver =
        rtori_core::os_solver::Solver::create(rtori_core::os_solver::BackendFlags::CPU)
            .block_on()
            .unwrap();

    solver.load_fold_in(&parsed_input.key_frame, allocator);
    solver.set_fold_percentage(fold_ratio).unwrap();

    solver.step(1).expect(&format!("Step failed"));

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

#[apply(pair_test)]
fn test_stability(fold_file: std::path::PathBuf) {
    initialize_tracing();

    let allocator = alloc::alloc::Global;

    let mut solver =
        rtori_core::os_solver::Solver::create(rtori_core::os_solver::BackendFlags::CPU)
            .block_on()
            .unwrap();

    let (name, fold_ratio) = parse_path(fold_file.as_ref());
    let full_name = format!("{name} ({:.1}%)", fold_ratio * 100.0);

    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(fold_file)
        .unwrap();
    let parsed_input = serde_json::from_reader::<_, fold::File>(file)
        .expect("{full_name}: source deserialization (json/fold file) failed");

    solver.load_fold_in(&parsed_input.key_frame, allocator);
    solver.set_fold_percentage(fold_ratio).unwrap();

    let mut positions_front = Vec::new();
    positions_front.resize(
        parsed_input.frame(0).unwrap().get().vertices.count(),
        rtori_os_model::Vector3F([6.9f32, 42.0f32, 6009.0f32]),
    );

    let mut positions_back = Vec::new();
    positions_back.resize(
        parsed_input.frame(0).unwrap().get().vertices.count(),
        rtori_os_model::Vector3F([6.9f32, 42.0f32, 6009.0f32]),
    );

    const MAXIMUM_ITERATIONS: i32 = 32_000;
    for step_index in 0..MAXIMUM_ITERATIONS {
        solver.step(1).expect(&format!(
            "{full_name}: Step failed at iteration {step_index}"
        ));

        let result = solver
            .extract(rtori_os_model::ExtractFlags::all())
            .expect("{full_name}: extract call failed");

        result.copy_node_position(&mut positions_front[..], 0);
        for (i, pos) in positions_front.iter().enumerate() {
            assert!(
                pos.0.iter().all(|v| !v.is_nan()),
                "{full_name}: Iteration {step_index}: got a NaN in vertex {i} (got position: {pos:?})"
            );
        }

        // Check that the distance is not more than a tolerance
        const FAIL_TOLERANCE: f32 = 0.001;
        let max_distance = positions_front
            .iter()
            .map(|v| {
                v.0.into_iter()
                    .map(|component| component * component)
                    .sum::<f32>()
                    .sqrt()
            })
            .max_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap())
            .unwrap();
        assert!(
            max_distance < FAIL_TOLERANCE,
            "{full_name}: Failure at step count {step_index}, offset should be less than zero (tolerance: +- {FAIL_TOLERANCE}) but is {max_distance}"
        );

        // If the distance to the last iteration is even more negligeable, stop here
        if step_index > 0 {
            const CONVERGED_TOLERANCE: f32 = 0.0; // f32::EPSILON;
            let max_diff = positions_front
                .iter()
                .zip(&positions_back)
                .map(|(front, back)| {
                    front
                        .0
                        .into_iter()
                        .zip(back.0)
                        .map(|(f, b)| f - b)
                        .map(|diff| diff * diff)
                        .sum::<f32>()
                        .sqrt()
                })
                .max_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap())
                .unwrap();
            if step_index > 10 && max_diff < CONVERGED_TOLERANCE {
                println!("{full_name}: Early exit as diff is {max_diff} < {CONVERGED_TOLERANCE}");
                return;
            }
        }

        std::mem::swap(&mut positions_front, &mut positions_back);
    }

    println!("{full_name}: Success after {MAXIMUM_ITERATIONS} steps");
}
