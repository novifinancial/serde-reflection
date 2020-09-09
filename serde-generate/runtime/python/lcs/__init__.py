# Copyright (c) Facebook, Inc. and its affiliates
# SPDX-License-Identifier: MIT OR Apache-2.0

import dataclasses
import collections
import typing
from typing import get_type_hints

import serde_types as st
import serde_binary as sb


LCS_MAX_LENGTH = 1 << 31
LCS_MAX_U32 = (1 << 32) - 1


def _encode_u32_as_uleb128(value: int) -> bytes:
    res = b""
    while value >= 0x80:
        res += ((value & 0x7F) | 0x80).to_bytes(1, "little", signed=False)
        value >>= 7
    res += value.to_bytes(1, "little", signed=False)
    return res


def _encode_length(value: int) -> bytes:
    if value > LCS_MAX_LENGTH:
        raise st.SerializationError("Length exceeds the maximum supported value.")
    return _encode_u32_as_uleb128(value)


def _encode_variant_index(value: int) -> bytes:
    if value > LCS_MAX_U32:
        raise st.SerializationError(
            "Variant index exceeds the maximum supported value."
        )
    return _encode_u32_as_uleb128(value)


def _decode_uleb128_as_u32(content: bytes) -> typing.Tuple[int, bytes]:
    value = 0
    for shift in range(0, 32, 7):
        byte = int.from_bytes(sb.peek(content, 1), "little", signed=False)
        content = content[1:]
        digit = byte & 0x7F
        value |= digit << shift
        if value > LCS_MAX_U32:
            raise st.DeserializationError(
                "Overflow while parsing uleb128-encoded uint32 value"
            )
        if digit == byte:
            if shift > 0 and digit == 0:
                raise st.DeserializationError(
                    "Invalid uleb128 number (unexpected zero digit)"
                )
            return value, content

    raise st.DeserializationError("Overflow while parsing uleb128-encoded uint32 value")


def _decode_length(content: bytes) -> typing.Tuple[int, bytes]:
    value, content = _decode_uleb128_as_u32(content)
    if value > LCS_MAX_LENGTH:
        raise st.DeserializationError("Length exceeds the maximum supported value.")
    return value, content


def _decode_variant_index(content: bytes) -> typing.Tuple[int, bytes]:
    return _decode_uleb128_as_u32(content)


def _check_that_key_slices_are_increasing(key1: bytes, key2: bytes):
    if key1 >= key2:
        raise st.DeserializationError(
            "Serialized keys in a map must be ordered by increasing lexicographic order"
        )


_lcs_serialization_config = sb.SerializationConfig(
    encode_length=_encode_length,
    encode_variant_index=_encode_variant_index,
    sort_map_entries=lambda entries: sorted(entries),
)


_lcs_deserialization_config = sb.DeserializationConfig(
    decode_length=_decode_length,
    decode_variant_index=_decode_variant_index,
    check_that_key_slices_are_increasing=_check_that_key_slices_are_increasing,
)


def serialize(obj: typing.Any, obj_type) -> bytes:
    return sb.serialize_with_config(_lcs_serialization_config, obj, obj_type)


def deserialize(content: bytes, obj_type) -> typing.Tuple[typing.Any, bytes]:
    return sb.deserialize_with_config(_lcs_deserialization_config, content, obj_type)
