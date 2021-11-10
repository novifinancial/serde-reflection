//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public class BincodeSerializer: BinarySerializer {
    public init() {
        super.init(maxContainerDepth: Int64.max)
    }

    override public func serialize_len(value: Int64) throws {
        try serialize_i64(value: value)
    }

    override public func serialize_variant_index(value: Int) throws {
        try serialize_i32(value: Int32(value))
    }

    override public func sort_map_entries(offsets _: [Int]) {
        // Not required by the format.
    }
}
