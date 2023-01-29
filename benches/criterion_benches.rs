use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ed25519_dalek_instrumented::Verifier;
use ed25519_hacking::BenchmarkInput;

const RANDOM_10_PATH: &str = "./res/random_10.ron";
// const FORGED_10_PATH: &str = "./res/forged_10.ron";
const BYZ_10_PATH: &str = "./res/hard_byz.ron";

pub fn criterion_benchmark(c: &mut Criterion) {
    let raw = std::fs::read_to_string(RANDOM_10_PATH).unwrap();
    let random_inputs: Vec<BenchmarkInput> =
        ron::from_str(&raw).expect("failed parsing benchmark inputs");
    // let raw = std::fs::read_to_string(FORGED_10_PATH).unwrap();
    // let forged_inputs: Vec<BenchmarkInput> =
    // ron::from_str(&raw).expect("failed parsing benchmark inputs");

    // // Measure all 10 random input together
    // c.bench_with_input(
    //     BenchmarkId::new("traditional_single", "batch of 10"),
    //     &random_inputs,
    //     |b, s| {
    //         b.iter(|| {
    //             for input in s {
    //                 input
    //                     .public_key
    //                     .verify(input.msg.as_bytes(), &input.signature)
    //                     .unwrap();
    //             }
    //         });
    //     },
    // );

    // Measure each of the 10 separately
    let mut group = c.benchmark_group("traditional_group");
    for (i, input) in random_inputs.iter().enumerate() {
        group
            .throughput(criterion::Throughput::Elements(1))
            .bench_with_input(BenchmarkId::from_parameter(i), input, |b, input| {
                b.iter(|| {
                    input
                        .public_key
                        .verify(input.msg.as_bytes(), &input.signature)
                        .unwrap();
                });
            });
    }
    group.finish();

    let raw = std::fs::read_to_string(BYZ_10_PATH).unwrap();
    let byz_inputs: Vec<BenchmarkInput> =
        ron::from_str(&raw).expect("failed parsing benchmark inputs");

    let mut group = c.benchmark_group("byz_samples_warmup");
    for (i, input) in byz_inputs.iter().enumerate() {
        group
            .throughput(criterion::Throughput::Elements(1))
            .bench_with_input(BenchmarkId::from_parameter(i), input, |b, input| {
                b.iter(|| {
                    input
                        .public_key
                        .verify(input.msg.as_bytes(), &input.signature)
                        .unwrap();
                });
            });
    }
    group.finish();

    let mut c = std::mem::take(c)
        .warm_up_time(Duration::from_nanos(1))
        .sample_size(10)
        .measurement_time(Duration::from_micros(10));

    let mut group = c.benchmark_group("byz_samples_nowarmup");
    for (i, input) in byz_inputs.iter().enumerate() {
        group
            .throughput(criterion::Throughput::Elements(1))
            .bench_with_input(BenchmarkId::from_parameter(i), input, |b, input| {
                b.iter(|| {
                    input
                        .public_key
                        .verify(input.msg.as_bytes(), &input.signature)
                        .unwrap();
                });
            });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
