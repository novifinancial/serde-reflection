// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::Encoding;
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_reflection::{Registry, Result, Samples, Tracer, TracerConfig};
use std::collections::BTreeMap;

// Simple data formats used to create and test values in each language.
#[derive(Serialize, Deserialize)]
pub struct Test {
    pub a: Vec<u32>,
    pub b: (i64, u64),
    pub c: Choice,
}

#[derive(Serialize, Deserialize)]
pub enum Choice {
    A,
    B(u64),
    C { x: u8 },
}

pub fn get_simple_registry() -> Result<Registry> {
    let mut tracer = Tracer::new(TracerConfig::default());
    let samples = Samples::new();
    tracer.trace_type::<Test>(&samples)?;
    tracer.trace_type::<Choice>(&samples)?;
    Ok(tracer.registry()?)
}

// More complex data format used to test re-serialization and basic fuzzing.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SerdeData {
    PrimitiveTypes(PrimitiveTypes),
    OtherTypes(OtherTypes),
    UnitVariant,
    NewTypeVariant(String),
    TupleVariant(u32, u64),
    StructVariant {
        f0: UnitStruct,
        f1: NewTypeStruct,
        f2: TupleStruct,
        f3: Struct,
    },
    ListWithMutualRecursion(List<Box<SerdeData>>),
    TreeWithMutualRecursion(Tree<Box<SerdeData>>),
    TupleArray([u32; 3]),
    UnitVector(Vec<()>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PrimitiveTypes {
    f_bool: bool,
    f_u8: u8,
    f_u16: u16,
    f_u32: u32,
    f_u64: u64,
    f_u128: u128,
    f_i8: i8,
    f_i16: i16,
    f_i32: i32,
    f_i64: i64,
    f_i128: i128,
    // The following types are not supported by our bincode and LCS runtimes, therefore
    // we don't populate them for testing.
    f_f32: Option<f32>,
    f_f64: Option<f64>,
    f_char: Option<char>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct OtherTypes {
    f_string: String,
    f_bytes: ByteBuf,
    f_option: Option<Struct>,
    f_unit: (),
    f_seq: Vec<Struct>,
    f_tuple: (u64, u32),
    f_stringmap: BTreeMap<String, u32>,
    f_intset: BTreeMap<u64, ()>, // Avoiding BTreeSet because Serde treats them as sequences.
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UnitStruct;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct NewTypeStruct(u64);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TupleStruct(u32, u64);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Struct {
    x: u32,
    y: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum List<T> {
    Empty,
    Node(T, Box<List<T>>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Tree<T> {
    value: T,
    children: Vec<Tree<T>>,
}

/// The registry corresponding to the test data structures above .
pub fn get_registry() -> Result<Registry> {
    let mut tracer = Tracer::new(TracerConfig::default());
    let samples = Samples::new();
    tracer.trace_type::<SerdeData>(&samples)?;
    tracer.trace_type::<List<SerdeData>>(&samples)?;
    tracer.registry()
}

/// Manually generate sample values.
/// Avoid maps with more than one element when `has_canonical_maps` is false so that
/// we can test re-serialization.
pub fn get_sample_values(has_canonical_maps: bool) -> Vec<SerdeData> {
    let v0 = SerdeData::PrimitiveTypes(PrimitiveTypes {
        f_bool: false,
        f_u8: 6,
        f_u16: 5,
        f_u32: 4,
        f_u64: 3,
        f_u128: 2,
        f_i8: 1,
        f_i16: 0,
        f_i32: -1,
        f_i64: -2,
        f_i128: -3,
        f_f32: None,
        f_f64: None,
        f_char: None,
    });

    let v1 = SerdeData::PrimitiveTypes(PrimitiveTypes {
        f_bool: true,
        f_u8: u8::MAX,
        f_u16: u16::MAX,
        f_u32: u32::MAX,
        f_u64: u64::MAX,
        f_u128: u128::MAX,
        f_i8: i8::MIN,
        f_i16: i16::MIN,
        f_i32: i32::MIN,
        f_i64: i64::MIN,
        f_i128: i128::MIN,
        f_f32: None,
        f_f64: None,
        f_char: None,
    });

    let v2 = SerdeData::OtherTypes(OtherTypes {
        f_string: "testing...\u{00a2}\u{0939}\u{20ac}\u{d55c}\u{10348}...".to_string(),
        f_bytes: ByteBuf::from(b"bytes".to_vec()),
        f_option: Some(Struct { x: 2, y: 3 }),
        f_unit: (),
        f_seq: vec![Struct { x: 1, y: 3 }],
        f_tuple: (4, 5),
        f_stringmap: if has_canonical_maps {
            btreemap! {"foo".to_string() => 1, "bar".to_string() => 2}
        } else {
            btreemap! {"foo".to_string() => 1}
        },
        f_intset: if has_canonical_maps {
            btreemap! {1 => (), 5 => (), 16 => (), 64 => (), 257 => (), 1024 => ()}
        } else {
            btreemap! {64 => ()}
        },
    });

    let v2bis = SerdeData::OtherTypes(OtherTypes {
        f_string: "".to_string(),
        f_bytes: ByteBuf::from(b"".to_vec()),
        f_option: None,
        f_unit: (),
        f_seq: Vec::new(),
        f_tuple: (4, 5),
        f_stringmap: BTreeMap::new(),
        f_intset: BTreeMap::new(),
    });

    let v2ter = SerdeData::OtherTypes(OtherTypes {
        f_string: vec!["1"; 1000].join(""),
        f_bytes: ByteBuf::from(vec![1u8; 300]),
        f_option: None,
        f_unit: (),
        f_seq: Vec::new(),
        f_tuple: (4, 5),
        f_stringmap: BTreeMap::new(),
        f_intset: if has_canonical_maps {
            std::iter::repeat(())
                .take(200)
                .enumerate()
                .map(|(i, ())| (i as u64, ()))
                .collect()
        } else {
            BTreeMap::new()
        },
    });

    let v3 = SerdeData::UnitVariant;

    let v4 = SerdeData::NewTypeVariant("test".to_string());

    let v5 = SerdeData::TupleVariant(3, 6);

    let v6 = SerdeData::StructVariant {
        f0: UnitStruct,
        f1: NewTypeStruct(1),
        f2: TupleStruct(2, 3),
        f3: Struct { x: 4, y: 5 },
    };

    let v7 = SerdeData::ListWithMutualRecursion(List::Empty);

    let v8 = SerdeData::TreeWithMutualRecursion(Tree {
        value: Box::new(SerdeData::PrimitiveTypes(PrimitiveTypes {
            f_bool: false,
            f_u8: 0,
            f_u16: 1,
            f_u32: 2,
            f_u64: 3,
            f_u128: 4,
            f_i8: 5,
            f_i16: 6,
            f_i32: 7,
            f_i64: 8,
            f_i128: 9,
            f_f32: None,
            f_f64: None,
            f_char: None,
        })),
        children: vec![Tree {
            value: Box::new(SerdeData::PrimitiveTypes(PrimitiveTypes {
                f_bool: false,
                f_u8: 0,
                f_u16: 0,
                f_u32: 0,
                f_u64: 0,
                f_u128: 0,
                f_i8: 0,
                f_i16: 0,
                f_i32: 0,
                f_i64: 0,
                f_i128: 0,
                f_f32: None,
                f_f64: None,
                f_char: None,
            })),
            children: vec![],
        }],
    });

    let v9 = SerdeData::TupleArray([0, 2, 3]);

    let v10 = SerdeData::UnitVector(vec![(); 1000]);

    vec![v0, v1, v2, v2bis, v2ter, v3, v4, v5, v6, v7, v8, v9, v10]
}

/// Structure used to factorize code in runtime tests.
#[derive(Copy, Clone)]
pub enum Runtime {
    Lcs,
    Bincode,
}

impl std::convert::Into<Encoding> for Runtime {
    fn into(self) -> Encoding {
        match self {
            Runtime::Lcs => Encoding::Lcs,
            Runtime::Bincode => Encoding::Bincode,
        }
    }
}

impl Runtime {
    pub fn name(self) -> &'static str {
        <Self as std::convert::Into<Encoding>>::into(self).name()
    }

    pub fn rust_package(self) -> &'static str {
        match self {
            Self::Lcs => "lcs = { git = \"https://github.com/libra/libra.git\", branch = \"testnet\", package = \"libra-canonical-serialization\" }",
            Self::Bincode => "bincode = \"1.2\"",
        }
    }

    #[cfg(feature = "runtime-testing")]
    pub fn serialize<T>(self, value: &T) -> Vec<u8>
    where
        T: serde::Serialize,
    {
        match self {
            Self::Lcs => libra_canonical_serialization::to_bytes(value).unwrap(),
            Self::Bincode => bincode::serialize(value).unwrap(),
        }
    }

    pub fn quote_serialize(self) -> &'static str {
        match self {
            Self::Lcs => "lcs::to_bytes",
            Self::Bincode => "bincode::serialize",
        }
    }

    pub fn quote_deserialize(self) -> &'static str {
        match self {
            Self::Lcs => "lcs::from_bytes",
            Self::Bincode => "bincode::deserialize",
        }
    }

    /// Whether the encoding enforces ordering of map keys.
    /// Note that both encodings are canonical on other data structures.
    pub fn has_canonical_maps(self) -> bool {
        match self {
            Self::Lcs => true,
            Self::Bincode => false,
        }
    }

    pub fn maximum_length(self) -> Option<usize> {
        match self {
            Self::Lcs => Some(1 << 31),
            Self::Bincode => None,
        }
    }

    pub fn maximum_container_depth(self) -> Option<usize> {
        match self {
            Self::Lcs => Some(500),
            Self::Bincode => None,
        }
    }
}

#[test]
fn test_get_simple_registry() {
    let registry = get_simple_registry().unwrap();
    assert_eq!(
        serde_yaml::to_string(&registry).unwrap() + "\n",
        r#"---
Choice:
  ENUM:
    0:
      A: UNIT
    1:
      B:
        NEWTYPE: U64
    2:
      C:
        STRUCT:
          - x: U8
Test:
  STRUCT:
    - a:
        SEQ: U32
    - b:
        TUPLE:
          - I64
          - U64
    - c:
        TYPENAME: Choice
"#
    );
}

#[test]
fn test_get_registry() {
    let registry = get_registry().unwrap();
    assert_eq!(
        serde_yaml::to_string(&registry).unwrap() + "\n",
        r#"---
List:
  ENUM:
    0:
      Empty: UNIT
    1:
      Node:
        TUPLE:
          - TYPENAME: SerdeData
          - TYPENAME: List
NewTypeStruct:
  NEWTYPESTRUCT: U64
OtherTypes:
  STRUCT:
    - f_string: STR
    - f_bytes: BYTES
    - f_option:
        OPTION:
          TYPENAME: Struct
    - f_unit: UNIT
    - f_seq:
        SEQ:
          TYPENAME: Struct
    - f_tuple:
        TUPLE:
          - U64
          - U32
    - f_stringmap:
        MAP:
          KEY: STR
          VALUE: U32
    - f_intset:
        MAP:
          KEY: U64
          VALUE: UNIT
PrimitiveTypes:
  STRUCT:
    - f_bool: BOOL
    - f_u8: U8
    - f_u16: U16
    - f_u32: U32
    - f_u64: U64
    - f_u128: U128
    - f_i8: I8
    - f_i16: I16
    - f_i32: I32
    - f_i64: I64
    - f_i128: I128
    - f_f32:
        OPTION: F32
    - f_f64:
        OPTION: F64
    - f_char:
        OPTION: CHAR
SerdeData:
  ENUM:
    0:
      PrimitiveTypes:
        NEWTYPE:
          TYPENAME: PrimitiveTypes
    1:
      OtherTypes:
        NEWTYPE:
          TYPENAME: OtherTypes
    2:
      UnitVariant: UNIT
    3:
      NewTypeVariant:
        NEWTYPE: STR
    4:
      TupleVariant:
        TUPLE:
          - U32
          - U64
    5:
      StructVariant:
        STRUCT:
          - f0:
              TYPENAME: UnitStruct
          - f1:
              TYPENAME: NewTypeStruct
          - f2:
              TYPENAME: TupleStruct
          - f3:
              TYPENAME: Struct
    6:
      ListWithMutualRecursion:
        NEWTYPE:
          TYPENAME: List
    7:
      TreeWithMutualRecursion:
        NEWTYPE:
          TYPENAME: Tree
    8:
      TupleArray:
        NEWTYPE:
          TUPLEARRAY:
            CONTENT: U32
            SIZE: 3
    9:
      UnitVector:
        NEWTYPE:
          SEQ: UNIT
Struct:
  STRUCT:
    - x: U32
    - y: U64
Tree:
  STRUCT:
    - value:
        TYPENAME: SerdeData
    - children:
        SEQ:
          TYPENAME: Tree
TupleStruct:
  TUPLESTRUCT:
    - U32
    - U64
UnitStruct: UNITSTRUCT
"#
        .to_string()
    );
}
