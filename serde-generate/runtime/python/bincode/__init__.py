# Copyright (c) Facebook, Inc. and its affiliates
# SPDX-License-Identifier: MIT OR Apache-2.0

import dataclasses
import collections
import typing
from typing import get_type_hints

import serde_types as st
import serde_binary as sb

# Maximum length in practice for sequences (e.g. in Java).
MAX_LENGTH = (1 << 31) - 1


def _encode_length(value: int) -> bytes:
    if value > MAX_LENGTH:
        raise st.SerializationError("Length exceeds the maximum supported value.")
    return int(value).to_bytes(8, "little", signed=False)


def _decode_length(content: bytes) -> typing.Tuple[int, bytes]:
    value = int.from_bytes(sb.peek(content, 8), byteorder="little", signed=False)
    if value > MAX_LENGTH:
        raise st.DeserializationError("Length exceeds the maximum supported value.")
    return value, content[8:]


_bincode_serialization_config = sb.SerializationConfig(
    encode_length=_encode_length,
    encode_variant_index=lambda x: int(x).to_bytes(4, "little", signed=False),
    sort_map_entries=lambda entries: list(entries),
    max_container_depth=None,
)


_bincode_deserialization_config = sb.DeserializationConfig(
    decode_length=_decode_length,
    decode_variant_index=lambda content: (
        int.from_bytes(sb.peek(content, 4), byteorder="little", signed=False),
        content[4:],
    ),
    check_that_key_slices_are_increasing=lambda key1, key2: None,
    max_container_depth=None,
)


def serialize(obj: typing.Any, obj_type) -> bytes:
    return sb.serialize_with_config(_bincode_serialization_config, obj, obj_type, 0)


def deserialize(content: bytes, obj_type) -> typing.Tuple[typing.Any, bytes]:
    return sb.deserialize_with_config(
        _bincode_deserialization_config, content, obj_type, 0
    )
