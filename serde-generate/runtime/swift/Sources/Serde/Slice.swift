//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public struct Slice {
    public var start: Int
    public var end: Int

    public init(start: Int, end: Int) {
        self.start = start
        self.end = end
    }

    // Lexicographic comparison between the (unsigned!) bytes referenced by `slice1` and `slice2`
    // into `content`.
    static func compare_bytes(_ content: [UInt8], _ slice1: Slice, _ slice2: Slice) -> Int {
        let start1: Int = slice1.start
        let end1: Int = slice1.end
        let start2: Int = slice2.start
        let end2: Int = slice2.end

        for i in 0 ..< end1 - start1 {
            let byte1 = Int(content[start1 + i] & 0xFF)
            if start2 + i >= end2 {
                return 1
            }
            let byte2 = Int(content[start2 + i] & 0xFF)
            if byte1 > byte2 {
                return 1
            }
            if byte1 < byte2 {
                return -1
            }
        }

        if end2 - start2 > end1 - start1 {
            return -1
        }
        return 0
    }
}

extension Slice: Equatable {
    public static func == (lhs: Slice, rhs: Slice) -> Bool {
        return lhs.start == rhs.start
    }
}

extension Slice: Comparable {
    public static func < (lhs: Slice, rhs: Slice) -> Bool {
        return lhs.start < rhs.start
    }
}
