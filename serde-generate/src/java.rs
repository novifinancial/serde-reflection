// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::indent::{IndentConfig, IndentedWriter};
use include_dir::include_dir as include_directory;
use serde_reflection::{ContainerFormat, Format, FormatHolder, Named, Registry, VariantFormat};
use std::{
    collections::BTreeMap,
    io::{Result, Write},
    path::PathBuf,
};

pub fn output(out: &mut dyn Write, registry: &Registry, class_name: &str) -> Result<()> {
    let mut emitter = JavaEmitter {
        out: IndentedWriter::new(out, IndentConfig::Space(4)),
        package_name: None,
        nested_class: true,
        package_prefix: format!("{}.", class_name),
    };

    emitter.output_preambule()?;
    writeln!(emitter.out, "public final class {} {{\n", class_name)?;
    emitter.out.inc_level();
    for (name, format) in registry {
        emitter.output_container(name, format)?;
        writeln!(emitter.out)?;
    }
    emitter.output_trait_helpers(registry)?;
    emitter.out.dec_level();
    writeln!(emitter.out, "}}")
}

fn write_container_class(
    install_dir: &std::path::Path,
    package_name: &str,
    name: &str,
    format: &ContainerFormat,
) -> Result<()> {
    let mut file = std::fs::File::create(install_dir.join(name.to_string() + ".java"))?;
    let mut emitter = JavaEmitter {
        out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
        package_name: Some(package_name),
        nested_class: false,
        package_prefix: format!("{}.", package_name),
    };

    emitter.output_preambule()?;
    emitter.output_container(name, format)
}

fn write_helper_class(
    install_dir: &std::path::Path,
    package_name: &str,
    registry: &Registry,
) -> Result<()> {
    let mut file = std::fs::File::create(install_dir.join("TraitHelpers.java"))?;
    let mut emitter = JavaEmitter {
        out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
        package_name: Some(package_name),
        nested_class: false,
        package_prefix: format!("{}.", package_name),
    };

    emitter.output_preambule()?;
    emitter.output_trait_helpers(registry)
}

struct JavaEmitter<'a, T> {
    out: IndentedWriter<T>,
    package_name: Option<&'a str>,
    nested_class: bool,
    package_prefix: String,
}

impl<'a, T> JavaEmitter<'a, T>
where
    T: Write,
{
    fn output_preambule(&mut self) -> Result<()> {
        if let Some(name) = self.package_name {
            writeln!(self.out, "package {};", name)?;
        }
        // Java doesn't let us annotate fully-qualified class names.
        writeln!(
            self.out,
            r#"
import java.math.BigInteger;
"#
        )?;
        Ok(())
    }

    /// A non-empty `package_prefix` is required when the type is quoted from a nested struct.
    fn quote_type(format: &Format, package_prefix: &str) -> String {
        use Format::*;
        match format {
            TypeName(x) => format!("{}{}", package_prefix, x),
            Unit => "com.facebook.serde.Unit".into(),
            Bool => "Boolean".into(),
            I8 => "Byte".into(),
            I16 => "Short".into(),
            I32 => "Integer".into(),
            I64 => "Long".into(),
            I128 => "@com.facebook.serde.Int128 BigInteger".into(),
            U8 => "@com.facebook.serde.Unsigned Byte".into(),
            U16 => "@com.facebook.serde.Unsigned Short".into(),
            U32 => "@com.facebook.serde.Unsigned Integer".into(),
            U64 => "@com.facebook.serde.Unsigned Long".into(),
            U128 => "@com.facebook.serde.Unsigned @com.facebook.serde.Int128 BigInteger".into(),
            F32 => "Float".into(),
            F64 => "Double".into(),
            Char => "Character".into(),
            Str => "String".into(),
            Bytes => "com.facebook.serde.Bytes".into(),

            Option(format) => format!(
                "java.util.Optional<{}>",
                Self::quote_type(format, package_prefix)
            ),
            Seq(format) => format!(
                "java.util.List<{}>",
                Self::quote_type(format, package_prefix)
            ),
            Map { key, value } => format!(
                "java.util.Map<{}, {}>",
                Self::quote_type(key, package_prefix),
                Self::quote_type(value, package_prefix)
            ),
            Tuple(formats) => format!(
                "com.facebook.serde.Tuple{}<{}>",
                formats.len(),
                Self::quote_types(formats, package_prefix)
            ),
            TupleArray { content, size } => format!(
                "{} @com.facebook.serde.ArrayLen(length={}) []",
                Self::quote_type(content, package_prefix),
                size
            ),
            Variable(_) => panic!("unexpected value"),
        }
    }

    fn quote_types(formats: &[Format], package_prefix: &str) -> String {
        formats
            .iter()
            .map(|f| Self::quote_type(f, package_prefix))
            .collect::<Vec<_>>()
            .join(", ")
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
        let prefix = if self.nested_class { "static " } else { "" };
        writeln!(self.out, "{}final class TraitHelpers {{", prefix)?;
        self.out.inc_level();
        for (mangled_name, subtype) in &subtypes {
            self.output_serialization_helper(mangled_name, subtype)?;
            self.output_deserialization_helper(mangled_name, subtype)?;
        }
        self.out.dec_level();
        writeln!(self.out, "}}\n")
    }

    fn mangle_type(format: &Format) -> String {
        use Format::*;
        match format {
            TypeName(x) => x.to_string(),
            Unit => "unit".into(),
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
            Str => "str".into(),
            Bytes => "bytes".into(),

            Option(format) => format!("option_{}", Self::mangle_type(format)),
            Seq(format) => format!("vector_{}", Self::mangle_type(format)),
            Map { key, value } => format!(
                "map_{}_to_{}",
                Self::mangle_type(key),
                Self::mangle_type(value)
            ),
            Tuple(formats) => format!(
                "tuple{}_{}",
                formats.len(),
                formats
                    .iter()
                    .map(Self::mangle_type)
                    .collect::<Vec<_>>()
                    .join("_")
            ),
            TupleArray { content, size } => {
                format!("array{}_{}_array", size, Self::mangle_type(content))
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

    fn quote_serialize_value(value: &str, format: &Format) -> String {
        use Format::*;
        match format {
            TypeName(_) => format!("{}.serialize(serializer);", value),
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
                "TraitHelpers.serialize_{}({}, serializer);",
                Self::mangle_type(format),
                value
            ),
        }
    }

    fn quote_deserialize(format: &Format, package_prefix: &str) -> String {
        use Format::*;
        match format {
            TypeName(name) => format!("{}{}.deserialize(deserializer)", package_prefix, name),
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
                "TraitHelpers.deserialize_{}(deserializer)",
                Self::mangle_type(format),
            ),
        }
    }

    fn output_serialization_helper(&mut self, name: &str, format0: &Format) -> Result<()> {
        use Format::*;

        write!(
        self.out,
        "static void serialize_{}({} value, com.facebook.serde.Serializer serializer) throws java.lang.Exception {{",
        name,
        Self::quote_type(format0, "")
    )?;
        self.out.inc_level();
        match format0 {
            Option(format) => {
                write!(
                    self.out,
                    r#"
if (value.isPresent()) {{
    serializer.serialize_option_tag(true);
    {}
}} else {{
    serializer.serialize_option_tag(false);
}}
"#,
                    Self::quote_serialize_value("value.get()", format)
                )?;
            }

            Seq(format) => {
                write!(
                    self.out,
                    r#"
serializer.serialize_len(value.size());
for ({} item : value) {{
    {}
}}
"#,
                    Self::quote_type(format, ""),
                    Self::quote_serialize_value("item", format)
                )?;
            }

            Map { key, value } => {
                write!(
                    self.out,
                    r#"
serializer.serialize_len(value.size());
int[] offsets = new int[value.size()];
int count = 0;
for (java.util.Map.Entry<{}, {}> entry : value.entrySet()) {{
    offsets[count++] = serializer.get_buffer_offset();
    {}
    {}
}}
serializer.sort_map_entries(offsets);
"#,
                    Self::quote_type(key, ""),
                    Self::quote_type(value, ""),
                    Self::quote_serialize_value("entry.getKey()", key),
                    Self::quote_serialize_value("entry.getValue()", value)
                )?;
            }

            Tuple(formats) => {
                writeln!(self.out)?;
                for (index, format) in formats.iter().enumerate() {
                    let expr = format!("value.field{}", index);
                    writeln!(self.out, "{}", Self::quote_serialize_value(&expr, format))?;
                }
            }

            TupleArray { content, size } => {
                write!(
                    self.out,
                    r#"
assert value.length == {};
for ({} item : value) {{
    {}
}}
"#,
                    size,
                    Self::quote_type(content, ""),
                    Self::quote_serialize_value("item", content),
                )?;
            }

            _ => panic!("unexpected case"),
        }
        self.out.dec_level();
        writeln!(self.out, "}}\n")
    }

    fn output_deserialization_helper(&mut self, name: &str, format0: &Format) -> Result<()> {
        use Format::*;

        write!(
        self.out,
        "static {} deserialize_{}(com.facebook.serde.Deserializer deserializer) throws java.lang.Exception {{",
        Self::quote_type(format0, ""),
        name,
    )?;
        self.out.inc_level();
        match format0 {
            Option(format) => {
                write!(
                    self.out,
                    r#"
boolean tag = deserializer.deserialize_option_tag();
if (!tag) {{
    return java.util.Optional.empty();
}} else {{
    return java.util.Optional.of({});
}}
"#,
                    Self::quote_deserialize(format, "")
                )?;
            }

            Seq(format) => {
                write!(
                    self.out,
                    r#"
long length = deserializer.deserialize_len();
java.util.List<{0}> obj = new java.util.ArrayList<{0}>((int) length);
for (long i = 0; i < length; i++) {{
    obj.add({1});
}}
return obj;
"#,
                    Self::quote_type(format, ""),
                    Self::quote_deserialize(format, "")
                )?;
            }

            Map { key, value } => {
                write!(
                    self.out,
                    r#"
long length = deserializer.deserialize_len();
java.util.Map<{0}, {1}> obj = new java.util.HashMap<{0}, {1}>();
int previous_key_start = 0;
int previous_key_end = 0;
for (long i = 0; i < length; i++) {{
    int key_start = deserializer.get_buffer_offset();
    {0} key = {2};
    int key_end = deserializer.get_buffer_offset();
    if (i > 0) {{
        deserializer.check_that_key_slices_are_increasing(
            new com.facebook.serde.Slice(previous_key_start, previous_key_end),
            new com.facebook.serde.Slice(key_start, key_end));
    }}
    previous_key_start = key_start;
    previous_key_end = key_end;
    {1} value = {3};
    obj.put(key, value);
}}
return obj;
"#,
                    Self::quote_type(key, ""),
                    Self::quote_type(value, ""),
                    Self::quote_deserialize(key, ""),
                    Self::quote_deserialize(value, ""),
                )?;
            }

            Tuple(formats) => {
                write!(
                    self.out,
                    r#"
return new {}({}
);
"#,
                    Self::quote_type(format0, ""),
                    formats
                        .iter()
                        .map(|f| format!("\n    {}", Self::quote_deserialize(f, "")))
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
                    Self::quote_type(content, ""),
                    size,
                    Self::quote_deserialize(content, "")
                )?;
            }

            _ => panic!("unexpected case"),
        }
        self.out.dec_level();
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
        // Beginning of class
        if let Some(base) = variant_base {
            writeln!(self.out)?;
            writeln!(
                self.out,
                "public static final class {} extends {} {{",
                name, base
            )?;
        } else {
            let prefix = if self.nested_class {
                "public static "
            } else {
                "public "
            };
            writeln!(self.out, "{}final class {} {{", prefix, name)?;
        }
        self.out.inc_level();
        // Fields
        for field in fields {
            writeln!(
                self.out,
                "public final {} {};",
                Self::quote_type(&field.value, &self.package_prefix),
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
                .map(|f| format!(
                    "{} {}",
                    Self::quote_type(&f.value, &self.package_prefix),
                    &f.name
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        self.out.inc_level();
        for field in fields {
            writeln!(self.out, "assert {} != null;", &field.name)?;
        }
        for field in fields {
            writeln!(self.out, "this.{} = {};", &field.name, &field.name)?;
        }
        self.out.dec_level();
        writeln!(self.out, "}}")?;
        // Serialize
        writeln!(
        self.out,
        "\npublic void serialize(com.facebook.serde.Serializer serializer) throws java.lang.Exception {{",
    )?;
        self.out.inc_level();
        if let Some(index) = variant_index {
            writeln!(self.out, "serializer.serialize_variant_index({});", index)?;
        }
        for field in fields {
            writeln!(
                self.out,
                "{}",
                Self::quote_serialize_value(&field.name, &field.value)
            )?;
        }
        self.out.dec_level();
        writeln!(self.out, "}}\n")?;
        // Deserialize (struct) or Load (variant)
        if variant_index.is_none() {
            writeln!(
            self.out,
            "public static {} deserialize(com.facebook.serde.Deserializer deserializer) throws java.lang.Exception {{",
            name,
        )?;
        } else {
            writeln!(
            self.out,
            "static {} load(com.facebook.serde.Deserializer deserializer) throws java.lang.Exception {{",
            name,
        )?;
        }
        self.out.inc_level();
        writeln!(self.out, "Builder builder = new Builder();")?;
        for field in fields {
            writeln!(
                self.out,
                "builder.{} = {};",
                field.name,
                Self::quote_deserialize(&field.value, &self.package_prefix)
            )?;
        }
        writeln!(self.out, "return builder.build();")?;
        self.out.dec_level();
        writeln!(self.out, "}}\n")?;
        // Equality
        write!(self.out, "public boolean equals(Object obj) {{")?;
        self.out.inc_level();
        writeln!(
            self.out,
            r#"
if (this == obj) return true;
if (obj == null) return false;
if (getClass() != obj.getClass()) return false;
{0} other = ({0}) obj;"#,
            name,
        )?;
        for field in fields {
            writeln!(
                self.out,
                "if (!java.util.Objects.equals(this.{0}, other.{0})) {{ return false; }}",
                &field.name,
            )?;
        }
        writeln!(self.out, "return true;")?;
        self.out.dec_level();
        writeln!(self.out, "}}\n")?;
        // Hashing
        writeln!(self.out, "public int hashCode() {{")?;
        self.out.inc_level();
        writeln!(self.out, "int value = 7;",)?;
        for field in fields {
            writeln!(
                self.out,
                "value = 31 * value + (this.{0} != null ? this.{0}.hashCode() : 0);",
                &field.name
            )?;
        }
        writeln!(self.out, "return value;")?;
        self.out.dec_level();
        writeln!(self.out, "}}")?;
        // Builder
        self.output_struct_or_variant_container_builder(name, fields)?;
        // End of class
        self.out.dec_level();
        writeln!(self.out, "}}")
    }

    fn output_struct_or_variant_container_builder(
        &mut self,
        name: &str,
        fields: &[Named<Format>],
    ) -> Result<()> {
        // Beginning of builder class
        writeln!(self.out)?;
        writeln!(self.out, "public static final class Builder {{")?;
        self.out.inc_level();
        // Fields
        for field in fields {
            writeln!(
                self.out,
                "public {} {};",
                Self::quote_type(&field.value, &self.package_prefix),
                field.name
            )?;
        }
        if !fields.is_empty() {
            writeln!(self.out)?;
        }
        // Finalization
        writeln!(
            self.out,
            r#"public {0} build() {{
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
        // End of class
        self.out.dec_level();
        writeln!(self.out, "}}")
    }

    fn output_enum_container(
        &mut self,
        name: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> Result<()> {
        let prefix = if self.nested_class {
            "public static "
        } else {
            "public "
        };
        writeln!(self.out, "{}abstract class {} {{", prefix, name)?;
        self.out.inc_level();
        writeln!(
        self.out,
        "abstract public void serialize(com.facebook.serde.Serializer serializer) throws java.lang.Exception;\n",
    )?;
        write!(
        self.out,
        "public static {} deserialize(com.facebook.serde.Deserializer deserializer) throws java.lang.Exception {{", name)?;
        self.out.inc_level();
        writeln!(
            self.out,
            r#"
{} obj;
int index = deserializer.deserialize_variant_index();
switch (index) {{"#,
            name,
        )?;
        self.out.inc_level();
        for (index, variant) in variants {
            writeln!(
                self.out,
                "case {}: return {}.load(deserializer);",
                index, variant.name,
            )?;
        }
        writeln!(
            self.out,
            "default: throw new java.lang.Exception(\"Unknown variant index for {}: \" + index);",
            name,
        )?;
        self.out.dec_level();
        writeln!(self.out, "}}")?;
        self.out.dec_level();
        writeln!(self.out, "}}")?;
        self.output_variants(name, variants)?;
        self.out.dec_level();
        writeln!(self.out, "}}\n")
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
        package_name: &str,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error> {
        let parts = package_name.split('.').collect::<Vec<_>>();
        let mut dir_path = self.install_dir.clone();
        for part in &parts {
            dir_path = dir_path.join(part);
        }
        std::fs::create_dir_all(&dir_path)?;

        for (name, format) in registry {
            write_container_class(&dir_path, package_name, name, format)?;
        }
        write_helper_class(&dir_path, package_name, registry)?;
        Ok(())
    }

    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(
            include_directory!("runtime/java/com/facebook/serde"),
            "com/facebook/serde",
        )
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(
            include_directory!("runtime/java/com/facebook/bincode"),
            "com/facebook/bincode",
        )
    }

    fn install_lcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(
            include_directory!("runtime/java/com/facebook/lcs"),
            "com/facebook/lcs",
        )
    }
}
