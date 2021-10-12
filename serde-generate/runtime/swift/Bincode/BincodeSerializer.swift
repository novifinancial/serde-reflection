//  Copyright © Diem Association. All rights reserved.

import Foundation

public class BincodeSerializer : BinarySerializer {
  
  init() {
    super.init(maxContainerDepth: Int64.max)
  }
  
  public override func serialize_len(value: Int64) {
    output.write(data: Data(fromArray: [value]))
  }
  
  public override func serialize_variant_index(value: Int) {
    output.write(data: Data(fromArray: [value]))
  }
  
  public override func sort_map_entries(offsets: [Int]) {
    // Not required by the format.
  }
}
