// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{golang, test_utils, CodeGeneratorConfig};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

fn test_that_golang_code_compiles_with_config(config: &CodeGeneratorConfig) -> std::path::PathBuf {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.go");
    let mut source = File::create(&source_path).unwrap();

    let generator = golang::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    writeln!(&mut source, "func main() {{}}").unwrap();

    let go_path = format!(
        "{}:{}",
        std::env::var("GOPATH").unwrap_or_default(),
        std::env::current_dir()
            .unwrap()
            .join("runtime/golang")
            .to_str()
            .unwrap()
    );
    let status = Command::new("go")
        .current_dir(dir.path())
        .env("GOPATH", go_path)
        .arg("build")
        .arg(source_path.clone())
        .status()
        .unwrap();
    assert!(status.success());

    source_path.clone()
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
fn test_that_golang_code_compiles_without_comments() {
    let config = CodeGeneratorConfig::new("main".to_string()).with_serialization(false);
    test_that_golang_code_compiles_with_config(&config);
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
