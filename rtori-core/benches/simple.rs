#![feature(allocator_api)]

use core::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};

extern crate alloc;
use pollster::FutureExt as _;
use rtori_os_fold_importer::{import_in, transform::transform_in};

const SIMPLE_FOLD: &'static str = include_str!("../../fold/testdata/simple.fold");
const THIRTEEN_HORNS_FOLD: &'static str =
    include_str!("../../fold/testdata/13-horns-123-vertices-augmented-triangulated.fold");

fn bench_source(c: &mut Criterion, name: &str, fold_str: &str) {
    let parsed_input = serde_json::from_str::<fold::File>(fold_str)
        .expect("source deserialization (json/fold file) failed");

    let allocator = alloc::alloc::Global;
    let mut solver =
        rtori_core::os_solver::Solver::create(rtori_core::os_solver::BackendFlags::CPU)
            .block_on()
            .unwrap();

    solver.load_fold(&parsed_input.key_frame);

    c.bench_function(name, |b| b.iter(|| black_box(solver.step(1))));
}

fn simple_benchmark(c: &mut Criterion) {
    bench_source(c, "step_simple", SIMPLE_FOLD);
    bench_source(c, "step_thirteen_horns", THIRTEEN_HORNS_FOLD);
}

criterion_group!(benches, simple_benchmark);
criterion_main!(benches);
