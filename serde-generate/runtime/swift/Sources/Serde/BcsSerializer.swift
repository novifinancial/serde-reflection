//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public enum BcsSerializerError: Error {
    case invalidInput(issue: String)
}

public class BcsSerializer: BinarySerializer {
    public let MAX_LENGTH: Int = 1 << 31 - 1
    public let MAX_CONTAINER_DEPTH: Int = 500

    public init() {
        super.init(maxContainerDepth: MAX_CONTAINER_DEPTH)
    }

    private func serialize_u32_as_uleb128(value: UInt32) throws {
        var input = value
        while input >= 0x80 {
            writeByte(UInt8((value & 0x7F) | 0x80))
            input >>= 7
        }
        writeByte(UInt8(input))
    }

    override public func serialize_len(value: Int) throws {
        if value < 0 || value > MAX_LENGTH {
            throw BcsSerializerError.invalidInput(issue: "Invalid length value")
        }
        try serialize_u32_as_uleb128(value: UInt32(value))
    }

    override public func serialize_variant_index(value: UInt32) throws {
        try serialize_u32_as_uleb128(value: value)
    }

    override public func sort_map_entries(offsets _: [Int]) {
        // TODO:
    }
}
