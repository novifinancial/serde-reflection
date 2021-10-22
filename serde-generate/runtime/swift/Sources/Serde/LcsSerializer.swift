//  Copyright © Diem Association. All rights reserved.

import Foundation

enum LcsSerializerError: Error {
    case lcsException(issue: String)
}

public class LcsSerializer: BinarySerializer {
    public let MAX_LENGTH = Int64(Int.max)
    public let MAX_CONTAINER_DEPTH: Int64 = 500

    public init() {
        super.init(maxContainerDepth: MAX_CONTAINER_DEPTH)
    }

    private func serialize_u32_as_uleb128(value: UInt32) {
        var input = value
        while input >> 7 != 0 {
            output.write(data: Data(fromArray: [(UInt8)((value & 0x7F) | 0x80)]))
            input >>= 7
        }
        output.write(data: Data(fromArray: [(UInt8)(input)]))
    }

    override public func serialize_len(value len: Int64) {
        serialize_u32_as_uleb128(value: (UInt32)(len))
    }

    override public func serialize_variant_index(value: Int) {
        return serialize_u32_as_uleb128(value: (UInt32)(value))
    }

    override public func sort_map_entries(offsets _: [Int]) {
        // Not required by the format.
    }
}
