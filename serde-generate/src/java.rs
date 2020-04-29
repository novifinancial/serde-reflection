// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use include_dir::include_dir;
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};
use std::collections::BTreeMap;
use std::io::{Result, Write};
use std::path::PathBuf;

pub fn output(out: &mut dyn Write, registry: &Registry, class_name: &str) -> Result<()> {
    output_preambule(out, None)?;

    writeln!(out, "public class {} {{", class_name)?;
    for (name, format) in registry {
        output_container(out, name, format, /* nested class */ true)?;
    }
    writeln!(out, "}}")
}

fn output_preambule(out: &mut dyn Write, package_name: Option<&str>) -> Result<()> {
    if let Some(name) = package_name {
        writeln!(out, "package {};", name,)?;
    }
    writeln!(
        out,
        r#"
import java.math.BigInteger;
import java.nio.ByteBuffer;
import java.util.Optional;
import java.util.Vector;
import java.util.SortedMap;
import serde.FixedLength;
import serde.Int128;
import serde.Unsigned;
import serde.Tuple2;
import serde.Tuple3;
import serde.Tuple4;
import serde.Tuple5;
import serde.Tuple6;
"#
    )
}

fn quote_type(format: &Format) -> String {
    use Format::*;
    match format {
        TypeName(x) => x.to_string(),
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
        Variable(_) => panic!("unexpected value"),
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
    variant: &VariantFormat,
) -> Result<()> {
    use VariantFormat::*;
    let class = format!("    public static class {} extends {}", name, base);
    match variant {
        Unit => writeln!(out, "\n{} {{}}", class),
        NewType(format) => writeln!(
            out,
            "\n{} {{\n        public {} value;\n    }}",
            class,
            quote_type(format),
        ),
        Tuple(formats) => writeln!(
            out,
            "\n{} {{\n        public {} value;\n    }}",
            class,
            quote_type(&Format::Tuple(formats.clone())),
        ),
        Struct(fields) => {
            writeln!(out, "\n{} {{", class)?;
            output_fields(out, 8, fields)?;
            writeln!(out, "    }}")
        }
        Variable(_) => panic!("incorrect value"),
    }
}

fn output_variants(
    out: &mut dyn Write,
    base: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
) -> Result<()> {
    for (_index, variant) in variants {
        output_variant(out, base, &variant.name, &variant.value)?;
    }
    Ok(())
}

fn output_container(
    out: &mut dyn Write,
    name: &str,
    format: &ContainerFormat,
    nested_class: bool,
) -> Result<()> {
    use ContainerFormat::*;
    let prefix = if nested_class {
        "public static "
    } else {
        "public "
    };
    match format {
        UnitStruct => writeln!(out, "{}class {} {{}}\n", prefix, name),
        NewTypeStruct(format) => writeln!(
            out,
            "{}class {} {{\n    public {} value;\n}}\n",
            prefix,
            name,
            quote_type(format)
        ),
        TupleStruct(formats) => writeln!(
            out,
            "{}class {} {{\n    public {} value;\n}}\n",
            prefix,
            name,
            quote_type(&Format::Tuple(formats.clone()))
        ),
        Struct(fields) => {
            writeln!(out, "{}class {} {{", prefix, name)?;
            output_fields(out, 4, fields)?;
            writeln!(out, "}}\n")
        }
        Enum(variants) => {
            writeln!(
                out,
                r#"{}abstract class {} {{"#,
                prefix,
                name
            )?;
            output_variants(out, name, variants)?;
            writeln!(out, "}}")
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
            output_container(&mut file, name, format, /* nested class */ false)?;
        }
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
