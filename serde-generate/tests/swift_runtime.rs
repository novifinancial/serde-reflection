// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{
    swift, test_utils,
    test_utils::{Choice, Runtime, Test},
    CodeGeneratorConfig, SourceInstaller,
};
use std::{fs::File, io::Write, process::Command, sync::Mutex};

lazy_static::lazy_static! {
    // Avoid interleaving compiler calls because the output gets very messy.
    static ref MUTEX: Mutex<()> = Mutex::new(());
}

#[test]
fn test_swift_runtime_autotests() {
    let runtime_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../../serde-generate/runtime/swift");

    let status = Command::new("swift")
        .current_dir(runtime_path.to_str().unwrap())
        .arg("test")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_swift_bcs_runtime_on_simple_data() {
    test_swift_runtime_on_simple_data(Runtime::Bcs);
}

#[test]
fn test_swift_bincode_runtime_on_simple_data() {
    test_swift_runtime_on_simple_data(Runtime::Bincode);
}

fn test_swift_runtime_on_simple_data(runtime: Runtime) {
    // To see the source, uncomment this and replace `dir.path()` by `my_path` below.
    // let my_path = std::path::Path::new("../test");
    // std::fs::remove_dir_all(my_path).unwrap_or(());
    // std::fs::create_dir_all(my_path).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let config =
        CodeGeneratorConfig::new("Testing".to_string()).with_encodings(vec![runtime.into()]);
    let registry = test_utils::get_simple_registry().unwrap();
    let installer = swift::Installer::new(dir.path().to_path_buf());
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap(); // also installs bcs and bincode

    let reference = runtime.serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    });

    std::fs::create_dir_all(dir.path().join("Sources/main")).unwrap();
    let main_path = dir.path().join("Sources/main/main.swift");
    let mut main = File::create(main_path).unwrap();
    writeln!(
        main,
        r#"
import Serde
import Testing

var input : [UInt8] = [{0}]
let value = try Test.{1}Deserialize(input: input)

let value2 = Test.init(
    a: [4, 6],
    b: Tuple2.init(-3, 5),
    c: Choice.C(x: 7)
)
assert(value == value2, "value != value2")

let output = try value2.{1}Serialize()
assert(input == output, "input != output")

input += [0]
do {{
    let _ = try Test.{1}Deserialize(input: input)
    assertionFailure("Was expecting an error")
}}
catch {{}}

do {{
    let input2 : [UInt8] = [0, 1]
    let _ = try Test.{1}Deserialize(input: input2)
    assertionFailure("Was expecting an error")
}}
catch {{}}
"#,
        reference
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<_>>()
            .join(", "),
        runtime.name(),
    )
    .unwrap();

    let mut file = File::create(dir.path().join("Package.swift")).unwrap();
    write!(
        file,
        r#"// swift-tools-version:5.3

import PackageDescription

let package = Package(
    name: "Testing",
    targets: [
        .target(
            name: "Serde",
            dependencies: []),
        .target(
            name: "Testing",
            dependencies: ["Serde"]),
        .target(
            name: "main",
            dependencies: ["Serde", "Testing"]
        ),
    ]
)
"#
    )
    .unwrap();

    {
        let _lock = MUTEX.lock().unwrap();
        let status = Command::new("swift")
            .current_dir(dir.path())
            .arg("run")
            .status()
            .unwrap();
        assert!(status.success());
    }
}
