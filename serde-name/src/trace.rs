// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::de::Visitor;
use thiserror::Error;

/// Compute the Serde name of a container.
pub fn trace_name<'de, T>() -> Option<&'static str>
where
    T: serde::de::Deserialize<'de>,
{
    match T::deserialize(NameTracer) {
        Err(NameTracerError(name)) => name,
        _ => unreachable!(),
    }
}

/// Minimal instrumented implementation of `serde::de::Deserializer`
/// This always returns a `NameTracerError` as soon as we have learnt the name
/// of the type (or the absence of name) from Serde.
struct NameTracer;

/// Custom error value used to report the result of the analysis.
#[derive(Clone, Debug, Error, PartialEq)]
#[error("{0:?}")]
struct NameTracerError(Option<&'static str>);

impl serde::de::Error for NameTracerError {
    fn custom<T: std::fmt::Display>(_msg: T) -> Self {
        unreachable!();
    }
}

macro_rules! declare_deserialize {
    ($method:ident) => {
        fn $method<V>(self, _visitor: V) -> std::result::Result<V::Value, NameTracerError>
        where
            V: Visitor<'de>,
        {
            Err(NameTracerError(None))
        }
    };
}

impl<'de> serde::de::Deserializer<'de> for NameTracer {
    type Error = NameTracerError;

    declare_deserialize!(deserialize_any);
    declare_deserialize!(deserialize_identifier);
    declare_deserialize!(deserialize_ignored_any);
    declare_deserialize!(deserialize_bool);
    declare_deserialize!(deserialize_i8);
    declare_deserialize!(deserialize_i16);
    declare_deserialize!(deserialize_i32);
    declare_deserialize!(deserialize_i64);
    declare_deserialize!(deserialize_i128);
    declare_deserialize!(deserialize_u8);
    declare_deserialize!(deserialize_u16);
    declare_deserialize!(deserialize_u32);
    declare_deserialize!(deserialize_u64);
    declare_deserialize!(deserialize_u128);
    declare_deserialize!(deserialize_f32);
    declare_deserialize!(deserialize_f64);
    declare_deserialize!(deserialize_char);
    declare_deserialize!(deserialize_str);
    declare_deserialize!(deserialize_string);
    declare_deserialize!(deserialize_bytes);
    declare_deserialize!(deserialize_byte_buf);
    declare_deserialize!(deserialize_option);
    declare_deserialize!(deserialize_unit);
    declare_deserialize!(deserialize_seq);
    declare_deserialize!(deserialize_map);

    fn deserialize_tuple<V>(
        self,
        _len: usize,
        _visitor: V,
    ) -> std::result::Result<V::Value, NameTracerError>
    where
        V: Visitor<'de>,
    {
        Err(NameTracerError(None))
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        _visitor: V,
    ) -> std::result::Result<V::Value, NameTracerError>
    where
        V: Visitor<'de>,
    {
        Err(NameTracerError(Some(name)))
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        _visitor: V,
    ) -> std::result::Result<V::Value, NameTracerError>
    where
        V: Visitor<'de>,
    {
        Err(NameTracerError(Some(name)))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> std::result::Result<V::Value, NameTracerError>
    where
        V: Visitor<'de>,
    {
        Err(NameTracerError(Some(name)))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> std::result::Result<V::Value, NameTracerError>
    where
        V: Visitor<'de>,
    {
        Err(NameTracerError(Some(name)))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> std::result::Result<V::Value, NameTracerError>
    where
        V: Visitor<'de>,
    {
        Err(NameTracerError(Some(name)))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}
