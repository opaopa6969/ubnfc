//! harness が生成物に同梱する pure scanner 集合。既存 Prebuilt は変更しない。
use super::Result;
use std::{fs, path::Path};

pub(crate) fn java(artifact: &Path) -> Result<()> {
    let directory = artifact.join("src/main/java/ubnfc/scanners");
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join("Registry.java"),
        include_str!("../../../../scanners/java/Registry.java").replace("generated.", "target."),
    )?;
    Ok(())
}

pub(crate) fn rust(artifact: &Path) -> Result<()> {
    fs::write(
        artifact.join("src/bin/scanners.rs"),
        include_str!("../../../../scanners/rust/scanners.rs"),
    )?;
    fs::write(
        artifact.join("src/bin/scanners_tests.rs"),
        include_str!("../../../../scanners/rust/scanners_tests.rs"),
    )?;
    let path = artifact.join("Cargo.toml");
    let manifest = fs::read_to_string(&path)?;
    if !manifest.contains("scanners = []") || manifest.matches("default = []").count() != 1 {
        return Err(super::Error(
            "生成 crate に scanner feature hook がありません".into(),
        ));
    }
    fs::write(
        path,
        manifest.replacen("default = []", "default = [\"scanners\"]", 1),
    )?;
    Ok(())
}
