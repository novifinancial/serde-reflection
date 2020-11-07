using System;
using System.Numerics;
using NUnit.Framework;
using Serde.Lcs;

namespace Serde.Tests
{
    public class TestLcs
    {
        [Test]
        public void TestSerializeU128()
        {
            LcsSerializer serializer = new LcsSerializer();
            serializer.serialize_u128((BigInteger.One << 128) - 1);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 });

            serializer = new LcsSerializer();
            serializer.serialize_u128(BigInteger.One);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 });

            serializer = new LcsSerializer();
            serializer.serialize_u128(BigInteger.Zero);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 });

            Assert.Throws<ArgumentOutOfRangeException>(() => serializer.serialize_u128(BigInteger.MinusOne));

            Assert.Throws<ArgumentOutOfRangeException>(() => serializer.serialize_u128((BigInteger.One << 128) + 1));
        }

        [Test]
        public void TestSerializeI128()
        {
            LcsSerializer serializer = new LcsSerializer();
            serializer.serialize_i128(BigInteger.MinusOne);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 });

            serializer = new LcsSerializer();
            serializer.serialize_i128(BigInteger.One);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 });

            serializer = new LcsSerializer();
            serializer.serialize_i128((BigInteger.One << 127) - 1);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 127 });

            serializer = new LcsSerializer();
            serializer.serialize_i128(-(BigInteger.One << 127));
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80 });

            Assert.Throws<ArgumentOutOfRangeException>(() => serializer.serialize_i128(BigInteger.One << 127));

            Assert.Throws<ArgumentOutOfRangeException>(() => serializer.serialize_i128(-((BigInteger.One << 127) + 1)));
        }

        [Test]
        public void TestSliceOrdering()
        {
            LcsSerializer serializer = new LcsSerializer();
            serializer.serialize_u8(255);
            serializer.serialize_u32(1);
            serializer.serialize_u32(1);
            serializer.serialize_u32(2);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 255, /**/ 1, /**/ 0, 0, /**/ 0, 1, 0, /**/ 0, /**/ 0, /**/ 2, 0, 0, 0 });

            int[] offsets = { 1, 2, 4, 7, 8, 9 };
            serializer.sort_map_entries(offsets);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 255, /**/ 0, /**/ 0, /**/ 0, 0, /**/ 0, 1, 0,  /**/ 1, /**/ 2, 0, 0, 0 });
        }

        [Test]
        public void TestULEB128Encoding()
        {
            LcsSerializer serializer = new LcsSerializer();
            serializer.serialize_len(0);
            serializer.serialize_len(1);
            serializer.serialize_len(127);
            serializer.serialize_len(128);
            serializer.serialize_len(3000);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 0, 1, 127, 128, 1, 184, 23 });
        }
    }
}