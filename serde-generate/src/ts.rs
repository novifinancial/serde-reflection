// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{
    collections::{BTreeMap, HashMap},
    io::{Result, Write},
    path::PathBuf,
};

use include_dir::include_dir as include_directory;

use serde_reflection::{ContainerFormat, Format, FormatHolder, Named, Registry, VariantFormat};

use crate::{
    CodeGeneratorConfig,
    indent::{IndentConfig, IndentedWriter},
};

/// Main configuration object for code-generation in TypeScript.
pub struct CodeGenerator<'a> {
    /// Language-independent configuration.
    config: &'a CodeGeneratorConfig,
    /// Mapping from external type names to fully-qualified class names (e.g. "MyClass" -> "com.facebook.my_package.MyClass").
    /// Derived from `config.external_definitions`.
    external_qualified_names: HashMap<String, String>,
    /// Should include TraitHelpers preamble?
    import_trait_helpers: bool
}

/// Shared state for the code generation of a TypeScript source file.
struct TypeScriptEmitter<'a, T> {
    /// Writer.
    out: IndentedWriter<T>,
    /// Generator.
    generator: &'a CodeGenerator<'a>,
    /// Current namespace (e.g. vec!["com", "facebook", "my_package", "MyClass"])
    current_namespace: Vec<String>,
    /// Current (non-qualified) generated class names that could clash with names in the registry
    /// (e.g. "Builder" or variant classes).
    /// * We cound multiplicities to allow inplace backtracking.
    /// * Names in the registry (and a few base types such as "BigInteger") are assumed to never clash.
    current_reserved_names: HashMap<String, usize>,
}

impl<'a> CodeGenerator<'a> {
    /// Create a TypeScript code generator for the given config.
    pub fn new(config: &'a CodeGeneratorConfig, import_trait_helpers: bool) -> Self {
        let mut external_qualified_names = HashMap::new();
        for (_, names) in &config.external_definitions {
            for name in names {
                external_qualified_names
                    .insert(name.to_string(), format!("{}", name));
            }
        }
        Self {
            config,
            external_qualified_names,
            import_trait_helpers
        }
    }

    /// Output class definitions for ` registry` in separate source files.
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
        let mut file = std::fs::File::create(dir_path.join(name.to_string() + ".ts"))?;
        let mut emitter = TypeScriptEmitter {
            out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
            generator: self,
            current_namespace,
            current_reserved_names: HashMap::new(),
        };

        emitter.output_preamble(name)?;
        emitter.output_container(name, format)
    }

    fn write_helper_class(
        &self,
        dir_path: &std::path::Path,
        current_namespace: Vec<String>,
        registry: &Registry,
    ) -> Result<()> {
        let name = "traitHelpers.ts";
        let mut file = std::fs::File::create(dir_path.join(name))?;
        let mut emitter = TypeScriptEmitter {
            out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
            generator: self,
            current_namespace,
            current_reserved_names: HashMap::new(),
        };

        emitter.output_preamble(name)?;
        emitter.output_trait_helpers(registry)
    }
}

impl<'a, T> TypeScriptEmitter<'a, T>
    where
        T: Write,
{
    fn output_preamble(&mut self, name: &str) -> Result<()> {
        writeln!(self.out, "import {{BigNumber}} from '@ethersproject/bignumber';")?;
        writeln!(self.out, "import {{Int64LE, Uint64LE}} from 'int64-buffer';\n")?;
        writeln!(self.out, "import bytes from '@ethersproject/bytes';\n")?;
        writeln!(self.out, "import {{ Serializer }} from '../serde/serializer';\n")?;
        writeln!(self.out, "import {{ Deserializer }} from '../serde/deserializer';\n")?;
        if self.generator.import_trait_helpers && !name.starts_with("traitHelpers") {
            writeln!(self.out, "import {{ TraitHelpers }} from './traitHelpers';")?;
        } else {
            writeln!(self.out, "{}\n",
                     self
                         .generator
                         .external_qualified_names
                         .iter()
                         .map(|(_, name)| format!("import {{ {0} }} from '../libraTypes/{0}';\n", name))
                         .collect::<String>()
            )?;
        }

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
            .unwrap_or_else(|| format!("{}", name));
        qname
    }

    fn output_comment(&mut self, name: &str) -> std::io::Result<()> {
        let mut path = self.current_namespace.clone();
        path.push(name.to_string());
        if let Some(doc) = self.generator.config.comments.get(&path) {
            let text = textwrap::indent(doc, " * ").replace("\n\n", "\n *\n");
            writeln!(self.out, "/**\n{} */", text)?;
        }
        Ok(())
    }

    fn quote_leading_path(&self, format: &Format) -> String {
        use Format::*;
        let mut ret: String = ".".into();
        match format {
            TypeName(x) => {
                if self.generator.external_qualified_names.contains_key(x) {
                    ret = "../libraTypes".into()
                }
            }
            _ => {}
        }
        ret
    }

    fn quote_type(&self, format: &Format) -> String {
        use Format::*;
        match format {
            TypeName(x) => self.quote_qualified_name(x),
            Unit => "void".into(),
            Bool => "boolean".into(),
            I8 => "number".into(),
            I16 => "number".into(),
            I32 => "number".into(),
            I64 => "Int64LE".into(),
            I128 => "BigNumber".into(),
            U8 => "number".into(),
            U16 => "number".into(),
            U32 => "number".into(),
            U64 => "Uint64LE".into(),
            U128 => "BigNumber".into(),
            F32 => "number".into(),
            F64 => "number".into(),
            Char => "string".into(),
            Str => "string".into(),
            Bytes => "Uint8Array".into(),

            Option(format) => format!("Optional{}", self.quote_type(format)),
            Seq(format) => format!("List{}", self.quote_type(format)),
            Map { key, value } => format!(
                "Map{}{}",
                self.quote_type(key),
                self.quote_type(value)
            ),
            Tuple(formats) => format!(
                "Tuple{}",
                self.quote_types(formats)
            ),
            TupleArray { content, size: _size } => format!(
                "ListTuple{}",
                self.quote_type(content),
            ),
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
            .join("")
    }

    fn output_trait_helpers(&mut self, registry: &Registry) -> Result<()> {
        let mut subtypes = BTreeMap::new();
        for format in registry.values() {
            format
                .visit(&mut |f| {
                    if Self::needs_helper(f) {
                        subtypes.insert(Self::mangle_type(f), f.clone());
                    }
                    Ok(())
                })
                .unwrap();
        }

        let mut imported = Vec::new();
        for (_mangled_name, subtype) in &subtypes {
            self.output_typedef_helper(subtype, &mut imported)?;
        }

        writeln!(self.out, "export class TraitHelpers {{")?;
        let reserved_names = &[];
        self.enter_class("TraitHelpers", reserved_names);
        for (mangled_name, subtype) in &subtypes {
            self.output_serialization_helper(mangled_name, subtype)?;
            self.output_deserialization_helper(mangled_name, subtype)?;
        }
        self.leave_class(reserved_names);
        writeln!(self.out, "}}\n")
    }

    fn mangle_type(format: &Format) -> String {
        use Format::*;
        match format {
            TypeName(x) => x.to_string(),
            Unit => "void".into(),
            Bool => "boolean".into(),
            I8 => "number".into(),
            I16 => "number".into(),
            I32 => "number".into(),
            I64 => "Int64LE".into(),
            I128 => "BigNumber".into(),
            U8 => "number".into(),
            U16 => "number".into(),
            U32 => "number".into(),
            U64 => "Uint64LE".into(),
            U128 => "BigNumber".into(),
            F32 => "number".into(),
            F64 => "number".into(),
            Char => "string".into(),
            Str => "string".into(),
            Bytes => "Uint8Array".into(),

            Option(format) => format!("Optional{}", Self::mangle_type(format)),
            Seq(format) => format!("List{}", Self::mangle_type(format)),
            Map { key, value } => format!(
                "Map{}{}",
                Self::mangle_type(key),
                Self::mangle_type(value)
            ),
            Tuple(formats) => format!(
                "Tuple{}",
                formats
                    .iter()
                    .map(Self::mangle_type)
                    .collect::<Vec<_>>()
                    .join("")
            ),
            TupleArray { content, size: _size } => {
                format!("ListTuple{}", Self::mangle_type(content))
            }
            Variable(_) => panic!("unexpected value"),
        }
    }

    fn needs_helper(format: &Format) -> bool {
        use Format::*;
        match format {
            Option(_) | Seq(_) | Map { .. } | Tuple(_) | TupleArray { .. } => true,
            _ => false,
        }
    }

    fn needs_import(format: &Format) -> bool {
        use Format::*;
        match format {
            TypeName { .. } => true,
            _ => false,
        }
    }

    fn quote_serialize_value(&self, value: &str, format: &Format, use_this: bool) -> String {
        use Format::*;
        let this_str = if use_this { "this." } else { "" };

        match format {
            TypeName(_) => format!("{}{}.serialize(serializer);", this_str, value),
            Unit => format!("serializer.serializeUnit({}{});", this_str, value),
            Bool => format!("serializer.serializeBool({}{});", this_str, value),
            I8 => format!("serializer.serializeI8({}{});", this_str, value),
            I16 => format!("serializer.serializeI16({}{});", this_str, value),
            I32 => format!("serializer.serializeI32({}{});", this_str, value),
            I64 => format!("serializer.serializeI64({}{});", this_str, value),
            I128 => format!("serializer.serializeI128({}{});", this_str, value),
            U8 => format!("serializer.serializeU8({}{});", this_str, value),
            U16 => format!("serializer.serializeU16({}{});", this_str, value),
            U32 => format!("serializer.serializeU32({}{});", this_str, value),
            U64 => format!("serializer.serializeU64({}{});", this_str, value),
            U128 => format!("serializer.serializeU128({}{});", this_str, value),
            F32 => format!("serializer.serializeF32({}{});", this_str, value),
            F64 => format!("serializer.serializeF64({}{});", this_str, value),
            Char => format!("serializer.serializeChar({}{});", this_str, value),
            Str => format!("serializer.serializeStr({}{});", this_str, value),
            Bytes => format!("serializer.serializeBytes({}{});", this_str, value),
            _ => format!(
                "{}.serialize{}({}{}, serializer);",
                self.quote_qualified_name("TraitHelpers"),
                Self::mangle_type(format),
                this_str,
                value
            ),
        }
    }

    fn quote_deserialize(&self, format: &Format) -> String {
        use Format::*;
        match format {
            TypeName(name) => format!(
                "{}.deserialize(deserializer)",
                self.quote_qualified_name(name)
            ),
            Unit => "deserializer.deserializeUnit()".to_string(),
            Bool => "deserializer.deserializeBool()".to_string(),
            I8 => "deserializer.deserializeI8()".to_string(),
            I16 => "deserializer.deserializeI16()".to_string(),
            I32 => "deserializer.deserializeI32()".to_string(),
            I64 => "deserializer.deserializeI64()".to_string(),
            I128 => "deserializer.deserializeI128()".to_string(),
            U8 => "deserializer.deserializeU8()".to_string(),
            U16 => "deserializer.deserializeU16()".to_string(),
            U32 => "deserializer.deserializeU32()".to_string(),
            U64 => "deserializer.deserializeU64()".to_string(),
            U128 => "deserializer.deserializeU128()".to_string(),
            F32 => "deserializer.deserializeF32()".to_string(),
            F64 => "deserializer.deserializeF64()".to_string(),
            Char => "deserializer.deserializeChar()".to_string(),
            Str => "deserializer.deserializeStr()".to_string(),
            Bytes => "deserializer.deserializeBytes()".to_string(),
            _ => format!(
                "{}.deserialize{}(deserializer)",
                self.quote_qualified_name("TraitHelpers"),
                Self::mangle_type(format),
            ),
        }
    }

    fn output_serialization_helper(&mut self, name: &str, format0: &Format) -> Result<()> {
        use Format::*;

        write!(
            self.out,
            "static serialize{}(value: {}, serializer: Serializer): void {{",
            name,
            self.quote_type(format0)
        )?;
        self.out.indent();
        match format0 {
            Option(format) => {
                write!(
                    self.out,
                    r#"
if (value) {{
    serializer.serializeOptionTag(true);
    {}
}} else {{
    serializer.serializeOptionTag(false);
}}
"#,
                    self.quote_serialize_value("value", format, false)
                )?;
            }

            Seq(format) => {
                write!(
                    self.out,
                    r#"
serializer.serializeLen(value.length);
value.forEach((item: {}) => {{
    {}
}});
"#,
                    self.quote_type(format),
                    self.quote_serialize_value("item", format, false)
                )?;
            }

            Map { key, value } => {
                write!(
                    self.out,
                    r#"
const keys = Object.keys(value);
serializer.serializeLen(keys.length);
const offsets: number[] = [];
keys.forEach((key: {}) => {{
    offsets.push(serializer.getBufferOffset());
    {}
    {}
}});
serializer.sortMapEntries(offsets);
"#,
                    self.quote_type(key),
                    self.quote_serialize_value("key", key, false),
                    self.quote_serialize_value("value[key]", value, false)
                )?;
            }

            Tuple(formats) => {
                writeln!(self.out)?;
                for (index, format) in formats.iter().enumerate() {
                    let expr = format!("value[{}]", index);
                    writeln!(self.out, "{}", self.quote_serialize_value(&expr, format, false))?;
                }
            }

            TupleArray { content, size: _size } => {
                write!(
                    self.out,
                    r#"
value.forEach((item) =>{{
    {}
}});
"#,
                    self.quote_serialize_value("item", content, false)
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
            "static deserialize{}(deserializer: Deserializer): {} {{",
            name,
            self.quote_type(format0),
        )?;
        self.out.indent();
        match format0 {
            Option(format) => {
                write!(
                    self.out,
                    r#"
const tag = deserializer.deserializeOptionTag();
if (!tag) {{
    return null;
}} else {{
    return {};
}}
"#,
                    self.quote_deserialize(format),
                )?;
            }

            Seq(format) => {
                write!(
                    self.out,
                    r#"
const length = deserializer.deserializeLen();
const list: {} = [];
for (let i = 0; i < length; i++) {{
    list.push({});
}}
return list;
"#,
                    self.quote_type(format0),
                    self.quote_deserialize(format)
                )?;
            }

            Map { key, value } => {
                write!(
                    self.out,
                    r#"
const length = deserializer.deserializeLen();
const obj: {{[key: {0}]: {1}}} = {{}};
let previousKeyStart = 0;
let previousKeyEnd = 0;
for (let i = 0; i < length; i++) {{
    const keyStart = deserializer.getBufferOffset();
    const key = {2};
    const keyEnd = deserializer.getBufferOffset();
    if (i > 0) {{
        deserializer.checkThatKeySlicesAreIncreasing(
            [previousKeyStart, previousKeyEnd],
            [keyStart, keyEnd]);
    }}
    previousKeyStart = keyStart;
    previousKeyEnd = keyEnd;
    const value = {3};
    obj[key] =  value;
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
return [{}
];
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
const list: {} = [];
for (let i = 0; i < {}; i++) {{
    list.push({});
}}
return list;
"#,
                    self.quote_type(format0),
                    size,
                    self.quote_deserialize(content)
                )?;
            }

            _ => panic!("unexpected case"),
        }
        self.out.unindent();
        writeln!(self.out, "}}\n")
    }

    fn output_typedef_helper(&mut self, format0: &Format, imported: &mut Vec<String>) -> Result<()> {
        use Format::*;

        match format0 {
            Option(format) => {
                if Self::needs_import(format) && !imported.contains(&self.quote_type(format)){
                    writeln!(self.out, "import {{{0}}} from './{0}';", self.quote_type(format))?;
                    imported.push(self.quote_type(format));
                }
                writeln!(self.out, "export type Optional{0} = {0} | null;", self.quote_type(format))?;
            }

            Seq(format) => {
                if Self::needs_import(format) && !imported.contains(&self.quote_type(format)){
                    writeln!(self.out, "import {{{0}}} from './{0}';", self.quote_type(format))?;
                    imported.push(self.quote_type(format));
                }
                writeln!(self.out, "export type List{0} = {0}[];", self.quote_type(format))?;
            }

            Map { key, value } => {
                if Self::needs_import(key) && !imported.contains(&self.quote_type(key)){
                    writeln!(self.out, "import {{{0}}} from './{0}';", self.quote_type(key))?;
                    imported.push(self.quote_type(key));
                }
                if Self::needs_import(value) && !imported.contains(&self.quote_type(value)){
                    writeln!(self.out, "import {{{0}}} from './{0}';", self.quote_type(value))?;
                    imported.push(self.quote_type(value));
                }
                writeln!(self.out, "export type Map{0}{1} = Record<{0}, {1}>;", self.quote_type(key), self.quote_type(value))?;
            }

            Tuple(formats) => {
                for (_index, format) in formats.iter().enumerate() {
                    if Self::needs_import(format) && !imported.contains(&self.quote_type(format)){
                        writeln!(self.out, "import {{{0}}} from './{0}';", self.quote_type(format))?;
                        imported.push(self.quote_type(format));
                    }
                }

                write!(self.out, "export type Tuple")?;
                for (_index, format) in formats.iter().enumerate() {
                    write!(self.out, "{}", self.quote_type(format))?;
                }
                write!(self.out, " = [{}",
                       formats
                           .iter()
                           .map(|f| format!("{}", self.quote_type(f)))
                           .collect::<Vec<_>>()
                           .join(", "))?;
                writeln!(self.out, "];")?;
            }

            TupleArray { content, size: _size } => {
                // discover if import needed (key, value) might be primitives
                writeln!(self.out, "export type ListTuple{} = {}[];", self.quote_type(content), self.quote_type(content))?;
            }

            _ => panic!("unexpected case"),
        }
        Ok(())
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
                name: "value".to_string(),
                value: format.as_ref().clone(),
            }],
            Tuple(formats) => formats
                .iter()
                .enumerate()
                .map(|(i, f)| Named {
                    name: format!("field{}", i),
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
        // add imports to field types
        if self.generator.import_trait_helpers {
            let mut imported_types = Vec::new();
            for field in fields {
                let field_type = &field.value;
                if let Some(base) = variant_base {
                    if base.eq(&self.quote_type(&field.value)) { // no need to import myself
                        continue;
                    }
                }
                if imported_types.contains(&self.quote_type(field_type)) {
                    continue;
                }
                imported_types.push(self.quote_type(field_type));
                match &field.value {
                    Format::TypeName(_) => {
                        writeln!(self.out, "import {{{0}}} from '{1}/{0}';", self.quote_type(&field.value), self.quote_leading_path(&field.value))?;
                    }
                    Format::Tuple(_) => {
                        writeln!(self.out, "import {{{0}}} from './traitHelpers';", self.quote_type(&field.value))?;
                    }
                    Format::Option(_) => {
                        writeln!(self.out, "import {{{0}}} from './traitHelpers';", self.quote_type(&field.value))?;
                    }
                    Format::Seq(_) => {
                        writeln!(self.out, "import {{{0}}} from './traitHelpers';", self.quote_type(&field.value))?;
                    }
                    Format::TupleArray { content: _content, size: _size } => {
                        writeln!(self.out, "import {{{0}}} from './traitHelpers';", self.quote_type(&field.value))?;
                    }
                    Format::Map { key: _key, value: _value } => {
                        writeln!(self.out, "import {{{0}}} from './traitHelpers';", self.quote_type(&field.value))?;
                    }
                    _ => {}
                }
            }
        }
        let mut variant_base_name = format!("");

        // Beginning of class
        if let Some(base) = variant_base {
            writeln!(self.out)?;
            self.output_comment(name)?;
            writeln!(
                self.out,
                "export class {0}Variant{1} extends {0} {{",
                base, name
            )?;
            variant_base_name = format!("{0}Variant", base);
        } else {
            self.output_comment(name)?;
            writeln!(self.out, "export class {} {{", name)?;
        }
        let reserved_names = &["Builder"];
        self.enter_class(name, reserved_names);
        if !fields.is_empty() {
            writeln!(self.out)?;
        }
        // Constructor.
        writeln!(
            self.out,
            "constructor ({}) {{",
            fields
                .iter()
                .map(|f| format!("public {}: {}", &f.name, self.quote_type(&f.value)))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        if let Some(_base) = variant_base {
            self.out.indent();
            writeln!(self.out, "super();")?;
            self.out.unindent();
        }
        writeln!(self.out, "}}\n")?;
        // Serialize
        if self.generator.config.serialization {
            writeln!(
                self.out,
                "public serialize(serializer: Serializer): void {{",
            )?;
            self.out.indent();
            if let Some(index) = variant_index {
                writeln!(self.out, "serializer.serializeVariantIndex({});", index)?;
            }
            for field in fields {
                writeln!(
                    self.out,
                    "{}",
                    self.quote_serialize_value(&field.name, &field.value, true)
                )?;
            }
            self.out.unindent();
            writeln!(self.out, "}}\n")?;
        }
        // Deserialize (struct) or Load (variant)
        if self.generator.config.serialization {
            if variant_index.is_none() {
                writeln!(
                    self.out,
                    "static deserialize(deserializer: Deserializer): {} {{",
                    name,
                )?;
            } else {
                writeln!(
                    self.out,
                    "static load(deserializer: Deserializer): {}{} {{",
                    variant_base_name,
                    name,
                )?;
            }
            self.out.indent();
            for field in fields {
                writeln!(
                    self.out,
                    "const {} = {};",
                    field.name,
                    self.quote_deserialize(&field.value)
                )?;
            }
            writeln!(
                self.out,
                r#"return new {0}{1}({2});"#,
                variant_base_name,
                name,
                fields
                    .iter()
                    .map(|f| format!("{}", f.name))
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
            self.out.unindent();
            writeln!(self.out, "}}\n")?;
        }
        // End of class
        self.leave_class(reserved_names);
        writeln!(self.out, "}}")
    }

    fn output_enum_container(
        &mut self,
        name: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> Result<()> {
        self.output_comment(name)?;
        writeln!(self.out, "export abstract class {} {{", name)?;
        let reserved_names = variants
            .values()
            .map(|v| v.name.as_str())
            .collect::<Vec<_>>();
        self.enter_class(name, &reserved_names);
        if self.generator.config.serialization {
            writeln!(
                self.out,
                "abstract serialize(serializer: Serializer): void;\n"
            )?;
            write!(
                self.out,
                "static deserialize(deserializer: Deserializer): {} {{",
                name
            )?;
            self.out.indent();
            writeln!(
                self.out,
                r#"
const index = deserializer.deserializeVariantIndex();
switch (index) {{"#,
            )?;
            self.out.indent();
            for (index, variant) in variants {
                writeln!(
                    self.out,
                    "case {}: return {}Variant{}.load(deserializer);",
                    index, name, variant.name,
                )?;
            }
            writeln!(
                self.out,
                "default: throw new Error(\"Unknown variant index for {}: \" + index);",
                name,
            )?;
            self.out.unindent();
            writeln!(self.out, "}}")?;
            self.out.unindent();
            writeln!(self.out, "}}")?;
        }
        self.leave_class(&reserved_names);
        writeln!(self.out, "}}\n")?;
        self.output_variants(name, variants)?;
        Ok(())
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

/// Installer for generated source files in TypeScript.
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
            println!("writing file {}", entry.path().to_string_lossy());
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
        let generator = CodeGenerator::new(config, true);
        generator.write_source_files(self.install_dir.clone(), registry)?;
        Ok(())
    }

    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(
            include_directory!("runtime/ts/serde"),
            "serde",
        )
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(
            include_directory!("runtime/ts/bincode"),
            "bincode",
        )
    }

    fn install_lcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(
            include_directory!("runtime/ts/lcs"),
            "lcs",
        )
    }
}
