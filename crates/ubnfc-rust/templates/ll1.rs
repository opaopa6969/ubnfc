// LL(1) 認識専用 VM。失敗・予算超過では Session を変更せず参照経路へ戻す。
type Ll1Ranges = &'static [(u32, u32)];
#[derive(Clone, Copy)]
enum Ll1Op {
    Sequence(&'static [usize]),
    Choice(&'static [(Ll1Ranges, usize)]),
    Optional(usize),
    Repeat(usize, u64, Option<u64>),
    Call(usize),
    Literal(&'static str, bool),
    Class(Ll1Ranges),
    Eof,
}
#[derive(Clone, Copy)]
enum Ll1Frame {
    Sequence(&'static [usize], usize),
    Optional(usize),
    Repeat { target: usize, min: u64, max: Option<u64>, count: u64, last: usize },
    Call,
}
fn ll1_contains(ranges: Ll1Ranges, cp: u32) -> bool {
    ranges.iter().any(|&(lo, hi)| lo <= cp && cp <= hi)
}
impl<const DIAG: bool> Session<'_, DIAG> {
    pub fn ll1_run(&self, root: usize, start: usize, scan: impl Fn(usize,usize)->Option<Option<usize>>) -> Option<usize> {
        let mut p = start;
        let mut current = root;
        let mut frames = Vec::with_capacity(32);
        let mut calls = 0usize;
        'enter: loop {
            // 明示 stack の作業量を制限。深い再帰の資源診断は従来経路で確定する。
            if frames.len() >= 256 { return None; }
            let mut ok = true;
            if let Some(end) = scan(current, p) {
                match end { Some(end) => p = end, None => ok = false }
            } else {
            match LL1_TABLE[current] {
                Ll1Op::Sequence(items) => {
                    if let Some(&first) = items.first() {
                        frames.push(Ll1Frame::Sequence(items, 1));
                        current = first;
                        continue;
                    }
                }
                Ll1Op::Choice(rows) => {
                    let cp = self.input.cp_at(p).map(|(c, _)| c as u32);
                    if let Some((_, target)) = rows.iter().find(|(rs, _)| cp.is_some_and(|c| ll1_contains(rs, c))) {
                        current = *target;
                        continue;
                    }
                    ok = false;
                }
                Ll1Op::Optional(target) => {
                    frames.push(Ll1Frame::Optional(p));
                    current = target;
                    continue;
                }
                Ll1Op::Repeat(target, min, max) => {
                    if max == Some(0) { ok = min == 0; }
                    else {
                        frames.push(Ll1Frame::Repeat { target, min, max, count: 0, last: p });
                        current = target;
                        continue;
                    }
                }
                Ll1Op::Call(target) => {
                    if calls >= 32 || self.depth.saturating_add(calls) >= self.options.max_depth { return None; }
                    calls += 1;
                    frames.push(Ll1Frame::Call);
                    current = target;
                    continue;
                }
                Ll1Op::Literal(word, boundary) => {
                    ok = self.input.starts_with(p, word)
                        && !(boundary && self.input.identifier_byte_at(p + word.len()));
                    if ok { p += word.len(); }
                }
                Ll1Op::Class(ranges) => {
                    if let Some((c, n)) = self.input.cp_at(p).filter(|(c, _)| ll1_contains(ranges, *c as u32)) {
                        let _ = c;
                        p += n;
                    } else { ok = false; }
                }
                Ll1Op::Eof => ok = p == self.text.len(),
            }
            }
            loop {
                match frames.pop() {
                    None => return ok.then_some(p),
                    Some(Ll1Frame::Call) => calls -= 1,
                    Some(Ll1Frame::Sequence(items, next)) => {
                        if ok && next < items.len() {
                            frames.push(Ll1Frame::Sequence(items, next + 1));
                            current = items[next];
                            continue 'enter;
                        }
                    }
                    Some(Ll1Frame::Optional(start)) => {
                        if !ok { p = start; ok = true; }
                    }
                    Some(Ll1Frame::Repeat { target, min, max, count, last }) => {
                        if ok {
                            if p == last { return None; }
                            let count = count + 1;
                            if max.is_none_or(|max| count < max) {
                                frames.push(Ll1Frame::Repeat { target, min, max, count, last: p });
                                current = target;
                                continue 'enter;
                            }
                            ok = count >= min;
                        } else {
                            p = last;
                            ok = count >= min;
                        }
                    }
                }
            }
        }
    }
}
