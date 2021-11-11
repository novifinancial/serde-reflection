//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public enum BcsDeserializerError: Error {
    case invalidInput(issue: String)
}

public class BcsDeserializer: BinaryDeserializer {
    public let MAX_LENGTH: Int = 1 << 31 - 1
    public let MAX_CONTAINER_DEPTH: Int = 500

    public init(input: [UInt8]) {
        super.init(input: input, maxContainerDepth: MAX_CONTAINER_DEPTH)
    }

    private func deserialize_uleb128_as_u32() throws -> UInt32 {
        var value: UInt64 = 0
        for shift in stride(from: 0, to: 32, by: 7) {
            let x = try deserialize_u8()
            let digit = x & 0x7F
            value |= UInt64(digit) << shift
            if value > UInt32.max {
                throw BcsDeserializerError.invalidInput(issue: "Overflow while parsing uleb128-encoded uint32 value")
            }
            if digit == x {
                if shift > 0, digit == 0 {
                    throw BcsDeserializerError.invalidInput(issue: "Invalid uleb128 number (unexpected zero digit)")
                }
                return UInt32(value)
            }
        }
        throw BcsDeserializerError.invalidInput(issue: "Overflow while parsing uleb128-encoded uint32 value")
    }

    override public func deserialize_len() throws -> Int {
        let value = try deserialize_uleb128_as_u32()
        if value > MAX_LENGTH {
            throw BcsDeserializerError.invalidInput(issue: "Overflow while parsing length value")
        }
        return Int(value)
    }

    override public func deserialize_variant_index() throws -> UInt32 {
        return try deserialize_uleb128_as_u32()
    }

    public func check_that_key_slices_are_increasing(key1 _: Range<Int>, key2 _: Range<Int>) {
        // TODO:
    }
}
