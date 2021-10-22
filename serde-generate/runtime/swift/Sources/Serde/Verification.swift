//  Copyright (c) Facebook, Inc. and its affiliates.

import Foundation

public enum Verification {
    /// - Parameters:
    ///   - key1: (startIndex, endIndex)
    ///   - key2: (startIndex, endIndex)
    /// - Returns:  Returns an integer corresponding to the lexicographic ordering of the two input byte strings.
    public static func CompareLexicographic(key1: Range<UInt8>, key2: Range<UInt8>) -> Int {
        let key1Length = key1.endIndex - key1.startIndex
        let key2Length = key2.endIndex - key2.startIndex

        for i in 0 ..< key1Length {
            let byte1 = key1[i]
            if i >= key2Length {
                return 1
            }
            let byte2 = key2[i]
            if byte1 > byte2 {
                return 1
            }
            if byte1 < byte2 {
                return -1
            }
        }
        if key2Length > key1Length {
            return -1
        }
        return 0
    }
}
