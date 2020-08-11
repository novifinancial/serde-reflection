# serde-generate

[![serde-generate on crates.io](https://img.shields.io/crates/v/serde-generate)](https://crates.io/crates/serde-generate)
[![Documentation (latest release)](https://docs.rs/serde-generate/badge.svg)](https://docs.rs/serde-generate/)
[![Documentation (master)](https://img.shields.io/badge/docs-master-brightgreen)](https://facebookincubator.github.io/serde-reflection/serde_generate/)
[![License](https://img.shields.io/badge/license-Apache-green.svg)](../LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../LICENSE-MIT)

This crate aims to compile the data formats extracted from Rust by [`serde_reflection`](https://crates.io/crates/serde_reflection)
into type definitions for other programming languages.

### Supported Languages

The following target languages are currently supported:

* C++ 17
* Java 8
* Python 3
* Rust 2018

### Supported Encodings

Type definitions in a target language are meant to be used together with a runtime library that
provides (de)serialization in a particular [Serde encoding format](https://serde.rs/#data-formats).

This crate provides easy-to-deploy runtime libraries for the following binary formats, in all supported languages:

* [Bincode](https://docs.rs/bincode/1.3.1/bincode/),
* [Libra Canonical Serialization](https://libra.github.io/libra/libra_canonical_serialization/index.html) ("LCS" for short).

### Quick Start with Python and Bincode

In the following example, we transfer a `Test` value from Rust to Python using [`bincode`](https://docs.rs/bincode/1.3.1/bincode/).
```rust
use serde::{Deserialize, Serialize};
use serde_reflection::{Registry, Samples, Tracer, TracerConfig};
use std::io::Write;

#[derive(Serialize, Deserialize)]
struct Test {
    a: Vec<u64>,
    b: (u32, u32),
}

// Obtain the Serde format of `Test`. (In practice, formats are more often read from a file.)
let mut tracer = Tracer::new(TracerConfig::default());
tracer.trace_type::<Test>(&Samples::new()).unwrap();
let registry = tracer.registry().unwrap();

// Create Python class definitions.
let mut source = Vec::new();
let inner = serde_generate::CodeGeneratorConfig::new("testing".to_string());
let config = serde_generate::python3::CodeGenerator::new(&inner);
config.output(&mut source, &registry)?;

assert_eq!(
    String::from_utf8_lossy(&source),
    r#"
from dataclasses import dataclass
import typing
import serde_types as st

@dataclass
class Test:
    a: typing.Sequence[st.uint64]
    b: typing.Tuple[st.uint32, st.uint32]

"#.to_string());

// Append some test code to demonstrate Bincode deserialization
// using the runtime in `serde_generate/runtime/python/bincode`.
writeln!(
    source,
    r#"
import bincode

value, _ = bincode.deserialize(bytes.fromhex("{}"), Test)
assert value == Test(a=[4, 6], b=(3, 5))
"#,
    hex::encode(&bincode::serialize(&Test { a: vec![4, 6], b: (3, 5) }).unwrap()),
)?;

// Execute the Python code.
let mut child = std::process::Command::new("python3")
    .arg("-")
    .env("PYTHONPATH", std::env::var("PYTHONPATH").unwrap_or_default() + ":runtime/python")
    .stdin(std::process::Stdio::piped())
    .spawn()?;
child.stdin.as_mut().unwrap().write_all(&source)?;
let output = child.wait_with_output()?;
assert!(output.status.success());
```

### Binary Tool

In addition to a Rust library, this crate provides a binary tool `serdegen` to process Serde formats
saved on disk.

Assuming that a `serde_reflection::Registry` object has been serialized in a YAML file `test.yaml`,
the following command will generate Python class definitions and write them into `test.py`:
```bash
cargo run -p serde-generate -- --language python3 test.yaml > test.py
```

To create a python module `test` and install the bincode runtime in a directory `$DEST`, you may run:
```bash
cargo run -p serde-generate -- --language python3 --with-runtimes serde bincode --module-name test --target-source-dir "$DEST" test.yaml
```

See the help message of the tool with `--help` for more options.

Note: Outside of this repository, you may install the tool with `cargo install serde-generate` then use `$HOME/.cargo/bin/serdegen`.

## Contributing

See the [CONTRIBUTING](../CONTRIBUTING.md) file for how to help out.

## License

This project is available under the terms of either the [Apache 2.0 license](../LICENSE-APACHE) or the [MIT
license](../LICENSE-MIT).

<!--
README.md is generated from README.tpl by cargo readme. To regenerate:

cargo install cargo-readme
cargo readme > README.md
-->
