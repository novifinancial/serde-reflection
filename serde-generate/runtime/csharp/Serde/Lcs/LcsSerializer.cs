// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;
using System.IO;

namespace Serde.Lcs
{
    public class LcsSerializer: BinarySerializer {
        public const long MAX_LENGTH = int.MaxValue;
        public const long MAX_CONTAINER_DEPTH = 500;

        public LcsSerializer() : base(MAX_CONTAINER_DEPTH) {}

        private void serialize_u32_as_uleb128(uint value) {
            while ((value >> 7) != 0) {
                output.Write((byte)((value & 0x7f) | 0x80));
                value >>= 7;
            }
            output.Write((byte)value);
        }

        public override void serialize_len(long value) {
            if ((value < 0) || (value > MAX_LENGTH)) {
                throw new SerializationException("Incorrect Length value");
            }
            serialize_u32_as_uleb128((uint)value);
        }

        public override void serialize_variant_index(int value) => serialize_u32_as_uleb128((uint)value);

        public override void sort_map_entries(int[] offsets) {
            if (offsets.Length <= 1) {
                return;
            }
            int offset0 = offsets[0];
            Range[] ranges = new Range[offsets.Length];
            for (int i = 0; i < offsets.Length - 1; i++) {
                ranges[i] = new Range(offsets[i] - offset0, offsets[i + 1] - offset0);
            }
            ranges[^1] = new Range(offsets[^1] - offset0, (int)buffer.Length - offset0);

            byte[] data = new byte[buffer.Length - offset0];
            buffer.Seek(offset0, SeekOrigin.Begin);
            buffer.Read(data);

            Array.Sort(ranges, (l, r) => Verification.CompareLexicographic(data[l], data[r]));

            buffer.Seek(offset0, SeekOrigin.Begin);
            foreach (var range in ranges) {
                buffer.Write(data[range]);
            }
        }
    }
}
