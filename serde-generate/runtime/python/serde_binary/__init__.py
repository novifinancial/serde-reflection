# Copyright (c) Facebook, Inc. and its affiliates
# SPDX-License-Identifier: MIT OR Apache-2.0

import dataclasses
import collections
import typing
from typing import get_type_hints

import serde_types as st


EncodeInteger = typing.Callable[[int], bytes]
DecodeInteger = typing.Callable[[bytes], typing.Tuple[int, bytes]]
SortMapEntries = typing.Callable[[typing.Iterator[bytes]], typing.Sequence[bytes]]
CheckThatKeySlicesAreIncreasing = typing.Callable[[bytes, bytes], None]


@dataclasses.dataclass(init=False)
class SerializationConfig:
    """Configuration object for `serialize_with_config`.

    "Binary" serialization formats may differ in the way they encode sequence lengths, variant
    index, and how they sort map entries (or not). These 3 elements are passed to the
    constructor and the rest is deduced.
    """

    primitive_encode_map: typing.Dict[typing.Any, typing.Callable[[typing.Any], bytes]]
    encode_length: EncodeInteger
    encode_variant_index: EncodeInteger
    sort_map_entries: SortMapEntries

    def __init__(
        self,
        encode_length: EncodeInteger,
        encode_variant_index: EncodeInteger,
        sort_map_entries: SortMapEntries,
    ):
        self.encode_length = encode_length
        self.encode_variant_index = encode_variant_index
        self.sort_map_entries = sort_map_entries
        self.primitive_encode_map = {
            st.bool: lambda x: int(x).to_bytes(1, "little", signed=False),
            st.uint8: lambda x: int(x).to_bytes(1, "little", signed=False),
            st.uint16: lambda x: int(x).to_bytes(2, "little", signed=False),
            st.uint32: lambda x: int(x).to_bytes(4, "little", signed=False),
            st.uint64: lambda x: int(x).to_bytes(8, "little", signed=False),
            st.uint128: lambda x: int(x).to_bytes(16, "little", signed=False),
            st.int8: lambda x: int(x).to_bytes(1, "little", signed=True),
            st.int16: lambda x: int(x).to_bytes(2, "little", signed=True),
            st.int32: lambda x: int(x).to_bytes(4, "little", signed=True),
            st.int64: lambda x: int(x).to_bytes(8, "little", signed=True),
            st.int128: lambda x: int(x).to_bytes(16, "little", signed=True),
            st.float32: lambda x: _not_implemented(),
            st.float64: lambda x: _not_implemented(),
            st.unit: lambda x: b"",
            st.char: lambda x: _not_implemented(),
            str: lambda x: _encode_str(encode_length, x),
            bytes: lambda x: _encode_bytes(encode_length, x),
        }


@dataclasses.dataclass(init=False)
class DeserializationConfig:
    """Configuration object for `deserialize_with_config`.

    "Binary" serialization formats may differ in the way they encode sequence lengths, variant
    index, and how they verify the ordering of keys in map entries (or not). These 3 elements
    are passed to the constructor and the rest is deduced.
    """

    primitive_decode_map: typing.Dict[
        typing.Any, typing.Callable[[bytes], typing.Tuple[bytes, typing.Any]]
    ]
    decode_length: DecodeInteger
    decode_variant_index: DecodeInteger
    check_that_key_slices_are_increasing: CheckThatKeySlicesAreIncreasing

    def __init__(
        self,
        decode_length: DecodeInteger,
        decode_variant_index: DecodeInteger,
        check_that_key_slices_are_increasing: CheckThatKeySlicesAreIncreasing,
    ):
        self.decode_length = decode_length
        self.decode_variant_index = decode_variant_index
        self.check_that_key_slices_are_increasing = check_that_key_slices_are_increasing
        self.primitive_decode_map = {
            st.bool: _decode_bool,
            st.uint8: lambda content: (
                st.uint8(int.from_bytes(content[:1], byteorder="little", signed=False)),
                content[1:],
            ),
            st.uint16: lambda content: (
                st.uint16(
                    int.from_bytes(content[:2], byteorder="little", signed=False)
                ),
                content[2:],
            ),
            st.uint32: lambda content: (
                st.uint32(
                    int.from_bytes(content[:4], byteorder="little", signed=False)
                ),
                content[4:],
            ),
            st.uint64: lambda content: (
                st.uint64(
                    int.from_bytes(content[:8], byteorder="little", signed=False)
                ),
                content[8:],
            ),
            st.uint128: lambda content: (
                st.uint128(
                    int.from_bytes(content[:16], byteorder="little", signed=False)
                ),
                content[16:],
            ),
            st.int8: lambda content: (
                st.int8(int.from_bytes(content[:1], byteorder="little", signed=True)),
                content[1:],
            ),
            st.int16: lambda content: (
                st.int16(int.from_bytes(content[:2], byteorder="little", signed=True)),
                content[2:],
            ),
            st.int32: lambda content: (
                st.int32(int.from_bytes(content[:4], byteorder="little", signed=True)),
                content[4:],
            ),
            st.int64: lambda content: (
                st.int64(int.from_bytes(content[:8], byteorder="little", signed=True)),
                content[8:],
            ),
            st.int128: lambda content: (
                st.int128(
                    int.from_bytes(content[:16], byteorder="little", signed=True)
                ),
                content[16:],
            ),
            st.float32: lambda content: _not_implemented(),
            st.float64: lambda content: _not_implemented(),
            st.unit: lambda content: (None, content),
            st.char: lambda content: _not_implemented(),
            str: lambda content: _decode_str(decode_length, content),
            bytes: lambda content: _decode_bytes(decode_length, content),
        }


def _encode_bytes(encode_length: EncodeInteger, value: bytes) -> bytes:
    return encode_length(len(value)) + value


def _encode_str(encode_length: EncodeInteger, value: str) -> bytes:
    return encode_length(len(value)) + value.encode()


def _decode_bool(content: bytes) -> typing.Tuple[st.bool, bytes]:
    b = int.from_bytes(content[:1], byteorder="little", signed=False)
    content = content[1:]
    if b == 0:
        val = False
    elif b == 1:
        val = True
    else:
        raise ValueError("Unexpected byte value for Bool:", b)
    return val, content


def _decode_bytes(
    decode_length: DecodeInteger, content: bytes
) -> typing.Tuple[bytes, bytes]:
    len, content = decode_length(content)
    val, content = content[:len], content[len:]
    return val, content


def _decode_str(
    decode_length: DecodeInteger, content: bytes
) -> typing.Tuple[str, bytes]:
    strlen, content = decode_length(content)
    val, content = content[0:strlen].decode(), content[strlen:]
    return val, content


def _not_implemented():
    raise NotImplementedError


# noqa: C901
def serialize_with_config(
    config: SerializationConfig, obj: typing.Any, obj_type
) -> bytes:
    result = b""

    if obj_type in config.primitive_encode_map:
        result += config.primitive_encode_map[obj_type](obj)

    elif hasattr(obj_type, "__origin__"):  # Generic type
        types = getattr(obj_type, "__args__")

        if getattr(obj_type, "__origin__") == collections.abc.Sequence:  # Sequence
            assert len(types) == 1
            item_type = types[0]
            result += config.encode_length(len(obj))
            result += b"".join(
                [serialize_with_config(config, item, item_type) for item in obj]
            )

        elif getattr(obj_type, "__origin__") == tuple:  # Tuple
            for i in range(len(obj)):
                result += serialize_with_config(config, obj[i], types[i])

        elif getattr(obj_type, "__origin__") == typing.Union:  # Option
            assert len(types) == 2 and types[1] == type(None)
            if obj is None:
                result += b"\x00"
            else:
                result += b"\x01"
                result += serialize_with_config(config, obj, types[0])

        elif getattr(obj_type, "__origin__") == dict:  # Map
            assert len(types) == 2
            item_type = typing.Tuple[types[0], types[1]]
            result += config.encode_length(len(obj))
            serialized_items = config.sort_map_entries(
                serialize_with_config(config, item, item_type) for item in obj.items()
            )
            for s in serialized_items:
                result += s

        else:
            raise ValueError("Unexpected type", obj_type)

    else:
        if not dataclasses.is_dataclass(obj_type):  # Enum
            if not hasattr(obj_type, "VARIANTS"):
                raise ValueError("Unexpected type", obj_type)
            if not hasattr(obj, "INDEX"):
                raise ValueError("Wrong Value for the type", obj, obj_type)
            result += config.encode_variant_index(obj.__class__.INDEX)
            # Proceed to variant
            obj_type = obj_type.VARIANTS[obj.__class__.INDEX]
            if not dataclasses.is_dataclass(obj_type):
                raise ValueError("Unexpected type", obj_type)

        # pyre-ignore
        if not isinstance(obj, obj_type):
            raise ValueError("Wrong Value for the type", obj, obj_type)

        # Content of struct or variant
        fields = dataclasses.fields(obj_type)
        types = get_type_hints(obj_type)
        for field in fields:
            field_type = types[field.name]
            field_value = obj.__dict__[field.name]
            result += serialize_with_config(config, field_value, field_type)

    return result


# noqa
def deserialize_with_config(
    config: DeserializationConfig, content: bytes, obj_type
) -> typing.Tuple[typing.Any, bytes]:
    if obj_type in config.primitive_decode_map:
        res, content = config.primitive_decode_map[obj_type](content)
        return res, content

    elif hasattr(obj_type, "__origin__"):  # Generic type
        types = getattr(obj_type, "__args__")
        if getattr(obj_type, "__origin__") == collections.abc.Sequence:  # Sequence
            assert len(types) == 1
            item_type = types[0]
            seqlen, content = config.decode_length(content)
            res = []
            for i in range(0, seqlen):
                item, content = deserialize_with_config(config, content, item_type)
                res.append(item)

            return res, content

        elif getattr(obj_type, "__origin__") == tuple:  # Tuple
            res = []
            for i in range(len(types)):
                item, content = deserialize_with_config(config, content, types[i])
                res.append(item)
            return tuple(res), content

        elif getattr(obj_type, "__origin__") == typing.Union:  # Option
            assert len(types) == 2 and types[1] == type(None)
            tag = int.from_bytes(content[:1], byteorder="little", signed=False)
            content = content[1:]
            if tag == 0:
                return None, content
            elif tag == 1:
                return deserialize_with_config(config, content, types[0])
            else:
                raise ValueError("Wrong tag for Option value")

        elif getattr(obj_type, "__origin__") == dict:  # Map
            assert len(types) == 2
            seqlen, content = config.decode_length(content)
            res = dict()
            previous_serialized_key = None
            for i in range(0, seqlen):
                previous_content = content
                key, content = deserialize_with_config(
                    config, previous_content, types[0]
                )
                serialized_key = previous_content[: -len(content)]
                value, content = deserialize_with_config(config, content, types[1])
                if previous_serialized_key is not None:
                    config.check_that_key_slices_are_increasing(
                        previous_serialized_key, serialized_key
                    )
                previous_serialized_key = serialized_key
                res[key] = value

            return res, content

        else:
            raise ValueError("Unexpected type", obj_type)

    else:
        # handle structs
        if dataclasses.is_dataclass(obj_type):
            values = []
            fields = dataclasses.fields(obj_type)
            typing_hints = get_type_hints(obj_type)
            for field in fields:
                field_type = typing_hints[field.name]
                field_value, content = deserialize_with_config(
                    config, content, field_type
                )
                values.append(field_value)

            res = obj_type(*values)
            return res, content

        # handle variant
        elif hasattr(obj_type, "VARIANTS"):
            variant_index, content = config.decode_variant_index(content)
            new_type = obj_type.VARIANTS[variant_index]
            res, content = deserialize_with_config(config, content, new_type)
            return res, content

        else:
            raise ValueError("Unexpected type", obj_type)
