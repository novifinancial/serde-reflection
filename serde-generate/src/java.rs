// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use include_dir::include_dir;
use serde_reflection::{ContainerFormat, Format, FormatHolder, Named, Registry, VariantFormat};
use std::collections::BTreeMap;
use std::io::{Result, Write};
use std::path::PathBuf;

pub fn output(out: &mut dyn Write, registry: &Registry, class_name: &str) -> Result<()> {
    output_preambule(out, None)?;

    writeln!(out, "public class {} {{\n", class_name)?;
    for (name, format) in registry {
        output_container(
            out,
            name,
            format,
            /* nested class */ true,
            &format!("{}.", class_name),
        )?;
    }
    output_trait_helpers(out, registry, /* nested class */ true)?;
    writeln!(out, "}}")
}

fn output_preambule(out: &mut dyn Write, package_name: Option<&str>) -> Result<()> {
    if let Some(name) = package_name {
        writeln!(out, "package {};", name,)?;
    }
    writeln!(
        out,
        r#"
import java.io.IOException;
import java.math.BigInteger;
import java.util.Optional;
import java.util.ArrayList;
import java.util.Map;
import java.util.HashMap;
import serde.ArrayLen;
import serde.Deserializer;
import serde.Int128;
import serde.Unsigned;
import serde.Serializer;
import serde.Tuple2;
import serde.Tuple3;
import serde.Tuple4;
import serde.Tuple5;
import serde.Tuple6;
"#
    )
}

/// A non-empty `package_prefix` is required when the type is quoted from a nested struct.
fn quote_type(format: &Format, package_prefix: &str) -> String {
    use Format::*;
    match format {
        TypeName(x) => format!("{}{}", package_prefix, x),
        Unit => "Void".into(),
        Bool => "Boolean".into(),
        I8 => "Byte".into(),
        I16 => "Short".into(),
        I32 => "Integer".into(),
        I64 => "Long".into(),
        I128 => "@Int128 BigInteger".into(),
        U8 => "@Unsigned Byte".into(),
        U16 => "@Unsigned Short".into(),
        U32 => "@Unsigned Integer".into(),
        U64 => "@Unsigned Long".into(),
        U128 => "@Unsigned @Int128 BigInteger".into(),
        F32 => "Float".into(),
        F64 => "Double".into(),
        Char => "Character".into(),
        Str => "String".into(),
        Bytes => "byte[]".into(),

        Option(format) => format!("Optional<{}>", quote_type(format, package_prefix)),
        Seq(format) => format!("ArrayList<{}>", quote_type(format, package_prefix)),
        Map { key, value } => format!(
            "Map<{}, {}>",
            quote_type(key, package_prefix),
            quote_type(value, package_prefix)
        ),
        Tuple(formats) => format!(
            "Tuple{}<{}>",
            formats.len(),
            quote_types(formats, package_prefix)
        ),
        TupleArray { content, size } => format!(
            "{} @ArrayLen(length={}) []",
            quote_type(content, package_prefix),
            size
        ),
        Variable(_) => panic!("unexpected value"),
    }
}

fn quote_types(formats: &[Format], package_prefix: &str) -> String {
    formats
        .iter()
        .map(|f| quote_type(f, package_prefix))
        .collect::<Vec<_>>()
        .join(", ")
}

fn output_trait_helpers(
    out: &mut dyn Write,
    registry: &Registry,
    nested_class: bool,
) -> Result<()> {
    let mut subtypes = BTreeMap::new();
    for format in registry.values() {
        format
            .visit(&mut |f| {
                if needs_helper(f) {
                    subtypes.insert(mangle_type(f), f.clone());
                }
                Ok(())
            })
            .unwrap();
    }
    let prefix = if nested_class { "static " } else { "" };
    writeln!(out, "{}class TraitHelpers {{", prefix)?;
    for (mangled_name, subtype) in &subtypes {
        output_serialization_helper(out, mangled_name, subtype)?;
        output_deserialization_helper(out, mangled_name, subtype)?;
    }
    writeln!(out, "}}\n")
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

        Option(format) => format!("option_{}", mangle_type(format)),
        Seq(format) => format!("vector_{}", mangle_type(format)),
        Map { key, value } => format!("map_{}_to_{}", mangle_type(key), mangle_type(value)),
        Tuple(formats) => format!(
            "tuple{}_{}",
            formats.len(),
            formats
                .iter()
                .map(mangle_type)
                .collect::<Vec<_>>()
                .join("_")
        ),
        TupleArray { content, size } => format!("array{}_{}_array", size, mangle_type(content)),
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
            mangle_type(format),
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
            mangle_type(format),
        ),
    }
}

fn output_serialization_helper(out: &mut dyn Write, name: &str, format0: &Format) -> Result<()> {
    use Format::*;

    write!(
        out,
        "    static void serialize_{}({} value, Serializer serializer) throws IOException {{",
        name,
        quote_type(format0, "")
    )?;
    match format0 {
        Option(format) => {
            write!(
                out,
                r#"
        if (value.isPresent()) {{
            serializer.serialize_option_tag(true);
            {}
        }} else {{
            serializer.serialize_option_tag(false);
        }}
"#,
                quote_serialize_value("value.get()", format)
            )?;
        }

        Seq(format) => {
            write!(
                out,
                r#"
        serializer.serialize_len(value.size());
        for ({} item : value) {{
            {}
        }}
"#,
                quote_type(format, ""),
                quote_serialize_value("item", format)
            )?;
        }

        Map { key, value } => {
            write!(
                out,
                r#"
        serializer.serialize_len(value.size());
        for (Map.Entry<{}, {}> entry : value.entrySet()) {{
            {}
            {}
        }}
"#,
                quote_type(key, ""),
                quote_type(value, ""),
                quote_serialize_value("entry.getKey()", key),
                quote_serialize_value("entry.getValue()", value)
            )?;
        }

        Tuple(formats) => {
            writeln!(out)?;
            for (index, format) in formats.iter().enumerate() {
                let expr = format!("value.field{}", index);
                writeln!(out, "        {}", quote_serialize_value(&expr, format))?;
            }
        }

        TupleArray { content, size } => {
            write!(
                out,
                r#"
        assert value.length == {};
        for ({} item : value) {{
            {}
        }}
"#,
                size,
                quote_type(content, ""),
                quote_serialize_value("item", content),
            )?;
        }

        _ => panic!("unexpected case"),
    }
    writeln!(out, "    }}\n")
}

fn output_deserialization_helper(out: &mut dyn Write, name: &str, format0: &Format) -> Result<()> {
    use Format::*;

    write!(
        out,
        "    static {} deserialize_{}(Deserializer deserializer) throws IOException {{",
        quote_type(format0, ""),
        name,
    )?;
    match format0 {
        Option(format) => {
            write!(
                out,
                r#"
        boolean tag = deserializer.deserialize_option_tag();
        if (tag) {{
            return Optional.empty();
        }} else {{
            return Optional.of({});
        }}
"#,
                quote_deserialize(format, "")
            )?;
        }

        Seq(format) => {
            write!(
                out,
                r#"
        long length = deserializer.deserialize_len();
        ArrayList<{}> obj = new ArrayList<{}>((int) length);
        for (long i = 0; i < length; i++) {{
            obj.add({});
        }}
        return obj;
"#,
                quote_type(format, ""),
                quote_type(format, ""),
                quote_deserialize(format, "")
            )?;
        }

        Map { key, value } => {
            let key_type = quote_type(key, "");
            let value_type = quote_type(value, "");
            write!(
                out,
                r#"
        long length = deserializer.deserialize_len();
        Map<{}, {}> obj = new HashMap<{}, {}>();
        for (long i = 0; i < length; i++) {{
            {} key = {};
            {} value = {};
            obj.put(key, value);
        }}
        return obj;
"#,
                key_type,
                value_type,
                key_type,
                value_type,
                key_type,
                quote_deserialize(key, ""),
                value_type,
                quote_deserialize(value, ""),
            )?;
        }

        Tuple(formats) => {
            writeln!(
                out,
                "\n        {} obj = new {}();",
                quote_type(format0, ""),
                quote_type(format0, "")
            )?;
            for (index, format) in formats.iter().enumerate() {
                writeln!(
                    out,
                    "        obj.field{} = {};",
                    index,
                    quote_deserialize(format, "")
                )?;
            }
            writeln!(out, "        return obj;")?;
        }

        TupleArray { content, size } => {
            write!(
                out,
                r#"
        {}[] obj = new {}[{}];
        for (int i = 0; i < {}; i++) {{
            obj[i] = {};
        }}
        return obj;
"#,
                quote_type(content, ""),
                quote_type(content, ""),
                size,
                size,
                quote_deserialize(content, "")
            )?;
        }

        _ => panic!("unexpected case"),
    }
    writeln!(out, "    }}\n")
}

fn output_variant(
    out: &mut dyn Write,
    base: &str,
    index: u32,
    name: &str,
    variant: &VariantFormat,
    package_prefix: &str,
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
    output_struct_or_variant_container(
        out,
        4,
        Some(base),
        Some(index),
        name,
        &fields,
        false,
        package_prefix,
    )
}

fn output_variants(
    out: &mut dyn Write,
    base: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    package_prefix: &str,
) -> Result<()> {
    for (index, variant) in variants {
        output_variant(
            out,
            base,
            *index,
            &variant.name,
            &variant.value,
            package_prefix,
        )?;
    }
    Ok(())
}

fn output_struct_or_variant_container(
    out: &mut dyn Write,
    indentation: usize,
    variant_base: Option<&str>,
    variant_index: Option<u32>,
    name: &str,
    fields: &[Named<Format>],
    nested_class: bool,
    package_prefix: &str,
) -> Result<()> {
    let tab = " ".repeat(indentation);
    // Beginning of class
    if let Some(base) = variant_base {
        writeln!(
            out,
            "\n{}public static class {} extends {} {{",
            tab, name, base
        )?;
    } else {
        let prefix = if nested_class {
            "public static "
        } else {
            "public "
        };
        writeln!(out, "{}{}class {} {{", tab, prefix, name)?;
    }
    // Fields
    for field in fields {
        writeln!(
            out,
            "{}    public {} {};",
            tab,
            quote_type(&field.value, package_prefix),
            field.name
        )?;
    }
    // Nullary constructor.
    writeln!(out, "\n{}    public {}() {{}}", tab, name)?;
    // N-ary constructor if N > 0.
    if !fields.is_empty() {
        writeln!(
            out,
            "\n{}    public {}({}) {{",
            tab,
            name,
            fields
                .iter()
                .map(|f| format!("{} {}", quote_type(&f.value, package_prefix), &f.name))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        for field in fields {
            writeln!(out, "{}       this.{} = {};", tab, &field.name, &field.name,)?;
        }
        writeln!(out, "{}    }}", tab)?;
    }
    // Serialize
    writeln!(
        out,
        "\n{}    public void serialize(Serializer serializer) throws IOException {{",
        tab,
    )?;
    if let Some(index) = variant_index {
        writeln!(
            out,
            "{}        serializer.serialize_variant_index({});",
            tab, index
        )?;
    }
    for field in fields {
        writeln!(
            out,
            "{}        {}",
            tab,
            quote_serialize_value(&field.name, &field.value)
        )?;
    }
    writeln!(out, "{}    }}\n", tab)?;
    // Deserialize (struct) or Load (variant)
    if variant_index.is_none() {
        writeln!(
            out,
            "{}    public static {} deserialize(Deserializer deserializer) throws IOException {{",
            tab, name,
        )?;
        writeln!(out, "{}        {} obj = new {}();", tab, name, name)?;
    } else {
        writeln!(
            out,
            "{}    void load(Deserializer deserializer) throws IOException {{",
            tab,
        )?;
    }
    for field in fields {
        writeln!(
            out,
            "{}        {}.{} = {};",
            tab,
            if variant_index.is_none() {
                "obj"
            } else {
                "this"
            },
            field.name,
            quote_deserialize(&field.value, package_prefix)
        )?;
    }
    if variant_index.is_none() {
        writeln!(out, "{}        return obj;", tab)?;
    }
    writeln!(out, "{}    }}\n", tab)?;
    // Equality
    writeln!(
        out,
        r#"{}    public boolean equals(Object obj) {{
{}        if (this == obj) return true;
{}        if (obj == null) return false;
{}        if (getClass() != obj.getClass()) return false;
{}        {} other = ({}) obj;"#,
        tab, tab, tab, tab, tab, name, name,
    )?;
    for field in fields {
        writeln!(
            out,
            "{}        if (!this.{}.equals(other.{})) {{ return false; }}",
            tab, &field.name, &field.name,
        )?;
    }
    writeln!(out, "{}        return true;", tab)?;
    writeln!(out, "{}    }}\n", tab)?;
    // Hashing
    writeln!(
        out,
        "{}    public int hashCode() {{\n{}        int value = 7;",
        tab, tab
    )?;
    for field in fields {
        writeln!(
            out,
            "{}        value = 31 * value + this.{}.hashCode();",
            tab, &field.name,
        )?;
    }
    writeln!(out, "{}        return value;", tab)?;
    writeln!(out, "{}    }}", tab)?;

    // End of class
    writeln!(out, "{}}}\n", tab)
}

fn output_enum_container(
    out: &mut dyn Write,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    nested_class: bool,
    package_prefix: &str,
) -> Result<()> {
    let prefix = if nested_class {
        "public static "
    } else {
        "public "
    };
    writeln!(out, "{}abstract class {} {{", prefix, name)?;
    writeln!(
        out,
        "    abstract public void serialize(Serializer serializer) throws IOException;",
    )?;
    writeln!(
        out,
        "    abstract void load(Deserializer deserializer) throws IOException;",
    )?;
    write!(
        out,
        r#"
    public static {} deserialize(Deserializer deserializer) throws IOException {{
        {} obj;
        int index = deserializer.deserialize_variant_index();
        switch (index) {{
"#,
        name, name,
    )?;
    for (index, variant) in variants {
        writeln!(
            out,
            "            case {}: obj = new {}(); break;",
            index, variant.name,
        )?;
    }
    writeln!(
        out,
        "            default: throw new IOException(\"Unknown variant index for {}: \" + index);",
        name,
    )?;
    writeln!(out, "        }}",)?;
    writeln!(
        out,
        "        obj.load(deserializer);\n        return obj;\n    }}",
    )?;
    output_variants(out, name, variants, package_prefix)?;
    writeln!(out, "}}\n")
}

fn output_container(
    out: &mut dyn Write,
    name: &str,
    format: &ContainerFormat,
    nested_class: bool,
    package_prefix: &str,
) -> Result<()> {
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
            return output_enum_container(out, name, variants, nested_class, package_prefix);
        }
    };
    output_struct_or_variant_container(
        out,
        0,
        None,
        None,
        name,
        &fields,
        nested_class,
        package_prefix,
    )
}

pub struct Installer {
    install_dir: PathBuf,
}

impl Installer {
    pub fn new(install_dir: PathBuf) -> Self {
        Installer { install_dir }
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
            let mut file = std::fs::File::create(dir_path.join(name.to_string() + ".java"))?;
            output_preambule(&mut file, Some(package_name))?;
            output_container(
                &mut file,
                name,
                format,
                /* nested class */ false,
                &format!("{}.", package_name),
            )?;
        }
        let mut file = std::fs::File::create(dir_path.join("TraitHelpers.java"))?;
        output_preambule(&mut file, Some(package_name))?;
        output_trait_helpers(&mut file, registry, /* nested class */ false)?;
        Ok(())
    }

    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error> {
        let source_dir = include_dir!("runtime/java/serde");
        let dir_path = self.install_dir.join("serde");
        std::fs::create_dir_all(&dir_path)?;
        for entry in source_dir.files() {
            let mut file = std::fs::File::create(dir_path.join(entry.path()))?;
            file.write_all(entry.contents())?;
        }
        Ok(())
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        panic!("not implemented")
    }

    fn install_lcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        panic!("not implemented")
    }
}
