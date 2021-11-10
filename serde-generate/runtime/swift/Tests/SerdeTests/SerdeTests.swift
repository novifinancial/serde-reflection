//  Copyright (c) Facebook, Inc. and its affiliates.

import Serde
import XCTest

class SerdeTests: XCTestCase {
    func testSerializer() throws {
        let serializer = BcsSerializer()
        try serializer.serialize_u8(value: 255) // 1
        try serializer.serialize_u32(value: 1) // 4
        try serializer.serialize_u32(value: 1) // 4
        try serializer.serialize_u32(value: 2) // 4
        XCTAssertEqual(serializer.get_buffer_offset(), 13, "the buffer size should be same")
        XCTAssertEqual(serializer.output.getBuffer(), [255, 1, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0], "the array should be same")
    }

    func testDeserializer() throws {
        let deserializer = BincodeDeserializer(input: [1, 0, 0, 0])
        let result = try deserializer.deserialize_u32()
        XCTAssertEqual(result, 1, "should match")
    }

    func testSerializeUint8() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_u8(value: 255)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_u8()
        XCTAssertEqual(result, 255, "should be same")
    }

    func testSerializeUint16() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_u16(value: 65535)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_u16()
        XCTAssertEqual(result, 65535, "should be same")
    }

    func testSerializeUint32() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_u32(value: 4_294_967_295)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_u32()
        XCTAssertEqual(result, 4_294_967_295, "should be same")
    }

    func testSerializeInt8() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_u8(value: 127)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_u8()
        XCTAssertEqual(result, 127, "should be same")
    }

    func testSerializeInt16() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_i16(value: 32767)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_i16()
        XCTAssertEqual(result, 32767, "should be same")
    }

    func testSerializeInt32() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_i32(value: 2_147_483_647)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_i32()
        XCTAssertEqual(result, 2_147_483_647, "should be same")
    }

    func testSerializeInt64() throws {
        let serializer = BincodeSerializer()
        try serializer.serialize_i64(value: 9_223_372_036_854_775_807)
        let deserializer = BincodeDeserializer(input: serializer.get_bytes())
        let result = try deserializer.deserialize_i64()
        XCTAssertEqual(result, 9_223_372_036_854_775_807, "should be same")
    }

    func testSerializeU128() throws {
        let serializer = BcsSerializer()
        XCTAssertNoThrow(try serializer.serialize_u128(value: (BigInt8(1) << 128) - 1))
        XCTAssertEqual(serializer.output.getBuffer(), [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255], "the array should be same")

        let serializer2 = BcsSerializer()
        XCTAssertNoThrow(try serializer2.serialize_u128(value: BigInt8(1)))
        XCTAssertEqual(serializer2.output.getBuffer(), [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "the array should be same")

        let serializer3 = BcsSerializer()
        XCTAssertNoThrow(try serializer3.serialize_u128(value: BigInt8(0)))
        XCTAssertEqual(serializer3.output.getBuffer(), [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "the array should be same")

        let serializer4 = BcsSerializer()
        XCTAssertThrowsError(try serializer4.serialize_u128(value: BigInt8(-1)))
        XCTAssertThrowsError(try serializer4.serialize_u128(value: (BigInt8(1) << 128) + 1))
    }

    func testSerializeI128() throws {
        let serializer = BcsSerializer()
        XCTAssertNoThrow(try serializer.serialize_i128(value: BigInt8(-1)))
        XCTAssertEqual(serializer.output.getBuffer(), [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255], "the array should be same")

        let serializer2 = BcsSerializer()
        XCTAssertNoThrow(try serializer2.serialize_i128(value: BigInt8(1)))
        XCTAssertEqual(serializer2.output.getBuffer(), [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "the array should be same")

        let serializer3 = BcsSerializer()
        XCTAssertNoThrow(try serializer3.serialize_i128(value: (BigInt8(1) << 127) - 1))
        XCTAssertEqual(serializer3.output.getBuffer(), [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 127], "the array should be same")

        let serializer4 = BcsSerializer()
        XCTAssertNoThrow(try serializer4.serialize_i128(value: -(BigInt8(1) << 127)))
        XCTAssertEqual(serializer4.output.getBuffer(), [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80], "the array should be same")

        let serializer5 = BcsSerializer()
        XCTAssertThrowsError(try serializer5.serialize_i128(value: BigInt8(1) << 127))
        XCTAssertThrowsError(try serializer5.serialize_i128(value: (BigInt8(1) << 127) + 1))
    }

    func testULEB128Encoding() throws {
        let serializer = BcsSerializer()
        try serializer.serialize_len(value: 0)
        try serializer.serialize_len(value: 1)
        try serializer.serialize_len(value: 127)
        try serializer.serialize_len(value: 128)
        try serializer.serialize_len(value: 3000)
        XCTAssertEqual(serializer.output.getBuffer(), [0, 1, 127, 128, 1, 184, 23], "the array should be same")
    }

    func randomBitLength() -> Int {
        return Int.random(in: 2 ... 1000)
    }

    func testBigInt8() throws {
        let x: BigInt8 = 100
        let xComp = UInt8(x)
        XCTAssertEqual(x.description, xComp.description, "should be same")

        let y: BigInt8 = -100
        let yComp = Int8(y)
        XCTAssertEqual(y.description, yComp.description, "should be same")

        let zComp = Int.min + 1
        let z = BigInt8(zComp)
        XCTAssertEqual(z.description, zComp.description, "should be same")

        let randomBits = BigInt8(randomBits: 1_000_000)
        XCTAssertGreaterThan(randomBits, randomBits - 1, "should be bigger")
        let negativeRandomBits = -randomBits
        XCTAssertGreaterThan(negativeRandomBits, negativeRandomBits - 1, "should be bigger")

        let (x0, y0, x1, y1) = (
            BigInt8(randomBits: randomBitLength()),
            BigInt8(randomBits: randomBitLength()),
            BigInt8(randomBits: randomBitLength()),
            BigInt8(randomBits: randomBitLength())
        )
        let r1 = (x0 + y0) * (x1 + y1)
        let r2 = ((x0 * x1) + (x0 * y1), (y0 * x1) + (y0 * y1))
        XCTAssertEqual(r1, r2.0 + r2.1, "should be same")

        let x2 = BigInt8(-1)
        let z1 = -1 as Int
        for i in 0 ..< 64 {
            let a = x2 << i
            let b = z1 << i
            XCTAssertEqual(a.description, b.description, "should be same")
        }
    }
}
