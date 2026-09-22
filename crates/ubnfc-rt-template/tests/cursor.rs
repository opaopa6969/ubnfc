use ubnfc_rt_template::cursor::Cursor;

#[test]
fn advancing_preserves_the_lookahead_gap() {
    let mut cursor = Cursor::default();
    cursor.advance(3);
    cursor.advance_matched_only(4);
    assert_eq!(
        cursor,
        Cursor {
            consumed: 3,
            matched: 7
        }
    );
    cursor.advance(2);
    assert_eq!(
        cursor,
        Cursor {
            consumed: 5,
            matched: 9
        }
    );
    cursor.reset_matched_to_consumed();
    assert_eq!(
        cursor,
        Cursor {
            consumed: 5,
            matched: 5
        }
    );
}

#[test]
fn checkpoints_restore_both_positions() {
    let mut cursor = Cursor {
        consumed: 2,
        matched: 6,
    };
    let outer = cursor.checkpoint();
    cursor.advance(3);
    let inner = cursor.checkpoint();
    cursor.reset_matched_to_consumed();
    cursor.advance_matched_only(4);
    cursor.restore(inner);
    assert_eq!(
        cursor,
        Cursor {
            consumed: 5,
            matched: 9
        }
    );
    cursor.restore(outer);
    assert_eq!(
        cursor,
        Cursor {
            consumed: 2,
            matched: 6
        }
    );
}

#[test]
fn zero_width_operations_preserve_progress() {
    let mut cursor = Cursor {
        consumed: 1,
        matched: 3,
    };
    let before = cursor;
    cursor.advance(0);
    cursor.advance_matched_only(0);
    assert_eq!(cursor, before);
}

#[test]
fn overflow_does_not_partially_advance() {
    let mut cursor = Cursor {
        consumed: 1,
        matched: usize::MAX,
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cursor.advance(1)));
    assert!(result.is_err());
    assert_eq!(
        cursor,
        Cursor {
            consumed: 1,
            matched: usize::MAX
        }
    );
}
