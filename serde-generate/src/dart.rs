use crate::{
    CodeGeneratorConfig,
};
use serde_reflection::{ Registry};
use std::{
    collections::{ HashMap},
    io::{Result, Write},
    path::PathBuf,
};

/// Main configuration object for code-generation in Java.
pub struct CodeGenerator<'a> {
    /// Language-independent configuration.
    _config: &'a CodeGeneratorConfig,
    /// Mapping from external type names to fully-qualified class names (e.g. "MyClass" -> "com.facebook.my_package.MyClass").
    /// Derived from `config.external_definitions`.
    _external_qualified_names: HashMap<String, String>,
}

impl<'a> CodeGenerator<'a> {
    /// Create a Java code generator for the given config.
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        let mut external_qualified_names = HashMap::new();
        for (namespace, names) in &config.external_definitions {
            for name in names {
                external_qualified_names
                    .insert(name.to_string(), format!("{}.{}", namespace, name));
            }
        }
        Self {
            _config:config,
            _external_qualified_names:external_qualified_names,
        }
    }

    /// Output class definitions for `registry`.
    pub fn output(&self, _out: &mut dyn Write, _registry: &Registry) -> Result<()> {
        Ok(())
    }

    fn write_container_class(
        &self,
        dir_path: &std::path::Path,
        current_namespace: Vec<String>,
        name: &str,
        format: &ContainerFormat,
    ) -> Result<()> {
        let mut file = std::fs::File::create(dir_path.join(name.to_string() + ".dart"))?;
        let mut emitter = DartEmitter {
            out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
            generator: self,
            current_namespace,
        };

        emitter.output_preamble()?;
        emitter.output_container(name, format)
    }

    fn write_helper_class(
        &self,
        dir_path: &std::path::Path,
        current_namespace: Vec<String>,
        registry: &Registry,
    ) -> Result<()> {
        let mut file = std::fs::File::create(dir_path.join("TraitHelpers.dart"))?;
        let mut emitter = DartEmitter {
            out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
            generator: self,
            current_namespace,
        };

        emitter.output_preamble()?;
        emitter.output_trait_helpers(registry)
    }

}


impl<'a, T> DartEmitter<'a, T>
    where
        T: Write,
{

    fn output_preamble(&mut self, ) -> Result<()> {
        writeln!(
            self.out,
            "part of {}_types;\n\n",
            self.generator.config.module_name
        )?;

        Ok(())
    }

    fn quote_qualified_name(&self, name: &str) -> String {
        return name.to_string()
    }

    fn quote_type(&self, format: &Format) -> String {
        use Format::*;
        match format {
            TypeName(x) => self.quote_qualified_name(x),
            Unit => "Unit".into(),
            Bool => "bool".into(),
            I8 => "int".into(),
            I16 => "int".into(),
            I32 => "int".into(),
            I64 => "int".into(),
            I128 => "org.starcoin.serde.Unit".into(),
            U8 => "int".into(),
            U16 => "int".into(),
            U32 => "int".into(),
            U64 => "int".into(),
            U128 => "int".into(),
            F32 => "float".into(),
            F64 => "double".into(),
            Char => "int".into(),
            Str => "String".into(),
            Bytes => "Uint8List".into(),

            Option(format) => format!("Optional<{}>", self.quote_type(format)),
            Seq(format) => format!("List<{}>", self.quote_type(format)),
            Map { key, value } => format!(
                "Map<{}, {}>",
                self.quote_type(key),
                self.quote_type(value)
            ),
            Tuple(formats) => format!(
                "Tuple{}<{}>",
                formats.len(),
                self.quote_types(formats)
            ),
            TupleArray { content, size:_ } => format!("List<{}>", self.quote_type(content)),
            Variable(_) => panic!("unexpected value"),
        }
    }

    fn quote_types(&self, formats: &[Format]) -> String {
        formats
            .iter()
            .map(|f| self.quote_type(f))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn quote_serialize_value(&self, value: &str, format: &Format) -> String {
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
                "{}.deserialize(deserializer)",
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

    fn enter_class(&mut self, name: &str) {
        self.out.indent();
        self.current_namespace.push(name.to_string());
    }

    fn leave_class(&mut self) {
        self.out.unindent();
        self.current_namespace.pop();
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
        writeln!(self.out, "class TraitHelpers {{")?;
        self.enter_class("TraitHelpers");
        for (mangled_name, subtype) in &subtypes {
            self.output_serialization_helper(mangled_name, subtype)?;
            self.output_deserialization_helper(mangled_name, subtype)?;
        }
        self.leave_class();
        writeln!(self.out, "}}\n")
    }

    fn needs_helper(format: &Format) -> bool {
        use Format::*;
        match format {
            Option(_) | Seq(_) | Map { .. } | Tuple(_) | TupleArray { .. } => true,
            _ => false,
        }
    }

    fn output_serialization_helper(&mut self, name: &str, format0: &Format) -> Result<()> {
        use Format::*;

        write!(
            self.out,
            "static void serialize_{}({} value, BinarySerializer serializer) {{",
            name,
            self.quote_type(format0)
        )?;
        self.out.indent();
        match format0 {
            Option(format) => {
                write!(
                    self.out,
                    r#"
if (value.isPresent) {{
    serializer.serialize_option_tag(true);
    {}
}} else {{
    serializer.serialize_option_tag(false);
}}
"#,
                    self.quote_serialize_value("value.value", format)
                )?;
            }

            Seq(format) => {
                write!(
                    self.out,
                    r#"
serializer.serialize_len(value.length);
for ({} item in value) {{
    {}
}}
"#,
                    self.quote_type(format),
                    self.quote_serialize_value("item", format)
                )?;
            }

            Map { key, value } => {
                write!(
                    self.out,
                    r#"
serializer.serialize_len(value.length);
List<int> offsets = new List<int>;
int count = 0;
for (Map.Entry<{}, {}> entry : value.entrySet()) {{
    offsets[count++] = serializer.get_buffer_offset();
    {}
    {}
}}
serializer.sort_map_entries(offsets);
"#,
                    self.quote_type(key),
                    self.quote_type(value),
                    self.quote_serialize_value("entry.getKey()", key),
                    self.quote_serialize_value("entry.getValue()", value)
                )?;
            }

            Tuple(formats) => {
                writeln!(self.out)?;
                for (index, format) in formats.iter().enumerate() {
                    let expr = format!("value.field{}", index);
                    writeln!(self.out, "{}", self.quote_serialize_value(&expr, format))?;
                }
            }

            TupleArray { content, size } => {
                write!(
                    self.out,
                    r#"
assert (value.length == {});
for ({} item in value) {{
    {}
}}
"#,
                    size,
                    self.quote_type(content),
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
            "static {} deserialize_{}(BinaryDeserializer deserializer) {{",
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
    return Optional.empty();
}} else {{
    return Optional.of({});
}}
"#,
                    self.quote_deserialize(format),
                )?;
            }

            Seq(format) => {
                write!(
                    self.out,
                    r#"
int length = deserializer.deserialize_len();
List<{0}> obj = new List<{0}>(length);
for (int i = 0; i < length; i++) {{
    obj.add({1});
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
int length = deserializer.deserialize_len();
Map<{0}, {1}> obj = new HashMap<{0}, {1}>();
int previous_key_start = 0;
int previous_key_end = 0;
for (int i = 0; i < length; i++) {{
    int key_start = deserializer.get_buffer_offset();
    {0} key = {2};
    int key_end = deserializer.get_buffer_offset();
    if (i > 0) {{
        deserializer.check_that_key_slices_are_increasing(
            new Slice(previous_key_start, previous_key_end),
            new Slice(key_start, key_end));
    }}
    previous_key_start = key_start;
    previous_key_end = key_end;
    {1} value = {3};
    obj.put(key, value);
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
return new {}({}
);
"#,
                    self.quote_type(format0),
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
List<{0}> obj = new List<{0}>();
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

    fn output_struct_or_variant_container(
        &mut self,
        variant_base: Option<&str>,
        variant_index: Option<u32>,
        name: &str,
        fields: &[Named<Format>],
    ) -> Result<()> {
        // Beginning of class
        writeln!(self.out)?;
        if let Some(base) = variant_base {
            //self.output_comment(name)?;
            writeln!(
                self.out,
                "class {} extends {} {{",
                name, base
            )?;
        } else {
            //self.output_comment(name)?;
            writeln!(self.out, " class {} {{", name)?;
        }
        self.enter_class(name);
        // Fields
        for field in fields {
            //self.output_comment(&field.name)?;
            writeln!(
                self.out,
                "{} {};",
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
            "{}({}) {{",
            name,
            fields
                .iter()
                .map(|f| format!("{} {}", self.quote_type(&f.value), &f.name))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        self.out.indent();
        for field in fields {
            writeln!(self.out, "assert ({} != null);", &field.name)?;
        }
        for field in fields {
            writeln!(self.out, "this.{} = {};", &field.name, &field.name)?;
        }
        self.out.unindent();
        writeln!(self.out, "}}")?;

        // Serialize
        if self.generator.config.serialization {
            writeln!(
                self.out,
                "\nvoid serialize(BinarySerializer serializer){{",
            )?;
            self.out.indent();
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
                    "\nstatic {} deserialize(BinaryDeserializer deserializer){{",
                    name,
                )?;
            } else {
                writeln!(
                    self.out,
                    "\nstatic {} load(BinaryDeserializer deserializer){{",
                    name,
                )?;
            }
            self.out.indent();

            for field in fields {
                writeln!(
                    self.out,
                    "var {} = {};",
                    field.name,
                    self.quote_deserialize(&field.value)
                )?;

            }
            writeln!(self.out, "return new {}({});",name,fields
                .iter()
                .map(|f|  f.name.to_string())
                .collect::<Vec<_>>()
                .join(","))?;

            self.out.unindent();
            writeln!(self.out, "}}")?;

            if variant_index.is_none() {
                for encoding in &self.generator.config.encodings {
                    self.output_class_deserialize_for_encoding(name, *encoding)?;
                }
            }
        }
        // Equality
        write!(self.out, "\n@override")?;
        write!(self.out, "\nbool operator ==(covariant {0} other) {{",name)?;
        self.out.indent();
        writeln!(
            self.out,
            r#"
if (other == null) return false;"#,
        )?;
        let fields_num = fields.len();

        if fields_num > 0 {
            write!(self.out, "\nif (")?;

            for (index,field) in fields.iter().enumerate() {
            if index < fields_num-1{
                writeln!(
                    self.out,
                    " this.{0} == other.{0} &&",
                    &field.name,
                )?;
            }else {
                writeln!(
                    self.out,
                    " this.{0} == other.{0} ) {{",
                    &field.name,
                )?;
            }
            }
            writeln!(self.out, "return true;}}")?;
            writeln!(self.out, "else return false;")?;
        }else {
            writeln!(self.out, "return true;")?;
        }

        self.out.unindent();
        writeln!(self.out, "}}")?;
        // Hashing
        write!(self.out, "\n@override")?;
        writeln!(self.out, "\nint get hashCode {{")?;
        self.out.indent();
        writeln!(self.out, "int value = 7;",)?;
        for field in fields {
            writeln!(
                self.out,
                "value = 31 * value + (this.{0} != null ? this.{0}.hashCode : 0);",
                &field.name
            )?;
        }
        writeln!(self.out, "return value;")?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        // End of class
        self.leave_class();
        writeln!(self.out, "}}")
    }

    fn output_class_serialize_for_encoding(&mut self, encoding: Encoding) -> Result<()> {
        writeln!(
            self.out,
            r#"
  Uint8List {0}Serialize() {{
    var serializer = new {1}Serializer();
    serialize(serializer);
    return serializer.get_bytes();
}}"#,
            encoding.name(),
            encoding.name().to_camel_case(),
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
 static {0} {1}Deserialize(Uint8List input)  {{
   var deserializer = new {2}Deserializer(input);
    {0} value = deserialize(deserializer);
    if (deserializer.get_buffer_offset() < input.length) {{
         throw new Exception("Some input bytes were not read");
    }}
    return value;
}}"#,
            name,
            encoding.name(),
            encoding.name().to_camel_case()
        )
    }

    fn output_enum_container(
        &mut self,
        name: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> Result<()> {
        writeln!(self.out)?;
        //self.output_comment(name)?;
        writeln!(self.out, "abstract class {} {{", name)?;
        self.enter_class(name);
        if self.generator.config.serialization {
            writeln!(
                self.out,
                "\nvoid serialize(BinarySerializer serializer);"
            )?;
            write!(
                self.out,
                "\nstatic {} deserialize(BinaryDeserializer deserializer) {{",
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
                    "case {}: return {}{}Item.load(deserializer);",
                    index,name ,variant.name,
                )?;
            }
            writeln!(
                self.out,
                "default: throw new Exception(\"Unknown variant index for {}: \" + index.toString());",
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

        writeln!(self.out, "}}\n")?;

        self.output_variants(name, variants)?;
        self.leave_class();
        Ok(())
    }

    fn output_variants(
        &mut self,
        base: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> Result<()> {
        for (index, variant) in variants {
            self.output_variant(base, *index, &format!("{}{}Item",base,&variant.name), &variant.value)?;
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

}

/// Installer for generated source files in Go.
pub struct Installer {
    install_dir: PathBuf,
}

impl Installer {
    pub fn new(install_dir: PathBuf) -> Self {
        Installer {
            install_dir,
        }
    }

    fn runtimes_installation_not_required() -> std::result::Result<(), Box<dyn std::error::Error>> {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Runtime is installed by `go get`, no source code installation required",
        )))
    }
}

impl crate::SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_module(
        &self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error> {
        let dir_path = self.install_dir.join(&config.module_name);
        std::fs::create_dir_all(&dir_path)?;
        let source_path = dir_path.join("lib.go");
        let mut file = std::fs::File::create(source_path)?;

        let generator = CodeGenerator::new(config);
        // if let Some(path) = &self.serde_module_path {
        //     generator = generator.with_serde_module_path(path.clone());
        // }
        generator.output(&mut file, registry)?;
        Ok(())
    }

    fn install_serde_runtime(&self) -> std::result::Result<(), Self::Error> {
        Self::runtimes_installation_not_required()
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        Self::runtimes_installation_not_required()
    }

    fn install_lcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        Self::runtimes_installation_not_required()
    }
}
