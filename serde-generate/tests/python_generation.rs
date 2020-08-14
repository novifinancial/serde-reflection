// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{python3, test_utils, CodeGeneratorConfig, SourceInstaller};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_that_python_code_parses() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    let source_path = dir.path().join("test.py");
    let mut source = File::create(&source_path).unwrap();

    let config = CodeGeneratorConfig::new("testing".to_string());
    let generator = python3::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

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

#[test]
fn test_that_python_code_parses_with_codegen_options() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    let source_path = dir.path().join("test.py");
    let mut source = File::create(&source_path).unwrap();

    let config = CodeGeneratorConfig::new("testing".to_string()).with_comments(
        vec![
            (
                vec!["testing".to_string(), "SerdeData".to_string()],
                "Some\ncomments".to_string(),
            ),
            (
                vec![
                    "testing".to_string(),
                    "List".to_string(),
                    "Node".to_string(),
                ],
                "Some other comments".to_string(),
            ),
        ]
        .into_iter()
        .collect(),
    );
    let generator = python3::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    // Check that comments were correctly generated.
    let content = std::fs::read_to_string(&source_path).unwrap();
    assert!(content.contains(
        r#"
    """Some
    comments
    """
"#
    ));
    assert!(content.contains(
        r#"
    """Some other comments
    """
"#
    ));

    let python_path = format!(
        "{}:runtime/python",
        std::env::var("PYTHONPATH").unwrap_or_default()
    );
    let status = Command::new("python3")
        .arg(source_path.clone())
        .env("PYTHONPATH", python_path)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_python_code_with_external_definitions() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    let source_path = dir.path().join("test.py");
    let mut source = File::create(&source_path).unwrap();

    // Pretend that "Tree" is external.
    let mut definitions = BTreeMap::new();
    definitions.insert("pkg.foo".to_string(), vec!["Tree".to_string()]);
    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_external_definitions(definitions);
    let generator = python3::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    let content = std::fs::read_to_string(&source_path).unwrap();
    assert!(content.contains("from pkg import foo"));
    assert!(content.contains("value: foo.Tree"));
    assert!(!content.contains("value: Tree"));
}

#[test]
fn test_that_installed_python_code_passes_pyre_check() {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    let config = CodeGeneratorConfig::new("testing".to_string());
    let installer = python3::Installer::new(dir.path().join("src"), /* serde package */ None);
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_lcs_runtime().unwrap();

    // Sadly, we have to manage numpy typeshed manually for now until the next release of numpy.
    let status = Command::new("cp")
        .arg("-r")
        .arg("runtime/python/typeshed")
        .arg(dir.path())
        .status()
        .unwrap();
    assert!(status.success());

    let mut pyre_config = File::create(dir.path().join(".pyre_configuration")).unwrap();
    writeln!(
        &mut pyre_config,
        r#"{{
  "source_directories": [
    "src"
  ],
  "search_path": [
    "typeshed"
  ]
}}"#,
    )
    .unwrap();

    let status = Command::new("pyre")
        .current_dir(dir.path())
        .arg("check")
        .status()
        .unwrap();
    assert!(status.success());
}
