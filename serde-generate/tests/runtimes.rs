// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use heck::CamelCase;
use libra_canonical_serialization as lcs;
use serde::{Deserialize, Serialize};
use serde_generate::{cpp, golang, java, python3, rust, test_utils, CodeGeneratorConfig, Encoding};
use serde_reflection::{Registry, Result, Samples, Tracer, TracerConfig};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[derive(Serialize, Deserialize)]
struct Test {
    a: Vec<u32>,
    b: (i64, u64),
    c: Choice,
}

#[derive(Serialize, Deserialize)]
enum Choice {
    A,
    B(u64),
    C { x: u8 },
}

fn get_local_registry() -> Result<Registry> {
    let mut tracer = Tracer::new(TracerConfig::default());
    let samples = Samples::new();
    tracer.trace_type::<Test>(&samples)?;
    tracer.trace_type::<Choice>(&samples)?;
    Ok(tracer.registry()?)
}

#[derive(Copy, Clone)]
enum Runtime {
    Lcs,
    Bincode,
}

impl std::convert::Into<Encoding> for Runtime {
    fn into(self) -> Encoding {
        match self {
            Runtime::Lcs => Encoding::Lcs,
            Runtime::Bincode => Encoding::Bincode,
        }
    }
}

impl Runtime {
    fn name(self) -> &'static str {
        <Self as std::convert::Into<Encoding>>::into(self).name()
    }

    fn rust_package(self) -> &'static str {
        match self {
            Self::Lcs => "lcs = { git = \"https://github.com/libra/libra.git\", branch = \"testnet\", package = \"libra-canonical-serialization\" }",
            Self::Bincode => "bincode = \"1.2\"",
        }
    }

    fn serialize<T>(self, value: &T) -> Vec<u8>
    where
        T: serde::Serialize,
    {
        match self {
            Self::Lcs => lcs::to_bytes(value).unwrap(),
            Self::Bincode => bincode::serialize(value).unwrap(),
        }
    }

    fn quote_serialize(self) -> &'static str {
        match self {
            Self::Lcs => "lcs::to_bytes",
            Self::Bincode => "bincode::serialize",
        }
    }

    fn quote_deserialize(self) -> &'static str {
        match self {
            Self::Lcs => "lcs::from_bytes",
            Self::Bincode => "bincode::deserialize",
        }
    }
}

#[test]
fn test_python_lcs_runtime_on_simple_data() {
    test_python_runtime_on_simple_data(Runtime::Lcs);
}

#[test]
fn test_python_bincode_runtime_on_simple_data() {
    test_python_runtime_on_simple_data(Runtime::Bincode);
}

fn test_python_runtime_on_simple_data(runtime: Runtime) {
    let registry = get_local_registry().unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.py");
    let mut source = File::create(&source_path).unwrap();

    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_encodings(vec![runtime.into()]);
    let generator = python3::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    let reference = runtime.serialize(&Test {
        a: vec![4, 6],
        b: (3, 5),
        c: Choice::C { x: 7 },
    });
    writeln!(
        source,
        r#"
value = Test([4, 6], (3, 5), Choice__C(7))

s = value.{0}_serialize()
assert s == bytes.fromhex("{1}")

v = Test.{0}_deserialize(s)
assert v == value
assert v.c.x == 7

v.b = (3, 0)
t = v.{0}_serialize()
assert len(t) == len(s)
assert t != s

seen_error = False
try:
    Test.{0}_deserialize(bytes.fromhex("{1}") + bytes([0]))
except ValueError:
    seen_error = True
assert seen_error
"#,
        runtime.name(),
        hex::encode(&reference),
    )
    .unwrap();

    let python_path = std::env::var("PYTHONPATH").unwrap_or_default() + ":runtime/python";
    let status = Command::new("python3")
        .arg(source_path)
        .env("PYTHONPATH", python_path)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_python_lcs_runtime_on_all_supported_types() {
    test_python_runtime_on_all_supported_types(Runtime::Lcs);
}

#[test]
fn test_python_bincode_runtime_on_all_supported_types() {
    test_python_runtime_on_all_supported_types(Runtime::Bincode);
}

fn test_python_runtime_on_all_supported_types(runtime: Runtime) {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("test.py");
    let mut source = File::create(&source_path).unwrap();

    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_encodings(vec![runtime.into()]);
    let generator = python3::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    let values = test_utils::get_sample_values();
    let hex_encodings: Vec<_> = values
        .iter()
        .map(|v| format!("'{}'", hex::encode(&runtime.serialize(&v))))
        .collect();

    writeln!(
        source,
        r#"
encodings = [bytes.fromhex(s) for s in [{1}]]

for encoding in encodings:
    v = SerdeData.{0}_deserialize(encoding)
    s = v.{0}_serialize()
    assert s == encoding
"#,
        runtime.name(),
        hex_encodings.join(", ")
    )
    .unwrap();

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
fn test_rust_lcs_runtime() {
    test_rust_runtime(Runtime::Lcs);
}

#[test]
fn test_rust_bincode_runtime() {
    test_rust_runtime(Runtime::Bincode);
}

// Full test using cargo. This may take a while.
fn test_rust_runtime(runtime: Runtime) {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    let mut file = std::fs::File::create(dir.path().join("Cargo.toml")).unwrap();
    write!(
        &mut file,
        r#"[package]
name = "testing2"
version = "0.1.0"
edition = "2018"

[dependencies]
hex = "0.4.2"
serde = {{ version = "1.0.112", features = ["derive"] }}
serde_bytes = "0.11"
{}

[workspace]
"#,
        runtime.rust_package()
    )
    .unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();

    let config = CodeGeneratorConfig::new("testing".to_string());
    let generator = rust::CodeGenerator::new(&config);

    let source_path = dir.path().join("src/main.rs");
    let mut source = File::create(&source_path).unwrap();
    generator.output(&mut source, &registry).unwrap();

    let values = test_utils::get_sample_values();
    let hex_encodings: Vec<_> = values
        .iter()
        .map(|v| format!("\"{}\"", hex::encode(&runtime.serialize(&v))))
        .collect();

    writeln!(
        source,
        r#"
fn main() {{
    let hex_encodings = vec![{}];

    for hex_encoding in hex_encodings {{
        let encoding = hex::decode(hex_encoding).unwrap();
        let value = {}::<SerdeData>(&encoding).unwrap();

        let s = {}(&value).unwrap();
        assert_eq!(s, encoding);
    }}
}}
"#,
        hex_encodings.join(", "),
        runtime.quote_deserialize(),
        runtime.quote_serialize(),
    )
    .unwrap();

    // Use a stable `target` dir to avoid downloading and recompiling crates everytime.
    let target_dir = std::env::current_dir().unwrap().join("../target");
    let status = Command::new("cargo")
        .current_dir(dir.path())
        .arg("run")
        .arg("--target-dir")
        .arg(target_dir)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_cpp_lcs_runtime_on_simple_date() {
    test_cpp_runtime_on_simple_date(Runtime::Lcs);
}

#[test]
fn test_cpp_bincode_runtime_on_simple_date() {
    test_cpp_runtime_on_simple_date(Runtime::Bincode);
}

fn test_cpp_runtime_on_simple_date(runtime: Runtime) {
    let registry = get_local_registry().unwrap();
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

    let status = Command::new(dir.path().join("test")).status().unwrap();
    assert!(status.success());
}

#[test]
fn test_java_lcs_runtime_on_simple_data() {
    test_java_runtime_on_simple_data(Runtime::Lcs);
}

#[test]
fn test_java_bincode_runtime_on_simple_data() {
    test_java_runtime_on_simple_data(Runtime::Bincode);
}

fn test_java_runtime_on_simple_data(runtime: Runtime) {
    let registry = get_local_registry().unwrap();
    let dir = tempdir().unwrap();

    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_encodings(vec![runtime.into()]);
    let generator = java::CodeGenerator::new(&config);
    generator
        .write_source_files(dir.path().to_path_buf(), &registry)
        .unwrap();

    let reference = runtime.serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    });

    let mut source = File::create(&dir.path().join("Main.java")).unwrap();
    writeln!(
        source,
        r#"
import java.util.List;
import java.util.Arrays;
import com.facebook.serde.Unsigned;
import com.facebook.serde.Tuple2;
import testing.Choice;
import testing.Test;

public class Main {{
    public static void main(String[] args) throws java.lang.Exception {{
        byte[] input = new byte[] {{{0}}};

        Test test = Test.{1}Deserialize(input);

        List<@Unsigned Integer> a = Arrays.asList(4, 6);
        Tuple2<Long, @Unsigned Long> b = new Tuple2<>(Long.valueOf(-3), Long.valueOf(5));
        Choice c = new Choice.C(Byte.valueOf((byte) 7));
        Test test2 = new Test(a, b, c);

        assert test.equals(test2);

        byte[] output = test2.{1}Serialize();

        assert java.util.Arrays.equals(input, output);

        byte[] input2 = new byte[] {{{0}, 1}};
        try {{
            Test.{1}Deserialize(input2);
        }} catch (Exception e) {{
            return;
        }}
        assert false;
    }}
}}
"#,
        reference
            .iter()
            .map(|x| format!("{}", *x as i8))
            .collect::<Vec<_>>()
            .join(", "),
        runtime.name(),
    )
    .unwrap();

    let paths = std::iter::empty()
        .chain(std::fs::read_dir("runtime/java/com/facebook/serde").unwrap())
        .chain(
            std::fs::read_dir("runtime/java/com/facebook/".to_string() + runtime.name()).unwrap(),
        )
        .chain(std::fs::read_dir(dir.path().join("testing")).unwrap())
        .map(|e| e.unwrap().path());
    let status = Command::new("javac")
        .arg("-Xlint")
        .arg("-d")
        .arg(dir.path())
        .args(paths)
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("javac")
        .arg("-Xlint")
        .arg("-cp")
        .arg(dir.path())
        .arg("-d")
        .arg(dir.path())
        .arg(dir.path().join("Main.java"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("java")
        .arg("-enableassertions")
        .arg("-cp")
        .arg(dir.path())
        .arg("Main")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_java_lcs_runtime_on_supported_types() {
    test_java_runtime_on_supported_types(Runtime::Lcs);
}

#[test]
fn test_java_bincode_runtime_on_supported_types() {
    test_java_runtime_on_supported_types(Runtime::Bincode);
}

fn test_java_runtime_on_supported_types(runtime: Runtime) {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_encodings(vec![runtime.into()]);
    let generator = java::CodeGenerator::new(&config);
    generator
        .write_source_files(dir.path().to_path_buf(), &registry)
        .unwrap();

    let values = test_utils::get_sample_values();
    let encodings = values
        .iter()
        .map(|v| {
            let bytes = runtime.serialize(&v);
            format!(
                "\n{{{}}}",
                bytes
                    .iter()
                    .map(|x| format!("{}", *x as i8))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let mut source = File::create(&dir.path().join("Main.java")).unwrap();
    writeln!(
        source,
        r#"
import java.util.List;
import java.util.Arrays;
import com.facebook.serde.Unsigned;
import com.facebook.serde.Tuple2;
import testing.SerdeData;

public class Main {{
    public static void main(String[] args) throws java.lang.Exception {{
        byte[][] inputs = new byte[][] {{{0}}};

        for (int i = 0; i < inputs.length; i++) {{
            SerdeData test = SerdeData.{1}Deserialize(inputs[i]);

            byte[] output = test.{1}Serialize();

            assert java.util.Arrays.equals(inputs[i], output);
        }}
    }}
}}
"#,
        encodings,
        runtime.name(),
    )
    .unwrap();

    let paths = std::iter::empty()
        .chain(std::fs::read_dir("runtime/java/com/facebook/serde").unwrap())
        .chain(
            std::fs::read_dir("runtime/java/com/facebook/".to_string() + runtime.name()).unwrap(),
        )
        .chain(std::fs::read_dir(dir.path().join("testing")).unwrap())
        .map(|e| e.unwrap().path());
    let status = Command::new("javac")
        .arg("-Xlint")
        .arg("-d")
        .arg(dir.path())
        .args(paths)
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("javac")
        .arg("-Xlint")
        .arg("-cp")
        .arg(dir.path())
        .arg("-d")
        .arg(dir.path())
        .arg(dir.path().join("Main.java"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("java")
        .arg("-enableassertions")
        .arg("-cp")
        .arg(dir.path())
        .arg("Main")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_java_lcs_runtime_autotest() {
    let dir = tempdir().unwrap();
    let paths = std::iter::empty()
        .chain(std::fs::read_dir("runtime/java/com/facebook/serde").unwrap())
        .chain(std::fs::read_dir("runtime/java/com/facebook/lcs").unwrap())
        .map(|e| e.unwrap().path());
    let status = Command::new("javac")
        .arg("-Xlint")
        .arg("-d")
        .arg(dir.path())
        .args(paths)
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("java")
        .arg("-enableassertions")
        .arg("-cp")
        .arg(dir.path())
        .arg("com.facebook.lcs.LcsTest")
        .status()
        .unwrap();
    assert!(status.success());
}

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

fn test_golang_runtime_on_simple_data(runtime: Runtime) {
    let registry = get_local_registry().unwrap();
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
            "github.com/facebookincubator/serde-reflection/serde-generate/runtime/golang={}",
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
