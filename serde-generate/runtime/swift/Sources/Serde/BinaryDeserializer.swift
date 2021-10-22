//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

enum BinaryDeserializerError: Error {
    case deserializationException(issue: String)
}

public class BinaryDeserializer: Deserializer {
    let reader: BinaryReader
    fileprivate let input: [UInt8]
    fileprivate var containerDepthBudget: Int64

    init(input: [UInt8], maxContainerDepth: Int64) {
        self.input = input
        reader = BinaryReader(data: UPData(withData: Data(input)) as Readable)
        containerDepthBudget = maxContainerDepth
    }

    public func deserialize_len() throws -> Int64 {
        let value: Int64 = reader.readInt64()
        if value < 0 || value > Int.max {
            throw BincodeDeserializerError.bincodeDeserializerException(issue: "Incorrect length value")
        }
        return value
    }

    public func deserialize_variant_index() -> Int {
        return Int(reader.readInt())
    }

    public func deserialize_char() throws -> Character {
        throw BinaryDeserializerError.deserializationException(issue: "Not implemented: char deserialization")
    }

    public func deserialize_f32() -> Float {
        return reader.readFloat()
    }

    public func deserialize_f64() -> Double {
        return reader.readDouble()
    }

    func increase_container_depth() throws {
        if containerDepthBudget == 0 {
            throw BinaryDeserializerError.deserializationException(issue: "Exceeded maximum container depth")
        }
        containerDepthBudget -= 1
    }

    func decrease_container_depth() {
        containerDepthBudget += 1
    }

    public func deserialize_str() throws -> String {
        let len: Int64 = try deserialize_len()
        if len < 0 || len > Int.max {
            throw BinaryDeserializerError.deserializationException(issue: "Incorrect length value for Swift string")
        }
        let content: [UInt8] = reader.readBytes(count: (Int)(len))
        if content.count < len {
            throw BinaryDeserializerError.deserializationException(issue: "Need len - \(content.count) more bytes for string")
        }
        return String(bytes: content, encoding: .utf8)!
    }

    public func deserialize_bytes() throws -> [UInt8] {
        let len: Int64 = try deserialize_len()
        if len < 0 || len > Int.max {
            throw BinaryDeserializerError.deserializationException(issue: "Incorrect length value for Swift array")
        }
        let content: [UInt8] = reader.readBytes(count: (Int)(len))
        if content.count < len {
            throw BinaryDeserializerError.deserializationException(issue: "Need  \(len) - \(content.count) more bytes for byte array")
        }
        return content
    }

    public func deserialize_bool() throws -> Bool {
        return reader.readBool()
    }

    public func deserialize_unit() -> Unit {
        return Unit()
    }

    public func deserialize_u8() -> UInt8 {
        return reader.readUInt8()
    }

    public func deserialize_u16() -> UInt16 {
        return reader.readUInt16()
    }

    public func deserialize_u32() -> UInt32 {
        return reader.readUInt()
    }

    public func deserialize_u64() -> UInt64 {
        return reader.readUInt64()
    }

    public func deserialize_u128() throws -> BigInt8 {
        let signed: BigInt8 = try deserialize_i128()
        if signed >= 0 {
            return signed
        } else {
            return signed + (BigInt8(1) << 128)
        }
    }

    public func deserialize_i8() -> Int8 {
        return reader.readInt8()
    }

    public func deserialize_i16() -> Int16 {
        return reader.readInt16()
    }

    public func deserialize_i32() -> Int {
        return Int(reader.readInt())
    }

    public func deserialize_i64() -> Int64 {
        return reader.readInt64()
    }

    public func deserialize_i128() throws -> BigInt8 {
        let content: [UInt8] = reader.readBytes(count: 16)
        if content.count < 16 {
            throw BinaryDeserializerError.deserializationException(issue: "Need more bytes to deserialize 128-bit integer")
        }
        return BigInt8(content.map { UInt8($0) })
    }

    public func deserialize_option_tag() throws -> Bool {
        let value: UInt8 = reader.readUInt8()
        switch value {
        case 0: return false
        case 1: return true
        default: throw BinaryDeserializerError.deserializationException(issue: "Incorrect value for Option tag: \(value)")
        }
    }
}
