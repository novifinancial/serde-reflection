// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(feature = "runtime-testing")]

use serde_generate::{
    cpp, test_utils,
    test_utils::{Choice, Runtime, Test},
    CodeGeneratorConfig,
};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cpp_lcs_runtime_on_simple_date() {
    test_cpp_runtime_on_simple_date(Runtime::Lcs);
}

#[test]
fn test_cpp_bincode_runtime_on_simple_date() {
    test_cpp_runtime_on_simple_date(Runtime::Bincode);
}

fn test_cpp_runtime_on_simple_date(runtime: Runtime) {
    let registry = test_utils::get_simple_registry().unwrap();
    let dir = tempdir().unwrap();
    let header_path = dir.path().join("test.hpp");
    let mut header = File::create(&header_path).unwrap();

    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_encodings(vec![runtime.into()]);
    let generator = cpp::CodeGenerator::new(&config);
    generator.output(&mut header, &registry).unwrap();

    let reference = runtime.serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    });

    let source_path = dir.path().join("test.cpp");
    let mut source = File::create(&source_path).unwrap();
    writeln!(
        source,
        r#"
#include <cassert>
#include "test.hpp"

using namespace testing;

int main() {{
    std::vector<uint8_t> input = {{{0}}};
    auto test = Test::{1}Deserialize(input);

    auto a = std::vector<uint32_t> {{4, 6}};
    auto b = std::tuple<int64_t, uint64_t> {{-3, 5}};
    auto c = Choice {{ Choice::C {{ 7 }} }};
    auto test2 = Test {{a, b, c}};

    assert(test == test2);

    auto output = test2.{1}Serialize();

    assert(input == output);

    input.push_back(1);
    try {{
        Test::{1}Deserialize(input);
    }} catch (...) {{
        return 0;
    }}
    return 1;
}}
"#,
        reference
            .iter()
            .map(|x| format!("0x{:02x}", x))
            .collect::<Vec<_>>()
            .join(", "),
        runtime.name(),
    )
    .unwrap();

    let status = Command::new("clang++")
        .arg("--std=c++17")
        .arg("-o")
        .arg(dir.path().join("test"))
        .arg("-I")
        .arg("runtime/cpp")
        .arg(source_path)
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new(dir.path().join("test")).status().unwrap();
    assert!(status.success());
}

#[test]
fn test_cpp_lcs_runtime_on_supported_types() {
    test_cpp_runtime_on_supported_types(Runtime::Lcs);
}

#[test]
fn test_cpp_bincode_runtime_on_supported_types() {
    test_cpp_runtime_on_supported_types(Runtime::Bincode);
}

fn test_cpp_runtime_on_supported_types(runtime: Runtime) {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let header_path = dir.path().join("test.hpp");
    let mut header = File::create(&header_path).unwrap();

    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_encodings(vec![runtime.into()]);
    let generator = cpp::CodeGenerator::new(&config);
    generator.output(&mut header, &registry).unwrap();

    let values = test_utils::get_sample_values();
    let encodings = values
        .iter()
        .map(|v| {
            let bytes = runtime.serialize(&v);
            format!(
                "std::vector<uint8_t>{{{}}}",
                bytes
                    .iter()
                    .map(|x| format!("0x{:02x}", x))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let source_path = dir.path().join("test.cpp");
    let mut source = File::create(&source_path).unwrap();
    writeln!(
        source,
        r#"
#include <iostream>
#include <cassert>
#include "test.hpp"

using namespace testing;

int main() {{
    try {{
        std::vector<std::vector<uint8_t>> inputs = {{{0}}};

        for (auto input: inputs) {{
            auto test = SerdeData::{1}Deserialize(input);
            auto output = test.{1}Serialize();
            assert(input == output);

            // Test simple mutations of the input.
            for (int i = 0; i < input.size(); i++) {{
                auto input2 = input;
                input2[i] ^= 0x80;
                try {{
                    auto test2 = SerdeData::{1}Deserialize(input2);
                    assert(!(test2 == test));
                }} catch (serde::deserialization_error e) {{
                    // All good
                }} catch (std::bad_alloc const &e) {{
                    // All good
                }} catch (std::length_error const &e) {{
                    // All good
                }}
            }}
        }}
        return 0;
    }} catch (char const* e) {{
        std::cout << "Error: " << e << '\n';
        return -1;
    }}
}}
"#,
        encodings,
        runtime.name(),
    )
    .unwrap();

    let status = Command::new("clang++")
        .arg("--std=c++17")
        .arg("-g")
        .arg("-o")
        .arg(dir.path().join("test"))
        .arg("-I")
        .arg("runtime/cpp")
        .arg(source_path)
        .status()
        .unwrap();
    assert!(status.success());

    // Use `.status()` instead of `.output()` to debug error outputs.
    let output = Command::new(dir.path().join("test")).output().unwrap();
    assert!(output.status.success());
}
