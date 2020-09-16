// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.novi.lcs;

import com.novi.serde.DeserializationError;
import com.novi.serde.Slice;
import com.novi.serde.BinaryDeserializer;

public class LcsDeserializer extends BinaryDeserializer {
    public LcsDeserializer(byte[] input) {
        super(input);
    }

    private int deserialize_uleb128_as_u32() throws DeserializationError {
        long value = 0;
        for (int shift = 0; shift < 32; shift += 7) {
            byte x = getByte();
            byte digit = (byte) (x & 0x7F);
            value = value | (digit << shift);
            if ((value < 0) || (value > Integer.MAX_VALUE)) {
                throw new DeserializationError("Overflow while parsing uleb128-encoded uint32 value");
            }
            if (digit == x) {
                if (shift > 0 && digit == 0) {
                    throw new DeserializationError("Invalid uleb128 number (unexpected zero digit)");
                }
                return (int) value;
            }
        }
        throw new DeserializationError("Overflow while parsing uleb128-encoded uint32 value");
    }

    public long deserialize_len() throws DeserializationError {
        return deserialize_uleb128_as_u32();
    }

    public int deserialize_variant_index() throws DeserializationError {
        return deserialize_uleb128_as_u32();
    }

    public void check_that_key_slices_are_increasing(Slice key1, Slice key2) throws DeserializationError {
        if (Slice.compare_bytes(input.array(), key1, key2) >= 0) {
            throw new DeserializationError("Error while decoding map: keys are not serialized in the expected order");
        }
    }
}
