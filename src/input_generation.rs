use ed25519_hacking::BenchmarkInput;
use rand::rngs::OsRng;
use std::iter;

pub(crate) fn generate_random_10() -> Result<Vec<BenchmarkInput>, std::io::Error> {
    let num_sigs = 10;
    let mut csprng = OsRng::default();
    let cases: Vec<_> = iter::repeat_with(|| BenchmarkInput::random(&mut csprng))
        .take(num_sigs)
        .collect();
    Ok(cases)
}

pub(crate) fn generate_forged_10() -> Result<Vec<BenchmarkInput>, std::io::Error> {
    let num_sigs = 10;
    let cases: Vec<_> = (0..num_sigs).map(|i| BenchmarkInput::forged(i)).collect();
    Ok(cases)
}

pub(crate) fn generate_byzantine_samples(
    byzantine_threshold: usize,
) -> Result<Vec<BenchmarkInput>, std::io::Error> {
    let mut cases: Vec<_> = vec![];
    let mut csprng = OsRng::default();

    for _ in 0..10 {
        let input = find_byz_input(&mut csprng, byzantine_threshold);
        cases.push(input);
    }

    Ok(cases)
}

/// Try random inputs and print the, with their byzantine score in one line each.
#[allow(unused)]
pub(crate) fn try_many_random() -> Result<(), std::io::Error> {
    let mut csprng = OsRng::default();
    for _ in 0..100 {
        let min_score = 1;
        let input = find_byz_input(&mut csprng, min_score);
    }

    Ok(())
}

/// Try random inputs until one with a byzantine score of at least `min_score` is found.
pub(crate) fn find_byz_input(csprng: &mut OsRng, min_score: usize) -> BenchmarkInput {
    let mut score = 0;
    let mut input = BenchmarkInput::random(csprng);
    while score < min_score {
        input = BenchmarkInput::random(csprng);
        score = input
            .public_key
            .verify_byz_score(input.msg.as_bytes(), &input.signature);
    }

    let verbose = true;
    if verbose {
        let one_line_input = format!("{input:?}").replace("\n", "");
        println!("{score} {one_line_input}")
    } else {
        println!("{score}")
    }

    input
}
