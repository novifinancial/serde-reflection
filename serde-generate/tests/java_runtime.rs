// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(feature = "runtime-testing")]

use serde_generate::{
    java, test_utils,
    test_utils::{Choice, Runtime, Test},
    CodeGeneratorConfig,
};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_java_lcs_runtime_on_simple_data() {
    test_java_runtime_on_simple_data(Runtime::Lcs);
}

#[test]
fn test_java_bincode_runtime_on_simple_data() {
    test_java_runtime_on_simple_data(Runtime::Bincode);
}

fn test_java_runtime_on_simple_data(runtime: Runtime) {
    let registry = test_utils::get_simple_registry().unwrap();
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
import com.novi.serde.DeserializationError;
import com.novi.serde.Unsigned;
import com.novi.serde.Tuple2;
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
        }} catch (DeserializationError e) {{
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
        .chain(std::fs::read_dir("runtime/java/com/novi/serde").unwrap())
        .chain(std::fs::read_dir("runtime/java/com/novi/".to_string() + runtime.name()).unwrap())
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

    let values = test_utils::get_sample_values(runtime.has_canonical_maps());
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
import com.novi.serde.DeserializationError;
import com.novi.serde.Unsigned;
import com.novi.serde.Tuple2;
import testing.SerdeData;

public class Main {{
    public static void main(String[] args) throws java.lang.Exception {{
        byte[][] inputs = new byte[][] {{{0}}};

        for (byte[] input : inputs) {{
            SerdeData test = SerdeData.{1}Deserialize(input);
            byte[] output = test.{1}Serialize();

            assert java.util.Arrays.equals(input, output);

            // Test simple mutations of the input.
            for (int i = 0; i < input.length; i++) {{
                byte[] input2 = input.clone();
                input2[i] ^= 0x80;
                try {{
                    SerdeData test2 = SerdeData.{1}Deserialize(input2);
                    assert test2 != test;
                }} catch (DeserializationError e) {{
                    // All good
                }}
            }}

        }}
    }}
}}
"#,
        encodings,
        runtime.name(),
    )
    .unwrap();

    let paths = std::iter::empty()
        .chain(std::fs::read_dir("runtime/java/com/novi/serde").unwrap())
        .chain(std::fs::read_dir("runtime/java/com/novi/".to_string() + runtime.name()).unwrap())
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
        .chain(std::fs::read_dir("runtime/java/com/novi/serde").unwrap())
        .chain(std::fs::read_dir("runtime/java/com/novi/lcs").unwrap())
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
        .arg("com.novi.lcs.LcsTest")
        .status()
        .unwrap();
    assert!(status.success());
}
