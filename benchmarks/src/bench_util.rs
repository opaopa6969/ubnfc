// 速度比較の共通計測部（`include!` で各 family の bench から取り込む）。
//
// - 計時は `std::time::Instant`、単一 thread。
// - 1 run の反復数は 1 parse の所要から `UBNFC_CMP_RUN_SECONDS`（既定 0.5 秒）を満たすよう決める。
// - run 数は `UBNFC_CMP_RUNS`（既定 7）。中央値・最小・最大を出す。
// - 確保回数と byte は GlobalAlloc を数える別 parse で採り、計時には含めない。

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

#[allow(dead_code)]
pub struct Counting;
static ACTIVE: AtomicBool = AtomicBool::new(false);
static COUNT: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if ACTIVE.load(Relaxed) {
            COUNT.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size(), Relaxed);
        }
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        if ACTIVE.load(Relaxed) {
            COUNT.fetch_add(1, Relaxed);
            BYTES.fetch_add(n, Relaxed);
        }
        unsafe { System.realloc(p, l, n) }
    }
}

#[allow(dead_code)]
fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}
#[allow(dead_code)]
fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

#[allow(dead_code)]
pub fn header() {
    println!("family,impl,input,bytes,mode,iterations,runs,median_ms,min_ms,max_ms,us_per_byte,allocs_per_parse,alloc_bytes_per_parse");
}

/// `parse` を計測して 1 行出す。`parse` は毎回新しい解析を行い、結果を破棄する。
#[allow(dead_code)]
pub fn row(family: &str, name: &str, input: &str, bytes: usize, mode: &str, mut parse: impl FnMut()) {
    let run_seconds = env_f64("UBNFC_CMP_RUN_SECONDS", 0.5);
    let runs = env_usize("UBNFC_CMP_RUNS", 7).max(3);
    let once = Instant::now();
    parse();
    let per = once.elapsed().as_secs_f64().max(1e-9);
    let iterations = ((run_seconds / per) as usize).clamp(3, 200_000);
    let warmup = (iterations / 2).clamp(3, 2_000);
    for _ in 0..warmup {
        parse();
    }
    let mut samples = vec![0.0f64; runs];
    for sample in &mut samples {
        let start = Instant::now();
        for _ in 0..iterations {
            parse();
        }
        *sample = start.elapsed().as_secs_f64() * 1e3 / iterations as f64;
    }
    samples.sort_by(f64::total_cmp);
    COUNT.store(0, Relaxed);
    BYTES.store(0, Relaxed);
    ACTIVE.store(true, Relaxed);
    parse();
    ACTIVE.store(false, Relaxed);
    let (allocs, alloc_bytes) = (COUNT.load(Relaxed), BYTES.load(Relaxed));
    let median = samples[runs / 2];
    println!(
        "{family},{name},{input},{bytes},{mode},{iterations},{runs},{median:.6},{:.6},{:.6},{:.4},{allocs},{alloc_bytes}",
        samples[0],
        samples[runs - 1],
        median * 1e3 / bytes.max(1) as f64,
    );
    black_box(());
}
