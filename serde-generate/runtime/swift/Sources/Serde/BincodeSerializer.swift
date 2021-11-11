//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public class BincodeSerializer: BinarySerializer {
    public init() {
        super.init(maxContainerDepth: Int.max)
    }

    override public func serialize_len(value: Int) throws {
        try serialize_u64(value: UInt64(value))
    }

    override public func serialize_variant_index(value: UInt32) throws {
        try serialize_u32(value: value)
    }

    override public func sort_map_entries(offsets _: [Int]) {
        // Not required by the format.
    }
}
