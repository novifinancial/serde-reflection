//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public enum BinarySerializerError: Error {
    case invalidValue(issue: String)
}

public class BinarySerializer: Serializer {
    public var output = OutputStream.toMemory()
    private var containerDepthBudget: Int64

    public init(maxContainerDepth: Int64) {
        output.open()
        containerDepthBudget = maxContainerDepth
    }

    deinit {
        output.close()
    }

    public func increase_container_depth() throws {
        if containerDepthBudget == 0 {
            throw BinarySerializerError.invalidValue(issue: "Exceeded maximum container depth")
        }
        containerDepthBudget -= 1
    }

    public func decrease_container_depth() {
        containerDepthBudget += 1
    }

    public func serialize_char(value _: Character) throws {
        throw BinarySerializerError.invalidValue(issue: "Not implemented: char serialization")
    }

    public func serialize_f32(value: Float) throws {
        try serialize_u32(value: value.bitPattern)
    }

    public func serialize_f64(value: Double) throws {
        try serialize_u64(value: value.bitPattern)
    }

    public func get_bytes() -> [UInt8] {
        return output.getBuffer()
    }

    public func serialize_str(value: String) throws {
        try serialize_bytes(value: Array(value.utf8))
    }

    public func serialize_bytes(value: [UInt8]) throws {
        try serialize_len(value: Int64(value.count))
        output.write(data: Data(value))
    }

    public func serialize_bool(value: Bool) throws {
        try writeByte(value ? 1 : 0)
    }

    public func serialize_unit(value _: Unit) throws {}

    func writeByte(_ value: UInt8) throws {
        if output.write(data: Data([value])) != 1 {
            throw BinarySerializerError.invalidValue(issue: "Error while outputting byte")
        }
    }

    public func serialize_u8(value: UInt8) throws {
        try writeByte(value)
    }

    public func serialize_u16(value: UInt16) throws {
        try writeByte(UInt8(truncatingIfNeeded: value))
        try writeByte(UInt8(truncatingIfNeeded: value >> 8))
    }

    public func serialize_u32(value: UInt32) throws {
        try writeByte(UInt8(truncatingIfNeeded: value))
        try writeByte(UInt8(truncatingIfNeeded: value >> 8))
        try writeByte(UInt8(truncatingIfNeeded: value >> 16))
        try writeByte(UInt8(truncatingIfNeeded: value >> 24))
    }

    public func serialize_u64(value: UInt64) throws {
        try writeByte(UInt8(truncatingIfNeeded: value))
        try writeByte(UInt8(truncatingIfNeeded: value >> 8))
        try writeByte(UInt8(truncatingIfNeeded: value >> 16))
        try writeByte(UInt8(truncatingIfNeeded: value >> 24))
        try writeByte(UInt8(truncatingIfNeeded: value >> 32))
        try writeByte(UInt8(truncatingIfNeeded: value >> 40))
        try writeByte(UInt8(truncatingIfNeeded: value >> 48))
        try writeByte(UInt8(truncatingIfNeeded: value >> 56))
    }

    public func serialize_u128(value: BigInt8) throws {
        if value >> 128 != 0 {
            throw BinarySerializerError.invalidValue(issue: "Invalid value for an unsigned int128")
        }

        assert(value._data.count <= 16 || value._data[16] == 0)

        for i in 0 ..< 16 {
            if i < value._data.count {
                try writeByte(value._data[i])
            } else {
                try writeByte(0)
            }
        }
    }

    public func serialize_i8(value: Int8) throws {
        try serialize_u8(value: UInt8(bitPattern: value))
    }

    public func serialize_i16(value: Int16) throws {
        try serialize_u16(value: UInt16(bitPattern: value))
    }

    public func serialize_i32(value: Int32) throws {
        try serialize_u32(value: UInt32(bitPattern: value))
    }

    public func serialize_i64(value: Int64) throws {
        try serialize_u64(value: UInt64(bitPattern: value))
    }

    public func serialize_i128(value: BigInt8) throws {
        if value >= 0 {
            if value >> 127 != 0 {
                throw BinarySerializerError.invalidValue(issue: "Invalid value for a signed int128")
            }
            try serialize_u128(value: value)
        } else {
            if -(value + 1) >> 127 != 0 {
                throw BinarySerializerError.invalidValue(issue: "Invalid value for a signed int128")
            }
            try serialize_u128(value: value + (BigInt8(1) << 128))
        }
    }

    public func serialize_option_tag(value: Bool) throws {
        try writeByte(value ? 1 : 0)
    }

    public func get_buffer_offset() -> Int {
        return output.getOffset()
    }

    public func serialize_len(value _: Int64) throws {
        assertionFailure("Not implemented")
    }

    public func serialize_variant_index(value _: Int) throws {
        assertionFailure("Not implemented")
    }

    public func sort_map_entries(offsets _: [Int]) {
        assertionFailure("Not implemented")
    }
}
