import unittest
import serde_types as st
import lcs
import typing
import sys
from dataclasses import dataclass
from collections import OrderedDict


def encode_u32_as_uleb128(value: int) -> bytes:
    serializer = lcs.LcsSerializer()
    serializer.serialize_u32_as_uleb128(value)
    return serializer.get_buffer()


def decode_uleb128_as_u32(content: bytes) -> int:
    deserializer = lcs.LcsDeserializer(content)
    value = deserializer.deserialize_uleb128_as_u32()
    assert deserializer.get_remaining_buffer() == b""
    return value


def encode_length(value: int) -> bytes:
    serializer = lcs.LcsSerializer()
    serializer.serialize_len(value)
    return serializer.get_buffer()


def decode_length(content: bytes) -> int:
    deserializer = lcs.LcsDeserializer(content)
    value = deserializer.deserialize_len()
    assert deserializer.get_remaining_buffer() == b""
    return value


class LcsTestCase(unittest.TestCase):
    def test_lcs_bool(self):
        self.assertEqual(lcs.serialize(False, st.bool), b"\x00")
        self.assertEqual(lcs.serialize(True, st.bool), b"\x01")
        self.assertEqual(lcs.deserialize(b"\x00", st.bool), (False, b""))
        self.assertEqual(lcs.deserialize(b"\x01", st.bool), (True, b""))
        with self.assertRaises(st.DeserializationError):
            lcs.deserialize(b"\x02", st.bool)
        with self.assertRaises(st.DeserializationError):
            lcs.deserialize(b"", st.bool)

    def test_lcs_u8(self):
        self.assertEqual(lcs.serialize(0x1, st.uint8), b"\x01")
        self.assertEqual(lcs.deserialize(b"\xff", st.uint8), (255, b""))

    def test_lcs_u16(self):
        self.assertEqual(lcs.serialize(0x0102, st.uint16), b"\x02\x01")
        self.assertEqual(lcs.deserialize(b"\xff\xff", st.uint16), (65535, b""))

    def test_lcs_u32(self):
        self.assertEqual(lcs.serialize(0x01020304, st.uint32), b"\x04\x03\x02\x01")
        self.assertEqual(
            lcs.deserialize(b"\xff\xff\xff\xff", st.uint32), (4294967295, b"")
        )

    def test_lcs_u64(self):
        self.assertEqual(
            lcs.serialize(0x0102030405060708, st.uint64),
            b"\x08\x07\x06\x05\x04\x03\x02\x01",
        )
        self.assertEqual(lcs.deserialize(b"\xff" * 8, st.uint64), ((1 << 64) - 1, b""))

    def test_lcs_u128(self):
        self.assertEqual(
            lcs.serialize(st.uint128(0x0102030405060708090A0B0C0D0E0F10), st.uint128),
            b"\x10\x0f\x0e\r\x0c\x0b\n\t\x08\x07\x06\x05\x04\x03\x02\x01",
        )
        self.assertEqual(
            lcs.deserialize(b"\xff" * 16, st.uint128), (st.uint128((1 << 128) - 1), b"")
        )

    def test_lcs_i8(self):
        self.assertEqual(lcs.serialize(4, st.int8), b"\x04")
        self.assertEqual(lcs.serialize(-2, st.int8), b"\xfe")
        self.assertEqual(lcs.deserialize(b"\xff", st.int8), (-1, b""))

    def test_lcs_i16(self):
        self.assertEqual(lcs.serialize(0x0102, st.int16), b"\x02\x01")
        self.assertEqual(lcs.deserialize(b"\xff\xff", st.int16), (-1, b""))

    def test_lcs_i32(self):
        self.assertEqual(lcs.serialize(0x01020304, st.int32), b"\x04\x03\x02\x01")
        self.assertEqual(lcs.deserialize(b"\xff\xff\xff\xff", st.int32), (-1, b""))

    def test_lcs_i64(self):
        self.assertEqual(
            lcs.serialize(0x0102030405060708, st.int64),
            b"\x08\x07\x06\x05\x04\x03\x02\x01",
        )
        self.assertEqual(lcs.deserialize(b"\xff" * 8, st.int64), (-1, b""))

    def test_lcs_i128(self):
        self.assertEqual(
            lcs.serialize(st.int128(0x0102030405060708090A0B0C0D0E0F10), st.int128),
            b"\x10\x0f\x0e\r\x0c\x0b\n\t\x08\x07\x06\x05\x04\x03\x02\x01",
        )
        self.assertEqual(lcs.deserialize(b"\xff" * 16, st.int128), (st.int128(-1), b""))

    def test_encode_u32_as_uleb128(self):
        self.assertEqual(encode_u32_as_uleb128(0), b"\x00")
        self.assertEqual(encode_u32_as_uleb128(3), b"\x03")
        self.assertEqual(encode_u32_as_uleb128(0x7F), b"\x7f")
        self.assertEqual(encode_u32_as_uleb128(0x3F01), b"\x81\x7e")
        self.assertEqual(encode_u32_as_uleb128(0x8001), b"\x81\x80\x02")
        self.assertEqual(encode_u32_as_uleb128(lcs.MAX_U32), b"\xff\xff\xff\xff\x0f")

        self.assertEqual(decode_uleb128_as_u32(b"\x00"), 0)
        self.assertEqual(decode_uleb128_as_u32(b"\x03"), 3)
        self.assertEqual(decode_uleb128_as_u32(b"\x7f"), 0x7F)
        self.assertEqual(decode_uleb128_as_u32(b"\x81\x7e"), 0x3F01)
        self.assertEqual(decode_uleb128_as_u32(b"\x80\x80\x01"), 0x4000)
        self.assertEqual(decode_uleb128_as_u32(b"\x80\x80\x01"), 0x4000)
        self.assertEqual(decode_uleb128_as_u32(b"\x81\x80\x02"), 0x8001)
        self.assertEqual(decode_uleb128_as_u32(b"\xff\xff\xff\xff\x0f"), lcs.MAX_U32)
        with self.assertRaises(st.DeserializationError):
            decode_uleb128_as_u32(b"\x80\x00")
        with self.assertRaises(st.DeserializationError):
            decode_uleb128_as_u32(b"\xff\xff\xff\xff\x10")

    def test_encode_length(self):
        self.assertEqual(encode_length(lcs.MAX_LENGTH), b"\xff\xff\xff\xff\x07")
        with self.assertRaises(st.SerializationError):
            encode_length(lcs.MAX_LENGTH + 1)
        with self.assertRaises(st.DeserializationError):
            decode_length(b"\xff\xff\xff\xff\x08")

    def test_serialize_bytes(self):
        self.assertEqual(lcs.serialize(b"", bytes), b"\x00")
        self.assertEqual(lcs.serialize(b"\x00\x00", bytes), b"\x02\x00\x00")
        self.assertEqual(
            lcs.serialize(b"\x00" * 128, bytes), b"\x80\x01" + b"\x00" * 128
        )

        self.assertEqual(lcs.deserialize(b"\x00", bytes), (b"", b""))

    def test_serialize_tuple(self):
        T = typing.Tuple[st.uint8, st.uint16]
        self.assertEqual(lcs.serialize((0, 1), T), b"\x00\x01\x00")
        self.assertEqual(lcs.deserialize(b"\x02\x01\x00", T), ((2, 1), b""))

    def test_serialize_option(self):
        T = typing.Optional[st.uint16]
        self.assertEqual(lcs.serialize(None, T), b"\x00")
        self.assertEqual(lcs.serialize(6, T), b"\x01\x06\x00")
        self.assertEqual(lcs.deserialize(b"\x00", T), (None, b""))
        self.assertEqual(lcs.deserialize(b"\x01\x02\x00", T), (2, b""))
        with self.assertRaisesRegex(st.DeserializationError, "Wrong tag.*"):
            # Must enforce canonical encoding.
            lcs.deserialize(b"\x02\x06\x00", T)

    def test_serialize_sequence(self):
        Seq = typing.Sequence[st.uint16]
        self.assertEqual(lcs.serialize([], Seq), b"\x00")
        self.assertEqual(lcs.serialize([0, 1], Seq), b"\x02\x00\x00\x01\x00")
        self.assertEqual(
            lcs.serialize([256] * 128, Seq), b"\x80\x01" + b"\x00\x01" * 128
        )
        self.assertEqual(lcs.deserialize(b"\x01\x03\x00", Seq), ([3], b""))

    def test_serialize_str(self):
        self.assertEqual(lcs.serialize("ABC\u0394", str), b"\x05ABC\xce\x94")
        self.assertEqual(lcs.deserialize(b"\x05ABC\xce\x94A", str), ("ABC\u0394", b"A"))
        with self.assertRaises(st.DeserializationError):
            lcs.deserialize(b"\x03AB", str)
        with self.assertRaises(st.DeserializationError):
            lcs.deserialize(b"\x03\x80ab", str)

    def test_serialize_map(self):
        Map = typing.Dict[st.uint16, st.uint8]
        m = OrderedDict([(1, 5), (256, 3)])
        e = lcs.serialize(m, Map)
        self.assertEqual(e, b"\x02\x00\x01\x03\x01\x00\x05")
        self.assertEqual(
            (m, b""), lcs.deserialize(b"\x02\x00\x01\x03\x01\x00\x05", Map)
        )
        m2 = OrderedDict([(256, 3), (1, 5)])
        e2 = lcs.serialize(m2, Map)
        self.assertEqual(e2, e)
        with self.assertRaises(st.DeserializationError):
            # Must enforce canonical encoding.
            lcs.deserialize(b"\x02\x01\x00\x05\x00\x01\x03", Map)

    def test_serialize_set(self):
        Set = typing.Dict[st.uint16, st.unit]
        m = {256: None, 1: None}
        e = lcs.serialize(m, Set)
        self.assertEqual(e, b"\x02\x00\x01\x01\x00")
        self.assertEqual((m, b""), lcs.deserialize(b"\x02\x00\x01\x01\x00", Set))
        with self.assertRaises(st.DeserializationError):
            # Must enforce canonical encoding.
            lcs.deserialize(b"\x02\x01\x00\x00\x01", Set)

    @dataclass
    class Foo:
        x: st.uint8
        y: st.uint16

    def test_struct(self):
        self.assertEqual(
            lcs.serialize(LcsTestCase.Foo(x=0, y=1), LcsTestCase.Foo), b"\x00\x01\x00"
        )
        self.assertEqual(
            lcs.deserialize(b"\x02\x01\x00", LcsTestCase.Foo),
            (LcsTestCase.Foo(x=2, y=1), b""),
        )

    class Bar:
        VARIANTS = []  # type: typing.Sequence[typing.Type['Bar']]

    @dataclass
    class Bar1(Bar):
        INDEX = 1
        x: st.uint8
        y: st.uint16

    Bar.VARIANTS = [None, Bar1, None]

    def test_enum(self):
        self.assertEqual(
            lcs.serialize(LcsTestCase.Bar1(x=0, y=1), LcsTestCase.Bar),
            b"\x01\x00\x01\x00",
        )
        self.assertEqual(
            lcs.deserialize(b"\x01\x02\x01\x00", LcsTestCase.Bar),
            (LcsTestCase.Bar1(x=2, y=1), b""),
        )

    @dataclass
    class List:
        next: typing.Optional[typing.Tuple[st.uint64, "LcsTestCase.List"]]

        @staticmethod
        def empty() -> "LcsTestCase.List":
            return LcsTestCase.List(next=None)

        @staticmethod
        def cons(value: st.uint64, tail: "LcsTestCase.List") -> "LcsTestCase.List":
            return LcsTestCase.List(next=(value, tail))

        @staticmethod
        def integers(size: int) -> "LcsTestCase.List":
            result = LcsTestCase.List.empty()
            for i in range(size):
                result = LcsTestCase.List.cons(st.uint64(i), result)
            return result

    def test_max_container_depth(self):
        # Required to avoid RecursionError's in python.
        sys.setrecursionlimit(lcs.MAX_CONTAINER_DEPTH * 5)

        l1 = LcsTestCase.List.integers(4)
        b1 = lcs.serialize(l1, LcsTestCase.List)
        self.assertEqual(
            b1,
            bytes(
                [
                    # fmt: off
                    1, 3, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ]
            ),
        )
        self.assertEqual(lcs.deserialize(b1, LcsTestCase.List), (l1, b""))

        l2 = LcsTestCase.List.integers(lcs.MAX_CONTAINER_DEPTH - 1)
        b2 = lcs.serialize(l2, LcsTestCase.List)
        self.assertEqual(lcs.deserialize(b2, LcsTestCase.List), (l2, b""))

        l3 = LcsTestCase.List.integers(lcs.MAX_CONTAINER_DEPTH)
        with self.assertRaises(st.SerializationError):
            lcs.serialize(l3, LcsTestCase.List)

        b3 = bytes([1, 243, 1, 0, 0, 0, 0, 0, 0]) + b2
        with self.assertRaisesRegex(
            st.DeserializationError, "Exceeded maximum container depth.*"
        ):
            self.assertEqual(lcs.deserialize(b3, LcsTestCase.List))

        # Pairs don't count in "container depth".
        P = typing.Tuple[LcsTestCase.List, LcsTestCase.List]
        self.assertEqual(lcs.deserialize(b2 + b2, P), ((l2, l2), b""))
        with self.assertRaisesRegex(
            st.DeserializationError, "Exceeded maximum container depth.*"
        ):
            lcs.deserialize(b2 + b3, P)
