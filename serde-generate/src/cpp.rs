// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
    analyzer,
    indent::{IndentConfig, IndentedWriter},
    CodegenConfig,
};
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{Result, Write};
use std::path::PathBuf;

/// Main configuration object for code-generation in C++.
pub struct CppCodegenConfig<'a> {
    /// Language-independent configuration.
    inner: &'a CodegenConfig,
    /// Mapping from external type names to suitably qualified names (e.g. "MyClass" -> "name::MyClass").
    /// Derived from `config.external_definitions`.
    external_qualified_names: HashMap<String, String>,
}

/// Shared state for the code generation of a C++ source file.
struct CppEmitter<'a, T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Configuration.
    config: &'a CppCodegenConfig<'a>,
    /// Track which type names have been declared so far. (Used to add forward declarations.)
    known_names: HashSet<&'a str>,
    /// Track which definitions have a known size. (Used to add shared pointers.)
    known_sizes: HashSet<&'a str>,
    /// Current namespace (e.g. vec!["name", "MyClass"])
    current_namespace: Vec<String>,
}

impl<'a> CppCodegenConfig<'a> {
    pub fn new(inner: &'a CodegenConfig) -> Self {
        let mut external_qualified_names = HashMap::new();
        for (namespace, names) in &inner.external_definitions {
            for name in names {
                external_qualified_names
                    .insert(name.to_string(), format!("{}::{}", namespace, name));
            }
        }
        Self {
            inner,
            external_qualified_names,
        }
    }

    pub fn output(
        &self,
        out: &mut dyn Write,
        registry: &Registry,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let current_namespace = self
            .inner
            .module_name
            .split("::")
            .map(String::from)
            .collect();
        let mut emitter = CppEmitter {
            out: IndentedWriter::new(out, IndentConfig::Space(4)),
            config: self,
            known_names: HashSet::new(),
            known_sizes: HashSet::new(),
            current_namespace,
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

    fn enter_class(&mut self, name: &str) {
        self.out.indent();
        self.current_namespace.push(name.to_string());
    }

    fn leave_class(&mut self) {
        self.out.unindent();
        self.current_namespace.pop();
    }

    fn output_comment(&mut self, name: &str) -> std::io::Result<()> {
        let mut path = self.current_namespace.clone();
        path.push(name.to_string());
        if let Some(doc) = self.config.inner.comments.get(&path) {
            let text = textwrap::indent(doc, "/// ").replace("\n\n", "\n///\n");
            write!(self.out, "\n{}", text)?;
        }
        Ok(())
    }

    /// Compute a fully qualified reference to the container type `name`.
    fn quote_qualified_name(&self, name: &str) -> String {
        self.config
            .external_qualified_names
            .get(name)
            .cloned()
            .unwrap_or_else(|| format!("{}::{}", self.config.inner.module_name, name))
    }

    fn quote_type(&self, format: &Format, require_known_size: bool) -> String {
        use Format::*;
        match format {
            TypeName(x) => {
                let qname = self.quote_qualified_name(x);
                if require_known_size && !self.known_sizes.contains(x.as_str()) {
                    // Cannot use unique_ptr because we need a copy constructor (e.g. for vectors).
                    format!("std::shared_ptr<{}>", qname)
                } else {
                    qname
                }
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
                self.quote_type(format, require_known_size)
            ),
            Seq(format) => format!("std::vector<{}>", self.quote_type(format, false)),
            Map { key, value } => format!(
                "std::map<{}, {}>",
                self.quote_type(key, false),
                self.quote_type(value, false)
            ),
            Tuple(formats) => format!(
                "std::tuple<{}>",
                self.quote_types(formats, require_known_size)
            ),
            TupleArray { content, size } => format!(
                "std::array<{}, {}>",
                self.quote_type(content, require_known_size),
                *size
            ),

            Variable(_) => panic!("unexpected value"),
        }
    }

    fn quote_types(&self, formats: &[Format], require_known_size: bool) -> String {
        formats
            .iter()
            .map(|x| self.quote_type(x, require_known_size))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn output_fields(&mut self, fields: &[Named<Format>]) -> Result<()> {
        for field in fields {
            writeln!(
                self.out,
                "{} {};",
                self.quote_type(&field.value, true),
                field.name
            )?;
        }
        Ok(())
    }

    fn output_variant(&mut self, name: &str, variant: &VariantFormat) -> Result<()> {
        self.output_comment(name)?;
        use VariantFormat::*;
        let operator = format!("friend bool operator==(const {}&, const {}&);", name, name);
        match variant {
            Unit => writeln!(self.out, "struct {} {{\n    {}\n}};", name, operator),
            NewType(format) => writeln!(
                self.out,
                "struct {} {{\n    {} value;\n    {}\n}};",
                name,
                self.quote_type(format, true),
                operator,
            ),
            Tuple(formats) => writeln!(
                self.out,
                "struct {} {{\n    std::tuple<{}> value;\n    {}\n}};",
                name,
                self.quote_types(formats, true),
                operator
            ),
            Struct(fields) => {
                self.output_comment(name)?;
                writeln!(self.out, "struct {} {{", name)?;
                self.enter_class(name);
                self.output_fields(fields)?;
                writeln!(self.out, "{}", operator)?;
                self.leave_class();
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
                self.quote_type(format, true),
                operator,
            ),
            TupleStruct(formats) => writeln!(
                self.out,
                "struct {} {{\n    std::tuple<{}> value;\n    {}\n}};\n",
                name,
                self.quote_types(formats, true),
                operator,
            ),
            Struct(fields) => {
                self.output_comment(name)?;
                writeln!(self.out, "struct {} {{", name)?;
                self.enter_class(name);
                self.output_fields(fields)?;
                writeln!(self.out, "{}", operator)?;
                self.leave_class();
                writeln!(self.out, "}};\n")
            }
            Enum(variants) => {
                self.output_comment(name)?;
                writeln!(self.out, "struct {} {{", name)?;
                self.enter_class(name);
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
                self.leave_class();
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
        let namespaced_name = self.quote_qualified_name(name);
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
