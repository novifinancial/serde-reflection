//  Copyright © Diem Association. All rights reserved.

enum BinarySerializerError: Error {
  case serializationException(issue: String)
}

import Foundation

public class BinarySerializer : Serializer {
  let buffer: InputStream
  public var output = OutputStream.toMemory()
  private var containerDepthBudget:Int64
  
  init(maxContainerDepth:Int64) {
    buffer = InputStream()
    output.open()
    containerDepthBudget = maxContainerDepth;
  }
  
  deinit {
    output.close()
  }
  
  func increase_container_depth() throws {
    if containerDepthBudget == 0 {
      throw BinarySerializerError.serializationException(issue: "Exceeded maximum container depth")
    }
    containerDepthBudget -= 1
  }
  
  func decrease_container_depth() {
    containerDepthBudget += 1
  }
  
  func serialize_char(value: Character) throws {
    throw BinarySerializerError.serializationException(issue: "Not implemented: char serialization")
  }
  
  func serialize_f32(value: Float) {
    output.write(data: Data(fromArray:[value]))
  }
  
  func serialize_f64(value: Double) {
    output.write(data: Data(fromArray:[value]))
  }
  
  func get_bytes() -> [UInt8] {
    return output.getBuffer()
    //return [UInt8](Data(reading: buffer))
  }
  
  func serialize_str(value: String) throws {
    let buf: [UInt8] = Array(value.utf8)
    serialize_bytes(value: buf)
  }
  
  func serialize_bytes(value: [UInt8]) {
    serialize_len(value: Int64(value.count))
    for b in value {
      output.write(data: Data([b]))
    }
  }
  
  func serialize_bool(value: Bool) {
    var valueInInt = value == true ? 1 : 0
    let data = Data(bytes: &valueInInt, count: MemoryLayout.size(ofValue: valueInInt)) //Int to Data
    output.write(data: data)
  }
  
  func serialize_unit(value: Unit) {
    output.write(data: Data(fromArray:[value]))
  }
  
  func serialize_u8(value: UInt8) {
    output.write(data: Data(fromArray:[value]))
  }
  
  private func custom_write<T>(value: T) -> Void {
    var value = value
    let size = MemoryLayout.size(ofValue: value)
    withUnsafeBytes(of: &value) {
      ptr in
      output.write(ptr.baseAddress!.assumingMemoryBound(to: UInt8.self), maxLength: size)
    }
  }
  
  func serialize_u16(value: UInt16) {
    output.write(data: Data(fromArray: [value]))
  }
  
  func serialize_u32(value: UInt32) {
    output.write(data: Data(fromArray: [value]))
  }
  
  func serialize_u64(value: UInt64) {
    output.write(data: Data(fromArray: [value]))
  }
  
  func serialize_u128(value: BigInt8) throws {
    if value >> 128 != 0 {
      throw BinarySerializerError.serializationException(issue: "Invalid value for an unsigned int128")
    }
    
    assert(value._data.count <= 16 || value._data[16] == 0)
    
    for i in 0..<16 {
      if i < value._data.count {
        output.write(data: Data(fromArray: [value._data[i]]))
      } else {
        output.write(data: Data(fromArray: [UInt8(0)]))
      }
    }
  }
  
  func serialize_i8(value: Int8) {
    output.write(data: Data(fromArray: [value]))
    return print(value)
  }
  
  func serialize_i16(value: Int16) {
    output.write(data: Data(fromArray: [value]))
    return print(value)
  }
  
  func serialize_i32(value: Int32) {
    output.write(data: Data(fromArray: [value]))
    return print(value)
  }
  
  func serialize_i64(value: Int64) {
    output.write(data: Data(fromArray: [value]))
    return print(value)
  }
  
  func serialize_i128(value: BigInt8) throws {
    if value >= 0 {
      if value >> 127 != 0 {
        throw BinarySerializerError.serializationException(issue: "Invalid value for a signed int128")
      }
      try serialize_u128(value: value)
    }
    else {
      if -(value + 1) >> 127 != 0 {
        throw BinarySerializerError.serializationException(issue: "Invalid value for a signed int128")
      }
      try serialize_u128(value: value + (BigInt8(1) << 128))
    }
  }
  
  func serialize_option_tag(value: Bool) {
    output.write(data: Data(fromArray: [value]))
  }
  
  func get_buffer_offset() -> Int {
    return output.getOffset()
  }
  
  func serialize_len(value: Int64) {
    output.write(data: Data(fromArray: [value]))
  }
  
  func serialize_variant_index(value: Int) {
    output.write(data: Data(fromArray: [value]))
    print(value)
  }
  
  func sort_map_entries(offsets: [Int]) {
    // Not required by the format.
  }
  
  private func write<T>(value: T) {
    var value = value
    let size = MemoryLayout.size(ofValue: value)
    withUnsafeBytes(of: &value) {
      ptr in
      output.write(ptr.baseAddress!.assumingMemoryBound(to: UInt8.self), maxLength: size)
    }
  }
}
