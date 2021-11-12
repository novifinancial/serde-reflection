// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use include_dir::include_dir as include_directory;
use serde_generate::{dart, test_utils, CodeGeneratorConfig, SourceInstaller};
use serde_reflection::Registry;
use std::fs::{copy, create_dir_all, read_dir, File};
use std::{io::Result, io::Write, path::Path, process::Command};
use tempfile::tempdir;
use test_utils::{Choice, Runtime, Test};

fn install_test_dependency(path: &Path) -> Result<()> {
    Command::new("dart")
        .current_dir(path)
        .env("PUB_CACHE", "../.pub-cache")
        .args(["pub", "add", "-d", "test"])
        .status()?;

    Ok(())
}

#[test]
fn test_dart_runtime_autotest() {
    let tests = include_directory!("runtime/dart/test");
    let tempdir = tempdir().unwrap();
    let source_path = tempdir.path().join("dart_project_autotest");

    let config = CodeGeneratorConfig::new("example".to_string());

    let installer = dart::Installer::new(source_path.clone());
    installer
        .install_module(&config, &Registry::default())
        .unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_bcs_runtime().unwrap();

    create_dir_all(source_path.join("test")).unwrap();
    for f in read_dir(tests.path().join("runtime/dart/test")).unwrap() {
        let file = f.unwrap();
        copy(
            file.path().to_path_buf(),
            source_path.join("test").join(file.file_name()),
        )
        .unwrap();
    }

    install_test_dependency(&source_path).unwrap();

    let dart_test = Command::new("dart")
        .current_dir(&source_path)
        .env("PUB_CACHE", "../.pub-cache")
        .args(["test"])
        .status()
        .unwrap();

    assert!(dart_test.success());
}

#[test]
fn test_dart_bcs_runtime_on_simple_data() {
    test_dart_runtime_on_simple_data(Runtime::Bcs);
}

#[test]
fn test_dart_bincode_runtime_on_simple_data() {
    test_dart_runtime_on_simple_data(Runtime::Bincode);
}

fn test_dart_runtime_on_simple_data(runtime: Runtime) {
    let tempdir = tempdir().unwrap();
    let source_path = tempdir
        .path()
        .join(format!("dart_project_{}", runtime.name().to_lowercase()));

    let registry = test_utils::get_simple_registry().unwrap();

    let config = CodeGeneratorConfig::new("example".to_string())
        .with_encodings(vec![runtime.into()])
        .with_c_style_enums(false);

    let installer = dart::Installer::new(source_path.clone());
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_bcs_runtime().unwrap();

    create_dir_all(source_path.join("test")).unwrap();

    install_test_dependency(&source_path).unwrap();

    let mut source = File::create(source_path.join("test/runtime_test.dart")).unwrap();
    writeln!(
        source,
        r#"
import 'dart:typed_data';
import 'package:example/example.dart';
import 'package:test/test.dart';
import 'package:tuple/tuple.dart';
import '../lib/src/bcs/bcs.dart';
import '../lib/src/bincode/bincode.dart';

void main() {{"#
    )
    .unwrap();
    let reference = runtime.serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    });

    writeln!(
        source,
        r#"
    test('{1} serialization matches deserialization', () {{
        final expectedBytes = Uint8List.fromList([{0}]);
        Test deserializedInstance = Test.{1}Deserialize(expectedBytes);

        Test expectedInstance = Test(
            a: [4, 6],
            b: Tuple2(-3, Uint64.parse('5')),
            c: ChoiceCItem(x: 7),
        );

        expect(deserializedInstance, equals(expectedInstance));

        final serializedBytes = expectedInstance.{1}Serialize();

        expect(serializedBytes, equals(expectedBytes));
    }});"#,
        reference
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<_>>()
            .join(", "),
        runtime.name().to_lowercase(),
    )
    .unwrap();

    writeln!(source, "}}").unwrap();

    let dart_test = Command::new("dart")
        .current_dir(&source_path)
        .env("PUB_CACHE", "../.pub-cache")
        .args(["test", "test/runtime_test.dart"])
        .status()
        .unwrap();

    assert!(dart_test.success());
}
