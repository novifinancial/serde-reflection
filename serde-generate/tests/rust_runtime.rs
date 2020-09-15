// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(feature = "runtime-testing")]

use serde_generate::{rust, test_utils, test_utils::Runtime, CodeGeneratorConfig};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_rust_lcs_runtime() {
    test_rust_runtime(Runtime::Lcs);
}

#[test]
fn test_rust_bincode_runtime() {
    test_rust_runtime(Runtime::Bincode);
}

// Full test using cargo. This may take a while.
fn test_rust_runtime(runtime: Runtime) {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let mut file = std::fs::File::create(dir.path().join("Cargo.toml")).unwrap();
    write!(
        &mut file,
        r#"[package]
name = "testing2"
version = "0.1.0"
edition = "2018"

[dependencies]
hex = "0.4.2"
serde = {{ version = "1.0.112", features = ["derive"] }}
serde_bytes = "0.11"
{}

[workspace]
"#,
        runtime.rust_package()
    )
    .unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();

    let config = CodeGeneratorConfig::new("testing".to_string());
    let generator = rust::CodeGenerator::new(&config);

    let source_path = dir.path().join("src/main.rs");
    let mut source = File::create(&source_path).unwrap();
    generator.output(&mut source, &registry).unwrap();

    let values = test_utils::get_sample_values(runtime.has_canonical_maps());
    let hex_encodings: Vec<_> = values
        .iter()
        .map(|v| format!("\"{}\"", hex::encode(&runtime.serialize(&v))))
        .collect();

    writeln!(
        source,
        r#"
fn main() {{
    let hex_encodings = vec![{}];

    for hex_encoding in hex_encodings {{
        let encoding = hex::decode(hex_encoding).unwrap();
        let value = {}::<SerdeData>(&encoding).unwrap();

        let s = {}(&value).unwrap();
        assert_eq!(s, encoding);
    }}
}}
"#,
        hex_encodings.join(", "),
        runtime.quote_deserialize(),
        runtime.quote_serialize(),
    )
    .unwrap();

    // Use a stable `target` dir to avoid downloading and recompiling crates everytime.
    let target_dir = std::env::current_dir().unwrap().join("../target");
    let status = Command::new("cargo")
        .current_dir(dir.path())
        .arg("run")
        .arg("--target-dir")
        .arg(target_dir)
        .status()
        .unwrap();
    assert!(status.success());
}