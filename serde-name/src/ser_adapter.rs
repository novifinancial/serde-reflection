// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::Serialize;

/// An adapter for `serde::ser::Serializer` implementations that lets you override the
/// name of the top-level container.
pub struct SerializeNameAdapter<S> {
    inner: S,
    name: &'static str,
}

impl<S> SerializeNameAdapter<S> {
    pub fn new(inner: S, name: &'static str) -> Self {
        Self { inner, name }
    }
}

macro_rules! declare_serialize {
    ($method:ident, $type:ty) => {
        fn $method(self, content: $type) -> Result<Self::Ok, Self::Error> {
            self.inner.$method(content)
        }
    };
}

impl<S> serde::ser::Serializer for SerializeNameAdapter<S>
where
    S: serde::ser::Serializer,
{
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = S::SerializeSeq;
    type SerializeTuple = S::SerializeTuple;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = S::SerializeTupleVariant;
    type SerializeMap = S::SerializeMap;
    type SerializeStruct = S::SerializeStruct;
    type SerializeStructVariant = S::SerializeStructVariant;

    declare_serialize!(serialize_bool, bool);
    declare_serialize!(serialize_i8, i8);
    declare_serialize!(serialize_i16, i16);
    declare_serialize!(serialize_i32, i32);
    declare_serialize!(serialize_i64, i64);
    declare_serialize!(serialize_i128, i128);
    declare_serialize!(serialize_u8, u8);
    declare_serialize!(serialize_u16, u16);
    declare_serialize!(serialize_u32, u32);
    declare_serialize!(serialize_u64, u64);
    declare_serialize!(serialize_u128, u128);
    declare_serialize!(serialize_f32, f32);
    declare_serialize!(serialize_f64, f64);
    declare_serialize!(serialize_char, char);
    declare_serialize!(serialize_str, &str);
    declare_serialize!(serialize_bytes, &[u8]);

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_none()
    }

    fn serialize_some<T>(self, content: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.inner.serialize_some(content)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit_struct(self.name)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.inner
            .serialize_unit_variant(self.name, variant_index, variant_name)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        content: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.inner.serialize_newtype_struct(self.name, content)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        content: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.inner
            .serialize_newtype_variant(self.name, variant_index, variant_name, content)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.inner.serialize_seq(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.inner.serialize_tuple(len)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.inner.serialize_tuple_struct(self.name, len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.inner
            .serialize_tuple_variant(self.name, variant_index, variant_name, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.inner.serialize_map(len)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.inner.serialize_struct(self.name, len)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.inner
            .serialize_struct_variant(self.name, variant_index, variant_name, len)
    }

    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }
}
