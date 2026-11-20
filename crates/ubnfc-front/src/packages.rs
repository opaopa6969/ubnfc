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
//! package 内の相対 import と推移的依存も、検証済み artifact の files / dependencies
//! だけから読む。生成時の取得や、artifact の外へ出る相対 import は行わない。

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

/// package 内の import を、同じ artifact の相対 member または明示依存へ写す。
/// marker は package ID と区別し、entry の実パスは検証済み artifact から求める。
pub fn import_path(path: &Path, import: &str) -> Option<Result<PathBuf, String>> {
    let (root, descriptor) = split(path)?;
    let (owner, member) = descriptor
        .split_once("/.entry-relative/")
        .unwrap_or((&descriptor, "__entry__"));
    if let Some(id) = import.strip_prefix("pkg:") {
        return Some(
            check_id(id).map(|()| root.join(VIRTUAL_DIR).join(owner).join(".deps").join(id)),
        );
    }
    let relative = Path::new(import);
    if relative.is_absolute() || import.contains('\\') {
        return Some(Err(
            "package 内の import は相対パスで指定してください".into()
        ));
    }
    let mut member_path = Path::new(member)
        .parent()
        .unwrap_or(Path::new(""))
        .to_path_buf();
    for component in relative.components() {
        match component {
            Component::Normal(value) => member_path.push(value),
            Component::CurDir => {}
            Component::ParentDir => {
                // entry が入れ子の場合の ../ は、検証済み entry の位置を読むまで保留する。
                if member_path.file_name().is_some_and(|name| name != ":up") {
                    member_path.pop();
                } else {
                    member_path.push(":up");
                }
            }
            _ => return Some(Err("package の外へ出る相対 import は使えません".into())),
        }
    }
    Some(Ok(root
        .join(VIRTUAL_DIR)
        .join(owner)
        .join(".entry-relative")
        .join(member_path)))
}

fn safe_member(path: &Path) -> io::Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(invalid("artifact の file path が不正です".into()));
    }
    Ok(())
}

/// manifest → lock → artifact を照合する。依存の source は取得時だけの情報であり、
/// 生成では builtin または lock の raw bytes cache のみを読み、外部へ取得しない。
fn read(
    dir: &Path,
    descriptor: &str,
    base: &dyn Fn(&Path) -> io::Result<String>,
) -> io::Result<String> {
    let manifest_path = dir.join("ubnf.json");
    let manifest = json(&base(&manifest_path)?, &manifest_path)?;
    schema_v1(&manifest, "ubnf.json")?;
    let lock_path = dir.join("ubnf.lock.json");
    let lock = json(&base(&lock_path).map_err(|e| io::Error::new(e.kind(), format!("lock を読めません: {e}。unlaxer deps resolve --manifest ubnf.json で作ってください")))?, &lock_path)?;
    schema_v1(&lock, "ubnf.lock.json")?;
    let roots = field(&lock, "roots", "ubnf.lock.json")?;
    let dependencies = field(&manifest, "dependencies", "ubnf.json")?
        .as_object()
        .ok_or_else(|| invalid("manifest dependencies は object が必要です".into()))?;
    let expected: serde_json::Map<String, Value> = dependencies
        .iter()
        .map(|(id, dep)| {
            Ok((
                id.clone(),
                Value::String(string(dep, "version", "manifest dependency")?.into()),
            ))
        })
        .collect::<io::Result<_>>()?;
    if roots != &Value::Object(expected) {
        return Err(invalid(
            "stale lock: manifest と roots が一致しません".into(),
        ));
    }
    let (chain, member) = descriptor
        .split_once("/.entry-relative/")
        .map_or((descriptor, None), |(c, m)| (c, Some(m)));
    let mut available = Value::Object(dependencies.clone());
    let mut envelope = Value::Null;
    let mut visiting = std::collections::BTreeSet::new();
    for id in chain.split("/.deps/") {
        check_id(id).map_err(invalid)?;
        if !visiting.insert(id) || visiting.len() > 64 {
            return Err(invalid("package 依存が循環しています".into()));
        }
        let dependency = available
            .get(id)
            .ok_or_else(|| invalid(format!("package '{id}' が dependencies にありません")))?;
        let version = string(dependency, "version", "dependency")?;
        let source = string(dependency, "source", "dependency")?;
        let locked = field(&lock, "packages", "lock")?
            .get(id)
            .ok_or_else(|| invalid(format!("lock に package '{id}' がありません")))?;
        if string(locked, "id", "lock")? != id || string(locked, "version", "lock")? != version {
            return Err(invalid(format!(
                "package '{id}' の ID / 版が lock と一致しません"
            )));
        }
        let sha = string(locked, "sha256", "lock")?;
        if sha.len() != 64 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(invalid("lock の SHA-256 が不正です".into()));
        }
        let artifact = if let Some(builtin) = source.strip_prefix("builtin:") {
            BUILTIN
                .iter()
                .find(|(bid, bv, _)| {
                    *bid == id && *bv == version && builtin == format!("{bid}@{bv}")
                })
                .map(|(_, _, text)| (*text).to_owned())
                .ok_or_else(|| invalid(format!("未同梱の builtin: {source}")))?
        } else {
            base(&dir.join(".ubnf-cache/packages").join(format!("{sha}.json")))?
        };
        let actual = crate::lower::sha256(artifact.as_bytes());
        if actual != sha {
            return Err(invalid(format!(
                "package '{id}' の SHA-256 が lock と一致しません"
            )));
        }
        envelope = json(&artifact, Path::new(id))?;
        schema_v1(&envelope, "artifact")?;
        if string(&envelope, "id", "artifact")? != id
            || string(&envelope, "version", "artifact")? != version
        {
            return Err(invalid(format!(
                "artifact '{id}' の ID / 版が lock と一致しません"
            )));
        }
        available = field(&envelope, "dependencies", "artifact")?.clone();
        let refs: serde_json::Map<String, Value> = available
            .as_object()
            .ok_or_else(|| invalid("artifact dependencies は object が必要です".into()))?
            .iter()
            .map(|(id, dep)| {
                Ok((
                    id.clone(),
                    Value::String(string(dep, "version", "artifact dependency")?.into()),
                ))
            })
            .collect::<io::Result<_>>()?;
        if field(locked, "dependencies", "lock")? != &Value::Object(refs) {
            return Err(invalid("artifact の依存が lock と一致しません".into()));
        }
        let files = field(&envelope, "files", "artifact")?
            .as_object()
            .ok_or_else(|| invalid("artifact files は object が必要です".into()))?;
        for name in files.keys() {
            safe_member(Path::new(name))?;
        }
    }
    let entry = Path::new(string(&envelope, "entry", "artifact")?);
    safe_member(entry)?;
    let file = match member {
        Some(member) => {
            let mut file = entry.parent().unwrap_or(Path::new("")).to_path_buf();
            for component in Path::new(member).components() {
                match component {
                    Component::Normal(name) if name == ":up" => {
                        if !file.pop() {
                            return Err(invalid(
                                "package の外へ出る相対 import は使えません".into(),
                            ));
                        }
                    }
                    Component::Normal(name) => file.push(name),
                    _ => return Err(invalid("package の相対 import が不正です".into())),
                }
            }
            file
        }
        None => entry.to_path_buf(),
    };
    safe_member(&file)?;
    string(
        field(&envelope, "files", "artifact")?,
        &file.to_string_lossy(),
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
/// 同梱物と**構文木が同一の** import だけを javaStyle として扱う。その他の宣言的 token は
/// javaStyle に読み替えず、Delimiter::Lexical で原来の program を走査する。
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
    #[test]
    fn package_members_use_the_entry_directory_and_cannot_escape_the_artifact() {
        let artifact = serde_json::json!({"schemaVersion":1,"id":"team/layout","version":"1.0.0",
            "entry":"nested/layout.ubnf","files":{"nested/layout.ubnf":"entry","gap.ubnf":"gap"},"dependencies":{}}).to_string();
        let sha = crate::lower::sha256(artifact.as_bytes());
        let manifest = r#"{"schemaVersion":1,"dependencies":{"team/layout":{"version":"1.0.0","source":"local:layout.json"}}}"#;
        let lock = serde_json::json!({"schemaVersion":1,"roots":{"team/layout":"1.0.0"},"packages":{"team/layout":{
            "id":"team/layout","version":"1.0.0","sha256":sha,"dependencies":{}}}}).to_string();
        let fs = files(&[
            ("/g/ubnf.json", manifest.into()),
            ("/g/ubnf.lock.json", lock),
            (&format!("/g/.ubnf-cache/packages/{sha}.json"), artifact),
        ]);
        let base = |p: &Path| {
            fs.get(p)
                .cloned()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, p.display().to_string()))
        };
        let load = loader(&base);
        let entry = virtual_path(Path::new("/g"), "pkg:team/layout")
            .unwrap()
            .unwrap();
        assert_eq!(load(&entry).unwrap(), "entry");
        let gap = import_path(&entry, "../gap.ubnf").unwrap().unwrap();
        assert_eq!(load(&gap).unwrap(), "gap");
        let outside = import_path(&entry, "../../outside.ubnf").unwrap().unwrap();
        assert!(load(&outside).unwrap_err().to_string().contains("外へ"));
        let undeclared = import_path(&entry, "pkg:other/layout").unwrap().unwrap();
        assert!(load(&undeclared)
            .unwrap_err()
            .to_string()
            .contains("dependencies"));
    }
}
