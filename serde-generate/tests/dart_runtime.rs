// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use include_dir::include_dir as include_directory;
use serde_generate::{dart, test_utils, CodeGeneratorConfig, Encoding, SourceInstaller};
use std::{io::Result, path::Path, process::Command};
use tempfile::tempdir;

fn install_test_dependency(path: &Path) -> Result<()> {
    Command::new("dart")
        .current_dir(path)
        .env("PUB_CACHE", "../.pub-cache")
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
        .with_c_style_enums(false);

    let installer = dart::Installer::new(source_path.clone());
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_bcs_runtime().unwrap();

    install_test_dependency(&source_path).unwrap();

    copy_runtime_test(Encoding::Bcs, &config, &source_path, false);

    let dart_bcs_test = Command::new("dart")
        .current_dir(&source_path)
        .env("PUB_CACHE", "../.pub-cache")
        .args(["test", "test/runtime_test.dart"])
        .status()
        .unwrap();

    assert!(dart_bcs_test.success());

    copy_runtime_test(Encoding::Bincode, &config, &source_path, false);

    let dart_bincode_test = Command::new("dart")
        .current_dir(&source_path)
        .env("PUB_CACHE", "../.pub-cache")
        .args(["test", "test/runtime_test.dart"])
        .status()
        .unwrap();

    assert!(dart_bincode_test.success());
}

#[test]
fn test_dart_runtime_with_c_enums() {
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

    copy_runtime_test(Encoding::Bcs, &config, &source_path, true);

    let dart_bcs_test = Command::new("dart")
        .current_dir(&source_path)
        .env("PUB_CACHE", "../.pub-cache")
        .args(["test", "test/runtime_test.dart"])
        .status()
        .unwrap();

    assert!(dart_bcs_test.success());

    copy_runtime_test(Encoding::Bincode, &config, &source_path, true);

    let dart_bincode_test = Command::new("dart")
        .current_dir(&source_path)
        .env("PUB_CACHE", "../.pub-cache")
        .args(["test", "test/runtime_test.dart"])
        .status()
        .unwrap();

    assert!(dart_bincode_test.success());
}

fn copy_runtime_test(
    encoding: Encoding,
    config: &CodeGeneratorConfig,
    source_path: &std::path::PathBuf,
    with_c_style_enums: bool,
) {
    let tests = include_directory!("runtime/dart/test");
    let mut tmpl =
        std::fs::read_to_string(tests.path().join("runtime/dart/test/runtime.dart")).unwrap();

    tmpl = tmpl.replace(
        "<package_path>",
        &format!(
            "import 'package:{name}/{name}.dart';",
            name = &config.module_name()
        ),
    );
    tmpl = if with_c_style_enums {
        tmpl.replace(
            "<enum_test>",
            r#"test('C Enum', () {
        final val = CStyleEnum.a;
        expect(
            CStyleEnumExtension.<encoding>Deserialize(val.<encoding>Serialize()),
            equals(val));
      });"#,
        )
    } else {
        tmpl.replace(
            "<enum_test>",
            r#"test('Enum', () {
        final val = CStyleEnumAItem();
        expect(CStyleEnum.<encoding>Deserialize(val.<encoding>Serialize()), equals(val));
      });"#,
        )
    };

    tmpl = tmpl.replace("<encoding>", encoding.name());

    std::fs::write(source_path.join("test/runtime_test.dart"), tmpl).unwrap();
}
