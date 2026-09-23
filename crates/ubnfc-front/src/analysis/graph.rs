/// Tarjan。出力順は依存先 SCC → 依存元 SCC。各 SCC 内は入力順。
///
/// 明示スタックで書く。規則参照グラフは規則数ぶんの深さになりうる
/// （`A ::= B; B ::= C; …` の連鎖）ので、再帰では 16,000 段で stack overflow した。
/// 走査順・`low`/`index` の更新順は再帰版と同じなので、出力は完全に一致する。
pub(super) fn scc(graph: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = graph.len();
    let mut index: Vec<Option<usize>> = vec![None; n];
    let mut low = vec![0usize; n];
    let mut active = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut result: Vec<Vec<usize>> = Vec::new();
    let mut next = 0usize;
    // (頂点, 次に辿る出辺の添字) = 再帰版の 1 フレーム。
    let mut frames: Vec<(usize, usize)> = Vec::new();
    for root in 0..n {
        if index[root].is_some() {
            continue;
        }
        index[root] = Some(next);
        low[root] = next;
        next += 1;
        stack.push(root);
        active[root] = true;
        frames.push((root, 0));
        while let Some(&(v, edge)) = frames.last() {
            if let Some(&w) = graph[v].get(edge) {
                frames.last_mut().expect("frame は直前に確認済み").1 += 1;
                if index[w].is_none() {
                    index[w] = Some(next);
                    low[w] = next;
                    next += 1;
                    stack.push(w);
                    active[w] = true;
                    frames.push((w, 0));
                } else if active[w] {
                    low[v] = low[v].min(index[w].expect("index は Some"));
                }
                continue;
            }
            frames.pop();
            if low[v] == index[v].expect("index は Some") {
                let mut component = Vec::new();
                loop {
                    let w = stack.pop().expect("成分の要素は stack 上にある");
                    active[w] = false;
                    component.push(w);
                    if w == v {
                        break;
                    }
                }
                component.sort_unstable();
                result.push(component);
            }
            // 再帰版の `self.visit(w)` から戻った直後の low 伝播に対応する。
            if let Some(&(parent, _)) = frames.last() {
                low[parent] = low[parent].min(low[v]);
            }
        }
    }
    result
}
