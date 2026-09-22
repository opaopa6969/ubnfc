use ubnfc_rt_template::input::Input;

#[test]
fn ascii_offsets_are_identity_including_eof() {
    for text in ["", "abc\0\n XYZ"] {
        let input = Input::new(text);
        assert_eq!(input.bytes(), text.as_bytes());
        for offset in 0..=text.len() {
            assert_eq!(input.byte_to_cp(offset), Some(offset));
            assert_eq!(input.cp_to_byte(offset), Some(offset));
        }
        assert_eq!(input.cp_len(0, text.len()), Some(text.len()));
        assert_eq!(input.byte_to_cp(text.len() + 1), None);
        assert_eq!(input.cp_to_byte(text.len() + 1), None);
    }
}

#[test]
fn mixed_unicode_offsets_round_trip_at_every_boundary() {
    for text in ["aé日😀z", "😀😀", "éabc😀z日", "e\u{301}\0\u{10ffff}"] {
        let input = Input::new(text);
        let mut boundaries: Vec<_> = text.char_indices().map(|(byte, _)| byte).collect();
        boundaries.push(text.len());
        for (cp, &byte) in boundaries.iter().enumerate() {
            assert_eq!(input.byte_to_cp(byte), Some(cp));
            assert_eq!(input.cp_to_byte(cp), Some(byte));
            for (end_cp, &end_byte) in boundaries.iter().enumerate().skip(cp) {
                assert_eq!(input.cp_len(byte, end_byte), Some(end_cp - cp));
            }
        }
        for byte in 0..text.len() {
            if !text.is_char_boundary(byte) {
                assert_eq!(input.byte_to_cp(byte), None);
                assert_eq!(input.cp_at(byte), None);
                assert!(!input.starts_with(byte, ""));
                assert!(!input.starts_with_ignore_case(byte, ""));
            }
        }
        assert_eq!(input.cp_to_byte(boundaries.len()), None);
    }
}

#[test]
fn code_point_access_and_invalid_spans() {
    let input = Input::new("aé日😀z");
    for (byte, expected) in [
        (0, ('a', 1)),
        (1, ('é', 2)),
        (3, ('日', 3)),
        (6, ('😀', 4)),
        (10, ('z', 1)),
    ] {
        assert_eq!(input.cp_at(byte), Some(expected));
    }
    assert_eq!(input.cp_at(11), None);
    assert_eq!(input.cp_at(usize::MAX), None);
    assert_eq!(input.byte_to_cp(usize::MAX), None);
    assert_eq!(input.cp_to_byte(usize::MAX), None);
    assert_eq!(input.cp_len(6, 1), None);
    assert_eq!(input.cp_len(2, 10), None);
    assert_eq!(input.cp_len(1, 9), None);
    assert_eq!(input.cp_len(0, 12), None);
}

#[test]
fn literals_match_prefixes_without_word_boundaries() {
    let input = Input::new("😀letter");
    assert!(input.starts_with(0, "😀"));
    assert!(input.starts_with(4, "let"));
    assert!(!input.starts_with(4, "LET"));
    assert!(!input.starts_with(4, "letters"));
    assert!(input.starts_with(10, ""));
    assert!(!input.starts_with(11, ""));
    assert!(!input.starts_with(usize::MAX, ""));
}

#[test]
fn ignore_case_handles_java_style_scalar_comparisons() {
    for (left, right) in [
        ("Letter", "let"),
        ("Σ", "ς"),
        ("σ", "ς"),
        ("I", "ı"),
        ("İ", "i"),
        ("İ", "ı"),
        ("K", "k"),
        ("ſ", "s"),
        ("ß", "ẞ"),
        ("𐐀", "𐐨"),
    ] {
        assert!(
            Input::new(left).starts_with_ignore_case(0, right),
            "{left} / {right}"
        );
    }
    let input = Input::new("😀Éx");
    assert!(input.starts_with_ignore_case(4, "é"));
    assert!(input.starts_with_ignore_case(7, ""));
    assert!(!input.starts_with_ignore_case(7, "x"));
    assert!(!input.starts_with_ignore_case(8, ""));
    assert!(!input.starts_with_ignore_case(usize::MAX, ""));
}

#[test]
fn ignore_case_does_not_expand_normalize_or_skip_characters() {
    for (left, right) in [
        ("ß", "SS"),
        ("SS", "ß"),
        ("ﬀ", "ff"),
        ("ﬀ", "f"),
        ("é", "e\u{301}"),
        ("a", "ab"),
        ("😀", "😁"),
    ] {
        assert!(
            !Input::new(left).starts_with_ignore_case(0, right),
            "{left} / {right}"
        );
    }
}
