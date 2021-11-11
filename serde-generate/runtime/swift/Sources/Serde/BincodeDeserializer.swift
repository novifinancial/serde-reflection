//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public enum BincodeDeserializerError: Error {
    case invalidInput(issue: String)
}

public class BincodeDeserializer: BinaryDeserializer {
    public init(input: [UInt8]) {
        super.init(input: input, maxContainerDepth: Int.max)
    }

    override public func deserialize_len() throws -> Int {
        let value = try deserialize_i64()
        if value < 0 || value > Int.max {
            throw BincodeDeserializerError.invalidInput(issue: "Incorrect length value")
        }
        return Int(value)
    }

    override public func deserialize_variant_index() throws -> UInt32 {
        return try deserialize_u32()
    }
}
