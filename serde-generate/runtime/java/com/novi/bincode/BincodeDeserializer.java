// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.novi.bincode;

import com.novi.serde.DeserializationError;
import com.novi.serde.Slice;
import com.novi.serde.BinaryDeserializer;

public class BincodeDeserializer extends BinaryDeserializer {
    public BincodeDeserializer(byte[] input) {
        super(input);
    }

    public long deserialize_len() throws DeserializationError {
        long value = getLong();
        if (value < 0 || value > Integer.MAX_VALUE) {
            throw new DeserializationError("Incorrect length value");
        }
        return value;
    }

    public int deserialize_variant_index() throws DeserializationError {
        return getInt();
    }

    public void check_that_key_slices_are_increasing(Slice key1, Slice key2) throws DeserializationError {
        // Not required by the format.
    }
}
