// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package serde;

import java.io.IOException;
import java.math.BigInteger;
import java.nio.ByteBuffer;
import java.util.Optional;
import java.util.Vector;
import java.util.SortedMap;

public interface Serializer {
    void serialize_str(String value) throws IOException;
    void serialize_bytes(byte[] value) throws IOException;

    void serialize_bool(Boolean value) throws IOException;
    void serialize_unit(Void value) throws IOException;
    void serialize_char(Character value) throws IOException;
    void serialize_f32(Float value) throws IOException;
    void serialize_f64(Double value) throws IOException;

    void serialize_u8(@Unsigned Byte value) throws IOException;
    void serialize_u16(@Unsigned Short value) throws IOException;
    void serialize_u32(@Unsigned Integer value) throws IOException;
    void serialize_u64(@Unsigned Long value) throws IOException;
    void serialize_u128(@Unsigned @Int128 BigInteger value) throws IOException;

    void serialize_i8(Byte value) throws IOException;
    void serialize_i16(Short value) throws IOException;
    void serialize_i32(Integer value) throws IOException;
    void serialize_i64(Long value) throws IOException;
    void serialize_i128(@Int128 BigInteger value) throws IOException;

    void serialize_len(long value) throws IOException;
    void serialize_variant_index(int value) throws IOException;
    void serialize_option_tag(boolean value) throws IOException;

    ByteBuffer bytes();
}
