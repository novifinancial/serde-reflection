// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

#![forbid(unsafe_code)]

//! This crate provides fast and reliable ways to extract and to override the Serde name
//! of a Rust container.
//!
//! # Extracting Serde names
//!
//! Name extraction relies on the Deserialize trait of Serde:
//!
//! ```rust
//! # use serde::Deserialize;
//! # use serde_name::trace_name;
//! #[derive(Deserialize)]
//! struct Foo {
//!   bar: Bar,
//! }
//!
//! #[derive(Deserialize)]
//! #[serde(rename = "ABC")]
//! enum Bar { A, B, C }
//!
//! assert_eq!(trace_name::<Foo>(), Some("Foo"));
//! assert_eq!(trace_name::<Bar>(), Some("ABC"));
//! assert_eq!(trace_name::<Option<Bar>>(), None);
//! ```
//!
//! # Overriding Serde names
//!
//! `SerializeNameAdapter` and `DeserializeNameAdapter` may be used to override the name
//! of a container in the cases where `#[serde(rename = "..")]` is not flexible enough.
//!
//! ```rust
//! # use serde_name::{SerializeNameAdapter, DeserializeNameAdapter, trace_name};
//! # use serde::{Deserialize, Serialize};
//! struct Foo<T> {
//!     data: T,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! #[serde(remote = "Foo")]
//! struct FooInternal<S> {
//!     data: S,
//! }
//!
//! impl<'de, T> Deserialize<'de> for Foo<T>
//! where
//!     T: Deserialize<'de>,
//! {
//!     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//!     where
//!         D: serde::de::Deserializer<'de>,
//!     {
//!         FooInternal::deserialize(DeserializeNameAdapter::new(
//!             deserializer,
//!             std::any::type_name::<Self>(),
//!         ))
//!     }
//! }
//!
//! impl<T> Serialize for Foo<T>
//! where
//!     T: Serialize,
//! {
//!     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//!     where
//!         S: serde::ser::Serializer,
//!     {
//!         FooInternal::serialize(
//!             self,
//!             SerializeNameAdapter::new(serializer, std::any::type_name::<Self>()),
//!         )
//!     }
//! }
//!
//! // Testing the Deserialize implementation
//! assert!(trace_name::<Foo<u64>>().unwrap().ends_with("Foo<u64>"));
//!
//! // Testing the Serialize implementation
//! use serde_reflection::*;
//! let mut tracer = Tracer::new(TracerConfig::default());
//! let mut samples = Samples::new();
//! let (mut ident, _) = tracer.trace_value(&mut samples, &Foo { data: 1u64 }).unwrap();
//! ident.normalize().unwrap();
//! assert!(matches!(ident, Format::TypeName(s) if s.ends_with("Foo<u64>")));
//! ```

mod de_adapter;
mod ser_adapter;
mod trace;

pub use de_adapter::DeserializeNameAdapter;
pub use ser_adapter::SerializeNameAdapter;
pub use trace::trace_name;
