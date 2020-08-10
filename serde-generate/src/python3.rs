// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::indent::{IndentConfig, IndentedWriter};
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};
use std::collections::BTreeMap;
use std::io::{Result, Write};
use std::path::PathBuf;

/// Write container definitions in Python.
/// * The packages `dataclasses` and `typing` are assumed to be available.
/// * The module `serde_types` is assumed to be available.
pub fn output(out: &mut dyn Write, registry: &Registry) -> Result<()> {
    output_with_optional_serde_package(out, registry, None)
}

fn output_with_optional_serde_package(
    out: &mut dyn Write,
    registry: &Registry,
    serde_package_name: Option<String>,
) -> Result<()> {
    let mut emitter = PythonEmitter {
        out: IndentedWriter::new(out, IndentConfig::Space(4)),
        serde_package_name,
    };
    emitter.output_preamble()?;
    for (name, format) in registry {
        emitter.output_container(name, format)?;
    }
    Ok(())
}

struct PythonEmitter<T> {
    out: IndentedWriter<T>,
    serde_package_name: Option<String>,
}

impl<T> PythonEmitter<T>
where
    T: Write,
{
    fn output_preamble(&mut self) -> Result<()> {
        writeln!(
            self.out,
            r#"from dataclasses import dataclass
import typing
{}import serde_types as st"#,
            match &self.serde_package_name {
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

            Option(format) => format!("typing.Optional[{}]", Self::quote_type(format)),
            Seq(format) => format!("typing.Sequence[{}]", Self::quote_type(format)),
            Map { key, value } => format!(
                "typing.Dict[{}, {}]",
                Self::quote_type(key),
                Self::quote_type(value)
            ),
            Tuple(formats) => format!("typing.Tuple[{}]", Self::quote_types(formats)),
            TupleArray { content, size } => format!(
                "typing.Tuple[{}]",
                Self::quote_types(&vec![content.as_ref().clone(); *size])
            ), // Sadly, there are no fixed-size arrays in python.

            Variable(_) => panic!("unexpected value"),
        }
    }

    fn quote_types(formats: &[Format]) -> String {
        formats
            .iter()
            .map(Self::quote_type)
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn output_fields(&mut self, fields: &[Named<Format>]) -> Result<()> {
        for field in fields {
            writeln!(
                self.out,
                "{}: {}",
                field.name,
                Self::quote_type(&field.value)
            )?;
        }
        Ok(())
    }

    fn output_variant(
        &mut self,
        base: &str,
        name: &str,
        index: u32,
        variant: &VariantFormat,
    ) -> Result<()> {
        use VariantFormat::*;
        match variant {
            Unit => writeln!(
                self.out,
                "\n@dataclass\nclass {}__{}({}):\n    INDEX = {}\n",
                base, name, base, index,
            ),
            NewType(format) => writeln!(
                self.out,
                "\n@dataclass\nclass {}__{}({}):\n    INDEX = {}\n    value: {}\n",
                base,
                name,
                base,
                index,
                Self::quote_type(format)
            ),
            Tuple(formats) => writeln!(
                self.out,
                "\n@dataclass\nclass {}__{}({}):\n    INDEX = {}\n    value: typing.Tuple[{}]\n",
                base,
                name,
                base,
                index,
                Self::quote_types(formats)
            ),
            Struct(fields) => {
                writeln!(
                    self.out,
                    "\n@dataclass\nclass {}__{}({}):\n    INDEX = {}",
                    base, name, base, index
                )?;
                self.out.indent();
                self.output_fields(fields)?;
                self.out.unindent();
                writeln!(self.out)
            }
            Variable(_) => panic!("incorrect value"),
        }
    }

    fn output_variants(
        &mut self,
        base: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> Result<()> {
        for (index, variant) in variants {
            self.output_variant(base, &variant.name, *index, &variant.value)?;
        }
        Ok(())
    }

    fn output_container(&mut self, name: &str, format: &ContainerFormat) -> Result<()> {
        use ContainerFormat::*;
        match format {
            UnitStruct => writeln!(self.out, "\n@dataclass\nclass {}:\n    pass\n", name),
            NewTypeStruct(format) => writeln!(
                self.out,
                "\n@dataclass\nclass {}:\n    value: {}\n",
                name,
                Self::quote_type(format)
            ),
            TupleStruct(formats) => writeln!(
                self.out,
                "\n@dataclass\nclass {}:\n    value: typing.Tuple[{}]\n",
                name,
                Self::quote_types(formats)
            ),
            Struct(fields) => {
                writeln!(self.out, "\n@dataclass\nclass {}:", name)?;
                self.out.indent();
                self.output_fields(fields)?;
                self.out.unindent();
                writeln!(self.out)
            }
            Enum(variants) => {
                // Initializing VARIANTS with a temporary value for typechecking purposes.
                writeln!(self.out, "\nclass {}:\n    VARIANTS = []\n", name)?;
                self.output_variants(name, variants)?;
                writeln!(
                    self.out,
                    "{}.VARIANTS = [\n{}]\n",
                    name,
                    variants
                        .iter()
                        .map(|(_, v)| format!("    {}__{},\n", name, v.name))
                        .collect::<Vec<_>>()
                        .join("")
                )
            }
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

    fn create_module_init_file(&self, name: &str) -> Result<std::fs::File> {
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
        config: &crate::CodegenConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error> {
        let mut file = self.create_module_init_file(&config.module_name)?;
        output_with_optional_serde_package(&mut file, registry, self.serde_package_name.clone())?;
        Ok(())
    }

    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error> {
        let mut file = self.create_module_init_file("serde_types")?;
        write!(
            file,
            "{}",
            self.fix_serde_package(include_str!("../runtime/python/serde_types/__init__.py"))
        )?;
        Ok(())
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        let mut file = self.create_module_init_file("bincode")?;
        write!(
            file,
            "{}",
            self.fix_serde_package(include_str!("../runtime/python/bincode/__init__.py"))
        )?;
        Ok(())
    }

    fn install_lcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        let mut file = self.create_module_init_file("lcs")?;
        write!(
            file,
            "{}",
            self.fix_serde_package(include_str!("../runtime/python/lcs/__init__.py"))
        )?;
        Ok(())
    }
}
