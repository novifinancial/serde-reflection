//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public enum BincodeDeserializerError: Error {
    case invalidInput(issue: String)
}

public class BincodeDeserializer: BinaryDeserializer {
    public init(input: [UInt8]) {
        super.init(input: input, maxContainerDepth: Int64.max)
    }

    override public func deserialize_len() throws -> Int64 {
        let value: Int64 = reader.readInt64()
        if value < 0 || value > Int.max {
            throw BincodeDeserializerError.invalidInput(issue: "Incorrect length value")
        }
        return value
    }

    override public func deserialize_variant_index() -> Int {
        return Int(reader.readInt32())
    }
}
