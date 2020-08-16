// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{golang, test_utils, CodeGeneratorConfig};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::{tempdir, TempDir};

fn test_that_golang_code_compiles_with_config(
    config: &CodeGeneratorConfig,
) -> (TempDir, std::path::PathBuf) {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.go");
    let mut source = File::create(&source_path).unwrap();

    let generator = golang::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    writeln!(&mut source, "func main() {{}}").unwrap();

    let status = Command::new("go")
        .current_dir(dir.path())
        .arg("mod")
        .arg("init")
        .arg("example.com/test")
        .status()
        .unwrap();
    assert!(status.success());

    let mut runtime_mod_path = std::env::current_exe().unwrap();
    runtime_mod_path = runtime_mod_path.to_owned();
    runtime_mod_path.pop();
    runtime_mod_path.pop();
    runtime_mod_path.pop();
    runtime_mod_path.pop();
    runtime_mod_path.push("serde-generate/runtime/golang");

    let status = Command::new("go")
        .current_dir(dir.path())
        .arg("mod")
        .arg("edit")
        .arg("-replace")
        .arg(format!(
            "github.com/facebookincubator/serde-reflection/serde-generate/runtime/golang={}",
            runtime_mod_path.as_os_str().to_str().unwrap()
        ))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("go")
        .current_dir(dir.path())
        .arg("build")
        .arg(source_path.clone())
        .status()
        .unwrap();
    assert!(status.success());

    (dir, source_path.clone())
}

#[test]
fn test_that_golang_code_compiles() {
    let config = CodeGeneratorConfig::new("main".to_string());
    test_that_golang_code_compiles_with_config(&config);
}

#[test]
fn test_that_golang_code_compiles_without_serialization() {
    let config = CodeGeneratorConfig::new("main".to_string()).with_serialization(false);
    test_that_golang_code_compiles_with_config(&config);
}

#[test]
fn test_that_golang_code_compiles_with_comments() {
    let comments = vec![
        (
            vec!["main".to_string(), "SerdeData".to_string()],
            "Some\ncomments".to_string(),
        ),
        (
            vec!["main".to_string(), "List".to_string(), "Node".to_string()],
            "Some other comments".to_string(),
        ),
    ]
    .into_iter()
    .collect();
    let config = CodeGeneratorConfig::new("main".to_string()).with_comments(comments);

    let (_dir, source_path) = test_that_golang_code_compiles_with_config(&config);
    // Comments were correctly generated.
    let content = std::fs::read_to_string(&source_path).unwrap();
    assert!(content.contains(
        r#"
// Some
// comments
"#
    ));
    assert!(content.contains(
        r#"
// Some other comments
"#
    ));
}

#[test]
fn test_golang_code_with_external_definitions() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.go");
    let mut source = File::create(&source_path).unwrap();

    // Pretend that "Tree" is external.
    let mut definitions = BTreeMap::new();
    definitions.insert("foo".to_string(), vec!["Tree".to_string()]);
    let config =
        CodeGeneratorConfig::new("main".to_string()).with_external_definitions(definitions);
    let generator = golang::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    // References were updated.
    let content = std::fs::read_to_string(source_path).unwrap();
    assert!(content.contains("foo.Tree"));
}
