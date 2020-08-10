// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use heck::CamelCase;
use libra_canonical_serialization as lcs;
use serde::{Deserialize, Serialize};
use serde_generate::{cpp, java, python3, rust, test_utils, CodegenConfig};
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

impl Runtime {
    fn name(self) -> &'static str {
        match self {
            Self::Lcs => "lcs",
            Self::Bincode => "bincode",
        }
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
    python3::output(&mut source, &registry).unwrap();

    let reference = runtime.serialize(&Test {
        a: vec![4, 6],
        b: (3, 5),
        c: Choice::C { x: 7 },
    });
    writeln!(
        source,
        r#"
import {0}

value = Test([4, 6], (3, 5), Choice__C(7))

s = {0}.serialize(value, Test)
assert s == bytes.fromhex("{1}")

v, buffer = {0}.deserialize(s, Test)
assert len(buffer) == 0
assert v == value
assert v.c.x == 7

v.b = (3, 0)
t = {0}.serialize(v, Test)
assert len(t) == len(s)
assert t != s
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
    python3::output(&mut source, &registry).unwrap();

    let values = test_utils::get_sample_values();
    let hex_encodings: Vec<_> = values
        .iter()
        .map(|v| format!("'{}'", hex::encode(&runtime.serialize(&v))))
        .collect();

    writeln!(
        source,
        r#"
import {0}

encodings = [bytes.fromhex(s) for s in [{1}]]

for encoding in encodings:
    v, buffer = {0}.deserialize(encoding, SerdeData)
    assert len(buffer) == 0

    s = {0}.serialize(v, SerdeData)
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
    let source_path = dir.path().join("src/main.rs");
    let mut source = File::create(&source_path).unwrap();
    rust::output(&mut source, /* with_derive_macros */ true, &registry).unwrap();

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
    cpp::output(&mut header, &registry, "testing").unwrap();

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
#include "{1}.hpp"
#include "test.hpp"

using namespace serde;
using namespace testing;

int main() {{
    std::vector<uint8_t> input = {{{0}}};

    auto deserializer = {2}Deserializer(input);
    auto test = Deserializable<Test>::deserialize(deserializer);

    auto a = std::vector<uint32_t> {{4, 6}};
    auto b = std::tuple<int64_t, uint64_t> {{-3, 5}};
    auto c = Choice {{ Choice::C {{ 7 }} }};
    auto test2 = Test {{a, b, c}};

    assert(test == test2);

    auto serializer = {2}Serializer();
    Serializable<Test>::serialize(test2, serializer);
    auto output = std::move(serializer).bytes();

    assert(input == output);

    return 0;
}}
"#,
        reference
            .iter()
            .map(|x| format!("0x{:02x}", x))
            .collect::<Vec<_>>()
            .join(", "),
        runtime.name(),
        runtime.name().to_camel_case(),
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
    cpp::output(&mut header, &registry, "testing").unwrap();

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
#include "{1}.hpp"
#include "test.hpp"

using namespace serde;
using namespace testing;

int main() {{
    try {{
        std::vector<std::vector<uint8_t>> inputs = {{{0}}};

        for (auto input: inputs) {{
            auto deserializer = {2}Deserializer(input);
            auto test = Deserializable<SerdeData>::deserialize(deserializer);

            auto serializer = {2}Serializer();
            Serializable<SerdeData>::serialize(test, serializer);
            auto output = std::move(serializer).bytes();

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
        runtime.name().to_camel_case(),
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

    let inner = CodegenConfig::new("testing".to_string());
    let config = java::JavaCodegenConfig::new(&inner);
    config
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
import com.facebook.serde.Deserializer;
import com.facebook.serde.Serializer;
import com.facebook.serde.Unsigned;
import com.facebook.serde.Tuple2;
import com.facebook.{1}.{2}Deserializer;
import com.facebook.{1}.{2}Serializer;
import testing.Choice;
import testing.Test;

public class Main {{
    public static void main(String[] args) throws java.lang.Exception {{
        byte[] input = new byte[] {{{0}}};

        Deserializer deserializer = new {2}Deserializer(input);
        Test test = Test.deserialize(deserializer);

        List<@Unsigned Integer> a = Arrays.asList(4, 6);
        Tuple2<Long, @Unsigned Long> b = new Tuple2<>(Long.valueOf(-3), Long.valueOf(5));
        Choice c = new Choice.C(Byte.valueOf((byte) 7));
        Test test2 = new Test(a, b, c);

        assert test.equals(test2);

        Serializer serializer = new {2}Serializer();
        test2.serialize(serializer);
        byte[] output = serializer.get_bytes();

        assert java.util.Arrays.equals(input, output);
    }}
}}
"#,
        reference
            .iter()
            .map(|x| format!("{}", *x as i8))
            .collect::<Vec<_>>()
            .join(", "),
        runtime.name(),
        runtime.name().to_camel_case(),
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

    let inner = CodegenConfig::new("testing".to_string());
    let config = java::JavaCodegenConfig::new(&inner);
    config
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
import com.facebook.serde.Deserializer;
import com.facebook.serde.Serializer;
import com.facebook.serde.Unsigned;
import com.facebook.serde.Tuple2;
import com.facebook.{1}.{2}Deserializer;
import com.facebook.{1}.{2}Serializer;
import testing.SerdeData;

public class Main {{
    public static void main(String[] args) throws java.lang.Exception {{
        byte[][] inputs = new byte[][] {{{0}}};

        for (int i = 0; i < inputs.length; i++) {{
            Deserializer deserializer = new {2}Deserializer(inputs[i]);
            SerdeData test = SerdeData.deserialize(deserializer);

            Serializer serializer = new {2}Serializer();
            test.serialize(serializer);
            byte[] output = serializer.get_bytes();

            assert java.util.Arrays.equals(inputs[i], output);
        }}
    }}
}}
"#,
        encodings,
        runtime.name(),
        runtime.name().to_camel_case(),
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
