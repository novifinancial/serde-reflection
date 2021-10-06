// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{typescript, CodeGeneratorConfig};
use include_dir::include_dir as include_directory;
use serde_reflection::Registry;
use std::{
    io::{Result, Write},
    path::PathBuf,
};

/// Main configuration object for code-generation in Deno.
pub struct CodeGenerator<'a> {
    inner: typescript::CodeGenerator<'a>,
}

impl<'a> CodeGenerator<'a> {
    /// Create a Deno code generator for the given config.
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        let inner = typescript::CodeGenerator::new_with_runtime(
            config,
            typescript::TypeScriptRuntime::Deno,
        );
        Self { inner }
    }

    /// Output class definitions for `registry` in a single source file.
    pub fn output(&self, out: &mut dyn Write, registry: &Registry) -> Result<()> {
        self.inner.output(out, registry)
    }
}

/// Installer for generated source files in Deno.
pub struct Installer {
    install_dir: PathBuf,
}

impl Installer {
    pub fn new(install_dir: PathBuf) -> Self {
        Installer { install_dir }
    }

    fn install_runtime(
        &self,
        source_dir: include_dir::Dir,
        path: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir_path = self.install_dir.join(path);
        std::fs::create_dir_all(&dir_path)?;
        for entry in source_dir.files() {
            let mut file = std::fs::File::create(dir_path.join(entry.path()))?;
            file.write_all(entry.contents())?;
        }
        Ok(())
    }
}

impl crate::SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_module(
        &self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error> {
        let dir_path = self.install_dir.join(&config.module_name);
        std::fs::create_dir_all(&dir_path)?;
        let source_path = dir_path.join("index.ts");
        let mut file = std::fs::File::create(source_path)?;

        let generator = CodeGenerator::new(config);
        generator.output(&mut file, registry)?;
        Ok(())
    }

    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(include_directory!("runtime/deno/serde"), "serde")
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        unimplemented!("no deno support for bincode yet");
    }

    fn install_bcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(include_directory!("runtime/deno/bcs"), "bcs")
    }
}
