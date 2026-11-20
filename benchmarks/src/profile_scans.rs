// 通常の時間 A/B とは別の、走査関数への補助計装。
use std::hint::black_box;
fn main() {
    let text = std::fs::read_to_string(std::env::args().nth(1).expect("入力ファイル")).unwrap();
    let parse = || {
        let result = json_after::parse_with_options(
            black_box(&text),
            json_after::ParseOptions {
                build_ast: false,
                ..Default::default()
            },
        );
        assert!(result.ok);
        black_box(result);
    };
    for _ in 0..10 {
        parse();
    }
    json_after::scan_profile::clear();
    for _ in 0..10 {
        parse();
    }
    json_after::scan_profile::report();
}
