#![feature(allocator_api)]

extern crate alloc;
use pollster::FutureExt as _;
use rtori_os_fold_importer::{import_in, transform::transform_in};
use rtori_os_model::ExtractorDyn;

use rstest::rstest;
use rstest_reuse::{self, *};

#[template]
#[rstest]
fn pair_test(#[files("./testdata/*/*.fold")] fold_file: std::path::PathBuf) {}

#[apply(pair_test)]
fn test_onestep(fold_file: std::path::PathBuf) {
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

fn parse_path(path: &std::path::Path) -> (&str, f32) {
    let stem = path.file_stem().unwrap();
    let (name, percentage_str) = stem.to_str().unwrap().rsplit_once('_').unwrap();
    let percentage: f32 = percentage_str.parse().expect(&format!(
        "Tried to parse {percentage_str} as a percentage, but failed"
    ));

    (name, percentage / 100.0)
}

#[apply(pair_test)]
fn test_stability(fold_file: std::path::PathBuf) {
    let allocator = alloc::alloc::Global;

    let mut solver =
        rtori_core::os_solver::Solver::create(rtori_core::os_solver::BackendFlags::CPU)
            .block_on()
            .unwrap();

    let (_, fold_ratio) = parse_path(fold_file.as_ref());
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(fold_file)
        .unwrap();
    let parsed_input = serde_json::from_reader::<_, fold::File>(file)
        .expect("source deserialization (json/fold file) failed");

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
        solver
            .step(1)
            .expect(&format!("Step failed at iteration {step_index}"));

        let result = solver
            .extract(rtori_os_model::ExtractFlags::all())
            .expect("extract call failed");

        result.copy_node_position(&mut positions_front[..], 0);
        for (i, pos) in positions_front.iter().enumerate() {
            assert!(
                pos.0.iter().all(|v| !v.is_nan()),
                "Iteration {step_index}: got a NaN in vertex {i} (got position: {pos:?})"
            );
        }

        // Check that the distance is not more than a tolerance
        const FAIL_TOLERANCE: f32 = 0.00001;
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
            "offset should be less than zero (+- {FAIL_TOLERANCE})"
        );

        // If the distance to the last iteration is even more negligeable, stop here
        if step_index > 0 {
            const CONVERGED_TOLERANCE: f32 = f32::EPSILON;
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
            if max_diff < CONVERGED_TOLERANCE {
                println!("Early exit as diff is {max_diff} < {CONVERGED_TOLERANCE}");
                return;
            }
        }

        std::mem::swap(&mut positions_front, &mut positions_back);
    }
}
