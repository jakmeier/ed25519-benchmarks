use std::hint::black_box;

use ed25519_dalek_instrumented::{Keypair, PublicKey, Verifier};
use ed25519_dalek_instrumented::{Signature, Signer};
use once_cell::sync::OnceCell;
use rand::rngs::OsRng;
use rand::{CryptoRng, Rng, RngCore};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct BenchmarkInput {
    pub public_key: PublicKey,
    pub msg: String,
    pub signature: Signature,
}

const MESSAGE: &str = "This is a test of the tsunami alert system.";

impl BenchmarkInput {
    pub fn random<R: CryptoRng + RngCore>(csprng: &mut R) -> Self {
        let (keypair, signature) = random_signature(csprng);
        Self {
            public_key: keypair.public_key(),
            msg: MESSAGE.to_owned(),
            signature,
        }
    }

    pub fn forged(index: usize) -> Self {
        const FORGED: [[u8; 32]; 4] = [[255; 32], [0; 32], BYTES_A, BYTES_B];
        let (public_key, signature) = half_forged_signature(FORGED[index % FORGED.len()]);
        Self {
            public_key,
            msg: MESSAGE.to_owned(),
            signature,
        }
    }
}

const BYTES_A: [u8; 32] = [
    255, 042, 127, 042, 127, 010, 127, 042, 127, 042, 127, 042, 127, 042, 127, 042, 127, 042, 127,
    042, 127, 042, 255, 042, 127, 042, 127, 042, 127, 042, 127, 042,
];

const BYTES_B: [u8; 32] = [
    255, 042, 127, 042, 127, 010, 127, 042, 127, 042, 127, 042, 127, 042, 127, 042, 127, 042, 127,
    042, 127, 042, 255, 042, 127, 042, 127, 042, 127, 042, 127, 042,
];

#[allow(unused)]
fn main() {
    let mut csprng = OsRng::default();
    let n = 20000;

    for b in [
        [255; 32], [0; 32], BYTES_A, BYTES_A, BYTES_B, BYTES_A, BYTES_A,
    ] {
        let (pk, sign) = half_forged_signature(b);
        let durations_of_key = measure_n_verifications(n, pk, MESSAGE.as_bytes(), sign);
        print_confidence_intervals(&durations_of_key, 30.0, 40.0);
    }

    println!("start of random signatures");
    for _ in 0..30 {
        let (keypair, signature) = random_signature(&mut csprng);
        let durations_of_key =
            measure_n_verifications(n, keypair.public_key(), MESSAGE.as_bytes(), signature);
        // let (table_duration, other) =
        //     measure_n_verifications_timed(n, keypair.public_key(), MESSAGE, signature)
        //         .into_iter()
        //         .unzip();

        // print_confidence_intervals(&durations_of_key);
    }
}

pub fn half_forged_signature(s: [u8; 32]) -> (PublicKey, Signature) {
    let mut s_r = [0; 64];
    s_r[..32].copy_from_slice(&s);
    let pk = PublicKey::from_bytes(&[255; 32]).unwrap();
    let sign = Signature::from_bytes(&s_r).unwrap();
    (pk, sign)
}

pub fn random_signature<R>(csprng: &mut R) -> (Keypair, Signature)
where
    R: CryptoRng + RngCore,
{
    let keypair: Keypair = Keypair::generate(csprng);
    let signature: Signature = keypair.sign(MESSAGE.as_bytes());
    (keypair, signature)
}

fn print_confidence_intervals(durations_of_key: &[i32], left: f32, right: f32) {
    let n = durations_of_key.len();
    let sample_mean = mean(&durations_of_key).unwrap();
    let stddev = std_deviation(&durations_of_key).unwrap();
    let z = 1.44;
    let ci = z * stddev / f32::sqrt(n as f32);

    let small = sample_mean - ci;
    let big = sample_mean + ci;

    let width = 120;
    let step_width = 1.0 / width as f32 * (right - left);
    let mut printed = false;
    for i in 0..width {
        let pivot = left + i as f32 * step_width;
        let next_pivot = left + (i + 1) as f32 * step_width;
        if (pivot >= small && pivot < big) || (!printed && next_pivot > big) {
            printed = true;
            print!("■")
        } else {
            print!("□")
        }
    }
    // println!("{sample_mean:>3.2} us {stddev:>3.2} {private_key:?}");
    println!("   {small:>3.2}-{big:>3.2}");
}

fn measure_n_verifications(
    n: i32,
    pk: PublicKey,
    message: &[u8],
    signature: Signature,
) -> Vec<i32> {
    (0..n)
        .map(|_| {
            let ts = std::time::Instant::now();
            // pk.verify(message, &signature).expect("verification failed");
            _ = pk.verify(message, &signature);
            ts.elapsed().as_micros() as i32
        })
        .collect()
}

#[allow(unused)]
fn measure_n_verifications_timed(
    n: i32,
    pk: PublicKey,
    message: &[u8],
    signature: Signature,
) -> Vec<(i32, (i32, (i32, (i32, i32))))> {
    (0..n)
        .map(|_| {
            let ts = std::time::Instant::now();
            // pk.verify(message, &signature).expect("verification failed");
            let (table_duration, a_duration, b_duration, compress_dur) =
                pk.verify_timed(message, &signature).unwrap();
            (
                ts.elapsed().as_micros() as i32,
                (
                    table_duration.as_micros() as i32,
                    (
                        a_duration.as_micros() as i32,
                        (
                            b_duration.as_micros() as i32,
                            compress_dur.as_micros() as i32,
                        ),
                    ),
                ),
            )
        })
        .collect()
}

pub fn mean(data: &[i32]) -> Option<f32> {
    let sum = data.iter().sum::<i32>() as f32;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}

pub fn std_deviation(data: &[i32]) -> Option<f32> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data
                .iter()
                .map(|value| {
                    let diff = data_mean - (*value as f32);

                    diff * diff
                })
                .sum::<f32>()
                / count as f32;

            Some(variance.sqrt())
        }
        _ => None,
    }
}

#[derive(Clone, Copy)]
pub enum CacheFlushMode {
    /// Don't flush caches before measuring.
    None,
    /// Read random data until data caches are almost certainly free from useful
    /// data. Does nothing to instruction caches.
    Simple,
    /// Use the WBINV instruction to properly clear all caches, requires a
    /// kernel module (https://github.com/batmac/wbinvd).
    Wbinv,
}

pub fn byzantine_bench(function: impl Fn(), flush_config: CacheFlushMode) -> Vec<i32> {
    let repetitions = 100;
    (0..repetitions)
        .map(|_| {
            freeze(flush_config);
            let ts = std::time::Instant::now();
            function();
            ts.elapsed().as_nanos() as i32
        })
        .collect()
}

static DUMMY_DATA: OnceCell<Vec<u8>> = OnceCell::new();
// anti warm up
fn freeze(flush_config: CacheFlushMode) {
    match flush_config {
        CacheFlushMode::None => (),
        CacheFlushMode::Simple => {
            const ASSUMED_L3_SIZE: usize = 64 * 1024 * 1024;
            const FACTOR: usize = 2;
            let dummy_data = DUMMY_DATA.get_or_init(|| {
                let mut rng = rand::thread_rng();
                std::iter::repeat_with(|| rng.gen::<u8>())
                    .take(ASSUMED_L3_SIZE * FACTOR)
                    .collect()
            });
            let mut sink = 0;
            for i in 0..dummy_data.len() {
                *black_box(&mut sink) = black_box(dummy_data[i]);
            }
        }
        CacheFlushMode::Wbinv => {
            std::fs::read_to_string("/proc/wbinvd").unwrap();
        }
    }
}
