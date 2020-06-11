// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::test_utils;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_that_installed_python_code_parses() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let yaml_path = dir.path().join("test.yaml");
    std::fs::write(yaml_path.clone(), serde_yaml::to_string(&registry).unwrap()).unwrap();

    Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("serde-generate")
        .arg("--")
        .arg("--language")
        .arg("python3")
        .arg("--target-source-dir")
        .arg(dir.path())
        .arg("--name")
        .arg("test_types")
        .arg("--with-runtimes")
        .arg("serde")
        .arg("bincode")
        .arg("--")
        .arg(yaml_path)
        .status()
        .unwrap();

    let python_path = format!(
        "{}:{}",
        std::env::var("PYTHONPATH").unwrap_or_default(),
        dir.path().to_string_lossy(),
    );
    let output = Command::new("python3")
        .arg("-c")
        .arg("import serde_types; import bincode; import test_types")
        .env("PYTHONPATH", python_path)
        .output()
        .unwrap();
    assert_eq!(String::new(), String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}

#[test]
fn test_that_installed_rust_code_compiles() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let yaml_path = dir.path().join("test.yaml");
    std::fs::write(yaml_path.clone(), serde_yaml::to_string(&registry).unwrap()).unwrap();

    Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("serde-generate")
        .arg("--")
        .arg("--language")
        .arg("rust")
        .arg("--target-source-dir")
        .arg(dir.path())
        .arg(yaml_path)
        .status()
        .unwrap();

    // Use a stable `target` dir to avoid downloading and recompiling crates everytime.
    let target_dir = std::env::current_dir().unwrap().join("../target");
    let status = Command::new("cargo")
        .current_dir(dir.path().join("test"))
        .arg("build")
        .arg("--target-dir")
        .arg(target_dir)
        .status()
        .unwrap();
    assert!(status.success());
}
