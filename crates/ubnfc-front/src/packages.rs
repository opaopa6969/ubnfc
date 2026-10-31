//! `@import alias from 'pkg:ID'`（Classic の明示 package 依存。unlaxer-parser
//! `docs/ubnf-packages.md`、#401）を、ubnfc の import 解決へ載せる。
//!
//! Classic と同じく、package の取得は行わない。grammar と同じディレクトリの
//! `ubnf.json`（manifest）と `ubnf.lock.json`（lock）を読み、lock が固定した
//! artifact の SHA-256 を照合してから entry の `.ubnf` を返す。
//!
//! - `builtin:` の source は ubnfc に同梱した artifact（Classic の同梱物と byte 同一）を
//!   使う。cache は読まない。lock の SHA-256 が同梱物と違えば拒否する。
//! - それ以外の source（`local:` / `https:`）は `.ubnf-cache/packages/<sha256>.json` を読み、
//!   内容の SHA-256 が lock と一致するときだけ使う。
//!
//! resolve は `pkg:ID` を仮想パス `<grammar のディレクトリ>/.ubnf-packages/ID` に写し、
//! pipeline の loader（[`loader`]）がその仮想パスを上の手順で読む。IR の `sources[]` には
//! この論理パスと entry の本文の SHA-256 が載る。
//!
//! 範囲外（位置付きの `E-IMPORT-UNRESOLVED` になる）: package の中からの import
//! （package 内の相対 import と推移的な package 依存）。std/layout はどちらも持たない。

use std::io;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

/// 仮想パスの目印になるディレクトリ名。実在のディレクトリは要らない。
pub const VIRTUAL_DIR: &str = ".ubnf-packages";

/// ubnfc に同梱する artifact（`builtin:` source）。Classic の
/// `unlaxer-dsl/src/main/resources/ubnf-packages/` と byte 同一。
const BUILTIN: &[(&str, &str, &str)] = &[(
    "std/layout",
    "1.0.0",
    include_str!("../packages/std-layout-1.0.0.json"),
)];

/// `pkg:ID` の import を仮想パスに写す。`pkg:` で始まらなければ `None`。
/// ID の検査（空の segment、`.`、`..`、backslash、colon）に通らなければ `Err`。
pub fn virtual_path(dir: &Path, import: &str) -> Option<Result<PathBuf, String>> {
    let id = import.strip_prefix("pkg:")?;
    Some(check_id(id).map(|()| {
        let mut path = dir.join(VIRTUAL_DIR);
        for segment in id.split('/') {
            path.push(segment);
        }
        path
    }))
}

fn check_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == ".." || s.contains(['\\', ':']))
    {
        return Err(format!("package ID が不正です: '{id}'"));
    }
    Ok(())
}

/// text loader を包み、仮想パスを package として読む。それ以外の path は `base` に渡す。
pub fn loader<'a>(
    base: &'a dyn Fn(&Path) -> io::Result<String>,
) -> impl Fn(&Path) -> io::Result<String> + 'a {
    move |path| match split(path) {
        Some((dir, id)) => read(&dir, &id, base),
        None => base(path),
    }
}

/// `std::fs::read_to_string` に package の仮想パスの読み込みを足したもの。
/// `resolve::resolve` を直接呼ぶ側（pipeline を通らない側）が使う。
pub fn read_to_string(path: &Path) -> io::Result<String> {
    let base = |p: &Path| std::fs::read_to_string(p);
    match split(path) {
        Some((dir, id)) => read(&dir, &id, &base),
        None => base(path),
    }
}

/// 仮想パスを（grammar のディレクトリ, package ID）に分ける。
fn split(path: &Path) -> Option<(PathBuf, String)> {
    let components: Vec<_> = path.components().collect();
    let at = components
        .iter()
        .position(|c| matches!(c, Component::Normal(s) if *s == VIRTUAL_DIR))?;
    let dir: PathBuf = components[..at].iter().collect();
    let id = components[at + 1..]
        .iter()
        .map(|c| c.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()?
        .join("/");
    Some((dir, id))
}

fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn json(text: &str, what: &Path) -> io::Result<Value> {
    serde_json::from_str(text)
        .map_err(|e| invalid(format!("{}: JSON として読めません: {e}", what.display())))
}

fn field<'v>(value: &'v Value, key: &str, what: &str) -> io::Result<&'v Value> {
    value
        .get(key)
        .ok_or_else(|| invalid(format!("{what}: '{key}' がありません")))
}

fn string<'v>(value: &'v Value, key: &str, what: &str) -> io::Result<&'v str> {
    field(value, key, what)?
        .as_str()
        .ok_or_else(|| invalid(format!("{what}: '{key}' は文字列でなければなりません")))
}

fn schema_v1(value: &Value, what: &str) -> io::Result<()> {
    if field(value, "schemaVersion", what)?.as_u64() != Some(1) {
        return Err(invalid(format!("{what}: schemaVersion 1 だけを扱います")));
    }
    Ok(())
}

/// manifest → lock → artifact の順に照合し、entry の本文を返す。
fn read(dir: &Path, id: &str, base: &dyn Fn(&Path) -> io::Result<String>) -> io::Result<String> {
    check_id(id).map_err(invalid)?;
    let manifest_path = dir.join("ubnf.json");
    let manifest = json(
        &base(&manifest_path).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "{}: package '{id}' の manifest を読めません（{e}）",
                    manifest_path.display()
                ),
            )
        })?,
        &manifest_path,
    )?;
    schema_v1(&manifest, "ubnf.json")?;
    let dependency = field(&manifest, "dependencies", "ubnf.json")?
        .get(id)
        .ok_or_else(|| {
            invalid(format!(
                "ubnf.json: package '{id}' が dependencies にありません"
            ))
        })?;
    let version = string(dependency, "version", "ubnf.json")?;
    let source = string(dependency, "source", "ubnf.json")?;

    let lock_path = dir.join("ubnf.lock.json");
    let lock = json(&base(&lock_path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "{}: lock を読めません（{e}）。Classic の `unlaxer deps resolve --manifest ubnf.json` で作ってください",
                lock_path.display()
            ),
        )
    })?, &lock_path)?;
    schema_v1(&lock, "ubnf.lock.json")?;
    let roots = field(&lock, "roots", "ubnf.lock.json")?;
    if roots.get(id).and_then(Value::as_str) != Some(version) {
        return Err(invalid(format!(
            "ubnf.lock.json: package '{id}' の版が manifest（{version}）と一致しません（stale lock）"
        )));
    }
    let locked = field(&lock, "packages", "ubnf.lock.json")?
        .get(id)
        .ok_or_else(|| invalid(format!("ubnf.lock.json: package '{id}' がありません")))?;
    if string(locked, "id", "ubnf.lock.json")? != id
        || string(locked, "version", "ubnf.lock.json")? != version
    {
        return Err(invalid(format!(
            "ubnf.lock.json: package '{id}' の ID / 版が一致しません"
        )));
    }
    let sha = string(locked, "sha256", "ubnf.lock.json")?;

    let artifact = if let Some(builtin) = source.strip_prefix("builtin:") {
        let (_, _, text) = BUILTIN
            .iter()
            .find(|(bid, bversion, _)| {
                builtin == format!("{bid}@{bversion}") && *bid == id && *bversion == version
            })
            .ok_or_else(|| {
                invalid(format!(
                    "ubnf.json: ubnfc に同梱していない builtin package です: '{source}'"
                ))
            })?;
        (*text).to_owned()
    } else {
        let cache = dir
            .join(".ubnf-cache")
            .join("packages")
            .join(format!("{sha}.json"));
        base(&cache).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "{}: package '{id}' の cache がありません（{e}）",
                    cache.display()
                ),
            )
        })?
    };
    let actual = crate::lower::sha256(artifact.as_bytes());
    if actual != sha {
        return Err(invalid(format!(
            "package '{id}' の SHA-256 が lock と一致しません（lock {sha}、実際 {actual}）"
        )));
    }
    let envelope = json(&artifact, Path::new(id))?;
    schema_v1(&envelope, "artifact")?;
    if string(&envelope, "id", "artifact")? != id
        || string(&envelope, "version", "artifact")? != version
    {
        return Err(invalid(format!(
            "artifact: package '{id}' の ID / 版が lock と一致しません"
        )));
    }
    if field(&envelope, "dependencies", "artifact")?
        .as_object()
        .is_some_and(|d| !d.is_empty())
    {
        return Err(invalid(format!(
            "artifact: package '{id}' の推移的な package 依存は ubnfc ではまだ扱えません"
        )));
    }
    let entry = string(&envelope, "entry", "artifact")?;
    string(
        field(&envelope, "files", "artifact")?,
        entry,
        "artifact files",
    )
    .map(str::to_owned)
}

/// grammar が直接 import した std/layout 1.0.0 の `SPACES_AND_COMMENTS` を、`@whitespace` に
/// 書ける profile 名（`alias.SPACES_AND_COMMENTS`）として返す。
///
/// std/layout の `SPACES_AND_COMMENTS` は javaStyle と同じ区切り集合（ASCII space と
/// TAB〜CR、`//` から CR / LF の直前まで、`/*` から最初の `*/` まで。未閉鎖の block comment
/// は trivia として成功しない）であり、Classic も javaStyle の 11 文法・24 入力を明示 import に
/// 置き換えて同値を確かめている（unlaxer-parser `docs/ubnf-packages.md`）。そこで ubnfc は
/// 同梱物と**構文木が同一の** import だけを javaStyle として扱い、それ以外の token を profile に
/// する綴りは従来どおり `E-WHITESPACE-*-STYLE` で拒否する。
pub fn whitespace_profiles(
    resolved: &crate::resolve::ResolvedGrammar,
    grammar: &crate::resolve::Grammar,
) -> std::collections::BTreeSet<String> {
    let Some(builtin) = BUILTIN
        .iter()
        .find(|(id, version, _)| *id == "std/layout" && *version == "1.0.0")
        .and_then(|(_, _, text)| entry_text(text))
        .and_then(|text| crate::syntax::parse_file(&text).ok())
        .and_then(|mut file| {
            // resolve が持つ unit の AST は脱糖後なので、同梱物も同じ手順を通して比べる。
            crate::desugar::file(&mut file, &mut Vec::new());
            file.grammars.into_iter().next()
        })
    else {
        return Default::default();
    };
    resolved
        .imports
        .iter()
        .filter(|edge| edge.grammar == grammar.id && edge.from == grammar.unit)
        .filter(|edge| {
            resolved
                .units
                .iter()
                .find(|unit| unit.id == edge.to)
                .is_some_and(|unit| {
                    split(&unit.path).is_some_and(|(_, id)| id == "std/layout")
                        && strip_spans(&unit.ast) == strip_spans(&builtin)
                })
        })
        .map(|edge| format!("{}.SPACES_AND_COMMENTS", edge.alias))
        .collect()
}

fn entry_text(artifact: &str) -> Option<String> {
    let envelope: Value = serde_json::from_str(artifact).ok()?;
    let entry = envelope.get("entry")?.as_str()?;
    Some(envelope.get("files")?.get(entry)?.as_str()?.to_owned())
}

/// 構文木の比較から位置を除く（同じ本文なら span も同じだが、比較の意図を明示する）。
fn strip_spans(grammar: &crate::syntax::ast::GrammarDecl) -> String {
    let mut value = serde_json::to_value(grammar).unwrap_or(Value::Null);
    fn walk(value: &mut Value) {
        match value {
            Value::Object(map) => {
                map.remove("span");
                map.values_mut().for_each(walk);
            }
            Value::Array(items) => items.iter_mut().for_each(walk),
            _ => {}
        }
    }
    walk(&mut value);
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    const STD_SHA: &str = "2701f6884ea6c712d28c76ec3d340c3b27fca4575846239bd0854933205f4b67";

    fn files(entries: &[(&str, String)]) -> BTreeMap<PathBuf, String> {
        entries
            .iter()
            .map(|(p, t)| (PathBuf::from(p), t.clone()))
            .collect()
    }

    fn manifest(source: &str) -> String {
        format!(
            r#"{{"schemaVersion":1,"dependencies":{{"std/layout":{{"version":"1.0.0","source":"{source}"}}}}}}"#
        )
    }

    fn lock(sha: &str) -> String {
        format!(
            r#"{{"packages":{{"std/layout":{{"dependencies":{{}},"id":"std/layout","sha256":"{sha}","version":"1.0.0"}}}},"roots":{{"std/layout":"1.0.0"}},"schemaVersion":1}}"#
        )
    }

    fn run(fs: &BTreeMap<PathBuf, String>) -> io::Result<String> {
        let base = |p: &Path| {
            fs.get(p)
                .cloned()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, p.display().to_string()))
        };
        let path = virtual_path(Path::new("/g"), "pkg:std/layout")
            .unwrap()
            .unwrap();
        let wrapped = loader(&base);
        let result = wrapped(&path);
        result
    }

    #[test]
    fn builtin_std_layout_matches_classic_bytes() {
        assert_eq!(crate::lower::sha256(BUILTIN[0].2.as_bytes()), STD_SHA);
        let fs = files(&[
            ("/g/ubnf.json", manifest("builtin:std/layout@1.0.0")),
            ("/g/ubnf.lock.json", lock(STD_SHA)),
        ]);
        let text = run(&fs).unwrap();
        assert!(text.starts_with("grammar StandardLayout {"));
        assert!(
            text.contains("token SPACES_AND_COMMENTS ::= SPACES | LINE_COMMENT | BLOCK_COMMENT;")
        );
    }

    #[test]
    fn local_source_reads_the_hash_checked_cache() {
        let artifact = BUILTIN[0].2.to_owned();
        let fs = files(&[
            ("/g/ubnf.json", manifest("local:packages/layout.json")),
            ("/g/ubnf.lock.json", lock(STD_SHA)),
            (
                &format!("/g/.ubnf-cache/packages/{STD_SHA}.json"),
                artifact.clone(),
            ),
        ]);
        assert!(run(&fs).unwrap().contains("StandardLayout"));
        // 改変した cache は拒否する。
        let tampered = artifact.replace("'//'", "'#'");
        let fs = files(&[
            ("/g/ubnf.json", manifest("local:packages/layout.json")),
            ("/g/ubnf.lock.json", lock(STD_SHA)),
            (&format!("/g/.ubnf-cache/packages/{STD_SHA}.json"), tampered),
        ]);
        assert!(run(&fs).unwrap_err().to_string().contains("SHA-256"));
    }

    #[test]
    fn missing_lock_stale_lock_and_bad_ids_are_rejected() {
        let fs = files(&[("/g/ubnf.json", manifest("builtin:std/layout@1.0.0"))]);
        assert!(run(&fs).unwrap_err().to_string().contains("lock"));
        let stale = lock(STD_SHA).replace(
            r#""roots":{"std/layout":"1.0.0"}"#,
            r#""roots":{"std/layout":"0.9.0"}"#,
        );
        let fs = files(&[
            ("/g/ubnf.json", manifest("builtin:std/layout@1.0.0")),
            ("/g/ubnf.lock.json", stale),
        ]);
        assert!(run(&fs).unwrap_err().to_string().contains("stale"));
        let fs = files(&[
            ("/g/ubnf.json", manifest("builtin:std/layout@1.0.0")),
            ("/g/ubnf.lock.json", lock(&"0".repeat(64))),
        ]);
        assert!(run(&fs).unwrap_err().to_string().contains("SHA-256"));
        for bad in ["pkg:", "pkg:std//layout", "pkg:../x", "pkg:a\\b", "pkg:c:d"] {
            assert!(
                virtual_path(Path::new("/g"), bad).unwrap().is_err(),
                "{bad}"
            );
        }
        assert!(virtual_path(Path::new("/g"), "lexical/x.ubnf").is_none());
    }
}
