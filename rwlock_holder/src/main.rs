pub mod holder;
use chrono::prelude::*;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crate::holder::Holder;

pub static MAP: OnceLock<Holder<String, String>> = OnceLock::new();

pub fn setup(expire_interval: std::time::Duration) {
    MAP.get_or_init(|| Holder::new(expire_interval));
}

const KEY_SPACE: usize = 1024;
const OPS_PER_THREAD: usize = 200_000;

fn key_for(i: usize) -> String {
    format!("key-{:04}", i % KEY_SPACE)
}

fn prepopulate() {
    let now = Utc::now();
    let map = MAP.get().unwrap();
    for i in 0..KEY_SPACE {
        map.insert(key_for(i), format!("value-{}", i), now);
    }
}

fn run_scenario(name: &str, reader_threads: usize, writer_threads: usize) {
    let total_reads = AtomicUsize::new(0);
    let total_writes = AtomicUsize::new(0);

    let start = Instant::now();
    std::thread::scope(|scope| {
        for t in 0..reader_threads {
            let total_reads = &total_reads;
            scope.spawn(move || {
                let map = MAP.get().unwrap();
                let mut local_seed = t.wrapping_mul(2654435761);
                for _ in 0..OPS_PER_THREAD {
                    local_seed = local_seed.wrapping_mul(1103515245).wrapping_add(12345);
                    let key = key_for(local_seed);
                    let _ = map.get(&key, Utc::now());
                }
                total_reads.fetch_add(OPS_PER_THREAD, Ordering::Relaxed);
            });
        }

        for t in 0..writer_threads {
            let total_writes = &total_writes;
            scope.spawn(move || {
                let map = MAP.get().unwrap();
                let mut local_seed = (t + 1000).wrapping_mul(2654435761);
                for i in 0..OPS_PER_THREAD {
                    local_seed = local_seed.wrapping_mul(1103515245).wrapping_add(12345);
                    let key = key_for(local_seed);
                    map.insert(key, format!("v-{}-{}", t, i), Utc::now());
                }
                total_writes.fetch_add(OPS_PER_THREAD, Ordering::Relaxed);
            });
        }
    });
    let elapsed = start.elapsed();

    let reads = total_reads.load(Ordering::Relaxed);
    let writes = total_writes.load(Ordering::Relaxed);
    let total_ops = reads + writes;
    let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();
    println!(
        "[{:>20}] readers={:>2} writers={:>2} reads={:>8} writes={:>8} elapsed={:>8.3?} throughput={:>12.0} ops/s",
        name, reader_threads, writer_threads, reads, writes, elapsed, ops_per_sec
    );
}

fn main() {
    // expire interval is set long enough that insert always overwrites and
    // get always finds a fresh entry — we are measuring pure lock contention.
    setup(std::time::Duration::from_secs(3600));
    prepopulate();

    println!("=== RwLock Holder Benchmark ===");
    println!("KEY_SPACE={}, OPS_PER_THREAD={}", KEY_SPACE, OPS_PER_THREAD);

    // warm-up
    run_scenario("warmup", 2, 1);

    // read-heavy: where RwLock should win in theory
    run_scenario("read-heavy 8r/0w", 8, 0);
    run_scenario("read-heavy 8r/1w", 8, 1);
    run_scenario("read-heavy 16r/1w", 16, 1);

    // mixed
    run_scenario("mixed 4r/4w", 4, 4);
    run_scenario("mixed 8r/8w", 8, 8);

    // write-heavy: lock type matters less, mostly serialized
    run_scenario("write-heavy 0r/8w", 0, 8);
    run_scenario("write-heavy 1r/8w", 1, 8);
}
