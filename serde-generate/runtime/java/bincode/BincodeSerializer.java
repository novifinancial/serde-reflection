// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package bincode;

import java.lang.Exception;
import java.math.BigInteger;
import java.io.ByteArrayOutputStream;

import serde.Unsigned;
import serde.Int128;
import serde.Bytes;
import serde.Unit;


public class BincodeSerializer implements serde.Serializer {
    private ByteArrayOutputStream output;

    public BincodeSerializer() {
        output = new ByteArrayOutputStream();
    }

    public void serialize_str(String value) throws Exception {
        serialize_bytes(new Bytes(value.getBytes()));
    }

    public void serialize_bytes(Bytes value) throws Exception {
        byte[] content = value.content();
        serialize_len(content.length);
        output.write(content, 0, content.length);
    }

    public void serialize_bool(Boolean value) throws Exception {
        output.write((value.booleanValue() ? 1 : 0));
    }

    public void serialize_unit(Unit value) throws Exception {
    }

    public void serialize_char(Character value) throws Exception {
        throw new Exception("Not implemented: serialize_char");
    }

    public void serialize_f32(Float value) throws Exception {
        throw new Exception("Not implemented: serialize_f32");
    }

    public void serialize_f64(Double value) throws Exception {
        throw new Exception("Not implemented: serialize_f64");
    }

    public void serialize_u8(@Unsigned Byte value) throws Exception {
        output.write(value.byteValue());
    }

    public void serialize_u16(@Unsigned Short value) throws Exception {
        short val = value.shortValue();
        output.write((byte) (val >>> 0));
        output.write((byte) (val >>> 8));
    }

    public void serialize_u32(@Unsigned Integer value) throws Exception {
        int val = value.intValue();
        output.write((byte) (val >>> 0));
        output.write((byte) (val >>> 8));
        output.write((byte) (val >>> 16));
        output.write((byte) (val >>> 24));
    }

    public void serialize_u64(@Unsigned Long value) throws Exception {
        long val = value.longValue();
        output.write((byte) (val >>> 0));
        output.write((byte) (val >>> 8));
        output.write((byte) (val >>> 16));
        output.write((byte) (val >>> 24));
        output.write((byte) (val >>> 32));
        output.write((byte) (val >>> 40));
        output.write((byte) (val >>> 48));
        output.write((byte) (val >>> 56));
    }

    public void serialize_u128(@Unsigned @Int128 BigInteger value) throws Exception {
        assert value.compareTo(BigInteger.ZERO) >= 0;
        assert value.shiftRight(128).equals(BigInteger.ZERO);
        byte[] content = value.toByteArray();
        // BigInteger.toByteArray() may add a 16th most-significant zero
        // byte for signing purpose: ignore it.
        assert content.length <= 16 || content[0] == 0;
        int len = Math.min(content.length, 16);
        // Write content in little-endian order.
        for (int i = 0; i < len; i++) {
            output.write(content[content.length - 1 - i]);
        }
        // Complete with zeros if needed.
        for (int i = len; i < 16; i++) {
            output.write(0);
        }
    }

    public void serialize_i8(Byte value) throws Exception {
        serialize_u8(value);
    }

    public void serialize_i16(Short value) throws Exception {
        serialize_u16(value);
    }

    public void serialize_i32(Integer value) throws Exception {
        serialize_u32(value);
    }

    public void serialize_i64(Long value) throws Exception {
        serialize_u64(value);
    }

    public void serialize_i128(@Int128 BigInteger value) throws Exception {
        if (value.compareTo(BigInteger.ZERO) >= 0) {
            serialize_u128(value);
        } else {
            serialize_u128(value.add(BigInteger.ONE.shiftLeft(128)));
        }
    }

    public void serialize_len(long value) throws Exception {
        serialize_u64(value);
    }

    public void serialize_variant_index(int value) throws Exception {
        serialize_u32(value);
    }

    public void serialize_option_tag(boolean value) throws Exception {
        output.write((value ? (byte) 1 : (byte) 0));
    }

    public int get_buffer_offset() {
        return output.size();
    }

    public void sort_map_entries(int[] offsets) {
        // Not required by the format.
    }

    public byte[] get_bytes() {
        return output.toByteArray();
    }
}
