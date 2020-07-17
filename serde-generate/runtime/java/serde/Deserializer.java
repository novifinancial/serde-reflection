// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package serde;

import java.lang.Exception;
import java.math.BigInteger;

import serde.Bytes;
import serde.Slice;
import serde.Unit;


public interface Deserializer {
    String deserialize_str() throws Exception;

    Bytes deserialize_bytes() throws Exception;

    Boolean deserialize_bool() throws Exception;

    Unit deserialize_unit() throws Exception;

    Character deserialize_char() throws Exception;

    Float deserialize_f32() throws Exception;

    Double deserialize_f64() throws Exception;

    @Unsigned Byte deserialize_u8() throws Exception;

    @Unsigned Short deserialize_u16() throws Exception;

    @Unsigned Integer deserialize_u32() throws Exception;

    @Unsigned Long deserialize_u64() throws Exception;

    @Unsigned @Int128 BigInteger deserialize_u128() throws Exception;

    Byte deserialize_i8() throws Exception;

    Short deserialize_i16() throws Exception;

    Integer deserialize_i32() throws Exception;

    Long deserialize_i64() throws Exception;

    @Int128 BigInteger deserialize_i128() throws Exception;

    long deserialize_len() throws Exception;

    int deserialize_variant_index() throws Exception;

    boolean deserialize_option_tag() throws Exception;

    int get_buffer_offset();

    void check_that_key_slices_are_increasing(Slice key1, Slice key2) throws Exception;
}
