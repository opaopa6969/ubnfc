use ubnfc_front::selfhost::{decode_string, SourceOffsets};
use ubnfc_front::syntax::ast::Span;

#[test]
fn code_points_become_exact_utf8_boundaries_and_invalid_spans_are_errors() {
    let offsets = SourceOffsets::new("a😀文\r\nb");
    assert_eq!(
        offsets.byte_span([1, 3]).unwrap(),
        Span { start: 1, end: 8 }
    );
    assert_eq!(
        offsets.byte_span([6, 6]).unwrap(),
        Span { start: 11, end: 11 }
    );
    assert_eq!(offsets.text([1, 3]).unwrap(), "😀文");
    assert!(offsets.byte_span([3, 2]).is_err());
    assert!(offsets.byte_span([0, 7]).is_err());
}

#[test]
fn escapes_keep_unknown_sequences_and_literal_raw_data_is_not_changed() {
    let raw = r"'😀\n\t\r\'\\\d\q'";
    let value = decode_string(
        raw,
        Span {
            start: 0,
            end: raw.len(),
        },
    )
    .unwrap();
    assert_eq!(value, "😀\n\t\r'\\\\d\\q");
    assert!(decode_string("unquoted", Span::default()).is_err());
    assert!(decode_string("'unfinished\\'", Span::default()).is_err());
}
