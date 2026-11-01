use std::hint::black_box;
extern "C" {
    fn ubnfc_profile_start();
    fn ubnfc_profile_stop();
}
#[inline(never)]
fn measured(parse: &dyn Fn(), n: usize) {
    for _ in 0..n {
        parse();
    }
}
fn main() {
    let a: Vec<_> = std::env::args().collect();
    let text = std::fs::read_to_string(&a[2]).unwrap();
    let parse: Box<dyn Fn()> = if a[1] == "before" {
        Box::new(|| {
            let r = json_before::parse_with_options(
                black_box(&text),
                json_before::ParseOptions {
                    build_ast: false,
                    ..Default::default()
                },
            );
            assert!(r.ok);
            black_box(r);
        })
    } else {
        Box::new(|| {
            let r = json_after::parse_with_options(
                black_box(&text),
                json_after::ParseOptions {
                    build_ast: false,
                    ..Default::default()
                },
            );
            assert!(r.ok);
            black_box(r);
        })
    };
    for _ in 0..100 {
        parse();
    }
    unsafe {
        ubnfc_profile_start();
    }
    measured(&*parse, a[3].parse().unwrap());
    unsafe {
        ubnfc_profile_stop();
    }
}
