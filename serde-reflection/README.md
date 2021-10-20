# serde-reflection

[![serde-reflection on crates.io](https://img.shields.io/crates/v/serde-reflection)](https://crates.io/crates/serde-reflection)
[![Documentation (latest release)](https://docs.rs/serde-reflection/badge.svg)](https://docs.rs/serde-reflection/)
[![Documentation (main)](https://img.shields.io/badge/docs-main-brightgreen)](https://novifinancial.github.io/serde-reflection/serde_reflection/)
[![License](https://img.shields.io/badge/license-Apache-green.svg)](../LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../LICENSE-MIT)

This crate provides a way to extract format descriptions for Rust containers that
implement the Serialize and/or Deserialize trait(s) of Serde.

Format descriptions are useful in several ways:
* Stored under version control, formats can be tested to prevent unintended modifications
of binary serialization formats (e.g. by changing variant order).
* Formats can be passed to [`serde-generate`](https://docs.rs/serde-generate)
in order to generate class definitions and provide Serde-compatible binary
serialization in other languages (C++, python, Java, etc).

## Quick Start

Very often, Serde traits are simply implemented using Serde derive macros. In this case,
you may obtain format descriptions as follows:
* call `trace_simple_type` on the desired top-level container definition(s), then
* add a call to `trace_simple_type` for each `enum` type. (This will fix any `MissingVariants` error.)

```rust
#[derive(Deserialize)]
struct Foo {
  bar: Bar,
  choice: Choice,
}

#[derive(Deserialize)]
struct Bar(u64);

#[derive(Deserialize)]
enum Choice { A, B, C }

// Start the tracing session.
let mut tracer = Tracer::new(TracerConfig::default());

// Trace the desired top-level type(s).
tracer.trace_simple_type::<Foo>()?;

// Also trace each enum type separately to fix any `MissingVariants` error.
tracer.trace_simple_type::<Choice>()?;

// Obtain the registry of Serde formats and serialize it in YAML (for instance).
let registry = tracer.registry()?;
let data = serde_yaml::to_string(&registry).unwrap();
assert_eq!(&data, r#"---
Bar:
  NEWTYPESTRUCT: U64
Choice:
  ENUM:
    0:
      A: UNIT
    1:
      B: UNIT
    2:
      C: UNIT
Foo:
  STRUCT:
    - bar:
        TYPENAME: Bar
    - choice:
        TYPENAME: Choice
"#);
```

## Features and Limitations

`serde_reflection` is meant to extract formats for Rust containers (i.e. structs and
enums) with "reasonable" implementations of the Serde traits `Serialize` and
`Deserialize`.

Supported implementations include:

* Plain derived implementations obtained with `#[derive(Serialize, Deserialize)]` for
Rust containers in the Serde [data model](https://serde.rs/data-model.html)

* Customized derived implementations using Serde attributes that are compatible with
binary serialization formats, such as `#[serde(rename = "Name")]`.

* Hand-written implementations of `Deserialize` that are more restrictive than the
derived ones, provided that `trace_value` is used during tracing to provide sample
values for all such constrained types (see the detailed example below).

* Mutually recursive types provided that the first variant of each enum is
recursion-free. (For instance, `enum List { None, Some(Box<List>)}`.) Note that each
enum must be traced separately with `trace_type` to discover all the variants.

Unsupported idioms include:

* Containers sharing the same base name (e.g. `Foo`) but from different modules. (Work
around: use `#[serde(rename = ..)]`)

* Generic types instantiated multiple times in the same tracing session. (Work around:
use the crate [`serde-name`](https://crates.io/crates/serde-name) and its adapters `SerializeNameAdapter` and `DeserializeNameAdapter`.)

* Attributes that are not compatible with binary formats (e.g. `#[serde(flatten)]`, `#[serde(tag = ..)]`)

* Tracing type aliases. (E.g. `type Pair = (u32, u64)` will not create an entry "Pair".)

* Mutually recursive types for which picking the first variant of each enum does not
terminate. (Work around: re-order the variants. For instance `enum List {
Some(Box<List>), None}` must be rewritten `enum List { None, Some(Box<List>)}`.)

## Troubleshooting

The error type used in this crate provides a method `error.explanation()` to help with
troubleshooting during format tracing.

## Detailed Example

In the following, more complete example, we extract the Serde formats of two containers
`Name` and `Person` and demonstrate how to handle a custom implementation of `serde::Deserialize`
for `Name`.

```rust
use serde_reflection::{ContainerFormat, Error, Format, Samples, Tracer, TracerConfig};

#[derive(Serialize, PartialEq, Eq, Debug, Clone)]
struct Name(String);
// impl<'de> Deserialize<'de> for Name { ... }

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
enum Person {
    NickName(Name),
    FullName { first: Name, last: Name },
}

// Start a session to trace formats.
let mut tracer = Tracer::new(TracerConfig::default());
// Create a store to hold samples of Rust values.
let mut samples = Samples::new();

// For every type (here `Name`), if a user-defined implementation of `Deserialize` exists and
// is known to perform custom validation checks, use `trace_value` first so that `samples`
// contains a valid Rust value of this type.
let bob = Name("Bob".into());
tracer.trace_value(&mut samples, &bob)?;
assert!(samples.value("Name").is_some());

// Now, let's trace deserialization for the top-level type `Person`.
// We pass a reference to `samples` so that sampled values are used for custom types.
let (format, values) = tracer.trace_type::<Person>(&samples)?;
assert_eq!(format, Format::TypeName("Person".into()));

// As a byproduct, we have also obtained sample values of type `Person`.
// We can see that the user-provided value `bob` was used consistently to pass
// validation checks for `Name`.
assert_eq!(values[0], Person::NickName(bob.clone()));
assert_eq!(values[1], Person::FullName { first: bob.clone(), last: bob.clone() });

// We have no more top-level types to trace, so let's stop the tracing session and obtain
// a final registry of containers.
let registry = tracer.registry()?;

// We have successfully extracted a format description of all Serde containers under `Person`.
assert_eq!(
    registry.get("Name").unwrap(),
    &ContainerFormat::NewTypeStruct(Box::new(Format::Str)),
);
match registry.get("Person").unwrap() {
    ContainerFormat::Enum(variants) => assert_eq!(variants.len(), 2),
     _ => panic!(),
};

// Export the registry in YAML.
let data = serde_yaml::to_string(&registry).unwrap();
assert_eq!(&data, r#"---
Name:
  NEWTYPESTRUCT: STR
Person:
  ENUM:
    0:
      NickName:
        NEWTYPE:
          TYPENAME: Name
    1:
      FullName:
        STRUCT:
          - first:
              TYPENAME: Name
          - last:
              TYPENAME: Name
"#);
```

## Tracing Serialization with `trace_value`

Tracing the serialization of a Rust value `v` consists of visiting the structural
components of `v` in depth and recording Serde formats for all the visited types.

```rust
#[derive(Serialize)]
struct FullName<'a> {
  first: &'a str,
  middle: Option<&'a str>,
  last: &'a str,
}

let mut tracer = Tracer::new(TracerConfig::default());
let mut samples = Samples::new();
tracer.trace_value(&mut samples, &FullName { first: "", middle: Some(""), last: "" })?;
let registry = tracer.registry()?;
match registry.get("FullName").unwrap() {
    ContainerFormat::Struct(fields) => assert_eq!(fields.len(), 3),
    _ => panic!(),
};
```

This approach works well but it can only recover the formats of datatypes for which
nontrivial samples have been provided:

* In enums, only the variants explicitly covered by user samples will be recorded.

* Providing a `None` value or an empty vector `[]` within a sample may result in
formats that are partially unknown.

```rust
let mut tracer = Tracer::new(TracerConfig::default());
let mut samples = Samples::new();
tracer.trace_value(&mut samples, &FullName { first: "", middle: None, last: "" })?;
assert_eq!(tracer.registry().unwrap_err(), Error::UnknownFormatInContainer("FullName".to_string()));
```

For this reason, we introduce a complementary set of APIs to trace deserialization of types.

## Tracing Deserialization with `trace_type<T>`

Deserialization-tracing APIs take a type `T`, the current tracing state, and a
reference to previously recorded samples as input.

### Core Algorithm and High-Level API

The core algorithm `trace_type_once<T>`
attempts to reconstruct a witness value of type `T` by exploring the graph of all the types
occurring in the definition of `T`. At the same time, the algorithm records the
formats of all the visited structs and enum variants.

For the exploration to be able to terminate, the core algorithm `trace_type_once<T>` explores
each possible recursion point only once (see paragraph below).
In particular, if `T` is an enum, `trace_type_once<T>` discovers only one variant of `T` at a time.

For this reason, the high-level API `trace_type<T>`
will repeat calls to `trace_type_once<T>` until all the variants of `T` are known.
Variant cases of `T` are explored in sequential order, starting with index `0`.

### Coverage Guarantees

Under the assumptions listed below, a single call to `trace_type<T>` is guaranteed to
record formats for all the types that `T` depends on. Besides, if `T` is an enum, it
will record all the variants of `T`.

(0) Container names must not collide. If this happens, consider using `#[serde(rename = "name")]`,
or implementing serde traits manually.

(1) The first variants of mutually recursive enums must be a "base case". That is,
defaulting to the first variant for every enum type (along with `None` for option values
and `[]` for sequences) must guarantee termination of depth-first traversals of the graph of type
declarations.

(2) If a type runs custom validation checks during deserialization, sample values must have been provided
previously by calling `trace_value`. Besides, the corresponding registered formats
must not contain unknown parts.

### Design Considerations

Whenever we traverse the graph of type declarations using deserialization callbacks, the type
system requires us to return valid Rust values of type `V::Value`, where `V` is the type of
a given `visitor`. This contraint limits the way we can stop graph traversal to only a few cases.

The first 4 cases are what we have called *possible recursion points* above:

* while visiting an `Option<T>` for the second time, we choose to return the value `None` to stop;
* while visiting an `Seq<T>` for the second time, we choose to return the empty sequence `[]`;
* while visiting an `Map<K, V>` for the second time, we choose to return the empty map `{}`;
* while visiting an `enum T` for the second time, we choose to return the first variant, i.e.
a "base case" by assumption (1) above.

In addition to the cases above,

* while visiting a container, if the container's name is mapped to a recorded value,
we MAY decide to use it.

The default configuration `TracerConfig:default()` always picks the recorded value for a
`NewTypeStruct` and never does in the other cases.

For efficiency reasons, the current algorithm does not attempt to scan the variants of enums
other than the parameter `T` of the main call `trace_type<T>`. As a consequence, each enum type must be
traced separately.

## Contributing

See the [CONTRIBUTING](../CONTRIBUTING.md) file for how to help out.

## License

This project is available under the terms of either the [Apache 2.0 license](../LICENSE-APACHE) or the [MIT license](../LICENSE-MIT).

<!--
README.md is generated from README.tpl by cargo readme. To regenerate:

cargo install cargo-readme
cargo readme > README.md
-->
