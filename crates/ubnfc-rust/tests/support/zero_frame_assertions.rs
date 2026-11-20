#[cfg(test)]
mod zero_frames {
    use super::*;
    use super::super::rt::cursor::State;
    fn check<const DIAG: bool>() {
        for reset in [false, true] { for invert in [false, true] { for memo in [false, true] {
            let mut scanner = RejectExtern;
            let mut session = Session::<DIAG>::new("abc", ParseOptions { memo, ..Default::default() }, &mut scanner, usize::MAX);
            let state = State { consumed: 1, matched: 2, reset, invert };
            for _ in 0..2 {
                let step = session.e0_c(state);
                assert!(step.ok);
                assert_eq!(step.state, state);
                assert_eq!(step.diag, Diag::NONE);
            }
            // 本体を直接通しても、begin・子の event・診断を作らない。
            let step = session.b0_c(state);
            assert!(step.ok);
            assert_eq!(step.state, state);
            assert_eq!(step.events.0, 0);
            assert_eq!(step.diag, Diag::NONE);
        } } }
    }
    #[test]
    fn zero_max_preserves_stale_matched_reset_inversion_and_memo_replay() {
        check::<false>(); check::<true>();
    }
}
