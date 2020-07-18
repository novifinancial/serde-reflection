// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package bincode;

import java.lang.Exception;
import serde.Slice;
import serde.BinaryDeserializer;

public class BincodeDeserializer extends BinaryDeserializer {
    public BincodeDeserializer(byte[] input) {
        super(input);
    }

    public long deserialize_len() throws Exception {
        return input.getLong();
    }

    public int deserialize_variant_index() throws Exception {
        return input.getInt();
    }

    public void check_that_key_slices_are_increasing(Slice key1, Slice key2) throws Exception {
        // Not required by the format.
    }
}
