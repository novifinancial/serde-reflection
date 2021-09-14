// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::{de::DeserializeOwned, Deserialize};
use serde_name::DeserializeNameAdapter;

enum E {
    Unit,
}

struct Unit;

struct NewType(u64);

struct Tuple(u64, u32);

#[allow(dead_code)]
struct Struct {
    a: u64,
}

#[derive(Deserialize)]
#[serde(remote = "E")]
enum _E {
    Unit,
}

#[derive(Deserialize)]
#[serde(remote = "Unit")]
struct _Unit;

#[derive(Deserialize)]
#[serde(remote = "NewType")]
struct _NewType(u64);

#[derive(Deserialize)]
#[serde(remote = "Tuple")]
struct _Tuple(u64, u32);

#[derive(Deserialize)]
#[serde(remote = "Struct")]
#[allow(dead_code)]
struct _Struct {
    a: u64,
}

macro_rules! impl_deserialize {
    ($name:ident, $internal:ident) => {
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                $internal::deserialize(DeserializeNameAdapter::new(
                    deserializer,
                    std::any::type_name::<Self>(),
                ))
            }
        }
    };
}

impl_deserialize!(E, _E);
impl_deserialize!(Unit, _Unit);
impl_deserialize!(NewType, _NewType);
impl_deserialize!(Tuple, _Tuple);
impl_deserialize!(Struct, _Struct);

fn test_type<T>(expected_name: &'static str)
where
    T: DeserializeOwned,
{
    // this crate
    assert_eq!(serde_name::trace_name::<T>(), Some(expected_name));
}

#[test]
fn test_overriden_name() {
    test_type::<E>("de_adapter::E");
    test_type::<Unit>("de_adapter::Unit");
    test_type::<NewType>("de_adapter::NewType");
    test_type::<Tuple>("de_adapter::Tuple");
    test_type::<Struct>("de_adapter::Struct");
}
