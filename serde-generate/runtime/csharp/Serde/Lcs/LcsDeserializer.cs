// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;

namespace Serde.Lcs
{
    public class LcsDeserializer: BinaryDeserializer {
        public LcsDeserializer(byte[] input) : base(input, LcsSerializer.MAX_CONTAINER_DEPTH) { }
        public LcsDeserializer(ArraySegment<byte> input) : base(input, LcsSerializer.MAX_CONTAINER_DEPTH) { }

        private int deserialize_uleb128_as_u32() {
            long value = 0;
            for (int shift = 0; shift < 32; shift += 7) {
                byte x = reader.ReadByte();
                byte digit = (byte) (x & 0x7F);
                value |= ((long)digit << shift);
                if ((value < 0) || (value > int.MaxValue)) {
                    throw new DeserializationException("Overflow while parsing uleb128-encoded uint32 value");
                }
                if (digit == x) {
                    if (shift > 0 && digit == 0) {
                        throw new DeserializationException("Invalid uleb128 number (unexpected zero digit)");
                    }
                    return (int) value;
                }
            }
            throw new DeserializationException("Overflow while parsing uleb128-encoded uint32 value");
        }

        public override long deserialize_len() => deserialize_uleb128_as_u32();

        public override int deserialize_variant_index() => deserialize_uleb128_as_u32();

        public override void check_that_key_slices_are_increasing(Range key1, Range key2) {
            if (Verification.CompareLexicographic(input.Slice(key1), input.Slice(key2)) >= 0) {
                throw new DeserializationException("Error while decoding map: keys are not serialized in the expected order");
            }
        }
    }
}
