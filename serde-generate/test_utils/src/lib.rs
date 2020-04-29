// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_reflection::{Registry, Result, Samples, Tracer, TracerConfig};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
enum SerdeData {
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
}

#[derive(Serialize, Deserialize)]
struct PrimitiveTypes {
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
    f_f32: f32,
    f_f64: f64,
    f_char: char,
}

#[derive(Serialize, Deserialize)]
struct OtherTypes {
    f_string: String,
    f_bytes: ByteBuf,
    f_option: Option<Struct>,
    f_unit: (),
    f_seq: Vec<Struct>,
    f_tuple: (u64, u32),
    f_map: BTreeMap<String, u32>,
}

#[derive(Serialize, Deserialize)]
struct UnitStruct;

#[derive(Serialize, Deserialize)]
struct NewTypeStruct(f64);

#[derive(Serialize, Deserialize)]
struct TupleStruct(f32, f64);

#[derive(Serialize, Deserialize)]
struct Struct {
    x: u32,
    y: u64,
}

#[derive(Serialize, Deserialize)]
enum List<T> {
    Empty,
    Node(T, Box<List<T>>),
}

#[derive(Serialize, Deserialize)]
struct Tree<T> {
    value: T,
    children: Vec<Tree<T>>,
}

pub fn get_registry() -> Result<Registry> {
    let mut tracer = Tracer::new(TracerConfig::default());
    let samples = Samples::new();
    tracer.trace_type::<SerdeData>(&samples)?;
    tracer.trace_type::<List<SerdeData>>(&samples)?;
    tracer.registry()
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
  NEWTYPESTRUCT: F64
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
    - f_map:
        MAP:
          KEY: STR
          VALUE: U32
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
    - f_f32: F32
    - f_f64: F64
    - f_char: CHAR
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
    - F32
    - F64
UnitStruct: UNITSTRUCT
"#
        .to_string()
    );
}
