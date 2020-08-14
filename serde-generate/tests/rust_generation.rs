// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{rust, test_utils, CodeGeneratorConfig};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

// Quick test using rustc directly.
#[test]
fn test_that_rust_code_compiles() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.rs");
    let mut source = File::create(&source_path).unwrap();

    let config = CodeGeneratorConfig::new("testing".to_string()).with_serialization(false);
    let generator = rust::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    let status = Command::new("rustc")
        .current_dir(dir.path())
        .arg("--crate-type")
        .arg("lib")
        .arg("--edition")
        .arg("2018")
        .arg(source_path)
        .status()
        .unwrap();
    assert!(status.success());
}

// Quick test using rustc directly.
#[test]
fn test_that_rust_code_compiles_with_codegen_options() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.rs");
    let mut source = File::create(&source_path).unwrap();
    let comments = vec![(
        vec!["testing".to_string(), "SerdeData".to_string()],
        "Some\ncomments".to_string(),
    )]
    .into_iter()
    .collect();
    let definitions = vec![
        ("foo".to_string(), vec!["Map".to_string()]),
        (String::new(), vec!["Bytes".into()]),
    ]
    .into_iter()
    .collect();

    let config = CodeGeneratorConfig::new("testing".to_string())
        .with_comments(comments)
        .with_external_definitions(definitions)
        .with_serialization(false);
    let generator = rust::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    // Comment was correctly generated.
    let content = std::fs::read_to_string(&source_path).unwrap();
    assert!(content.contains("/// Some\n/// comments\n"));

    let output = Command::new("rustc")
        .current_dir(dir.path())
        .arg("--crate-type")
        .arg("lib")
        .arg("--edition")
        .arg("2018")
        .arg(source_path.clone())
        .output()
        .unwrap();
    assert!(!output.status.success());

    // Externally defined names "Map" and "Bytes" have caused the usual imports to be
    // replaced by `use foo::Map` (and nothing, respectively), so we must add the definitions.
    writeln!(
        &mut source,
        r#"
type Bytes = Vec<u8>;

mod foo {{
    pub type Map<K, V> = std::collections::BTreeMap<K, V>;
}}
"#
    )
    .unwrap();

    let status = Command::new("rustc")
        .current_dir(dir.path())
        .arg("--crate-type")
        .arg("lib")
        .arg("--edition")
        .arg("2018")
        .arg(source_path)
        .status()
        .unwrap();
    assert!(status.success());
}

// Full test using cargo. This may take a while.
#[test]
fn test_that_rust_code_compiles_with_derive_macros() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        r#"[package]
name = "testing"
version = "0.1.0"
edition = "2018"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_bytes = "0.11"

[workspace]
"#,
    )
    .unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();

    let config = CodeGeneratorConfig::new("testing".to_string());
    let generator = rust::CodeGenerator::new(&config);

    let source_path = dir.path().join("src/lib.rs");
    let mut source = File::create(&source_path).unwrap();
    generator.output(&mut source, &registry).unwrap();

    // Use a stable `target` dir to avoid downloading and recompiling crates everytime.
    let target_dir = std::env::current_dir().unwrap().join("../target");
    let status = Command::new("cargo")
        .current_dir(dir.path())
        .arg("build")
        .arg("--target-dir")
        .arg(target_dir)
        .status()
        .unwrap();
    assert!(status.success());
}
