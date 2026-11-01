//! corpus の modules / manifest を Classic と同じ raw artifact・lock/cache へ展開する。
use super::*;
use std::io::Write;
use std::process::Stdio;

fn relative(name: &str) -> Result<&Path> {
    let path = Path::new(name);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(Error(format!("不正な fixture 相対パス: {name}")));
    }
    Ok(path)
}
fn insert(files: &mut BTreeMap<String, String>, name: String, text: String) -> Result<()> {
    relative(&name)?;
    if let Some(old) = files.insert(name.clone(), text.clone()) {
        if old != text {
            return Err(Error(format!("fixture の内容が衝突しました: {name}")));
        }
    }
    Ok(())
}
fn digest(text: &str) -> Result<String> {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child.stdin.take().unwrap().write_all(text.as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(Error("fixture の SHA-256 を計算できません".into()));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap()
        .into())
}
pub(super) fn expand(metadata: &Value) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    for key in ["imports", "modules"] {
        if let Some(value) = metadata.get(key) {
            let entries: BTreeMap<String, String> =
                serde_json::from_value(value.clone()).map_err(|e| Error(format!("{key}: {e}")))?;
            for (name, text) in entries {
                insert(&mut files, name, text)?;
            }
        }
    }
    if let Some(manifest) = metadata.get("manifest") {
        let dependencies = manifest
            .get("dependencies")
            .and_then(Value::as_object)
            .ok_or_else(|| Error("manifest.dependencies が必要です".into()))?;
        let mut packages = BTreeMap::new();
        let mut visiting = BTreeSet::new();
        for (id, dependency) in dependencies {
            resolve(
                id,
                dependency,
                Path::new(""),
                &mut files,
                &mut packages,
                &mut visiting,
            )?;
        }
        let roots: BTreeMap<_, _> = dependencies
            .iter()
            .map(|(id, dep)| (id, dep["version"].clone()))
            .collect();
        insert(
            &mut files,
            "ubnf.json".into(),
            serde_json::to_string(manifest).unwrap(),
        )?;
        insert(
            &mut files,
            "ubnf.lock.json".into(),
            serde_json::to_string(&serde_json::json!({
                "schemaVersion":1, "roots":roots, "packages":packages
            }))
            .unwrap(),
        )?;
    }
    Ok(files)
}
fn resolve(
    id: &str,
    dependency: &Value,
    directory: &Path,
    files: &mut BTreeMap<String, String>,
    packages: &mut BTreeMap<String, Value>,
    visiting: &mut BTreeSet<String>,
) -> Result<()> {
    if !visiting.insert(id.into()) || visiting.len() > 64 {
        return Err(Error(format!("fixture package の循環または深さ上限: {id}")));
    }
    let source = dependency["source"]
        .as_str()
        .ok_or_else(|| Error("package.source が必要です".into()))?;
    let (text, child_directory) = if source == "builtin:std/layout@1.0.0" {
        (
            include_str!("../../../ubnfc-front/packages/std-layout-1.0.0.json").to_owned(),
            directory.to_path_buf(),
        )
    } else if let Some(local) = source.strip_prefix("local:") {
        relative(local)?;
        let path = directory.join(local);
        let name = path
            .to_str()
            .ok_or_else(|| Error("fixture path は UTF-8 が必要です".into()))?;
        (
            files
                .get(name)
                .cloned()
                .ok_or_else(|| Error(format!("fixture artifact がありません: {name}")))?,
            path.parent().unwrap().to_path_buf(),
        )
    } else {
        return Err(Error(format!(
            "fixture の package source はオフラインで解決できません: {source}"
        )));
    };
    let artifact: Value =
        serde_json::from_str(&text).map_err(|e| Error(format!("artifact {id}: {e}")))?;
    if artifact["schemaVersion"] != 1
        || artifact["id"] != id
        || artifact["version"] != dependency["version"]
    {
        return Err(Error(format!(
            "fixture artifact の ID / 版 / schema が一致しません: {id}"
        )));
    }
    let nested = artifact["dependencies"]
        .as_object()
        .ok_or_else(|| Error("artifact.dependencies が必要です".into()))?;
    let references: BTreeMap<_, _> = nested
        .iter()
        .map(|(id, dep)| (id, dep["version"].clone()))
        .collect();
    let sha = digest(&text)?;
    let pinned = serde_json::json!({"id":id,"version":artifact["version"],"sha256":sha,"dependencies":references});
    if let Some(previous) = packages.get(id) {
        if previous != &pinned {
            return Err(Error(format!(
                "fixture package の同一性が衝突しました: {id}"
            )));
        }
    } else {
        packages.insert(id.into(), pinned);
        insert(files, format!(".ubnf-cache/packages/{sha}.json"), text)?;
        for (id, dep) in nested {
            resolve(id, dep, &child_directory, files, packages, visiting)?;
        }
    }
    visiting.remove(id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn expanded_modules_manifests_and_raw_artifacts_define_cache_identity() {
        let fixture = |gap: &str| {
            json!({
                "modules":{"layout.json":json!({"schemaVersion":1,"id":"team/layout","version":"1.0.0","entry":"layout.ubnf",
                    "files":{"layout.ubnf":gap},"dependencies":{}}).to_string()},
                "manifest":{"schemaVersion":1,"dependencies":{"team/layout":{"version":"1.0.0","source":"local:layout.json"}}}
            })
        };
        let first = expand(&fixture("grammar L { @ubnf: v2 token GAP ::= '#'; }")).unwrap();
        let second = expand(&fixture("grammar L { @ubnf: v2 token GAP ::= '+'; }")).unwrap();
        assert_ne!(first, second);
        assert_ne!(
            source_hash(&serde_json::to_string(&first).unwrap()),
            source_hash(&serde_json::to_string(&second).unwrap())
        );
        assert!(first.contains_key("ubnf.json"));
        let lock: Value = serde_json::from_str(&first["ubnf.lock.json"]).unwrap();
        let sha = lock["packages"]["team/layout"]["sha256"].as_str().unwrap();
        assert_eq!(
            first[&format!(".ubnf-cache/packages/{sha}.json")],
            first["layout.json"]
        );
    }
    #[test]
    fn missing_sources_cycles_conflicts_and_path_escapes_are_rejected() {
        for name in ["../escape", "/absolute", "a/../../b"] {
            assert!(expand(&json!({"modules":{name:"text"}})).is_err());
        }
        assert!(expand(&json!({"imports":{"x":"one"},"modules":{"x":"two"}})).is_err());
        assert!(expand(&json!({"manifest":{"schemaVersion":1,"dependencies":{"team/layout":{"version":"1","source":"local:missing.json"}}}})).is_err());
    }
}
