import unittest
import serde_types as st
import lcs
import typing


class LcsTestCase(unittest.TestCase):
    def test_lcs_bool(self):
        self.assertEqual(lcs.serialize(False, st.bool), b"\x00")
        self.assertEqual(lcs.serialize(True, st.bool), b"\x01")
        self.assertEqual(lcs.deserialize(b"\x00", st.bool), (False, b""))
        self.assertEqual(lcs.deserialize(b"\x01", st.bool), (True, b""))
        with self.assertRaises(ValueError):
            lcs.deserialize(b"\x02", st.bool)
        with self.assertRaises(ValueError):
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
        self.assertEqual(lcs._encode_u32_as_uleb128(0), b"\x00")
        self.assertEqual(lcs._encode_u32_as_uleb128(3), b"\x03")
        self.assertEqual(lcs._encode_u32_as_uleb128(0x7F), b"\x7f")
        self.assertEqual(lcs._encode_u32_as_uleb128(0x3F01), b"\x81\x7e")
        self.assertEqual(lcs._encode_u32_as_uleb128(0x8001), b"\x81\x80\x02")
        self.assertEqual(
            lcs._encode_u32_as_uleb128(lcs.LCS_MAX_U32), b"\xff\xff\xff\xff\x0f"
        )

        self.assertEqual(lcs._decode_uleb128_as_u32(b"\x00"), (0, b""))
        self.assertEqual(lcs._decode_uleb128_as_u32(b"\x03"), (3, b""))
        self.assertEqual(lcs._decode_uleb128_as_u32(b"\x7f"), (0x7F, b""))
        self.assertEqual(lcs._decode_uleb128_as_u32(b"\x81\x7e"), (0x3F01, b""))
        self.assertEqual(lcs._decode_uleb128_as_u32(b"\x80\x80\x01"), (0x4000, b""))
        self.assertEqual(
            lcs._decode_uleb128_as_u32(b"\x80\x80\x01\x00"), (0x4000, b"\x00")
        )
        self.assertEqual(lcs._decode_uleb128_as_u32(b"\x81\x80\x02"), (0x8001, b""))
        self.assertEqual(
            lcs._decode_uleb128_as_u32(b"\xff\xff\xff\xff\x0f"), (lcs.LCS_MAX_U32, b"")
        )
        with self.assertRaises(ValueError):
            lcs._decode_uleb128_as_u32(b"\x80\x00")
        with self.assertRaises(ValueError):
            lcs._decode_uleb128_as_u32(b"\xff\xff\xff\xff\x10")

    def test_encode_length(self):
        self.assertEqual(
            lcs._encode_length(lcs.LCS_MAX_LENGTH), b"\x80\x80\x80\x80\x08"
        )
        with self.assertRaises(ValueError):
            lcs._decode_length(b"\x80\x80\x80\x80\x09")

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

    def test_serialize_sequence(self):
        Seq = typing.Sequence[st.uint16]
        self.assertEqual(lcs.serialize([], Seq), b"\x00")
        self.assertEqual(lcs.serialize([0, 1], Seq), b"\x02\x00\x00\x01\x00")
        self.assertEqual(
            lcs.serialize([256] * 128, Seq), b"\x80\x01" + b"\x00\x01" * 128
        )
        self.assertEqual(lcs.deserialize(b"\x01\x03\x00", Seq), ([3], b""))

    def test_serialize_str(self):
        self.assertEqual(lcs.serialize("ABC", str), b"\x03ABC")
        self.assertEqual(lcs.deserialize(b"\x03ABC", str), ("ABC", b""))
        with self.assertRaises(ValueError):
            lcs.deserialize(b"\x03AB", str)

    def test_serialize_map(self):
        Map = typing.Dict[st.uint16, st.uint8]
        m = {256: 3, 1: 5}
        e = lcs.serialize(m, Map)
        self.assertEqual(e, b"\x02\x00\x01\x03\x01\x00\x05")
        self.assertEqual(
            (m, b""), lcs.deserialize(b"\x02\x00\x01\x03\x01\x00\x05", Map)
        )
        with self.assertRaises(ValueError):
            # Must enforce canonical encoding.
            lcs.deserialize(b"\x02\x01\x00\x05\x00\x01\x03", Map)
