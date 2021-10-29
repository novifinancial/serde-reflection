// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{dart, test_utils, CodeGeneratorConfig, Encoding, SourceInstaller};
use std::{io::Result, path::Path, process::Command};
use tempfile::tempdir;

fn install_test_dependency(path: &Path) -> Result<()> {
    Command::new("dart")
        .current_dir(path)
        .args(["pub", "add", "-d", "test"])
        .status()?;

    Ok(())
}
#[test]
fn test_dart_runtime() {
    let tempdir = tempdir().unwrap();
    let source_path = tempdir.path().join("dart_project");

    let registry = test_utils::get_registry().unwrap();

    let config = CodeGeneratorConfig::new("example".to_string())
        .with_encodings(vec![Encoding::Bcs, Encoding::Bincode])
        .with_c_style_enums(true);

    let installer = dart::Installer::new(source_path.clone());
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_bcs_runtime().unwrap();

    install_test_dependency(&source_path).unwrap();

    let dart_test = Command::new("dart")
        .current_dir(source_path)
        .args(["test"])
        .status()
        .unwrap();

    assert!(dart_test.success());
}
