//
//  ByteBuffer.swift
//
// https://stackoverflow.com/questions/53190176/java-bytebuffer-equivalent-in-swift-4

import Foundation

public class ByteBuffer {
  
  public init(size: Int) {
    array.reserveCapacity(size)
  }
  
  public func allocate(_ size: Int) {
    array = [UInt8]()
    array.reserveCapacity(size)
    currentIndex = 0
  }
  
  public func nativeByteOrder() -> Endianness {
    return hostEndianness
  }
  
  public func currentByteOrder() -> Endianness {
    return currentEndianness
  }
  
  public func order(_ endianness: Endianness) -> ByteBuffer {
    currentEndianness = endianness
    return self
  }
  
  public func put(_ value: UInt8) -> ByteBuffer {
    array.append(value)
    return self
  }
  
  public func put(_ value: Int32) -> ByteBuffer {
    if currentEndianness == .little {
      array.append(contentsOf: to(value.littleEndian))
      return self
    }
    
    array.append(contentsOf: to(value.bigEndian))
    return self
  }
  
  public func put(_ value: Int64) -> ByteBuffer {
    if currentEndianness == .little {
      array.append(contentsOf: to(value.littleEndian))
      return self
    }
    
    array.append(contentsOf: to(value.bigEndian))
    return self
  }
  
  public func put(_ value: Int) -> ByteBuffer {
    if currentEndianness == .little {
      array.append(contentsOf: to(value.littleEndian))
      return self
    }
    
    array.append(contentsOf: to(value.bigEndian))
    return self
  }
  
  public func put(_ value: Float) -> ByteBuffer {
    if currentEndianness == .little {
      array.append(contentsOf: to(value.bitPattern.littleEndian))
      return self
    }
    
    array.append(contentsOf: to(value.bitPattern.bigEndian))
    return self
  }
  
  public func put(_ value: Double) -> ByteBuffer {
    if currentEndianness == .little {
      array.append(contentsOf: to(value.bitPattern.littleEndian))
      return self
    }
    
    array.append(contentsOf: to(value.bitPattern.bigEndian))
    return self
  }
  
  public func get() -> UInt8 {
    let result = array[currentIndex]
    currentIndex += 1
    return result
  }
  
  public func get(_ index: Int) -> UInt8 {
    return array[index]
  }
  
  public func getInt32() -> Int32 {
    let result = from(Array(array[currentIndex..<currentIndex + MemoryLayout<Int32>.size]), Int32.self)
    currentIndex += MemoryLayout<Int32>.size
    return currentEndianness == .little ? result.littleEndian : result.bigEndian
  }
  
  public func getInt32(_ index: Int) -> Int32 {
    let result = from(Array(array[index..<index + MemoryLayout<Int32>.size]), Int32.self)
    return currentEndianness == .little ? result.littleEndian : result.bigEndian
  }
  
  public func getInt64() -> Int64 {
    let result = from(Array(array[currentIndex..<currentIndex + MemoryLayout<Int64>.size]), Int64.self)
    currentIndex += MemoryLayout<Int64>.size
    return currentEndianness == .little ? result.littleEndian : result.bigEndian
  }
  
  public func getInt64(_ index: Int) -> Int64 {
    let result = from(Array(array[index..<index + MemoryLayout<Int64>.size]), Int64.self)
    return currentEndianness == .little ? result.littleEndian : result.bigEndian
  }
  
  public func getInt() -> Int {
    let result = from(Array(array[currentIndex..<currentIndex + MemoryLayout<Int>.size]), Int.self)
    currentIndex += MemoryLayout<Int>.size
    return currentEndianness == .little ? result.littleEndian : result.bigEndian
  }
  
  public func getInt(_ index: Int) -> Int {
    let result = from(Array(array[index..<index + MemoryLayout<Int>.size]), Int.self)
    return currentEndianness == .little ? result.littleEndian : result.bigEndian
  }
  
  public func getFloat() -> Float {
    let result = from(Array(array[currentIndex..<currentIndex + MemoryLayout<UInt32>.size]), UInt32.self)
    currentIndex += MemoryLayout<UInt32>.size
    return currentEndianness == .little ? Float(bitPattern: result.littleEndian) : Float(bitPattern: result.bigEndian)
  }
  
  public func getFloat(_ index: Int) -> Float {
    let result = from(Array(array[index..<index + MemoryLayout<UInt32>.size]), UInt32.self)
    return currentEndianness == .little ? Float(bitPattern: result.littleEndian) : Float(bitPattern: result.bigEndian)
  }
  
  public func getDouble() -> Double {
    let result = from(Array(array[currentIndex..<currentIndex + MemoryLayout<UInt64>.size]), UInt64.self)
    currentIndex += MemoryLayout<UInt64>.size
    return currentEndianness == .little ? Double(bitPattern: result.littleEndian) : Double(bitPattern: result.bigEndian)
  }
  
  public func getDouble(_ index: Int) -> Double {
    let result = from(Array(array[index..<index + MemoryLayout<UInt64>.size]), UInt64.self)
    return currentEndianness == .little ? Double(bitPattern: result.littleEndian) : Double(bitPattern: result.bigEndian)
  }
  
  
  public enum Endianness {
    case little
    case big
  }
  
  private func to<T>(_ value: T) -> [UInt8] {
    var value = value
    return withUnsafeBytes(of: &value, Array.init)
  }
  
  private func from<T>(_ value: [UInt8], _: T.Type) -> T {
    return value.withUnsafeBytes {
      $0.load(fromByteOffset: 0, as: T.self)
    }
  }
  
  private var array = [UInt8]()
  private var currentIndex: Int = 0
  
  private var currentEndianness: Endianness = .big
  private let hostEndianness: Endianness = OSHostByteOrder() == OSLittleEndian ? .little : .big
}
