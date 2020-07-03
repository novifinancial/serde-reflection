// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::analyzer;
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};
use std::collections::{BTreeMap, HashSet};
use std::io::{Result, Write};
use std::path::PathBuf;

/// Write container definitions in Rust.
/// * All definitions will be made public.
/// * The crate `serde_bytes` may be required.
/// * If `with_derive_macros` is true, the crate `serde` is required.
pub fn output(
    out: &mut dyn Write,
    with_derive_macros: bool,
    registry: &Registry,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let dependencies = analyzer::get_dependency_map(registry)?;
    let entries = analyzer::best_effort_topological_sort(&dependencies);

    output_preamble(out, with_derive_macros)?;
    let mut known_sizes = HashSet::new();
    for name in entries {
        let format = &registry[name];
        output_container(
            out,
            with_derive_macros,
            /* track visibility */ true,
            name,
            format,
            &known_sizes,
        )?;
        known_sizes.insert(name);
    }
    Ok(())
}

/// For each container, generate a Rust definition suitable for documentation purposes.
pub fn quote_container_definitions(
    registry: &Registry,
) -> std::result::Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let dependencies = analyzer::get_dependency_map(registry)?;
    let entries = analyzer::best_effort_topological_sort(&dependencies);

    let mut result = BTreeMap::new();
    let mut known_sizes = HashSet::new();
    for name in entries {
        let format = &registry[name];
        let mut content = Vec::new();
        output_container(
            &mut content,
            /* with derive macros */ false,
            /* track visibility */ false,
            name,
            format,
            &known_sizes,
        )?;
        known_sizes.insert(name);
        result.insert(
            name.to_string(),
            String::from_utf8_lossy(&content).trim().to_string() + "\n",
        );
    }
    Ok(result)
}

fn output_preamble(out: &mut dyn Write, with_derive_macros: bool) -> Result<()> {
    writeln!(
        out,
        r#"#![allow(unused_imports)]
use serde_bytes::ByteBuf as Bytes;
use std::collections::BTreeMap as Map;
"#
    )?;
    if with_derive_macros {
        writeln!(out, "use serde::{{Serialize, Deserialize}};\n")?;
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

        Option(format) => format!("Option<{}>", quote_type(format, known_sizes)),
        Seq(format) => format!("Vec<{}>", quote_type(format, None)),
        Map { key, value } => format!(
            "Map<{}, {}>",
            quote_type(key, None),
            quote_type(value, None)
        ),
        Tuple(formats) => format!("({})", quote_types(formats, known_sizes)),
        TupleArray { content, size } => {
            format!("[{}; {}]", quote_type(content, known_sizes), *size)
        }

        Variable(_) => panic!("unexpected value"),
    }
}

fn quote_types(formats: &[Format], known_sizes: Option<&HashSet<&str>>) -> String {
    formats
        .iter()
        .map(|x| quote_type(x, known_sizes))
        .collect::<Vec<_>>()
        .join(", ")
}

fn output_fields(
    out: &mut dyn Write,
    indentation: usize,
    fields: &[Named<Format>],
    is_pub: bool,
    known_sizes: &HashSet<&str>,
) -> Result<()> {
    let mut tab = " ".repeat(indentation);
    if is_pub {
        tab += " pub ";
    }
    for field in fields {
        writeln!(
            out,
            "{}{}: {},",
            tab,
            field.name,
            quote_type(&field.value, Some(known_sizes)),
        )?;
    }
    Ok(())
}

fn output_variant(
    out: &mut dyn Write,
    name: &str,
    variant: &VariantFormat,
    known_sizes: &HashSet<&str>,
) -> Result<()> {
    use VariantFormat::*;
    match variant {
        Unit => writeln!(out, "    {},", name),
        NewType(format) => writeln!(
            out,
            "    {}({}),",
            name,
            quote_type(format, Some(known_sizes))
        ),
        Tuple(formats) => writeln!(
            out,
            "    {}({}),",
            name,
            quote_types(formats, Some(known_sizes))
        ),
        Struct(fields) => {
            writeln!(out, "    {} {{", name)?;
            output_fields(out, 8, fields, false, known_sizes)?;
            writeln!(out, "    }},")
        }
        Variable(_) => panic!("incorrect value"),
    }
}

fn output_variants(
    out: &mut dyn Write,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    known_sizes: &HashSet<&str>,
) -> Result<()> {
    for (expected_index, (index, variant)) in variants.iter().enumerate() {
        assert_eq!(*index, expected_index as u32);
        output_variant(out, &variant.name, &variant.value, known_sizes)?;
    }
    Ok(())
}

fn output_container(
    out: &mut dyn Write,
    with_derive_macros: bool,
    track_visibility: bool,
    name: &str,
    format: &ContainerFormat,
    known_sizes: &HashSet<&str>,
) -> Result<()> {
    use ContainerFormat::*;
    let mut prefix = if with_derive_macros {
        "#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd)]\n".to_string()
    } else {
        String::new()
    };
    if track_visibility {
        prefix.push_str("pub ");
    }
    match format {
        UnitStruct => writeln!(out, "{}struct {};\n", prefix, name),
        NewTypeStruct(format) => writeln!(
            out,
            "{}struct {}({});\n",
            prefix,
            name,
            quote_type(format, Some(known_sizes))
        ),
        TupleStruct(formats) => writeln!(
            out,
            "{}struct {}({});\n",
            prefix,
            name,
            quote_types(formats, Some(known_sizes))
        ),
        Struct(fields) => {
            writeln!(out, "{}struct {} {{", prefix, name)?;
            output_fields(out, 4, fields, track_visibility, known_sizes)?;
            writeln!(out, "}}\n")
        }
        Enum(variants) => {
            writeln!(out, "{}enum {} {{", prefix, name)?;
            output_variants(out, variants, known_sizes)?;
            writeln!(out, "}}\n")
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
