// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_reflection::{ContainerFormat, Format, Named, RegistryOwned, VariantFormat};
use std::collections::BTreeMap;
use std::io::{Result, Write};

pub fn output(out: &mut dyn Write, registry: &RegistryOwned) -> Result<()> {
    output_preambule(out)?;

    writeln!(out, "public class Test {{")?;
    writeln!(
        out,
        r#"
public static class Unit {{}};
public static class Tuple2<T1, T2> {{ public T1 field1; public T2 field2; }};
public static class Tuple3<T1, T2, T3> {{ public T1 field1; public T2 field2; public T3 field3; }};
public static class Tuple4<T1, T2, T3, T4> {{ public T1 field1; public T2 field2; public T3 field3; public T4 field4; }};
public static class Integer128 {{ public Long high; public Long low; }};
"#
    )?;
    for (name, format) in registry {
        output_container(out, name, format)?;
    }
    writeln!(out, "}}")
}

fn output_preambule(out: &mut dyn Write) -> Result<()> {
    writeln!(
        out,
        r#"
import java.nio.ByteBuffer;
import java.util.Optional;
import java.util.Vector;
import java.util.SortedMap;
import java.lang.Class;
import java.lang.annotation.ElementType;
import java.lang.annotation.Target;

@Target({{ElementType.TYPE_USE}})
@interface Unsigned {{}}

@Target({{ElementType.TYPE_USE}})
@interface FixedLength {{
    int length();
}}

@Target({{ElementType.TYPE_USE}})
@interface Enum {{
    Class<?>[] variants();
}}

@Target({{ElementType.TYPE_USE}})
@interface Variant {{
    long index();
}}
"#
    )
}

fn quote_type(format: &Format) -> String {
    use Format::*;
    match format {
        TypeName(x) => x.to_string(),
        Unit => "Unit".into(),
        Bool => "Boolean".into(),
        I8 => "Byte".into(),
        I16 => "Short".into(),
        I32 => "Integer".into(),
        I64 => "Long".into(),
        I128 => "Integer128".into(),
        U8 => "@Unsigned Byte".into(),
        U16 => "@Unsigned Short".into(),
        U32 => "@Unsigned Integer".into(),
        U64 => "@Unsigned Long".into(),
        U128 => "@Unsigned Integer128".into(),
        F32 => "Float".into(),
        F64 => "Double".into(),
        Char => "Char".into(),
        Str => "String".into(),
        Bytes => "ByteBuffer".into(),

        Option(format) => format!("Optional<{}>", quote_type(format)),
        Seq(format) => format!("Vector<{}>", quote_type(format)),
        Map { key, value } => format!("SortedMap<{}, {}>", quote_type(key), quote_type(value)),
        Tuple(formats) => format!("Tuple{}<{}>", formats.len(), quote_types(formats)),
        TupleArray { content, size } => format!(
            "@FixedLength(length={}) Vector<{}>",
            size,
            quote_type(content)
        ),
        Unknown => panic!("unexpected value"),
    }
}

fn quote_types(formats: &[Format]) -> String {
    formats
        .iter()
        .map(quote_type)
        .collect::<Vec<_>>()
        .join(", ")
}

fn output_fields(out: &mut dyn Write, indentation: usize, fields: &[Named<Format>]) -> Result<()> {
    let tab = " ".repeat(indentation);
    for field in fields {
        writeln!(
            out,
            "{} public {} {};",
            tab,
            quote_type(&field.value),
            field.name
        )?;
    }
    Ok(())
}

fn output_variant(
    out: &mut dyn Write,
    base: &str,
    name: &str,
    index: u32,
    variant: &VariantFormat,
) -> Result<()> {
    use VariantFormat::*;
    let annotation = format!("@Variant(index = {})\n", index);
    let class = format!("public static class {}_{} extends {}", base, name, base);
    match variant {
        Unit => writeln!(out, "\n{}{} {{}};", annotation, class),
        NewType(format) => writeln!(
            out,
            "\n{}{} {{\n    {} value;\n}};",
            annotation,
            class,
            quote_type(format),
        ),
        Tuple(formats) => writeln!(
            out,
            "\n{}{} {{\n    {} value;\n}};",
            annotation,
            class,
            quote_type(&Format::Tuple(formats.clone())),
        ),
        Struct(fields) => {
            writeln!(out, "\n{}{} {{", annotation, class)?;
            output_fields(out, 4, fields)?;
            writeln!(out, "}};")
        }
        Unknown => panic!("incorrect value"),
    }
}

fn output_variants(
    out: &mut dyn Write,
    base: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
) -> Result<()> {
    for (index, variant) in variants {
        output_variant(out, base, &variant.name, *index, &variant.value)?;
    }
    Ok(())
}

fn output_container(out: &mut dyn Write, name: &str, format: &ContainerFormat) -> Result<()> {
    use ContainerFormat::*;
    match format {
        UnitStruct => writeln!(out, "public static class {} {{}};\n", name),
        NewTypeStruct(format) => writeln!(
            out,
            "public static class {} {{\n    {} value;\n}};\n",
            name,
            quote_type(format)
        ),
        TupleStruct(formats) => writeln!(
            out,
            "public static class {} {{\n    {} value;\n}};\n",
            name,
            quote_type(&Format::Tuple(formats.clone()))
        ),
        Struct(fields) => {
            writeln!(out, "public static class {} {{", name)?;
            output_fields(out, 4, fields)?;
            writeln!(out, "}};\n")
        }
        Enum(variants) => {
            writeln!(
                out,
                r#"@Enum(variants={{
    {}
}})
public abstract static class {} {{}};
"#,
                variants
                    .iter()
                    .map(|(_, v)| format!("{}_{}.class", name, v.name))
                    .collect::<Vec<_>>()
                    .join(",\n    "),
                name
            )?;
            output_variants(out, name, variants)
        }
    }
}
