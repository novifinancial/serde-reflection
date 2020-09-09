# Copyright (c) Facebook, Inc. and its affiliates
# SPDX-License-Identifier: MIT OR Apache-2.0

import dataclasses
import collections
import typing
from typing import get_type_hints

import serde_types as st
import serde_binary as sb


_bincode_serialization_config = sb.SerializationConfig(
    encode_length=lambda x: int(x).to_bytes(8, "little", signed=False),
    encode_variant_index=lambda x: int(x).to_bytes(4, "little", signed=False),
    sort_map_entries=lambda entries: list(entries),
)


_bincode_deserialization_config = sb.DeserializationConfig(
    decode_length=lambda content: (
        int.from_bytes(sb.peek(content, 8), byteorder="little", signed=False),
        content[8:],
    ),
    decode_variant_index=lambda content: (
        int.from_bytes(sb.peek(content, 4), byteorder="little", signed=False),
        content[4:],
    ),
    check_that_key_slices_are_increasing=lambda key1, key2: None,
)


def serialize(obj: typing.Any, obj_type) -> bytes:
    return sb.serialize_with_config(_bincode_serialization_config, obj, obj_type)


def deserialize(content: bytes, obj_type) -> typing.Tuple[typing.Any, bytes]:
    return sb.deserialize_with_config(
        _bincode_deserialization_config, content, obj_type
    )
