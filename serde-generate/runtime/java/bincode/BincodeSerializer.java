// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package bincode;

import java.lang.Exception;
import serde.BinarySerializer;

public class BincodeSerializer extends BinarySerializer {
    public void serialize_len(long value) throws Exception {
        serialize_u64(value);
    }

    public void serialize_variant_index(int value) throws Exception {
        serialize_u32(value);
    }

    public void sort_map_entries(int[] offsets) {
        // Not required by the format.
    }
}
