//  Copyright © Diem Association. All rights reserved.

import Foundation

extension OutputStream {
  
  public func getBuffer() -> [UInt8] {
    return [UInt8](self.property(forKey: .dataWrittenToMemoryStreamKey) as! Data)
  }
  
  public func getOffset() -> Int {
    return (self.property(forKey: .dataWrittenToMemoryStreamKey) as! Data).count
  }
  
  @discardableResult
  func write(data: Data) -> Int {
    return data.withUnsafeBytes {
      write($0.bindMemory(to: UInt8.self).baseAddress!, maxLength: data.count)
    }
  }
}
