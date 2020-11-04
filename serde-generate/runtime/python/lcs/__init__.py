# Copyright (c) Facebook, Inc. and its affiliates
# SPDX-License-Identifier: MIT OR Apache-2.0

import dataclasses
import collections
import io
import typing
from copy import copy
from typing import get_type_hints

import serde_types as st
import serde_binary as sb

MAX_LENGTH = (1 << 31) - 1
MAX_U32 = (1 << 32) - 1
MAX_CONTAINER_DEPTH = 500


class LcsSerializer(sb.BinarySerializer):
    def __init__(self):
        super().__init__(
            output=io.BytesIO(), container_depth_budget=MAX_CONTAINER_DEPTH
        )

    def serialize_u32_as_uleb128(self, value: int):
        while value >= 0x80:
            b = (value & 0x7F) | 0x80
            self.output.write(b.to_bytes(1, "little", signed=False))
            value >>= 7
        self.output.write(value.to_bytes(1, "little", signed=False))

    def serialize_len(self, value: int):
        if value > MAX_LENGTH:
            raise st.SerializationError("Length exceeds the maximum supported value.")
        self.serialize_u32_as_uleb128(value)

    def serialize_variant_index(self, value: int):
        if value > MAX_U32:
            raise st.SerializationError(
                "Variant index exceeds the maximum supported value."
            )
        self.serialize_u32_as_uleb128(value)

    def sort_map_entries(self, offsets: typing.List[int]):
        if len(offsets) < 1:
            return
        buf = self.output.getbuffer()
        offsets.append(len(buf))
        slices = []
        for i in range(1, len(offsets)):
            slices.append(bytes(buf[offsets[i - 1] : offsets[i]]))
        buf.release()
        slices.sort()
        self.output.seek(offsets[0])
        for s in slices:
            self.output.write(s)
        assert offsets[-1] == len(self.output.getbuffer())


class LcsDeserializer(sb.BinaryDeserializer):
    def __init__(self, content):
        super().__init__(
            input=io.BytesIO(content), container_depth_budget=MAX_CONTAINER_DEPTH
        )

    def deserialize_uleb128_as_u32(self) -> int:
        value = 0
        for shift in range(0, 32, 7):
            byte = int.from_bytes(self.read(1), "little", signed=False)
            digit = byte & 0x7F
            value |= digit << shift
            if value > MAX_U32:
                raise st.DeserializationError(
                    "Overflow while parsing uleb128-encoded uint32 value"
                )
            if digit == byte:
                if shift > 0 and digit == 0:
                    raise st.DeserializationError(
                        "Invalid uleb128 number (unexpected zero digit)"
                    )
                return value

        raise st.DeserializationError(
            "Overflow while parsing uleb128-encoded uint32 value"
        )

    def deserialize_len(self) -> int:
        value = self.deserialize_uleb128_as_u32()
        if value > MAX_LENGTH:
            raise st.DeserializationError("Length exceeds the maximum supported value.")
        return value

    def deserialize_variant_index(self) -> int:
        return self.deserialize_uleb128_as_u32()

    def check_that_key_slices_are_increasing(
        self, slice1: typing.Tuple[int, int], slice2: typing.Tuple[int, int]
    ):
        key1 = bytes(self.input.getbuffer()[slice1[0] : slice1[1]])
        key2 = bytes(self.input.getbuffer()[slice2[0] : slice2[1]])
        if key1 >= key2:
            raise st.DeserializationError(
                "Serialized keys in a map must be ordered by increasing lexicographic order"
            )


def serialize(obj: typing.Any, obj_type) -> bytes:
    serializer = LcsSerializer()
    serializer.serialize_any(obj, obj_type)
    return serializer.get_buffer()


def deserialize(content: bytes, obj_type) -> typing.Tuple[typing.Any, bytes]:
    deserializer = LcsDeserializer(content)
    value = deserializer.deserialize_any(obj_type)
    return value, deserializer.get_remaining_buffer()
