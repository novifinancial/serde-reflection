// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::Command;

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
