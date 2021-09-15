// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::Serialize;
use serde_name::SerializeNameAdapter;
use serde_reflection::{Format, FormatHolder, Samples, Tracer, TracerConfig};

#[derive(Serialize)]
#[serde(remote = "E")]
enum E {
    Unit,
}

#[derive(Serialize)]
#[serde(remote = "Unit")]
struct Unit;

#[derive(Serialize)]
#[serde(remote = "NewType")]
struct NewType(u64);

#[derive(Serialize)]
#[serde(remote = "Tuple")]
struct Tuple(u64, u32);

#[derive(Serialize)]
#[serde(remote = "Struct")]
#[allow(dead_code)]
struct Struct {
    a: u64,
}

macro_rules! impl_serialize {
    ($name:ident) => {
        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                $name::serialize(
                    self,
                    SerializeNameAdapter::new(serializer, std::any::type_name::<Self>()),
                )
            }
        }
    };
}

impl_serialize!(E);
impl_serialize!(Unit);
impl_serialize!(NewType);
impl_serialize!(Tuple);
impl_serialize!(Struct);

fn test_type<T>(value: &T, expected_name: &'static str)
where
    T: Serialize,
{
    let mut tracer = Tracer::new(TracerConfig::default());
    let mut samples = Samples::new();
    let (mut ident, _) = tracer.trace_value(&mut samples, value).unwrap();
    ident.normalize().unwrap();
    assert_eq!(ident, Format::TypeName(expected_name.into()));
}

#[test]
fn test_serialize_adapter() {
    test_type::<E>(&E::Unit, "ser_adapter::E");
    test_type::<Unit>(&Unit, "ser_adapter::Unit");
    test_type::<NewType>(&NewType(3), "ser_adapter::NewType");
    test_type::<Tuple>(&Tuple(1, 2), "ser_adapter::Tuple");
    test_type::<Struct>(&Struct { a: 3 }, "ser_adapter::Struct");
}
