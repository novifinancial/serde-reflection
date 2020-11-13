// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;
using System.Diagnostics;
using System.IO;
using System.Numerics;
using System.Text;

namespace Serde
{
    public abstract class BinarySerializer : ISerializer, IDisposable
    {
        protected readonly MemoryStream buffer;
        protected readonly BinaryWriter output;
        protected readonly Encoding utf8 = Encoding.GetEncoding("utf-8", new EncoderExceptionFallback(), new DecoderExceptionFallback());
        private long containerDepthBudget;

        public BinarySerializer(long maxContainerDepth)
        {
            buffer = new MemoryStream();
            output = new BinaryWriter(buffer);
            containerDepthBudget = maxContainerDepth;
        }

        public BinarySerializer(byte[] bufferArray, long maxContainerDepth) : this(new ArraySegment<byte>(bufferArray), maxContainerDepth) { }

        public BinarySerializer(ArraySegment<byte> bufferArray, long maxContainerDepth)
        {
            buffer = new MemoryStream(bufferArray.Array, bufferArray.Offset, bufferArray.Count);
            output = new BinaryWriter(buffer);
            containerDepthBudget = maxContainerDepth;
        }

        public void Dispose() => output.Dispose();

        public void increase_container_depth()
        {
            if (containerDepthBudget == 0)
            {
                throw new SerializationException("Exceeded maximum container depth");
            }
            containerDepthBudget -= 1;
        }

        public void decrease_container_depth()
        {
            containerDepthBudget += 1;
        }

        public abstract void serialize_len(long len);

        public abstract void serialize_variant_index(int value);

        public abstract void sort_map_entries(int[] offsets);

        public virtual void serialize_char(char value) => throw new NotImplementedException();

        public virtual void serialize_f32(float value) => throw new NotImplementedException();

        public virtual void serialize_f64(double value) => throw new NotImplementedException();

        public byte[] get_bytes() => buffer.ToArray();

        public void serialize_str(string value) => serialize_bytes(utf8.GetBytes(value));

        public void serialize_bytes(byte[] value)
        {
            serialize_len(value.Length);
            output.Write(value);
        }

        public void serialize_bool(bool value) => output.Write(value);

        public void serialize_unit(Unit value) { }

        public void serialize_u8(byte value) => output.Write(value);

        public void serialize_u16(ushort value) => output.Write(value);

        public void serialize_u32(uint value) => output.Write(value);

        public void serialize_u64(ulong value) => output.Write(value);

        public void serialize_u128(BigInteger value)
        {
            if (value >> 128 != 0)
            {
                throw new SerializationException("Invalid value for an unsigned int128");
            }
            byte[] content = value.ToByteArray();
            // BigInteger.ToByteArray() may add a 16th most-significant zero
            // byte for signing purpose: ignore it.
            Debug.Assert(content.Length <= 16 || content[0] == 0);
            int len = Math.Min(content.Length, 16);

            output.Write(content);

            // Complete with zeros if needed.
            for (int i = len; i < 16; i++)
            {
                output.Write((byte)0);
            }
        }

        public void serialize_i8(sbyte value) => output.Write(value);

        public void serialize_i16(short value) => output.Write(value);

        public void serialize_i32(int value) => output.Write(value);

        public void serialize_i64(long value) => output.Write(value);

        public void serialize_i128(BigInteger value)
        {
            if (value >= 0)
            {
                if (value >> 127 != 0)
                {
                    throw new SerializationException("Invalid value for a signed int128");
                }
                serialize_u128(value);
            }
            else
            {
                if ((-(value + 1)) >> 127 != 0)
                {
                    throw new SerializationException("Invalid value for a signed int128");
                }
                serialize_u128(value + (BigInteger.One << 128));
            }
        }

        public void serialize_option_tag(bool value) => output.Write(value);

        public int get_buffer_offset() => (int)output.BaseStream.Position;
    }
}
