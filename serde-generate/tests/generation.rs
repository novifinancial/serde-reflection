// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{cpp, python3, rust};
use serde_reflection::RegistryOwned;
use std::fs::File;
use std::process::Command;
use tempfile::tempdir;

fn get_registry() -> RegistryOwned {
    // A real-life example taken from Libra.
    let path = "tests/staged/libra.yaml";
    let mut content = std::fs::read_to_string(path).unwrap();
    // Make sure to cover all types.
    content += r#"
SomethingWithIntegers:
  ENUM:
    0:
      A:
        NEWTYPE:
          U8
    1:
      B:
        NEWTYPE:
          U16
    2:
      C:
        NEWTYPE:
          U32
    3:
      D:
        NEWTYPE:
          U64
    4:
      E:
        NEWTYPE:
          U128
    5:
      F:
        NEWTYPE:
          I8
    6:
      G:
        NEWTYPE:
          I16
    7:
      H:
        NEWTYPE:
          I32
    8:
      I:
        NEWTYPE:
          I64
    9:
      J:
        NEWTYPE:
          I128
SomethingWithAMap:
  STRUCT:
    - map:
        MAP:
          KEY: U64
          VALUE:
            TUPLE:
              - TYPENAME: SomethingWithIntegers
              - STR
"#;
    serde_yaml::from_str(content.as_str()).unwrap()
}

#[test]
fn test_that_python_code_parses() {
    let registry = get_registry();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.py");
    let mut source = File::create(&source_path).unwrap();
    python3::output(&mut source, &registry).unwrap();

    let output = Command::new("python3").arg(source_path).output().unwrap();
    assert_eq!(String::new(), String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}

#[test]
fn test_that_cpp_code_compiles() {
    let registry = get_registry();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.cpp");
    let mut source = File::create(&source_path).unwrap();
    cpp::output(&mut source, &registry).unwrap();

    let output = Command::new("clang++")
        .arg("--std=c++17")
        .arg("-c")
        .arg("-o")
        .arg(dir.path().join("test.o"))
        .arg("-I")
        .arg("runtime/cpp")
        .arg(source_path)
        .output()
        .unwrap();
    assert_eq!(String::new(), String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}

// Quick test using rustc directly.
#[test]
fn test_that_rust_code_compiles() {
    let registry = get_registry();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.rs");
    let mut source = File::create(&source_path).unwrap();
    rust::output(&mut source, /* with_derive_macros */ false, &registry).unwrap();

    let output = Command::new("rustc")
        .current_dir(dir.path())
        .arg("--crate-type")
        .arg("lib")
        .arg("--edition")
        .arg("2018")
        .arg(source_path)
        .output()
        .unwrap();
    assert_eq!(String::new(), String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}

// Full test using cargo. This may take a while.
#[test]
fn test_that_rust_code_compiles_with_derive_macros() {
    let registry = get_registry();
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
    let source_path = dir.path().join("src/lib.rs");
    let mut source = File::create(&source_path).unwrap();
    rust::output(&mut source, /* with_derive_macros */ true, &registry).unwrap();

    let output = Command::new("cargo")
        .current_dir(dir.path())
        .arg("build")
        .output()
        .unwrap();
    assert!(output.status.success());
}
