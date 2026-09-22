#![allow(dead_code)]
mod generated;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
struct Counting;
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
#[global_allocator]
static ALLOC: Counting = Counting;
fn main() {
    println!("implementation,input,ast,alloc_calls,allocated_bytes,memo_entries,recipes");
    for (name, input) in [
        ("short", "1+2*3".to_owned()),
        ("depth8", format!("{}1+2*3{}", "(".repeat(8), ")".repeat(8))),
        ("atoms512", vec!["1"; 512].join("+")),
    ] {
        for ast in [false, true] {
            for target in [false, true] {
                COUNT.store(0, Relaxed);
                BYTES.store(0, Relaxed);
                ACTIVE.store(true, Relaxed);
                let (entries, recipes) = if target {
                    let r = ubnfc_target_rust::operators::parse_with_options(
                        &input,
                        ubnfc_target_rust::ParseOptions {
                            build_ast: ast,
                            require_eof: true,
                        },
                    );
                    (r.statistics.memo_entries, r.statistics.recipes)
                } else {
                    let r = generated::parse_with_options(
                        &input,
                        generated::ParseOptions {
                            build_ast: ast,
                            require_eof: true,
                            lexical: false,
                        },
                    );
                    (r.statistics.memo_entries, r.statistics.recipes)
                };
                ACTIVE.store(false, Relaxed);
                println!(
                    "{},{name},{ast},{},{},{entries},{recipes}",
                    if target { "handwritten" } else { "generated" },
                    COUNT.load(Relaxed),
                    BYTES.load(Relaxed)
                );
            }
        }
    }
}
