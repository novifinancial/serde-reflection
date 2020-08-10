// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Serde code generator
//!
//! '''bash
//! cargo run --bin serdegen -- --help
//! '''

use serde_generate::{cpp, java, python3, rust, CodegenConfig, SourceInstaller};
use serde_reflection::Registry;
use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
#[derive(Debug, StructOpt)]
enum Language {
    Python3,
    Cpp,
    Rust,
    Java,
}
}

arg_enum! {
#[derive(Debug, StructOpt, PartialEq, Eq, PartialOrd, Ord)]
enum Runtime {
    Serde,
    Bincode,
    LCS,
}
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Serde code generator",
    about = "Generate code for Serde containers"
)]
struct Options {
    /// Path to the YAML-encoded Serde formats.
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,

    /// Language for code generation.
    #[structopt(long, possible_values = &Language::variants(), case_insensitive = true, default_value = "Python3")]
    language: Language,

    /// Directory where to write generated modules (otherwise print code on stdout).
    #[structopt(long)]
    target_source_dir: Option<PathBuf>,

    /// Optional runtimes to install in the `target_source_dir`.
    #[structopt(long, possible_values = &Runtime::variants(), case_insensitive = true)]
    with_runtimes: Vec<Runtime>,

    /// Module name for the Serde formats installed in the `target_source_dir`.
    /// Rust crates may contain a version number separated with a colon, e.g. "test:1.2.0".
    /// (By default, the installer will use version "0.1.0".)
    #[structopt(long)]
    module_name: Option<String>,

    /// Optional package name where to find the `serde_types` module (useful in Python).
    #[structopt(long)]
    serde_package_name: Option<String>,
}

fn main() {
    let options = Options::from_args();
    let serde_package_name_opt = options.serde_package_name.clone();
    let named_registry_opt = match &options.input {
        None => None,
        Some(input) => {
            let name = options.module_name.clone().unwrap_or_else(|| {
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
            if let Some((registry, name)) = named_registry_opt {
                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                match options.language {
                    Language::Python3 => python3::output(&mut out, &registry).unwrap(),
                    Language::Rust => {
                        rust::output(&mut out, /* with_derive_macros */ true, &registry).unwrap()
                    }
                    Language::Cpp => cpp::output(&mut out, &registry, &name).unwrap(),
                    Language::Java => panic!("Code generation in Java requires `--install-dir`"),
                }
            }
        }

        Some(install_dir) => {
            let installer: Box<dyn SourceInstaller<Error = Box<dyn std::error::Error>>> =
                match options.language {
                    Language::Python3 => {
                        Box::new(python3::Installer::new(install_dir, serde_package_name_opt))
                    }
                    Language::Rust => Box::new(rust::Installer::new(install_dir)),
                    Language::Cpp => Box::new(cpp::Installer::new(install_dir)),
                    Language::Java => Box::new(java::Installer::new(install_dir)),
                };

            if let Some((registry, name)) = named_registry_opt {
                let config = CodegenConfig::new(name);
                installer.install_module(&config, &registry).unwrap();
            }

            let runtimes: std::collections::BTreeSet<_> =
                options.with_runtimes.into_iter().collect();

            for runtime in runtimes {
                match runtime {
                    Runtime::Serde => installer.install_serde_runtime().unwrap(),
                    Runtime::Bincode => installer.install_bincode_runtime().unwrap(),
                    Runtime::LCS => installer.install_lcs_runtime().unwrap(),
                }
            }
        }
    }
}
