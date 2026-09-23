#[cfg(test)]
mod scanner_state_regression {
    use super::*;
    #[test]
    fn both_modes_and_both_values_of_every_cursor_flag() {
        struct Scanner {
            expected: State,
            mode: bool,
            calls: usize,
        }
        impl TokenScanner for Scanner {
            fn scan(
                &mut self,
                _: &str,
                _: &str,
                state: State,
                mode: bool,
                _: &super::super::rt::scope::ScopeStore,
            ) -> ScanResult {
                assert_eq!(state, self.expected);
                assert_eq!(mode, self.mode);
                self.calls += 1;
                ScanResult {
                    ok: true,
                    consumed_end: state.consumed,
                    matched_end: state.matched,
                    value_span: [1, 1],
                    diagnostics: vec![],
                    effects: vec![],
                }
            }
        }
        for mode in [false, true] {
            for invert in [false, true] {
                for reset in [false, true] {
                    let state = State {
                        consumed: 1,
                        matched: 2,
                        invert,
                        reset,
                    };
                    let mut scanner = Scanner {
                        expected: state,
                        mode,
                        calls: 0,
                    };
                    // 診断は const DIAG で単相化されている（D-027）。fast 側で確かめる。
                    let mut session =
                        Session::<false>::new("abc", Default::default(), &mut scanner);
                    let out = if mode {
                        session.token_0::<true>(state, "T", "T")
                    } else {
                        session.token_0::<false>(state, "T", "T")
                    };
                    assert!(out.ok);
                    assert_eq!(out.state, state);
                    assert_eq!(scanner.calls, 1);
                }
            }
        }
    }
}
