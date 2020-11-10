// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
    common,
    indent::{IndentConfig, IndentedWriter},
    CodeGeneratorConfig, Encoding,
};
use heck::CamelCase;
use include_dir::include_dir as include_directory;
use serde_reflection::{ContainerFormat, Format, FormatHolder, Named, Registry, VariantFormat};
use std::{
    collections::{BTreeMap, HashMap},
    io::{Result, Write},
    path::PathBuf,
};

/// Main configuration object for code-generation in C#.
pub struct CodeGenerator<'a> {
    /// Language-independent configuration.
    config: &'a CodeGeneratorConfig,
    /// Mapping from external type names to fully-qualified class names (e.g. "MyClass" -> "MyNamespace.MyClass").
    /// Derived from `config.external_definitions`.
    external_qualified_names: HashMap<String, String>,
}

/// Shared state for the code generation of a C# source file.
struct CsharpEmitter<'a, T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Generator.
    generator: &'a CodeGenerator<'a>,
    /// Current namespace (e.g. vec!["MyNamespace", "MyClass"])
    current_namespace: Vec<String>,
    /// Current (non-qualified) generated class names that could clash with names in the registry
    /// (e.g. "Builder" or variant classes).
    /// * We count multiplicities to allow inplace backtracking.
    /// * Names in the registry (and a few base types such as "Decimal") are assumed to never clash.
    current_reserved_names: HashMap<String, usize>,
}

impl<'a> CodeGenerator<'a> {
    /// Create a C# code generator for the given config.
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        let mut external_qualified_names = HashMap::new();
        for (namespace, names) in &config.external_definitions {
            for name in names {
                external_qualified_names
                    .insert(name.to_string(), format!("{}.{}", namespace, name));
            }
        }
        Self {
            config,
            external_qualified_names,
        }
    }

    /// Output class definitions for `registry` in separate source files.
    /// Source files will be created in a subdirectory of `install_dir` corresponding to the given
    /// package name (if any, otherwise `install_dir` it self).
    pub fn write_source_files(
        &self,
        install_dir: std::path::PathBuf,
        registry: &Registry,
    ) -> Result<()> {
        let current_namespace = self
            .config
            .module_name
            .split('.')
            .map(String::from)
            .collect::<Vec<_>>();

        let mut dir_path = install_dir;
        for part in &current_namespace {
            dir_path = dir_path.join(part);
        }
        std::fs::create_dir_all(&dir_path)?;

        for (name, format) in registry {
            self.write_container_class(&dir_path, current_namespace.clone(), name, format)?;
        }
        if self.config.serialization {
            self.write_helper_class(&dir_path, current_namespace, registry)?;
        }
        Ok(())
    }

    fn write_container_class(
        &self,
        dir_path: &std::path::Path,
        current_namespace: Vec<String>,
        name: &str,
        format: &ContainerFormat,
    ) -> Result<()> {
        let mut file = std::fs::File::create(dir_path.join(name.to_string() + ".cs"))?;
        let mut emitter = CsharpEmitter {
            out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
            generator: self,
            current_namespace,
            current_reserved_names: HashMap::new(),
        };

        emitter.output_preamble()?;
        emitter.output_open_namespace()?;
        emitter.output_container(name, format)?;
        emitter.output_close_namespace()?;

        Ok(())
    }

    fn write_helper_class(
        &self,
        dir_path: &std::path::Path,
        current_namespace: Vec<String>,
        registry: &Registry,
    ) -> Result<()> {
        let mut file = std::fs::File::create(dir_path.join("TraitHelpers.cs"))?;
        let mut emitter = CsharpEmitter {
            out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
            generator: self,
            current_namespace,
            current_reserved_names: HashMap::new(),
        };

        emitter.output_preamble()?;
        emitter.output_open_namespace()?;
        emitter.output_trait_helpers(registry)?;
        emitter.output_close_namespace()?;

        Ok(())
    }
}

impl<'a, T> CsharpEmitter<'a, T>
where
    T: Write,
{
    fn output_preamble(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r"using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;
using AD.FunctionalExtensions;"
        )?;
        Ok(())
    }

    fn output_open_namespace(&mut self) -> Result<()> {
        writeln!(
            self.out,
            "\nnamespace {} {{",
            self.generator.config.module_name
        )?;
        self.out.indent();
        Ok(())
    }

    fn output_close_namespace(&mut self) -> Result<()> {
        self.out.unindent();
        writeln!(
            self.out,
            "\n}} // end of namespace {}",
            self.generator.config.module_name
        )?;
        Ok(())
    }

    /// Compute a safe reference to the registry type `name` in the given context.
    /// If `name` is not marked as "reserved" (e.g. "Builder"), we compare the global
    /// name `self.qualified_names[name]` with the current namespace and try to use the
    /// short string `name` if possible.
    fn quote_qualified_name(&self, name: &str) -> String {
        let qname = self
            .generator
            .external_qualified_names
            .get(name)
            .cloned()
            .unwrap_or_else(|| format!("{}.{}", self.generator.config.module_name, name));
        let mut path = qname.split('.').collect::<Vec<_>>();
        if path.len() <= 1 {
            return qname;
        }
        let name = path.pop().unwrap();
        if self.current_reserved_names.contains_key(name) {
            return qname;
        }
        for (index, element) in path.iter().enumerate() {
            match self.current_namespace.get(index) {
                Some(e) if e == element => (),
                _ => {
                    return qname;
                }
            }
        }
        name.to_string()
    }

    fn output_comment(&mut self, name: &str) -> std::io::Result<()> {
        let mut path = self.current_namespace.clone();
        path.push(name.to_string());
        if let Some(doc) = self.generator.config.comments.get(&path) {
            let text = textwrap::indent(doc, "/// ").replace("\n\n", "\n///\n");
            write!(self.out, "{}", text)?;
        }
        Ok(())
    }

    fn output_custom_code(&mut self) -> std::io::Result<()> {
        if let Some(code) = self
            .generator
            .config
            .custom_code
            .get(&self.current_namespace)
        {
            writeln!(self.out, "\n{}", code)?;
        }
        Ok(())
    }

    fn quote_type(&self, format: &Format) -> String {
        use Format::*;
        match format {
            TypeName(x) => self.quote_qualified_name(x),
            Unit => "Unit".into(),
            Bool => "bool".into(),
            I8 => "sbyte".into(),
            I16 => "short".into(),
            I32 => "int".into(),
            I64 => "long".into(),
            I128 => "BigInteger".into(),
            U8 => "byte".into(),
            U16 => "ushort".into(),
            U32 => "uint".into(),
            U64 => "ulong".into(),
            U128 => "BigInteger".into(),
            F32 => "float".into(),
            F64 => "double".into(),
            Char => "Rune".into(),
            Str => "string".into(),
            Bytes => "Bytes".into(),

            Option(format) => format!("Option<{}>", self.quote_type(format)),
            Seq(format) => format!("List<{}>", self.quote_type(format)),
            Map { key, value } => format!(
                "Dictionary<{}, {}>",
                self.quote_type(key),
                self.quote_type(value)
            ),
            Tuple(formats) => format!("({})", self.quote_types(formats)),
            TupleArray {
                content,
                size: _size,
            } => format!("{}[]", self.quote_type(content),),
            Variable(_) => panic!("unexpected value"),
        }
    }

    fn enter_class(&mut self, name: &str, reserved_subclass_names: &[&str]) {
        self.out.indent();
        self.current_namespace.push(name.to_string());
        for name in reserved_subclass_names {
            let entry = self
                .current_reserved_names
                .entry(name.to_string())
                .or_insert(0);
            *entry += 1;
        }
    }

    fn leave_class(&mut self, reserved_subclass_names: &[&str]) {
        self.out.unindent();
        self.current_namespace.pop();
        for name in reserved_subclass_names {
            let entry = self.current_reserved_names.get_mut(*name).unwrap();
            *entry -= 1;
            if *entry == 0 {
                self.current_reserved_names.remove(*name);
            }
        }
    }

    fn quote_types(&self, formats: &[Format]) -> String {
        formats
            .iter()
            .map(|f| self.quote_type(f))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn output_trait_helpers(&mut self, registry: &Registry) -> Result<()> {
        let mut subtypes = BTreeMap::new();
        for format in registry.values() {
            format
                .visit(&mut |f| {
                    if Self::needs_helper(f) {
                        subtypes.insert(common::mangle_type(f), f.clone());
                    }
                    Ok(())
                })
                .unwrap();
        }
        writeln!(self.out, "static class TraitHelpers {{")?;
        let reserved_names = &[];
        self.enter_class("TraitHelpers", reserved_names);
        for (mangled_name, subtype) in &subtypes {
            self.output_serialization_helper(mangled_name, subtype)?;
            self.output_deserialization_helper(mangled_name, subtype)?;
        }
        self.leave_class(reserved_names);
        writeln!(self.out, "}}\n")
    }

    fn needs_helper(format: &Format) -> bool {
        use Format::*;
        matches!(format, Option(_) | Seq(_) | Map { .. } | Tuple(_) | TupleArray { .. })
    }

    fn quote_serialize_value(&self, value: &str, format: &Format) -> String {
        use Format::*;
        match format {
            TypeName(_) => format!("{}.Serialize(serializer);", value),
            Unit => format!("serializer.serialize_unit({});", value),
            Bool => format!("serializer.serialize_bool({});", value),
            I8 => format!("serializer.serialize_i8({});", value),
            I16 => format!("serializer.serialize_i16({});", value),
            I32 => format!("serializer.serialize_i32({});", value),
            I64 => format!("serializer.serialize_i64({});", value),
            I128 => format!("serializer.serialize_i128({});", value),
            U8 => format!("serializer.serialize_u8({});", value),
            U16 => format!("serializer.serialize_u16({});", value),
            U32 => format!("serializer.serialize_u32({});", value),
            U64 => format!("serializer.serialize_u64({});", value),
            U128 => format!("serializer.serialize_u128({});", value),
            F32 => format!("serializer.serialize_f32({});", value),
            F64 => format!("serializer.serialize_f64({});", value),
            Char => format!("serializer.serialize_char({});", value),
            Str => format!("serializer.serialize_str({});", value),
            Bytes => format!("serializer.serialize_bytes({});", value),
            _ => format!(
                "{}.serialize_{}({}, serializer);",
                self.quote_qualified_name("TraitHelpers"),
                common::mangle_type(format),
                value
            ),
        }
    }

    fn quote_deserialize(&self, format: &Format) -> String {
        use Format::*;
        match format {
            TypeName(name) => format!(
                "{}.Deserialize(deserializer)",
                self.quote_qualified_name(name)
            ),
            Unit => "deserializer.deserialize_unit()".to_string(),
            Bool => "deserializer.deserialize_bool()".to_string(),
            I8 => "deserializer.deserialize_i8()".to_string(),
            I16 => "deserializer.deserialize_i16()".to_string(),
            I32 => "deserializer.deserialize_i32()".to_string(),
            I64 => "deserializer.deserialize_i64()".to_string(),
            I128 => "deserializer.deserialize_i128()".to_string(),
            U8 => "deserializer.deserialize_u8()".to_string(),
            U16 => "deserializer.deserialize_u16()".to_string(),
            U32 => "deserializer.deserialize_u32()".to_string(),
            U64 => "deserializer.deserialize_u64()".to_string(),
            U128 => "deserializer.deserialize_u128()".to_string(),
            F32 => "deserializer.deserialize_f32()".to_string(),
            F64 => "deserializer.deserialize_f64()".to_string(),
            Char => "deserializer.deserialize_char()".to_string(),
            Str => "deserializer.deserialize_str()".to_string(),
            Bytes => "deserializer.deserialize_bytes()".to_string(),
            _ => format!(
                "{}.deserialize_{}(deserializer)",
                self.quote_qualified_name("TraitHelpers"),
                common::mangle_type(format),
            ),
        }
    }

    fn output_serialization_helper(&mut self, name: &str, format0: &Format) -> Result<()> {
        use Format::*;

        write!(
            self.out,
            "public static void serialize_{}({} value, ISerializer serializer) {{",
            name,
            self.quote_type(format0)
        )?;
        self.out.indent();
        match format0 {
            Option(format) => {
                write!(
                    self.out,
                    r#"
if (value.IsSome(out var val)) {{
    serializer.serialize_option_tag(true);
    {}
}} else {{
    serializer.serialize_option_tag(false);
}}
"#,
                    self.quote_serialize_value("val", format)
                )?;
            }

            Seq(format) => {
                write!(
                    self.out,
                    r#"
serializer.serialize_len(value.Count);
foreach (var item in value) {{
    {}
}}
"#,
                    self.quote_serialize_value("item", format)
                )?;
            }

            Map { key, value } => {
                write!(
                    self.out,
                    r#"
serializer.serialize_len(value.Count);
int[] offsets = new int[value.Count];
int count = 0;
foreach (var entry in value) {{
    offsets[count++] = serializer.get_buffer_offset();
    {}
    {}
}}
serializer.sort_map_entries(offsets);
"#,
                    self.quote_serialize_value("entry.Key", key),
                    self.quote_serialize_value("entry.Value", value)
                )?;
            }

            Tuple(formats) => {
                writeln!(self.out)?;
                for (index, format) in formats.iter().enumerate() {
                    let expr = format!("value.Item{}", index + 1);
                    writeln!(self.out, "{}", self.quote_serialize_value(&expr, format))?;
                }
            }

            TupleArray { content, size } => {
                write!(
                    self.out,
                    r#"
if (value.Length != {0}) {{
    throw new ArgumentException("Invalid length for fixed-size array: " + value.Length + " instead of "+ {0});
}}
foreach (var item in value) {{
    {1}
}}
"#,
                    size,
                    self.quote_serialize_value("item", content),
                )?;
            }

            _ => panic!("unexpected case"),
        }
        self.out.unindent();
        writeln!(self.out, "}}\n")
    }

    fn output_deserialization_helper(&mut self, name: &str, format0: &Format) -> Result<()> {
        use Format::*;

        write!(
            self.out,
            "public static {} deserialize_{}(IDeserializer deserializer) {{",
            self.quote_type(format0),
            name,
        )?;
        self.out.indent();
        match format0 {
            Option(format) => {
                write!(
                    self.out,
                    r#"
bool tag = deserializer.deserialize_option_tag();
if (!tag) {{
    return Option<{}>.None;
}} else {{
    return Option.Some({});
}}
"#,
                    self.quote_type(format),
                    self.quote_deserialize(format),
                )?;
            }

            Seq(format) => {
                write!(
                    self.out,
                    r#"
long length = deserializer.deserialize_len();
var obj = new List<{0}>((int)length);
for (long i = 0; i < length; i++) {{
    obj.Add({1});
}}
return obj;
"#,
                    self.quote_type(format),
                    self.quote_deserialize(format)
                )?;
            }

            Map { key, value } => {
                write!(
                    self.out,
                    r#"
long length = deserializer.deserialize_len();
var obj = new Dictionary<{}, {}>();
int previous_key_start = 0;
int previous_key_end = 0;
for (long i = 0; i < length; i++) {{
    int key_start = deserializer.get_buffer_offset();
    var key = {};
    int key_end = deserializer.get_buffer_offset();
    if (i > 0) {{
        deserializer.check_that_key_slices_are_increasing(
            previous_key_start..previous_key_end, key_start..key_end);
    }}
    previous_key_start = key_start;
    previous_key_end = key_end;
    var value = {};
    obj[key] = value;
}}
return obj;
"#,
                    self.quote_type(key),
                    self.quote_type(value),
                    self.quote_deserialize(key),
                    self.quote_deserialize(value),
                )?;
            }

            Tuple(formats) => {
                write!(
                    self.out,
                    r#"
return ({}
);
"#,
                    formats
                        .iter()
                        .map(|f| format!("\n    {}", self.quote_deserialize(f)))
                        .collect::<Vec<_>>()
                        .join(",")
                )?;
            }

            TupleArray { content, size } => {
                write!(
                    self.out,
                    r#"
{0}[] obj = new {0}[{1}];
for (int i = 0; i < {1}; i++) {{
    obj[i] = {2};
}}
return obj;
"#,
                    self.quote_type(content),
                    size,
                    self.quote_deserialize(content)
                )?;
            }

            _ => panic!("unexpected case"),
        }
        self.out.unindent();
        writeln!(self.out, "}}\n")
    }

    fn output_variant(
        &mut self,
        base: &str,
        index: u32,
        name: &str,
        variant: &VariantFormat,
    ) -> Result<()> {
        use VariantFormat::*;
        let fields = match variant {
            Unit => Vec::new(),
            NewType(format) => vec![Named {
                name: "Value".to_string(),
                value: format.as_ref().clone(),
            }],
            Tuple(formats) => formats
                .iter()
                .enumerate()
                .map(|(i, f)| Named {
                    name: format!("Field{}", i),
                    value: f.clone(),
                })
                .collect(),
            Struct(fields) => fields.clone(),
            Variable(_) => panic!("incorrect value"),
        };
        self.output_struct_or_variant_container(Some(base), Some(index), name, &fields)
    }

    fn output_variants(
        &mut self,
        base: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> Result<()> {
        for (index, variant) in variants {
            self.output_variant(base, *index, &variant.name, &variant.value)?;
        }
        Ok(())
    }

    fn output_struct_or_variant_container(
        &mut self,
        variant_base: Option<&str>,
        variant_index: Option<u32>,
        name: &str,
        fields: &[Named<Format>],
    ) -> Result<()> {
        // Beginning of class
        writeln!(self.out)?;
        let fn_mods = if let Some(base) = variant_base {
            self.output_comment(name)?;
            writeln!(self.out, "public sealed class {}: {} {{", name, base)?;
            "override "
        } else {
            self.output_comment(name)?;
            writeln!(self.out, "public sealed class {} {{", name)?;
            ""
        };
        let reserved_names = &["Builder"];
        self.enter_class(name, reserved_names);
        // Fields
        for field in fields {
            self.output_comment(&field.name)?;
            writeln!(
                self.out,
                "public {} {};",
                self.quote_type(&field.value),
                field.name
            )?;
        }
        if !fields.is_empty() {
            writeln!(self.out)?;
        }
        // Constructor.
        writeln!(
            self.out,
            "public {}({}) {{",
            name,
            fields
                .iter()
                .map(|f| format!("{} _{}", self.quote_type(&f.value), &f.name))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        self.out.indent();
        // TODO: check arguments for null?
        for field in fields {
            writeln!(self.out, "{} = _{};", &field.name, &field.name)?;
        }
        self.out.unindent();
        writeln!(self.out, "}}")?;
        // Serialize
        if self.generator.config.serialization {
            writeln!(
                self.out,
                "\npublic {}void Serialize(ISerializer serializer) {{",
                fn_mods
            )?;
            self.out.indent();
            writeln!(self.out, "serializer.increase_container_depth();")?;
            if let Some(index) = variant_index {
                writeln!(self.out, "serializer.serialize_variant_index({});", index)?;
            }
            for field in fields {
                writeln!(
                    self.out,
                    "{}",
                    self.quote_serialize_value(&field.name, &field.value)
                )?;
            }
            writeln!(self.out, "serializer.decrease_container_depth();")?;
            self.out.unindent();
            writeln!(self.out, "}}")?;

            if variant_index.is_none() {
                for encoding in &self.generator.config.encodings {
                    self.output_class_serialize_for_encoding(*encoding)?;
                }
            }
        }
        // Deserialize (struct) or Load (variant)
        if self.generator.config.serialization {
            if variant_index.is_none() {
                writeln!(
                    self.out,
                    "\npublic static {}{} Deserialize(IDeserializer deserializer) {{",
                    fn_mods, name,
                )?;
            } else {
                writeln!(
                    self.out,
                    "\ninternal static {} Load(IDeserializer deserializer) {{",
                    name,
                )?;
            }
            self.out.indent();
            if fields.len() > 0 {
                writeln!(self.out, "deserializer.increase_container_depth();")?;
                writeln!(self.out, "Builder builder = new Builder();")?;
                for field in fields {
                    writeln!(
                        self.out,
                        "builder.{} = {};",
                        field.name,
                        self.quote_deserialize(&field.value)
                    )?;
                }
                writeln!(self.out, "deserializer.decrease_container_depth();")?;
                writeln!(self.out, "return builder.Build();")?;
            } else {
                writeln!(self.out, "return new {}();", name)?;
            }
            self.out.unindent();
            writeln!(self.out, "}}")?;

            if variant_index.is_none() {
                for encoding in &self.generator.config.encodings {
                    self.output_class_deserialize_for_encoding(name, *encoding)?;
                }
            }
        }
        // Equality
        write!(self.out, "\npublic override bool Equals(object obj) {{")?;
        self.out.indent();
        writeln!(
            self.out,
            r#"
if (obj == null) return false;
if (ReferenceEquals(this, obj)) return true;
if (GetType() != obj.GetType()) return false;
{0} other = ({0}) obj;"#,
            name,
        )?;
        for field in fields {
            if let Format::Seq(_) = field.value {
                writeln!(
                    self.out,
                    "if (!Enumerable.SequenceEqual({0}, other.{0})) return false;",
                    &field.name,
                )?;
            } else {
                writeln!(
                    self.out,
                    "if (!{0}.Equals(other.{0})) return false;",
                    &field.name,
                )?;
            }
        }
        writeln!(self.out, "return true;")?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        // Hashing
        writeln!(self.out, "\npublic override int GetHashCode() {{")?;
        self.out.indent();
        writeln!(self.out, "int value = 7;",)?;
        for field in fields {
            writeln!(
                self.out,
                "value = 31 * value + {0}.GetHashCode();",
                &field.name
            )?;
        }
        writeln!(self.out, "return value;")?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        if fields.len() > 0 {
            self.output_struct_or_variant_container_builder(name, fields)?;
        }
        // Custom code
        self.output_custom_code()?;
        // End of class
        self.leave_class(reserved_names);
        writeln!(self.out, "}}")
    }

    fn output_struct_or_variant_container_builder(
        &mut self,
        name: &str,
        fields: &[Named<Format>],
    ) -> Result<()> {
        // Beginning of builder class
        writeln!(self.out)?;
        writeln!(self.out, "public class Builder {{")?;
        let reserved_names = &[];
        self.enter_class("Builder", reserved_names);
        // Fields
        for field in fields {
            writeln!(
                self.out,
                "public {} {};",
                self.quote_type(&field.value),
                field.name
            )?;
        }
        if !fields.is_empty() {
            writeln!(self.out)?;
        }
        // Finalization
        writeln!(
            self.out,
            r#"public {0} Build() {{
    return new {0}({1}
    );
}}"#,
            name,
            fields
                .iter()
                .map(|f| format!("\n        {}", f.name))
                .collect::<Vec<_>>()
                .join(",")
        )?;
        // Custom code
        self.output_custom_code()?;
        // End of class
        self.leave_class(reserved_names);
        writeln!(self.out, "}}")
    }

    fn output_enum_container(
        &mut self,
        name: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> Result<()> {
        writeln!(self.out)?;
        self.output_comment(name)?;
        writeln!(self.out, "public abstract class {} {{", name)?;
        let reserved_names = variants
            .values()
            .map(|v| v.name.as_str())
            .collect::<Vec<_>>();
        self.enter_class(name, &reserved_names);
        if self.generator.config.serialization {
            writeln!(
                self.out,
                "\npublic abstract void Serialize(ISerializer serializer);"
            )?;
            write!(
                self.out,
                "\npublic static {} Deserialize(IDeserializer deserializer) {{",
                name
            )?;
            self.out.indent();
            writeln!(
                self.out,
                r#"
int index = deserializer.deserialize_variant_index();
switch (index) {{"#,
            )?;
            self.out.indent();
            for (index, variant) in variants {
                writeln!(
                    self.out,
                    "case {}: return {}.Load(deserializer);",
                    index, variant.name,
                )?;
            }
            writeln!(
                self.out,
                "default: throw new DeserializationException(\"Unknown variant index for {}: \" + index);",
                name,
            )?;
            self.out.unindent();
            writeln!(self.out, "}}")?;
            self.out.unindent();
            writeln!(self.out, "}}")?;

            for encoding in &self.generator.config.encodings {
                self.output_class_serialize_for_encoding(*encoding)?;
                self.output_class_deserialize_for_encoding(name, *encoding)?;
            }
        }

        self.output_variants(name, variants)?;
        self.leave_class(&reserved_names);
        writeln!(self.out, "}}\n")
    }

    fn output_class_serialize_for_encoding(&mut self, encoding: Encoding) -> Result<()> {
        writeln!(
            self.out,
            r#"
public byte[] {0}Serialize()  {{
    ISerializer serializer = new {0}.{0}Serializer();
    Serialize(serializer);
    return serializer.get_bytes();
}}"#,
            encoding.name().to_camel_case()
        )
    }

    fn output_class_deserialize_for_encoding(
        &mut self,
        name: &str,
        encoding: Encoding,
    ) -> Result<()> {
        writeln!(
            self.out,
            r#"
public static {0} {1}Deserialize(byte[] input) {{
    if (input == null) {{
         throw new DeserializationException("Cannot deserialize null array");
    }}
    IDeserializer deserializer = new {1}.{1}Deserializer(input);
    {0} value = Deserialize(deserializer);
    if (deserializer.get_buffer_offset() < input.Length) {{
         throw new DeserializationException("Some input bytes were not read");
    }}
    return value;
}}"#,
            name,
            encoding.name().to_camel_case()
        )
    }

    fn output_container(&mut self, name: &str, format: &ContainerFormat) -> Result<()> {
        use ContainerFormat::*;
        let fields = match format {
            UnitStruct => Vec::new(),
            NewTypeStruct(format) => vec![Named {
                name: "value".to_string(),
                value: format.as_ref().clone(),
            }],
            TupleStruct(formats) => formats
                .iter()
                .enumerate()
                .map(|(i, f)| Named {
                    name: format!("field{}", i),
                    value: f.clone(),
                })
                .collect::<Vec<_>>(),
            Struct(fields) => fields.clone(),
            Enum(variants) => {
                self.output_enum_container(name, variants)?;
                return Ok(());
            }
        };
        self.output_struct_or_variant_container(None, None, name, &fields)
    }
}

/// Installer for generated source files in C#.
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
        let generator = CodeGenerator::new(config);
        generator.write_source_files(self.install_dir.clone(), registry)?;
        Ok(())
    }

    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(include_directory!("runtime/csharp/Serde"), "Serde")
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(
            include_directory!("runtime/csharp/Serde/Bincode"),
            "Serde/Bincode",
        )
    }

    fn install_lcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(include_directory!("runtime/csharp/Serde/Lcs"), "Serde/Lcs")
    }
}
