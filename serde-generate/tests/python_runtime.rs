// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(feature = "runtime-testing")]

use serde_generate::{
    python3, test_utils,
    test_utils::{Choice, Runtime, Test},
    CodeGeneratorConfig,
};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_python_lcs_runtime_on_simple_data() {
    test_python_runtime_on_simple_data(Runtime::Lcs);
}

#[test]
fn test_python_bincode_runtime_on_simple_data() {
    test_python_runtime_on_simple_data(Runtime::Bincode);
}

fn test_python_runtime_on_simple_data(runtime: Runtime) {
    let registry = test_utils::get_simple_registry().unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.py");
    let mut source = File::create(&source_path).unwrap();

    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_encodings(vec![runtime.into()]);
    let generator = python3::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    let reference = runtime.serialize(&Test {
        a: vec![4, 6],
        b: (3, 5),
        c: Choice::C { x: 7 },
    });
    writeln!(
        source,
        r#"
value = Test([4, 6], (3, 5), Choice__C(7))

s = value.{0}_serialize()
assert s == bytes.fromhex("{1}")

v = Test.{0}_deserialize(s)
assert v == value
assert v.c.x == 7

v.b = (3, 0)
t = v.{0}_serialize()
assert len(t) == len(s)
assert t != s

seen_error = False
try:
    Test.{0}_deserialize(bytes.fromhex("{1}") + bytes([0]))
except st.DeserializationError:
    seen_error = True
assert seen_error
"#,
        runtime.name(),
        hex::encode(&reference),
    )
    .unwrap();

    let python_path = std::env::var("PYTHONPATH").unwrap_or_default() + ":runtime/python";
    let status = Command::new("python3")
        .arg(source_path)
        .env("PYTHONPATH", python_path)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_python_lcs_runtime_on_all_supported_types() {
    test_python_runtime_on_all_supported_types(Runtime::Lcs);
}

#[test]
fn test_python_bincode_runtime_on_all_supported_types() {
    test_python_runtime_on_all_supported_types(Runtime::Bincode);
}

fn test_python_runtime_on_all_supported_types(runtime: Runtime) {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.py");
    let mut source = File::create(&source_path).unwrap();

    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_encodings(vec![runtime.into()]);
    let generator = python3::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    let values = test_utils::get_sample_values();
    let hex_encodings: Vec<_> = values
        .iter()
        .map(|v| format!("'{}'", hex::encode(&runtime.serialize(&v))))
        .collect();

    writeln!(
        source,
        r#"
from copy import copy
import serde_types as st
encodings = [bytes.fromhex(s) for s in [{1}]]

for encoding in encodings:
    v = SerdeData.{0}_deserialize(encoding)
    s = v.{0}_serialize()
    assert s == encoding

    for i in range(len(encoding)):
        encoding2 = bytearray(encoding)
        encoding2[i] ^= 0x80
        try:
            v2 = SerdeData.{0}_deserialize(encoding2)
            assert v2 != v
        except st.DeserializationError:
            pass
"#,
        runtime.name(),
        hex_encodings.join(", ")
    )
    .unwrap();

    let python_path = format!(
        "{}:runtime/python",
        std::env::var("PYTHONPATH").unwrap_or_default()
    );
    let status = Command::new("python3")
        .arg(source_path)
        .env("PYTHONPATH", python_path)
        .status()
        .unwrap();
    assert!(status.success());
}
