// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package serde;

import java.io.IOException;
import java.math.BigInteger;
import java.nio.ByteBuffer;
import java.util.Optional;
import java.util.Vector;
import java.util.SortedMap;

public interface Deserializer {
    String deserialize_str() throws IOException;
    byte[] deserialize_bytes() throws IOException;

    Boolean deserialize_bool() throws IOException;
    Void deserialize_unit() throws IOException;
    Character deserialize_char() throws IOException;
    Float deserialize_f32() throws IOException;
    Double deserialize_f64() throws IOException;

    @Unsigned Byte deserialize_u8() throws IOException;
    @Unsigned Short deserialize_u16() throws IOException;
    @Unsigned Integer deserialize_u32() throws IOException;
    @Unsigned Long deserialize_u64() throws IOException;
    @Unsigned @Int128 BigInteger deserialize_u128() throws IOException;

    Byte deserialize_i8() throws IOException;
    Short deserialize_i16() throws IOException;
    Integer deserialize_i32() throws IOException;
    Long deserialize_i64() throws IOException;
    @Int128 BigInteger deserialize_i128() throws IOException;

    long deserialize_len() throws IOException;
    int deserialize_variant_index() throws IOException;
    boolean deserialize_option_tag() throws IOException;
}
