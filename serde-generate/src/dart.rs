use crate::{
    CodeGeneratorConfig,
};
use serde_reflection::{ Registry};
use std::{
    collections::{ HashMap},
    io::{Result, Write},
    path::PathBuf,
};

/// Main configuration object for code-generation in Java.
pub struct CodeGenerator<'a> {
    /// Language-independent configuration.
    _config: &'a CodeGeneratorConfig,
    /// Mapping from external type names to fully-qualified class names (e.g. "MyClass" -> "com.facebook.my_package.MyClass").
    /// Derived from `config.external_definitions`.
    _external_qualified_names: HashMap<String, String>,
}

impl<'a> CodeGenerator<'a> {
    /// Create a Java code generator for the given config.
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        let mut external_qualified_names = HashMap::new();
        for (namespace, names) in &config.external_definitions {
            for name in names {
                external_qualified_names
                    .insert(name.to_string(), format!("{}.{}", namespace, name));
            }
        }
        Self {
            _config:config,
            _external_qualified_names:external_qualified_names,
        }
    }

    /// Output class definitions for `registry`.
    pub fn output(&self, _out: &mut dyn Write, _registry: &Registry) -> Result<()> {
        Ok(())
    }
}

/// Installer for generated source files in Go.
pub struct Installer {
    install_dir: PathBuf,
}

impl Installer {
    pub fn new(install_dir: PathBuf) -> Self {
        Installer {
            install_dir,
        }
    }

    fn runtimes_installation_not_required() -> std::result::Result<(), Box<dyn std::error::Error>> {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Runtime is installed by `go get`, no source code installation required",
        )))
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
        let source_path = dir_path.join("lib.go");
        let mut file = std::fs::File::create(source_path)?;

        let generator = CodeGenerator::new(config);
        // if let Some(path) = &self.serde_module_path {
        //     generator = generator.with_serde_module_path(path.clone());
        // }
        generator.output(&mut file, registry)?;
        Ok(())
    }

    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error> {
        Self::runtimes_installation_not_required()
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        Self::runtimes_installation_not_required()
    }

    fn install_lcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        Self::runtimes_installation_not_required()
    }
}
