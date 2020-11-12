// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;

namespace Serde.Bincode {
    public class BincodeSerializer : BinarySerializer {
        public BincodeSerializer() : base(long.MaxValue) { }
        public BincodeSerializer(byte[] buffer) : base(buffer, long.MaxValue) {}
        public BincodeSerializer(ArraySegment<byte> buffer) : base(buffer, long.MaxValue) {}

        public override void serialize_len(long value) => output.Write(value);

        public override void serialize_variant_index(int value) => output.Write(value);

        public override void sort_map_entries(int[] offsets) {
            // Not required by the format.
        }
    }
}
