//  Copyright © Diem Association. All rights reserved.

import Foundation

extension Data {
    init<T>(fromArray values: [T]) {
        self = values.withUnsafeBytes { Data($0) }
    }

    func toArray<T>(type _: T.Type) -> [T] where T: ExpressibleByIntegerLiteral {
        var array = [T](repeating: 0, count: count / MemoryLayout<T>.stride)
        _ = array.withUnsafeMutableBytes { copyBytes(to: $0) }
        return array
    }
}
