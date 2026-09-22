use super::*;
use std::collections::BTreeSet;
use std::path::{Component, Path};

#[derive(Debug, Clone)]
pub struct SourceInput {
    pub path: PathBuf,
    pub logical_path: String,
    pub text: String,
    pub ast: Option<a::GrammarAst>,
}
impl SourceInput {
    pub fn new(
        path: impl Into<PathBuf>,
        logical_path: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            logical_path: logical_path.into(),
            text: text.into(),
            ast: None,
        }
    }

    pub fn parsed(
        path: impl Into<PathBuf>,
        logical_path: impl Into<String>,
        text: impl Into<String>,
        ast: a::GrammarAst,
    ) -> Self {
        Self {
            path: path.into(),
            logical_path: logical_path.into(),
            text: text.into(),
            ast: Some(ast),
        }
    }
}
pub(super) struct Sources {
    pub records: Vec<Source>,
    inputs: BTreeMap<PathBuf, (String, Vec<u64>, String)>,
}
impl Sources {
    pub fn new(
        grammar: &ResolvedGrammar,
        sources: &[SourceInput],
    ) -> Result<Self, Vec<LowerError>> {
        let mut result = Self {
            records: vec![],
            inputs: BTreeMap::new(),
        };
        let mut errors = vec![];
        let mut logicals = BTreeSet::new();
        for unit in &grammar.units {
            if result.inputs.contains_key(&unit.path) {
                continue;
            }
            let Some(input) = sources.iter().find(|s| s.path == unit.path) else {
                errors.push(error(
                    "E-LOWER-SOURCE",
                    format!("元ソースがありません: {}", unit.path.display()),
                ));
                continue;
            };
            let Some(path) = normalize_path(&input.logical_path) else {
                errors.push(error(
                    "E-LOWER-SOURCE-PATH",
                    format!("論理ルート相対パスが必要: {}", input.logical_path),
                ));
                continue;
            };
            if !logicals.insert(path.clone())
                || sources.iter().filter(|s| s.path == unit.path).count() != 1
            {
                errors.push(error("E-LOWER-SOURCE-DUPLICATE", path));
                continue;
            }
            let parsed = input
                .ast
                .clone()
                .map(Ok)
                .unwrap_or_else(|| crate::syntax::parse_file(&input.text));
            if !parsed.as_ref().is_ok_and(|parsed| {
                grammar
                    .units
                    .iter()
                    .filter(|u| u.path == unit.path)
                    .all(|u| parsed.grammars.get(u.declaration_index) == Some(&u.ast))
            }) {
                errors.push(error(
                    "E-LOWER-SOURCE-STALE",
                    format!("resolve 時の AST と元ソースが一致しません: {path}"),
                ));
                continue;
            }
            let id = encode(&path);
            let mut offsets = vec![0; input.text.len() + 1];
            let mut count = 0;
            for (byte, _) in input.text.char_indices() {
                offsets[byte] = count;
                count += 1;
            }
            offsets[input.text.len()] = count;
            result.records.push(Source {
                code_point_length: count,
                id: id.clone(),
                path,
                sha256: sha256(input.text.as_bytes()),
            });
            result
                .inputs
                .insert(unit.path.clone(), (id, offsets, input.text.clone()));
        }
        if errors.is_empty() {
            Ok(result)
        } else {
            Err(errors)
        }
    }
    pub fn text(&self, path: &Path) -> &str {
        &self.inputs[path].2
    }
    pub fn span(&self, path: &Path, span: a::Span) -> Span {
        let (source_id, offsets, _) = &self.inputs[path];
        Span {
            source_id: source_id.clone(),
            start: offsets[span.start],
            end: offsets[span.end],
        }
    }
}
fn normalize_path(path: &str) -> Option<String> {
    if path.contains('\\') || path.contains(':') {
        return None;
    }
    let mut parts = vec![];
    for component in Path::new(path).components() {
        match component {
            Component::Normal(p) => parts.push(p.to_str()?),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            _ => return None,
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}
pub(super) fn encode(path: &str) -> String {
    let mut out = String::new();
    for b in path.bytes() {
        if b.is_ascii_alphanumeric() || b"/._~-".contains(&b) {
            out.push(b as char);
        } else {
            use std::fmt::Write;
            write!(out, "%{b:02X}").unwrap();
        }
    }
    out
}
// FIPS 180-4 SHA-256。外部コマンドや新規依存を必要としない source digest。
fn sha256(input: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut bytes = input.to_vec();
    bytes.push(0x80);
    while bytes.len() % 64 != 56 {
        bytes.push(0);
    }
    bytes.extend_from_slice(&((input.len() as u64) * 8).to_be_bytes());
    for chunk in bytes.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, c) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(c.try_into().unwrap());
        }
        for i in 16..64 {
            let a = w[i - 15];
            let b = w[i - 2];
            w[i] = w[i - 16]
                .wrapping_add(a.rotate_right(7) ^ a.rotate_right(18) ^ (a >> 3))
                .wrapping_add(w[i - 7])
                .wrapping_add(b.rotate_right(17) ^ b.rotate_right(19) ^ (b >> 10));
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let t1 = h
                .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
                .wrapping_add((e & f) ^ (!e & g))
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let t2 = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22))
                .wrapping_add((a & b) ^ (a & c) ^ (b & c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (s, v) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *s = s.wrapping_add(v);
        }
    }
    state.iter().map(|s| format!("{s:08x}")).collect()
}
#[test]
fn standard_digest_vectors() {
    assert_eq!(
        sha256(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256(&vec![b'a'; 1_000_000]),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}
