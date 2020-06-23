// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};
use std::collections::BTreeMap;
use std::io::{Result, Write};
use std::path::PathBuf;

/// Write container definitions in Python.
/// * The packages `dataclasses` and `typing` are assumed to be available.
/// * The module `serde_types` is assumed to be available.
pub fn output(out: &mut dyn Write, registry: &Registry) -> Result<()> {
    output_preamble(out, None)?;
    for (name, format) in registry {
        output_container(out, name, format)?;
    }
    Ok(())
}

fn output_with_optional_serde_package(
    out: &mut dyn Write,
    registry: &Registry,
    serde_package_name: Option<String>,
) -> Result<()> {
    output_preamble(out, serde_package_name)?;
    for (name, format) in registry {
        output_container(out, name, format)?;
    }
    Ok(())
}

fn output_preamble(out: &mut dyn Write, serde_package_name: Option<String>) -> Result<()> {
    writeln!(
        out,
        r#"# pyre-ignore-all-errors
from dataclasses import dataclass
import typing
{}import serde_types as st"#,
        match serde_package_name {
            None => "".to_string(),
            Some(name) => format!("from {} ", name),
        }
    )
}

fn quote_type(format: &Format) -> String {
    use Format::*;
    match format {
        TypeName(x) => format!("\"{}\"", x), // Need quotes because of circular dependencies.
        Unit => "st.unit".into(),
        Bool => "st.bool".into(),
        I8 => "st.int8".into(),
        I16 => "st.int16".into(),
        I32 => "st.int32".into(),
        I64 => "st.int64".into(),
        I128 => "st.int128".into(),
        U8 => "st.uint8".into(),
        U16 => "st.uint16".into(),
        U32 => "st.uint32".into(),
        U64 => "st.uint64".into(),
        U128 => "st.uint128".into(),
        F32 => "st.float32".into(),
        F64 => "st.float64".into(),
        Char => "st.char".into(),
        Str => "str".into(),
        Bytes => "bytes".into(),

        Option(format) => format!("typing.Optional[{}]", quote_type(format)),
        Seq(format) => format!("typing.Sequence[{}]", quote_type(format)),
        Map { key, value } => format!("typing.Dict[{}, {}]", quote_type(key), quote_type(value)),
        Tuple(formats) => format!("typing.Tuple[{}]", quote_types(formats)),
        TupleArray { content, size } => format!(
            "typing.Tuple[{}]",
            quote_types(&vec![content.as_ref().clone(); *size])
        ), // Sadly, there are no fixed-size arrays in python.

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
        writeln!(out, "{}{}: {}", tab, field.name, quote_type(&field.value))?;
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
    match variant {
        Unit => writeln!(
            out,
            "\n@dataclass\nclass _{}_{}({}):\n    INDEX = {}\n",
            base, name, base, index,
        ),
        NewType(format) => writeln!(
            out,
            "\n@dataclass\nclass _{}_{}({}):\n    INDEX = {}\n    value: {}\n",
            base,
            name,
            base,
            index,
            quote_type(format)
        ),
        Tuple(formats) => writeln!(
            out,
            "\n@dataclass\nclass _{}_{}({}):\n    INDEX = {}\n    value: typing.Tuple[{}]\n",
            base,
            name,
            base,
            index,
            quote_types(formats)
        ),
        Struct(fields) => {
            writeln!(
                out,
                "\n@dataclass\nclass _{}_{}({}):\n    INDEX = {}",
                base, name, base, index
            )?;
            output_fields(out, 4, fields)?;
            writeln!(out)
        }
        Variable(_) => panic!("incorrect value"),
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

fn output_variant_aliases(
    out: &mut dyn Write,
    base: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
) -> Result<()> {
    writeln!(out)?;
    for variant in variants.values() {
        writeln!(
            out,
            "{}.{} = _{}_{}",
            base, &variant.name, base, &variant.name
        )?;
    }
    Ok(())
}

fn output_container(out: &mut dyn Write, name: &str, format: &ContainerFormat) -> Result<()> {
    use ContainerFormat::*;
    match format {
        UnitStruct => writeln!(out, "\n@dataclass\nclass {}:\n    pass\n", name),
        NewTypeStruct(format) => writeln!(
            out,
            "\n@dataclass\nclass {}:\n    value: {}\n",
            name,
            quote_type(format)
        ),
        TupleStruct(formats) => writeln!(
            out,
            "\n@dataclass\nclass {}:\n    value: typing.Tuple[{}]\n",
            name,
            quote_types(formats)
        ),
        Struct(fields) => {
            writeln!(out, "\n@dataclass\nclass {}:", name)?;
            output_fields(out, 4, fields)?;
            writeln!(out)
        }
        Enum(variants) => {
            writeln!(out, "\nclass {}:\n    pass\n", name)?;
            output_variants(out, name, variants)?;
            output_variant_aliases(out, name, variants)?;
            writeln!(
                out,
                "{}.VARIANTS = [\n{}]\n",
                name,
                variants
                    .iter()
                    .map(|(_, v)| format!("    {}.{},\n", name, v.name))
                    .collect::<Vec<_>>()
                    .join("")
            )
        }
    }
}

pub struct Installer {
    install_dir: PathBuf,
    serde_package_name: Option<String>,
}

impl Installer {
    pub fn new(install_dir: PathBuf, serde_package_name: Option<String>) -> Self {
        Installer {
            install_dir,
            serde_package_name,
        }
    }

    fn open_module_init_file(&self, name: &str) -> Result<std::fs::File> {
        let dir_path = self.install_dir.join(name);
        std::fs::create_dir_all(&dir_path)?;
        std::fs::File::create(dir_path.join("__init__.py"))
    }

    fn fix_serde_package(&self, content: &str) -> String {
        match &self.serde_package_name {
            None => content.into(),
            Some(name) => content.replace(
                "import serde_types",
                &format!("from {} import serde_types", name),
            ),
        }
    }
}

impl crate::SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_module(
        &self,
        name: &str,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error> {
        let mut file = self.open_module_init_file(name)?;
        output_with_optional_serde_package(&mut file, registry, self.serde_package_name.clone())?;
        Ok(())
    }

    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error> {
        let mut file = self.open_module_init_file("serde_types")?;
        write!(
            file,
            "{}",
            self.fix_serde_package(include_str!("../runtime/python/serde_types/__init__.py"))
        )?;
        Ok(())
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        let mut file = self.open_module_init_file("bincode")?;
        write!(
            file,
            "{}",
            self.fix_serde_package(include_str!("../runtime/python/bincode/__init__.py"))
        )?;
        Ok(())
    }

    fn install_lcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        let mut file = self.open_module_init_file("lcs")?;
        write!(
            file,
            "{}",
            self.fix_serde_package(include_str!("../runtime/python/lcs/__init__.py"))
        )?;
        Ok(())
    }
}
