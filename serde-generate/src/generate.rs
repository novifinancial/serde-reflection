// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Serde code generator
//!
//! '''bash
//! cargo run -p serde-generate -- --help
//! '''

use serde_generate::{cpp, python3, rust};
use serde_reflection::Registry;
use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
#[derive(Debug, StructOpt)]
enum Language {
    Python3,
    Cpp,
    Rust,
}
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Serde code generator",
    about = "Generate code for Serde containers"
)]
struct Options {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(long, possible_values = &Language::variants(), case_insensitive = true, default_value = "Python3")]
    language: Language,
}

fn main() {
    let options = Options::from_args();
    let content =
        std::fs::read_to_string(options.input.as_os_str()).expect("input file must be readable");
    let registry = serde_yaml::from_str::<Registry>(content.as_str()).unwrap();

    let mut out = std::io::stdout();

    match options.language {
        Language::Python3 => python3::output(&mut out, &registry).unwrap(),
        Language::Cpp => cpp::output(&mut out, &registry).unwrap(),
        Language::Rust => rust::output(&mut out, /* with_derive_macros */ true, &registry).unwrap(),
    }
}
