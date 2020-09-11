// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(feature = "runtime-testing")]

use heck::CamelCase;
use serde_generate::{
    golang, test_utils,
    test_utils::{Choice, Runtime, Test},
    CodeGeneratorConfig,
};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_golang_runtime_autotests() {
    let runtime_mod_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../../serde-generate/runtime/golang");

    let status = Command::new("go")
        .current_dir(runtime_mod_path.to_str().unwrap())
        .arg("test")
        .arg("./...")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_golang_lcs_runtime_on_simple_data() {
    test_golang_runtime_on_simple_data(Runtime::Lcs);
}

#[test]
fn test_golang_bincode_runtime_on_simple_data() {
    test_golang_runtime_on_simple_data(Runtime::Bincode);
}

fn test_golang_runtime_on_simple_data(runtime: Runtime) {
    let registry = test_utils::get_simple_registry().unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.go");
    let mut source = File::create(&source_path).unwrap();

    let config = CodeGeneratorConfig::new("main".to_string())
        .with_encodings(vec![runtime.into()])
        .with_external_definitions(
            vec![("github.com/google/go-cmp/cmp".to_string(), vec![])]
                .into_iter()
                .collect(),
        );
    let generator = golang::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    let reference = runtime.serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    });

    writeln!(
        source,
        r#"
func main() {{
	input := []byte{{{0}}}
	test, err := {1}DeserializeTest(input)
	if err != nil {{ panic("failed to deserialize") }}

        test2 := Test {{
		A: []uint32{{ 4, 6 }},
		B: struct {{ Field0 int64; Field1 uint64 }} {{ -3, 5 }},
		C: &Choice__C {{ X: 7 }},
	}}
	if !cmp.Equal(test, test2) {{ panic("test != test2") }}

	output, err := test2.{1}Serialize()
	if err != nil {{ panic("failed to serialize") }}
	if !cmp.Equal(input, output) {{ panic("input != output") }}

	input2 := []byte{{{0}, 1}}
	test2, err2 := {1}DeserializeTest(input2)
	if err2 == nil {{ panic("was expecting an error") }}
}}
"#,
        reference
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<_>>()
            .join(", "),
        runtime.name().to_camel_case(),
    )
    .unwrap();

    let status = Command::new("go")
        .current_dir(dir.path())
        .arg("mod")
        .arg("init")
        .arg("testing")
        .status()
        .unwrap();
    assert!(status.success());

    let runtime_mod_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../../serde-generate/runtime/golang");
    let status = Command::new("go")
        .current_dir(dir.path())
        .arg("mod")
        .arg("edit")
        .arg("-replace")
        .arg(format!(
            "github.com/novifinancial/serde-reflection/serde-generate/runtime/golang={}",
            runtime_mod_path.to_str().unwrap()
        ))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("go")
        .current_dir(dir.path())
        .arg("run")
        .arg(source_path.clone())
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_golang_lcs_runtime_on_supported_types() {
    test_golang_runtime_on_supported_types(Runtime::Lcs);
}

#[test]
fn test_golang_bincode_runtime_on_supported_types() {
    test_golang_runtime_on_supported_types(Runtime::Bincode);
}

fn test_golang_runtime_on_supported_types(runtime: Runtime) {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.go");
    let mut source = File::create(&source_path).unwrap();

    let config = CodeGeneratorConfig::new("main".to_string())
        .with_encodings(vec![runtime.into()])
        .with_external_definitions(
            vec![("github.com/google/go-cmp/cmp".to_string(), vec![])]
                .into_iter()
                .collect(),
        );
    let generator = golang::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    let values = test_utils::get_sample_values(runtime.has_canonical_maps());
    let encodings = values
        .iter()
        .map(|v| {
            let bytes = runtime.serialize(&v);
            format!(
                "{{{}}}",
                bytes
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    writeln!(
        source,
        r#"
func main() {{
	inputs := [][]byte{{{0}}}

	for _, input := range(inputs) {{
		test, err := {1}DeserializeSerdeData(input)
		if err != nil {{ panic(fmt.Sprintf("failed to deserialize input: %v", err)) }}
		output, err := test.{1}Serialize()
		if err != nil {{ panic(fmt.Sprintf("failed to serialize: %v", err)) }}
		if !cmp.Equal(input, output) {{ panic(fmt.Sprintf("input != output:\n  %v\n  %v", input, output)) }}
		test2, err := {1}DeserializeSerdeData(output)
		if err != nil {{ panic(fmt.Sprintf("failed to deserialize output: %v", err)) }}
		if !cmp.Equal(test, test2) {{ panic(fmt.Sprintf("test != test2:\n  %v\n  %v", test, test2)) }}
	}}
}}
"#,
        encodings,
        runtime.name().to_camel_case(),
    )
    .unwrap();

    let status = Command::new("go")
        .current_dir(dir.path())
        .arg("mod")
        .arg("init")
        .arg("testing")
        .status()
        .unwrap();
    assert!(status.success());

    let runtime_mod_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../../serde-generate/runtime/golang");
    let status = Command::new("go")
        .current_dir(dir.path())
        .arg("mod")
        .arg("edit")
        .arg("-replace")
        .arg(format!(
            "github.com/novifinancial/serde-reflection/serde-generate/runtime/golang={}",
            runtime_mod_path.to_str().unwrap()
        ))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("go")
        .current_dir(dir.path())
        .arg("run")
        .arg(source_path)
        .status()
        .unwrap();
    assert!(status.success());
}
