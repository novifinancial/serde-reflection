//  Copyright © Diem Association. All rights reserved.

import Foundation

protocol Serializer {
    func serialize_str(value: String) throws
    func serialize_bytes(value: [UInt8])
    func serialize_bool(value: Bool)
    func serialize_unit(value: Unit)
    func serialize_char(value: Character) throws
    func serialize_f32(value: Float)
    func serialize_f64(value: Double)
    func serialize_u8(value: UInt8)
    func serialize_u16(value: UInt16)
    func serialize_u32(value: UInt32)
    func serialize_u64(value: UInt64)
    func serialize_u128(value: BigInt8) throws
    func serialize_i8(value: Int8)
    func serialize_i16(value: Int16)
    func serialize_i32(value: Int32)
    func serialize_i64(value: Int64)
    func serialize_i128(value: BigInt8) throws
    func serialize_len(value: Int64)
    func serialize_variant_index(value: Int)
    func serialize_option_tag(value: Bool)
    func increase_container_depth() throws
    func decrease_container_depth()
    func get_buffer_offset() -> Int
    func sort_map_entries(offsets: [Int])
    func get_bytes() -> [UInt8]
}
