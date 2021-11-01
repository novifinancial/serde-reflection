// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{dart, test_utils, CodeGeneratorConfig, Encoding, SourceInstaller};
use std::{
    io::Result,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::tempdir;

fn install_test_dependencies(path: &Path) -> Result<()> {
    Command::new("dart")
        .current_dir(path)
        .args(["pub", "add", "-d", "test"])
        .status()?;

    Ok(())
}

fn generate_with_config(source_path: PathBuf, config: &CodeGeneratorConfig) -> PathBuf {
    let registry = test_utils::get_registry().unwrap();

    let installer = dart::Installer::new(source_path.clone());
    installer.install_module(config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_bcs_runtime().unwrap();

    install_test_dependencies(&source_path).unwrap();

    let dart_analyze = Command::new("dart")
        .current_dir(&source_path)
        .args(["analyze"])
        .status()
        .unwrap();

    assert!(
        dart_analyze.success(),
        "Generated Dart source code did not pass `dart analyze`"
    );

    source_path
}

#[test]
fn test_dart_code_compiles() {
    let source_path = tempdir().unwrap().path().join("dart_basic_project");

    let config = CodeGeneratorConfig::new("example".to_string())
        .with_encodings(vec![Encoding::Bcs, Encoding::Bincode])
        .with_c_style_enums(true);

    generate_with_config(source_path, &config);
}

#[test]
fn test_dart_code_compiles_with_comments() {
    let source_path = tempdir().unwrap().path().join("dart_comment_project");

    let comments = vec![(
        vec!["example".to_string(), "SerdeData".to_string()],
        "Some\ncomments".to_string(),
    )]
    .into_iter()
    .collect();

    let config = CodeGeneratorConfig::new("example".to_string())
        .with_encodings(vec![Encoding::Bincode])
        .with_c_style_enums(true)
        .with_comments(comments);

    let path = generate_with_config(source_path, &config);

    // Comment was correctly generated.
    let content = std::fs::read_to_string(
        path.join("lib")
            .join("src")
            .join(config.module_name())
            .join("serde_data.dart"),
    )
    .unwrap();

    assert!(content.contains(
        r#"
/// Some
/// comments
"#
    ));
}

#[test]
fn test_dart_code_compiles_with_class_enums() {
    let source_path = tempdir().unwrap().path().join("dart_enum_project");

    let config = CodeGeneratorConfig::new("example".to_string())
        .with_encodings(vec![Encoding::Bcs, Encoding::Bincode])
        .with_c_style_enums(false);

    generate_with_config(source_path, &config);
}
