// 局所診断を共有する arena。memo replay の rule path は親に接続して復元する。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Diag(pub usize);
#[derive(Clone, Copy)]
enum Node {
    Empty,
    Failure(usize, &'static str),
    Join(Diag, Diag),
    Rule(usize, Diag),
}
pub struct Diagnostics {
    nodes: Vec<(usize, Node)>,
}
impl Diagnostics {
    pub fn new() -> Self {
        Self {
            nodes: vec![(0, Node::Empty)],
        }
    }
    fn push(&mut self, pos: usize, node: Node) -> Diag {
        let id = Diag(self.nodes.len());
        self.nodes.push((pos, node));
        id
    }
    pub fn fail(&mut self, pos: usize, label: &'static str) -> Diag {
        self.push(pos, Node::Failure(pos, label))
    }
    pub fn join(&mut self, a: Diag, b: Diag) -> Diag {
        if a.0 == 0 {
            return b;
        }
        if b.0 == 0 || a == b {
            return a;
        }
        let pa = self.nodes[a.0].0;
        let pb = self.nodes[b.0].0;
        if pa > pb {
            a
        } else if pb > pa {
            b
        } else {
            self.push(pa, Node::Join(a, b))
        }
    }
    pub fn rule(&mut self, rule: usize, child: Diag) -> Diag {
        if child.0 == 0 {
            child
        } else {
            self.push(self.nodes[child.0].0, Node::Rule(rule, child))
        }
    }
    pub fn summary(&self, root: Diag) -> Option<(usize, Vec<&'static str>, Vec<usize>)> {
        if root.0 == 0 {
            return None;
        }
        let mut expected = Vec::new();
        let mut deepest = Vec::new();
        let mut path = Vec::new();
        let mut stack = vec![(root, false)];
        while let Some((id, leave)) = stack.pop() {
            if leave {
                path.pop();
                continue;
            }
            match self.nodes[id.0].1 {
                Node::Empty => {}
                Node::Failure(pos, label) => {
                    debug_assert_eq!(pos, self.nodes[root.0].0);
                    if !expected.contains(&label) {
                        expected.push(label);
                    }
                    if path.len() > deepest.len() {
                        deepest.clone_from(&path);
                    }
                }
                Node::Join(a, b) => {
                    stack.push((b, false));
                    stack.push((a, false));
                }
                Node::Rule(rule, child) => {
                    path.push(rule);
                    stack.push((id, true));
                    stack.push((child, false));
                }
            }
        }
        Some((self.nodes[root.0].0, expected, deepest))
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}
