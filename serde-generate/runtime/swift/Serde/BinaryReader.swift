//
//  BinaryReader.swift
//  UnityPack-Swift
//
//  Created by Istvan Fehervari on 05/01/2017.
//  Copyright © 2017 Benjamin Michotte. All rights reserved.
//
// Retrieved from : https://github.com/HearthSim/UnityPack-Swift/tree/master/Sources

import Foundation

public typealias Byte = UInt8

extension Data {
  
  func toByteArray() -> [Byte] {
    let count = self.count / MemoryLayout<Byte>.size
    var array = [Byte](repeating: 0, count: count)
    copyBytes(to: &array, count:count * MemoryLayout<Byte>.size)
    return array
  }
}

public protocol Readable {
  func readBytes(count: Int) -> [UInt8]
  func seek(count: Int, whence: Int)
  var tell: Int { get }
}

class UPData : Readable {
  var location: Int = 0
  var data: Data
  
  init(withData data: Data) {
    self.data = data
  }
  
  func readBytes(count: Int) -> [UInt8] {
    if location >= data.count {
      return [UInt8]()
    }
    
    let startIndex = location
    let endIndex = location + count
    
    var bytes = [UInt8](repeating:0, count: count)
    data.copyBytes(to: &bytes, from: startIndex..<endIndex)
    
    location += count
    return bytes
  }
  
  var tell: Int { return location }
  
  func seek(count: Int, whence: Int = 0) {
    location = count
  }
}

class FileData : Readable {
  let fileHandle: FileHandle
  
  init(withFileHandle fileHandle: FileHandle) {
    self.fileHandle = fileHandle
  }
  
  func readBytes(count: Int) -> [UInt8] {
    return fileHandle.readData(ofLength: count).toByteArray()
  }
  
  var tell: Int {return Int(fileHandle.offsetInFile)}
  
  func seek(count: Int, whence: Int = 0) {
    fileHandle.seek(toFileOffset: UInt64(count))
  }
}

public enum ByteOrder {
  case bigEndian
  case littleEndian
  
  /// Machine specific byte order
  static let nativeByteOrder: ByteOrder = (Int(CFByteOrderGetCurrent()) == Int(CFByteOrderLittleEndian.rawValue)) ? .littleEndian : .bigEndian
}

public class BinaryReader: Readable {
  
  public var tell: Int { return buffer.tell }
  
  public func seek(count: Int, whence: Int) {
    buffer.seek(count: count, whence: whence)
  }
  
  var buffer: Readable
  var endianness: ByteOrder = ByteOrder.littleEndian
  
  init(data: Readable) {
    self.buffer = data
  }
  
  public func readBytes(count: Int) -> [UInt8] {
    return buffer.readBytes(count: count)
  }
  
  func seek(count: Int32) {
    buffer.seek(count: Int(count), whence: 0)
  }
  
  func align() {
    let old = self.tell
    let new = (old + 3) & -4
    if new > old {
      self.seek(count: Int32(new))
    }
  }
  
  func readUInt8() -> UInt8 {
    let bytes = readBytes(count: 1)
    return bytes[0]
  }
  
  func readInt8() -> Int8 {
    let bytes = readBytes(count: 1)
    return Int8(bitPattern: bytes[0])
  }
  
  
  func readBool() -> Bool {
    let byte = readBytes(count: 1)[0]
    return byte != 0
  }
  
  func readInt() -> Int32 {
    return self.readInt(byteOrder: self.endianness)
  }
  
  func readInt(byteOrder: ByteOrder) -> Int32 {
    let b = buffer.readBytes(count: 4)
    let int: Int32 = BinaryReader.fromByteArray(b, Int32.self, byteOrder: byteOrder)
    return int
  }
  
  func readInt16() -> Int16 {
    return self.readInt16(byteOrder: self.endianness)
  }
  
  func readInt16(byteOrder: ByteOrder) -> Int16 {
    let b = buffer.readBytes(count: 2)
    let int: Int16 = BinaryReader.fromByteArray(b, Int16.self, byteOrder: byteOrder)
    return int
  }
  
  func readInt64() -> Int64 {
    return self.readInt64(byteOrder: self.endianness)
  }
  
  func readInt64(byteOrder: ByteOrder) -> Int64 {
    let b = buffer.readBytes(count: 8)
    let int: Int64 = BinaryReader.fromByteArray(b, Int64.self, byteOrder: byteOrder)
    return int
  }
  
  func readUInt() -> UInt32 {
    return self.readUInt(byteOrder: self.endianness)
  }
  
  func readUInt(byteOrder: ByteOrder) -> UInt32 {
    let b = buffer.readBytes(count: 4)
    let int: UInt32 = BinaryReader.fromByteArray(b, UInt32.self, byteOrder: byteOrder)
    return int
  }
  
  // added
  func readUInt16() -> UInt16 {
    return self.readUInt16(byteOrder: self.endianness)
  }
  
  func readUInt16(byteOrder: ByteOrder) -> UInt16 {
    let b = buffer.readBytes(count: 2)
    let int: UInt16 = BinaryReader.fromByteArray(b, UInt16.self, byteOrder: byteOrder)
    return int
  }
  func readUInt64() -> UInt64 {
    return self.readUInt64(byteOrder: self.endianness)
  }
  
  func readUInt64(byteOrder: ByteOrder) -> UInt64 {
    let b = buffer.readBytes(count: 8)
    let int: UInt64 = BinaryReader.fromByteArray(b, UInt64.self, byteOrder: byteOrder)
    return int
  }
  
  func readFloat() -> Float {
    let bytes = buffer.readBytes(count: 4)
    var f:Float = 0.0
    
    memcpy(&f, bytes, 4)
    return f
  }
  
  // added
  func readDouble() -> Double {
    let bytes = buffer.readBytes(count: 8)
    var d:Double = 0.0
    
    memcpy(&d, bytes, 8)
    return d
  }
  
  func readString(size: Int = -1) -> String {
    var bytes:[UInt8] = []
    
    if size >= 0 {
      bytes = readBytes(count: Int(size))
    } else {
      while true {
        if let byte = readBytes(count: 1).first {
          if UInt32(byte) == ("\0" as UnicodeScalar).value {
            break
          }
          bytes.append(byte)
        } else {
          break
        }
      }
    }
    
    //print("Bytes: \(bytes)")
    //print("\(MemoryLayout<String>.size)")
    //let bytes = readBytes(count: 8)
    let string = String(bytes: bytes, encoding: .utf8)?
      .filter { $0 != "\0" }
      .map { String($0) }
      .joined()
    return string ?? ""
  }
  
  static func toByteArray<T>(_ value: T) -> [UInt8] {
    var value = value
    return withUnsafeBytes(of: &value) { Array($0) }
  }
  
  static func fromByteArray<T>(_ value: [UInt8], _: T.Type, byteOrder: ByteOrder = ByteOrder.nativeByteOrder) -> T {
    let bytes: [UInt8] = (byteOrder == .littleEndian) ? value : value.reversed()
    return bytes.withUnsafeBufferPointer {
      return $0.baseAddress!.withMemoryRebound(to: T.self, capacity: 1) {
        $0.pointee
      }
    }
  }
}
