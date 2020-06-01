// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::analyzer;
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};
use std::collections::{BTreeMap, HashSet};
use std::io::{Result, Write};

pub fn output(
    out: &mut dyn Write,
    with_derive_macros: bool,
    registry: &Registry,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let dependencies = analyzer::get_dependency_map(registry)?;
    let entries = analyzer::best_effort_topological_sort(&dependencies);

    output_preambule(out, with_derive_macros)?;
    let mut known_sizes = HashSet::new();
    for name in entries {
        let format = &registry[name];
        output_container(out, with_derive_macros, name, format, &known_sizes)?;
        known_sizes.insert(name);
    }
    Ok(())
}

fn output_preambule(out: &mut dyn Write, with_derive_macros: bool) -> Result<()> {
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
        Bytes => "serde_bytes::ByteBuf".into(),

        Option(format) => format!("Option<{}>", quote_type(format, known_sizes)),
        Seq(format) => format!("Vec<{}>", quote_type(format, None)),
        Map { key, value } => format!(
            "std::collections::BTreeMap<{}, {}>",
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
    name: &str,
    format: &ContainerFormat,
    known_sizes: &HashSet<&str>,
) -> Result<()> {
    use ContainerFormat::*;
    let traits = if with_derive_macros {
        "#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd)]\n"
    } else {
        ""
    };
    match format {
        UnitStruct => writeln!(out, "{}pub struct {};\n", traits, name),
        NewTypeStruct(format) => writeln!(
            out,
            "{}pub struct {}({});\n",
            traits,
            name,
            quote_type(format, Some(known_sizes))
        ),
        TupleStruct(formats) => writeln!(
            out,
            "{}pub struct {}({});\n",
            traits,
            name,
            quote_types(formats, Some(known_sizes))
        ),
        Struct(fields) => {
            writeln!(out, "{}pub struct {} {{", traits, name)?;
            output_fields(out, 4, fields, true, known_sizes)?;
            writeln!(out, "}}\n")
        }
        Enum(variants) => {
            writeln!(out, "{}pub enum {} {{", traits, name)?;
            output_variants(out, variants, known_sizes)?;
            writeln!(out, "}}\n")
        }
    }
}
