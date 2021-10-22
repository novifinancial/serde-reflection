//  Copyright © Diem Association. All rights reserved.

import Foundation

public extension OutputStream {
    func getBuffer() -> [UInt8] {
        return [UInt8](property(forKey: .dataWrittenToMemoryStreamKey) as! Data)
    }

    func getOffset() -> Int {
        return (property(forKey: .dataWrittenToMemoryStreamKey) as! Data).count
    }

    @discardableResult
    internal func write(data: Data) -> Int {
        return data.withUnsafeBytes {
            write($0.bindMemory(to: UInt8.self).baseAddress!, maxLength: data.count)
        }
    }
}
