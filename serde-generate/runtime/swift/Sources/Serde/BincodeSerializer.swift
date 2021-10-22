//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public class BincodeSerializer: BinarySerializer {
    public init() {
        super.init(maxContainerDepth: Int64.max)
    }

    override public func serialize_len(value: Int64) {
        output.write(data: Data(fromArray: [value]))
    }

    override public func serialize_variant_index(value: Int) {
        output.write(data: Data(fromArray: [value]))
    }

    override public func sort_map_entries(offsets _: [Int]) {
        // Not required by the format.
    }
}
