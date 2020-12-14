// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

//! This crate aims to compile the data formats extracted from Rust by [`serde_reflection`](https://crates.io/crates/serde_reflection)
//! into type definitions for other programming languages.
//!
//! ## Supported Languages
//!
//! The following target languages are currently supported:
//!
//! * C++ 17
//! * Java 8
//! * Python 3
//! * Rust 2018
//! * Go >= 1.13
//! * C# (NetCoreApp >= 2.1)
//!
//! ## Work in progress
//! * TypeScript > 3.2 (make sure to enable `esnext.BigInt` and `dom` at tsconfig.json -> lib)
//!
//! ## Supported Encodings
//!
//! Type definitions in a target language are meant to be used together with a runtime library that
//! provides (de)serialization in a particular [Serde encoding format](https://serde.rs/#data-formats).
//!
//! This crate provides easy-to-deploy runtime libraries for the following binary formats, in all supported languages:
//!
//! * [Bincode](https://docs.rs/bincode/1.3.1/bincode/),
//! * [Libra Canonical Serialization](https://libra.github.io/libra/binary_canonical_serialization/index.html) ("BCS" for short).
//!
//! ## Quick Start with Python and Bincode
//!
//! In the following example, we transfer a `Test` value from Rust to Python using [`bincode`](https://docs.rs/bincode/1.3.1/bincode/).
//! ```
//! use serde::{Deserialize, Serialize};
//! use serde_reflection::{Registry, Samples, Tracer, TracerConfig};
//! use std::io::Write;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Test {
//!     a: Vec<u64>,
//!     b: (u32, u32),
//! }
//!
//! # fn main() -> Result<(), std::io::Error> {
//! // Obtain the Serde format of `Test`. (In practice, formats are more often read from a file.)
//! let mut tracer = Tracer::new(TracerConfig::default());
//! tracer.trace_type::<Test>(&Samples::new()).unwrap();
//! let registry = tracer.registry().unwrap();
//!
//! // Create Python class definitions.
//! let mut source = Vec::new();
//! let config = serde_generate::CodeGeneratorConfig::new("testing".to_string())
//!     .with_encodings(vec![serde_generate::Encoding::Bincode]);
//! let generator = serde_generate::python3::CodeGenerator::new(&config);
//! generator.output(&mut source, &registry)?;
//!
//! assert!(
//!     String::from_utf8_lossy(&source).contains(
//!     r#"
//! @dataclass(frozen=True)
//! class Test:
//!     a: typing.Sequence[st.uint64]
//!     b: typing.Tuple[st.uint32, st.uint32]
//! "#));
//!
//! // Append some test code to demonstrate Bincode deserialization
//! // using the runtime in `serde_generate/runtime/python/bincode`.
//! writeln!(
//!     source,
//!     r#"
//! value = Test.bincode_deserialize(bytes({:?}))
//! assert value == Test(a=[4, 6], b=(3, 5))
//! "#,
//!     bincode::serialize(&Test { a: vec![4, 6], b: (3, 5) }).unwrap(),
//! )?;
//!
//! // Execute the Python code.
//! let mut child = std::process::Command::new("python3")
//!     .arg("-")
//!     .env("PYTHONPATH", std::env::var("PYTHONPATH").unwrap_or_default() + ":runtime/python")
//!     .stdin(std::process::Stdio::piped())
//!     .spawn()?;
//! child.stdin.as_mut().unwrap().write_all(&source)?;
//! let output = child.wait_with_output()?;
//! assert!(output.status.success());
//! # Ok(())
//! # }
//! ```
//!
//! ## Binary Tool
//!
//! In addition to a Rust library, this crate provides a binary tool `serdegen` to process Serde formats
//! saved on disk.
//!
//! Assuming that a `serde_reflection::Registry` object has been serialized in a YAML file `test.yaml`,
//! the following command will generate Python class definitions and write them into `test.py`:
//! ```bash
//! cargo run -p serde-generate -- --language python3 test.yaml > test.py
//! ```
//!
//! To create a python module `test` and install the bincode runtime in a directory `$DEST`, you may run:
//! ```bash
//! cargo run -p serde-generate -- --language python3 --with-runtimes serde bincode --module-name test --target-source-dir "$DEST" test.yaml
//! ```
//!
//! See the help message of the tool with `--help` for more options.
//!
//! Note: Outside of this repository, you may install the tool with `cargo install serde-generate` then use `$HOME/.cargo/bin/serdegen`.

/// Dependency analysis and topological sort for Serde formats.
pub mod analyzer;
/// Support for code-generation in C++
pub mod cpp;
/// Support for code-generation in C#
pub mod csharp;
/// Utility function to generate indented text
pub mod golang;
/// Support for code-generation in Go
pub mod indent;
/// Support for code-generation in Java
pub mod java;
/// Support for code-generation in Python 3
pub mod python3;
/// Support for code-generation in Rust
pub mod rust;
/// Support for code-generation in TypeScript/JavaScript
pub mod typescript;

#[doc(hidden)]
/// Utility functions to help testing code generators.
pub mod test_utils;

/// Common logic for codegen.
mod common;
/// Common configuration objects and traits used in public APIs.
mod config;

pub use config::*;
