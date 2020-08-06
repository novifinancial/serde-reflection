// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
    analyzer,
    indent::{IndentConfig, IndentedWriter},
    DocComments, ExternalDefinitions,
};
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::io::{Result, Write};
use std::path::PathBuf;

/// Write container definitions in Rust.
/// * All definitions are made `pub`.
/// * If `with_derive_macros` is true, the crate `serde` and `serde_bytes` are assumed to be available.
pub fn output(
    out: &mut dyn Write,
    with_derive_macros: bool,
    registry: &Registry,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    output_with_external_dependencies_and_comments(
        out,
        with_derive_macros,
        registry,
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
}

/// Same as `output` but allow some type definitions to be provided by external modules, and
/// doc comments to be attached to named components.
/// * A `use` statement will be generated for every external definition provided by a non-empty module name.
/// * The empty module name is allowed and can be used to signal that custom definitions
/// (including for Map and Bytes) will be added manually at the end of the generated file.
pub fn output_with_external_dependencies_and_comments(
    out: &mut dyn Write,
    with_derive_macros: bool,
    registry: &Registry,
    external_definitions: &ExternalDefinitions,
    comments: &DocComments,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let external_names = external_definitions.values().cloned().flatten().collect();
    let dependencies =
        analyzer::get_dependency_map_with_external_dependencies(registry, &external_names)?;
    let entries = analyzer::best_effort_topological_sort(&dependencies);

    let known_sizes = external_names
        .iter()
        .map(<String as std::ops::Deref>::deref)
        .collect::<HashSet<_>>();

    let mut emitter = RustEmitter {
        out: IndentedWriter::new(out, IndentConfig::Space(4)),
        comments,
        track_visibility: true,
        with_derive_macros,
        known_sizes: Cow::Owned(known_sizes),
    };

    emitter.output_preamble(external_definitions)?;
    for name in entries {
        let format = &registry[name];
        emitter.output_container(name, format)?;
        emitter.known_sizes.to_mut().insert(name);
    }
    Ok(())
}

/// For each container, generate a Rust definition suitable for documentation purposes.
pub fn quote_container_definitions(
    registry: &Registry,
) -> std::result::Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    quote_container_definitions_with_comments(registry, &BTreeMap::new())
}

/// Same as quote_container_definitions but including doc comments.
pub fn quote_container_definitions_with_comments(
    registry: &Registry,
    comments: &DocComments,
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
                comments,
                track_visibility: false,
                with_derive_macros: false,
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

struct RustEmitter<'a, T> {
    out: IndentedWriter<T>,
    comments: &'a DocComments,
    track_visibility: bool,
    with_derive_macros: bool,
    known_sizes: Cow<'a, HashSet<&'a str>>,
}

impl<'a, T> RustEmitter<'a, T>
where
    T: std::io::Write,
{
    fn output_comment(&mut self, qualified_name: &[&str]) -> std::io::Result<()> {
        if let Some(doc) = self.comments.get(
            &qualified_name
                .to_vec()
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>(),
        ) {
            let prefix = "/// ";
            let empty_line = "\n".to_string() + "///\n";
            let text = textwrap::indent(doc, &prefix).replace("\n\n", &empty_line);
            write!(self.out, "\n{}", text)?;
        }
        Ok(())
    }

    fn output_preamble(&mut self, external_definitions: &ExternalDefinitions) -> Result<()> {
        let external_names = external_definitions
            .values()
            .cloned()
            .flatten()
            .collect::<HashSet<_>>();
        writeln!(self.out, "#![allow(unused_imports)]")?;
        if !external_names.contains("Map") {
            writeln!(self.out, "use std::collections::BTreeMap as Map;")?;
        }
        if self.with_derive_macros {
            writeln!(self.out, "use serde::{{Serialize, Deserialize}};")?;
        }
        if self.with_derive_macros && !external_names.contains("Bytes") {
            writeln!(self.out, "use serde_bytes::ByteBuf as Bytes;")?;
        }
        for (module, definitions) in external_definitions {
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
        if !self.with_derive_macros && !external_names.contains("Bytes") {
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
        let prefix = if self.track_visibility { "pub " } else { "" };
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
                let tracking = std::mem::replace(&mut self.track_visibility, false);
                self.output_fields(&[base, name], fields)?;
                self.track_visibility = tracking;
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
        let mut prefix = if self.with_derive_macros {
            "#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]\n".to_string()
        } else {
            String::new()
        };
        if self.track_visibility {
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
                if self.track_visibility { "pub " } else { "" },
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
        public_name: &str,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error> {
        let (name, version) = {
            let parts = public_name.splitn(2, ':').collect::<Vec<_>>();
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
        output(&mut source, /* with_derive_macros */ true, &registry)
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
