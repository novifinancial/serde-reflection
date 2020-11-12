// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;

namespace Serde.Bincode
{
    public class BincodeDeserializer : BinaryDeserializer {
        public BincodeDeserializer(byte[] input) : base(input, long.MaxValue) { }
        public BincodeDeserializer(ArraySegment<byte> input) : base(input, long.MaxValue) { }

        public override long deserialize_len() {
            long value = reader.ReadInt64();
            if (value < 0 || value > int.MaxValue) {
                throw new DeserializationException("Incorrect length value");
            }
            return value;
        }

        public override int deserialize_variant_index() => reader.ReadInt32();

        public override void check_that_key_slices_are_increasing(Range key1, Range key2) {
            // Not required by the format.
        }
    }
}
