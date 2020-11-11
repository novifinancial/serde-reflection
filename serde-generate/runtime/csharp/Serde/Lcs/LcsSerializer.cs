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
                throw new SerializationException("length value doesn't fit in uint32");
            }
            serialize_u32_as_uleb128((uint)value);
        }

        public override void serialize_variant_index(int value) => serialize_u32_as_uleb128((uint)value);

        static ReadOnlySpan<byte> Slice(byte[] array, (int start, int end) tup) =>
            new ReadOnlySpan<byte>(array, tup.start, tup.end - tup.start);

        public override void sort_map_entries(int[] offsets) {
            if (offsets.Length <= 1) {
                return;
            }
            int offset0 = offsets[0];
            var ranges = new (int start, int end)[offsets.Length];
            for (int i = 0; i < offsets.Length - 1; i++) {
                ranges[i] = (offsets[i] - offset0, offsets[i + 1] - offset0);
            }
            ranges[ranges.Length - 1] = (offsets[offsets.Length - 1] - offset0, (int)buffer.Length - offset0);

            byte[] data = new byte[buffer.Length - offset0];
            buffer.Seek(offset0, SeekOrigin.Begin);
            buffer.Read(data, 0, data.Length);

            Array.Sort(ranges, (l, r) => Verification.CompareLexicographic(Slice(data, l), Slice(data, r)));

            buffer.Seek(offset0, SeekOrigin.Begin);
            foreach (var range in ranges) {
                buffer.Write(data, range.start, range.end - range.start);
            }
        }
    }
}
