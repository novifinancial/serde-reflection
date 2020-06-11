// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Serde code generator
//!
//! '''bash
//! cargo run -p serde-generate -- --help
//! '''

use serde_generate::{python3, rust, SourceInstaller};
use serde_reflection::Registry;
use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
#[derive(Debug, StructOpt)]
enum Language {
    Python3,
    Rust,
}
}

arg_enum! {
#[derive(Debug, StructOpt, PartialEq, Eq, PartialOrd, Ord)]
enum Runtime {
    Serde,
    Bincode,
}
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Serde code generator",
    about = "Generate code for Serde containers"
)]
struct Options {
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(long, possible_values = &Language::variants(), case_insensitive = true, default_value = "Python3")]
    language: Language,

    #[structopt(long)]
    with_runtimes: Vec<Runtime>,

    #[structopt(long)]
    name: Option<String>,

    #[structopt(long)]
    target_source_dir: Option<PathBuf>,
}

fn main() {
    let options = Options::from_args();
    let named_registry_opt = match &options.input {
        None => None,
        Some(input) => {
            let name = options.name.clone().unwrap_or_else(|| {
                input
                    .file_stem()
                    .expect("failed to deduce module name from input path")
                    .to_string_lossy()
                    .into_owned()
            });
            let content = std::fs::read_to_string(input).expect("input file must be readable");
            let registry = serde_yaml::from_str::<Registry>(content.as_str()).unwrap();
            Some((registry, name))
        }
    };

    match options.target_source_dir {
        None => {
            if let Some((registry, _)) = named_registry_opt {
                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                match options.language {
                    Language::Python3 => python3::output(&mut out, &registry).unwrap(),
                    Language::Rust => {
                        rust::output(&mut out, /* with_derive_macros */ true, &registry).unwrap()
                    }
                }
            }
        }

        Some(install_dir) => {
            let installer: Box<dyn SourceInstaller<Error = Box<dyn std::error::Error>>> =
                match options.language {
                    Language::Python3 => Box::new(python3::Installer::new(install_dir)),
                    Language::Rust => Box::new(rust::Installer::new(install_dir)),
                };

            if let Some((registry, name)) = named_registry_opt {
                installer.install_module(&name, &registry).unwrap();
            }

            let runtimes: std::collections::BTreeSet<_> =
                options.with_runtimes.into_iter().collect();

            for runtime in runtimes {
                match runtime {
                    Runtime::Serde => installer.install_serde_runtime().unwrap(),
                    Runtime::Bincode => installer.install_bincode_runtime().unwrap(),
                }
            }
        }
    }
}
