#![feature(allocator_api)]

extern crate alloc;
use pollster::FutureExt as _;
use rtori_os_fold_importer::{import_in, transform::transform_in};

const SIMPLE_FOLD: &'static str = include_str!("../../fold/testdata/simple.fold");

#[test]
fn test_load() {
    let parsed_input = serde_json::from_str::<fold::File>(SIMPLE_FOLD)
        .expect("source deserialization (json/fold file) failed");

    let allocator = alloc::alloc::Global;
    let mut solver =
        rtori_core::os_solver::Solver::create(rtori_core::os_solver::BackendFlags::CPU)
            .block_on()
            .unwrap();

    solver.load_fold_in(&parsed_input.key_frame, alloc::alloc::Global);

    solver.step(50_000_000);
}
