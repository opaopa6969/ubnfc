/// Tarjan。出力順は依存先 SCC → 依存元 SCC。各 SCC 内は入力順。
pub(super) fn scc(graph: &[Vec<usize>]) -> Vec<Vec<usize>> {
    struct Search<'a> {
        graph: &'a [Vec<usize>],
        next: usize,
        index: Vec<Option<usize>>,
        low: Vec<usize>,
        stack: Vec<usize>,
        active: Vec<bool>,
        result: Vec<Vec<usize>>,
    }
    impl Search<'_> {
        fn visit(&mut self, v: usize) {
            let index = self.next;
            self.next += 1;
            self.index[v] = Some(index);
            self.low[v] = index;
            self.stack.push(v);
            self.active[v] = true;
            for &w in &self.graph[v] {
                if self.index[w].is_none() {
                    self.visit(w);
                    self.low[v] = self.low[v].min(self.low[w]);
                } else if self.active[w] {
                    self.low[v] = self.low[v].min(self.index[w].unwrap());
                }
            }
            if self.low[v] == index {
                let mut component = Vec::new();
                loop {
                    let w = self.stack.pop().unwrap();
                    self.active[w] = false;
                    component.push(w);
                    if w == v {
                        break;
                    }
                }
                component.sort_unstable();
                self.result.push(component);
            }
        }
    }
    let n = graph.len();
    let mut search = Search {
        graph,
        next: 0,
        index: vec![None; n],
        low: vec![0; n],
        stack: Vec::new(),
        active: vec![false; n],
        result: Vec::new(),
    };
    for v in 0..n {
        if search.index[v].is_none() {
            search.visit(v);
        }
    }
    search.result
}
