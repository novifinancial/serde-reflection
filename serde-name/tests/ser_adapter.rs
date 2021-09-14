// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::Serialize;
use serde_name::SerializeNameAdapter;
use serde_reflection::{Format, FormatHolder, Samples, Tracer, TracerConfig};

enum E {
    Unit,
}

struct Unit;

struct NewType(u64);

struct Tuple(u64, u32);

struct Struct {
    a: u64,
}

#[derive(Serialize)]
#[serde(remote = "E")]
enum _E {
    Unit,
}

#[derive(Serialize)]
#[serde(remote = "Unit")]
struct _Unit;

#[derive(Serialize)]
#[serde(remote = "NewType")]
struct _NewType(u64);

#[derive(Serialize)]
#[serde(remote = "Tuple")]
struct _Tuple(u64, u32);

#[derive(Serialize)]
#[serde(remote = "Struct")]
#[allow(dead_code)]
struct _Struct {
    a: u64,
}

macro_rules! impl_serialize {
    ($name:ident, $internal:ident) => {
        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                $internal::serialize(
                    self,
                    SerializeNameAdapter::new(serializer, std::any::type_name::<Self>()),
                )
            }
        }
    };
}

impl_serialize!(E, _E);
impl_serialize!(Unit, _Unit);
impl_serialize!(NewType, _NewType);
impl_serialize!(Tuple, _Tuple);
impl_serialize!(Struct, _Struct);

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
fn test_overriden_name() {
    test_type::<E>(&E::Unit, "ser_adapter::E");
    test_type::<Unit>(&Unit, "ser_adapter::Unit");
    test_type::<NewType>(&NewType(3), "ser_adapter::NewType");
    test_type::<Tuple>(&Tuple(1, 2), "ser_adapter::Tuple");
    test_type::<Struct>(&Struct { a: 3 }, "ser_adapter::Struct");
}
