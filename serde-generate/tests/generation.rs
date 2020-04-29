// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::python3;
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
