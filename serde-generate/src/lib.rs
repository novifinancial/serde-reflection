// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

//! This crate aims to compile the data formats extracted from Rust by [`serde_reflection`](https://crates.io/crates/serde_reflection)
//! into type definitions for other programming languages.
//!
//! ## Supported Languages
//!
//! The following target languages are currently supported:
//!
//! * Python 3
//! * C++ 17
//! * Rust 2018
//!
//! ## Supported Encodings
//!
//! Type definitions in a target language are meant to be used together with a runtime library that
//! provides (de)serialization in a particular [Serde encoding format](https://serde.rs/#data-formats).
//!
//! This crate provides easy-to-deploy runtime libraries for the following binary formats, in all supported languages:
//!
//! * [Bincode](https://docs.rs/bincode/1.3.1/bincode/),
//! * [Libra Canonical Serialization](https://libra.github.io/libra/libra_canonical_serialization/index.html) ("LCS" for short).
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
//! serde_generate::python3::output(&mut source, &registry)?;
//!
//! assert_eq!(
//! #  "\n".to_string() + &
//!     String::from_utf8_lossy(&source),
//!     r#"
//! from dataclasses import dataclass
//! import typing
//! import serde_types as st
//!
//! @dataclass
//! class Test:
//!     a: typing.Sequence[st.uint64]
//!     b: typing.Tuple[st.uint32, st.uint32]
//!
//! "#.to_string());
//!
//! // Append some test code to demonstrate Bincode deserialization
//! // using the runtime in `serde_generate/runtime/python/bincode`.
//! writeln!(
//!     source,
//!     r#"
//! import bincode
//!
//! value, _ = bincode.deserialize(bytes.fromhex("{}"), Test)
//! assert value == Test(a=[4, 6], b=(3, 5))
//! "#,
//!     hex::encode(&bincode::serialize(&Test { a: vec![4, 6], b: (3, 5) }).unwrap()),
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
/// Support for code-generation in Python 3
pub mod python3;
/// Support for code-generation in Rust
pub mod rust;

#[doc(hidden)]
/// Utility functions to help testing code generators.
pub mod test_utils;

/// How to copy generated source code and available runtimes for a given language.
pub trait SourceInstaller {
    type Error;

    /// Create a module exposing the container types contained in the registry.
    fn install_module(
        &self,
        name: &str,
        registry: &serde_reflection::Registry,
    ) -> std::result::Result<(), Self::Error>;

    /// Install the serde runtime.
    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error>;

    /// Install the bincode runtime.
    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error>;

    /// Install the Libra Canonical Serialization (LCS) runtime.
    fn install_lcs_runtime(&self) -> std::result::Result<(), Self::Error>;
}
