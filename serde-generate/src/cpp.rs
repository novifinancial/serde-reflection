// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::analyzer;
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};
use std::collections::{BTreeMap, HashSet};
use std::io::{Result, Write};
use std::path::PathBuf;

pub fn output(
    out: &mut dyn Write,
    registry: &Registry,
    namespace: Option<&str>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    output_preamble(out)?;
    output_open_namespace(out, namespace)?;

    let dependencies = analyzer::get_dependency_map(registry)?;
    let entries = analyzer::best_effort_topological_sort(&dependencies);
    let mut known_names = HashSet::new();
    let mut known_sizes = HashSet::new();
    for name in entries {
        for dependency in &dependencies[name] {
            if !known_names.contains(dependency) {
                output_container_forward_definition(out, *dependency)?;
                known_names.insert(*dependency);
            }
        }
        let format = &registry[name];
        output_container(out, name, format, &known_sizes, namespace)?;
        known_sizes.insert(name);
        known_names.insert(name);
    }

    output_close_namespace(out, namespace)?;
    writeln!(out)?;
    for (name, format) in registry {
        output_container_traits(out, &name, format, namespace)?;
    }
    Ok(())
}

fn output_preamble(out: &mut dyn std::io::Write) -> Result<()> {
    writeln!(
        out,
        r#"
#pragma once

#include "serde.hpp"
"#
    )
}

fn output_open_namespace(out: &mut dyn std::io::Write, namespace: Option<&str>) -> Result<()> {
    if let Some(name) = namespace {
        writeln!(out, "\nnamespace {} {{", name,)?;
    }
    Ok(())
}

fn output_close_namespace(out: &mut dyn std::io::Write, namespace: Option<&str>) -> Result<()> {
    if let Some(name) = namespace {
        writeln!(out, "}} // end of namespace {}", name,)?;
    }
    Ok(())
}

/// If known_sizes is present, we must try to return a type with a known size as well.
/// A non-empty `namespace_prefix` is required when the type is quoted from a nested struct.
fn quote_type(
    format: &Format,
    known_sizes: Option<&HashSet<&str>>,
    namespace_prefix: &str,
) -> String {
    use Format::*;
    match format {
        TypeName(x) => {
            if let Some(set) = known_sizes {
                if !set.contains(x.as_str()) {
                    // Cannot use unique_ptr because we need a copy constructor (e.g. for vectors).
                    return format!("std::shared_ptr<{}{}>", namespace_prefix, x);
                }
            }
            format!("{}{}", namespace_prefix, x)
        }
        Unit => "std::monostate".into(),
        Bool => "bool".into(),
        I8 => "int8_t".into(),
        I16 => "int16_t".into(),
        I32 => "int32_t".into(),
        I64 => "int64_t".into(),
        I128 => "serde::int128_t".into(),
        U8 => "uint8_t".into(),
        U16 => "uint16_t".into(),
        U32 => "uint32_t".into(),
        U64 => "uint64_t".into(),
        U128 => "serde::uint128_t".into(),
        F32 => "float".into(),
        F64 => "double".into(),
        Char => "char32_t".into(),
        Str => "std::string".into(),
        Bytes => "std::vector<uint8_t>".into(),

        Option(format) => format!(
            "std::optional<{}>",
            quote_type(format, known_sizes, namespace_prefix)
        ),
        Seq(format) => format!(
            "std::vector<{}>",
            quote_type(format, None, namespace_prefix)
        ),
        Map { key, value } => format!(
            "std::map<{}, {}>",
            quote_type(key, None, namespace_prefix),
            quote_type(value, None, namespace_prefix)
        ),
        Tuple(formats) => format!(
            "std::tuple<{}>",
            quote_types(formats, known_sizes, namespace_prefix)
        ),
        TupleArray { content, size } => format!(
            "std::array<{}, {}>",
            quote_type(content, known_sizes, namespace_prefix),
            *size
        ),

        Variable(_) => panic!("unexpected value"),
    }
}

fn quote_types(
    formats: &[Format],
    known_sizes: Option<&HashSet<&str>>,
    namespace_prefix: &str,
) -> String {
    formats
        .iter()
        .map(|x| quote_type(x, known_sizes, namespace_prefix))
        .collect::<Vec<_>>()
        .join(", ")
}

fn output_fields(
    out: &mut dyn std::io::Write,
    indentation: usize,
    fields: &[Named<Format>],
    known_sizes: &HashSet<&str>,
    namespace_prefix: &str,
) -> Result<()> {
    let tab = " ".repeat(indentation);
    for field in fields {
        writeln!(
            out,
            "{}{} {};",
            tab,
            quote_type(&field.value, Some(known_sizes), namespace_prefix),
            field.name
        )?;
    }
    Ok(())
}

fn output_variant(
    out: &mut dyn std::io::Write,
    name: &str,
    variant: &VariantFormat,
    known_sizes: &HashSet<&str>,
    namespace: Option<&str>,
) -> Result<()> {
    use VariantFormat::*;
    let operator = format!("friend bool operator==(const {}&, const {}&);", name, name);
    let namespace_prefix = match namespace {
        None => "::".into(),
        Some(name) => format!("{}::", name),
    };
    match variant {
        Unit => writeln!(out, "    struct {} {{\n        {}\n    }};", name, operator),
        NewType(format) => writeln!(
            out,
            "    struct {} {{\n        {} value;\n        {}\n    }};",
            name,
            quote_type(format, Some(known_sizes), &namespace_prefix),
            operator,
        ),
        Tuple(formats) => writeln!(
            out,
            "    struct {} {{\n        std::tuple<{}> value;\n        {}\n    }};",
            name,
            quote_types(formats, Some(known_sizes), &namespace_prefix),
            operator
        ),
        Struct(fields) => {
            writeln!(out, "    struct {} {{", name)?;
            output_fields(out, 8, fields, known_sizes, &namespace_prefix)?;
            writeln!(out, "        {}\n    }};", operator)
        }
        Variable(_) => panic!("incorrect value"),
    }
}

fn output_variants(
    out: &mut dyn std::io::Write,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    known_sizes: &HashSet<&str>,
    namespace: Option<&str>,
) -> Result<()> {
    for (expected_index, (index, variant)) in variants.iter().enumerate() {
        assert_eq!(*index, expected_index as u32);
        output_variant(out, &variant.name, &variant.value, known_sizes, namespace)?;
    }
    Ok(())
}

fn output_container_forward_definition(out: &mut dyn std::io::Write, name: &str) -> Result<()> {
    writeln!(out, "struct {};\n", name)
}

fn output_container(
    out: &mut dyn std::io::Write,
    name: &str,
    format: &ContainerFormat,
    known_sizes: &HashSet<&str>,
    namespace: Option<&str>,
) -> Result<()> {
    use ContainerFormat::*;
    let operator = format!("friend bool operator==(const {}&, const {}&);", name, name);
    match format {
        UnitStruct => writeln!(out, "struct {} {{\n    {}\n}};\n", name, operator),
        NewTypeStruct(format) => writeln!(
            out,
            "struct {} {{\n    {} value;\n    {}\n}};\n",
            name,
            quote_type(format, Some(known_sizes), ""),
            operator,
        ),
        TupleStruct(formats) => writeln!(
            out,
            "struct {} {{\n    std::tuple<{}> value;\n    {}\n}};\n",
            name,
            quote_types(formats, Some(known_sizes), ""),
            operator,
        ),
        Struct(fields) => {
            writeln!(out, "struct {} {{", name)?;
            output_fields(out, 4, fields, known_sizes, "")?;
            writeln!(out, "    {}\n}};\n", operator)
        }
        Enum(variants) => {
            writeln!(out, "struct {} {{", name)?;
            output_variants(out, variants, known_sizes, namespace)?;
            writeln!(
                out,
                "    std::variant<{}> value;\n    {}\n}};\n",
                variants
                    .iter()
                    .map(|(_, v)| v.name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
                operator,
            )
        }
    }
}

fn output_struct_equality_test(
    out: &mut dyn std::io::Write,
    name: &str,
    fields: &[&str],
) -> Result<()> {
    writeln!(
        out,
        "bool operator==(const {} &lhs, const {} &rhs) {{",
        name, name,
    )?;
    for field in fields {
        writeln!(
            out,
            "    if (!(lhs.{} == rhs.{})) {{ return false; }}",
            field, field,
        )?;
    }
    writeln!(out, "    return true;\n}}")
}

fn output_struct_serializable(
    out: &mut dyn std::io::Write,
    name: &str,
    fields: &[&str],
) -> Result<()> {
    writeln!(
        out,
        r#"
template <>
template <typename Serializer>
void serde::Serializable<{}>::serialize(const {} &obj, Serializer &serializer) {{"#,
        name, name,
    )?;
    for field in fields {
        writeln!(
            out,
            "    serde::Serializable<decltype(obj.{})>::serialize(obj.{}, serializer);",
            field, field,
        )?;
    }
    writeln!(out, "}}")
}

fn output_struct_deserializable(
    out: &mut dyn std::io::Write,
    name: &str,
    fields: &[&str],
) -> Result<()> {
    writeln!(
        out,
        r#"
template <>
template <typename Deserializer>
{} serde::Deserializable<{}>::deserialize(Deserializer &deserializer) {{
    {} obj;"#,
        name, name, name,
    )?;
    for field in fields {
        writeln!(
            out,
            "    obj.{} = serde::Deserializable<decltype(obj.{})>::deserialize(deserializer);",
            field, field,
        )?;
    }
    writeln!(out, "    return obj;\n}}")
}

fn output_struct_traits(
    out: &mut dyn std::io::Write,
    name: &str,
    fields: &[&str],
    namespace: Option<&str>,
) -> Result<()> {
    output_open_namespace(out, namespace)?;
    output_struct_equality_test(out, name, fields)?;
    output_close_namespace(out, namespace)?;
    let namespaced_name = match namespace {
        None => name.into(),
        Some(prefix) => format!("{}::{}", prefix, name),
    };
    output_struct_serializable(out, &namespaced_name, fields)?;
    output_struct_deserializable(out, &namespaced_name, fields)
}

fn get_variant_fields(format: &VariantFormat) -> Vec<&str> {
    use VariantFormat::*;
    match format {
        Unit => Vec::new(),
        NewType(_format) => vec!["value"],
        Tuple(_formats) => vec!["value"],
        Struct(fields) => fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        Variable(_) => panic!("incorrect value"),
    }
}

fn output_container_traits(
    out: &mut dyn std::io::Write,
    name: &str,
    format: &ContainerFormat,
    namespace: Option<&str>,
) -> Result<()> {
    use ContainerFormat::*;
    match format {
        UnitStruct => output_struct_traits(out, name, &[], namespace),
        NewTypeStruct(_format) => output_struct_traits(out, name, &["value"], namespace),
        TupleStruct(_formats) => output_struct_traits(out, name, &["value"], namespace),
        Struct(fields) => output_struct_traits(
            out,
            name,
            &fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            namespace,
        ),
        Enum(variants) => {
            output_struct_traits(out, name, &["value"], namespace)?;
            for variant in variants.values() {
                output_struct_traits(
                    out,
                    &format!("{}::{}", name, variant.name),
                    &get_variant_fields(&variant.value),
                    namespace,
                )?;
            }
            Ok(())
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

    fn create_header_file(&self, name: &str) -> Result<std::fs::File> {
        let dir_path = &self.install_dir;
        std::fs::create_dir_all(dir_path)?;
        std::fs::File::create(dir_path.join(name.to_string() + ".hpp"))
    }
}

impl crate::SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_module(
        &self,
        name: &str,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error> {
        let mut file = self.create_header_file(name)?;
        output(&mut file, &registry, Some(name))
    }

    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error> {
        let mut file = self.create_header_file("serde")?;
        write!(file, "{}", include_str!("../runtime/cpp/serde.hpp"))?;
        Ok(())
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        let mut file = self.create_header_file("bincode")?;
        write!(file, "{}", include_str!("../runtime/cpp/bincode.hpp"))?;
        Ok(())
    }

    fn install_lcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        let mut file = self.create_header_file("lcs")?;
        write!(file, "{}", include_str!("../runtime/cpp/lcs.hpp"))?;
        Ok(())
    }
}
