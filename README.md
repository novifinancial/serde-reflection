# serde-reflection: Format Description and Code Generation for Serde

[![Build Status](https://circleci.com/gh/facebookincubator/serde-reflection/tree/master.svg?style=shield&circle-token=4380502426d703f8f000b5467195728e5e8e4ff5)](https://circleci.com/gh/facebookincubator/serde-reflection/tree/master)
[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)

This repository contains the source code for:

* [`serde-reflection`](serde-reflection): a library to extract and represent Serde data formats [![serde-reflection on crates.io](https://img.shields.io/crates/v/serde-reflection)](https://crates.io/crates/serde-reflection) [![Documentation (latest release)](https://docs.rs/serde-reflection/badge.svg)](https://docs.rs/serde-reflection/) [![Documentation (master)](https://img.shields.io/badge/docs-master-59f)](https://facebookincubator.github.io/serde-reflection/serde_reflection/)

* [`serde-generate`](serde-generate): a tool to generate type definitions and provide (de)serialization in other programming languages [![serde-generate on crates.io](https://img.shields.io/crates/v/serde-generate)](https://crates.io/crates/serde-generate) [![Documentation (latest release)](https://docs.rs/serde-generate/badge.svg)](https://docs.rs/serde-generate/) [![Documentation (master)](https://img.shields.io/badge/docs-master-59f)](https://facebookincubator.github.io/serde-reflection/serde_generate/)

* [`serde-name`](serde-name): a minimal library to compute the Serde name of Rust containers [![serde-name on crates.io](https://img.shields.io/crates/v/serde-name)](https://crates.io/crates/serde-name) [![Documentation (latest release)](https://docs.rs/serde-name/badge.svg)](https://docs.rs/serde-name/) [![Documentation (master)](https://img.shields.io/badge/docs-master-59f)](https://facebookincubator.github.io/serde-reflection/serde_name/)


The code in this repository is under active development.

## Use cases

This project aims to facilitate the implementation of distributed protocols and storage protocols using Serde. [Serde](https://serde.rs/) is an essential component of the Rust ecosystem that provides (de)serialization of Rust data structures in [many encoding formats](https://serde.rs/#data-formats).

`serde-reflection` makes it easy to to extract a concise representation of the Serde data formats used in a Rust project. This is useful:

* to detect accidental changes to the data formats (e.g. using version control),

* to generate code using our tool `serde-generate` and interoperate with other languages.

In addition to ensuring an optimal developer experience in Rust, the approach based on Serde and `serde-reflection` empowers protocol designers to experiment and choose the best encoding format for their data: either [one of the encoding formats](https://serde.rs/#data-formats) officially supported by Serde, or [a new encoding format](https://serde.rs/data-format.html) developed in the Serde framework.

This project was initially motivated by the need for canonical serialization and cryptographic hashing in the [Libra](https://github.com/libra/libra) project.

In this context, `serde-name` has been used to provide predictable cryptographic seeds for Rust containers.

## Contributing

See the [CONTRIBUTING](CONTRIBUTING.md) file for how to help out.

## License

This project is available under the terms of either the [Apache 2.0 license](LICENSE-APACHE) or the [MIT license](LICENSE-MIT).
