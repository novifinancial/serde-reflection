// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{java, test_utils, CodeGeneratorConfig};
use std::collections::BTreeMap;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_that_java_code_compiles() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    let config = CodeGeneratorConfig::new("testing".to_string());
    let generator = java::CodeGenerator::new(&config);
    generator
        .write_source_files(dir.path().to_path_buf(), &registry)
        .unwrap();

    let paths = std::iter::empty()
        .chain(std::fs::read_dir("runtime/java/com/facebook/serde").unwrap())
        .chain(std::fs::read_dir("runtime/java/com/facebook/bincode").unwrap())
        .chain(std::fs::read_dir(dir.path().join("testing")).unwrap())
        .map(|e| e.unwrap().path());
    let status = Command::new("javac")
        .arg("-Xlint")
        .arg("-d")
        .arg(dir.path())
        .args(paths)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_that_java_code_compiles_with_codegen_options() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    let config = CodeGeneratorConfig::new("testing".to_string()).with_comments(
        vec![(
            vec!["testing".to_string(), "SerdeData".to_string()],
            "Some\ncomments".to_string(),
        )]
        .into_iter()
        .collect(),
    );
    let generator = java::CodeGenerator::new(&config);
    generator
        .write_source_files(dir.path().to_path_buf(), &registry)
        .unwrap();

    // Comment was correctly generated.
    let content = std::fs::read_to_string(dir.path().join("testing/SerdeData.java")).unwrap();
    assert!(content.contains(
        r#"
/**
 * Some
 * comments
 */
"#
    ));

    // Files compile.
    let paths = std::iter::empty()
        .chain(std::fs::read_dir("runtime/java/com/facebook/serde").unwrap())
        .chain(std::fs::read_dir("runtime/java/com/facebook/bincode").unwrap())
        .chain(std::fs::read_dir(dir.path().join("testing")).unwrap())
        .map(|e| e.unwrap().path());
    let status = Command::new("javac")
        .arg("-Xlint")
        .arg("-d")
        .arg(dir.path())
        .args(paths)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_java_code_with_external_definitions() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    // (wrongly) Declare TraitHelpers as external.
    let mut definitions = BTreeMap::new();
    definitions.insert("foo".to_string(), vec!["TraitHelpers".to_string()]);
    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_external_definitions(definitions);
    let generator = java::CodeGenerator::new(&config);

    generator
        .write_source_files(dir.path().to_path_buf(), &registry)
        .unwrap();

    // References were updated.
    let content = std::fs::read_to_string(dir.path().join("testing/SerdeData.java")).unwrap();
    assert!(content.contains("foo.TraitHelpers."));
}
