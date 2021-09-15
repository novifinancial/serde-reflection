// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::{de::DeserializeOwned, Deserialize};
use serde_name::DeserializeNameAdapter;

#[derive(Deserialize)]
#[serde(remote = "E")]
enum E {
    Unit,
}

#[derive(Deserialize)]
#[serde(remote = "Unit")]
struct Unit;

#[derive(Deserialize)]
#[serde(remote = "NewType")]
struct NewType(u64);

#[derive(Deserialize)]
#[serde(remote = "Tuple")]
struct Tuple(u64, u32);

#[derive(Deserialize)]
#[serde(remote = "Struct")]
#[allow(dead_code)]
struct Struct {
    a: u64,
}

macro_rules! impl_deserialize {
    ($name:ident) => {
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                $name::deserialize(DeserializeNameAdapter::new(
                    deserializer,
                    std::any::type_name::<Self>(),
                ))
            }
        }
    };
}

impl_deserialize!(E);
impl_deserialize!(Unit);
impl_deserialize!(NewType);
impl_deserialize!(Tuple);
impl_deserialize!(Struct);

fn test_type<T>(expected_name: &'static str)
where
    T: DeserializeOwned,
{
    // this crate
    assert_eq!(serde_name::trace_name::<T>(), Some(expected_name));
}

#[test]
fn test_deserialize_adapter() {
    test_type::<E>("de_adapter::E");
    test_type::<Unit>("de_adapter::Unit");
    test_type::<NewType>("de_adapter::NewType");
    test_type::<Tuple>("de_adapter::Tuple");
    test_type::<Struct>("de_adapter::Struct");
}
