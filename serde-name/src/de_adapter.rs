// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::de::Visitor;

/// An adapter for `serde::de::Deserializer` implementations that lets you override the
/// name of the top-level container.
pub struct DeserializeNameAdapter<D> {
    inner: D,
    name: &'static str,
}

impl<D> DeserializeNameAdapter<D> {
    pub fn new(inner: D, name: &'static str) -> Self {
        Self { inner, name }
    }
}

macro_rules! declare_deserialize {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            self.inner.$method(visitor)
        }
    };
}

impl<'de, D> serde::de::Deserializer<'de> for DeserializeNameAdapter<D>
where
    D: serde::de::Deserializer<'de>,
{
    type Error = D::Error;

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
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner.deserialize_tuple(len, visitor)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner.deserialize_unit_struct(self.name, visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner.deserialize_newtype_struct(self.name, visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner.deserialize_tuple_struct(self.name, len, visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner.deserialize_struct(self.name, fields, visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner.deserialize_enum(self.name, variants, visitor)
    }

    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }
}
