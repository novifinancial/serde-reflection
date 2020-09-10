from collections import OrderedDict
from dataclasses import dataclass
import unittest
import serde_types as st
import bincode
import typing


class BincodeTestCase(unittest.TestCase):
    def test_bincode_bool(self):
        self.assertEqual(bincode.serialize(False, st.bool), b"\x00")
        self.assertEqual(bincode.serialize(True, st.bool), b"\x01")
        self.assertEqual(bincode.deserialize(b"\x00", st.bool), (False, b""))
        self.assertEqual(bincode.deserialize(b"\x01", st.bool), (True, b""))
        with self.assertRaises(st.DeserializationError):
            bincode.deserialize(b"\x02", st.bool)
        with self.assertRaises(st.DeserializationError):
            bincode.deserialize(b"", st.bool)

    def test_bincode_u8(self):
        self.assertEqual(bincode.serialize(0x1, st.uint8), b"\x01")
        self.assertEqual(bincode.deserialize(b"\xff", st.uint8), (255, b""))

    def test_bincode_u16(self):
        self.assertEqual(bincode.serialize(0x0102, st.uint16), b"\x02\x01")
        self.assertEqual(bincode.deserialize(b"\xff\xff", st.uint16), (65535, b""))

    def test_bincode_u32(self):
        self.assertEqual(bincode.serialize(0x01020304, st.uint32), b"\x04\x03\x02\x01")
        self.assertEqual(
            bincode.deserialize(b"\xff\xff\xff\xff", st.uint32), (4294967295, b"")
        )

    def test_bincode_u64(self):
        self.assertEqual(
            bincode.serialize(0x0102030405060708, st.uint64),
            b"\x08\x07\x06\x05\x04\x03\x02\x01",
        )
        self.assertEqual(
            bincode.deserialize(b"\xff" * 8, st.uint64), ((1 << 64) - 1, b"")
        )

    def test_bincode_u128(self):
        self.assertEqual(
            bincode.serialize(
                st.uint128(0x0102030405060708090A0B0C0D0E0F10), st.uint128
            ),
            b"\x10\x0f\x0e\r\x0c\x0b\n\t\x08\x07\x06\x05\x04\x03\x02\x01",
        )
        self.assertEqual(
            bincode.deserialize(b"\xff" * 16, st.uint128),
            (st.uint128((1 << 128) - 1), b""),
        )

    def test_bincode_i8(self):
        self.assertEqual(bincode.serialize(4, st.int8), b"\x04")
        self.assertEqual(bincode.serialize(-2, st.int8), b"\xfe")
        self.assertEqual(bincode.deserialize(b"\xff", st.int8), (-1, b""))

    def test_bincode_i16(self):
        self.assertEqual(bincode.serialize(0x0102, st.int16), b"\x02\x01")
        self.assertEqual(bincode.deserialize(b"\xff\xff", st.int16), (-1, b""))

    def test_bincode_i32(self):
        self.assertEqual(bincode.serialize(0x01020304, st.int32), b"\x04\x03\x02\x01")
        self.assertEqual(bincode.deserialize(b"\xff\xff\xff\xff", st.int32), (-1, b""))

    def test_bincode_i64(self):
        self.assertEqual(
            bincode.serialize(0x0102030405060708, st.int64),
            b"\x08\x07\x06\x05\x04\x03\x02\x01",
        )
        self.assertEqual(bincode.deserialize(b"\xff" * 8, st.int64), (-1, b""))

    def test_bincode_i128(self):
        self.assertEqual(
            bincode.serialize(st.int128(0x0102030405060708090A0B0C0D0E0F10), st.int128),
            b"\x10\x0f\x0e\r\x0c\x0b\n\t\x08\x07\x06\x05\x04\x03\x02\x01",
        )
        self.assertEqual(
            bincode.deserialize(b"\xff" * 16, st.int128), (st.int128(-1), b"")
        )

    def test_serialize_bytes(self):
        self.assertEqual(bincode.serialize(b"", bytes), b"\x00" * 8)
        self.assertEqual(
            bincode.serialize(b"\x00\x00", bytes),
            b"\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        )
        self.assertEqual(
            bincode.serialize(b"\x00" * 128, bytes),
            b"\x80\x00\x00\x00\x00\x00\x00\x00" + b"\x00" * 128,
        )

        self.assertEqual(bincode.deserialize(b"\x00" * 8, bytes), (b"", b""))

    def test_serialize_tuple(self):
        T = typing.Tuple[st.uint8, st.uint16]
        self.assertEqual(bincode.serialize((0, 1), T), b"\x00\x01\x00")
        self.assertEqual(bincode.deserialize(b"\x02\x01\x00", T), ((2, 1), b""))

    def test_serialize_option(self):
        T = typing.Optional[st.uint16]
        self.assertEqual(bincode.serialize(None, T), b"\x00")
        self.assertEqual(bincode.serialize(6, T), b"\x01\x06\x00")
        self.assertEqual(bincode.deserialize(b"\x00", T), (None, b""))
        self.assertEqual(bincode.deserialize(b"\x01\x02\x00", T), (2, b""))
        with self.assertRaisesRegex(st.DeserializationError, "Wrong tag.*"):
            bincode.deserialize(b"\x02\x06\x00", T)

    def test_serialize_sequence(self):
        Seq = typing.Sequence[st.uint16]
        self.assertEqual(
            bincode.serialize([], Seq), b"\x00\x00\x00\x00\x00\x00\x00\x00"
        )
        self.assertEqual(
            bincode.serialize([0, 1], Seq),
            b"\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x00",
        )
        self.assertEqual(
            bincode.serialize([256] * 256, Seq),
            b"\x00\x01\x00\x00\x00\x00\x00\x00" + b"\x00\x01" * 256,
        )
        self.assertEqual(
            bincode.deserialize(b"\x01\x00\x00\x00\x00\x00\x00\x00\x03\x00", Seq),
            ([3], b""),
        )

    def test_serialize_str(self):
        self.assertEqual(
            bincode.serialize("ABC", str), b"\x03\x00\x00\x00\x00\x00\x00\x00ABC"
        )
        self.assertEqual(
            bincode.deserialize(b"\x03\x00\x00\x00\x00\x00\x00\x00ABC", str),
            ("ABC", b""),
        )
        with self.assertRaises(st.DeserializationError):
            bincode.deserialize(b"\x03AB", str)

    def test_serialize_map(self):
        Map = typing.Dict[st.uint16, st.uint8]
        m = OrderedDict([(256, 3), (1, 5)])
        e = bincode.serialize(m, Map)
        self.assertEqual(e, b"\x02\x00\x00\x00\x00\x00\x00\x00\x00\x01\x03\x01\x00\x05")
        self.assertEqual(
            (m, b""),
            bincode.deserialize(
                b"\x02\x00\x00\x00\x00\x00\x00\x00\x00\x01\x03\x01\x00\x05", Map
            ),
        )
        self.assertEqual(
            (m, b""),
            bincode.deserialize(
                b"\x02\x00\x00\x00\x00\x00\x00\x00\x01\x00\x05\x00\x01\x03", Map
            ),
        )

    @dataclass
    class Foo:
        x: st.uint8
        y: st.uint16

    def test_struct(self):
        self.assertEqual(
            bincode.serialize(BincodeTestCase.Foo(x=0, y=1), BincodeTestCase.Foo),
            b"\x00\x01\x00",
        )
        self.assertEqual(
            bincode.deserialize(b"\x02\x01\x00", BincodeTestCase.Foo),
            (BincodeTestCase.Foo(x=2, y=1), b""),
        )

    class Bar:
        VARIANTS = [None, None, None]

    @dataclass
    class Bar1(Bar):
        INDEX = 1
        x: st.uint8
        y: st.uint16

    Bar.VARIANTS[1] = Bar1

    def test_enum(self):
        self.assertEqual(
            bincode.serialize(BincodeTestCase.Bar1(x=0, y=1), BincodeTestCase.Bar),
            b"\x01\x00\x00\x00\x00\x01\x00",
        )
        self.assertEqual(
            bincode.deserialize(b"\x01\x00\x00\x00\x02\x01\x00", BincodeTestCase.Bar),
            (BincodeTestCase.Bar1(x=2, y=1), b""),
        )
