//  Copyright © Diem Association. All rights reserved.

import Foundation

protocol Deserializer {
  func deserialize_str() throws -> String
  func deserialize_bytes() throws -> [UInt8]
  func deserialize_bool() throws -> Bool
  func deserialize_unit() -> Unit
  func deserialize_char() throws -> Character
  func deserialize_f32() -> Float
  func deserialize_f64() -> Double
  func deserialize_u8() -> UInt8
  func deserialize_u16() ->UInt16
  func deserialize_u32() -> UInt32
  func deserialize_u64() -> UInt64
  func deserialize_u128() throws -> BigInt
  func deserialize_i8() -> Int8
  func deserialize_i16() -> Int16
  func deserialize_i32() -> Int
  func deserialize_i64() ->Int64
  func deserialize_i128() throws -> BigInt
  func deserialize_len() throws -> Int64
  func deserialize_variant_index() -> Int
  func deserialize_option_tag() throws -> Bool
  func increase_container_depth() throws
  func decrease_container_depth()
}
