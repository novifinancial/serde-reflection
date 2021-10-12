//  Copyright © Diem Association. All rights reserved.

import Foundation

public struct Unit: Equatable {
  public func Equals(obj:Any) -> Bool {
    return (obj as? Unit) != nil
  }
  
  public static func ==(l:Unit, r:Unit) -> Bool {
    return true
  }
  
  public static func !=(l:Unit, r:Unit) -> Bool {
    return false
  }
  
  public func GetHashCode() -> Int { return 793253941 }
}
