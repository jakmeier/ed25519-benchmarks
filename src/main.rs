#![allow(unused)]
use std::time::Instant;

use ed25519_dalek_instrumented::Verifier;
use ed25519_hacking::{byzantine_bench, mean, std_deviation, BenchmarkInput, CacheFlushMode};

mod input_generation;

// empirically found to be higher than average (71-72) but still quick to find
const BYZANTINE_THRESHOLD: usize = 78;

fn main() -> Result<(), std::io::Error> {
    // generate these once, then reuse the generated files for consistency
    // generate_inputs();

    // experimentation
    // input_generation::try_many_random()?;
    let cases = input_generation::generate_byzantine_samples(BYZANTINE_THRESHOLD + 2)?;
    std::fs::write("./res/very_hard_byz.ron", ron::to_string(&cases).unwrap())?;

    // full_byzantine_bench();

    Ok(())
}

fn generate_inputs() -> Result<(), std::io::Error> {
    let random_cases = input_generation::generate_random_10()?;
    std::fs::write(
        "./res/random_10.ron",
        ron::to_string(&random_cases).unwrap(),
    )?;

    let forged_cases = input_generation::generate_forged_10()?;
    std::fs::write(
        "./res/forged_10.ron",
        ron::to_string(&forged_cases).unwrap(),
    );

    let cases = input_generation::generate_byzantine_samples(BYZANTINE_THRESHOLD)?;
    std::fs::write("./res/byz.ron", ron::to_string(&cases).unwrap())?;
    Ok(())
}

#[test]
fn full_byzantine_bench() {
    const BYZ_10_PATH: &str = "./res/hard_byz.ron";
    let raw = std::fs::read_to_string(BYZ_10_PATH).unwrap();
    let byz_inputs: Vec<BenchmarkInput> =
        ron::from_str(&raw).expect("failed parsing benchmark inputs");

    for index in 0..10 {
        let input = byz_inputs[0].clone();
        let f = || {
            input
                .public_key
                .verify(input.msg.as_bytes(), &input.signature)
                .unwrap()
        };

        bench_mode(f, "no_flush", index, CacheFlushMode::None);
        bench_mode(f, "overwrite", index, CacheFlushMode::Simple);
        bench_mode(f, "wbinv", index, CacheFlushMode::Wbinv);
    }
}

fn bench_mode(f: impl Fn(), header: &str, index: usize, config: CacheFlushMode) {
    let t = Instant::now();
    let nanos = byzantine_bench(f, config);
    println!("{} (took {:#?})", header, t.elapsed());
    println!(
        "{:.2} +/-{:.2} {:?}",
        mean(&nanos).unwrap() / 1000.0,
        std_deviation(&nanos).unwrap() / 1000.0,
        nanos
    );
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&format!("./results/{header}{index}.json"))
        .unwrap();
    let data = serde_json::to_writer(file, &nanos);
}
