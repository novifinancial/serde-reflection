//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public enum BcsDeserializerError: Error {
    case invalidInput(issue: String)
}

public class BcsDeserializer: BinaryDeserializer {
    public init(input: [UInt8]) {
        super.init(input: input, maxContainerDepth: Int64.max)
    }

    private func deserialize_uleb128_as_u32() throws -> Int {
        var value: Int64 = 0
        for shift in stride(from: 0, to: 32, by: 7) {
            let x: UInt8 = reader.readUInt8()
            let digit: UInt8 = (UInt8)(x & 0x7F)
            value |= ((Int64)(digit) << shift)
            if value < 0 || value > Int.max {
                throw BcsDeserializerError.invalidInput(issue: "Overflow while parsing uleb128-encoded uint32 value")
            }
            if digit == x {
                if shift > 0, digit == 0 {
                    throw BcsDeserializerError.invalidInput(issue: "Invalid uleb128 number (unexpected zero digit)")
                }
                return (Int)(value)
            }
        }
        throw BcsDeserializerError.invalidInput(issue: "Overflow while parsing uleb128-encoded uint32 value")
    }

    override public func deserialize_len() throws -> Int64 {
        return Int64(try deserialize_uleb128_as_u32())
    }

    override public func deserialize_variant_index() throws -> Int {
        return Int(try deserialize_uleb128_as_u32())
    }

    public func check_that_key_slices_are_increasing(key1 _: Range<Int>, key2 _: Range<Int>) {
        // TODO
    }
}
