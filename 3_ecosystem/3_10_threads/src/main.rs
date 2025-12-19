use crossbeam_channel::{Receiver, Sender, bounded};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use rayon::prelude::*;
use std::thread;

const DEFAULT_MATRIX_SIZE: usize = 4096;
const DEFAULT_ITERATIONS: usize = 3;
const DEFAULT_CONSUMERS: usize = 2;

#[derive(Debug, Clone)]
struct Config {
    matrix_size: usize,
    iterations: usize,
    consumer_count: usize,
    rng_seed: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            matrix_size: DEFAULT_MATRIX_SIZE,
            iterations: DEFAULT_ITERATIONS,
            consumer_count: DEFAULT_CONSUMERS,
            rng_seed: None,
        }
    }
}

fn main() {
    let results = run_pipeline(Config::default());
    for (idx, sum) in results.iter().enumerate() {
        println!("Matrix #{idx}: sum = {sum}");
    }
}

fn run_pipeline(config: Config) -> Vec<u64> {
    let (tx, rx) = bounded::<Option<Vec<u8>>>(config.consumer_count * 2);

    let producer = spawn_producer(config.clone(), tx);
    let consumers = spawn_consumers(config.consumer_count, rx);

    producer
        .join()
        .expect("producer panicked while generating matrices");

    let mut results = Vec::with_capacity(config.iterations);
    for consumer in consumers {
        let mut partial = consumer
            .join()
            .expect("consumer panicked while processing matrices");
        results.append(&mut partial);
    }

    results
}

fn spawn_producer(config: Config, tx: Sender<Option<Vec<u8>>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut rng = create_rng(config.rng_seed);
        let matrix_len = config
            .matrix_size
            .checked_mul(config.matrix_size)
            .expect("matrix size overflow");

        for _ in 0..config.iterations {
            let mut matrix = vec![0u8; matrix_len];
            rng.fill_bytes(&mut matrix);
            tx.send(Some(matrix)).expect("channel closed unexpectedly");
        }

        for _ in 0..config.consumer_count {
            tx.send(None).expect("channel closed unexpectedly");
        }
    })
}

fn spawn_consumers(
    consumer_count: usize,
    rx: Receiver<Option<Vec<u8>>>,
) -> Vec<thread::JoinHandle<Vec<u64>>> {
    (0..consumer_count)
        .map(|_| {
            let rx = rx.clone();
            thread::spawn(move || {
                let mut sums = Vec::new();
                while let Ok(message) = rx.recv() {
                    match message {
                        Some(matrix) => sums.push(parallel_sum(&matrix)),
                        None => break,
                    }
                }
                sums
            })
        })
        .collect()
}

fn parallel_sum(matrix: &[u8]) -> u64 {
    matrix
        .par_chunks(2048)
        .map(|chunk| chunk.iter().map(|&byte| byte as u64).sum::<u64>())
        .sum()
}

fn create_rng(seed: Option<u64>) -> Box<dyn RngCore + Send> {
    match seed {
        Some(value) => Box::new(StdRng::seed_from_u64(value)),
        None => Box::new(StdRng::from_entropy()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{RngCore, SeedableRng};

    fn expected_sums(matrix_size: usize, iterations: usize, seed: u64) -> Vec<u64> {
        let mut rng = StdRng::seed_from_u64(seed);
        let len = matrix_size * matrix_size;
        let mut sums = Vec::with_capacity(iterations);
        for _ in 0..iterations {
            let mut matrix = vec![0u8; len];
            rng.fill_bytes(&mut matrix);
            let sum: u64 = matrix.iter().map(|&b| b as u64).sum();
            sums.push(sum);
        }
        sums
    }

    #[test]
    fn processes_all_matrices() {
        let config = Config {
            matrix_size: 8,
            iterations: 5,
            consumer_count: 2,
            rng_seed: Some(42),
        };

        let results = run_pipeline(config.clone());
        let mut expected = expected_sums(config.matrix_size, config.iterations, 42);

        assert_eq!(results.len(), config.iterations);
        expected.sort_unstable();
        let mut actual = results.clone();
        actual.sort_unstable();
        assert_eq!(actual, expected);
    }

    #[test]
    fn uses_multiple_consumers() {
        let config = Config {
            matrix_size: 4,
            iterations: 4,
            consumer_count: 2,
            rng_seed: Some(7),
        };

        let results = run_pipeline(config.clone());
        assert_eq!(results.len(), config.iterations);
        assert!(results.iter().all(|sum| *sum > 0));
    }
}
