// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
    analyzer,
    indent::{IndentConfig, IndentedWriter},
    CodegenConfig,
};
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};
use std::collections::{BTreeMap, HashSet};
use std::io::{Result, Write};
use std::path::PathBuf;

/// Main configuration object for code-generation in C++.
pub struct CppCodegenConfig<'a> {
    /// Language-independent configuration.
    inner: &'a CodegenConfig,
}

/// Shared state for the code generation of a C++ source file.
struct CppEmitter<'a, T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Configuration.
    config: &'a CppCodegenConfig<'a>,
    /// Qualifiers to be added in front of container names, when needed.
    namespace_prefix: String,
    /// Track which type names have been declared so far. (Used to add forward declarations.)
    known_names: HashSet<&'a str>,
    /// Track which definitions have a known size. (Used to add shared pointers.)
    known_sizes: HashSet<&'a str>,
}

impl<'a> CppCodegenConfig<'a> {
    pub fn new(inner: &'a CodegenConfig) -> Self {
        Self { inner }
    }

    pub fn output(
        &self,
        out: &mut dyn Write,
        registry: &Registry,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let namespace_prefix = format!("{}::", self.inner.module_name);
        let mut emitter = CppEmitter {
            out: IndentedWriter::new(out, IndentConfig::Space(4)),
            config: self,
            namespace_prefix,
            known_names: HashSet::new(),
            known_sizes: HashSet::new(),
        };

        emitter.output_preamble()?;
        emitter.output_open_namespace()?;

        let dependencies = analyzer::get_dependency_map(registry)?;
        let entries = analyzer::best_effort_topological_sort(&dependencies);

        for name in entries {
            for dependency in &dependencies[name] {
                if !emitter.known_names.contains(dependency) {
                    emitter.output_container_forward_definition(*dependency)?;
                    emitter.known_names.insert(*dependency);
                }
            }
            let format = &registry[name];
            emitter.output_container(name, format)?;
            emitter.known_sizes.insert(name);
            emitter.known_names.insert(name);
        }

        emitter.output_close_namespace()?;
        writeln!(emitter.out)?;
        for (name, format) in registry {
            emitter.output_container_traits(&name, format)?;
        }
        Ok(())
    }
}

impl<'a, T> CppEmitter<'a, T>
where
    T: std::io::Write,
{
    fn output_preamble(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"#pragma once

#include "serde.hpp""#
        )
    }

    fn output_open_namespace(&mut self) -> Result<()> {
        writeln!(
            self.out,
            "\nnamespace {} {{\n",
            self.config.inner.module_name
        )?;
        self.out.indent();
        Ok(())
    }

    fn output_close_namespace(&mut self) -> Result<()> {
        self.out.unindent();
        writeln!(
            self.out,
            "}} // end of namespace {}",
            self.config.inner.module_name
        )?;
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
                Self::quote_type(format, known_sizes, namespace_prefix)
            ),
            Seq(format) => format!(
                "std::vector<{}>",
                Self::quote_type(format, None, namespace_prefix)
            ),
            Map { key, value } => format!(
                "std::map<{}, {}>",
                Self::quote_type(key, None, namespace_prefix),
                Self::quote_type(value, None, namespace_prefix)
            ),
            Tuple(formats) => format!(
                "std::tuple<{}>",
                Self::quote_types(formats, known_sizes, namespace_prefix)
            ),
            TupleArray { content, size } => format!(
                "std::array<{}, {}>",
                Self::quote_type(content, known_sizes, namespace_prefix),
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
            .map(|x| Self::quote_type(x, known_sizes, namespace_prefix))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn output_fields(&mut self, fields: &[Named<Format>]) -> Result<()> {
        for field in fields {
            writeln!(
                self.out,
                "{} {};",
                Self::quote_type(
                    &field.value,
                    Some(&self.known_sizes),
                    &self.namespace_prefix
                ),
                field.name
            )?;
        }
        Ok(())
    }

    fn output_variant(&mut self, name: &str, variant: &VariantFormat) -> Result<()> {
        use VariantFormat::*;
        let operator = format!("friend bool operator==(const {}&, const {}&);", name, name);
        match variant {
            Unit => writeln!(self.out, "struct {} {{\n    {}\n}};", name, operator),
            NewType(format) => writeln!(
                self.out,
                "struct {} {{\n    {} value;\n    {}\n}};",
                name,
                Self::quote_type(format, Some(&self.known_sizes), &self.namespace_prefix),
                operator,
            ),
            Tuple(formats) => writeln!(
                self.out,
                "struct {} {{\n    std::tuple<{}> value;\n    {}\n}};",
                name,
                Self::quote_types(formats, Some(&self.known_sizes), &self.namespace_prefix),
                operator
            ),
            Struct(fields) => {
                writeln!(self.out, "struct {} {{", name)?;
                self.out.indent();
                self.output_fields(fields)?;
                writeln!(self.out, "{}", operator)?;
                self.out.unindent();
                writeln!(self.out, "}};")
            }
            Variable(_) => panic!("incorrect value"),
        }
    }

    fn output_variants(&mut self, variants: &BTreeMap<u32, Named<VariantFormat>>) -> Result<()> {
        for (expected_index, (index, variant)) in variants.iter().enumerate() {
            assert_eq!(*index, expected_index as u32);
            self.output_variant(&variant.name, &variant.value)?;
        }
        Ok(())
    }

    fn output_container_forward_definition(&mut self, name: &str) -> Result<()> {
        writeln!(self.out, "struct {};\n", name)
    }

    fn output_container(&mut self, name: &str, format: &ContainerFormat) -> Result<()> {
        use ContainerFormat::*;
        let operator = format!("friend bool operator==(const {0}&, const {0}&);", name);
        match format {
            UnitStruct => writeln!(self.out, "struct {} {{\n    {}\n}};\n", name, operator),
            NewTypeStruct(format) => writeln!(
                self.out,
                "struct {} {{\n    {} value;\n    {}\n}};\n",
                name,
                Self::quote_type(format, Some(&self.known_sizes), ""),
                operator,
            ),
            TupleStruct(formats) => writeln!(
                self.out,
                "struct {} {{\n    std::tuple<{}> value;\n    {}\n}};\n",
                name,
                Self::quote_types(formats, Some(&self.known_sizes), ""),
                operator,
            ),
            Struct(fields) => {
                writeln!(self.out, "struct {} {{", name)?;
                self.out.indent();
                self.output_fields(fields)?;
                writeln!(self.out, "{}", operator)?;
                self.out.unindent();
                writeln!(self.out, "}};\n")
            }
            Enum(variants) => {
                writeln!(self.out, "struct {} {{", name)?;
                self.out.indent();
                self.output_variants(variants)?;
                writeln!(
                    self.out,
                    "std::variant<{}> value;",
                    variants
                        .iter()
                        .map(|(_, v)| v.name.clone())
                        .collect::<Vec<_>>()
                        .join(", "),
                )?;
                writeln!(self.out, "{}", operator)?;
                self.out.unindent();
                writeln!(self.out, "}};\n")
            }
        }
    }

    fn output_struct_equality_test(&mut self, name: &str, fields: &[&str]) -> Result<()> {
        writeln!(
            self.out,
            "inline bool operator==(const {0} &lhs, const {0} &rhs) {{",
            name,
        )?;
        self.out.indent();
        for field in fields {
            writeln!(
                self.out,
                "if (!(lhs.{0} == rhs.{0})) {{ return false; }}",
                field,
            )?;
        }
        writeln!(self.out, "return true;")?;
        self.out.unindent();
        writeln!(self.out, "}}\n")
    }

    fn output_struct_serializable(&mut self, name: &str, fields: &[&str]) -> Result<()> {
        writeln!(
            self.out,
            r#"
template <>
template <typename Serializer>
void serde::Serializable<{0}>::serialize(const {0} &obj, Serializer &serializer) {{"#,
            name,
        )?;
        self.out.indent();
        for field in fields {
            writeln!(
                self.out,
                "serde::Serializable<decltype(obj.{0})>::serialize(obj.{0}, serializer);",
                field,
            )?;
        }
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_struct_deserializable(&mut self, name: &str, fields: &[&str]) -> Result<()> {
        writeln!(
            self.out,
            r#"
template <>
template <typename Deserializer>
{0} serde::Deserializable<{0}>::deserialize(Deserializer &deserializer) {{"#,
            name,
        )?;
        self.out.indent();
        writeln!(self.out, "{} obj;", name)?;
        for field in fields {
            writeln!(
                self.out,
                "obj.{0} = serde::Deserializable<decltype(obj.{0})>::deserialize(deserializer);",
                field,
            )?;
        }
        writeln!(self.out, "return obj;")?;
        self.out.unindent();
        writeln!(self.out, "}}")
    }

    fn output_struct_traits(&mut self, name: &str, fields: &[&str]) -> Result<()> {
        self.output_open_namespace()?;
        self.output_struct_equality_test(name, fields)?;
        self.output_close_namespace()?;
        let namespaced_name = format!("{}{}", self.namespace_prefix, name);
        if self.config.inner.serialization {
            self.output_struct_serializable(&namespaced_name, fields)?;
            self.output_struct_deserializable(&namespaced_name, fields)?;
        }
        Ok(())
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

    fn output_container_traits(&mut self, name: &str, format: &ContainerFormat) -> Result<()> {
        use ContainerFormat::*;
        match format {
            UnitStruct => self.output_struct_traits(name, &[]),
            NewTypeStruct(_format) => self.output_struct_traits(name, &["value"]),
            TupleStruct(_formats) => self.output_struct_traits(name, &["value"]),
            Struct(fields) => self.output_struct_traits(
                name,
                &fields
                    .iter()
                    .map(|field| field.name.as_str())
                    .collect::<Vec<_>>(),
            ),
            Enum(variants) => {
                self.output_struct_traits(name, &["value"])?;
                for variant in variants.values() {
                    self.output_struct_traits(
                        &format!("{}::{}", name, variant.name),
                        &Self::get_variant_fields(&variant.value),
                    )?;
                }
                Ok(())
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
        inner: &crate::CodegenConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error> {
        let mut file = self.create_header_file(&inner.module_name)?;
        let config = CppCodegenConfig::new(&inner);
        config.output(&mut file, &registry)
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
