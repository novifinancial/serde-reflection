# Copyright (c) Facebook, Inc. and its affiliates
# SPDX-License-Identifier: MIT OR Apache-2.0

# pyre-ignore-all-errors

import dataclasses
import collections

import serde_types as st
import typing
from typing import get_type_hints


def decode_uleb128_as_u32(content: bytes) -> typing.Tuple[st.uint32, bytes]:
    value = 0
    for shift in range(0, 32, 7):
        byte = int.from_bytes(content[0:1], "little", signed=False)
        content = content[1:]
        digit = byte & 0x7F
        value |= digit << shift
        if digit == byte:
            if shift > 0 and digit == 0:
                raise ValueError("Invalid uleb128 number (unexpected zero digit)")
            break

    if value >= 1 << 32:
        raise ValueError("Overflow while parsing uleb128-encoded uint32 value")
    return value, content


def decode_length(content: bytes) -> typing.Tuple[int, bytes]:
    value, content = decode_uleb128_as_u32(content)
    if value > 1 << 31:
        raise ValueError("Length exceeds the maximum supported value.")
    return value, content


def decode_variant_index(content: bytes) -> typing.Tuple[int, bytes]:
    return decode_uleb128_as_u32(content)


def decode_str(content: bytes) -> typing.Tuple[str, bytes]:
    strlen, content = decode_length(content)
    val, content = content[0:strlen].decode(), content[strlen:]
    return val, content


def decode_bytes(content: bytes) -> typing.Tuple[bytes, bytes]:
    len, content = decode_length(content)
    val, content = content[:len], content[len:]
    return val, content


def encode_u32_as_uleb128(value: int) -> bytes:
    res = b""
    while value >= 0x80:
        res += ((value & 0x7F) | 0x80).to_bytes(1, "little", signed=False)
        value >>= 7
    res += value.to_bytes(1, "little", signed=False)
    return res


def encode_length(value: int) -> bytes:
    return encode_u32_as_uleb128(value)


def encode_variant_index(value: int) -> bytes:
    return encode_u32_as_uleb128(value)


def encode_str(value: str) -> bytes:
    return encode_length(len(value)) + value.encode()


def encode_bytes(value: bytes) -> bytes:
    return encode_length(len(value)) + value


def not_implemented():
    raise NotImplementedError


primitive_encode_map = {
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
    st.float32: lambda x: not_implemented(),
    st.float64: lambda x: not_implemented(),
    st.unit: lambda x: b"",
    st.char: lambda x: not_implemented(),
    str: lambda x: encode_str(x),
    bytes: lambda x: encode_bytes(x),
}

primitive_decode_map = {
    st.bool: lambda content: (
        st.bool(int.from_bytes(content[:1], byteorder="little", signed=False)),
        content[1:],
    ),
    st.uint8: lambda content: (
        st.uint8(int.from_bytes(content[:1], byteorder="little", signed=False)),
        content[1:],
    ),
    st.uint16: lambda content: (
        st.uint16(int.from_bytes(content[:2], byteorder="little", signed=False)),
        content[2:],
    ),
    st.uint32: lambda content: (
        st.uint32(int.from_bytes(content[:4], byteorder="little", signed=False)),
        content[4:],
    ),
    st.uint64: lambda content: (
        st.uint64(int.from_bytes(content[:8], byteorder="little", signed=False)),
        content[8:],
    ),
    st.uint128: lambda content: (
        st.uint128(int.from_bytes(content[:16], byteorder="little", signed=False)),
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
        st.int128(int.from_bytes(content[:16], byteorder="little", signed=True)),
        content[16:],
    ),
    st.float32: lambda content: not_implemented(),
    st.float64: lambda content: not_implemented(),
    st.unit: lambda content: (None, content),
    st.char: lambda content: not_implemented(),
    str: lambda content: decode_str(content),
    bytes: lambda content: decode_bytes(content),
}

# noqa: C901
def serialize(obj: typing.Any, obj_type) -> bytes:
    result = b""

    if obj_type in primitive_encode_map:
        result += primitive_encode_map[obj_type](obj)

    elif hasattr(obj_type, "__origin__"):  # Generic type
        types = getattr(obj_type, "__args__")

        if getattr(obj_type, "__origin__") == collections.abc.Sequence:  # Sequence
            assert len(types) == 1
            item_type = types[0]
            result += encode_length(len(obj))
            result += b"".join([serialize(item, item_type) for item in obj])

        elif getattr(obj_type, "__origin__") == tuple:  # Tuple
            for i in range(len(obj)):
                result += serialize(obj[i], types[i])

        elif getattr(obj_type, "__origin__") == typing.Union:  # Option
            assert len(types) == 2 and types[1] == type(None)
            if obj is None:
                result += b"\x00"
            else:
                result += b"\x01"
                result += serialize(obj, types[0])

        elif getattr(obj_type, "__origin__") == dict:  # Map
            assert len(types) == 2
            item_type = typing.Tuple[types[0], types[1]]
            result += encode_length(len(obj))
            # Sorting by lexicographic order on the serialized item (or equivalently, serialized keys).
            serialized_items = sorted(
                serialize(item, item_type) for item in obj.items()
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
            result += encode_variant_index(obj.__class__.INDEX)
            # Proceed to variant
            obj_type = obj_type.VARIANTS[obj.__class__.INDEX]
            if not dataclasses.is_dataclass(obj_type):
                raise ValueError("Unexpected type", obj_type)

        if not isinstance(obj, obj_type):
            raise ValueError("Wrong Value for the type", obj, obj_type)

        # Content of struct or variant
        fields = dataclasses.fields(obj_type)
        types = get_type_hints(obj_type)
        for field in fields:
            field_type = types[field.name]
            field_value = obj.__dict__[field.name]
            result += serialize(field_value, field_type)

    return result


# noqa
def deserialize(content: bytes, obj_type):
    if obj_type in primitive_decode_map:
        res, content = primitive_decode_map[obj_type](content)
        return res, content

    elif hasattr(obj_type, "__origin__"):  # Generic type
        types = getattr(obj_type, "__args__")
        if getattr(obj_type, "__origin__") == collections.abc.Sequence:  # Sequence
            assert len(types) == 1
            item_type = types[0]
            seqlen, content = decode_length(content)
            res = []
            for i in range(0, seqlen):
                item, content = deserialize(content, item_type)
                res.append(item)

            return res, content

        elif getattr(obj_type, "__origin__") == tuple:  # Tuple
            res = []
            for i in range(len(types)):
                item, content = deserialize(content, types[i])
                res.append(item)
            return tuple(res), content

        elif getattr(obj_type, "__origin__") == typing.Union:  # Option
            assert len(types) == 2 and types[1] == type(None)
            tag = int.from_bytes(content[:1], byteorder="little", signed=False)
            content = content[1:]
            if tag == 0:
                return None, content
            elif tag == 1:
                return deserialize(content, types[0])
            else:
                raise ValueError("Wrong tag for Option value")

        elif getattr(obj_type, "__origin__") == dict:  # Map
            assert len(types) == 2
            seqlen, content = decode_length(content)
            res = dict()
            previous_serialized_key = None
            for i in range(0, seqlen):
                previous_content = content
                key, content = deserialize(previous_content, types[0])
                serialized_key = previous_content[: -len(content)]
                value, content = deserialize(content, types[1])
                if (
                    previous_serialized_key is not None
                    and previous_serialized_key >= serialized_key
                ):
                    raise ValueError(
                        "Serialized keys in a map must be ordered by increasing lexicographic order"
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
                field_value, content = deserialize(content, field_type)
                values.append(field_value)

            res = obj_type(*values)
            return res, content

        # handle variant
        elif hasattr(obj_type, "VARIANTS"):
            variant_index, content = decode_variant_index(content)
            new_type = obj_type.VARIANTS[variant_index]
            res, content = deserialize(content, new_type)
            return res, content

        else:
            raise ValueError("Unexpected type", obj_type)
