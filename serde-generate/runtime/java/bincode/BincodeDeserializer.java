// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package bincode;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.lang.Exception;
import java.math.BigInteger;
import serde.Unsigned;
import serde.Int128;
import serde.Bytes;
import serde.Slice;

public class BincodeDeserializer implements serde.Deserializer {
    ByteBuffer input;

    public BincodeDeserializer(byte[] input) {
        this.input = ByteBuffer.wrap(input);
        this.input.order(ByteOrder.LITTLE_ENDIAN);
    }

    public String deserialize_str() throws Exception {
        Bytes value = deserialize_bytes();
        return new String(value.content);
    }

    public Bytes deserialize_bytes() throws Exception {
        long len = deserialize_len();
        if (len < 0 || len > Integer.MAX_VALUE) {
            throw new Exception("The length of a Java array cannot exceed MAXINT");
        }
        byte[] content = new byte[(int)len];
        input.get(content);
        return new Bytes(content);
    }

    public Boolean deserialize_bool() throws Exception {
        byte value = input.get();
        if (value == 0) {
            return Boolean.valueOf(false);
        }
        if (value == 1) {
            return Boolean.valueOf(true);
        }
        throw new Exception("Incorrect boolean value");
    }

    public Void deserialize_unit() throws Exception {
        return null;
    }

    public Character deserialize_char() throws Exception {
        throw new Exception("Not implemented: deserialize_char");
    }

    public Float deserialize_f32() throws Exception {
        throw new Exception("Not implemented: deserialize_f32");
    }

    public Double deserialize_f64() throws Exception {
        throw new Exception("Not implemented: deserialize_f64");
    }

    public @Unsigned Byte deserialize_u8() throws Exception {
        return Byte.valueOf(input.get());
    }

    public @Unsigned Short deserialize_u16() throws Exception {
        return Short.valueOf(input.getShort());
    }

    public @Unsigned Integer deserialize_u32() throws Exception {
        return Integer.valueOf(input.getInt());
    }

    public @Unsigned Long deserialize_u64() throws Exception {
        return Long.valueOf(input.getLong());
    }

    public @Unsigned @Int128 BigInteger deserialize_u128() throws Exception {
        BigInteger signed = deserialize_i128();
        if (signed.compareTo(BigInteger.ZERO) >= 0) {
            return signed;
        } else {
            return signed.add(BigInteger.ONE.shiftLeft(128));
        }
    }

    public Byte deserialize_i8() throws Exception {
        return Byte.valueOf(input.get());
    }

    public Short deserialize_i16() throws Exception {
        return Short.valueOf(input.getShort());
    }

    public Integer deserialize_i32() throws Exception {
        return Integer.valueOf(input.getInt());
    }

    public Long deserialize_i64() throws Exception {
        return Long.valueOf(input.getLong());
    }

    public @Int128 BigInteger deserialize_i128() throws Exception {
        byte[] content = new byte[16];
        input.get(content);
        byte[] reversed = new byte[16];
        for (int i = 0; i < 16; i++) {
            reversed[i] = content[15 - i];
        }
        return new BigInteger(reversed);
    }

    public long deserialize_len() throws Exception {
        return input.getLong();
    }

    public int deserialize_variant_index() throws Exception {
        return input.getInt();
    }

    public boolean deserialize_option_tag() throws Exception {
        return deserialize_bool().booleanValue();
    }

    public boolean enforce_strict_map_ordering() { return false; }

    public int get_buffer_offset() { return input.position(); }

    public void check_that_key_slices_are_increasing(Slice key1, Slice key2) throws Exception {}
}
