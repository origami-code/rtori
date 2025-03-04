#![feature(allocator_api)]

use core::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};

extern crate alloc;
use pollster::FutureExt as _;
use rtori_os_fold_importer::{import_in, supplement::transform_in};

const SIMPLE_FOLD: &'static str = include_str!("../../fold/testdata/simple.fold");
const THIRTEEN_HORNS_FOLD: &'static str =
    include_str!("../../fold/testdata/13-horns-123-vertices-augmented-triangulated.fold");

fn bench_source(c: &mut Criterion, name: &str, fold_str: &str, step_count: u32) {
    let parsed_input = serde_json::from_str::<fold::File>(fold_str)
        .expect("source deserialization (json/fold file) failed");

    let allocator = alloc::alloc::Global;
    let mut solver =
        rtori_core::os_solver::Solver::create(rtori_core::os_solver::BackendFlags::CPU)
            .block_on()
            .unwrap();

    solver.load_fold_in(&parsed_input.key_frame, allocator);

    let mut group = c.benchmark_group("stepping");
    group.sample_size(500);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(criterion::Throughput::Elements(u64::from(step_count)));
    group.bench_function(name, |b| b.iter(|| black_box(solver.step(step_count))));
}

fn simple_benchmark(c: &mut Criterion) {
    {
        const STEP_COUNT: u32 = 100;
        bench_source(
            c,
            &format!("step_simple_{STEP_COUNT}_step"),
            SIMPLE_FOLD,
            STEP_COUNT,
        );
    }
    {
        const STEP_COUNT: u32 = 1;
        bench_source(
            c,
            &format!("step_thirteen_horns_{STEP_COUNT}_step"),
            THIRTEEN_HORNS_FOLD,
            STEP_COUNT,
        );
    }
}

criterion_group!(benches, simple_benchmark);
criterion_main!(benches);
