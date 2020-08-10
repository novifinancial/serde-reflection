// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
    analyzer,
    indent::{IndentConfig, IndentedWriter},
    CodegenConfig,
};
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::io::{Result, Write};
use std::path::PathBuf;

/// Main configuration object for code-generation in Rust.
pub struct RustCodegenConfig<'a> {
    /// Language-independent configuration.
    inner: &'a CodegenConfig,
    /// Which derive macros should be added (independently from serialization).
    derive_macros: Vec<String>,
    /// Additional block of text added before each new container definition.
    custom_derive_block: Option<String>,
    /// Whether definitions and fields should be marked as `pub`.
    track_visibility: bool,
}

/// Shared state for the code generation of a Rust source file.
struct RustEmitter<'a, T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Configuration.
    config: &'a RustCodegenConfig<'a>,
    /// Track which definitions have a known size. (Used to add `Box` types.)
    known_sizes: Cow<'a, HashSet<&'a str>>,
}

impl<'a> RustCodegenConfig<'a> {
    /// Default config for Rust code generation.
    pub fn new(inner: &'a CodegenConfig) -> Self {
        Self {
            inner,
            derive_macros: vec!["Clone", "Debug", "PartialEq", "PartialOrd"]
                .into_iter()
                .map(String::from)
                .collect(),
            custom_derive_block: None,
            track_visibility: true,
        }
    }

    /// Which derive macros should be added (independently from serialization).
    pub fn with_derive_macros(mut self, derive_macros: Vec<String>) -> Self {
        self.derive_macros = derive_macros;
        self
    }

    /// Additional block of text added after `derive_macros` (if any), before each new
    /// container definition.
    pub fn with_custom_derive_block(mut self, custom_derive_block: Option<String>) -> Self {
        self.custom_derive_block = custom_derive_block;
        self
    }

    /// Whether definitions and fields should be marked as `pub`.
    pub fn with_track_visibility(mut self, track_visibility: bool) -> Self {
        self.track_visibility = track_visibility;
        self
    }

    /// Write container definitions in Rust.
    /// * All definitions are made `pub`.
    /// * If `with_serialization` is true, the crate `serde` and `serde_bytes` are assumed to be available.
    pub fn output(
        &self,
        out: &mut dyn Write,
        registry: &Registry,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let external_names = self
            .inner
            .external_definitions
            .values()
            .cloned()
            .flatten()
            .collect();
        let dependencies =
            analyzer::get_dependency_map_with_external_dependencies(registry, &external_names)?;
        let entries = analyzer::best_effort_topological_sort(&dependencies);

        let known_sizes = external_names
            .iter()
            .map(<String as std::ops::Deref>::deref)
            .collect::<HashSet<_>>();

        let mut emitter = RustEmitter {
            out: IndentedWriter::new(out, IndentConfig::Space(4)),
            config: self,
            known_sizes: Cow::Owned(known_sizes),
        };

        emitter.output_preamble()?;
        for name in entries {
            let format = &registry[name];
            emitter.output_container(name, format)?;
            emitter.known_sizes.to_mut().insert(name);
        }
        Ok(())
    }

    /// For each container, generate a Rust definition.
    pub fn quote_container_definitions(
        &self,
        registry: &Registry,
    ) -> std::result::Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
        let dependencies = analyzer::get_dependency_map(registry)?;
        let entries = analyzer::best_effort_topological_sort(&dependencies);

        let mut result = BTreeMap::new();
        let mut known_sizes = HashSet::new();

        for name in entries {
            let mut content = Vec::new();
            {
                let mut emitter = RustEmitter {
                    out: IndentedWriter::new(&mut content, IndentConfig::Space(4)),
                    config: self,
                    known_sizes: Cow::Borrowed(&known_sizes),
                };
                let format = &registry[name];
                emitter.output_container(name, format)?;
            }
            known_sizes.insert(name);
            result.insert(
                name.to_string(),
                String::from_utf8_lossy(&content).trim().to_string() + "\n",
            );
        }
        Ok(result)
    }
}

impl<'a, T> RustEmitter<'a, T>
where
    T: std::io::Write,
{
    fn output_comment(&mut self, qualified_name: &[&str]) -> std::io::Result<()> {
        if let Some(doc) = self.config.inner.comments.get(
            &qualified_name
                .to_vec()
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>(),
        ) {
            let text = textwrap::indent(doc, "/// ").replace("\n\n", "\n///\n");
            write!(self.out, "\n{}", text)?;
        }
        Ok(())
    }

    fn output_preamble(&mut self) -> Result<()> {
        let external_names = self
            .config
            .inner
            .external_definitions
            .values()
            .cloned()
            .flatten()
            .collect::<HashSet<_>>();
        writeln!(self.out, "#![allow(unused_imports)]")?;
        if !external_names.contains("Map") {
            writeln!(self.out, "use std::collections::BTreeMap as Map;")?;
        }
        if self.config.inner.serialization {
            writeln!(self.out, "use serde::{{Serialize, Deserialize}};")?;
        }
        if self.config.inner.serialization && !external_names.contains("Bytes") {
            writeln!(self.out, "use serde_bytes::ByteBuf as Bytes;")?;
        }
        for (module, definitions) in &self.config.inner.external_definitions {
            // Skip the empty module name.
            if !module.is_empty() {
                writeln!(
                    self.out,
                    "use {}::{{{}}};",
                    module,
                    definitions.to_vec().join(", "),
                )?;
            }
        }
        writeln!(self.out)?;
        if !self.config.inner.serialization && !external_names.contains("Bytes") {
            // If we are not going to use Serde derive macros, use plain vectors.
            writeln!(self.out, "type Bytes = Vec<u8>;\n")?;
        }
        Ok(())
    }

    fn quote_type(format: &Format, known_sizes: Option<&HashSet<&str>>) -> String {
        use Format::*;
        match format {
            TypeName(x) => {
                if let Some(set) = known_sizes {
                    if !set.contains(x.as_str()) {
                        return format!("Box<{}>", x);
                    }
                }
                x.to_string()
            }
            Unit => "()".into(),
            Bool => "bool".into(),
            I8 => "i8".into(),
            I16 => "i16".into(),
            I32 => "i32".into(),
            I64 => "i64".into(),
            I128 => "i128".into(),
            U8 => "u8".into(),
            U16 => "u16".into(),
            U32 => "u32".into(),
            U64 => "u64".into(),
            U128 => "u128".into(),
            F32 => "f32".into(),
            F64 => "f64".into(),
            Char => "char".into(),
            Str => "String".into(),
            Bytes => "Bytes".into(),

            Option(format) => format!("Option<{}>", Self::quote_type(format, known_sizes)),
            Seq(format) => format!("Vec<{}>", Self::quote_type(format, None)),
            Map { key, value } => format!(
                "Map<{}, {}>",
                Self::quote_type(key, None),
                Self::quote_type(value, None)
            ),
            Tuple(formats) => format!("({})", Self::quote_types(formats, known_sizes)),
            TupleArray { content, size } => {
                format!("[{}; {}]", Self::quote_type(content, known_sizes), *size)
            }

            Variable(_) => panic!("unexpected value"),
        }
    }

    fn quote_types(formats: &[Format], known_sizes: Option<&HashSet<&str>>) -> String {
        formats
            .iter()
            .map(|x| Self::quote_type(x, known_sizes))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn output_fields(&mut self, base: &[&str], fields: &[Named<Format>]) -> Result<()> {
        // Do not add 'pub' within variants.
        let prefix = if base.len() <= 1 && self.config.track_visibility {
            "pub "
        } else {
            ""
        };
        for field in fields {
            let qualified_name = {
                let mut name = base.to_vec();
                name.push(&field.name);
                name
            };
            self.output_comment(&qualified_name)?;
            writeln!(
                self.out,
                "{}{}: {},",
                prefix,
                field.name,
                Self::quote_type(&field.value, Some(&self.known_sizes)),
            )?;
        }
        Ok(())
    }

    fn output_variant(&mut self, base: &str, name: &str, variant: &VariantFormat) -> Result<()> {
        use VariantFormat::*;
        match variant {
            Unit => writeln!(self.out, "{},", name),
            NewType(format) => writeln!(
                self.out,
                "{}({}),",
                name,
                Self::quote_type(format, Some(&self.known_sizes))
            ),
            Tuple(formats) => writeln!(
                self.out,
                "{}({}),",
                name,
                Self::quote_types(formats, Some(&self.known_sizes))
            ),
            Struct(fields) => {
                writeln!(self.out, "{} {{", name)?;
                self.out.indent();
                self.output_fields(&[base, name], fields)?;
                self.out.unindent();
                writeln!(self.out, "}},")
            }
            Variable(_) => panic!("incorrect value"),
        }
    }

    fn output_variants(
        &mut self,
        base: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> Result<()> {
        for (expected_index, (index, variant)) in variants.iter().enumerate() {
            assert_eq!(*index, expected_index as u32);
            self.output_comment(&[base, &variant.name])?;
            self.output_variant(base, &variant.name, &variant.value)?;
        }
        Ok(())
    }

    fn output_container(&mut self, name: &str, format: &ContainerFormat) -> Result<()> {
        self.output_comment(&[name])?;
        let mut derive_macros = self.config.derive_macros.clone();
        if self.config.inner.serialization {
            derive_macros.push("Serialize".to_string());
            derive_macros.push("Deserialize".to_string());
        }
        let mut prefix = String::new();
        if !derive_macros.is_empty() {
            prefix.push_str(&format!("#[derive({})]\n", derive_macros.join(", ")));
        }
        if let Some(text) = &self.config.custom_derive_block {
            prefix.push_str(text);
            prefix.push_str("\n");
        }
        if self.config.track_visibility {
            prefix.push_str("pub ");
        }

        use ContainerFormat::*;
        match format {
            UnitStruct => writeln!(self.out, "{}struct {};\n", prefix, name),
            NewTypeStruct(format) => writeln!(
                self.out,
                "{}struct {}({}{});\n",
                prefix,
                name,
                if self.config.track_visibility {
                    "pub "
                } else {
                    ""
                },
                Self::quote_type(format, Some(&self.known_sizes))
            ),
            TupleStruct(formats) => writeln!(
                self.out,
                "{}struct {}({});\n",
                prefix,
                name,
                Self::quote_types(formats, Some(&self.known_sizes))
            ),
            Struct(fields) => {
                writeln!(self.out, "{}struct {} {{", prefix, name)?;
                self.out.indent();
                self.output_fields(&[name], fields)?;
                self.out.unindent();
                writeln!(self.out, "}}\n")
            }
            Enum(variants) => {
                writeln!(self.out, "{}enum {} {{", prefix, name)?;
                self.out.indent();
                self.output_variants(name, variants)?;
                self.out.unindent();
                writeln!(self.out, "}}\n")
            }
        }
    }
}

pub struct Installer {
    install_dir: PathBuf,
}

impl Installer {
    pub fn new(install_dir: PathBuf) -> Self {
        Installer { install_dir }
    }

    fn runtimes_not_implemented() -> std::result::Result<(), Box<dyn std::error::Error>> {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Installing runtimes is not implemented: use cargo instead",
        )))
    }
}

impl crate::SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_module(
        &self,
        inner: &CodegenConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error> {
        let config = RustCodegenConfig::new(inner);
        let (name, version) = {
            let parts = inner.module_name.splitn(2, ':').collect::<Vec<_>>();
            if parts.len() >= 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                (parts[0].to_string(), "0.1.0".to_string())
            }
        };
        let dir_path = self.install_dir.join(&name);
        std::fs::create_dir_all(&dir_path)?;
        let mut cargo = std::fs::File::create(&dir_path.join("Cargo.toml"))?;
        write!(
            cargo,
            r#"[package]
name = "{}"
version = "{}"
edition = "2018"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_bytes = "0.11"
"#,
            name, version,
        )?;
        std::fs::create_dir(dir_path.join("src"))?;
        let source_path = dir_path.join("src/lib.rs");
        let mut source = std::fs::File::create(&source_path)?;
        config.output(&mut source, &registry)
    }

    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error> {
        Self::runtimes_not_implemented()
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        Self::runtimes_not_implemented()
    }

    fn install_lcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        Self::runtimes_not_implemented()
    }
}
